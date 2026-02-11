# UI Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add log source history dropdown, session color grouping by directory, and simplify log level filter.

**Architecture:**
- Frontend changes in Vue component with reactive state management
- History storage using existing Tauri backend pattern
- Color generation via hash-based algorithm with HSL color space

**Tech Stack:** Vue 3, Element Plus, Tauri (Rust backend)

---

## Task 1: Add Log Source History Storage

**Files:**
- Create: `src-tauri/src/history.rs` (new module)
- Modify: `src-tauri/src/main.rs` - register history commands
- Modify: `src-tauri/Cargo.toml` - add dependencies if needed

**Step 1: Read existing storage pattern**

Read: `src-tauri/src/storage.rs`
Look for: How `last_log_directory.txt` is read/written

**Step 2: Create history module**

Create: `src-tauri/src/history.rs`

```rust
use std::fs;
use std::path::PathBuf;

/// Get the history file path
fn get_history_file() -> PathBuf {
    // Reuse same logic as storage.rs for config location
    // Usually in AppData or user home directory
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
```

**Step 3: Register Tauri commands**

Modify: `src-tauri/src/main.rs`

Add to `invoke_handler`:
```rust
#[tauri::command]
fn get_log_history() -> Vec<String> {
    history::get_recent_history(10)
}

#[tauri::command]
fn save_log_history_entry(entry: String) -> Result<(), String> {
    history::save_history(entry)
}
```

Add module import:
```rust
mod history;
```

Register in main:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    get_log_history,
    save_log_history_entry,
])
```

**Step 4: Test Rust code**

Run: `cargo test` (if tests exist) or `cargo check`
Expected: No compilation errors

**Step 5: Commit**

```bash
cd src-tauri
git add src/history.rs src/main.rs
git commit -m "feat: add log source history storage module"
```

---

## Task 2: Replace Input with Select for History Dropdown

**Files:**
- Modify: `src/App.vue` - replace el-input with el-select, add history state

**Step 1: Read current implementation**

Read: `src/App.vue` lines 1589-1597 (log source input)

**Step 2: Add reactive state for history**

Add in `<script setup>` section (around line 253):

```javascript
const logSourceHistory = ref([]) // History entries for dropdown
const historyLoaded = ref(false)  // Track if history has been loaded
```

**Step 3: Add function to load history**

Add after `loadSessions()` function:

```javascript
// Load log source history
async function loadLogSourceHistory() {
  try {
    const history = await invoke('get_log_history')
    logSourceHistory.value = history
    historyLoaded.value = true
  } catch (error) {
    console.error('Error loading log history:', error)
    logSourceHistory.value = []
  }
}
```

**Step 4: Call history load when dialog opens**

Modify: `handleSourceDialogOpened` function (around line 1426)

```javascript
async function handleSourceDialogOpened() {
  if (logSourceInputRef.value) {
    logSourceInputRef.value.focus()
  }
  // Load history when dialog opens
  await loadLogSourceHistory()
}
```

**Step 5: Update openLogSource to save history**

Modify: `openLogSource` function (around line 470)

After successful log load, add:

```javascript
// Save to history
try {
  await invoke('save_log_history_entry', { entry: input })
  // Refresh history
  await loadLogSourceHistory()
} catch (error) {
  console.error('Error saving to history:', error)
}
```

**Step 6: Replace el-input with el-select**

Modify: Template (around line 1589)

Replace:
```vue
<el-input
  ref="logSourceInputRef"
  v-model="logSourceInput"
  :placeholder="inputSourceType === 'url' ? '例如: http://logs.example.com/test-logs/' : '选择或输入本地文件夹路径，或输入 HTTP URL'"
  :prefix-icon="inputSourceType === 'url' ? Link : Folder"
  clearable
  size="large"
  class="log-source-input"
  @keyup.enter="handleSourceDialogEnter" />
```

With:
```vue
<el-select
  ref="logSourceInputRef"
  v-model="logSourceInput"
  filterable
  allow-create
  default-first-option
  :placeholder="inputSourceType === 'url' ? '例如: http://logs.example.com/test-logs/' : '选择或输入本地文件夹路径，或输入 HTTP URL'"
  :prefix-icon="inputSourceType === 'url' ? Link : Folder"
  clearable
  size="large"
  class="log-source-input"
  popper-class="log-source-history-dropdown"
  @keyup.enter="handleSourceDialogEnter">
  <el-option
    v-for="(item, index) in logSourceHistory"
    :key="index"
    :label="item"
    :value="item">
    <div class="history-option">
      <el-icon><component :is="inputSourceType === 'url' ? Link : Folder" /></el-icon>
      <span class="history-text-truncated">{{ truncatePath(item) }}</span>
    </div>
  </el-option>
