# HTTP Log Loading Design

**Date:** 2026-01-22
**Status:** Design Approved
**Author:** Collaborative Design Session

## Overview

Add support for loading test logs from HTTP servers in addition to local folders. Users will be able to enter a URL pointing to a directory listing (e.g., Apache/Nginx autoindex), and the application will fetch, parse, and display the logs the same way it does with local files.

## Motivation

Currently, logTerminator only loads logs from local directories. Many teams host test logs on web servers or CI artifacts. Loading directly from HTTP eliminates the need to download logs locally before viewing them.

## Architecture

### New Rust Module: `http_log_fetcher.rs`

A new module will handle HTTP log fetching alongside the existing local file parser in `log_parser.rs`.

**Core Component: `HttpLogFetcher`**

1. **Fetch directory listings** - Given a base URL, fetch the HTML directory listing page
2. **Parse directory links** - Extract file/directory links from listing HTML using `scraper` crate
3. **Filter test logs** - Apply same filename pattern (`<TestName>_<ID_X>---<Y>.html`) as local scanning
4. **Download log files** - Fetch each matching HTML file's content over HTTP
5. **Delegate to parser** - Pass fetched HTML content to `HtmlLogParser` for parsing

### Backend Integration

**New Tauri command in `lib.rs`:**
```rust
#[tauri::command]
async fn parse_log_http_url(url: String) -> Result<Vec<String>, String>
```

- Returns same format as `parse_log_directory`: `Vec<String>` of session IDs
- Uses `tokio::task::spawn_blocking` for background thread pool execution
- Reuses all existing parsing logic and database operations

**New dependency in `Cargo.toml`:**
```toml
reqwest = { version = "0.12", features = ["blocking"] }
```

## Data Flow

```
User clicks "Open Directory"
    ↓
Dialog shows: Local Folder OR HTTP URL
    ↓
User enters HTTP URL (e.g., http://logs.example.com/test-run/)
    ↓
Frontend: invoke('parse_log_http_url', { url })
    ↓
Backend: spawn_blocking task
    ↓
fetch_directory_listing(url) → HTML
    ↓
parse_directory_links(html) → Vec<Url>
    ↓
filter_test_log_files(links) → matching *.html files
    ↓
Sequential download with delay:
    for each url:
        fetch_html_content(url) → String
        parse_html_to_entries(content) → Vec<LogEntry>
    ↓
group_entries_by_session(entries)
    ↓
insert_sessions_and_entries_to_db()
    ↓
return session_ids
```

**URL Handling:**
- Accept URLs with or without trailing slash
- Resolve relative links against base URL
- Support both absolute and relative links in HTML
- Follow HTTP redirects (up to 5)

**Concurrency:**
- Download HTML files sequentially to avoid overwhelming server
- 50-100ms delay between requests for politeness

## UI/UX Changes

### Enhanced "Open Directory" Dialog

Replace simple directory picker with a dialog supporting both sources:

```
┌─────────────────────────────────────┐
│  Open Log Source                    │
├─────────────────────────────────────┤
│  ○ Local Folder                     │
│    [Browse...]                      │
│                                     │
│  ○ HTTP Server                      │
│    http://_______________            │
│                                     │
│         [Cancel]     [Open]         │
└─────────────────────────────────────┘
```

**Frontend logic:**
```javascript
async function openLogSource() {
  const selection = await showSourceDialog()
  if (selection.type === 'url' && selection.url.match(/^https?:\/\//)) {
    await invoke('parse_log_http_url', { url: selection.url })
  } else if (selection.type === 'folder') {
    await invoke('parse_log_directory', { directoryPath: selection.path })
  }
}
```

### Progress Messages

New loading states for HTTP fetching:
- "Connecting to server..."
- "Parsing directory listing..."
- "Found X test sessions"
- "Downloading log files (Y/Z)..."
- "Processing logs..."

### Session List Enhancement

Add visual indicator for log source:
- 📁 folder icon for local sessions
- 🌐 globe icon for HTTP sessions

## Error Handling

### Rust Error Types

```rust
pub enum HttpFetchError {
    InvalidUrl(String),
    NetworkError(reqwest::Error),
    TimeoutError,
    DirectoryListingNotFound,
    InvalidDirectoryListingFormat,
    DownloadFailed { url: String, reason: String },
    ParseError(String),
}
```

### Backend Error Handling

1. **Timeout**: 30-second timeout for directory listing and file downloads
2. **Retry logic**: Up to 2 retries with exponential backoff (1s, 2s)
3. **Partial recovery**: Continue with successful downloads if some fail
4. **Validation**: Only accept HTTP 200-299 status codes

### Frontend Error Messages

User-friendly messages for common errors:
- Invalid URL → "Invalid URL format. Please enter a valid HTTP/HTTPS URL."
- Timeout → "Request timed out. The server may be slow or unreachable."
- No directory listing → "Could not find directory listing. Check that the URL points to a directory."
- Download failed → "Failed to download log file. The server may be unavailable."

### Graceful Degradation

If HTTP fetch fails completely:
- Clear any partial data from that attempt
- Return to previous state (existing sessions remain intact)
- Show error but allow retry

## Testing

### Unit Tests

**`http_log_fetcher_tests.rs`:**
1. Mock HTML directory listings (Apache, Nginx formats) and verify link extraction
2. Test relative vs absolute link resolution
3. Verify test log filename pattern matching
4. Test error cases: invalid URLs, timeouts, malformed HTML

### Integration Testing

Create local test HTTP server for development:
```rust
// tests/http_test_server.rs
// Spin up a basic HTTP server serving static test log files
```

### Manual Testing Checklist

- [ ] Open URL with valid directory listing
- [ ] Open URL with no matching test logs
- [ ] Open invalid URL format
- [ ] Open URL that times out
- [ ] Open URL with mixed content (logs + other files)
- [ ] Switch between local and HTTP sources
- [ ] Verify sessions from HTTP show source icon
- [ ] Test with various directory listing formats (Apache, Nginx)

## Implementation Plan

1. Add `reqwest` dependency to `Cargo.toml`
2. Create `http_log_fetcher.rs` with core fetching logic
3. Add `parse_log_http_url` command to `lib.rs`
4. Update `App.vue` with enhanced dialog
5. Add source type indicator to database schema (optional, for display)
6. Write unit and integration tests
7. Manual testing with real servers

## Open Questions

1. Should we store the source URL in the database for future reference?
2. Should we add a "Re-download from URL" feature for HTTP sessions?
3. Do we need authentication support (basic auth, tokens)?
4. Should we support parallel downloads with a configurable limit?

## Notes

- This design assumes standard HTML directory listings (Apache mod_autoindex, Nginx autoindex)
- The same HTML log format is used - HTTP is just a different transport
- All existing features (bookmarks, filtering, search) work identically for HTTP-loaded logs
