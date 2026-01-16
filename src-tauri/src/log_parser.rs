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

        // Select table rows
        let row_selector = Selector::parse("table tr").ok();
        let row_selector = match row_selector {
            Some(s) => s,
            None => {
                println!("Warning: Could not parse row selector");
                return Ok(Vec::new());
            }
        };

        // Direct selectors for specific cell types
        let date_selector = Selector::parse("td.date").ok();
        let level_selector = Selector::parse("td.level").ok();
        let message_selector = Selector::parse("td.message").ok();
        let stack_selector = Selector::parse("td.stack[hidden]").ok();
        let th_selector = Selector::parse("th").ok();
        let td_any_selector = Selector::parse("td").ok();

        let th_selector = match th_selector {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let td_any_selector = match td_any_selector {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let mut entries = Vec::new();
        let mut line_number = 0;

        for row in document.select(&row_selector) {
            // Skip header row (has th elements)
            if row.select(&th_selector).next().is_some() {
                line_number += 1;
                continue;
            }

            // Extract timestamp from td.date
            let timestamp_text = if let Some(sel) = &date_selector {
                row.select(sel).next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            } else {
                row.select(&td_any_selector).next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            };

            if timestamp_text.is_empty() || timestamp_text == "Timestamp" {
                line_number += 1;
                continue;
            }

            // Extract level from td.level
            let level_text = if let Some(sel) = &level_selector {
                row.select(sel).next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            } else {
                let cells: Vec<_> = row.select(&td_any_selector).collect();
                if cells.len() > 1 {
                    cells[1].text().collect::<String>().trim().to_string()
                } else {
                    String::new()
                }
            };

            // Extract message from td.message
            let message_text = if let Some(sel) = &message_selector {
                row.select(sel).next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            } else {
                let cells: Vec<_> = row.select(&td_any_selector).collect();
                if !cells.is_empty() {
                    cells[cells.len() - 1].text().collect::<String>().trim().to_string()
                } else {
                    String::new()
                }
            };

            // Try to find hidden stack trace
            let stack_text = if let Some(sel) = &stack_selector {
                row.select(sel).next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

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
