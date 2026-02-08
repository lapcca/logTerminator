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

        for (file_url, _file_index) in &log_files {
            let semaphore = file_semaphore.clone();
            let fetcher_clone = fetcher.clone();
            let status = file_status.clone();
            let bytes_clone = bytes_downloaded.clone();
            let speed_clone = speed_calculator.clone();
            let file_url = file_url.clone();

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
                    speed_clone,
                ).await {
                    Ok(result) => {
                        // Update status to completed
                        let mut st = status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
                        if let Some(fs) = st.get_mut(&file_url) {
                            fs.status = FileDownloadStatus::Completed;
                        }
                        Ok((file_url, result.content))
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
        let results = futures::future::join_all(file_tasks).await;
        let mut downloaded_contents = Vec::new();

        for result in results {
            match result {
                Ok(Ok((url, content))) => downloaded_contents.push((url, content)),
                Ok(Err(e)) => eprintln!("File download failed: {}", e),
                Err(e) => eprintln!("Task join error: {}", e),
            }
        }

        // Update progress
        let status_map = file_status.lock().map_err(|e| HttpFetchError::ParseError(format!("Mutex error: {}", e)))?;
        let status_vec: Vec<FileStatus> = status_map.values().cloned().collect();
        drop(status_map); // Release lock before callback

        progress_callback(ProgressStatus::Downloading {
            total_sessions,
            current_session: session_num,
            total_files: status_vec.len(),
            completed_files: status_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Completed)).count(),
            failed_files: status_vec.iter().filter(|f| matches!(f.status, FileDownloadStatus::Failed)).count(),
            speed: speed_calculator.format_speed(),
            files: status_vec,
        });

        // Process and store in database
        progress_callback(ProgressStatus::Parsing { session: session_name.clone() });

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

        // Generate session ID
        let session_id = format!(
            "session_{}_{}",
            session_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        // Parse all downloaded files
        let mut all_entries = Vec::new();
        for (file_url, html_content) in downloaded_contents {
            let file_index = 0; // We don't track the exact index here
            let entries = HtmlLogParser::parse_html_string(&html_content, &file_url, &session_id, file_index)
                .map_err(|e| HttpFetchError::ParseError(format!("Failed to parse {}: {}", file_url, e)))?;
            all_entries.extend(entries);
        }

        if all_entries.is_empty() {
            println!("Warning: No valid log entries found for session {}", session_name);
            return Ok(session_id);
        }

        // Create a new database connection for this session
        let mut db_manager = DatabaseManager::new(&db_path)
            .map_err(|e| HttpFetchError::ParseError(format!("Failed to create database connection: {}", e)))?;

        // Create test session
        let test_session = crate::log_parser::TestSession {
            id: session_id.clone(),
            name: session_name.clone(),
            directory_path: url.clone(),
            file_count: log_files.len(),
            total_entries: all_entries.len(),
            created_at: Some(Utc::now()),
            last_parsed_at: Some(Utc::now()),
            source_type: Some("http".to_string()),
        };

        db_manager.create_test_session(&test_session)
            .map_err(|e| HttpFetchError::ParseError(format!("Failed to create session: {}", e)))?;

        let inserted_ids = db_manager.insert_entries(&all_entries)
            .map_err(|e| HttpFetchError::ParseError(format!("Failed to insert entries: {}", e)))?;

        // Assign IDs to entries for auto-bookmark detection
        let mut entries_with_ids = all_entries.clone();
        for (i, entry_id) in inserted_ids.iter().enumerate() {
            if i < entries_with_ids.len() {
                entries_with_ids[i].id = Some(*entry_id);
            }
        }

        // Find and create auto-bookmarks
        use crate::bookmark_utils::{create_auto_bookmark, find_auto_bookmark_markers};
        let auto_markers = find_auto_bookmark_markers(&entries_with_ids);
        if !auto_markers.is_empty() {
            println!("Found {} auto-bookmark markers", auto_markers.len());
            for (entry_id, title) in &auto_markers {
                let bookmark = create_auto_bookmark(*entry_id, title.clone());
                match db_manager.add_bookmark(&bookmark) {
                    Ok(_) => {
                        println!("Created auto-bookmark: '{}'", title);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to create auto-bookmark '{}': {}", title, e);
                    }
                }
            }
        }

        println!(
            "Completed session {}: {} files, {} entries, {} auto-bookmarks",
            session_name,
            log_files.len(),
            all_entries.len(),
            auto_markers.len()
        );

        Ok(session_id)
    }
}
