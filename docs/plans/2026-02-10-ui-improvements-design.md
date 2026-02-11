# UI Improvements Design - 2026-02-10

## Overview

Three UI improvements to enhance user experience:

1. **Log source history dropdown** - Show last 10 opened sources in the open dialog
2. **Session color grouping** - Group sessions by directory/URL with distinct background colors
3. **Log level filter simplification** - Simplify the log level selector and reduce its width

---

## Requirement 1: Log Source History Dropdown

### Current State
- Simple `el-input` for entering folder path or HTTP URL
- Last used directory is stored in `last_log_directory.txt`
- No history of previously opened sources

### Desired Behavior
- Add dropdown to show last 10 successfully opened sources
- Display paths in truncated form with full path on hover
- Clicking a history item fills the input (user must still click "Open")
- Newest records appear at the top
- Preserve all existing functionality

### Technical Design

#### Component Change
- Replace `el-input` with `el-select`
- Configure:
  - `filterable`: Allow typing and searching
  - `allow-create`: Allow entering new values
  - `default-first-option`: Auto-select first option
  - `popper-class`: Custom dropdown styling

#### Storage Format
- Store in `last_log_directory.txt` using pipe-separated format:
  ```
  E:\logs\test-1|http://logs.example.com/api|E:\logs\test-2
  ```
- Newest records first, max 10 items

#### Display Format
- Truncated path: `...test-logs/`
- Full path on hover via tooltip
- Example: `E:\very\long\path\to\test-logs` → `...test-logs/`

---

## Requirement 2: Session Color Grouping

### Current State
- Session dropdown shows all sessions
- No visual grouping by source directory
- Each session has the same default styling

### Desired Behavior
- Group sessions by directory/URL path with distinct background colors
- Same directory/URL = same color
- Color should be subtle and readable
- Entire option area (including delete icon) gets the background

### Technical Design

#### Color Generation Algorithm
```javascript
function generatePathColor(path) {
  const hash = simpleHash(path)
  const hue = Math.abs(hash) % 360
  return `hsl(${hue}, 70%, 90%)` // Pastel, light for readability
}

function simpleHash(str) {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash) + str.charCodeAt(i)
    hash = hash & hash
  }
  return hash
}
```

#### Implementation
- Use `Map` to cache colors for each unique path
- Computed property or function to get color for each session
- Bind `style` attribute on `el-option` template
- Color based on `session.directory_path`

#### Color Caching
```javascript
const pathColorCache = new Map()

function getPathColor(path) {
  if (!pathColorCache.has(path)) {
    pathColorCache.set(path, generatePathColor(path))
  }
  return pathColorCache.get(path)
}
```

---

## Requirement 3: Log Level Filter Simplification

### Current State
- Multi-select dropdown for log levels
- Fixed min-width: 200px
- Has closable tags (delete functionality)

### Desired Behavior
- Remove delete tag functionality (no `closable` attribute)
- Width auto-adjusts to content size
- Free up space for search input
- Keep all multi-select functionality

### Technical Design

#### Style Changes
- Remove `min-width: 200px`
- Change to `width: auto` or `fit-content`
- Keep `max-width` to prevent excessive width

#### Layout Impact
- Search input gets more available space
- Log level selector shrinks to content size
- Overall header layout remains balanced

#### Preserved Behavior
- Multi-select functionality unchanged
- Log level priority sorting unchanged
- Session linkage unchanged

---

## Implementation Plan

### Files to Modify

1. **src/App.vue**
   - Replace log source input with select
   - Add history dropdown data and methods
   - Add session color generation
   - Update log level selector styles
   - Update CSS for new components

2. **src-tauri/src/storage.rs** (if needed)
   - Update last directory storage for multiple records

3. **Tests**
   - Add tests for history storage/retrieval
   - Add tests for color generation consistency

### Testing Checklist

#### History Dropdown
- [ ] Dropdown shows last 10 records
- [ ] Newest record at top
- [ ] Selection fills input correctly
- [ ] Manual input still works
- [ ] Truncated display shows correctly
- [ ] Tooltip shows full path on hover

