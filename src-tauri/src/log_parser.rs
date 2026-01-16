use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<i64>,
    pub test_session_id: String,
    pub file_path: String,
    pub file_index: usize,
    pub timestamp: String,
    pub level: String,
    pub stack: String,
    pub message: String,
    pub line_number: usize,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub log_entry_id: i64,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub id: String,
    pub name: String,
    pub directory_path: String,
    pub file_count: usize,
    pub total_entries: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_parsed_at: Option<DateTime<Utc>>,
}

pub struct HtmlLogParser;

impl HtmlLogParser {
    pub fn parse_file(
        file_path: &str,
        test_session_id: &str,
        file_index: usize,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        println!("Parsing file: {}", file_path);

        let content = std::fs::read_to_string(file_path)?;
        let document = Html::parse_document(&content);

        // Safe selector parsing - handle invalid selectors gracefully
        let row_selector = Selector::parse("table tr").ok();
        let cell_selector = Selector::parse("td").ok();
        // Selector for hidden stack trace td
        let stack_selector = Selector::parse("td.stack[hidden]").ok();

        let row_selector = match row_selector {
            Some(s) => s,
            None => {
                println!("Warning: Could not parse row selector");
                return Ok(Vec::new());
            }
        };

        let mut entries = Vec::new();
        let mut line_number = 0;

        for row in document.select(&row_selector) {
            // Extract cells using available selector
            let cells: Vec<_> = if let Some(cell_sel) = &cell_selector {
                row.select(cell_sel).collect()
            } else {
                Vec::new()
            };

            // Skip header row and ensure we have enough cells
            if cells.len() >= 4 {
                let timestamp_text = cells[0].text().collect::<String>().trim().to_string();
                let level_text = cells[1].text().collect::<String>().trim().to_string();
                
                // Try to get stack from hidden td.stack element first
                let mut stack_text = String::new();
                if let Some(stack_sel) = &stack_selector {
                    if let Some(stack_elem) = row.select(stack_sel).next() {
                        stack_text = stack_elem.text().collect::<String>().trim().to_string();
                    }
                }

                // If no stack from hidden element, try cells[2]
                if stack_text.is_empty() && cells.len() > 2 {
                    let cell2_text = cells[2].text().collect::<String>().trim().to_string();
                    // If it looks like a stack trace, use it
                    if cell2_text.contains("File ") || cell2_text.contains(".py:") 
                        || cell2_text.contains("line ") || cell2_text.len() > 50 {
                        stack_text = cell2_text;
                    }
                }

                // Get message from last cell
                let mut message_text = cells[cells.len() - 1].text().collect::<String>().trim().to_string();

                // If message looks like a stack trace, it's probably the actual message
                // and the real stack is elsewhere (hidden), so swap them
                if message_text.contains("File ") && message_text.contains(".py:") 
                    && stack_text.contains("File ") && stack_text.len() > 100 {
                    // message has short trace, stack has full trace - keep stack in stack
                    // message might be empty or just a brief error
                } else if message_text.contains("File ") && message_text.contains(".py:")
                    && stack_text.is_empty() {
                    // Message contains the stack trace, put it in stack, clear message
                    // This means there was no hidden stack, message has it all
                    // But we need to find if there's a separate message
                    // For now, keep the trace in stack, leave message as is
                }

                // Skip empty rows or header rows
                if !timestamp_text.is_empty() && !level_text.is_empty() 
                    && timestamp_text != "Timestamp" && level_text != "Level" {
                    let entry = LogEntry {
                        id: None,
                        test_session_id: test_session_id.to_string(),
                        file_path: file_path.to_string(),
                        file_index,
                        timestamp: timestamp_text,
                        level: level_text,
                        stack: stack_text,
                        message: message_text,
                        line_number,
                        created_at: Some(Utc::now()),
                    };
                    entries.push(entry);
                }
            }
            line_number += 1;
        }

        println!("Parsed {} log entries from {}", entries.len(), file_path);
        Ok(entries)
    }

    pub fn scan_html_files(directory_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut html_files = Vec::new();

        for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "html" {
                        if let Some(path_str) = path.to_str() {
                            html_files.push(path_str.to_string());
                        }
                    }
                }
            }
        }

        // Sort files by index in filename (e.g., ---0.html, ---1.html, etc.)
        html_files.sort_by(|a, b| {
            let a_index = Self::extract_file_index(a);
            let b_index = Self::extract_file_index(b);
            a_index.cmp(&b_index)
        });

        println!("Found {} HTML files in {}", html_files.len(), directory_path);
        Ok(html_files)
    }

    pub fn extract_file_index(file_path: &str) -> usize {
        use std::path::Path;

        if let Some(filename) = Path::new(file_path).file_name() {
            if let Some(filename_str) = filename.to_str() {
                if let Some(dash_pos) = filename_str.rfind("---") {
                    if let Some(dot_pos) = filename_str.rfind(".html") {
                        let index_str = &filename_str[dash_pos + 3..dot_pos];
                        if let Ok(index) = index_str.parse::<usize>() {
                            return index;
                        }
                    }
                }
            }
        }
        0
    }
}