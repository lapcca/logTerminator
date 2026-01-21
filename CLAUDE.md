# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

logTerminator is a Tauri desktop application that parses HTML test logs and provides a UI for viewing, filtering, and bookmarking log entries. It consists of:

- **Frontend**: Vue 3 with Vuetify UI components
- **Backend**: Rust with Tauri framework
- **Database**: SQLite (stored as `logterminator.db`)
- **Package Manager**: pnpm

### Key Dependencies

**Frontend**:
- Vue 3: Reactive UI framework
- Vuetify 3: Material Design component library
- Vite: Fast build tool and dev server
- Tauri APIs: Communication with Rust backend
- @tauri-apps/plugin-dialog: File/directory picker

**Backend**:
- Tauri: Desktop application framework
- Rusqlite: SQLite database bindings (with bundled SQLite)
- Scraper: HTML parsing and CSS selectors
- Tokio: Asynchronous runtime
- Chrono: Date/time handling with serde support
- Walkdir: Directory traversal utilities
- Serde: Serialization/deserialization

## Common Commands

### Development
```bash
# Start development server (frontend + backend)
pnpm tauri dev

# Frontend dev only
pnpm dev
pnpm build
pnpm preview
```

### Rust Backend
```bash
cd src-tauri

# Run all tests
cargo test

# Run specific test
cargo test test_function_name

# Run tests with output
cargo test -- --nocapture

# Linting
cargo clippy
cargo clippy --fix

# Formatting
cargo fmt
cargo fmt --check

# Production build
cargo build --release
```

### Production Build
```bash
pnpm tauri build    # Creates installer
pnpm tauri bundle    # Bundle only
```

### Development Environment

**Recommended IDE Setup**:
- VS Code + Volar (Vue) + Tauri (Rust) + rust-analyzer
- pnpm is the package manager (specified in package.json)

**Debugging**:
- Frontend: Use browser developer tools (F12)
- Backend: Use `println!` or `dbg!` macros, check logs in terminal
- Full App: Use `pnpm tauri dev` for integrated debugging

**Code Style**:
- Rust: `cargo fmt` (4-space indentation), `cargo clippy` for linting
- Vue: Use `<script setup>` syntax with Composition API

## Architecture

### Backend (Rust)

The backend is organized into three main modules in `src-tauri/src/`:

**`lib.rs`** - Tauri command handlers and application state
- Contains `AppState` wrapping `DatabaseManager` in a `Mutex`
- Defines all Tauri commands exposed to frontend
- `parse_log_directory`: Async command that spawns blocking thread pool task for HTML parsing
- `get_log_entries`: Paginated retrieval with level filter and search term support
- `add_bookmark`, `get_bookmarks`, `delete_bookmark`: Bookmark CRUD operations
- `get_entry_page`: Calculates which page a specific log entry appears on (considering filters)

**`log_parser.rs`** - HTML parsing and file scanning
- `HtmlLogParser`: Parses HTML log files using `scraper` crate with CSS selectors
- Expected table structure: `table tr > td.date`, `td.level`, `td.message`, `td.stack[hidden]`
- `scan_html_files`: Recursively scans directory for test logs matching naming pattern
- `is_test_log_file`: Validates filename pattern `<TestName>_<ID_X>---<Y>.html`
- Files not matching this pattern are ignored (e.g., `MainRollup.html`, `summary.html`)

**`database.rs`** - SQLite operations
- `DatabaseManager`: Manages SQLite connection and queries
- Tables: `test_sessions`, `log_entries`, `bookmarks`
- Foreign key cascade: Deleting bookmarks when log entries are deleted
- Indexes on: `test_session_id`, `timestamp`, `level`, `log_entry_id`

#### Database Schema

**test_sessions**:
- `id` (TEXT, PRIMARY KEY): Session identifier
- `name` (TEXT): Human-readable session name
- `created_at` (DATETIME): When session was created

**log_entries**:
- `id` (INTEGER, PRIMARY KEY, AUTOINCREMENT): Entry ID
- `test_session_id` (TEXT, FOREIGN KEY): References test_sessions.id
- `timestamp` (DATETIME): Log entry timestamp
- `level` (TEXT): Log level (INFO, ERROR, WARN, etc.)
- `message` (TEXT): Log message content
- `stack_trace` (TEXT, optional): Stack trace if available
- `file_path` (TEXT): Original HTML file path
- `line_number` (INTEGER): Line number in original file

**bookmarks**:
- `id` (INTEGER, PRIMARY KEY, AUTOINCREMENT): Bookmark ID
- `log_entry_id` (INTEGER, FOREIGN KEY): References log_entries.id
- `title` (TEXT): Bookmark title
- `notes` (TEXT, optional): Bookmark notes
- `color` (TEXT, optional): Bookmark color
- `created_at` (DATETIME): When bookmark was created

### Data Flow

1. User selects directory via `openDirectory()` in frontend
2. `parse_log_directory` command scans for HTML files matching test log pattern
3. For each test group found:
   - Creates `TestSession` record
   - Parses each HTML file into `LogEntry` records
   - Inserts entries in transaction
4. Frontend displays sessions in sidebar
5. User selects session → `get_log_entries` fetches paginated results
6. Bookmarks are stored per log entry with title/notes/color

### Test Log Naming Convention

**Pattern:** `<TestName>_<ID_X>---<Y>.html`

- `<TestName>`: Test identifier (e.g., "TestEnableTcpdump", "TestABC")
- `<ID_X>`: Test instance ID (e.g., "ID_1")
- `<Y>`: Sequential file number (e.g., 0, 1, 2...)

**Examples:**
- `TestEnableTcpdump_ID_1---0.html` → Test session: TestEnableTcpdump_ID_1
- `TestABC_ID_1---0.html` → Test session: TestABC_ID_1

Each unique `<TestName>_<ID_X>` combination is treated as a separate test session.

### Frontend (Vue)

**`src/App.vue`** - Single-page application with:
- Reactive state: sessions, logEntries, bookmarks, filters (level, search)
- Sidebar with collapsible panels for Bookmarks and Test Sessions
- Custom pagination with page jump functionality
- Bookmark navigation that calculates target page considering active filters
- Row selection with multi-select support
- Debounced search (300ms delay)

Key interactions:
- `@tauri-apps/api/core`'s `invoke()` for Tauri commands
- `@tauri-apps/plugin-dialog`'s `open()` for directory picker
- Custom highlight animation when jumping to bookmark entries

### State Management

- **Rust**: `AppState` wraps `DatabaseManager` in `Mutex` for thread-safe access
- **Vue**: Reactive refs and computed properties (no Vuex/Pinia)
- Cross-filter consistency: `get_entry_page` applies same WHERE conditions as `get_log_entries`

## Important Implementation Notes

1. **Blocking operations**: HTML parsing and file I/O run in `tokio::task::spawn_blocking` to avoid blocking async runtime
2. **Session IDs**: Generated as `session_<sanitized_name>_<timestamp>` format
3. **Bookmark navigation**: Must account for active `levelFilter` and `searchTerm` when calculating target page
4. **File ordering**: Test log files sorted by numeric suffix after `---` for correct sequence
5. **Error handling**: Tauri commands return `Result<T, String>`; errors shown to user via alerts
6. **Database**: SQLite file is stored as `logterminator.db` in the same directory as the executable
7. **File filtering**: Only files matching the pattern `<TestName>_<ID_X>---<Y>.html` are processed
