# JSON Message Display Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add JSON viewing capability to message tooltips with syntax highlighting, toggle between raw/JSON views, and copy functionality

**Architecture:**
- Frontend-only implementation using JavaScript JSON parsing and custom syntax highlighter
- Message tooltip enhanced with Raw/JSON toggle buttons and copy functionality
- On-demand JSON parsing with caching for performance

**Tech Stack:**
- Vue 3 Composition API, Element Plus (el-popover, el-dialog), JavaScript JSON.parse, navigator.clipboard API

---

## Task 1: Create JSON Parser and Syntax Highlighter Utility

**Files:**
- Create: `src/utils/jsonViewer.js`

**Step 1: Create the JSON utility file with all functions**

```javascript
/**
 * Detect if a message contains valid JSON
 * @param {string} message - The message to check
 * @returns {Object} { success: boolean, parsed: object|null, error: string|null }
 */
export function detectJson(message) {
  if (!message || typeof message !== 'string') {
    return { success: false, parsed: null, error: 'Invalid message' }
  }

  const trimmed = message.trim()

  try {
    const parsed = JSON.parse(trimmed)
    return {
      success: true,
      parsed: parsed,
      error: null
    }
  } catch (e) {
    return {
      success: false,
      parsed: null,
      error: e.message
    }
  }
}

/**
 * Syntax highlight JSON with HTML spans
 * @param {*} jsonObj - The parsed JSON object
 * @returns {string} HTML string with syntax highlighting
 */
export function syntaxHighlightJson(jsonObj) {
  if (jsonObj === null) {
    return '<span style="color: #569cd6">null</span>'
  }

  if (typeof jsonObj === 'boolean') {
    return '<span style="color: #569cd6">' + jsonObj + '</span>'
  }

  if (typeof jsonObj === 'number') {
    return '<span style="color: #b5cea8">' + jsonObj + '</span>'
  }

  if (typeof jsonObj === 'string') {
    return '<span style="color: #ce9178">"' + escapeHtml(jsonObj) + '"</span>'
  }

  if (Array.isArray(jsonObj)) {
    if (jsonObj.length === 0) {
      return '[]'
    }

    let output = '['
    for (let i = 0; i < jsonObj.length; i++) {
      output += '<div>'
      output += syntaxHighlightJson(jsonObj[i])
      if (i < jsonObj.length - 1) {
        output += ','
      }
      output += '</div>'
    }
    output += ']'
    return output
  }

  if (typeof jsonObj === 'object') {
    const keys = Object.keys(jsonObj)
    if (keys.length === 0) {
      return '{}'
    }

    let output = '{'
    for (let i = 0; i < keys.length; i++) {
      const key = keys[i]
      output += '<div>'
      output += '<span style="color: #9cdcfe">"' + escapeHtml(key) + '"</span>'
      output += ': '
      output += syntaxHighlightJson(jsonObj[key])
      if (i < keys.length - 1) {
        output += ','
      }
      output += '</div>'
    }
    output += '}'
    return output
  }

  return String(jsonObj)
}

/**
 * Escape HTML special characters
 * @param {string} str - String to escape
 * @returns {string} Escaped string
 */
function escapeHtml(str) {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;')
}

/**
 * Prettify JSON for display
 * @param {string} jsonString - Valid JSON string
 * @returns {string} Prettified JSON string
 */
export function prettifyJson(jsonString) {
  try {
    const parsed = JSON.parse(jsonString)
    return JSON.stringify(parsed, null, 2)
  } catch (e) {
    return jsonString
  }
}

/**
 * Get JSON size in KB
 * @param {string} jsonString - JSON string
 * @returns {number} Size in KB
 */
export function getJsonSize(jsonString) {
  return new Blob([jsonString]).size / 1024
}

/**
 * Search in JSON object
 * @param {*} jsonObj - Parsed JSON object
 * @param {string} searchTerm - Term to search for
 * @returns {Array<string>} Paths to matching properties
 */
export function searchInJson(jsonObj, searchTerm) {
  const results = []
  const lowerSearchTerm = searchTerm.toLowerCase()

  function search(obj, path = '') {
    if (obj === null || typeof obj !== 'object') {
      const valueStr = String(obj).toLowerCase()
      if (valueStr.includes(lowerSearchTerm)) {
        results.push(path || 'value')
      }
      return
    }

    if (Array.isArray(obj)) {
      obj.forEach((item, index) => {
        search(item, path ? `${path}[${index}]` : `[${index}]`)
      })
    } else {
      Object.keys(obj).forEach(key => {
        const keyLower = key.toLowerCase()
        if (keyLower.includes(lowerSearchTerm)) {
          results.push(path ? `${path}.${key}` : key)
        }
        search(obj[key], path ? `${path}.${key}` : key)
      })
    }
  }

  try {
    const parsed = typeof jsonString === 'object' ? jsonObj : JSON.parse(jsonString)
    search(parsed)
  } catch (e) {
    // Return empty on parse error
  }

  return results
}
```