</el-select>
```

**Step 7: Add truncate helper function**

Add in `<script setup>`:

```javascript
// Truncate path for display (show last ~30 chars with ...)
function truncatePath(path) {
  if (path.length <= 40) return path
  return '...' + path.slice(-37)
}
```

**Step 8: Add dropdown styling**

Add to `<style>` section:

```css
.log-source-history-dropdown .history-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.log-source-history-dropdown .history-text-truncated {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
```

**Step 9: Test in browser**

Run: `pnpm dev`

Expected:
- Click "打开日志源" button
- Click input field
- See dropdown with recent history (if any)
- Type new value and enter - should still work
- Select from history - fills input
- Click "打开日志源" - loads logs

**Step 10: Commit**

```bash
git add src/App.vue
git commit -m "feat: add history dropdown to log source input"
```

---

## Task 3: Add Session Color Grouping

**Files:**
- Modify: `src/App.vue` - add color generation and apply to session options

**Step 1: Read current session dropdown implementation**

Read: `src/App.vue` lines 1680-1709 (session selector)

**Step 2: Add color cache and generation function**

Add in `<script setup>` (around line 180):

```javascript
// Cache for session path colors
const pathColorCache = new Map()

// Simple string hash for color generation
function stringHash(str) {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32-bit integer
  }
  return Math.abs(hash)
}

// Generate pastel color from hash (HSL: high lightness for readability)
function generatePathColor(path) {
  const hash = stringHash(path)
  const hue = hash % 360 // 0-359
  return `hsl(${hue}, 70%, 90%)` // Pastel, light background
}

// Get cached color for a path
function getPathColor(path) {
  if (!pathColorCache.has(path)) {
    pathColorCache.set(path, generatePathColor(path))
  }
  return pathColorCache.get(path)
}
```

**Step 3: Apply color to session option template**

Modify: Session option template (around line 1690)

Change:
```vue
<div class="session-option-item">
```

To:
```vue
<div
  class="session-option-item"
  :style="{ backgroundColor: getPathColor(session.directory_path) }">
```

**Step 4: Ensure text remains readable on colored background**

The HSL color with 90% lightness ensures text is readable, but let's verify style doesn't interfere.

Check existing CSS for `.session-option-item` - ensure no conflicting background styles.

**Step 5: Test in browser**

Run: `pnpm dev`

Expected:
- Open session dropdown
- Sessions from same directory have same background color
- Sessions from different directories have different colors
- Text (session name, record count) is clearly readable
- Delete icon is still accessible

**Step 6: Commit**

```bash
git add src/App.vue
git commit -m "feat: add session color grouping by directory path"
```

---

## Task 4: Simplify Log Level Filter

**Files:**
- Modify: `src/App.vue` - remove closable tags, adjust width styles

**Step 1: Read current log level filter implementation**

Read: `src/App.vue` lines around the log level select (search for `levelFilter`)

**Step 2: Find and remove closable functionality**

Look for `el-tag` with `closable` attribute and `@close` event handler in log level filter.

If found, remove `closable` and `@close` attributes.

**Step 3: Adjust width styles**

Modify: CSS for `.level-filter-select` (around line 2112)

Change:
```css
.level-filter-select {
  flex: 0 0 auto !important;
  width: calc(100vw - 1030px) !important;
  min-width: 200px !important;
  max-width: none !important;
}
```

To:
```css
.level-filter-select {
  flex: 0 1 auto !important;
  width: auto !important;
  min-width: 120px !important;
  max-width: 300px !important;
}
```

Also update sidebar-collapsed variant (around line 2120):

```css
.sidebar-collapsed .level-filter-select {
  width: auto !important;
  max-width: 300px !important;
}
```

**Step 4: Verify search input gets more space**

The search input should expand into freed space due to flex layout.

**Step 5: Test in browser**

Run: `pnpm dev`

Expected:
- Log level selector is narrower
- Can still select/deselect multiple log levels
- No 'x' close buttons on selected tags
- Search input is wider
- All filtering functionality works as before

**Step 6: Commit**

```bash
git add src/App.vue
git commit -m "refactor: simplify log level filter, reduce width"
```

---

## Task 5: Final Testing and Cleanup

**Files:**
- Test all changes together
- Update design doc with any deviations

**Step 1: Run all tests**

Run: `pnpm test run`

Expected: All existing tests pass (13 tests)

**Step 2: Manual testing checklist**

- [ ] Open log source dialog - dropdown shows history (if any exists)
- [ ] Select from history - fills input
- [ ] Type new path - works normally
- [ ] Open logs - new entry added to history
- [ ] Session dropdown - colors group by directory
- [ ] Refresh page - colors stay consistent
- [ ] Log level filter - multi-select works
- [ ] Log level filter - no close buttons
- [ ] Search input - has more space

**Step 3: Fix any issues**

Address any bugs found during testing.

**Step 4: Update design doc if needed**

If implementation deviated from design, note the reasons in: `docs/plans/2026-02-10-ui-improvements-design.md`

**Step 5: Final commit**

```bash
git add docs/plans/2026-02-10-ui-improvements-design.md
git commit -m "docs: update design doc with implementation notes"
```

---

## Summary

This plan implements three UI improvements:

1. **Log source history** - Replaces input with select, stores last 10 entries
2. **Session color grouping** - Hash-based pastel colors by directory path
3. **Log level simplification** - Removes close tags, reduces width

Total commits: ~5 (one per task plus cleanup)

Testing: Automated tests + manual checklist
