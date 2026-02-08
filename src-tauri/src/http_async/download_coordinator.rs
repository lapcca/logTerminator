//! Session-level download coordination

use crate::database::DatabaseManager;
use crate::http_log_fetcher::HttpFetchError;
use crate::log_parser::HtmlLogParser;
use super::types::{ProgressStatus, FileStatus, FileDownloadStatus};
use super::async_fetcher::AsyncHttpLogFetcher;
use super::progress_tracker::SpeedCalculator;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::collections::HashMap;
use tokio::sync::Semaphore;
use chrono::Utc;

/// Download coordinator for managing concurrent sessions and files
pub struct SessionDownloadCoordinator {
    max_sessions: usize,
    max_files_per_session: usize,
    max_retries: u32,
}

impl SessionDownloadCoordinator {
    /// Create a new download coordinator
    pub fn new(max_sessions: usize, max_files_per_session: usize, max_retries: u32) -> Self {
        Self {
            max_sessions,
            max_files_per_session,
            max_retries,
        }
    }

    /// Download all sessions from HTTP URL
    pub async fn download_sessions(
        &self,
        db_path: String,
        url: String,
        selected_tests: Option<Vec<String>>,
        progress_callback: Arc<dyn Fn(ProgressStatus) + Send + Sync>,
    ) -> Result<Vec<String>, HttpFetchError> {
        progress_callback(ProgressStatus::Connecting);

        // Create fetcher
        let fetcher = Arc::new(AsyncHttpLogFetcher::new(&url).await?);

        // Fetch directory listing
        progress_callback(ProgressStatus::Scanning { found: 0 });
        let listing_html = fetcher.fetch_directory_listing().await?;
        let all_urls = fetcher.parse_directory_listing(&listing_html).await?;

        // Filter to test log files
        let test_log_urls = crate::http_log_fetcher::HttpLogFetcher::filter_test_log_files(&all_urls);

        if test_log_urls.is_empty() {
            progress_callback(ProgressStatus::Complete);
            return Ok(vec![]);
        }

        progress_callback(ProgressStatus::Scanning { found: test_log_urls.len() });

        // Group by test session
        let mut session_groups: HashMap<String, Vec<(String, usize)>> = HashMap::new();
        for (index, log_url) in test_log_urls.iter().enumerate() {
            if let Some(filename) = log_url.rsplit('/').next() {
                if let Some(test_name) = HtmlLogParser::is_test_log_file(filename) {
                    session_groups.entry(test_name).or_default().push((log_url.clone(), index));
                }
            }
        }

        // Filter by selected tests if provided
        let session_groups: HashMap<String, Vec<(String, usize)>> = if let Some(selected) = selected_tests {
            session_groups.into_iter()
                .filter(|(key, _)| selected.contains(key))
                .collect()
        } else {
            session_groups
        };

        if session_groups.is_empty() {
            progress_callback(ProgressStatus::Complete);
            return Ok(vec![]);
        }

        // Download sessions with concurrency limits
        let session_semaphore = Arc::new(Semaphore::new(self.max_sessions));
        let bytes_downloaded = Arc::new(AtomicU64::new(0));
        let speed_calculator = Arc::new(SpeedCalculator::new());

        let mut session_tasks = Vec::new();
        let session_count = session_groups.len();
        let current_session = Arc::new(AtomicUsize::new(0));

        for (session_name, log_files) in session_groups {
            let semaphore = session_semaphore.clone();
            let fetcher_clone = fetcher.clone();
            let progress_cb = progress_callback.clone();
            let bytes = bytes_downloaded.clone();
            let speed = speed_calculator.clone();
            let url_clone = url.clone();
            let db_path_clone = db_path.clone();
            let session_num = current_session.fetch_add(1, Ordering::SeqCst) + 1;
            let max_files = self.max_files_per_session;
            let max_retries = self.max_retries;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await
                    .map_err(|_| HttpFetchError::ParseError("Semaphore closed".to_string()))?;
                Self::download_single_session(
                    fetcher_clone,
                    session_name,
                    log_files,
                    url_clone,
                    db_path_clone,
                    progress_cb,
                    bytes,
                    speed,
                    session_num,
                    session_count,
                    max_files,
                    max_retries,
                ).await
            });