**Step 2: Verify file was created**

Run: `ls -la src/utils/jsonViewer.js`
Expected: File exists at `src/utils/jsonViewer.js`

**Step 3: Run frontend build to check for syntax errors**

Run: `pnpm build`
Expected: Build succeeds with no errors

**Step 4: Commit**

```bash
git add src/utils/jsonViewer.js
git commit -m "feat: add JSON parser and syntax highlighter utility

- detectJson(): Check if message contains valid JSON
- syntaxHighlightJson(): Apply color-coded syntax highlighting
- prettifyJson(): Format JSON with 2-space indentation
- getJsonSize(): Calculate JSON size in KB
- searchInJson(): Search for keys/values in JSON object
- escapeHtml(): Helper for HTML escaping

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Create Message Tooltip Component

**Files:**
- Create: `src/components/MessageTooltip.vue`

**Step 1: Create the Vue component**

```vue
<template>
  <el-popover
    :width="popoverWidth"
    :placement="placement"
    trigger="click"
    popper-class="message-tooltip-popover">
    <template #reference>
      <span
        class="message-tooltip-trigger"
        :class="{ 'has-json': hasJson }">
        {{ truncatedMessage }}
      </span>
    </template>

    <div class="message-tooltip-content">
      <!-- Header with toggle and copy buttons -->
      <div class="tooltip-header">
        <div class="view-toggles">
          <el-button
            :type="viewMode === 'raw' ? 'primary' : 'default'"
            size="small"
            @click="viewMode = 'raw'">
            Raw
          </el-button>
          <el-button
            :type="viewMode === 'json' ? 'primary' : 'default'"
            size="small"
            :disabled="!hasJson"
            @click="viewMode = 'json'">
            JSON
          </el-button>
        </div>
        <div class="copy-buttons">
          <el-button
            size="small"
            @click="copyRaw">
            Copy Raw
          </el-button>
          <el-button
            size="small"
            :disabled="!hasJson"
            @click="copyJson">
            Copy JSON
          </el-button>
        </div>
      </div>

      <!-- Content area -->
      <div class="tooltip-body">
        <!-- Raw view -->
        <div v-if="viewMode === 'raw'" class="raw-view">
          <pre class="message-text">{{ message }}</pre>
        </div>

        <!-- JSON view -->
        <div v-else class="json-view">
          <div v-if="jsonError" class="json-error">
            Invalid JSON: {{ jsonError }}
          </div>
          <div v-else class="json-content" v-html="highlightedJson"></div>
        </div>
      </div>
    </div>
  </el-popover>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { detectJson, syntaxHighlightJson, prettifyJson, getJsonSize } from '../utils/jsonViewer.js'

const props = defineProps({
  message: {
    type: String,
    default: ''
  },
  useDialogForLargeJson: {
    type: Boolean,
    default: true
  },
  largeJsonThreshold: {
    type: Number,
    default: 2 // KB
  }
})

const emit = defineEmits(['json-detected'])

// State
const viewMode = ref('raw')
const hasJson = ref(false)
const parsedJson = ref(null)
const jsonError = ref(null)
const highlightedJson = ref('')

// Computed
const popoverWidth = computed(() => {
  if (!hasJson.value) {
    return '400px'
  }
  const size = getJsonSize(props.message)
  return size > props.largeJsonThreshold ? '80%' : '600px'
})

const placement = computed(() => {
  return props.useDialogForLargeJson && hasJson.value ? 'bottom' : 'bottom'
})

const truncatedMessage = computed(() => {
  if (!props.message) {
    return '-'
  }
  const maxLen = 60
  if (props.message.length > maxLen) {
    return props.message.substring(0, maxLen) + '...'
  }
  return props.message
})

// Methods
function detectJsonInMessage() {
  const result = detectJson(props.message)
  hasJson.value = result.success
  parsedJson.value = result.parsed
  jsonError.value = result.error

  if (hasJson.value) {
    emit('json-detected', { parsed: result.parsed })
  }
}

