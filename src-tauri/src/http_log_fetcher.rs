use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Errors that can occur during HTTP log fetching
pub enum HttpFetchError {
    InvalidUrl(String),
    NetworkError(reqwest::Error),
    TimeoutError,
    DirectoryListingNotFound,
    InvalidDirectoryListingFormat,
    DownloadFailed { url: String, reason: String },
    ParseError(String),
}

impl std::fmt::Display for HttpFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpFetchError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            HttpFetchError::NetworkError(e) => write!(f, "Network error: {}", e),
            HttpFetchError::TimeoutError => write!(f, "Request timeout"),
            HttpFetchError::DirectoryListingNotFound => write!(f, "Directory listing not found"),
            HttpFetchError::InvalidDirectoryListingFormat => write!(f, "Invalid directory listing format"),
            HttpFetchError::DownloadFailed { url, reason } => {
                write!(f, "Failed to download {}: {}", url, reason)
            }
            HttpFetchError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::fmt::Debug for HttpFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for HttpFetchError {}

/// HTTP log fetcher for downloading logs from web servers
pub struct HttpLogFetcher {
    client: Client,
    base_url: Url,
}

impl HttpLogFetcher {
    /// Create a new HTTP log fetcher
    pub fn new(base_url: &str) -> Result<Self, HttpFetchError> {
        let mut url = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

        // Ensure the URL path ends with '/' for proper directory listing
        // If it doesn't end with '/', append it to avoid URL parsing issues
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        Ok(HttpLogFetcher {
            client,
            base_url: url,
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Parse directory listing HTML and extract all file URLs
    pub fn parse_directory_listing(html: &str, base_url: &str) -> Result<Vec<String>, HttpFetchError> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a[href]").unwrap();

        let mut base = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

        // Ensure the URL path ends with '/' for proper directory listing
        // If it doesn't end with '/', append it to ensure relative links are resolved correctly
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }

        // Get the base path for filtering - ensure it ends with / for proper prefix matching
        let base_path = base.path().trim_end_matches('/');
        let base_path_prefix = format!("{}{}", base_path, "/");

        let mut urls = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Skip parent directory links explicitly
                if href == "../" || href.starts_with("../") || href.starts_with("?") {
                    eprintln!("Debug: Skipping parent/query link: {}", href);
                    continue;
                }

                // Skip directory links (ending with /)
                if href.ends_with('/') {
                    eprintln!("Debug: Skipping directory link: {}", href);
                    continue;
                }

                // Resolve relative URLs against base
                match base.join(href) {
                    Ok(full_url) => {
                        let url_string = full_url.to_string();
                        let resolved_path = full_url.path();

                        // Check if resolved path is within the base path
                        // The resolved path must start with base_path_prefix
                        if resolved_path.starts_with(&base_path_prefix) {
                            urls.push(url_string);
                        } else {
                            eprintln!("Warning: Skipping URL outside target directory:");
                            eprintln!("  Base path: {}", base_path_prefix);
                            eprintln!("  Resolved: {} (path: {})", url_string, resolved_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not resolve URL '{}': {}", href, e);
                        continue;
                    }
                }
            }
        }

        Ok(urls)
    }

    /// Fetch a single log file's HTML content
    pub fn fetch_log_file(&self, file_url: &str) -> Result<String, HttpFetchError> {
        let response = self
            .client
            .get(file_url)
            .send()
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(HttpFetchError::DownloadFailed {
                url: file_url.to_string(),
                reason: format!("HTTP status: {}", response.status()),
            });
        }

        response
            .text()
            .map_err(|e| HttpFetchError::NetworkError(e))
    }

