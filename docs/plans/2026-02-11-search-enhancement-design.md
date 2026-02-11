# Search Enhancement Design

## Overview

Enhance the logTerminator search functionality with three major improvements:
1. Regular expression search support (toggleable, default off)
2. Multi-condition search with AND/OR logic
3. Search results panel showing multiple search results simultaneously (inspired by Notepad++)

## Component Structure

```
App.vue (main application)
├── SearchPanel.vue (new component)
│   ├── SearchInput.vue (search input area)
│   ├── AdvancedSearch.vue (advanced search area, collapsible)
│   └── SearchResultsPanel.vue (search results panel, collapsible)
```

## State Management

### SearchPanel.vue State

```javascript
const searchState = reactive({
  // Current simple search term
  simpleTerm: '',

  // Regex mode toggle
  isRegexMode: false,

  // Advanced search mode toggle
  isAdvancedMode: false,

  // Advanced search conditions
  conditions: [
    { id: generateId(), term: '', operator: 'AND' }
  ],

  // Search history
  history: [],

  // Currently expanded search result group ID
  expandedSearchId: null
})

const showSearchResults = ref(false)
```

## Search Input Area Design

### Visual Design

Element Plus `el-input` component with two icon buttons in the suffix slot:

- **Regex toggle button**: Uses `Operation` icon when off, `Grid` icon when on
- **Advanced search toggle button**: Uses `Memo` icon when off, `List` icon when on

```html
<el-input
  v-model="currentSearchTerm"
  placeholder="搜索日志内容..."
  :prefix-icon="Search"
  clearable
  class="search-input">
  <template #suffix>
    <el-tooltip content="正则表达式" placement="top">
      <el-icon
        :class="{ 'regex-active': searchState.isRegexMode }"
        @click="toggleRegexMode"
        class="search-icon-btn">
        <Operation v-if="!searchState.isRegexMode" />
        <Grid v-else />
      </el-icon>
    </el-tooltip>

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
```

### Styles

```css
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
```

## Advanced Search Area Design

### Visual Design

Uses `el-collapse-transition` with `el-card` for each search condition:

```html
<el-collapse-transition>
  <div v-show="searchState.isAdvancedMode" class="advanced-search-panel">
    <el-card shadow="never">
      <!-- Search condition rows -->
      <div v-for="(condition, index) in searchState.conditions"
           :key="condition.id"
           class="search-condition-row">

        <!-- AND/OR selector (not shown for first condition) -->
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

        <!-- Delete button (keep at least one condition) -->
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
```

### Styles

```css
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
```

## Search Results Panel Design

### Visual Design

Collapsible panel at the bottom of main content area, inspired by Notepad++:

```html
<div class="search-results-container">
  <!-- Collapse/expand header -->
  <div class="search-results-header" @click="toggleSearchResults">
    <el-icon :class="{ 'is-collapsed': !showSearchResults }">
      <ArrowDown v-if="showSearchResults" />
      <ArrowUp v-else />
    </el-icon>
    <span class="results-title">搜索结果</span>
    <span class="results-count">({{ totalMatchCount }})</span>
    <!-- Clear history button -->
    <el-button
      :icon="Delete"
      text
      size="small"
      @click.stop="clearSearchHistory" />
  </div>

  <el-collapse-transition>
    <div v-show="showSearchResults" class="search-results-content">
      <!-- Search result groups -->
      <div v-for="search in searchHistory"
           :key="search.id"
           class="search-result-group">

        <!-- Search info header -->
        <div class="search-result-header" @click="toggleSearchGroup(search.id)">
          <el-icon>
            <ArrowRight v-if="!search.expanded" />
            <ArrowDown v-else />
          </el-icon>
          <span class="search-term">{{ displaySearchTerm(search) }}</span>
          <el-tag size="small" type="info">{{ search.matches.length }} 条</el-tag>
          <span class="search-time">{{ formatTime(search.timestamp) }}</span>
        </div>

        <!-- Match items list -->
        <el-collapse-transition>
          <div v-show="search.expanded" class="search-matches-list">
            <div v-for="match in search.matches"
                 :key="match.entryId"
                 class="search-match-item"
                 @click="jumpToEntry(match.entryId)">
              <span class="match-line">{{ match.lineNumber }}</span>
              <span class="match-message" v-html="highlightMatch(match.message, search.patterns)" />
            </div>
          </div>
        </el-collapse-transition>
      </div>
    </div>
  </el-collapse-transition>
</div>
```

### Styles

```css
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

/* Highlight matched text */
.match-message :deep(.highlight) {
  background: var(--el-color-warning);
  color: var(--el-color-danger);
  font-weight: bold;
  padding: 0 2px;
  border-radius: 2px;
}
```

## Data Flow

### Search Execution Flow

