pub mod bookmark_utils;
mod database;
mod history;
pub mod http_log_fetcher;
pub mod http_async;
pub mod log_parser;

use crate::bookmark_utils::{create_auto_bookmark, find_auto_bookmark_markers};
use crate::database::DatabaseManager;
use crate::log_parser::{Bookmark, HtmlLogParser, LogEntry, ScanResult, TestSession};
use std::sync::Mutex;
use tauri::{Emitter, State};

// App state
struct AppState {
    db_manager: Mutex<DatabaseManager>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Scan directory for test sessions without loading them
#[tauri::command]
fn scan_log_directory(
    state: State<'_, AppState>,
    directory_path: String,
) -> Result<Vec<ScanResult>, String> {
    log::info!("Scanning directory: {}", directory_path);

    // Get existing sessions to check which tests are already loaded
    let existing_sessions = {
        let db_manager = state.db_manager.lock().unwrap();
        db_manager.get_sessions()
            .map_err(|e| format!("Failed to get existing sessions: {}", e))?
    };

    // Create a map of existing session names to their info
    let mut existing_map: std::collections::HashMap<String, (String, usize)> = std::collections::HashMap::new();
    for session in &existing_sessions {
        if session.directory_path == directory_path {
            existing_map.insert(session.name.clone(), (session.id.clone(), session.total_entries));
        }
    }

    // Scan directory for test groups
    let test_groups = HtmlLogParser::scan_html_files(&directory_path)
        .map_err(|e| format!("Failed to scan directory: {}", e))?;

    if test_groups.is_empty() {
        return Err("No test log files found in the directory".to_string());
    }

    // Create scan results
    let mut results = Vec::new();
    for (test_name, html_files) in test_groups {
        let (existing_session_id, estimated_entries) = existing_map
            .get(&test_name)
            .map(|(id, entries)| (Some(id.clone()), Some(*entries)))
            .unwrap_or((None, None));

        results.push(ScanResult {
            test_name,
            file_count: html_files.len(),
            is_loaded: existing_session_id.is_some(),
            existing_session_id,
            estimated_entries,
        });
    }

    println!("Scanned {} test sessions", results.len());
    Ok(results)
}

// Helper function to do the actual parsing work (can run in blocking thread pool)
fn parse_directory_blocking(
    db_path: String,
    directory_path: String,
    selected_tests: Option<Vec<String>>,
) -> Result<Vec<(String, String, usize, usize)>, String> {
    println!("[BLOCKING] Starting to parse: {}", directory_path);

    let mut db_manager = DatabaseManager::new(&db_path)
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    let test_groups = HtmlLogParser::scan_html_files(&directory_path)
        .map_err(|e| format!("Failed to scan directory: {}", e))?;

    if test_groups.is_empty() {
        return Err("No test log files found in the directory".to_string());
    }

    println!("[BLOCKING] Found {} test groups", test_groups.len());

    let mut session_results = Vec::new();

    // Filter tests if selected_tests is provided
    let test_groups_to_parse: Vec<(String, Vec<String>)> = if let Some(selected) = selected_tests {
        println!("[BLOCKING] Selected tests: {:?}", selected);
        if selected.is_empty() {
            return Err("No tests selected for loading".to_string());
        }
        let filtered: Vec<_> = test_groups
            .into_iter()
            .filter(|(name, _)| selected.contains(name))
            .collect();
        println!("[BLOCKING] Filtered to {} test groups", filtered.len());
        filtered
    } else {
        println!("[BLOCKING] No selection filter, processing all tests");
        test_groups.into_iter().collect()
    };

    if test_groups_to_parse.is_empty() {
        return Err("No matching test sessions found for the selected tests".to_string());
    }

    for (test_name, html_files) in test_groups_to_parse {
        println!(
            "[BLOCKING] Processing test group: {} ({} files)",
            test_name,
            html_files.len()
        );

        // Delete existing session with the same name and directory path if it exists
        println!("[BLOCKING] Checking for existing session: name={}, dir={}", test_name, directory_path);

        // Use a separate database connection for deletion to avoid borrow issues
        let db_path_for_delete = "logterminator.db";
        match DatabaseManager::new(db_path_for_delete) {
            Ok(delete_db_manager) => {
                match delete_db_manager.delete_session_by_name_and_path(&test_name, &directory_path) {
                    Ok(Some(deleted_session_id)) => {
                        println!("[BLOCKING] Deleted existing session: {}", deleted_session_id);
                    }
                    Ok(None) => {
                        println!("[BLOCKING] No existing session found for {}", test_name);
                    }
                    Err(e) => {
                        println!("[BLOCKING] Warning: Failed to delete existing session: {}", e);
                        // Continue anyway - try to create the new session
                    }
                }
            }
            Err(e) => {
                println!("[BLOCKING] Warning: Could not create database connection for deletion: {}", e);
                // Continue anyway
            }
        }

        let session_id = format!(
            "session_{}_{}",
            test_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_"),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        println!("[BLOCKING] Generated new session_id: {}", session_id);

        let mut all_entries = Vec::new();
        let mut total_entries = 0;

        for (index, file_path) in html_files.iter().enumerate() {
            match HtmlLogParser::parse_file(file_path, &session_id, index) {
                Ok(entries) => {
                    total_entries += entries.len();
                    all_entries.extend(entries);
                }
                Err(e) => {
                    println!("Warning: Failed to parse {}: {}", file_path, e);
                }
            }
        }

        if all_entries.is_empty() {
            println!("Warning: No valid log entries found for test {}", test_name);
            continue;
        }

        println!("[BLOCKING] Creating session: id={}, name={}", session_id, test_name);
        let session = TestSession {
            id: session_id.clone(),
            name: test_name.clone(),
            directory_path: directory_path.clone(),
            file_count: html_files.len(),
            total_entries,
            created_at: Some(chrono::Utc::now()),
            last_parsed_at: Some(chrono::Utc::now()),
            source_type: Some("local".to_string()),
        };

        db_manager
            .create_test_session(&session)
            .map_err(|e| format!("Failed to create session: {}", e))?;
        println!("[BLOCKING] Session created successfully in database");

        let inserted_ids = db_manager
            .insert_entries(&all_entries)
            .map_err(|e| format!("Failed to insert entries: {}", e))?;
        println!("[BLOCKING] Inserted {} entries into database", inserted_ids.len());

        // Assign IDs to entries for auto-bookmark detection
        let mut entries_with_ids = all_entries;
        for (i, entry_id) in inserted_ids.iter().enumerate() {
            if i < entries_with_ids.len() {
                entries_with_ids[i].id = Some(*entry_id);
            }
        }

        // Find and create auto-bookmarks for ###MARKER### patterns
        let auto_markers = find_auto_bookmark_markers(&entries_with_ids);
        if !auto_markers.is_empty() {
            println!("[BLOCKING] Found {} auto-bookmark markers", auto_markers.len());
            for (entry_id, title) in &auto_markers {
                let bookmark = create_auto_bookmark(*entry_id, title.clone());
                match db_manager.add_bookmark(&bookmark) {
                    Ok(_) => {
                                        log::info!("[BLOCKING] Created auto-bookmark: '{}'", title);
                    }
                    Err(e) => {
                        log::warn!("Failed to create auto-bookmark '{}': {}", title, e);
                    }
                }
            }
        }

        println!(
            "[BLOCKING] Completed test {}: {} files, {} entries, {} auto-bookmarks",
            test_name,
            html_files.len(),
            total_entries,
            auto_markers.len()
        );

        session_results.push((session_id, test_name, html_files.len(), total_entries));
    }

    println!("[BLOCKING] Total session results: {}", session_results.len());
    for (id, name, files, entries) in &session_results {
        println!("[BLOCKING] - Session: id={}, name={}, files={}, entries={}", id, name, files, entries);
    }

    if session_results.is_empty() {
        return Err("No valid test sessions were created".to_string());
    }

    Ok(session_results)
}

// Parse log directory and create test sessions
#[tauri::command]
async fn parse_log_directory(
    _state: State<'_, AppState>,
    directory_path: String,
    selected_tests: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    println!("Starting async parse for: {}", directory_path);
    println!("selected_tests: {:?}", selected_tests);

    let db_path = "logterminator.db".to_string();

    println!("Spawning blocking task...");

    // Run blocking work in thread pool
    let result =
        tokio::task::spawn_blocking(move || parse_directory_blocking(db_path, directory_path, selected_tests))
            .await
            .map_err(|e| format!("Task failed: {:?}", e))?;

    println!("Blocking task completed successfully");

    result.map(|sessions| sessions.into_iter().map(|(id, _, _, _)| id).collect())
}

// Scan HTTP URL for test sessions without loading them
#[tauri::command]
async fn scan_log_http_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<Vec<ScanResult>, String> {
    println!("Scanning HTTP URL: {}", url);

    // Get existing sessions to check which tests are already loaded
    let existing_sessions = {
        let db_manager = state.db_manager.lock().unwrap();
        db_manager.get_sessions()
            .map_err(|e| format!("Failed to get existing sessions: {}", e))?
    };

    // Use spawn_blocking to avoid blocking async runtime
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let result = crate::http_log_fetcher::scan_http_url(url, &existing_sessions);
        let _ = tx.send(result);
    });

