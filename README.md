# logTerminator

A powerful desktop log viewer application built with Tauri and Vue.js, designed to parse, view, filter, and bookmark HTML test logs with ease.

![logTerminator](src-tauri/icons/icon.svg)

## Features

- **HTML Log Parsing**: Intelligently parses HTML test logs with automatic test session detection
- **Advanced Filtering**: Filter logs by level (INFO, ERROR, WARN, etc.) with multi-select support
- **Full-Text Search**: Search across all log entries with real-time highlighting
- **JSON Viewer**: Syntax-highlighted JSON display with collapsible sections for better readability
- **Bookmarking System**: Save important log entries with custom titles, notes, and color coding
- **Stack Trace Display**: Enhanced stack trace visualization with automatic parsing
- **Draggable Tooltips**: Hover over messages to view detailed tooltips with search and copy functionality
- **Auto-bookmarking**: Automatically bookmark entries matching specific patterns
- **Pagination**: Efficient pagination with page jump functionality for large log sets
- **Row Selection**: Multi-select support for batch operations

## Screenshot

<!-- Add your screenshot here -->
<img src="docs/screenshot.png" alt="logTerminator Screenshot" width="800"/>

## Tech Stack

### Frontend
- **Vue 3** - Progressive JavaScript framework with Composition API
- **Element Plus** - Vue 3 UI component library
- **Vite** - Fast build tool and development server

### Backend
- **Tauri 2** - Cross-platform desktop application framework
- **Rust** - Systems programming language for performance and safety
- **SQLite** - Embedded database for log storage
- **Scraper** - HTML parsing with CSS selectors
- **Tokio** - Asynchronous runtime

## Installation

### Prerequisites

- **Node.js** (v18 or higher)
- **pnpm** (v10 or higher)
- **Rust** (latest stable)
- **System Dependencies**:
  - **Windows**: WebView2 Runtime (usually pre-installed)
  - **macOS**: Xcode Command Line Tools
  - **Linux**: WebView2 development packages

### Build from Source

```bash
# Clone the repository
git clone https://github.com/lapcca/logTerminator.git
cd logTerminator

# Install dependencies
pnpm install

# Run development server
pnpm tauri dev

# Build production binary
pnpm tauri build
```

## Usage

### Opening Logs

1. Launch logTerminator
2. Click "Open Directory" or use the "Open Log Source" dialog
3. Select a folder containing your HTML test logs
4. Logs will be automatically parsed and organized by test session

### Navigating Logs

- **Session Selector**: Choose from detected test sessions in the dropdown
- **Log Level Filter**: Use the dropdown to filter by one or more log levels
- **Search**: Enter keywords to search across all log entries
- **Pagination**: Navigate through results using pagination controls or jump to a specific page

### Using Bookmarks

- **Create Bookmark**: Click the bookmark icon on any log entry
- **Add Details**: Enter title, notes, and select a color
- **Navigate**: Click bookmarked items in the sidebar to jump to that entry
- **Delete**: Remove bookmarks from the sidebar

### JSON Viewing

- Hover over messages containing JSON data
- Click "JSON" tab in the tooltip to view formatted JSON
- Use `[-]/[+]` buttons to expand/collapse objects and arrays
- Search within JSON using the search box
- Copy JSON to clipboard with the "Copy JSON" button

## Test Log Format

logTerminator expects HTML test logs following this naming convention:

```
<TestName>_<ID_X>---<Y>.html
```

- `<TestName>`: Test identifier (e.g., "TestEnableTcpdump", "TestABC")
- `<ID_X>`: Test instance ID (e.g., "ID_1")
- `<Y>`: Sequential file number (e.g., 0, 1, 2...)

**Example**:
- `TestEnableTcpdump_ID_1---0.html` → Test session: TestEnableTcpdump_ID_1
- `TestABC_ID_1---0.html` → Test session: TestABC_ID_1

**Files that don't match this pattern are ignored**, such as:
- `MainRollup.html`
- `summary.html`
- `TestWithoutID---0.html` (missing `_ID_` pattern)

Each unique `<TestName>_<ID_X>` combination is treated as a separate test session.

### Expected HTML Structure

```html
<table>
  <tr>
    <td class="date">2024-01-01 12:00:00</td>
    <td class="level">INFO</td>
    <td class="message">Log message here</td>
    <td class="stack" hidden>Stack trace here</td>
  </tr>
</table>
```

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm dev

# Run Rust tests
cd src-tauri
cargo test

# Run Rust tests with output
cargo test -- --nocapture

# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Build production
pnpm build
pnpm tauri build
```

## Project Structure

```
logTerminator/
├── src/                    # Vue.js frontend source
│   ├── components/         # Vue components
│   │   ├── MessageTooltip.vue
│   │   └── LogTerminatorIcon.vue
│   ├── utils/              # Utility functions
│   │   └── jsonViewer.js   # JSON parsing and syntax highlighting
│   └── App.vue             # Main application component
├── src-tauri/              # Rust backend source
│   ├── src/
│   │   ├── lib.rs          # Tauri command handlers
│   │   ├── log_parser.rs   # HTML log parsing
│   │   └── database.rs     # SQLite operations
│   ├── icons/              # Application icons
│   └── Cargo.toml          # Rust dependencies
└── CLAUDE.md               # Project documentation for contributors
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) - Vue language support
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) - Tauri integration
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust language support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI components by [Element Plus](https://element-plus.org/)
- Icons from [Element Plus Icons](https://element-plus.org/en-US/component/icon.html)

## Contact

Jan - [@lapcca](https://github.com/lapcca)

Project Link: [https://github.com/lapcca/logTerminator](https://github.com/lapcca/logTerminator)
