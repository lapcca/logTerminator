# HTTP Log Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add ability to load test logs from HTTP servers (static file servers with HTML directory listings) in addition to local folders.

**Architecture:** New `http_log_fetcher.rs` module fetches directory listings and log files over HTTP using `reqwest`, then delegates to existing `HtmlLogParser` for parsing. Single Tauri command `parse_log_http_url` mirrors existing `parse_log_directory` command.

**Tech Stack:** Rust (reqwest for HTTP, scraper for HTML parsing), Vue 3 (enhanced dialog for URL input)

---

## Task 1: Add reqwest dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add reqwest dependency**

Add to `[dependencies]` section in `Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["blocking"] }
```

**Step 2: Verify dependency resolves**

Run: `cargo build`
Expected: Successful compilation, reqwest crates downloaded

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add reqwest dependency for HTTP fetching"
```

---

## Task 2: Create http_log_fetcher.rs module with error types

**Files:**
- Create: `src-tauri/src/http_log_fetcher.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create module with error types**

Create `src-tauri/src/http_log_fetcher.rs`:

```rust
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
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
            HttpFetchError::DownloadFailed { url, reason } => write!(f, "Failed to download {}: {}", url, reason),
            HttpFetchError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::fmt::Debug for HttpFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Debug::Formatter<'_> {
        write!(f, "{}", self)
    }
}

/// HTTP log fetcher for downloading logs from web servers
pub struct HttpLogFetcher {
    client: Client,
    base_url: Url,
}
```

**Step 2: Add module to lib.rs**

Add to `src-tauri/src/lib.rs` at the top:

```rust
mod http_log_fetcher;
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src-tauri/src/http_log_fetcher.rs src-tauri/src/lib.rs
git commit -m "feat: add http_log_fetcher module with error types"
```

---

## Task 3: Implement directory listing fetching

**Files:**
- Modify: `src-tauri/src/http_log_fetcher.rs`

**Step 1: Write test for directory listing parsing**

Add to `src-tauri/tests/http_log_fetcher_tests.rs` (create new file):

```rust
use logterminator_lib::http_log_fetcher::HttpLogFetcher;

#[test]
fn test_parse_apache_directory_listing() {
    let html = r#"
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 3.2 Final//EN">
<html>
<head>
  <title>Index of /logs</title>
</head>
<body>
<h1>Index of /logs</h1>
<table>
  <tr><th valign="top"><img src="/icons/blank.gif" alt="[ICO]"></th><th><a href="?C=N;O=D">Name</a></th><th><a href="?C=M;O=A">Last modified</a></th><th><a href="?C=S;O=A">Size</a></th></tr>
  <tr><th colspan="4"><hr></th></tr>
  <tr><td valign="top"><img src="/icons/back.gif" alt="[PARENTDIR]"></td><td><a href="../">Parent Directory</a></td><td>&nbsp;</td><td align="right">  - </td></tr>
  <tr><td valign="top"><img src="/icons/unknown.gif" alt="[   ]"></td><td><a href="TestEnableTcpdump_ID_1---0.html">TestEnableTcpdump_ID_1---0.html</a></td><td align="right">2025-01-15 10:30  </td><td align="right"> 12K</td></tr>
  <tr><td valign="top"><img src="/icons/unknown.gif" alt="[   ]"></td><td><a href="TestEnableTcpdump_ID_1---1.html">TestEnableTcpdump_ID_1---1.html</a></td><td align="right">2025-01-15 10:31  </td><td align="right"> 15K</td></tr>
  <tr><td valign="top"><img src="/icons/folder.gif" alt="[DIR]"></td><td><a href="subdir/">subdir/</a></td><td align="right">2025-01-15 10:32  </td><td align="right">  - </td></tr>
</table>
</body></html>
    "#;

    // Will implement parsing logic next
    let urls = HttpLogFetcher::parse_directory_listing(html, "http://example.com/logs/").unwrap();
    assert_eq!(urls.len(), 2);
    assert!(urls.contains(&"http://example.com/logs/TestEnableTcpdump_ID_1---0.html".to_string()));
    assert!(urls.contains(&"http://example.com/logs/TestEnableTcpdump_ID_1---1.html".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_apache_directory_listing`