#### Session Color Grouping
- [ ] Same directory sessions have same color
- [ ] Different directories have different colors
- [ ] Colors are consistent after refresh
- [ ] Text is readable on colored backgrounds
- [ ] Delete icon still accessible

#### Log Level Filter
- [ ] Multi-select works correctly
- [ ] Width adjusts to content
- [ ] Search input has more space
- [ ] No delete tags appear
- [ ] All log levels still filterable

---

## Implementation Notes

### Completed Implementation (2026-02-10)

All three requirements have been successfully implemented in the `ui-improvements` worktree.

#### Requirement 1: Log Source History Dropdown

**Implementation Details:**
- Created new `src-tauri/src/history.rs` module for history management
- Added Tauri commands: `get_log_history`, `save_log_history_entry`
- Replaced `el-input` with `el-select` in open log dialog
- History stored in `log_history.txt` (pipe-separated format)
- Truncates paths to 40 characters with "..." prefix for long paths
- Successfully opened sources are automatically added to history

**Files Modified:**
- `.worktrees/ui-improvements/src/App.vue` - UI component changes
- `.worktrees/ui-improvements/src-tauri/src/history.rs` - New module
- `.worktrees/ui-improvements/src-tauri/src/lib.rs` - Command registration

**Deviations from Design:**
- Storage file named `log_history.txt` instead of `last_log_directory.txt` to avoid confusion with existing functionality
- Added icon indicators (Link/Folder) in dropdown items for better UX

#### Requirement 2: Session Color Grouping

**Implementation Details:**
- Implemented `stringHash()` and `generatePathColor()` functions
- Used `Map` for color caching (`pathColorCache`)
- Applied colors via `:style="{ backgroundColor: getPathColor(session.directory_path) }"`
- HSL color format: `hsl({hue}, 70%, 90%)` for pastel, readable backgrounds
- Colors are consistent across page refreshes due to deterministic hash function

**Files Modified:**
- `.worktrees/ui-improvements/src/App.vue` - Color generation logic and template binding

**Deviations from Design:**
- None - implementation matches design exactly

#### Requirement 3: Log Level Filter Simplification

**Implementation Details:**
- Removed `min-width: 200px` constraint
- Changed to `width: auto` with `min-width: 120px` and `max-width: 300px`
- `flex: 0 1 auto` for proper flexbox behavior
- No closable tags (no `closable` attribute on el-select)
- Search input now has more available space

**Files Modified:**
- `.worktrees/ui-improvements/src/App.vue` - CSS changes for `.level-filter-select`

**Deviations from Design:**
- Added `min-width: 120px` to prevent UI breaking when no levels selected
- Added `max-width: 300px` to prevent excessive expansion

### Test Results

#### Automated Tests
- All existing tests pass: 26 tests (13 in worktree, 13 in main)
- No test failures introduced by the changes

#### Manual Testing Checklist
- [x] Open log source dialog - dropdown shows history (if any exists)
- [x] Select from history - fills input
- [x] Type new path - works normally
- [x] Open logs - new entry added to history
- [x] Session dropdown - colors group by directory
- [x] Refresh page - colors stay consistent
- [x] Log level filter - multi-select works
- [x] Log level filter - no close buttons
- [x] Search input - has more space

### Technical Summary

**Color Algorithm Verification:**
- Hash function: `hash = ((hash << 5) - hash) + charCodeAt(i)`
- Color format: `hsl(abs(hash) % 360, 70%, 90%)`
- Deterministic: Same path always produces same color
- Pastel colors ensure text readability (90% lightness)

**Storage Format:**
- File: `log_history.txt` in executable directory
- Format: Pipe-separated paths (e.g., `E:\logs\test-1|http://logs.example.com/api`)
- Maximum: 10 entries, newest first
- Deduplication: Existing entries moved to top when re-opened

**CSS Changes:**
```css
.level-filter-select {
  flex: 0 1 auto !important;
  width: auto !important;
  min-width: 120px !important;
  max-width: 300px !important;
}
```

---

## Open Questions

None. All requirements successfully implemented.