    rx.await
        .map_err(|e| format!("Join error: {}", e))?
        .map_err(|e| e.to_string())
}

// Parse logs from HTTP server and create test sessions
#[tauri::command]
async fn parse_log_http_url(
    _state: State<'_, AppState>,
    window: tauri::Window,
    url: String,
    selected_tests: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    println!("Starting async HTTP parse for: {}", url);

    let db_path = "logterminator.db".to_string();

    // Use spawn_blocking to avoid blocking async runtime
    let (tx, rx) = tokio::sync::oneshot::channel();

    let url_clone = url.clone();
    let window_clone = window.clone();

    std::thread::spawn(move || {
        let result = crate::http_log_fetcher::fetch_logs_from_http(
            db_path,
            url_clone,
            |msg| {
                let _ = window_clone.emit("http-progress", msg);
            },
            selected_tests,
        );
        let _ = tx.send(result);
    });

    rx.await
        .map_err(|e| format!("Join error: {}", e))?
        .map_err(|e| e.to_string())
}

// Parse logs from HTTP server using async parallel downloads
#[tauri::command]
async fn parse_log_http_url_async(
    _state: State<'_, AppState>,
    window: tauri::Window,
    url: String,
    selected_tests: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    use crate::http_async::{SessionDownloadCoordinator, ProgressStatus};
    use std::sync::Arc;

    println!("[ASYNC] Starting async parallel HTTP parse for: {}", url);
    println!("[ASYNC] selected_tests: {:?}", selected_tests);

    let db_path = "logterminator.db".to_string();

    // Create progress callback that emits to frontend
    let window_clone = window.clone();
    let progress_callback = Arc::new(move |status: ProgressStatus| {
        let msg = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
        let _ = window_clone.emit("http-progress", msg);
    });

    // Create coordinator with configured limits
    let coordinator = SessionDownloadCoordinator::new(2, 4, 2);

    // Run the download
    coordinator.download_sessions(
        db_path,
        url,
        selected_tests,
        progress_callback,
    ).await
    .map_err(|e| e.to_string())
}

