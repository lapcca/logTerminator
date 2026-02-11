# Search Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add enhanced search functionality with regex support, multi-condition search, and a search results panel showing multiple search results simultaneously.

**Architecture:** Frontend SearchPanel component with state management for simple/advanced search modes, regex toggle, and search history. Backend Rust command using SQL LIKE/REGEXP with dynamic query building for AND/OR conditions.

**Tech Stack:** Vue 3 (Composition API), Element Plus, Tauri 2, Rust, SQLite with regex support

---

## Task 1: Create SearchPanel.vue Component Structure

**Files:**
- Create: `src/components/SearchPanel.vue`

**Step 1: Create the component skeleton with reactive state**

```vue
<script setup>
import { ref, reactive, computed } from 'vue'
import { ElMessage } from 'element-plus'
import {
  Search, Operation, Grid, Memo, List,
  Plus, Delete, ArrowDown, ArrowUp, ArrowRight, ArrowLeft
} from '@element-plus/icons-vue'

const emit = defineEmits(['jump-to-entry'])

// Search state
const searchState = reactive({
  simpleTerm: '',
  isRegexMode: false,
  isAdvancedMode: false,
  conditions: [
    { id: Date.now() + '_1', term: '', operator: 'AND' }
  ],
  history: [],
  expandedSearchId: null
})

const showSearchResults = ref(false)
const loading = ref(false)

// Computed
const totalMatchCount = computed(() =>
  searchState.history.reduce((sum, s) => sum + s.matches.length, 0)
)
</script>

<template>
  <div class="search-panel">
    <!-- Search content will be added in next tasks -->
  </div>
</template>

<style scoped>
.search-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
```

**Step 2: Verify the component compiles**

Run: `npm run build`
Expected: Success

**Step 3: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: create SearchPanel component skeleton"
```

---

## Task 2: Implement Search Input Area

**Files:**
- Modify: `src/components/SearchPanel.vue:template,style`

**Step 1: Add search input HTML to template**

```vue
<template>
  <div class="search-panel">
    <!-- Search Input -->
    <el-input
      v-model="searchState.simpleTerm"
      placeholder="搜索日志内容..."
      :prefix-icon="Search"
      clearable
      class="search-input"
      @keyup.enter="executeSimpleSearch">
      <template #suffix>
        <!-- Regex toggle button -->
        <el-tooltip content="正则表达式" placement="top">
          <el-icon
            :class="{ 'regex-active': searchState.isRegexMode }"
            @click="toggleRegexMode"
            class="search-icon-btn">
            <Operation v-if="!searchState.isRegexMode" />
            <Grid v-else />
          </el-icon>
        </el-tooltip>

        <!-- Advanced search toggle button -->
        <el-tooltip content="高级搜索" placement="top">
          <el-icon
            :class="{ 'advanced-active': searchState.isAdvancedMode }"
            @click="toggleAdvancedMode"
            class="search-icon-btn">
            <Memo v-if="!searchState.isAdvancedMode" />
            <List v-else />
          </el-icon>
        </el-tooltip>
      </template>
    </el-input>
  </div>
</template>
```

**Step 2: Add toggle functions and styles**

```vue
<script setup>
// ... existing imports and state ...

// Toggle functions
function toggleRegexMode() {
  searchState.isRegexMode = !searchState.isRegexMode
}

function toggleAdvancedMode() {
  searchState.isAdvancedMode = !searchState.isAdvancedMode
}
</script>

<style scoped>
/* ... existing styles ... */

.search-input {
  width: 400px;
}

.search-icon-btn {
  margin-left: 8px;
  cursor: pointer;
  color: var(--el-text-color-secondary);
  transition: all 0.3s;
}

.search-icon-btn:hover {
  color: var(--el-color-primary);
}

.regex-active,
.advanced-active {
  color: var(--el-color-primary);
}
</style>
```

**Step 3: Verify build**

Run: `npm run build`
Expected: Success

**Step 4: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: add search input with regex and advanced toggles"
```

---

## Task 3: Implement Advanced Search Area

**Files:**
- Modify: `src/components/SearchPanel.vue:template,script,style`

**Step 1: Add advanced search HTML**

