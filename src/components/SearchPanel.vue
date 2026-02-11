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

// Toggle functions
function toggleRegexMode() {
  searchState.isRegexMode = !searchState.isRegexMode
}

function toggleAdvancedMode() {
  searchState.isAdvancedMode = !searchState.isAdvancedMode
}

// Placeholder for executeSimpleSearch (will implement in later task)
function executeSimpleSearch() {
  // TODO: Implement in backend integration task
}
</script>

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
</style>
