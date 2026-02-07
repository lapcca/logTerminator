# UI Improvements Design - 2026-02-07

## Overview

This document describes the design for four UI/UX improvements to logTerminator:
1. Stack column iconization
2. Auto-bookmark MARKER entries
3. Message column formatting preservation
4. Log file output

## Feature 1: Stack Column Iconization

### Problem
The stack column displays full stack trace text, consuming significant horizontal space.

### Solution
Replace stack trace text with an icon. Show full stack trace in hover tooltip.

### Implementation Details

**Frontend Changes (`App.vue`)**
```vue
<el-table-column prop="stack" width="60" align="center">
  <template #default="{ row }">
    <el-tooltip v-if="row.stack" placement="left" :show-after="300" popper-class="stack-tooltip">
      <template #content>
        <div class="stack-trace">{{ row.stack }}</div>
      </template>
      <el-icon :size="18" class="stack-icon">
        <Warning />
      </el-icon>
    </el-tooltip>
  </template>
</el-table-column>
```

**CSS Styles**
```css
.stack-icon {
  cursor: pointer;
  color: var(--el-text-color-secondary);
  transition: color 0.2s;
}

.stack-icon:hover {
  color: var(--el-color-warning);
}

.stack-trace {
  font-family: 'Consolas', 'Monaco', monospace;
  white-space: pre-wrap;
  max-width: 600px;
  max-height: 400px;
  overflow: auto;
}
```

### Dependencies
- Element Plus `el-icon`, `el-tooltip`, `Warning` icon component

---

## Feature 2: Auto-bookmark MARKER Entries

### Problem
Users need to manually bookmark important MARKER entries. This is tedious for large sessions.

### Solution
Automatically bookmark all MARKER level entries (except those containing "###") when loading a new session.

### Implementation Details

**Backend Changes (`src-tauri/src/database.rs`)**
```rust
pub fn auto_bookmark_markers(conn: &Connection, session_id: i64) -> Result<Vec<Bookmark>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, message FROM log_entries
         WHERE session_id = ? AND level = 'MARKER'
         AND message NOT LIKE '%###%'"
    )?;

    let bookmarks = stmt
        .query_map([session_id], |row| {
            // Check if bookmark already exists
            // If not, create bookmark with full message as title
        })?
        .collect();

    Ok(bookmarks)
}
```

**Frontend Changes (`App.vue`)**
```javascript
const loadSession = async (sessionId) => {
  // Existing log loading logic
  await fetchLogs(sessionId);

  // New: auto-bookmark MARKER entries
  const autoBookmarks = await invoke('auto_bookmark_markers', { sessionId });
  bookmarks.value.push(...autoBookmarks);
};
```

### Bookmark Properties
- Title: Full message text
- Notes: Empty
- Color: Default (Element Plus primary blue)
- User can edit/delete like manual bookmarks

---

## Feature 3: Message Column Formatting Preservation

### Problem
Multi-line messages display as single line, losing readability.

### Solution
Preserve newline characters from original logs. Reserve extension points for future keyword highlighting.

### Implementation Details

**Frontend Changes (`App.vue`)**
```vue
<el-table-column prop="message" label="Message">
  <template #default="{ row }">
    <div class="message-cell">{{ formatMessage(row.message) }}</div>
  </template>
</el-table-column>
```

**CSS Styles**
```css
.message-cell {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
}
```

**New File: `src/utils/messageFormatter.js`**
```javascript
import { ref } from 'vue';

// Reserved for future keyword highlighting feature
const highlightRules = ref([]);

export function formatMessage(message) {
  // Current: direct return (CSS handles newlines via white-space: pre-wrap)
  // Future: apply highlightRules to wrap keywords with colored spans
  return message;
}

export function setHighlightRules(rules) {
  highlightRules.value = rules;
}
```

### Extension Points
Future enhancement: Add UI config panel for users to define keyword-color mappings.

---

## Feature 4: Log File Output

### Problem
Debug logs are only output to console, not persisted for troubleshooting.

### Solution
Output all console logs to a file in the same directory as the main executable.

### Implementation Details

**Dependencies (`src-tauri/Cargo.toml`)**
```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

**Initialization (`src-tauri/src/main.rs`)**
```rust
use log::info;
use std::fs::OpenOptions;

fn init_logging() -> std::io::Result<()> {
    let exe_path = std::env::current_exe()?
        .parent()
        .ok_or("Cannot get exe directory")?
        .to_path_buf();

    let log_file = exe_path.join("logterminator.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    Ok(())
}
```

**Usage Throughout Codebase**
Replace `println!`, `eprintln!` with appropriate log macros:

```rust
// Before
println!("Parsing file: {}", filename);

// After
log::info!("Parsing file: {}", filename);

// Error cases
log::error!("Failed to parse: {}", error);
```

### Log Levels
- `error!`: Critical failures
- `warn!`: Warning conditions
- `info!`: Informational messages
- `debug!`: Detailed debugging info
- `trace!`: Very detailed tracing

### Configuration
Set `RUST_LOG` environment variable to control verbosity:
- `RUST_LOG=info` (default in production)
- `RUST_LOG=debug` (development)

### File Management
- Single file: `logterminator.log`
- Location: Same directory as executable
- Growth: Unbounded (user can manually delete)

---

## Implementation Order

Recommended order (simplest to most complex):

1. **Feature 3** (Message formatting) - Pure frontend, CSS change
2. **Feature 1** (Stack icon) - Pure frontend, reuses tooltip pattern
3. **Feature 2** (Auto-bookmark) - Frontend + backend coordination
4. **Feature 4** (Log file) - Pure backend, independent

---

## Testing Considerations

- **Feature 1**: Verify tooltip positioning, text wrapping, hover states
- **Feature 2**: Test with sessions containing many MARKER entries, verify "###" exclusion
- **Feature 3**: Test with messages containing various newline patterns
- **Feature 4**: Verify log file creation, content, permissions on different OS platforms

---

## User Experience Impact

- **Reduced visual clutter**: Stack column narrower, table more readable
- **Improved workflow**: Automatic bookmarking saves manual effort
- **Better readability**: Multi-line messages display correctly
- **Better supportability**: Log files aid troubleshooting