```javascript
// Execute simple search
async function executeSimpleSearch() {
  if (!searchState.simpleTerm.trim()) return

  const searchRequest = {
    type: 'simple',
    term: searchState.simpleTerm,
    isRegex: searchState.isRegexMode,
    sessionId: currentSession.value
  }

  await performSearch(searchRequest)
}

// Execute advanced search
async function executeAdvancedSearch() {
  const validConditions = searchState.conditions
    .filter(c => c.term.trim())

  if (validConditions.length === 0) return

  const searchRequest = {
    type: 'advanced',
    conditions: validConditions.map(c => ({
      term: c.term,
      operator: c.operator
    })),
    isRegex: searchState.isRegexMode,
    sessionId: currentSession.value
  }

  await performSearch(searchRequest)
}

// Common search execution function
async function performSearch(request) {
  loading.value = true

  try {
    // Call backend search API
    const result = await invoke('search_entries', request)

    // Add to history
    const searchRecord = {
      id: generateId(),
      timestamp: Date.now(),
      request: request,
      matches: result.matches,  // { entryId, lineNumber, message, snippet }
      expanded: true
    }

    searchState.history.unshift(searchRecord)
    searchState.expandedSearchId = searchRecord.id

    showSearchResults.value = true
  } catch (error) {
    ElMessage.error('搜索失败: ' + error)
  } finally {
    loading.value = false
  }
}
```

### Event Communication

```javascript
// Emit event to parent component
const emit = defineEmits(['jump-to-entry'])

// Jump to matched entry
function jumpToEntry(entryId) {
  emit('jump-to-entry', entryId)
}
```

## Backend API Changes

### Rust Data Structures

```rust
#[derive(serde::Deserialize)]
struct SearchRequest {
    search_type: String,        // "simple" or "advanced"
    search_term: Option<String>,  // Simple search term
    conditions: Option<Vec<SearchCondition>>, // Advanced search conditions
    is_regex: bool,            // Regex mode flag
    session_id: String,
}

#[derive(serde::Deserialize)]
struct SearchCondition {
    term: String,
    operator: String,  // "AND" or "OR"
}

#[derive(serde::Serialize)]
struct SearchResult {
    id: String,
    timestamp: i64,
    line_number: i32,
    message: String,
    snippet: String,  // Context snippet around match
}
```

### Tauri Command

```rust
#[tauri::command]
async fn search_entries(
    search_type: String,
    search_term: Option<String>,
    conditions: Option<Vec<SearchCondition>>,
    is_regex: bool,
    session_id: String,
    db: DbState<'_>
) -> Result<Vec<SearchResult>, String> {
    let conn = db.lock()
        .map_err(|e| e.to_string())?;

    let mut query = String::from(
        "SELECT id, timestamp, message, line_number
         FROM log_entries
         WHERE session_id = ?"
    );
    let mut params: Vec<Box<dyn ToSql>> = vec![Box::new(session_id)];

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
            if let Some(conds) = conditions {
                for (i, cond) in conds.iter().enumerate() {
                    if i > 0 {
                        query.push_str(&format!(" {} ", cond.operator));
                    } else {
                        query.push_str(" AND (");
                    }

                    if is_regex {
                        query.push_str("message REGEXP ?");
                    } else {
                        query.push_str("message LIKE ?");
                    }
                    params.push(Box::new(format!("%{}%", cond.term)));
                }
                query.push_str(")");
            }
        }
        _ => return Err("Invalid search type".to_string())
    }

    let matches: Vec<SearchResult> = conn
        .prepare(&query)?
        .query_map(params.as_slice(), |row| {
            let message: String = row.get(2)?;
            Ok(SearchResult {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                message: message.clone(),
                line_number: row.get(3)?,
                snippet: extract_snippet(&message, &search_term),
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(matches)
}

// Extract context snippet around match (approximately 100 characters)
fn extract_snippet(message: &str, pattern: Option<&String>) -> String {
    // Implementation details...
    // Return message snippet containing matched text with context
}
```

### Dependencies

```toml
# Add to Cargo.toml
regex = "1.10"
```

## Implementation Notes

### Key Considerations

1. **Performance**: Search operations should be efficient - consider indexing for large log sets
2. **Regex Safety**: Validate regex patterns to prevent ReDoS attacks
3. **Search History**: Consider persisting search history to local storage
4. **UI/UX**: Maintain consistency with existing Element Plus components
5. **Error Handling**: Provide clear feedback for invalid regex patterns

### File Changes

**Frontend:**
- Create: `src/components/SearchPanel.vue`
- Modify: `src/App.vue` (integrate SearchPanel)

**Backend:**
- Modify: `src-tauri/src/lib.rs` (add search_entries command)
- Modify: `src-tauri/Cargo.toml` (add regex dependency)

## Future Enhancements

1. Search within specific log levels only
2. Export search results to file
3. Save/load search queries
4. Case-sensitive search toggle
5. Whole word search option
