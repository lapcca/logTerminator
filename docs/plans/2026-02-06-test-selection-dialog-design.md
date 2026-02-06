# Test Selection Dialog Design

## Overview

Add a test selection dialog that allows users to choose which test sessions to load after selecting a log directory or HTTP URL.

## User Flow

1. User inputs/selects directory or URL
2. System scans and displays list of available tests
3. User selects which tests to load (multi-select)
4. Only selected tests are loaded

## Data Structures

### Rust Backend

```rust
struct ScanResult {
    test_name: String,              // Test name
    file_count: usize,              // Number of associated HTML files
    is_loaded: bool,                // Whether already in database
    existing_session_id: Option<String>,  // Existing session_id if loaded
    estimated_entries: Option<usize>,     // Entry count if loaded
}
```

### New Tauri Commands

```rust
// Scan directory without loading
#[tauri::command]
async fn scan_log_directory(
    directory_path: String,
) -> Result<Vec<ScanResult>, String>

// Scan HTTP URL without loading
#[tauri::command]
async fn scan_log_http_url(
    url: String,
) -> Result<Vec<ScanResult>, String>
```

### Modified Commands

```rust
// Modified to accept selected tests
#[tauri::command]
async fn parse_log_directory(
    directory_path: String,
    selected_tests: Option<Vec<String>>,  // NEW: List of test names to load
) -> Result<Vec<String>, String>

// Modified to accept selected tests
#[tauri::command]
async fn parse_log_http_url(
    url: String,
    selected_tests: Option<Vec<String>>,  // NEW: List of test names to load
) -> Result<Vec<String>, String>
```

## Frontend Component

### TestSelectionDialog.vue

```
Test Selection Dialog
├── Title: "选择要加载的Test会话"
├── Content
│   ├── Top hint: "找到 X 个test会话，请选择要加载的项目"
│   ├── Test list (el-checkbox-group)
│   │   └── Each test item shows:
│   │       ├── Checkbox
│   │       ├── Test name
│   │       ├── File count badge
│   │       └── "已加载" label (if applicable)
│   └── Bottom hint: "已选择的test将覆盖现有数据"
└── Footer
    ├── Selected count
    ├── Cancel button
    └── Confirm button (disabled when no selection)
```

### State Management

```javascript
const testSelectionDialog = ref({
  visible: false,
  directoryPath: '',
  isHttp: false,
  scanResults: [],
  selectedTests: [],
  loading: false
})
```

## Design Decisions

| Question | Decision |
|----------|----------|
| Information to display | Basic info (name + file count) |
| Default selection | All unchecked (user must manually select) |
| Empty selection handling | Disable confirm button |
| Already loaded tests | Show with "已加载" marker, allow reload |
| HTTP loading | Same flow as local directory |

## Error Handling

### Scan Phase Errors

| Scenario | Handling |
|----------|----------|
| Directory not found | Error: "目录不存在，请检查路径" |
| No permission | Error: "无权限访问该目录" |
| No test files | Error: "未找到符合格式的test日志文件" |
| HTTP connection failed | Error: "无法连接到服务器，请检查URL" |
| HTTP timeout | Error: "连接超时，服务器可能响应缓慢" |

### Load Phase Errors

| Scenario | Handling |
|----------|----------|
| Partial load failure | Warning: "X个test加载成功，Y个test加载失败" |
| All load failure | Error: "所有test加载失败，请重试" |
| File parse error | Log and skip, continue processing |
| Database error | Error: "数据库错误，无法保存日志" |

## Implementation Plan

### Backend Changes
1. Add `scan_log_directory` command
2. Add `scan_log_http_url` command
3. Modify `parse_log_directory` to accept `selected_tests` parameter
4. Modify `parse_log_http_url` to accept `selected_tests` parameter
5. Add `ScanResult` struct

### Frontend Changes
1. Create `TestSelectionDialog.vue` component
2. Modify `openLogSource()` to call scan API first
3. Modify `loadFromDirectory()` to pass selected test list
4. Modify `loadFromHttpUrl()` to pass selected test list

## Test Strategy

### Unit Tests (Rust)
- Scan empty directory
- Scan directory with valid tests
- Scan directory with mixed files
- Detect already loaded tests
- Selective loading
- HTTP URL scanning

### Manual Test Scenarios
- Select local directory with 3 tests, choose 2 to load
- Select directory where all tests are loaded, choose 1 to reload
- Input HTTP URL, scan and select partial tests
- No test selected → confirm button disabled
- Cancel selection dialog → return to log source dialog
