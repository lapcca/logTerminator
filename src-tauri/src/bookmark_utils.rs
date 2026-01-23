use crate::log_parser::{Bookmark, LogEntry};
use regex::Regex;

/// Finds auto-bookmark markers in log entries.
///
/// Scans all log entries for messages matching the pattern `###MARKER###`
/// and extracts bookmark titles from content between the `###` markers.
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
/// // Message: "###TEST START###"
/// // Returns: vec![(entry_id, "TEST START".to_string())]
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
        // Find first match in the message
        if let Some(caps) = re.captures(&entry.message) {
            if let Some(title_match) = caps.get(1) {
                let title = title_match.as_str().trim().to_string();

                // Skip empty markers
                if title.is_empty() {
                    continue;
                }

                // Only proceed if entry has an ID (after database insertion)
                if let Some(entry_id) = entry.id {
                    markers.push((entry_id, title));
                }
            }
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