// Get paginated log entries
#[tauri::command]
fn get_log_entries(
    state: State<'_, AppState>,
    session_id: String,
    offset: usize,
    limit: usize,
    level_filter: Option<Vec<String>>, // Changed to Vec for multi-select
    search_term: Option<String>,
) -> Result<(Vec<LogEntry>, usize), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .get_entries_paginated(
            &session_id,
            offset,
            limit,
            level_filter.as_deref(),
            search_term.as_deref(),
        )
        .map_err(|e| format!("Database query error: {}", e))
}

// Add bookmark
#[tauri::command]
fn add_bookmark(
    state: State<'_, AppState>,
    log_entry_id: i64,
    title: Option<String>,
    notes: Option<String>,
    color: Option<String>,
) -> Result<i64, String> {
    let bookmark = Bookmark {
        id: None,
        log_entry_id,
        title,
        notes,
        color: color.or(Some("yellow".to_string())),
        created_at: Some(chrono::Utc::now()),
    };

    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .add_bookmark(&bookmark)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

// Get bookmarks for session
#[tauri::command]
fn get_bookmarks(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<(Bookmark, LogEntry)>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .get_bookmarks(&session_id)
        .map_err(|e| format!("Failed to get bookmarks: {}", e))
}

// Get all test sessions
#[tauri::command]
fn get_sessions(state: State<'_, AppState>) -> Result<Vec<TestSession>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .get_sessions()
        .map_err(|e| format!("Failed to get sessions: {}", e))
}

// Get all unique log levels for a session
#[tauri::command]
fn get_session_log_levels(state: State<'_, AppState>, session_id: String) -> Result<Vec<String>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .get_session_log_levels(&session_id)
        .map_err(|e| format!("Failed to get session log levels: {}", e))
}

// Ensure auto-bookmarks are created for a session (called when switching sessions)
#[tauri::command]
fn ensure_auto_bookmarks(state: State<'_, AppState>, session_id: String) -> Result<Vec<Bookmark>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .ensure_auto_bookmarks_for_session(&session_id)
        .map_err(|e| format!("Failed to ensure auto-bookmarks: {}", e))
}

// Delete test session
#[tauri::command]
fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .delete_session(&session_id)
        .map_err(|e| format!("Failed to delete session: {}", e))
}

