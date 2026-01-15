use logterminator_lib::log_parser::{HtmlLogParser, LogEntry};
use logterminator_lib::database::DatabaseManager;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing logTerminator functionality...");

    // Test HTML parsing
    println!("\n1. Testing HTML parsing...");
    let html_files = HtmlLogParser::scan_html_files("test_logs")?;
    println!("Found {} HTML files", html_files.len());

    let mut all_entries = Vec::new();
    for (index, file_path) in html_files.iter().enumerate() {
        println!("Parsing file: {}", file_path);
        let entries = HtmlLogParser::parse_file(file_path, "test_session_001", index)?;
        println!("  -> {} entries", entries.len());
        all_entries.extend(entries);
    }

    println!("Total entries parsed: {}", all_entries.len());

    // Test database operations
    println!("\n2. Testing database operations...");
    let db_manager = DatabaseManager::new("test_logterminator.db")?;

    // Insert entries
    println!("Inserting entries into database...");
    db_manager.insert_entries(&all_entries)?;

    // Query entries
    println!("Querying entries...");
    let (entries, total) = db_manager.get_entries_paginated("test_session_001", 0, 10, None, None)?;
    println!("Retrieved {} entries (total: {})", entries.len(), total);

    for entry in &entries {
        println!("  {} {} {}", entry.timestamp, entry.level, &entry.message[..std::cmp::min(50, entry.message.len())]);
    }

    println!("\n✅ All tests passed!");
    Ok(())
}