function generateHighlightedJson() {
  if (hasJson.value && parsedJson.value !== null) {
    highlightedJson.value = syntaxHighlightJson(parsedJson.value)
  }
}

async function copyRaw() {
  try {
    await navigator.clipboard.writeText(props.message)
    ElMessage.success('Raw message copied to clipboard')
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = props.message
    textArea.style.position = 'fixed'
    textArea.style.opacity = '0'
    document.body.appendChild(textArea)
    textArea.select()
    try {
      document.execCommand('copy')
      ElMessage.success('Raw message copied to clipboard')
    } catch (e) {
      ElMessage.error('Copy failed: ' + e.message)
    }
    document.body.removeChild(textArea)
  }
}

async function copyJson() {
  if (!hasJson.value) {
    return
  }

  const prettified = prettifyJson(props.message)

  try {
    await navigator.clipboard.writeText(prettified)
    ElMessage.success('JSON copied to clipboard')
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = prettified
    textArea.style.position = 'fixed'
    textArea.style.opacity = '0'
    document.body.appendChild(textArea)
    textArea.select()
    try {
      document.execCommand('copy')
      ElMessage.success('JSON copied to clipboard')
    } catch (e) {
      ElMessage.error('Copy failed: ' + e.message)
    }
    document.body.removeChild(textArea)
  }
}

// Lifecycle
watch(() => props.message, () => {
  detectJsonInMessage()
  if (hasJson.value) {
    generateHighlightedJson()
  }
}, { immediate: true })

watch(viewMode, (newMode) => {
  if (newMode === 'json' && !highlightedJson.value) {
    generateHighlightedJson()
  }
})
</script>

<style scoped>
.message-tooltip-trigger {
  cursor: pointer;
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
}

.message-tooltip-trigger.has-json {
  color: #409EFF;
  font-weight: 500;
}

.tooltip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #dcdfe6;
  margin-bottom: 8px;
}

.view-toggles {
  display: flex;
  gap: 8px;
}

.copy-buttons {
  display: flex;
  gap: 8px;
}

.tooltip-body {
  max-height: 400px;
  overflow-y: auto;
  padding: 12px;
}