// Delete bookmark
#[tauri::command]
fn delete_bookmark(state: State<'_, AppState>, bookmark_id: i64) -> Result<(), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .delete_bookmark(bookmark_id)
        .map_err(|e| format!("Failed to delete bookmark: {}", e))
}

// Update bookmark title
#[tauri::command]
async fn update_bookmark_title(
    state: State<'_, AppState>,
    bookmark_id: i64,
    title: &str,
) -> Result<(), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .update_bookmark_title(bookmark_id, title)
        .map_err(|e| format!("Failed to update bookmark title: {}", e))
}

// Get the page number for a specific log entry (for bookmark jumping)
#[tauri::command]
fn get_entry_page(
    state: State<'_, AppState>,
    entry_id: i64,
    items_per_page: usize,
    level_filter: Option<Vec<String>>, // Changed to Vec for multi-select
    search_term: Option<String>,
) -> Result<Option<usize>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager
        .get_entry_page(
            entry_id,
            items_per_page,
            level_filter.as_deref(),
            search_term.as_deref(),
        )
        .map_err(|e| format!("Failed to get entry page: {}", e))
}

/// Save the last used log directory
#[tauri::command]
fn save_last_directory(directory: String) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;
    use std::env;

    // Get the app data directory
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Cannot determine executable directory")?;

    let config_file = exe_dir.join("last_log_directory.txt");

    let mut file = File::create(&config_file)
        .map_err(|e| format!("Failed to create config file: {}", e))?;

    file.write_all(directory.as_bytes())
        .map_err(|e| format!("Failed to write directory: {}", e))?;

    log::info!("Saved last directory: {}", directory);
    Ok(())
}

/// Get the last used log directory
#[tauri::command]
fn get_last_directory() -> Result<String, String> {
    use std::fs;
    use std::env;

    // Get the app data directory
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Cannot determine executable directory")?;

    let config_file = exe_dir.join("last_log_directory.txt");

    match fs::read_to_string(&config_file) {
        Ok(content) => {
            let directory = content.trim().to_string();
            if !directory.is_empty() {
                log::info!("Retrieved last directory: {}", directory);
                Ok(directory)
            } else {
                Ok(String::new())
            }
        }
        Err(_) => Ok(String::new()), // File doesn't exist or can't be read, return empty string
    }
}

/// Get log source history
#[tauri::command]
fn get_log_history() -> Vec<String> {
    history::get_recent_history(10)
}

/// Save a log source history entry
#[tauri::command]
fn save_log_history_entry(entry: String) -> Result<(), String> {
    history::save_history(entry)
}

/// Initialize logging to both file and console
fn init_logging() -> std::io::Result<()> {
    use std::fs::OpenOptions;

    // Get the directory containing the executable
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cannot determine executable directory",
        ))?;

    // Create/open log file in the same directory
    let log_file = exe_dir.join("logterminator.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    // Initialize env_logger to write to BOTH file and console
    // Use Debug level for our code, Info for dependencies (to reduce noise)
    env_logger::Builder::new()
        .format_timestamp_millis()
        .filter_level(log::LevelFilter::Info) // Default to Info for dependencies
        .filter_module("logterminator", log::LevelFilter::Debug) // Debug for our app
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    // Log initialization
    println!("Logging initialized. Log file: {}", log_file.display());
    println!("Log level: Debug (app) / Info (dependencies)");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging to file
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }

    // Initialize database
    log::info!("Starting logTerminator application");
    let db_path = "logterminator.db";
    let db_manager = DatabaseManager::new(db_path).expect("Failed to initialize database");

    let app_state = AppState {
        db_manager: Mutex::new(db_manager),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_log_directory,
            scan_log_http_url,
            parse_log_directory,
            parse_log_http_url,
            parse_log_http_url_async,
            get_log_entries,
            add_bookmark,
            get_bookmarks,
            delete_bookmark,
            update_bookmark_title,
            get_entry_page,
            get_sessions,
            get_session_log_levels,
            ensure_auto_bookmarks,
            delete_session,
            save_last_directory,
            get_last_directory,
            get_log_history,
            save_log_history_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
