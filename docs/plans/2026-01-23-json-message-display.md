# Enhanced JSON Message Display Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add JSON viewing capability to message tooltips with syntax highlighting, copy functionality, and search

**Architecture:**
- Manual toggle between Raw/JSON views in message tooltip
- On-demand JSON parsing with syntax highlighting
- Copy functionality for both raw message and formatted JSON

**Tech Stack:**
- Frontend: Vue 3 + Element Plus (el-popover, el-dialog)
- JSON parsing: JavaScript JSON.parse with custom syntax highlighter
- Copy: navigator.clipboard API with fallback

---

## Overview

Enhance the existing message column hover/tooltip to display JSON data in a structured, readable format. Users can toggle between raw message view and parsed JSON view with syntax highlighting.

## Use Cases

1. **Search for specific keys or values** - Filter/highlight JSON content
2. **Compare JSON across entries** - Side-by-side comparison in tooltips
3. **View the full structure** - Prettified, indented JSON with syntax highlighting
4. **Copy/export JSON** - Copy raw message or formatted JSON

## Component Architecture

```
Message Column Cell
└── Click/hover trigger
    └── el-popover (or el-dialog for large JSON)
        ├── Header
        │   ├── View toggle buttons: [Raw] [JSON]
        │   └── Copy buttons: [Copy Raw] [Copy JSON]
        └── Content area (scrollable)
            ├── Raw view: Original message text
            └── JSON view: Syntax-highlighted, prettified JSON
```

## Features

### Must-Haves
1. **Syntax Highlighting**
   - Keys: Purple/Blue (`#9cdcfe`)
   - String values: Green (`#ce9178`)
   - Numbers: Orange (`#b5cea8`)
   - Booleans: Blue (`#569cd6`)
   - Null: Gray (`#569cd6`)
   - Brackets: Yellow (`#ffd700`)

2. **Copy Options**
   - Copy Raw: Original message text
   - Copy JSON: Formatted, prettified JSON as plain text

### Nice-to-Haves
3. **Search within JSON**
   - Filter to highlight keys or values matching search term
   - Case-insensitive search
   - Navigate between matches

4. **Collapsible Sections**
   - Expand/collapse nested objects and arrays
   - Show count of items in collapsed state
   - One-click expand-all/collapse-all

## JSON Detection

### Detection Logic
```javascript
function detectJson(message) {
  try {
    const parsed = JSON.parse(message);
    return {
      success: true,
      parsed: parsed,
      isValid: true
    };
  } catch (e) {
    return {
      success: false,
      error: e.message,
      isValid: false
    };
  }
}
```

**Rules:**
- Try to parse entire message as JSON
- If successful → enable "JSON" toggle button
- If failed → disable "JSON" button, show "Raw" only

### Display Triggers
- **Small JSON** (< 2KB): Use `el-popover`
- **Large JSON** (≥ 2KB): Use `el-dialog` for better viewing

## UI Layout

### Popover/Dialog Structure
```
┌─────────────────────────────────────────┐
│ [Raw] [JSON]        [Copy Raw] [Copy]  │ ← Header (flex row)
├─────────────────────────────────────────┤
│                                         │
│  Message content area (scrollable)     │
│                                         │
│  Raw: {"user":"admin","status":"ok"}   │
│                                         │
│  JSON: {                                │
│    "user": "admin",                     │
│    "status": "ok",                      │
│    "data": {                            │
│      "id": 123                          │
│    }                                    │
│  }                                      │
│                                         │
└─────────────────────────────────────────┘
```

### Styling Specifications

| Property | Value |
|----------|-------|
| Max width (popover) | 600px |
| Max width (dialog) | 80% of viewport |
| Max height | 400px (with scroll) |
| Font family | Monospace (Consolas, Fira Code, monospace) |
| Font size | 13px |
| Line height | 1.5 |
| Background (JSON view) | `#1e1e1e` (dark theme) |
| Padding | 12px |
| Border radius | 4px |

## Data Flow

```
User Interaction Flow:
1. User hovers/clicks message cell
2. Component calls detectJson(message)
3. If valid JSON:
   - Enable "JSON" toggle button
   - Store parsed result in cache
4. User clicks "JSON" button
5. Parse and prettify JSON with syntaxHighlight()
6. Render HTML with <span> tags and color classes
7. User clicks "Copy JSON"
8. Copy formatted text to clipboard
9. Show toast: "JSON copied to clipboard"
```

## Error Handling

| Error Scenario | Handling |
|----------------|----------|
| Invalid JSON | Disable "JSON" button, show only "Raw" view |
| Parse error | Show error: "Invalid JSON: <error message>" |
| Copy failed (no permission) | Toast: "Copy failed - please check permissions" |
| Very large JSON (>100KB) | Show warning, suggest alternative view |
| Circular references | Detect and show warning, limit depth |

## Performance Considerations

- **Lazy parsing**: Parse JSON only when "JSON" view is toggled
- **Cache**: Store parsed result to avoid re-parsing on toggle
- **Debounce search**: 300ms delay for search input
- **Virtual scrolling**: For JSON with >1000 properties
- **Depth limit**: Max nesting depth of 10 levels (prevent stack overflow)

## Accessibility

- **Keyboard navigation**: Tab to navigate between toggle buttons
- **ARIA labels**: "View raw message", "View JSON", "Copy to clipboard"
- **Focus management**: Return focus to trigger element after close
- **Screen reader support**: Announce view mode changes

## File Structure

### Frontend
```
src/
├── utils/
│   └── jsonViewer.js          # JSON parser and syntax highlighter
├── components/
│   └── MessageTooltip.vue      # Reusable tooltip component (optional)
└── App.vue                    # Main integration point
```

### Backend
No backend changes required - all processing in frontend.

## API Dependencies

- **Clipboard API**: `navigator.clipboard.writeText()`
- **Fallback**: `document.execCommand('copy')` for older browsers
- **Element Plus**: `el-popover`, `el-dialog`, `el-input` (search)

## Testing Scenarios

1. **Valid JSON** - Toggle enabled, syntax highlighting works
2. **Invalid JSON** - Toggle disabled, only raw view shown
3. **Empty message** - Both views handle gracefully
4. **Large JSON** - Dialog used instead of popover
5. **Copy functionality** - Both raw and JSON copy work
6. **Search** - Highlights matching keys/values
7. **Nested objects** - Collapsible sections work correctly
8. **Special characters** - Unicode, escape sequences handled

## Success Criteria

1. ✅ JSON messages can be viewed in formatted, syntax-highlighted view
2. ✅ Toggle between Raw and JSON views works smoothly
3. ✅ Copy functionality works for both raw and JSON
4. ✅ Invalid JSON is handled gracefully (no errors)
5. ✅ Large JSON uses dialog instead of popover
6. ✅ Search highlights matching content (if implemented)
7. ✅ Performance remains good with large messages
