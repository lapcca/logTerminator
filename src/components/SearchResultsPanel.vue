<script setup>
import { ref } from 'vue'
import { Delete, ArrowDown, ArrowUp, ArrowRight, ArrowLeft } from '@element-plus/icons-vue'

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
  }
})

const emit = defineEmits(['jump-to-entry', 'clear-history', 'toggle-search-group'])

const showSearchResults = ref(true)

function toggleSearchResults() {
  showSearchResults.value = !showSearchResults.value
}

function toggleSearchGroup(id) {
  emit('toggle-search-group', id)
}

function clearSearchHistory() {
  emit('clear-history')
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
    if (props.isRegexMode) {
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
        <div v-for="search in searchHistory"
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
