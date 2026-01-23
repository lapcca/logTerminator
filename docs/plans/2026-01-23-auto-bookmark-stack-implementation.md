# Auto-Bookmarking & Stack Trace Display Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add automatic bookmarking for `###MARKER###` messages and parse Python stack traces into readable collapsible tables

**Architecture:**
- Auto-bookmarking: Post-processing step after session load using regex detection, calling existing bookmark API with blue color
- Stack trace parsing: On-demand JavaScript regex parsing in frontend, displayed via Element Plus collapse and table components

**Tech Stack:** Rust (regex, database), Vue 3 Composition API, Element Plus (el-collapse, el-table, el-popover), JavaScript regex

---

## Task 1: Create Auto-Bookmark Detection Utility

**Files:**
- Create: `src-tauri/src/bookmark_utils.rs`

**Step 1: Create the module file with regex pattern**

```rust
use crate::log_parser::LogEntry;
use regex::Regex;

/// Find all ###MARKER### patterns in log entries
/// Returns list of (entry_id, marker_title) tuples
pub fn find_auto_bookmark_markers(entries: &[LogEntry]) -> Vec<(i64, String)> {
    let mut bookmarks = Vec::new();

    // Regex to match ###TEXT### pattern
    let marker_regex = Regex::new(r"###(.+?)###").unwrap();

    for entry in entries {
        if let Some(id) = entry.id {
            // Find all matches in the message
            for capture in marker_regex.captures_iter(&entry.message) {
                if let Some(marker_text) = capture.get(1) {
                    let title = marker_text.as_str().to_string();

                    // Skip empty markers like ######
                    if !title.trim().is_empty() {
                        bookmarks.push((id, title));
                    }
                }
            }
        }
    }

    bookmarks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_single_marker() {
        let entries = vec![
            LogEntry {
                id: Some(1),
                test_session_id: "test".to_string(),
                file_path: "test.log".to_string(),
                file_index: 0,
                timestamp: "2024-01-01 00:00:00".to_string(),
                level: "INFO".to_string(),
                stack: "".to_string(),
                message: "###TEST START###".to_string(),
                line_number: 1,
                created_at: None,
            }
        ];

        let result = find_auto_bookmark_markers(&entries);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[0].1, "TEST START");
    }

    #[test]
    fn test_skip_empty_marker() {
        let entries = vec![
            LogEntry {
                id: Some(1),
                test_session_id: "test".to_string(),
                file_path: "test.log".to_string(),
                file_index: 0,
                timestamp: "2024-01-01 00:00:00".to_string(),
                level: "INFO".to_string(),
                stack: "".to_string(),
                message: "######".to_string(),
                line_number: 1,
                created_at: None,
            }
        ];

        let result = find_auto_bookmark_markers(&entries);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_multiple_markers_same_entry() {
        let entries = vec![
            LogEntry {
                id: Some(1),
                test_session_id: "test".to_string(),
                file_path: "test.log".to_string(),
                file_index: 0,
                timestamp: "2024-01-01 00:00:00".to_string(),
                level: "INFO".to_string(),
                stack: "".to_string(),
                message: "###START### doing something ###END###".to_string(),
                line_number: 1,
                created_at: None,
            }
        ];

        let result = find_auto_bookmark_markers(&entries);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, "START");
        assert_eq!(result[1].1, "END");
    }
}
```

**Step 2: Add module declaration to lib.rs**

Add to `src-tauri/src/lib.rs` at the top with other module declarations:

```rust
mod bookmark_utils;
```

**Step 3: Add regex dependency**

Add to `src-tauri/Cargo.toml` in `[dependencies]` section:

```toml
regex = "1"
```

**Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test find_auto_bookmark_markers --lib`
Expected: All 3 tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/bookmark_utils.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add auto-bookmark detection utility with regex

- Extracts ###MARKER### patterns from log messages
- Skips empty markers
- Supports multiple markers per entry
- Includes comprehensive unit tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Integrate Auto-Bookmarking into Local File Parsing

**Files:**
- Modify: `src-tauri/src/lib.rs` (parse_log_directory function)

**Step 1: Find the insertion point in parse_log_directory**

Look for the section where entries are inserted, after `db_manager.insert_entries(&all_entries)` succeeds. Around line 90-100.

**Step 2: Add auto-bookmarking logic after insert_entries**

```rust
// After this line in parse_log_directory function:
db_manager
    .insert_entries(&all_entries)
    .map_err(|e| format!("Failed to insert entries: {}", e))?;