```vue
<template>
  <div class="search-panel">
    <!-- ... existing search input ... -->

    <!-- Advanced Search Panel -->
    <el-collapse-transition>
      <div v-show="searchState.isAdvancedMode" class="advanced-search-panel">
        <el-card shadow="never">
          <!-- Search condition rows -->
          <div v-for="(condition, index) in searchState.conditions"
               :key="condition.id"
               class="search-condition-row">

            <!-- AND/OR selector -->
            <el-select
              v-if="index > 0"
              v-model="condition.operator"
              class="operator-select">
              <el-option label="AND" value="AND" />
              <el-option label="OR" value="OR" />
            </el-select>

            <!-- Condition input -->
            <el-input
              v-model="condition.term"
              placeholder="输入搜索条件..."
              :prefix-icon="Search"
              clearable />

            <!-- Delete button -->
            <el-button
              v-if="searchState.conditions.length > 1"
              :icon="Delete"
              circle
              size="small"
              @click="removeCondition(index)" />
          </div>

          <!-- Add condition button -->
          <el-button
            :icon="Plus"
            @click="addCondition"
            class="add-condition-btn">
            添加条件
          </el-button>

          <!-- Search button -->
          <el-button
            type="primary"
            :icon="Search"
            @click="executeAdvancedSearch">
            搜索
          </el-button>
        </el-card>
      </div>
    </el-collapse-transition>
  </div>
</template>
```

**Step 2: Add condition management functions**

```vue
<script setup>
// ... existing code ...

function addCondition() {
  searchState.conditions.push({
    id: Date.now() + '_' + (searchState.conditions.length + 1),
    term: '',
    operator: 'AND'
  })
}

function removeCondition(index) {
  searchState.conditions.splice(index, 1)
}

function executeAdvancedSearch() {
  const validConditions = searchState.conditions.filter(c => c.term.trim())
  if (validConditions.length === 0) {
    ElMessage.warning('请输入至少一个搜索条件')
    return
  }
  // TODO: Will implement actual search in backend task
  console.log('Advanced search:', validConditions)
}
</script>
```

**Step 3: Add styles**

```vue
<style scoped>
/* ... existing styles ... */

.advanced-search-panel {
  margin-top: 12px;
}

.search-condition-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.operator-select {
  width: 80px;
}

.add-condition-btn {
  margin-right: 12px;
}
</style>
```

**Step 4: Verify build**

Run: `npm run build`
Expected: Success

**Step 5: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: add advanced search area with condition management"
```

---

## Task 4: Add Backend Search Command (Simple Search)

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add search_entries command to lib.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchRequest {
    search_type: String,
    #[serde(default)]
    search_term: Option<String>,
    #[serde(default)]
    conditions: Option<Vec<SearchCondition>>,
    is_regex: bool,
    session_id: String,
}

#[derive(Deserialize)]
struct SearchCondition {
    term: String,
    operator: String,
}

#[derive(Serialize)]
struct SearchResult {
    id: i64,
    timestamp: String,
    line_number: i32,
    message: String,
}

#[tauri::command]
async fn search_entries(
    search_type: String,
    #[serde(default)]
    search_term: Option<String>,
    #[serde(default)]
    conditions: Option<Vec<SearchCondition>>,
    is_regex: bool,
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let db_manager = state.db_manager.lock()
        .map_err(|e| e.to_string())?;

    let mut query = String::from(
        "SELECT id, timestamp, line_number, message
         FROM log_entries
         WHERE test_session_id = ?"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(session_id)];

    match search_type.as_str() {
        "simple" => {
            if let Some(term) = search_term {
                if is_regex {
                    query.push_str(" AND message REGEXP ?");
                } else {
                    query.push_str(" AND message LIKE ?");
                }
                params.push(Box::new(format!("%{}%", term)));
            }
        }
        "advanced" => {
            // Will implement in next task
            return Err("Advanced search not implemented yet".to_string());
        }
        _ => return Err("Invalid search type".to_string())
    }

    let conn = &db_manager.conn;
    let mut stmt = conn.prepare(&query)
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    let mut rows = stmt.query(params.as_slice())
        .map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        results.push(SearchResult {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            line_number: row.get(2)?,
            message: row.get(3)?,
        });
    }

    Ok(results)
}
```

**Step 2: Verify Cargo build**

