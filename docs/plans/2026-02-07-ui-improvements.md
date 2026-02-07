# UI Improvements Implementation Plan - 2026-02-07

## Overview

This plan implements four UI/UX improvements to logTerminator. See the companion design document for detailed specifications.

---

## Phase 1: Message Column Formatting (Feature 3)

**Status:** Pending
**Estimated Complexity:** Low

### Tasks

1. Create `src/utils/messageFormatter.js`
   - Add `formatMessage()` function
   - Add `setHighlightRules()` for future extension
   - Export functions

2. Modify `App.vue`
   - Import `formatMessage` from utils
   - Update message column template to use `formatMessage()`
   - Wrap message in `<div class="message-cell">`

3. Add CSS styles
   - Add `.message-cell` class with `white-space: pre-wrap`
   - Add `word-break: break-word`
   - Add `line-height: 1.5`

### Testing
- Load logs with multi-line messages
- Verify newlines display correctly
- Check long messages wrap properly

---

## Phase 2: Stack Column Iconization (Feature 1)

**Status:** Pending
**Estimated Complexity:** Low

### Tasks

1. Update `App.vue` stack column
   - Change width from current to `60px`
   - Add `align="center"`
   - Replace text display with icon template

2. Implement stack icon with tooltip
   - Use Element Plus `Warning` icon
   - Add `el-tooltip` component
   - Configure tooltip placement `left`, delay `300ms`
   - Add custom `popper-class="stack-tooltip"`

3. Add CSS styles
   - `.stack-icon` base styles
   - `.stack-icon:hover` interaction
   - `.stack-trace` tooltip content styles
   - Use monospace font for stack traces
   - Add max dimensions and overflow scroll

### Testing
- Verify icon appears for entries with stack traces
- Verify no icon for entries without stack traces
- Test tooltip display and positioning
- Check stack trace text formatting

---

## Phase 3: Auto-bookmark MARKER Entries (Feature 2)

**Status:** Pending
**Estimated Complexity:** Medium

### Backend Tasks

1. Add Tauri command in `src-tauri/src/lib.rs`
   ```rust
   #[tauri::command]
   pub async fn auto_bookmark_markers(session_id: i64) -> Result<Vec<Bookmark>, String>
   ```

2. Implement function in `src-tauri/src/database.rs`
   ```rust
   pub fn auto_bookmark_markers(conn: &Connection, session_id: i64) -> Result<Vec<Bookmark>>
   ```
   - Query MARKER level entries where message doesn't contain "###"
   - Check existing bookmarks to avoid duplicates
   - Insert new bookmarks with full message as title
   - Return created bookmarks

3. Update `src-tauri/src/lib.rs`
   - Register new command with Tauri

### Frontend Tasks

1. Update `App.vue` loadSession function
   - After fetching logs, call `auto_bookmark_markers`
   - Merge returned bookmarks into `bookmarks.value`

2. Ensure bookmark state updates
   - Verify UI refreshes with new bookmarks
   - Check bookmark sidebar updates

### Testing
- Load session with MARKER entries
- Verify auto-bookmarks appear
- Test "###" exclusion filter
- Verify no duplicate bookmarks on reload
- Test bookmark deletion/editing still works

---

## Phase 4: Log File Output (Feature 4)

**Status:** Pending
**Estimated Complexity:** Medium

### Backend Tasks

1. Update `src-tauri/Cargo.toml`
   - Add `log = "0.4"` dependency
   - Add `env_logger = "0.11"` dependency

2. Create logging setup in `src-tauri/src/main.rs`
   - Add `init_logging()` function
   - Get executable directory path
   - Create/open `logterminator.log` in append mode
   - Configure env_logger to write to file

3. Call `init_logging()` in main()
   - Initialize before other setup
   - Handle errors gracefully

4. Replace console output throughout codebase
   - In `log_parser.rs`: Replace `println!` with `log::info!`, `log::error!`
   - In `database.rs`: Add logging for database operations
   - In `lib.rs`: Add logging for Tauri commands

5. Test on different platforms
   - Windows: Verify file path handling
   - macOS: Verify permissions
   - Linux: Verify file creation

### Testing
- Run application and verify `logterminator.log` creation
- Check log content is written correctly
- Test with `RUST_LOG=debug` environment variable
- Verify log file is in same directory as executable
- Manually delete log file and verify recreation

---

## Success Criteria

- [ ] Multi-line messages display with proper formatting
- [ ] Stack column uses icon instead of text
- [ ] Stack tooltip shows full trace with monospace font
- [ ] MARKER entries auto-bookmark on session load
- [ ] Entries containing "###" are excluded from auto-bookmark
- [ ] Log file created in executable directory
- [ ] All debug output written to log file
- [ ] No regression in existing functionality

---

## Rollback Plan

Each phase is independent and can be rolled back individually:

- **Phase 1 & 2**: Revert CSS and template changes in `App.vue`
- **Phase 3**: Remove Tauri command and frontend call
- **Phase 4**: Remove log dependencies and revert to `println!`