            session_tasks.push(task);
        }

        // Wait for all sessions to complete
        let results = futures::future::join_all(session_tasks).await;

        let mut session_ids = Vec::new();
        for result in results {
            match result {
                Ok(Ok(id)) => session_ids.push(id),
                Ok(Err(e)) => eprintln!("Session download failed: {}", e),
                Err(e) => eprintln!("Task join error: {}", e),
            }
        }

        progress_callback(ProgressStatus::Complete);
        Ok(session_ids)
    }

    /// Download a single session with parallel file downloads
    async fn download_single_session(
        fetcher: Arc<AsyncHttpLogFetcher>,
        session_name: String,
        log_files: Vec<(String, usize)>,
        url: String,
        db_path: String,
        progress_callback: Arc<dyn Fn(ProgressStatus) + Send + Sync>,
        bytes_downloaded: Arc<AtomicU64>,
        speed_calculator: Arc<SpeedCalculator>,
        session_num: usize,
        total_sessions: usize,
        max_files: usize,
        max_retries: u32,
    ) -> Result<String, HttpFetchError> {
        let file_semaphore = Arc::new(Semaphore::new(max_files));
        let mut file_tasks = Vec::new();

        // Track file status with HashMap for O(1) lookups
        let file_status = Arc::new(std::sync::Mutex::new(HashMap::new()));

        for (file_url, file_index) in log_files.iter() {
            let semaphore = file_semaphore.clone();
            let fetcher_clone = fetcher.clone();
            let status = file_status.clone();
            let bytes_clone = bytes_downloaded.clone();
            let speed_clone = speed_calculator.clone();
            let progress_cb_clone = progress_callback.clone();
            let file_url = file_url.clone();
            let log_files_len = log_files.len();
            // Store the original file_index to preserve it through the async operation
            let original_file_index = *file_index;

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await
                    .map_err(|_| HttpFetchError::ParseError("Semaphore closed".to_string()))?;

                // Update status to downloading
                {
                    let mut st = status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
                    st.insert(file_url.clone(), FileStatus {
                        file_url: file_url.clone(),
                        status: FileDownloadStatus::Downloading,
                        retry_count: 0,
                        error_message: None,
                    });
                }

                match fetcher_clone.fetch_file_with_retry(
                    &file_url,
                    max_retries,
                    bytes_clone,
                    speed_clone.clone(),
                ).await {
                    Ok(result) => {
                        // Update status to completed
                        {
                            let mut st = status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
                            if let Some(fs) = st.get_mut(&file_url) {
                                fs.status = FileDownloadStatus::Completed;
                            }
                            log::info!("[DL] Completed: {}", file_url);
                        }

                        // Emit progress after each file completes
                        let progress_vec = {
                            let st = status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
                            st.values().cloned().collect::<Vec<_>>()
                        };
                        progress_cb_clone(ProgressStatus::Downloading {
                            total_sessions,
                            current_session: session_num,
                            total_files: log_files_len,
                            completed_files: progress_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Completed)).count(),
                            failed_files: progress_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Failed)).count(),
                            speed: speed_clone.format_speed(),
                            files: progress_vec,
                        });

                        // Return the URL, content, AND the original file_index
                        Ok((file_url, result.content, original_file_index))
                    }
                    Err(e) => {
                        // Update status to failed
                        let mut st = status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
                        if let Some(fs) = st.get_mut(&file_url) {
                            fs.status = FileDownloadStatus::Failed;
                            fs.error_message = Some(e.to_string());
                        }
                        Err(e)
                    }
                }
            });

            file_tasks.push(task);
        }

        // Wait for all files and collect results
        log::info!("Waiting for {} file downloads to complete...", log_files.len());
        let results = futures::future::join_all(file_tasks).await;
        log::info!("All download tasks completed");

        let total_results = results.len();
        let mut downloaded_contents = Vec::new();
        let mut total_downloaded_bytes = 0u64;
        let mut failed_downloads = Vec::new();

        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok((url, content, original_file_index))) => {
                    log::info!("File {}/{} downloaded successfully: {} ({} chars, original_index={})",
                        index + 1, total_results, url, content.len(), original_file_index);
                    total_downloaded_bytes += content.len() as u64;

                    // Verify content is not empty
                    if content.is_empty() {
                        log::error!("WARNING: Downloaded content is EMPTY for {}", url);
                    } else {
                        log::debug!("Content preview (first 200 chars): {}...", &content[..content.len().min(200)]);
                    }

                    // Store with the original file_index to preserve correct ordering
                    downloaded_contents.push((url, content, original_file_index));
                }
                Ok(Err(e)) => {
                    let error_msg = format!("File download {}/{} failed: {}", index + 1, total_results, e);
                    log::error!("{}", error_msg);
                    eprintln!("{}", error_msg);
                    failed_downloads.push(error_msg);
                }
                Err(e) => {
                    let error_msg = format!("Task {}/{} join error: {}", index + 1, total_results, e);
                    log::error!("{}", error_msg);
                    eprintln!("{}", error_msg);
                    failed_downloads.push(error_msg);
                }
            }
        }

        // CRITICAL: If any files failed to download, return error BEFORE deleting old session
        // We require ALL files to succeed for complete log data
        if !failed_downloads.is_empty() {
            let error_summary = format!(
                "Failed to download {}/{} files for session {}. Complete log data requires all files. Errors:\n  {}",
                failed_downloads.len(),
                total_results,
                session_name,
                failed_downloads.join("\n  ")
            );
            log::error!("{}", error_summary);
            eprintln!("{}", error_summary);
            return Err(HttpFetchError::DownloadFailed {
                url: session_name.clone(),
                reason: error_summary,
            });
        }

        // Sort by original file_index to ensure correct processing order
        downloaded_contents.sort_by_key(|(_, _, index)| *index);
        log::info!("Sorted {} downloaded files by original file_index", downloaded_contents.len());

        log::info!("Total downloaded: ~{} MB across {} files",
            total_downloaded_bytes / 1024 / 1024, downloaded_contents.len());

        // Update progress - do this BEFORE any async calls that might hold locks
        {
            let status_map = file_status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
            let status_vec: Vec<FileStatus> = status_map.values().cloned().collect();
            drop(status_map); // Release lock before any async operations

            progress_callback(ProgressStatus::Downloading {
                total_sessions,
                current_session: session_num,
                total_files: status_vec.len(),
                completed_files: status_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Completed)).count(),
                failed_files: status_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Failed)).count(),
                speed: speed_calculator.format_speed(),
                files: status_vec,
            });
        }

        // Process and store in database
        progress_callback(ProgressStatus::Parsing { session: session_name.clone() });
        log::info!("[DB] Starting database operations for session {}", session_name);

        // Generate session ID (needed for parsing)
        let session_id = format!(
            "session_{}_{}",
            session_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        // CRITICAL: Parse BEFORE deleting old session to avoid data loss on parse errors
        log::info!("Starting to parse {} downloaded files for session {}", downloaded_contents.len(), session_name);

        // Clone data needed for spawn_blocking
        let downloaded_contents_for_parse = downloaded_contents.clone();
        let session_id_for_parse = session_id.clone();

        // IMPORTANT: Use spawn_blocking for CPU-intensive parsing
        let parse_results = tokio::task::spawn_blocking(move || {
            let downloaded_count = downloaded_contents_for_parse.len();
            let mut entries = Vec::new();
            let mut parse_errors = Vec::new();

            for (i, (file_url, html_content, file_index)) in downloaded_contents_for_parse.into_iter().enumerate() {
                log::info!("[Parse {}/{}] Starting parse: {} ({} chars, file_index={})",
                    i + 1, downloaded_count, file_url, html_content.len(), file_index);

                match HtmlLogParser::parse_html_string(&html_content, &file_url, &session_id_for_parse, file_index) {
                    Ok(parsed_entries) => {
                        log::info!("[Parse {}/{}] Completed: {} entries from {} (file_index={})",
                            i + 1, downloaded_count, parsed_entries.len(), file_url, file_index);
                        entries.extend(parsed_entries);
                    }
                    Err(e) => {
                        let error_msg = format!("[Parse {}/{}] FAILED for {}: {}", i + 1, downloaded_count, file_url, e);
                        log::error!("{}", error_msg);
                        eprintln!("{}", error_msg);
                        parse_errors.push(error_msg);
                    }
                }
            }

            // CRITICAL: If any files failed to parse, return error
            // We require ALL files to parse successfully for complete log data
            if !parse_errors.is_empty() {
                return Err(format!(
                    "Failed to parse {}/{} files. Complete log data requires all files to parse. Errors:\n  {}",
                    parse_errors.len(),
                    downloaded_count,
                    parse_errors.join("\n  ")
                ));
            }

            Ok(entries)
        }).await
        .map_err(|e| HttpFetchError::ParseError(format!("Parse task failed: {}", e)))?
        .map_err(|e| HttpFetchError::ParseError(format!("Parse failed: {}", e)))?;

        let all_entries = parse_results;
        log::info!("Total entries parsed for session {}: {} entries from {} files", session_name, all_entries.len(), downloaded_contents.len());

        // CRITICAL: Only delete old session AFTER parsing succeeds
        // Delete existing session if any
        // Create a new connection for deletion to avoid lock issues
        match DatabaseManager::new(&db_path) {
            Ok(delete_db_manager) => {
                match delete_db_manager.delete_session_by_name_and_path(&session_name, &url) {
                    Ok(Some(deleted_session_id)) => {
                        println!("Deleted existing session: {}", deleted_session_id);
                    }
                    Ok(None) => {
                        println!("No existing session to delete for {}", session_name);
                    }
                    Err(e) => {
                        println!("Warning: Failed to delete existing session: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Warning: Could not create database connection for deletion: {}", e);
            }
        }

        if all_entries.is_empty() {
            log::error!("ERROR: No valid log entries found for session {}! Downloaded {} files but parsed 0 entries.", session_name, downloaded_contents.len());
            println!("Warning: No valid log entries found for session {}", session_name);
            return Ok(session_id);
        }

        // Create test session
        let entry_count = all_entries.len();
        let test_session = crate::log_parser::TestSession {
            id: session_id.clone(),
            name: session_name.clone(),
            directory_path: url.clone(),
            file_count: log_files.len(),
            total_entries: entry_count,
            created_at: Some(Utc::now()),
            last_parsed_at: Some(Utc::now()),
            source_type: Some("http".to_string()),
        };

        // Clone test_session for spawn_blocking
        let test_session_clone = test_session.clone();

        log::info!("Creating session in database: {} with {} entries", session_id, entry_count);

        // Clone data needed for logging after spawn_blocking
        let session_name_for_log = session_name.clone();
        let log_files_count = log_files.len();

        // Move all_entries into spawn_blocking
        tokio::task::spawn_blocking(move || {
            let mut db_manager = match DatabaseManager::new(&db_path) {
                Ok(manager) => manager,
                Err(e) => {
                    log::error!("Failed to create database connection: {}", e);
                    return;
                }
            };

            if let Err(e) = db_manager.create_test_session(&test_session_clone) {
                log::error!("Failed to create session: {}", e);
                return;
            }

            log::info!("Session created, inserting {} entries...", all_entries.len());

            let inserted_ids = match db_manager.insert_entries(&all_entries) {
                Ok(ids) => {
                    log::info!("Successfully inserted {} entries", ids.len());
                    ids
                }
                Err(e) => {
                    log::error!("Failed to insert entries: {}", e);
                    return;
                }
            };

            // Assign IDs to entries for auto-bookmark detection
            // IMPORTANT: Modify entries in place to avoid cloning
            let mut entries_with_ids = all_entries;
            for (i, entry_id) in inserted_ids.iter().enumerate() {
                if i < entries_with_ids.len() {
                    entries_with_ids[i].id = Some(*entry_id);
                }
            }

            // Find and create auto-bookmarks
            use crate::bookmark_utils::{create_auto_bookmark, find_auto_bookmark_markers};
            let auto_markers = find_auto_bookmark_markers(&entries_with_ids);
            if !auto_markers.is_empty() {
                log::info!("Found {} auto-bookmark markers, creating bookmarks...", auto_markers.len());
                for (entry_id, title) in &auto_markers {
                    let bookmark = create_auto_bookmark(*entry_id, title.clone());
                    if let Err(e) = db_manager.add_bookmark(&bookmark) {
                        log::warn!("Failed to create auto-bookmark '{}': {}", title, e);
                    }
                }
            }

            log::info!("Database operations completed for session {}", session_name_for_log);
        }).await
        .map_err(|e| HttpFetchError::ParseError(format!("Database task failed: {}", e)))?;

        println!(
            "Completed session {}: {} files, {} entries",
            session_name,
            log_files_count,
            entry_count
        );

        Ok(session_id)
    }
}