// Add this code:
// Auto-generate bookmarks for ###MARKER### patterns
use crate::bookmark_utils::find_auto_bookmark_markers;
use crate::log_parser::Bookmark;

let auto_markers = find_auto_bookmark_markers(&all_entries);
let mut auto_bookmark_count = 0;

for (entry_id, title) in auto_markers {
    let bookmark = Bookmark {
        id: None,
        log_entry_id: entry_id,
        title: Some(title),
        notes: None,
        color: Some("#409EFF".to_string()), // Blue for auto-bookmarks
        created_at: Some(chrono::Utc::now()),
    };

    if let Ok(_) = db_manager.add_bookmark(&bookmark) {
        auto_bookmark_count += 1;
    }
}

if auto_bookmark_count > 0 {
    println!(
        "Auto-generated {} bookmarks from ###MARKER### patterns",
        auto_bookmark_count
    );
}
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: No compilation errors

**Step 4: Test with real data**

Run: `cargo test`
Expected: All existing tests still pass

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: auto-generate bookmarks for ###MARKER### patterns in local files

- Scans all entries after insertion
- Creates blue bookmarks (#409EFF) for each marker
- Logs count of auto-generated bookmarks

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Integrate Auto-Bookmarking into HTTP Log Loading

**Files:**
- Modify: `src-tauri/src/http_log_fetcher.rs` (fetch_logs_from_http function)

**Step 1: Find the insertion point in fetch_logs_from_http**

Look for the section after `db_manager.insert_entries(&all_entries)` succeeds. Around line 265-268.

**Step 2: Add auto-bookmarking logic (same as Task 2)**

```rust
// After this line in fetch_logs_from_http function:
db_manager
    .insert_entries(&all_entries)
    .map_err(|e| HttpFetchError::ParseError(format!("Failed to insert entries: {}", e)))?;

// Add this code:
// Auto-generate bookmarks for ###MARKER### patterns
use crate::bookmark_utils::find_auto_bookmark_markers;
use crate::log_parser::Bookmark;

let auto_markers = find_auto_bookmark_markers(&all_entries);
let mut auto_bookmark_count = 0;

for (entry_id, title) in auto_markers {
    let bookmark = Bookmark {
        id: None,
        log_entry_id: entry_id,
        title: Some(title),
        notes: None,
        color: Some("#409EFF".to_string()), // Blue for auto-bookmarks
        created_at: Some(chrono::Utc::now()),
    };

    if let Ok(_) = db_manager.add_bookmark(&bookmark) {
        auto_bookmark_count += 1;
    }
}

if auto_bookmark_count > 0 {
    println!(
        "Auto-generated {} bookmarks from ###MARKER### patterns",
        auto_bookmark_count
    );
}
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: No compilation errors

**Step 4: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src-tauri/src/http_log_fetcher.rs
git commit -m "feat: auto-generate bookmarks for ###MARKER### in HTTP logs

- Same logic as local file parsing
- Creates blue bookmarks for markers in HTTP-loaded logs
- Maintains consistency across all log sources

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Create Stack Trace Parser Utility in Frontend

**Files:**
- Create: `src/utils/stackParser.js`

**Step 1: Create the parser utility file**

```javascript
/**
 * Parse a Python stack trace into structured frames
 * @param {string} stackRaw - Raw stack trace string
 * @returns {Array<{file: string, line: string, function: string, code: string}>}
 */
export function parsePythonStackTrace(stackRaw) {
  if (!stackRaw || stackRaw.trim() === '') {
    return []
  }

  const frames = []

  // Regex pattern for Python stack frames:
  // File "path", line N, in function
  //     code line
  const frameRegex = /File "([^"]+)", line (\d+), in (\S+)\s*\n\s+(.+)/g

  let match
  while ((match = frameRegex.exec(stackRaw)) !== null) {
    frames.push({
      file: match[1],
      line: match[2],
      function: match[3],
      code: match[4].trim()
    })
  }

  return frames
}

/**
 * Get a preview of the stack trace (first line or truncated)
 * @param {string} stackRaw - Raw stack trace string
 * @returns {string} Preview text
 */
export function getStackPreview(stackRaw) {
  if (!stackRaw || stackRaw.trim() === '') {
    return '-'
  }

  const lines = stackRaw.trim().split('\n')
  const firstLine = lines[0].trim()

  // Truncate if too long
  if (firstLine.length > 60) {
    return firstLine.substring(0, 60) + '...'
  }

  return firstLine
}

