<script setup>
import { ref, watch, nextTick } from 'vue'
import { Delete, ArrowDown, ArrowUp, ArrowRight, ArrowLeft, Close } from '@element-plus/icons-vue'

const props = defineProps({
  searchHistory: {
    type: Array,
    default: () => []
  },
  totalMatchCount: {
    type: Number,
    default: 0
  },
  isRegexMode: {
    type: Boolean,
    default: false
  },
  isCaseSensitive: {
    type: Boolean,
    default: false
  },
  // Callback functions for parent to call
  onToggleSearchGroup: {
    type: Function,
    default: null
  },
  onClearSearchHistory: {
    type: Function,
    default: null
  }
})

const emit = defineEmits(['jump-to-entry', 'delete-search-result', 'clear-history'])

const showSearchResults = ref(true)
const searchResultsContentRef = ref(null)

function toggleSearchResults() {
  showSearchResults.value = !showSearchResults.value
}

function toggleSearchGroup(id) {
  emit('toggle-search-group', id)
}

function clearSearchHistory() {
  // Emit clear-history event to parent
  emit('clear-history')
}

function displaySearchTerm(search) {
  if (!search) return ''

  const request = search.request
  const searchType = request.type

  if (searchType === 'simple') {
    return request.term
  } else if (searchType === 'advanced') {
    // Show terms with operators between them
    const parts = []
    for (let i = 0; i < request.conditions.length; i++) {
      const cond = request.conditions[i]
      parts.push(`"${cond.term}"`)
      if (i < request.conditions.length - 1) {
        // Add the operator from the NEXT condition (which connects current and next)
        const nextCond = request.conditions[i + 1]
        if (nextCond && nextCond.operator) {
          parts.push(nextCond.operator)
        }
      }
    }
    return parts.join(' ')
  }
  return ''
}

function formatTime(timestamp) {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

function highlightMatch(message, searchRequest) {
  let patterns = []

  const searchType = searchRequest.type

  if (searchType === 'simple') {
    patterns = [searchRequest.term]
  } else if (searchType === 'advanced') {
    // Only get terms, not operators
    patterns = searchRequest.conditions.map(c => c.term)
  }

  let highlighted = message
  patterns.forEach(pattern => {
    if (props.isRegexMode) {
      try {
        const flags = props.isCaseSensitive ? 'g' : 'gi'
        const regex = new RegExp(`(${pattern})`, flags)
        highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>')
      } catch (e) {
        // Invalid regex, skip highlighting
      }
    } else {
      // Escape special regex characters for safe string matching
      const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const flags = props.isCaseSensitive ? 'g' : 'gi'
      const regex = new RegExp(`(${escaped})`, flags)
      highlighted = highlighted.replace(regex, '<span class="highlight">$1</span>')
    }
  })

  return highlighted
}

function jumpToEntry(entryId) {
  emit('jump-to-entry', entryId)
}

function handleDeleteSearchResult(searchId, resultIndex) {
  // Emit delete event to parent
  emit('delete-search-result', { searchId, resultIndex })
}

function handleDeleteSingleSearch(searchId) {
  // Emit delete event to parent to delete entire search result group
  // Emit as separate parameters to match handler signature
  emit('delete-search-result', searchId, undefined, true)
}

// Auto-scroll to latest search result when searchHistory updates
watch(() => props.searchHistory, (newHistory, oldHistory) => {
  if (!newHistory || newHistory.length === 0) return

  // Check if there's a new search result (different ID or length increased)
  const oldIds = oldHistory ? new Set(oldHistory.map(s => s.id)) : new Set()
  const hasNewResult = newHistory.some(s => !oldIds.has(s.id))

  if (hasNewResult) {
    console.log('[SearchResultsPanel] New search result detected, scrolling to top')
    // New search result added, scroll to top (newest result)
    nextTick(() => {
      setTimeout(() => {
        // Try multiple methods to scroll
        if (searchResultsContentRef.value) {
          console.log('[SearchResultsPanel] Using ref to scroll')
          searchResultsContentRef.value.scrollTop = 0
        }

        const container = document.querySelector('.search-results-content')
        if (container) {
          console.log('[SearchResultsPanel] Using querySelector to scroll, current scrollTop:', container.scrollTop)
          container.scrollTop = 0
          console.log('[SearchResultsPanel] After setting, scrollTop:', container.scrollTop)
        }
      }, 150)
    })
  }
}, { deep: true })
</script>

<template>
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
      <div v-show="showSearchResults" class="search-results-content" ref="searchResultsContentRef">
        <div v-for="search in searchHistory"
             :key="search.id"
             class="search-result-group">
          <div class="search-result-header">
            <div class="search-result-header-left" @click="toggleSearchGroup(search.id)">
              <el-icon>
                <ArrowRight v-if="!search.expanded" />
                <ArrowDown v-else />
              </el-icon>
              <span class="search-term">{{ displaySearchTerm(search) }}</span>
              <el-tag size="small" type="info">{{ search.matches.length }} 条</el-tag>
              <span class="search-time">{{ formatTime(search.timestamp) }}</span>
            </div>
            <el-button
              :icon="Close"
              text
              size="small"
              class="delete-search-btn"
              @click.stop="handleDeleteSingleSearch(search.id)">
            </el-button>
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
</template>

<style scoped>
.search-results-container {
  border-top: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
  margin-top: 0;
  border-radius: 0 0 8px 8px;
}

.search-results-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  background: var(--el-fill-color-light);
  user-select: none;
  border-radius: 0 0 8px 8px;
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
  overflow-x: auto;
}

.search-result-group {
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.search-result-group:last-child {
  border-bottom: none;
}

.search-result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 16px;
}

.search-result-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  cursor: pointer;
  min-width: 0;
}

.delete-search-btn {
  flex-shrink: 0;
  opacity: 0.6;
}

.delete-search-btn:hover {
  opacity: 1;
}

.search-result-header:hover {
  background: var(--el-fill-color-lighter);
}

.search-term {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
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
  min-width: 600px;
  flex: 0 0 auto;
}

.match-message :deep(.highlight) {
  background: var(--el-color-warning);
  color: var(--el-color-danger);
  font-weight: bold;
  padding: 0 2px;
  border-radius: 2px;
}
</style>
