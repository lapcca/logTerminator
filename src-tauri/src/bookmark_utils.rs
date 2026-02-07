use crate::log_parser::{Bookmark, LogEntry};
use regex::Regex;

/// Finds auto-bookmark markers in log entries.
///
/// Scans all log entries for two patterns:
/// 1. Messages containing `###MARKER###` - extracts title from between ###
/// 2. Log entries with level "MARKER" - uses full message as title
///
/// If both conditions are met, only one bookmark is created (### pattern takes priority).
///
/// # Arguments
/// * `entries` - Slice of log entries to scan for markers
///
/// # Returns
/// * `Vec<(i64, String)>` - List of tuples containing (entry_id, title) for each marker found
///
/// # Examples
/// ```
/// use logterminator_lib::bookmark_utils::find_auto_bookmark_markers;
/// // Message: "###TEST START###", level: INFO
/// // Returns: vec![(entry_id, "TEST START".to_string())]
///
/// // Message: "Some marker message", level: MARKER
/// // Returns: vec![(entry_id, "Some marker message".to_string())]
///
/// // Message: "###TEST###", level: MARKER (both conditions)
/// // Returns: vec![(entry_id, "TEST".to_string())] (only once, ### pattern wins)
/// ```
pub fn find_auto_bookmark_markers(entries: &[LogEntry]) -> Vec<(i64, String)> {
    let mut markers = Vec::new();

    // Compile regex pattern for ###MARKER### format
    // Pattern: ###(text)###
    let re = match Regex::new(r"###(.+?)###") {
        Ok(regex) => regex,
        Err(e) => {
            eprintln!("Failed to compile auto-bookmark regex: {}", e);
            return markers;
        }
    };

    for entry in entries {
        // Only proceed if entry has an ID (after database insertion)
        let entry_id = match entry.id {
            Some(id) => id,
            None => continue,
        };

        // Priority 1: Check for ###TEXT### pattern in message
        let mut title = None;

        if let Some(caps) = re.captures(&entry.message) {
            if let Some(title_match) = caps.get(1) {
                let extracted_title = title_match.as_str().trim().to_string();
                // Skip empty markers
                if !extracted_title.is_empty() {
                    title = Some(extracted_title);
                }
            }
        }

        // Priority 2: If no ### pattern found, check for MARKER level
        if title.is_none() && entry.level == "MARKER" {
            title = Some(entry.message.clone());
        }

        // Add bookmark if we found a title
        if let Some(t) = title {
            markers.push((entry_id, t));
        }
    }

    markers
}

/// Creates an auto-bookmark with the standard blue color.
///
/// # Arguments
/// * `log_entry_id` - The ID of the log entry to bookmark
/// * `title` - The title extracted from the marker pattern
///
/// # Returns
/// * `Bookmark` - A bookmark with auto-generated properties
pub fn create_auto_bookmark(log_entry_id: i64, title: String) -> Bookmark {
    Bookmark {
        id: None,
        log_entry_id,
        title: Some(title),
        notes: None,
        color: Some("#409EFF".to_string()), // Element Plus primary blue
        created_at: Some(chrono::Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: i64, message: &str) -> LogEntry {
        LogEntry {
            id: Some(id),
            test_session_id: "test_session".to_string(),
            file_path: "/test/path".to_string(),
            file_index: 0,
            timestamp: "2024-01-01 12:00:00".to_string(),
            level: "INFO".to_string(),
            stack: "".to_string(),
            message: message.to_string(),
            line_number: 1,
            created_at: None,
        }
    }

    #[test]
    fn test_find_auto_bookmark_markers_basic() {
        let entries = vec![
            create_test_entry(1, "###TEST START###"),
            create_test_entry(2, "Normal log message"),
            create_test_entry(3, "###STEP 1###"),
        ];

        let markers = find_auto_bookmark_markers(&entries);

        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0], (1, "TEST START".to_string()));
        assert_eq!(markers[1], (3, "STEP 1".to_string()));
    }

    #[test]
    fn test_find_auto_bookmark_markers_marker_level() {
        let mut entry1 = create_test_entry(1, "Some marker message");
        entry1.level = "MARKER".to_string();

        let entries = vec![
            entry1,
            create_test_entry(2, "Normal log message"),
        ];

        let markers = find_auto_bookmark_markers(&entries);

        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0], (1, "Some marker message".to_string()));
    }

    #[test]
    fn test_find_auto_bookmark_markers_both_conditions() {
        // When both ### pattern and MARKER level exist, ### pattern wins
        let mut entry1 = create_test_entry(1, "###TEST START###");
        entry1.level = "MARKER".to_string();

        let entries = vec![entry1];

        let markers = find_auto_bookmark_markers(&entries);

        // Should only create one bookmark with the ### title
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0], (1, "TEST START".to_string()));
    }

    #[test]
    fn test_find_auto_bookmark_markers_empty_marker() {
        let entries = vec![
            create_test_entry(1, "######"),
            create_test_entry(2, "###VALID###"),
        ];

        let markers = find_auto_bookmark_markers(&entries);

        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0], (2, "VALID".to_string()));
    }

    #[test]
    fn test_find_auto_bookmark_markers_multiple_in_message() {
        let entries = vec![create_test_entry(1, "###FIRST### ###SECOND###")];

        let markers = find_auto_bookmark_markers(&entries);

        // Should use first match only
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0], (1, "FIRST".to_string()));
    }

    #[test]
    fn test_find_auto_bookmark_markers_no_id() {
        let mut entry = create_test_entry(1, "###TEST###");
        entry.id = None;

        let entries = vec![entry];
        let markers = find_auto_bookmark_markers(&entries);

        // Entries without IDs should be skipped
        assert_eq!(markers.len(), 0);
    }

    #[test]
    fn test_create_auto_bookmark() {
        let bookmark = create_auto_bookmark(123, "Test Title".to_string());

        assert_eq!(bookmark.log_entry_id, 123);
        assert_eq!(bookmark.title, Some("Test Title".to_string()));
        assert_eq!(bookmark.notes, None);
        assert_eq!(bookmark.color, Some("#409EFF".to_string()));
        assert!(bookmark.created_at.is_some());
    }
}