Expected: FAIL with "no such method" or similar

**Step 3: Implement directory listing parsing**

Add to `HttpLogFetcher` impl in `http_log_fetcher.rs`:

```rust
impl HttpLogFetcher {
    /// Parse directory listing HTML and extract all file URLs
    pub fn parse_directory_listing(html: &str, base_url: &str) -> Result<Vec<String>, HttpFetchError> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a[href]").unwrap();

        let base = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

        let mut urls = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Skip parent directory links
                if href == "../" || href.starts_with("?") {
                    continue;
                }

                // Skip directory links (ending with /)
                if href.ends_with('/') {
                    continue;
                }

                // Resolve relative URLs against base
                match base.join(href) {
                    Ok(full_url) => {
                        urls.push(full_url.to_string());
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
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_parse_apache_directory_listing`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/http_log_fetcher.rs src-tauri/tests/http_log_fetcher_tests.rs
git commit -m "feat: implement directory listing parsing"
```

---

## Task 4: Implement log file fetching

**Files:**
- Modify: `src-tauri/src/http_log_fetcher.rs`

**Step 1: Write test for fetching log file content**

Add to `src-tauri/tests/http_log_fetcher_tests.rs`:

```rust
#[test]
fn test_fetch_single_file() {
    // This test will be mocked or use a local test server
    // For now, we'll test the URL construction and client creation

    let fetcher = HttpLogFetcher::new("http://example.com/logs/").unwrap();
    assert_eq!(fetcher.base_url().as_str(), "http://example.com/logs/");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_fetch_single_file`
Expected: FAIL with "no such method" or similar

**Step 3: Implement HttpLogFetcher constructor and file fetching**

Add to `HttpLogFetcher` impl in `http_log_fetcher.rs`:

```rust
impl HttpLogFetcher {
    /// Create a new HTTP log fetcher
    pub fn new(base_url: &str) -> Result<Self, HttpFetchError> {
        let url = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

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

    /// Fetch a single log file's HTML content
    pub fn fetch_log_file(&self, file_url: &str) -> Result<String, HttpFetchError> {
        let response = self.client
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
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_fetch_single_file`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/http_log_fetcher.rs src-tauri/tests/http_log_fetcher_tests.rs
git commit -m "feat: implement log file fetching"
```

---

## Task 5: Implement main fetch orchestrator

**Files:**
- Modify: `src-tauri/src/http_log_fetcher.rs`

**Step 1: Write integration test for full flow**

Add to `src-tauri/tests/http_log_fetcher_tests.rs`:

```rust
#[test]
fn test_filter_test_log_files() {
    let urls = vec![
        "http://example.com/TestABC_ID_1---0.html".to_string(),
        "http://example.com/TestABC_ID_1---1.html".to_string(),
        "http://example.com/MainRollup.html".to_string(),
        "http://example.com/summary.html".to_string(),
        "http://example.com/TestEnableTcpdump_ID_2---0.html".to_string(),
    ];

    let filtered = HttpLogFetcher::filter_test_log_files(&urls);
    assert_eq!(filtered.len(), 3);
    assert!(filtered.contains(&"http://example.com/TestABC_ID_1---0.html".to_string()));
    assert!(filtered.contains(&"http://example.com/TestABC_ID_1---1.html".to_string()));
    assert!(filtered.contains(&"http://example.com/TestEnableTcpdump_ID_2---0.html".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_filter_test_log_files`
Expected: FAIL with "no such method" or similar

**Step 3: Implement filter_test_log_files**

Add to `HttpLogFetcher` in `http_log_fetcher.rs`:

```rust
impl HttpLogFetcher {
    /// Filter URLs to only include test log files matching the pattern
    pub fn filter_test_log_files(urls: &[String]) -> Vec<String> {
        urls.iter()
            .filter(|url| {
                if let Some(filename) = url.rsplit('/').next() {
                    crate::log_parser::HtmlLogParser::is_test_log_file(filename)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_filter_test_log_files`
Expected: PASS

**Step 5: Implement full orchestrator with progress callback**

Add to `http_log_fetcher.rs` (new function outside impl):

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Fetch all logs from HTTP server and return session IDs
pub fn fetch_logs_from_http(
    url: String,
    progress_callback: impl Fn(String),
) -> Result<Vec<String>, HttpFetchError> {
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

    // Download each log file
    let counter = Arc::new(AtomicUsize::new(0));
    let total = test_log_urls.len();
    let mut all_entries = Vec::new();

    for log_url in &test_log_urls {
        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
        progress_callback(format!("Downloading log files ({}/{})...", count, total));

        // Small delay for politeness
        std::thread::sleep(Duration::from_millis(50));

        let html_content = fetcher.fetch_log_file(log_url)?;

        // Parse using existing parser
        let filename = log_url.rsplit('/').next().unwrap_or("unknown.html");
        let entries = crate::log_parser::HtmlLogParser::parse_html_string(&html_content, filename)?;

        all_entries.extend(entries);
    }

    progress_callback("Processing logs...".to_string());

    // Group by session and insert into database
    // This will reuse existing database logic
    let session_ids = crate::database::insert_entries_from_parser(all_entries)?;

    progress_callback("Complete".to_string());

    Ok(session_ids)
}
```

**Step 6: Add helper function to database.rs**

Add to `src-tauri/src/database.rs`:

```rust
/// Insert log entries from parser and return session IDs
pub fn insert_entries_from_parser(entries: Vec<LogEntry>) -> Result<Vec<String>, String> {
    use crate::log_parser::TestSession;

    let conn = &mut get_connection()?;

    // Group by test session
    let mut sessions_map: std::collections::HashMap<String, Vec<LogEntry>> = std::collections::HashMap::new();

    for entry in entries {
        let key = format!("{}_{}", entry.test_name, entry.test_id);
        sessions_map.entry(key).or_default().push(entry);
    }

    let mut session_ids = Vec::new();

    for (session_key, session_entries) in sessions_map {
        // Create test session
        let first_entry = &session_entries[0];
        let test_name = format!("{}_{}", first_entry.test_name, first_entry.test_id);

        let session_id = format!("session_{}_{}", sanitize(&test_name), timestamp());
        let created_at = Utc::now();

        conn.execute(
            "INSERT INTO test_sessions (id, name, created_at) VALUES (?1, ?2, ?3)",
            [&session_id, &test_name, &created_at.to_string()],
        ).map_err(|e| e.to_string())?;

        // Insert entries
        let mut stmt = conn.prepare_cached(
            "INSERT INTO log_entries (test_session_id, timestamp, level, message, stack_trace, file_path, line_number)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        ).map_err(|e| e.to_string())?;

        for entry in session_entries {
            stmt.execute((
                &session_id,
                &entry.timestamp,
                &entry.level,
                &entry.message,
                &entry.stack_trace,
                &entry.file_path,
                &entry.line_number as &i32,
            )).map_err(|e| e.to_string())?;
        }

        session_ids.push(session_id);
    }

    Ok(session_ids)
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect()
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
}
```

**Step 7: Run tests to verify compilation**

Run: `cargo test`
Expected: Compilation succeeds, tests may fail (mocking issues expected)

**Step 8: Commit**

```bash
git add src-tauri/src/http_log_fetcher.rs src-tauri/src/database.rs src-tauri/tests/http_log_fetcher_tests.rs
git commit -m "feat: implement HTTP fetch orchestrator"
```

---

## Task 6: Add Tauri command for HTTP log loading

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add parse_log_http_url command**

Add to `src-tauri/src/lib.rs` after `parse_log_directory` command:

```rust
#[tauri::command]
async fn parse_log_http_url(
    url: String,
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
) -> Result<Vec<String>, String> {
    // Use spawn_blocking to avoid blocking async runtime
    let (tx, rx) = tokio::sync::oneshot::channel();

    let url_clone = url.clone();
    let window_clone = window.clone();

    std::thread::spawn(move || {
        let result = crate::http_log_fetcher::fetch_logs_from_http(
            url_clone,
            |msg| {
                let _ = window_clone.emit("http-progress", msg);
            },
        );
        let _ = tx.send(result);
    });

    rx.await.map_err(|e| format!("Join error: {}", e))?
}
```

**Step 2: Register command in main()**

Find the `invoke_handler` in `main.rs` or `lib.rs` and add the new command:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    parse_log_directory,
    parse_log_http_url,  // Add this
    // ... other commands ...
])
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add parse_log_http_url Tauri command"
```

---

## Task 7: Update Vue frontend with source dialog

**Files:**
- Modify: `src/App.vue`

**Step 1: Add new reactive state variables**

Add to `<script setup>` section in `App.vue`:

```javascript
// Log source dialog
const showSourceDialog = ref(false)
const sourceType = ref('folder') // 'folder' or 'url'
const httpUrl = ref('')
const selectedFolderPath = ref('')

// Helper for dialog
const canOpen = computed(() => {
  if (sourceType.value === 'folder') {
    return selectedFolderPath.value !== ''
  } else {
    return httpUrl.value !== '' && httpUrl.value.match(/^https?:\/\//)
  }
})
```

**Step 2: Create new dialog component**

Add to `<template>` section in `App.vue` (before the existing Edit Bookmark Dialog):

```vue
<!-- Log Source Dialog -->
<v-dialog v-model="showSourceDialog" max-width="500">
  <v-card>
    <v-card-title class="d-flex align-center">
      <v-icon class="mr-2">mdi-folder-open</v-icon>
      Open Log Source
    </v-card-title>
    <v-card-text class="pt-4">
      <v-radio-group v-model="sourceType">
        <v-radio label="Local Folder" value="folder"></v-radio>
        <v-btn
          v-if="sourceType === 'folder'"
          @click="selectLocalFolder"
          variant="outlined"
          prepend-icon="mdi-folder"
          class="ml-8 mb-4">
          {{ selectedFolderPath || 'Browse...' }}
        </v-btn>

        <v-radio label="HTTP Server" value="url" class="mt-4"></v-radio>
        <v-text-field
          v-if="sourceType === 'url'"
          v-model="httpUrl"
          label="Enter URL"
          variant="outlined"
          placeholder="http://logs.example.com/"
          prepend-inner-icon="mdi-web"
          class="ml-8"
          clearable>
        </v-text-field>
      </v-radio-group>
    </v-card-text>
    <v-card-actions>
      <v-spacer></v-spacer>
      <v-btn variant="text" @click="showSourceDialog = false">Cancel</v-btn>
      <v-btn
        color="primary"
        variant="flat"
        :disabled="!canOpen"
        @click="openLogSource">
        Open
      </v-btn>
    </v-card-actions>
  </v-card>
</v-dialog>
```

**Step 3: Update openDirectory function**

Replace the existing `openDirectory` function:

```javascript
// Show source dialog
async function openDirectory() {
  showSourceDialog.value = true
  sourceType.value = 'folder'
  selectedFolderPath.value = ''
  httpUrl.value = ''
}

// Select local folder
async function selectLocalFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Log Directory'
    })
    if (selected) {
      selectedFolderPath.value = selected
    }
  } catch (error) {
    console.error('Error selecting folder:', error)
  }
}

// Open log source (folder or URL)
async function openLogSource() {
  showSourceDialog.value = false

  if (sourceType.value === 'url' && httpUrl.value) {
    await loadFromHttpUrl(httpUrl.value)
  } else if (sourceType.value === 'folder' && selectedFolderPath.value) {
    await loadFromDirectory(selectedFolderPath.value)
  }
}

// Load from HTTP URL
async function loadFromHttpUrl(url) {
  loading.value = true
  loadingMessage.value = 'Connecting to server...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_http_url', { url })
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s) from server`
    }
  } catch (error) {
    console.error('Error loading from HTTP:', error)
    let userMsg = error
    if (error.includes('InvalidUrl')) {
      userMsg = 'Invalid URL format. Please enter a valid HTTP/HTTPS URL.'
    } else if (error.includes('Timeout')) {
      userMsg = 'Request timed out. The server may be slow or unreachable.'
    } else if (error.includes('DirectoryListingNotFound')) {
      userMsg = 'Could not find directory listing. Check that the URL points to a directory.'
    }
    alert(`Failed to load logs: ${userMsg}`)
    loadingMessage.value = ''
  } finally {
    setTimeout(() => {
      loading.value = false
      loadingMessage.value = ''
    }, 500)
  }
}

// Load from local directory
async function loadFromDirectory(directoryPath) {
  loading.value = true
  loadingMessage.value = 'Scanning directory...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_directory', { directoryPath })
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s)`
    }
  } catch (error) {
    console.error('Error loading from directory:', error)
    alert(`Error loading directory: ${error}`)
    loadingMessage.value = ''
  } finally {
    setTimeout(() => {
      loading.value = false
      loadingMessage.value = ''
    }, 500)
  }
}
```

**Step 4: Add http-progress event listener**

Add to `onMounted` or as a separate watcher:

```javascript
import { listen } from '@tauri-apps/api/event'

// In onMounted or separately
onMounted(() => {
  loadSessions()

  // Listen for HTTP progress events
  const unlisten = listen('http-progress', (event) => {
    loadingMessage.value = event.payload
  })

  // Clean up on unmount (if needed)
  // return () => { unlisten.then(fn => fn()) }
})
```

**Step 5: Verify frontend compiles**

Run: `npm run build` (or `pnpm build`)
Expected: Successful compilation

**Step 6: Test manually**

Run: `pnpm tauri dev`
Expected: App opens, "Open Directory" button shows new dialog

**Step 7: Commit**

```bash
git add src/App.vue
git commit -m "feat: add source dialog for HTTP/URL input"
```

---

## Task 8: Add visual indicator for log source type

**Files:**
- Modify: `src-tauri/src/database.rs`
- Modify: `src/App.vue`

**Step 1: Add source_type column to test_sessions table**

Add migration to `database.rs` in `get_connection()` or initialization:

```rust
// Add source_type column if not exists
conn.execute(
    "ALTER TABLE test_sessions ADD COLUMN source_type TEXT DEFAULT 'local'",
    []
).ok(); // Ignore error if column already exists
```

**Step 2: Update insert_entries_from_parser to accept source type**

Modify the function signature and insert:

```rust
pub fn insert_entries_from_parser(entries: Vec<LogEntry>, source_type: &str) -> Result<Vec<String>, String> {
    // ... existing code ...

    conn.execute(
        "INSERT INTO test_sessions (id, name, created_at, source_type) VALUES (?1, ?2, ?3, ?4)",
        [&session_id, &test_name, &created_at.to_string(), source_type],
    ).map_err(|e| e.to_string())?;

    // ... rest of code ...
}
```

**Step 3: Update fetch_logs_from_http to pass source type**

In `http_log_fetcher.rs`:

```rust
let session_ids = crate::database::insert_entries_from_parser(all_entries, "http")?;
```

**Step 4: Update parse_log_directory to pass source type**

In `lib.rs`, update the call to pass `"local"`:

```rust
let session_ids = crate::database::insert_entries_from_parser(all_entries, "local")?;
```

**Step 5: Update sessions query to include source_type**

Modify `get_sessions` command:

```rust
#[tauri::command]
async fn get_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.0.lock().unwrap();
    Ok(db.get_sessions_with_source()?)
}
```

Add method to database.rs:

```rust
pub fn get_sessions_with_source(&self) -> Result<Vec<Session>, String> {
    let conn = &mut get_connection()?;

    let mut stmt = conn.prepare_cached(
        "SELECT id, name, created_at, source_type,
         (SELECT COUNT(*) FROM log_entries WHERE test_session_id = test_sessions.id) as total_entries
         FROM test_sessions ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let sessions = stmt.query_map([], |row| {
        Ok(Session {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            total_entries: row.get(3)?,
            source_type: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| e.to_string())?;

    Ok(sessions)
}
```

Update Session struct in lib.rs:

```rust
#[derive(Serialize, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub total_entries: i64,
    pub source_type: String,  // Add this field
}
```

**Step 6: Update Vue template to show source icon**

In session list template:

```vue
<v-list-item
    v-for="session in sessions"
    :key="session.id"
    ...>
    <template v-slot:prepend>
    <v-avatar
        :color="currentSession === session.id ? 'primary' : 'grey-lighten-2'"
        size="32"
        class="mr-3">
        <v-icon
        :color="currentSession === session.id ? 'white' : 'grey'"
        size="small">
        {{ session.source_type === 'http' ? 'mdi-web' : 'mdi-folder' }}
        </v-icon>
    </v-avatar>
    </template>
    <!-- ... rest of template ... -->
</v-list-item>
```

**Step 7: Verify and test**

Run: `pnpm tauri dev`
Expected: Sessions show folder icon for local, globe icon for HTTP

**Step 8: Commit**

```bash
git add src-tauri/src/database.rs src-tauri/src/lib.rs src-tauri/src/http_log_fetcher.rs src/App.vue
git commit -m "feat: add source type indicator to sessions"
```

---

## Task 9: End-to-end testing

**Files:**
- Create: `src-tauri/tests/e2e_http_test.rs` (optional)

**Step 1: Manual testing checklist**

Run: `pnpm tauri dev`

Test:
- [ ] Open local folder (existing functionality still works)
- [ ] Open HTTP URL with valid directory listing
- [ ] Open HTTP URL with no matching test logs
- [ ] Open invalid URL format
- [ ] Verify source icons display correctly
- [ ] Search/filter works with HTTP-loaded logs
- [ ] Bookmarks work with HTTP-loaded logs

**Step 2: Document testing results**

Add to `docs/plans/2026-01-22-http-log-loading.md`:

```markdown
## Testing Results

Date: [Date]
Tester: [Name]

- [x] Local folder loading - PASS
- [x] HTTP URL loading - PASS
- [x] Source icons display - PASS
- [x] Search/filter with HTTP logs - PASS
- [x] Bookmarks with HTTP logs - PASS

Known issues: [List any]
```

**Step 3: Final commit if any fixes needed**

```bash
git add .
git commit -m "fix: [describe any issues found and fixed during testing]"
```

---

## Task 10: Clean up and merge

**Files:**
- No specific files

**Step 1: Run full test suite**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All tests pass (except known pre-existing doctest failure)

**Step 2: Build release version**

Run:
```bash
cd src-tauri && cargo build --release
```

Expected: Successful release build

**Step 3: Switch back to main branch and merge**

```bash
cd ../..
git checkout master
git merge feature/http-log-loading
```

**Step 4: Push to remote**

```bash
git push origin master
```

**Step 5: Clean up worktree (optional)**

```bash
git worktree remove .worktrees/http-log-loading
```

---

## Summary

This plan implements HTTP log loading in 10 bite-sized tasks:

1. ✅ Add reqwest dependency
2. ✅ Create http_log_fetcher module with error types
3. ✅ Implement directory listing parsing
4. ✅ Implement log file fetching
5. ✅ Implement main fetch orchestrator
6. ✅ Add Tauri command for HTTP log loading
7. ✅ Update Vue frontend with source dialog
8. ✅ Add visual indicator for log source type
9. ✅ End-to-end testing
10. ✅ Clean up and merge

Each task follows TDD: write test → run failing → implement → run passing → commit.