/**
 * Check if stack trace looks like a Python traceback
 * @param {string} stackRaw - Raw stack trace string
 * @returns {boolean}
 */
export function isPythonStackTrace(stackRaw) {
  if (!stackRaw) {
    return false
  }

  // Check for Python traceback indicators
  return /File "[^"]+", line \d+, in \S+/.test(stackRaw)
}
```

**Step 2: Verify syntax**

Run: `node -c src/utils/stackParser.js`
Expected: No syntax errors

**Step 3: Commit**

```bash
git add src/utils/stackParser.js
git commit -m "feat: add Python stack trace parser utility

- Parses Python stack frames into structured data
- Extracts file, line, function, code
- Includes helper functions for preview and detection
- Returns empty array for non-Python or empty traces

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Add Stack Trace Table Component to Frontend

**Files:**
- Modify: `src/App.vue`

**Step 1: Import the parser utility**

Add at the top of the `<script setup>` section:

```javascript
import { parsePythonStackTrace, getStackPreview, isPythonStackTrace } from './utils/stackParser.js'
```

**Step 2: Add stack table columns definition**

Add after the existing `headers` definition (around line 135):

```javascript
// Stack trace table columns
const stackTableColumns = [
  { prop: 'file', label: '文件', minWidth: 300, showOverflowTooltip: true },
  { prop: 'line', label: '行号', width: 80, align: 'center' },
  { prop: 'function', label: '函数', width: 200, showOverflowTooltip: true },
  { prop: 'code', label: '代码', minWidth: 250, showOverflowTooltip: true }
]
```

**Step 3: Add parsing function**

Add after the `truncateText` function (around line 400):

```javascript
// Parse stack trace for display
function parseStack(stackRaw) {
  if (!stackRaw) {
    return []
  }

  // Try to parse as Python stack trace
  if (isPythonStackTrace(stackRaw)) {
    return parsePythonStackTrace(stackRaw)
  }

  // For non-Python traces, return as single "raw" frame
  return [{
    file: 'Raw stack trace',
    line: '-',
    function: '-',
    code: stackRaw
  }]
}

// Track expanded stack rows
const expandedStackRows = ref(new Set())
```

**Step 4: Add toggle function for stack expansion**

```javascript
// Toggle stack trace expansion
function toggleStackExpansion(entryId) {
  if (expandedStackRows.value.has(entryId)) {
    expandedStackRows.value.delete(entryId)
  } else {
    expandedStackRows.value.add(entryId)
  }
}

// Check if stack is expanded
function isStackExpanded(entryId) {
  return expandedStackRows.value.has(entryId)
}
```

**Step 5: Update the stack column template**

Find the stack column in the el-table and replace it with:

```vue
<el-table-column
  prop="stack"
  label="调用栈"
  width="280">
  <template #default="{ row }">
    <div v-if="row.stack" class="stack-trace-container">
      <!-- Stack preview (clickable) -->
      <div
        class="stack-preview"
        @click="toggleStackExpansion(row.id)"
        :class="{ 'stack-clickable': true }">
        <el-icon :class="{ 'stack-expanded': isStackExpanded(row.id) }">
          <ArrowRight />
        </el-icon>
        <span>{{ getStackPreview(row.stack) }}</span>
      </div>

      <!-- Expanded stack table -->
      <el-collapse-transition>
        <div v-show="isStackExpanded(row.id)" class="stack-table-wrapper">
          <el-table
            :data="parseStack(row.stack)"
            :columns="stackTableColumns"
            size="small"
            :show-header="true"
            stripe
            class="stack-table">
            <el-table-column
              prop="file"
              label="文件"
              min-width="300"
              show-overflow-tooltip />
            <el-table-column
              prop="line"
              label="行号"
              width="80"
              align="center" />
            <el-table-column
              prop="function"
              label="函数"
              width="200"
              show-overflow-tooltip />
            <el-table-column
              prop="code"
              label="代码"
              min-width="250"
              show-overflow-tooltip />
          </el-table>
        </div>
      </el-collapse-transition>
    </div>
    <span v-else>-</span>
  </template>
</el-table-column>
```

**Step 6: Add CSS styles**

Add to the `<style scoped>` section:

```css
.stack-trace-container {
  width: 100%;
}

.stack-preview {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.stack-preview:hover {
  background-color: #f5f7fa;
}

.stack-clickable {
  cursor: pointer;
}

.stack-preview .el-icon {
  transition: transform 0.2s;
}

.stack-expanded {
  transform: rotate(90deg);
}

.stack-table-wrapper {
  margin-top: 8px;
  padding: 0 12px 12px 12px;
  background-color: #f5f7fa;
  border-radius: 4px;
}

.stack-table {
  font-size: 12px;
}

.stack-table .el-table__cell {
  padding: 4px 0;
}
```

**Step 7: Add ArrowRight icon import**

Add to the imports from `@element-plus/icons-vue`:

```javascript
import {
  Document, Folder, FolderOpened, Delete, Refresh, Search,
  ArrowRight  // Add this
} from '@element-plus/icons-vue'
```

**Step 8: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 9: Commit**

```bash
git add src/App.vue src/utils/stackParser.js
git commit -m "feat: add collapsible stack trace table display

- Parses Python stack traces into 4-column table
- Shows file, line, function, code columns
- Click to expand/collapse stack traces
- Fallback to raw display for non-Python traces
- Includes smooth animations and hover effects

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Add Color Legend for Bookmark Types

**Files:**
- Modify: `src/App.vue`

**Step 1: Add bookmark legend in bookmarks panel**

Find the bookmarks panel section and add a legend after the header. Add this after the bookmarks panel header (around line 880):

```vue
<!-- Bookmark color legend -->
<div class="bookmark-legend">
  <div class="legend-item">
    <span class="legend-color legend-auto"></span>
    <span class="legend-label">自动书签 (###MARKER###)</span>
  </div>
  <div class="legend-item">
    <span class="legend-color legend-manual"></span>
    <span class="legend-label">手动书签</span>
  </div>
</div>
```

**Step 2: Add CSS styles for the legend**

```css
.bookmark-legend {
  display: flex;
  gap: 16px;
  padding: 8px 12px;
  margin-bottom: 8px;
  background-color: #f5f7fa;
  border-radius: 4px;
  font-size: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-color {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  border: 1px solid #dcdfe6;
}

.legend-auto {
  background-color: #409EFF;
}

.legend-manual {
  background-color: #E6A23C;
}

.legend-label {
  color: #606266;
}
```

**Step 3: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: add color legend for bookmark types

- Shows blue for auto-generated bookmarks
- Shows orange for manual bookmarks
- Helps users distinguish bookmark sources

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Integration Testing

**Files:**
- No file changes - manual testing

**Step 1: Test auto-bookmarking with local files**

1. Run the app: `pnpm tauri dev`
2. Load a local directory with log files containing `###TEST###` patterns
3. Verify blue bookmarks appear automatically
4. Check bookmarks panel shows auto-generated bookmarks
5. Verify bookmark titles are extracted correctly (without ###)

**Step 2: Test auto-bookmarking with HTTP logs**

1. Load logs from HTTP URL with `###MARKER###` patterns
2. Verify blue bookmarks appear
3. Check bookmark count in console output

**Step 3: Test stack trace parsing**

1. Load session with Python stack traces
2. Click on stack trace preview
3. Verify table expands showing parsed frames
4. Check all 4 columns display correctly
5. Test with non-Python traces (should show raw format)

**Step 4: Test empty/edge cases**

1. Test with empty stack traces (should show "-")
2. Test with malformed traces (should show raw)
3. Test with very long file paths (tooltip should work)

**Step 5: Verify no performance regression**

1. Load a large session (1000+ entries)
2. Check UI remains responsive
3. Verify expand/collapse is smooth

**Step 6: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests pass

**Step 7: Build production version**

Run: `pnpm tauri build`
Expected: Build succeeds, installer created

**Step 8: Final commit if needed**

If any fixes were needed during testing:

```bash
git add .
git commit -m "fix: [description of fixes found during testing]

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Summary

This implementation plan adds:
1. **Auto-bookmarking**: Scans for `###MARKER###` patterns, creates blue bookmarks automatically
2. **Stack trace parsing**: Parses Python traces into readable 4-column tables
3. **Collapsible UI**: Click to expand/collapse stack traces with smooth animations
4. **Color legend**: Helps users distinguish auto vs manual bookmarks

**Total estimated steps**: 35 small steps across 7 tasks
**Testing**: Unit tests for regex detection, manual testing for UI components