Run: `cd src-tauri && cargo build`
Expected: Success (regex dependency already exists)

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add search_entries command for simple search"
```

---

## Task 5: Implement Advanced Search in Backend

**Files:**
- Modify: `src-tauri/src/lib.rs` (update search_entries command)

**Step 1: Update search_entries to support advanced search**

```rust
// Replace the "advanced" match arm in search_entries with:
"advanced" => {
    if let Some(conds) = conditions {
        if conds.is_empty() {
            return Err("At least one condition required".to_string());
        }
        query.push_str(" AND (");
        for (i, cond) in conds.iter().enumerate() {
            if i > 0 {
                query.push_str(&format!(" {} ", cond.operator));
            }
            if is_regex {
                query.push_str("message REGEXP ?");
            } else {
                query.push_str("message LIKE ?");
            }
            params.push(Box::new(format!("%{}%", cond.term)));
        }
        query.push_str(")");
    } else {
        return Err("Conditions required for advanced search".to_string());
    }
}
```

**Step 2: Verify Cargo build**

Run: `cd src-tauri && cargo build`
Expected: Success

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add advanced search support with AND/OR logic"
```

---

## Task 6: Connect Frontend to Backend Search

**Files:**
- Modify: `src/components/SearchPanel.vue:script`

**Step 1: Add search execution functions**

```vue
<script setup>
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

// Define props for session ID
const props = defineProps({
  sessionId: {
    type: String,
    required: true
  }
})

// ... existing state ...

// Generate unique ID
function generateId() {
  return Date.now() + '_' + Math.random().toString(36).substr(2, 9)
}

// Execute simple search
async function executeSimpleSearch() {
  if (!searchState.simpleTerm.trim()) return

  loading.value = true
  try {
    const result = await invoke('search_entries', {
      searchType: 'simple',
      searchTerm: searchState.simpleTerm,
      isRegex: searchState.isRegexMode,
      sessionId: props.sessionId
    })

    addSearchResult({
      type: 'simple',
      term: searchState.simpleTerm,
      matches: result
    })
  } catch (error) {
    ElMessage.error('搜索失败: ' + error)
  } finally {
    loading.value = false
  }
}

// Execute advanced search
async function executeAdvancedSearch() {
  const validConditions = searchState.conditions.filter(c => c.term.trim())
  if (validConditions.length === 0) {
    ElMessage.warning('请输入至少一个搜索条件')
    return
  }

  loading.value = true
  try {
    const result = await invoke('search_entries', {
      searchType: 'advanced',
      conditions: validConditions,
      isRegex: searchState.isRegexMode,
      sessionId: props.sessionId
    })

    addSearchResult({
      type: 'advanced',
      conditions: validConditions,
      matches: result
    })
  } catch (error) {
    ElMessage.error('搜索失败: ' + error)
  } finally {
    loading.value = false
  }
}

// Add search result to history
function addSearchResult(request) {
  const searchRecord = {
    id: generateId(),
    timestamp: Date.now(),
    request: request,
    matches: request.matches,  // { id, timestamp, lineNumber, message }
    expanded: true
  }

  searchState.history.unshift(searchRecord)
  searchState.expandedSearchId = searchRecord.id
  showSearchResults.value = true
}
</script>
```

**Step 2: Update template to pass session prop and show loading**

```vue
<template>
  <div class="search-panel" v-loading="loading">
    <!-- existing content -->
  </div>
</template>
```

**Step 3: Verify build**

Run: `npm run build`
Expected: Success