    /// Filter URLs to only include test log files matching the pattern
    pub fn filter_test_log_files(urls: &[String]) -> Vec<String> {
        urls.iter()
            .filter(|url| {
                if let Some(filename) = url.rsplit('/').next() {
                    crate::log_parser::HtmlLogParser::is_test_log_file(filename).is_some()
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}

/// Fetch all logs from HTTP server and return session IDs
pub fn fetch_logs_from_http(
    db_path: String,
    url: String,
    progress_callback: impl Fn(String),
) -> Result<Vec<String>, HttpFetchError> {
    use crate::database::DatabaseManager;
    use crate::log_parser::HtmlLogParser;
    use chrono::Utc;

    progress_callback("Connecting to server...".to_string());

    // Create fetcher
    let fetcher = HttpLogFetcher::new(&url)?;

    // Fetch directory listing
    progress_callback("Parsing directory listing...".to_string());
    let listing_html = fetcher.fetch_log_file(&fetcher.base_url().to_string())?;

    // Parse directory listing
    let all_urls = HttpLogFetcher::parse_directory_listing(&listing_html, fetcher.base_url().as_str())?;

    // Filter to test log files only
    let test_log_urls = HttpLogFetcher::filter_test_log_files(&all_urls);

    if test_log_urls.is_empty() {
        return Ok(vec![]);
    }

    progress_callback(format!("Found {} test log file(s)", test_log_urls.len()));

    // Group by test session (test_name + test_id)
    let mut session_groups: std::collections::HashMap<String, Vec<(String, usize)>> = std::collections::HashMap::new();
    for (index, log_url) in test_log_urls.iter().enumerate() {
        if let Some(filename) = log_url.rsplit('/').next() {
            if let Some(test_name) = HtmlLogParser::is_test_log_file(filename) {
                // Extract test_id from filename
                let test_id = if let Some(start) = filename.find("ID_") {
                    let rest = &filename[start..];
                    if let Some(end) = rest.find("---") {
                        rest[0..end].to_string()
                    } else {
                        "1".to_string()
                    }
                } else {
                    "1".to_string()
                };

                let key = format!("{}_{}", test_name, test_id);
                session_groups.entry(key).or_default().push((log_url.clone(), index));
            }
        }
    }

    // Initialize database
    let mut db_manager = DatabaseManager::new(&db_path)
        .map_err(|e| HttpFetchError::ParseError(format!("Failed to initialize database: {}", e)))?;

    let mut session_ids = Vec::new();

    // Download and process each session
    let counter = Arc::new(AtomicUsize::new(0));
    let total = session_groups.len();

    for (session_key, log_files) in session_groups {
        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
        progress_callback(format!("Processing session ({}/{})...", count, total));

        // Generate session ID
        let session_id = format!(
            "session_{}_{}",
            session_key.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );

        let mut all_entries = Vec::new();

        // Download each log file in the session
        for (file_index, (log_url, _)) in log_files.iter().enumerate() {
            progress_callback(format!("Downloading log files ({}/{})...", count, total));

            // Small delay for politeness
            std::thread::sleep(Duration::from_millis(50));

            let html_content = fetcher.fetch_log_file(log_url)?;

            // Parse using HTML string parser
            let entries = HtmlLogParser::parse_html_string(&html_content, log_url, &session_id, file_index)
                .map_err(|e| HttpFetchError::ParseError(format!("Failed to parse {}: {}", log_url, e)))?;

            all_entries.extend(entries);
        }

        if all_entries.is_empty() {
            println!("Warning: No valid log entries found for session {}", session_key);
            continue;
        }

        // Save the total entries count before moving all_entries
        let total_entries = all_entries.len();

        // Create test session
        let test_session = crate::log_parser::TestSession {
            id: session_id.clone(),
            name: session_key.clone(),
            directory_path: url.clone(),
            file_count: log_files.len(),
            total_entries,
            created_at: Some(Utc::now()),
            last_parsed_at: Some(Utc::now()),
            source_type: Some("http".to_string()),
        };

        db_manager
            .create_test_session(&test_session)
            .map_err(|e| HttpFetchError::ParseError(format!("Failed to create session: {}", e)))?;

        let inserted_ids = db_manager
            .insert_entries(&all_entries)
            .map_err(|e| HttpFetchError::ParseError(format!("Failed to insert entries: {}", e)))?;

        // Assign IDs to entries for auto-bookmark detection
        let mut entries_with_ids = all_entries;
        for (i, entry_id) in inserted_ids.iter().enumerate() {
            if i < entries_with_ids.len() {
                entries_with_ids[i].id = Some(*entry_id);
            }
        }

        // Find and create auto-bookmarks for ###MARKER### patterns
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

        session_ids.push(session_id);

        println!(
            "Completed session {}: {} files, {} entries, {} auto-bookmarks",
            session_key,
            log_files.len(),
            total_entries,
            auto_markers.len()
        );
    }

    progress_callback("Complete".to_string());

    Ok(session_ids)
}
