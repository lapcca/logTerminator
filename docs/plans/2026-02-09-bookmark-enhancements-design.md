# Bookmark Enhancements Design

## Date: 2026-02-09

## Overview

This document describes two enhancements to the bookmark system in logTerminator:

1. **User-Selectable Bookmark Colors** - Users can choose colors when adding bookmarks, displayed as background highlights in the sidebar
2. **Failure Anchor Auto-Bookmark** - Automatically bookmark log entries containing `<a id="failureAnchor">` with red color and "Failure" title

---

## Feature 1: User-Selectable Bookmark Colors

### Problem
Currently, all user-created bookmarks use the same amber color. This makes it difficult to distinguish between different types of user bookmarks and to visually separate user bookmarks from system auto-bookmarks.

### Solution
Add a color picker that appears when adding a bookmark. Users can select from:
- **6 Preset Colors**: Red, Orange, Green, Cyan, Purple, Pink
- **Custom Color**: Full color picker dialog

The selected color is displayed as a background highlight on the bookmark item in the sidebar.

### UI Design

```
Color Picker Popover:
┌─────────────────────────────────────────┐
│  Quick Colors        More Colors →      │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐  │
│  │    │ │    │ │    │ │    │ │    │  │
│  └────┘ └────┘ └────┘ └────┘ └────┘  │
│  Red  Orange Green Cyan Purple Pink    │
└─────────────────────────────────────────┘
```

### Color Scheme Reference

| Type | Color | Hex | Usage |
|------|-------|-----|-------|
| Auto (###MARKER###) | Blue | #409EFF | System auto-bookmarks |
| Failure | Red | #F56C6C | Failure anchor bookmarks |
| User Default | Amber | amber | Default user bookmark |
| User Presets | Various | See below | User-selectable colors |

**Preset Colors:**
- Red: `#F56C6C`
- Orange: `#E6A23C`
- Green: `#67C23A`
- Cyan: `#17B2E2`
- Purple: `#9C27B0`
- Pink: `#F48FB1`

### Implementation

**Frontend Components:**
1. **BookmarkColorPicker.vue** (new)
   - Shows 6 preset color buttons
   - "More Colors" button opens Element Plus color picker
   - Emits `color-selected` event

2. **App.vue** (modified)
   - Wrap bookmark star button with el-popover
   - Show color picker on bookmark click
   - Pass selected color to `add_bookmark` command

**Backend:**
- No changes needed (existing `color` parameter in `add_bookmark`)

---

## Feature 2: Failure Anchor Auto-Bookmark

### Problem
When tests fail, a `<a id="failureAnchor">` is inserted in the HTML log. Users must manually find and bookmark these failure points, which is time-consuming and error-prone.

### Solution
Automatically detect and bookmark entries containing `id="failureAnchor"` when loading a session.

### Detection Method (Hybrid Approach)

**During HTML Parsing** (`log_parser.rs`):
1. Extract message text using `.text().collect()` as usual
2. Additionally, use `el.inner_html()` to check if `<td class="message">` contains `id="failureAnchor"`
3. If found, append `[FAIL]` marker at the end of the message text
4. Example: "Test failed assertion" → "Test failed assertion [FAIL]"

**During Session Load** (`database.rs`, `bookmark_utils.rs`):
1. Query entries where message contains `[FAIL]`
2. Create or update bookmarks with:
   - Title: "Failure"
   - Color: "#F56C6C" (red)
   - Notes: null

### Priority Order

**Highest Priority: Failure Anchors**
- Check for `[FAIL]` marker FIRST
- If found and NOT already bookmarked with red → Create red bookmark
- If already bookmarked with different color → Update to red

**Lower Priority: ###MARKER### and MARKER level**
- Only apply if NOT a failure entry
- Use blue color (#409EFF)

This ensures failure bookmarks are always red, even if the entry was already auto-bookmarked.

---

## Architecture

### Backend Changes

**log_parser.rs:**
- Modify message extraction to check inner HTML for `id="failureAnchor"`
- Append `[FAIL]` marker when found

**bookmark_utils.rs:**
- Add `find_failure_anchor_markers(entries: &[LogEntry]) -> Vec<i64>`

**database.rs:**
- Add `query_failure_entries(session_id: &str) -> SqlResult<Vec<i64>>`
- Add `update_bookmark_color_and_title(bookmark_id: i64, color: &str, title: &str) -> SqlResult<()>`
- Modify `ensure_auto_bookmarks_for_session()` to handle priority order

### Frontend Changes

**New Component: BookmarkColorPicker.vue**
- Preset color buttons
- Custom color picker integration
- Emits selected color

**Modified: App.vue**
- Add color picker popover to bookmark button
- Update bookmark sidebar styling for background colors
- Handle color selection and bookmark creation

### Database Schema
No changes needed - existing `color` field in `bookmarks` table is sufficient.

---

## Error Handling

### Bookmark Color Selection
- Invalid color hex → Fallback to default amber
- Missing color parameter → Use default amber
- Backend validates hex format before storing

### Failure Anchor Detection
- Malformed HTML → Continue parsing, log warning
- Multiple failure anchors → Create bookmark for each
- Already manually bookmarked → Update color/title to red
- Message naturally contains `[FAIL]` → User can delete auto-bookmark

### Database Operations
- Bookmark update failure → Log error, continue
- Transaction rollback on critical errors

---

## Testing Strategy

### Unit Tests
1. Failure anchor detection in log_parser
2. Bookmark priority logic in bookmark_utils
3. Color validation

### Integration Tests
1. Load session with failure anchors → Verify red bookmarks created
2. Load session with mixed markers → Verify correct priorities
3. Manually bookmark then reload → Verify color updates to red
4. Color picker UI → Verify all preset colors work
5. Custom color selection → Verify hex color is saved and displayed

---

## Files to Modify

### Backend (Rust)
- `src-tauri/src/log_parser.rs` - Failure anchor detection during parsing
- `src-tauri/src/bookmark_utils.rs` - Failure marker function
- `src-tauri/src/database.rs` - Priority handling and update methods

### Frontend (Vue)
- `src/App.vue` - Color picker integration and sidebar styling
- `src/components/BookmarkColorPicker.vue` - NEW - Color picker component

---

## Future Enhancements
- Allow users to edit bookmark colors after creation
- Color themes for different bookmark categories
- Filter bookmarks by color
- Export bookmarks with color information