.raw-view .message-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.json-view {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.json-error {
  color: #f56c6c;
  padding: 12px;
}

.json-content {
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
}

.json-content > div {
  margin-left: 20px;
}

.json-content > div:first-child {
  margin-left: 0;
}
</style>
```

**Step 2: Verify component was created**

Run: `ls -la src/components/MessageTooltip.vue`
Expected: File exists

**Step 3: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/components/MessageTooltip.vue
git commit -m "feat: add MessageTooltip component with Raw/JSON toggle

- Toggle between raw message and parsed JSON views
- Syntax highlighting for JSON with color coding
- Copy functionality for both raw and JSON content
- Responsive width: 600px popover, 80% dialog for large JSON
- Disabled JSON button when message doesn't contain valid JSON

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Integrate MessageTooltip into Main Table

**Files:**
- Modify: `src/App.vue`

**Step 1: Import MessageTooltip component**

Add to imports section (around line 30):

```javascript
import MessageTooltip from './components/MessageTooltip.vue'
```

**Step 2: Replace message column template**

Find the message column in el-table (around line 1050-1060) and replace with:

```vue
<el-table-column
  prop="message"
  label="消息"
  min-width="300"
  showOverflowTooltip>
  <template #default="{ row }">
    <MessageTooltip
      :message="row.message"
      :useDialogForLargeJson="true"
      :largeJsonThreshold="2"
      @json-detected="(data) => handleJsonDetected(row.id, data)" />
  </template>
</el-table-column>
```

**Step 3: Add handler function for json-detected event**

Add after the other handler functions (around line 650):

```javascript
// Handle JSON detected in message
function handleJsonDetected(entryId, data) {
  console.log('JSON detected in entry', entryId, ':', data)
  // Can be used for statistics or future features
}
```

**Step 4: Remove old showOverflowTooltip from message column**

The MessageTooltip component handles display, so remove `showOverflowTooltip` prop from the column definition if present.

**Step 5: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 6: Test the tooltip functionality**

Run: `pnpm tauri dev`
Expected:
- App starts successfully
- Message column shows truncated messages
- Click on message opens tooltip with Raw/JSON buttons

**Step 7: Commit**

```bash
git add src/App.vue
git commit -m "feat: integrate MessageTooltip into main table

- Replace message column with MessageTooltip component
- Add handler for json-detected events
- Remove showOverflowTooltip (handled by component)
- Support click-to-open tooltip with Raw/JSON toggle

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Add Styles for Tooltip Popover

**Files:**
- Modify: `src/App.vue` (in style section)

**Step 1: Add global styles for message tooltip popover**

Add to the `<style>` section (not scoped) at the end of the file:

```css
/* Message Tooltip Popover Styles */
.message-tooltip-popover {
  max-width: 80%;
}

.message-tooltip-popover .el-popover__body {
  padding: 0;
}

.message-tooltip-popover .tooltip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #dcdfe6;
  background-color: #f5f7fa;
}

.message-tooltip-popover .view-toggles,
.message-tooltip-popover .copy-buttons {
  display: flex;
  gap: 8px;
}

.message-tooltip-popover .tooltip-body {
  max-height: 400px;
  overflow-y: auto;
  padding: 12px;
}

.message-tooltip-popover .raw-view .message-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.message-tooltip-popover .json-view {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.message-tooltip-popover .json-error {
  color: #f56c6c;
  padding: 12px;
}

.message-tooltip-popover .json-content {
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
}
```

**Step 2: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "style: add global styles for message tooltip popover

- Styles for popover header with toggle and copy buttons
- Monospace font for code readability
- Dark theme for JSON content
- Scrollable content area with max-height

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Add Search Functionality (Optional/Nice-to-Have)

**Files:**
- Modify: `src/components/MessageTooltip.vue`

**Step 1: Add search input to template**

Add after the header div, before tooltip-body:

```vue
<!-- Search bar (for JSON view) -->
<div v-if="viewMode === 'json' && hasJson" class="search-bar">
  <el-input
    v-model="searchTerm"
    placeholder="Search keys/values..."
    size="small"
    clearable
    @input="handleSearch">
    <template #prefix>
      <el-icon><Search /></el-icon>
    </template>
  </el-input>
</div>
```

**Step 2: Add search state and methods**

Add to script setup section:

```javascript
import { Search } from '@element-plus/icons-vue'

const searchTerm = ref('')
const searchResults = ref([])

function handleSearch() {
  if (!searchTerm.value || !parsedJson.value) {
    searchResults.value = []
    return
  }

  // Use searchInJson from utils
  import { searchInJson } from '../utils/jsonViewer.js'
  searchResults.value = searchInJson(parsedJson.value, searchTerm.value)
}
```

**Step 3: Highlight search results**

Update the json-content div to show highlighted results:

```vue
<div v-else class="json-content">
  <div v-if="searchResults.length > 0" class="search-results-info">
    Found {{ searchResults.length }} match(es)
  </div>
  <div v-html="highlightedJson"></div>
</div>
```

**Step 4: Add CSS for search bar**

```css
.search-bar {
  padding: 8px 12px;
  border-bottom: 1px solid #dcdfe6;
}

.search-results-info {
  padding: 8px 12px;
  background-color: #e6f7ff;
  color: #409EFF;
  font-size: 12px;
}
```

**Step 5: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 6: Commit**

```bash
git add src/components/MessageTooltip.vue
git commit -m "feat: add search functionality to JSON viewer

- Search input in JSON view
- Shows count of matches
- Searches both keys and values
- Clear button to reset search

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Add Collapsible Sections (Optional/Nice-to-Have)

**Files:**
- Modify: `src/utils/jsonViewer.js`
- Modify: `src/components/MessageTooltip.vue`

**Step 1: Update syntaxHighlightJson to support collapsible sections**

In `jsonViewer.js`, modify the function to add data attributes:

```javascript
export function syntaxHighlightJson(jsonObj, depth = 0) {
  // ... existing code for primitive types ...

  if (Array.isArray(jsonObj)) {
    if (jsonObj.length === 0) {
      return '[]'
    }

    let output = '<div class="json-array" data-depth="' + depth + '">'
    output += '<span class="json-toggle" onclick="toggleJsonNode(this)">[-]</span> ['

    for (let i = 0; i < jsonObj.length; i++) {
      output += '<div class="json-item" data-collapsed="false">'
      output += syntaxHighlightJson(jsonObj[i], depth + 1)
      if (i < jsonObj.length - 1) {
        output += ','
      }
      output += '</div>'
    }

    output += ']</div>'
    return output
  }

  if (typeof jsonObj === 'object') {
    const keys = Object.keys(jsonObj)
    if (keys.length === 0) {
      return '{}'
    }

    let output = '<div class="json-object" data-depth="' + depth + '">'
    output += '<span class="json-toggle" onclick="toggleJsonNode(this)">[-]</span> {'

    for (let i = 0; i < keys.length; i++) {
      const key = keys[i]
      output += '<div class="json-item" data-collapsed="false">'
      output += '<span style="color: #9cdcfe">"' + escapeHtml(key) + '"</span>'
      output += ': '
      output += syntaxHighlightJson(jsonObj[key], depth + 1)
      if (i < keys.length - 1) {
        output += ','
      }
      output += '</div>'
    }

    output += '}</div>'
    return output
  }

  return String(jsonObj)
}

// Global function for toggle (attached to window)
window.toggleJsonNode = function(element) {
  const parent = element.parentElement
  const items = parent.querySelectorAll(':scope > .json-item')
  const isCollapsed = parent.getAttribute('data-collapsed') === 'true'

  if (isCollapsed) {
    // Expand
    items.forEach(item => item.style.display = '')
    parent.setAttribute('data-collapsed', 'false')
    element.textContent = '[-]'
  } else {
    // Collapse
    items.forEach(item => item.style.display = 'none')
    parent.setAttribute('data-collapsed', 'true')
    element.textContent = '[+]'
  }
}
```

**Step 2: Add CSS for collapsible sections**

In MessageTooltip.vue `<style scoped>`:

```css
.json-array,
.json-object {
  position: relative;
}

.json-toggle {
  position: absolute;
  left: -20px;
  cursor: pointer;
  user-select: none;
  color: #409EFF;
  font-weight: bold;
  font-size: 12px;
}

.json-toggle:hover {
  color: #66b1ff;
}

.json-item {
  margin-left: 20px;
}

.json-item[data-collapsed="true"] {
  display: none;
}
```

**Step 3: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add src/utils/jsonViewer.js src/components/MessageTooltip.vue
git commit -m "feat: add collapsible sections for JSON viewer

- Click [-]/[+] to expand/collapse objects and arrays
- Visual indicator for collapsed state
- Improves readability for deeply nested JSON

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Integration Testing

**Files:**
- No file changes - manual testing

**Step 1: Test with valid JSON message**

1. Run the app: `pnpm tauri dev`
2. Load a session with JSON in messages
3. Click on a message containing JSON
4. Verify tooltip opens with both "Raw" and "JSON" buttons enabled
5. Click "JSON" button
6. Verify syntax highlighting is applied (colors match specification)
7. Click "Copy JSON"
8. Paste to verify correct format

**Step 2: Test with invalid JSON**

1. Find a message without valid JSON
2. Click on the message
3. Verify tooltip opens with only "Raw" button active
4. Verify "JSON" button is disabled

**Step 3: Test with large JSON**

1. Find or create a message with large JSON (>2KB)
2. Click on the message
3. Verify tooltip displays correctly (may be wider)
4. Verify scroll works if content overflows

**Step 4: Test copy functionality**

1. Click "Copy Raw" button
2. Verify success message appears
3. Paste to verify raw message was copied
4. Click "Copy JSON" button (if JSON available)
5. Verify prettified JSON is copied

**Step 5: Test search functionality** (if implemented)

1. In JSON view, type in search input
2. Verify matching results are counted
3. Check that search is case-insensitive

**Step 6: Test collapsible sections** (if implemented)

1. In JSON view with nested objects
2. Click [-] button to collapse
3. Click [+] to expand
4. Verify nested items hide/show correctly

**Step 7: Verify no performance regression**

1. Load a session with 100+ messages
2. Click through multiple messages
3. Verify UI remains responsive
4. Check memory usage is reasonable

**Step 8: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All existing tests pass

**Step 9: Build production version**

Run: `pnpm tauri build`
Expected: Build succeeds, installer created

**Step 10: Final commit if fixes were needed**

If any fixes were needed during testing:

```bash
git add .
git commit -m "fix: [description of fixes found during testing]

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Summary

This implementation plan adds JSON viewing capability to log messages with:

**Must-Haves:**
- ✅ Syntax highlighting with color-coded tokens
- ✅ Copy functionality for raw and formatted JSON
- ✅ Manual toggle between Raw/JSON views
- ✅ Smart JSON detection

**Nice-to-Haves:**
- ✅ Search within JSON
- ✅ Collapsible nested sections

**Total tasks:** 7 tasks with ~35 small steps
**Estimated time:** 2-3 hours
**Files created/modified:** 3 new files, 2 modified files

The implementation is frontend-only with no backend changes required, making it safe to add without affecting existing functionality.
