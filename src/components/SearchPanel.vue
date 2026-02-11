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