**Step 4: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: connect search panel to backend API"
```

---

## Task 7: Implement Search Results Panel UI

**Files:**
- Modify: `src/components/SearchPanel.vue:template,script,style`

**Step 1: Add search results panel HTML**

```vue
<template>
  <div class="search-panel">
    <!-- ... existing search input and advanced search ... -->

    <!-- Search Results Panel -->
    <div class="search-results-container">
      <div class="search-results-header" @click="toggleSearchResults">
        <el-icon :class="{ 'is-collapsed': !showSearchResults }">
          <ArrowDown v-if="showSearchResults" />
          <ArrowUp v-else />
        </el-icon>
        <span class="results-title">搜索结果</span>
        <span class="results-count">({{ totalMatchCount }})</span>
        <el-button
          :icon="Delete"
          text
          size="small"
          @click.stop="clearSearchHistory">
          清除
        </el-button>
      </div>

      <el-collapse-transition>
        <div v-show="showSearchResults" class="search-results-content">
          <div v-for="search in searchState.history"
               :key="search.id"
               class="search-result-group">
            <div class="search-result-header" @click="toggleSearchGroup(search.id)">
              <el-icon>
                <ArrowRight v-if="!search.expanded" />
                <ArrowDown v-else />
              </el-icon>
              <span class="search-term">{{ displaySearchTerm(search) }}</span>
              <el-tag size="small" type="info">{{ search.matches.length }} 条</el-tag>
              <span class="search-time">{{ formatTime(search.timestamp) }}</span>
            </div>

            <el-collapse-transition>
              <div v-show="search.expanded" class="search-matches-list">
                <div v-for="match in search.matches"
                     :key="match.id"
                     class="search-match-item"
                     @click="jumpToEntry(match.id)">
                  <span class="match-line">{{ match.lineNumber }}</span>
                  <span class="match-message">{{ match.message }}</span>
                </div>
              </div>
            </el-collapse-transition>
          </div>
        </div>
      </el-collapse-transition>
    </div>
  </div>
</template>
```

**Step 2: Add results panel functions**

```vue
<script setup>
// ... existing code ...

function toggleSearchResults() {
  showSearchResults.value = !showSearchResults.value
}

function toggleSearchGroup(id) {
  const search = searchState.history.find(s => s.id === id)
  if (search) {
    search.expanded = !search.expanded
  }
}

function clearSearchHistory() {
  searchState.history = []
  showSearchResults.value = false
}

function displaySearchTerm(search) {
  if (search.request.type === 'simple') {
    return search.request.term
  } else {
    return search.request.conditions.map(c => c.term).join(' ' +
           search.request.conditions.map(c => c.operator).join(' ') + ' ')
  }
}

function formatTime(timestamp) {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

function jumpToEntry(entryId) {
  emit('jump-to-entry', entryId)
}
</script>
```

**Step 3: Add search results styles**

```vue
<style scoped>
/* ... existing styles ... */

.search-results-container {
  border-top: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
}

.search-results-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  background: var(--el-fill-color-light);
  user-select: none;
}

.search-results-header:hover {
  background: var(--el-fill-color);
}

.results-title {
  font-weight: 600;
}

