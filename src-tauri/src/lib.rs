pub mod log_parser;
mod database;

use std::sync::Mutex;
use tauri::State;
use crate::database::DatabaseManager;
use crate::log_parser::{HtmlLogParser, LogEntry, Bookmark, TestSession};

// App state
struct AppState {
    db_manager: Mutex<DatabaseManager>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Parse log directory and create test session
#[tauri::command]
async fn parse_log_directory(
    state: State<'_, AppState>,
    directory_path: String,
) -> Result<String, String> {
    println!("Parsing log directory: {}", directory_path);

    // Generate session ID
    let session_id = format!("session_{}", chrono::Utc::now().timestamp());

    // Scan HTML files
    let html_files = HtmlLogParser::scan_html_files(&directory_path)
        .map_err(|e| format!("Failed to scan directory: {}", e))?;

    if html_files.is_empty() {
        return Err("No HTML log files found in the directory".to_string());
    }

    // Parse all files
    let mut all_entries = Vec::new();
    let mut total_entries = 0;

    for (index, file_path) in html_files.iter().enumerate() {
        println!("Processing file {}: {}", index + 1, file_path);
        match HtmlLogParser::parse_file(file_path, &session_id, index) {
            Ok(entries) => {
                total_entries += entries.len();
                all_entries.extend(entries);
            }
            Err(e) => {
                println!("Warning: Failed to parse {}: {}", file_path, e);
                // Continue with other files
            }
        }
    }

    if all_entries.is_empty() {
        return Err("No valid log entries found in the HTML files".to_string());
    }

    // Create test session
    let session = TestSession {
        id: session_id.clone(),
        name: std::path::Path::new(&directory_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Session")
            .to_string(),
        directory_path,
        file_count: html_files.len(),
        total_entries,
        created_at: Some(chrono::Utc::now()),
        last_parsed_at: Some(chrono::Utc::now()),
    };

    // Save to database
    let mut db_manager = state.db_manager.lock().unwrap();
    db_manager.create_test_session(&session)
        .map_err(|e| format!("Failed to create session: {}", e))?;

    db_manager.insert_entries(&all_entries)
        .map_err(|e| format!("Failed to insert entries: {}", e))?;

    println!("Successfully parsed {} files with {} total entries", html_files.len(), total_entries);

    Ok(session_id)
}

// Get paginated log entries
#[tauri::command]
fn get_log_entries(
    state: State<'_, AppState>,
    session_id: String,
    offset: usize,
    limit: usize,
    level_filter: Option<String>,
    search_term: Option<String>,
) -> Result<(Vec<LogEntry>, usize), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager.get_entries_paginated(
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
    db_manager.add_bookmark(&bookmark)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

// Get bookmarks for session
#[tauri::command]
fn get_bookmarks(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<(Bookmark, LogEntry)>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager.get_bookmarks(&session_id)
        .map_err(|e| format!("Failed to get bookmarks: {}", e))
}

// Get all test sessions
#[tauri::command]
fn get_sessions(state: State<'_, AppState>) -> Result<Vec<TestSession>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager.get_sessions()
        .map_err(|e| format!("Failed to get sessions: {}", e))
}

// Delete test session
#[tauri::command]
fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager.delete_session(&session_id)
        .map_err(|e| format!("Failed to delete session: {}", e))
}

// Delete bookmark
#[tauri::command]
fn delete_bookmark(state: State<'_, AppState>, bookmark_id: i64) -> Result<(), String> {
    let db_manager = state.db_manager.lock().unwrap();
    db_manager.delete_bookmark(bookmark_id)
        .map_err(|e| format!("Failed to delete bookmark: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db_path = "logterminator.db";
    let db_manager = DatabaseManager::new(db_path)
        .expect("Failed to initialize database");

    let app_state = AppState {
        db_manager: Mutex::new(db_manager),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            parse_log_directory,
            get_log_entries,
            add_bookmark,
            get_bookmarks,
            delete_bookmark,
            get_sessions,
            delete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
