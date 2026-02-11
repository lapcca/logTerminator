<script setup>
import { ref, reactive, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'
import {
  Search, Operation, Grid, Memo, List,
  Plus, Delete, ArrowDown, ArrowUp, ArrowRight, ArrowLeft
} from '@element-plus/icons-vue'

const emit = defineEmits(['jump-to-entry'])

// Define props for session ID
const props = defineProps({
  sessionId: {
    type: String,
    required: true
  }
})

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

// Generate unique ID
function generateId() {
  return Date.now() + '_' + Math.random().toString(36).substr(2, 9)
}

// Computed
const totalMatchCount = computed(() =>
  searchState.history.reduce((sum, s) => sum + s.matches.length, 0)
)

// Toggle functions
function toggleRegexMode() {
  searchState.isRegexMode = !searchState.isRegexMode
}

function toggleAdvancedMode() {
  searchState.isAdvancedMode = !searchState.isAdvancedMode
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

// Condition management functions
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

// Search Results Panel functions
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
    return search.request.conditions.map(c => c.term).join(' ') +
           search.request.conditions.map(c => c.operator).join(' ') + ' '
  }
}

function formatTime(timestamp) {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

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
      // Escape special regex characters for safe string matching
      const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const regex = new RegExp(`(${escaped})`, 'gi')
      highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>')
    }
  })

  return highlighted
}

function jumpToEntry(entryId) {
  emit('jump-to-entry', entryId)
}
</script>

<template>
  <div class="search-panel" v-loading="loading">
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
                  <span class="match-message" v-html="highlightMatch(match.message, search.request)"></span>
                </div>
              </div>
            </el-collapse-transition>
          </div>
        </div>
      </el-collapse-transition>
    </div>
  </div>
</template>

<style scoped>
.search-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

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

.match-message :deep(.highlight) {
  background: var(--el-color-warning);
  color: var(--el-color-danger);
  font-weight: bold;
  padding: 0 2px;
  border-radius: 2px;
}
</style>
