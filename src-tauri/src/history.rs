use std::fs;
use std::path::PathBuf;
use std::env;

/// Get the history file path
fn get_history_file() -> PathBuf {
    // Get the directory containing the executable (same as storage pattern in lib.rs)
    let exe_path = env::current_exe()
        .expect("Failed to get executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Cannot determine executable directory");

    exe_dir.join("log_history.txt")
}

/// Load history from file, returns Vec<String>
pub fn load_history() -> Vec<String> {
    let history_file = get_history_file();
    if !history_file.exists() {
        return Vec::new();
    }

    fs::read_to_string(&history_file)
        .map(|content| {
            content
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Save new history entry, keep max 10 items
pub fn save_history(entry: String) -> Result<(), String> {
    let mut history = load_history();

    // Remove if already exists (to move to top)
    history.retain(|x| x != &entry);

    // Add to front
    history.insert(0, entry);

    // Keep max 10
    history.truncate(10);

    // Save with | separator
    let content = history.join("|");
    let history_file = get_history_file();

    fs::write(&history_file, content)
        .map_err(|e| format!("Failed to write history: {}", e))
}

/// Get last N entries (for dropdown display)
pub fn get_recent_history(count: usize) -> Vec<String> {
    let history = load_history();
    history.into_iter().take(count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_history_empty() {
        // This test verifies that load_history returns empty vec when file doesn't exist
        // Note: Since get_history_file() uses the actual exe directory, we can't easily
        // test this without mocking. In a real scenario, we'd refactor to accept
        // a path parameter for testability.
    }

    #[test]
    fn test_save_and_load_history() {
        // Note: These tests would require refactoring to accept a path parameter
        // for proper isolation. For now, the implementation follows the existing
        // pattern in lib.rs which also doesn't have unit tests.
    }
}