.results-count {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.search-results-content {
  max-height: 300px;
  overflow-y: auto;
}

.search-result-group {
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.search-result-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
}

.search-result-header:hover {
  background: var(--el-fill-color-lighter);
}

.search-term {
  flex: 1;
  font-size: 13px;
}

.search-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.search-matches-list {
  padding: 0 16px 12px;
}

.search-match-item {
  display: flex;
  gap: 12px;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
}

.search-match-item:hover {
  background: var(--el-fill-color-light);
}

.match-line {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  min-width: 40px;
  text-align: right;
}

.match-message {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
```

**Step 4: Verify build**

Run: `npm run build`
Expected: Success

**Step 5: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: add search results panel UI"
```

---

## Task 8: Integrate SearchPanel into App.vue

**Files:**
- Modify: `src/App.vue` (header section and main layout)

**Step 1: Import and register SearchPanel component**

Find the imports section around line 34 and add:
```vue
import SearchPanel from './components/SearchPanel.vue'
```

**Step 2: Replace existing search input with SearchPanel**

Find the existing search input around line 1858-1865:
```vue
<!-- REMOVE THIS -->
<el-input
  v-model="searchTerm"
  placeholder="搜索日志内容..."
  :prefix-icon="Search"
  clearable
  style="width: 360px"
  @input="debouncedSearch">
</el-input>

<!-- REPLACE WITH -->
<SearchPanel
  :session-id="currentSession"
  @jump-to-entry="handleSearchResultJump" />
```

**Step 3: Add jump handler function**

Add this function in the script section:
```javascript
function handleSearchResultJump(entryId) {
  // Find the page where this entry is located
  invoke('find_entry_page', {
    sessionId: currentSession.value,
    entryId: entryId,
    itemsPerPage: options.itemsPerPage
  }).then(({ page }) => {
    if (page && page >= 1) {
      goToPage(page)
      // Highlight the entry after page loads
      nextTick(() => {
        highlightedEntryId.value = entryId
        const entryEl = document.querySelector(`.entry-id-${entryId}`)
        if (entryEl) {
          entryEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
        }
      })
    }
  }).catch(error => {
    ElMessage.error('跳转失败: ' + error)
  })
}
```

**Step 4: Add backend command for finding entry page**

Add to `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
fn find_entry_page(
    session_id: String,
    entry_id: i64,
    items_per_page: usize,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let db_manager = state.db_manager.lock()
        .map_err(|e| e.to_string())?;

    let conn = &db_manager.conn;

    // Count entries before this one (same ordering)
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM log_entries
         WHERE test_session_id = ? AND id < ?",
        params![session_id, entry_id],
    )?.map_err(|e| e.to_string())?;

    let page = (count as usize) / items_per_page + 1;
    Ok(page)
}
```

**Step 5: Verify full build**

Run: `npm run build`
Run: `cd src-tauri && cargo build`
Expected: Success for both

**Step 6: Commit**

```bash
git add src/App.vue src-tauri/src/lib.rs
git commit -m "feat: integrate SearchPanel into main app"
```

---

## Task 9: Add Match Highlighting in Search Results

**Files:**
- Modify: `src/components/SearchPanel.vue:template,script`

**Step 1: Add highlight function**

```vue
<script setup>
// ... existing code ...

function highlightMatch(message, searchRequest) {
  let patterns = []

  if (searchRequest.type === 'simple') {
    patterns = [searchRequest.term]
  } else {
    patterns = searchRequest.conditions.map(c => c.term)
  }

  let highlighted = message
  patterns.forEach(pattern => {
    if (searchState.isRegexMode) {
      try {
        const regex = new RegExp(`(${pattern})`, 'gi')
        highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>')
      } catch (e) {
        // Invalid regex, skip highlighting
      }
    } else {
      const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const regex = new RegExp(`(${escaped})`, 'gi')
      highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>')
    }
  })

  return highlighted
}
</script>
```

**Step 2: Update template to use highlighting**

```vue
<span class="match-message" v-html="highlightMatch(match.message, search.request)"></span>
```

**Step 3: Add highlight style**

```css
/* Add to style section */
.match-message :deep(.highlight) {
  background: var(--el-color-warning);
  color: var(--el-color-danger);
  font-weight: bold;
  padding: 0 2px;
  border-radius: 2px;
}
```

**Step 4: Verify build**

Run: `npm run build`
Expected: Success

**Step 5: Commit**

```bash
git add src/components/SearchPanel.vue
git commit -m "feat: add match highlighting in search results"
```

---

## Task 10: Final Testing and Polish

**Files:**
- All modified files

**Step 1: Manual testing checklist**

- Test simple search (text mode)
- Test simple search (regex mode)
- Test advanced search with AND conditions
- Test advanced search with OR conditions
- Test search results panel expand/collapse
- Test clicking search result jumps to correct entry
- Test search history accumulation
- Test clear search history
- Test edge cases (empty search, invalid regex)

**Step 2: Run full build and verify**

Run: `npm run build`
Run: `cd src-tauri && cargo build`
Expected: Success for both

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete search enhancement implementation"
```

---

## Testing Notes

**Manual Test Steps:**
1. Start application with a loaded log session
2. Type a search term and press Enter - verify results appear
3. Click regex toggle and try a regex pattern (e.g., `error.*\d+`)
4. Click advanced search and add multiple conditions
5. Verify AND/OR logic works correctly
6. Click search results in the panel - verify jump to entry
7. Test collapse/expand of search results panel

**Expected Behavior:**
- Simple search performs case-insensitive text search by default
- Regex mode respects regex patterns
- Advanced search combines conditions with AND/OR operators
- Search results panel shows all historical searches
- Clicking a search result jumps to that entry in the log table

---

## Summary

This plan implements:
1. **Regex search** with toggle button, default off
2. **Multi-condition search** with AND/OR logic
3. **Search results panel** showing multiple search results simultaneously

**Files Created/Modified:**
- New: `src/components/SearchPanel.vue`
- Modified: `src/App.vue`
- Modified: `src-tauri/src/lib.rs`
