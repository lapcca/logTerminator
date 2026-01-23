# Auto-Bookmarking & Enhanced Stack Trace Display

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add automatic bookmarking for `###MARKER###` messages and parse Python stack traces into readable tables

**Architecture:**
- Auto-bookmarking: Post-processing step after session load, regex detection in backend
- Stack trace parsing: On-demand JavaScript parsing in frontend, display as collapsible table

**Tech Stack:**
- Backend: Rust (regex, existing bookmark API)
- Frontend: Vue 3 + Element Plus (el-table, el-collapse, el-popover)

---

## Feature 1: Auto-Bookmarking

### Detection Pattern
- Scan all log entries for messages matching: `###(.+?)###`
- Extract bookmark title: content between `###` markers
- Example: `###TEST START###` → title: `TEST START`

### Bookmark Properties
| Field | Value |
|-------|-------|
| `title` | Extracted text (e.g., `TEST START`) |
| `color` | `#409EFF` (Element Plus primary blue - distinct from manual bookmarks) |
| `notes` | Empty or optional auto-generated marker |
| `log_entry_id` | ID of the log entry containing the marker |

### Implementation Points
1. **Local files**: Add step in `parse_log_directory()` after entries inserted
2. **HTTP logs**: Add step in `fetch_logs_from_http()` after entries inserted
3. **Database**: Use existing `add_bookmark()` - no schema changes needed

### Edge Cases
- Multiple `###MARKER###` in one message → use first match
- Empty marker `######` → skip (invalid)
- Marker spans multiple lines → handle gracefully
- Duplicate markers → create separate bookmarks per entry

---

## Feature 2: Enhanced Stack Trace Display

### Parsing Logic

**Regex Pattern** (Python stack traces):
```
File "([^"]+)", line (\d+), in (\w+)\s*\n\s+(.+)
```

**Parsed Structure**:
```typescript
interface ParsedStackFrame {
  file: string;      // Full file path
  line: string;      // Line number as string
  function: string;  // Function name
  code: string;      // Source code line
}
```

**Parser Function** (Frontend):
```javascript
function parsePythonStackTrace(stackRaw: string): ParsedStackFrame[] {
  // Returns parsed frames or empty array if parsing fails
}
```

### Display Component

**UI Structure**:
```
Log Entry Table Row
└── Stack Column (clickable text)
    └── el-collapse-item (expandable)
        └── el-table (parsed stack frames)
```

**Table Columns**:
```javascript
const stackTableColumns = [
  { prop: 'file', label: '文件', minWidth: 300, showOverflowTooltip: true },
  { prop: 'line', label: '行号', width: 80, align: 'center' },
  { prop: 'function', label: '函数', width: 200, showOverflowTooltip: true },
  { prop: 'code', label: '代码', minWidth: 250, showOverflowTooltip: true }
]
```

### Interaction Design
1. **Default**: Stack trace shows collapsed (first line or "点击查看调用栈")
2. **Click**: Expands to show full parsed table
3. **Parse failure**: Falls back to displaying raw stack trace
4. **Empty stack**: Shows "-"

### Alternative: Popover Preview
- On hover: Show `el-popover` with first 3-5 frames
- Full view: Click to expand all frames inline

---

## Implementation Tasks

### Task 1: Backend - Add Bookmark Parsing Helper

**Files:**
- Create: `src-tauri/src/bookmark_utils.rs` (new module)
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Create helper function to find `###MARKER###` patterns
2. Return list of `(entry_id, title)` tuples
3. Call existing `add_bookmark` for each match

**Function Signature:**
```rust
pub fn find_auto_bookmark_markers(entries: &[LogEntry]) -> Vec<(i64, String)>
```

---

### Task 2: Backend - Integrate Auto-Bookmarking

**Files:**
- Modify: `src-tauri/src/lib.rs` (in `parse_log_directory` function)
- Modify: `src-tauri/src/http_log_fetcher.rs` (in `fetch_logs_from_http` function)

**Steps:**
1. After `insert_entries()` succeeds, call `find_auto_bookmark_markers()`
2. For each marker, call `db_manager.add_bookmark()` with blue color
3. Log count of auto-generated bookmarks

---

### Task 3: Frontend - Stack Trace Parser

**Files:**
- Create: `src/utils/stackParser.js` (new utility file)
- Modify: `src/App.vue`

**Steps:**
1. Create `parsePythonStackTrace()` function with regex
2. Add fallback for non-Python/invalid traces
3. Export for use in App.vue

**Regex Implementation:**
```javascript
const STACK_FRAME_REGEX = /File "([^"]+)", line (\d+), in (\S+)\s*\n\s+(.+)/g;
```

---

### Task 4: Frontend - Stack Trace Display Component

**Files:**
- Modify: `src/App.vue`

**Steps:**
1. Add `el-collapse` around stack trace cell content
2. Create `el-table` for parsed frames
3. Add click handler to toggle collapse
4. Style with proper column widths

**Template Changes:**
```vue
<el-table-column prop="stack" label="调用栈" width="250">
  <template #default="{ row }">
    <el-collapse @change="handleStackToggle(row.id)">
      <el-collapse-item :name="row.id">
        <template #title>
          <span>{{ getStackPreview(row.stack) }}</span>
        </template>
        <el-table :data="parseStack(row.stack)" size="small">
          <!-- columns here -->
        </el-table>
      </el-collapse-item>
    </el-collapse>
  </template>
</el-table-column>
```

---

### Task 5: Testing

**Backend Tests:**
- Test auto-bookmark detection with various patterns
- Test edge cases (empty, multiple, malformed markers)
- Test bookmark color is set correctly

**Frontend Tests:**
- Test stack trace parser with real Python traces
- Test fallback for non-Python traces
- Test UI expand/collapse behavior

---

## Success Criteria

1. ✅ All `###MARKER###` messages auto-bookmarked with blue color
2. ✅ Python stack traces parsed into 4-column table format
3. ✅ Stack traces collapsible/expandable on click
4. ✅ Non-Python traces display as-is without errors
5. ✅ No performance degradation on large sessions

---

## Notes

- **Performance**: Parse stack traces on-demand (frontend), not stored in database
- **Backward compatibility**: Old sessions without auto-bookmarks still work
- **Color choice**: Blue (`#409EFF`) matches Element Plus theme, distinct from yellow manual bookmarks
