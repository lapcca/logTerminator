<script setup>
import { ref, reactive, onMounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Reactive data
const currentSession = ref('')
const logEntries = ref([])
const bookmarks = ref([])
const loading = ref(false)
const searchTerm = ref('')
const levelFilter = ref(null)
const totalEntries = ref(0)
const sessions = ref([])
const showSidebar = ref(true) // 控制左侧面板显示/隐藏
const showBookmarksPanel = ref(true) // 控制书签面板展开/折叠
const showSessionsPanel = ref(true) // 控制测试会话面板展开/折叠
const selectedEntryIds = ref([]) // 选中的日志条目ID
const highlightedEntryId = ref(null) // 当前高亮的条目ID

// Toggle functions
function toggleBookmarksPanel() {
  showBookmarksPanel.value = !showBookmarksPanel.value
}

function toggleSessionsPanel() {
  showSessionsPanel.value = !showSessionsPanel.value
}

// Data table options
const options = reactive({
  page: 1,
  itemsPerPage: 50,
  sortBy: ['timestamp'],
  sortDesc: [false]
})

// Log levels for filtering
const logLevels = ['ERROR', 'WARNING', 'INFO', 'DEBUG', 'TRACE']

// Table headers
const headers = [
  { title: '', key: 'data-table-select', width: '56px', sortable: false, align: 'center' },
  { title: '时间戳', key: 'timestamp', width: '180px', sortable: true },
  { title: '级别', key: 'level', width: '90px', sortable: true },
  { title: '调用栈', key: 'stack', width: '250px', sortable: false },
  { title: '消息', key: 'message', minWidth: '300px' },
  { title: '书签', key: 'bookmarked', width: '80px', sortable: false, align: 'center' }
]

// Computed selected entries
const selectedEntries = computed(() => {
  return logEntries.value.filter(entry => selectedEntryIds.value.includes(entry.id))
})

// Check if all entries on current page are selected
const allSelected = computed(() => {
  return logEntries.value.length > 0 && 
         logEntries.value.every(entry => selectedEntryIds.value.includes(entry.id))
})

// Check if some entries on current page are selected
const someSelected = computed(() => {
  return logEntries.value.some(entry => selectedEntryIds.value.includes(entry.id))
})

// Open log directory
async function openDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择日志目录'
    })

    if (selected) {
      loading.value = true
      selectedEntryIds.value = [] // 清空选择
      
      try {
        currentSession.value = await invoke('parse_log_directory', { directoryPath: selected })
        await loadSessions()
        await refreshLogs()
      } catch (error) {
        console.error('Error processing directory:', error)
        alert(`处理目录时出错：${error}`)
      } finally {
        loading.value = false
      }
    }
  } catch (error) {
    console.error('Error opening directory:', error)
  }
}

// Load test sessions
async function loadSessions() {
  try {
    sessions.value = await invoke('get_sessions')
  } catch (error) {
    console.error('Error loading sessions:', error)
  }
}

// Refresh log entries
async function refreshLogs() {
  if (!currentSession.value) return

  loading.value = true
  try {
    const result = await invoke('get_log_entries', {
      sessionId: currentSession.value,
      offset: (options.page - 1) * options.itemsPerPage,
      limit: options.itemsPerPage,
      levelFilter: levelFilter.value,
      searchTerm: searchTerm.value
    })

    const [entries, total] = result
    logEntries.value = entries || []
    totalEntries.value = total || 0

    await loadBookmarks()
  } catch (error) {
    console.error('Error fetching logs:', error)
    alert(`获取日志时出错：${error}`)
  } finally {
    loading.value = false
  }
}

// Load bookmarks for current session
async function loadBookmarks() {
  if (!currentSession.value) return

  try {
    bookmarks.value = await invoke('get_bookmarks', { sessionId: currentSession.value })
  } catch (error) {
    console.error('Error loading bookmarks:', error)
  }
}

// Check if entry is bookmarked
function isBookmarked(entryId) {
  return bookmarks.value.some(b => b[0].log_entry_id === entryId)
}

// Add bookmark
async function addBookmark(entry) {
  try {
    await invoke('add_bookmark', {
      logEntryId: entry.id,
      title: `书签 ${entry.timestamp}`,
      color: 'amber'
    })
    await loadBookmarks()
  } catch (error) {
    console.error('Error adding bookmark:', error)
    alert(`添加书签时出错：${error}`)
  }
}

// Remove bookmark
async function removeBookmark(entryId) {
  try {
    // Find and delete the bookmark
    const bookmark = bookmarks.value.find(b => b[0].log_entry_id === entryId)
    if (bookmark) {
      // Note: Need to add delete_bookmark command to backend
      // For now, just reload bookmarks
      await loadBookmarks()
    }
  } catch (error) {
    console.error('Error removing bookmark:', error)
  }
}

// Toggle bookmark
async function toggleBookmark(entry) {
  if (isBookmarked(entry.id)) {
    await removeBookmark(entry.id)
  } else {
    await addBookmark(entry)
  }
}

// Add bookmarks for selected entries
async function addBookmarksForSelected() {
  if (selectedEntryIds.value.length === 0) {
    alert('请先选择日志条目')
    return
  }

  try {
    for (const entry of selectedEntries.value) {
      if (!isBookmarked(entry.id)) {
        await invoke('add_bookmark', {
          logEntryId: entry.id,
          title: `书签 ${entry.timestamp}`,
          color: 'amber'
        })
      }
    }
    await loadBookmarks()
  } catch (error) {
    console.error('Error adding bookmarks:', error)
    alert(`批量添加书签时出错：${error}`)
  }
}

// Clear selection
function clearSelection() {
  selectedEntryIds.value = []
}

// Toggle select all
function toggleSelectAll() {
  if (allSelected.value) {
    // Deselect all on current page
    selectedEntryIds.value = selectedEntryIds.value.filter(
      id => !logEntries.value.find(e => e.id === id)
    )
  } else {
    // Select all on current page
    const newIds = logEntries.value
      .filter(entry => !selectedEntryIds.value.includes(entry.id))
      .map(entry => entry.id)
    selectedEntryIds.value = [...selectedEntryIds.value, ...newIds]
  }
}

// Toggle row selection
function toggleRowSelection(entry) {
  const index = selectedEntryIds.value.indexOf(entry.id)
  if (index > -1) {
    selectedEntryIds.value.splice(index, 1)
  } else {
    selectedEntryIds.value.push(entry.id)
  }
}

// Get level color for chips
function getLevelColor(level) {
  const colors = {
    'ERROR': 'error',
    'WARNING': 'warning',
    'INFO': 'info',
    'DEBUG': 'grey',
    'TRACE': 'purple'
  }
  return colors[level] || 'grey'
}

// Handle pagination changes
function handlePagination() {
  refreshLogs()
}

// Debounced search
let searchTimeout
function debouncedSearch() {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    options.page = 1
    refreshLogs()
  }, 300)
}

// Jump to bookmark with animation
async function jumpToBookmark(bookmark) {
  const bookmarkData = bookmark[1]
  if (!bookmarkData || !bookmarkData.timestamp) return

  try {
    // Find the entry in current page first
    let targetEntry = logEntries.value.find(e => e.timestamp === bookmarkData.timestamp)
    
    if (!targetEntry) {
      // Need to search in database - for now, show message
      alert(`未在当前页面找到该书签对应的日志条目`)
      return
    }

    // Calculate target page
    const entryIndex = logEntries.value.findIndex(e => e.timestamp === bookmarkData.timestamp)
    const targetPage = Math.floor(entryIndex / options.itemsPerPage) + 1
    
    // Navigate to page if needed
    if (targetPage !== options.page) {
      options.page = targetPage
      await refreshLogs()
      // Wait for render then highlight
      await nextTick()
      setTimeout(() => highlightAndScroll(targetEntry.id), 150)
    } else {
      highlightAndScroll(targetEntry.id)
    }
  } catch (error) {
    console.error('Error jumping to bookmark:', error)
  }
}

// Highlight and scroll to entry
function highlightAndScroll(entryId) {
  highlightedEntryId.value = entryId
  
  // Scroll to element
  const element = document.querySelector(`[data-entry-id="${entryId}"]`)
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
  
  // Remove highlight after animation
  setTimeout(() => {
    highlightedEntryId.value = null
  }, 3000)
}

// Check if entry is highlighted
function isHighlighted(entryId) {
  return highlightedEntryId.value === entryId
}

// Load sessions on mount
onMounted(() => {
  loadSessions()
})
</script>

<template>
  <v-app>
    <!-- App Bar -->
    <v-app-bar app color="primary" dark elevation="2">
      <v-btn
        icon
        variant="tonal"
        size="large"
        @click="showSidebar = !showSidebar"
        class="mr-3"
        :title="showSidebar ? '收起左侧面板' : '展开左侧面板'"
        style="background: rgba(255,255,255,0.15);">
        <v-icon size="28">{{ showSidebar ? 'mdi-arrow-collapse-horizontal' : 'mdi-arrow-expand-horizontal' }}</v-icon>
      </v-btn>
      
      <v-icon class="mr-2" size="28">mdi-file-document-multiple-outline</v-icon>
      <span class="text-h6 font-weight-medium">日志查看器</span>
      
      <v-spacer></v-spacer>
      
      <v-btn
        color="white"
        variant="outlined"
        prepend-icon="mdi-folder-open"
        :loading="loading"
        @click="openDirectory">
        打开目录
      </v-btn>
    </v-app-bar>

    <v-main class="bg-grey-lighten-4">
      <v-container fluid class="pa-4">
        <v-row>
          <!-- Left Sidebar -->
          <v-expand-x-transition>
            <v-col v-if="showSidebar" cols="3" class="pr-4">
              <!-- Bookmarks Panel -->
            <v-card class="mb-3" elevation="2">
              <v-card-title class="d-flex align-center py-2 px-4 bg-amber-lighten-5">
                <v-btn
                  icon
                  variant="text"
                  size="small"
                  @click="showBookmarksPanel = !showBookmarksPanel"
                  :title="showBookmarksPanel ? '收起书签面板' : '展开书签面板'"
                  class="mr-1">
                  <v-icon :class="{ 'rotate-180': !showBookmarksPanel }" size="20">
                    mdi-chevron-down
                  </v-icon>
                </v-btn>
                <v-icon color="amber-darken-2" class="mr-2">mdi-bookmark-multiple</v-icon>
                <span class="font-weight-medium">书签</span>
                <v-chip size="small" color="amber" variant="flat" class="ml-2">
                  {{ bookmarks.length }}
                </v-chip>
              </v-card-title>
              <v-expand-transition>
                <div v-show="showBookmarksPanel">
                  <v-divider></v-divider>
                  <v-card-text class="pa-0" style="max-height: 300px; overflow-y: auto;">
                    <v-list v-if="bookmarks.length > 0" density="comfortable">
                      <v-list-item
                        v-for="bookmark in bookmarks"
                        :key="bookmark[0]?.id"
                        @click="jumpToBookmark(bookmark)"
                        class="bookmark-item px-3"
                        rounded="sm">
                        <template v-slot:prepend>
                          <v-avatar color="amber-lighten-3" size="32" class="mr-3">
                            <v-icon color="amber-darken-2" size="small">mdi-bookmark</v-icon>
                          </v-avatar>
                        </template>
                        <v-list-item-title class="text-body-2 font-weight-medium">
                          {{ bookmark[0]?.title || '书签' }}
                        </v-list-item-title>
                        <v-list-item-subtitle class="text-caption">
                          {{ bookmark[1]?.timestamp }}
                        </v-list-item-subtitle>
                      </v-list-item>
                    </v-list>
                    <div v-else class="pa-6 text-center text-grey">
                      <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-bookmark-outline</v-icon>
                      <div class="text-body-2">暂无书签</div>
                      <div class="text-caption">点击日志条目旁的星号添加书签</div>
                    </div>
                  </v-card-text>
                </div>
              </v-expand-transition>
            </v-card>

            <!-- Sessions Panel -->
            <v-card elevation="2">
              <v-card-title class="d-flex align-center py-2 px-4 bg-blue-grey-lighten-5">
                <v-btn
                  icon
                  variant="text"
                  size="small"
                  @click="showSessionsPanel = !showSessionsPanel"
                  :title="showSessionsPanel ? '收起测试会话面板' : '展开测试会话面板'"
                  class="mr-1">
                  <v-icon :class="{ 'rotate-180': !showSessionsPanel }" size="20">
                    mdi-chevron-down
                  </v-icon>
                </v-btn>
                <v-icon color="blue-grey" class="mr-2">mdi-folder-multiple</v-icon>
                <span class="font-weight-medium">测试会话</span>
                <v-chip size="small" color="blue-grey" variant="flat" class="ml-2">
                  {{ sessions.length }}
                </v-chip>
              </v-card-title>
              <v-expand-transition>
                <div v-show="showSessionsPanel">
                  <v-divider></v-divider>
                  <v-card-text class="pa-0" style="max-height: calc(100vh - 500px); overflow-y: auto;">
                    <v-list v-if="sessions.length > 0" density="comfortable">
                      <v-list-item
                        v-for="session in sessions"
                        :key="session.id"
                        :class="{ 'bg-primary-lighten-5': currentSession === session.id }"
                        @click="currentSession = session.id; refreshLogs()"
                        class="session-item px-3"
                        rounded="sm">
                        <template v-slot:prepend>
                          <v-avatar 
                            :color="currentSession === session.id ? 'primary' : 'grey-lighten-2'" 
                            size="32" 
                            class="mr-3">
                            <v-icon 
                              :color="currentSession === session.id ? 'white' : 'grey'" 
                              size="small">
                              mdi-folder
                            </v-icon>
                          </v-avatar>
                        </template>
                        <v-list-item-title class="text-body-2 font-weight-medium">
                          {{ session.name }}
                        </v-list-item-title>
                        <v-list-item-subtitle class="text-caption">
                          {{ session.total_entries }} 条记录
                        </v-list-item-subtitle>
                      </v-list-item>
                    </v-list>
                    <div v-else class="pa-6 text-center text-grey">
                      <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-folder-outline</v-icon>
                      <div class="text-body-2">暂无会话</div>
                      <div class="text-caption">打开目录加载日志</div>
                    </div>
                  </v-card-text>
                </div>
              </v-expand-transition>
            </v-card>
          </v-col>
          </v-expand-x-transition>

          <!-- Main Content -->
          <v-col :cols="showSidebar ? 9 : 12">
            <!-- Filters Card -->
            <v-card class="mb-3" elevation="2">
              <v-card-text class="pa-4">
                <v-row align="center">
                  <v-col cols="12" md="3">
                    <v-select
                      v-model="levelFilter"
                      :items="logLevels"
                      label="日志级别"
                      clearable
                      variant="outlined"
                      density="comfortable"
                      prepend-inner-icon="mdi-filter"
                      bg-color="white"
                      @update:model-value="refreshLogs">
                    </v-select>
                  </v-col>
                  <v-col cols="12" md="5">
                    <v-text-field
                      v-model="searchTerm"
                      label="搜索日志内容..."
                      variant="outlined"
                      density="comfortable"
                      prepend-inner-icon="mdi-magnify"
                      clearable
                      bg-color="white"
                      @update:model-value="debouncedSearch">
                    </v-text-field>
                  </v-col>
                  <v-col cols="12" md="4" class="d-flex justify-end">
                    <v-btn
                      color="primary"
                      variant="elevated"
                      prepend-icon="mdi-refresh"
                      :loading="loading"
                      @click="refreshLogs">
                      刷新
                    </v-btn>
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>

            <!-- Selected Actions Bar (shown when items selected) -->
            <v-expand-transition>
              <v-card 
                v-if="selectedEntryIds.length > 0" 
                class="mb-3" 
                elevation="2"
                color="primary"
                variant="elevated">
                <v-card-text class="py-3 px-4 d-flex align-center">
                  <v-icon class="mr-3">mdi-checkbox-multiple-marked</v-icon>
                  <span class="font-weight-medium mr-4">
                    已选择 {{ selectedEntryIds.length }} 条记录
                  </span>
                  <v-spacer></v-spacer>
                  <v-btn
                    color="white"
                    variant="outlined"
                    class="mr-2"
                    prepend-icon="mdi-bookmark-plus"
                    @click="addBookmarksForSelected">
                    添加书签
                  </v-btn>
                  <v-btn
                    color="white"
                    variant="text"
                    prepend-icon="mdi-close"
                    @click="clearSelection">
                    取消选择
                  </v-btn>
                </v-card-text>
              </v-card>
            </v-expand-transition>

            <!-- Log Entries Table -->
            <v-card elevation="2">
              <v-card-title class="d-flex align-center py-3 px-4 bg-blue-lighten-5">
                <v-icon color="blue" class="mr-2">mdi-format-list-bulleted</v-icon>
                <span class="font-weight-medium">日志条目</span>
                <v-chip size="small" color="blue" variant="flat" class="ml-3">
                  {{ totalEntries }} 条
                </v-chip>
                <v-spacer></v-spacer>
                <span v-if="selectedEntryIds.length > 0" class="text-body-2 text-grey-darken-1">
                  已选 {{ selectedEntryIds.length }}
                </span>
              </v-card-title>
              
              <v-divider></v-divider>
              
              <v-card-text class="pa-0">
                <v-data-table-server
                  v-model="selectedEntryIds"
                  :headers="headers"
                  :items="logEntries"
                  :loading="loading"
                  :items-length="totalEntries"
                  :options.sync="options"
                  @update:options="handlePagination"
                  show-select
                  item-value="id"
                  class="log-table elevation-0"
                  density="comfortable"
                  fixed-header
                  hover
                  :height="showSidebar ? 'calc(100vh - 340px)' : 'calc(100vh - 220px)'">

                  <!-- Select All Checkbox -->
                  <template v-slot:header.data-table-select="{ on, modelValue, someSelected, allSelected }">
                    <v-checkbox
                      :model-value="allSelected"
                      :indeterminate="someSelected && !allSelected"
                      color="primary"
                      hide-details
                      @update:model-value="(val) => {
                        if (val) toggleSelectAll()
                        else if (someSelected) toggleSelectAll()
                      }">
                    </v-checkbox>
                  </template>

                  <!-- Row with click selection -->
                  <template v-slot:item="{ item, isSelected, toggleSelect }">
                    <tr 
                      :data-entry-id="item.id"
                      :class="{
                        'table-row-selected': isSelected,
                        'table-row-highlighted': isHighlighted(item.id),
                        'cursor-pointer': true
                      }"
                      @click="toggleRowSelection(item)">
                      <!-- Checkbox cell -->
                      <td class="text-center" @click.stop>
                        <v-checkbox
                          :model-value="isSelected"
                          color="primary"
                          hide-details
                          @update:model-value="toggleSelect">
                        </v-checkbox>
                      </td>
                      <!-- Timestamp -->
                      <td>
                        <span class="font-mono text-body-2">{{ item.timestamp }}</span>
                      </td>
                      <!-- Level -->
                      <td>
                        <v-chip
                          :color="getLevelColor(item.level)"
                          size="small"
                          variant="flat"
                          class="font-weight-medium">
                          {{ item.level }}
                        </v-chip>
                      </td>
                      <!-- Stack -->
                      <td>
                        <v-tooltip location="top">
                          <template v-slot:activator="{ props }">
                            <span 
                              v-bind="props" 
                              class="text-truncate d-block font-mono text-caption"
                              style="max-width: 240px; color: #666;">
                              {{ item.stack || '-' }}
                            </span>
                          </template>
                          <span class="font-mono">{{ item.stack }}</span>
                        </v-tooltip>
                      </td>
                      <!-- Message -->
                      <td>
                        <div 
                          class="text-body-2 text-wrap" 
                          style="word-break: break-word;">
                          {{ item.message }}
                        </div>
                      </td>
                      <!-- Bookmark action -->
                      <td class="text-center" @click.stop>
                        <v-btn
                          :icon="isBookmarked(item.id) ? 'mdi-star' : 'mdi-star-outline'"
                          :color="isBookmarked(item.id) ? 'amber' : 'grey'"
                          :variant="isBookmarked(item.id) ? 'flat' : 'text'"
                          size="small"
                          @click="toggleBookmark(item)"
                          :title="isBookmarked(item.id) ? '取消书签' : '添加书签'">
                        </v-btn>
                      </td>
                    </tr>
                  </template>

                  <!-- Empty state -->
                  <template v-slot:no-data>
                    <div class="text-center pa-8">
                      <v-icon size="64" color="grey-lighten-1" class="mb-4">mdi-text-search</v-icon>
                      <div class="text-h6 text-grey">暂无日志数据</div>
                      <div class="text-body-2 text-grey-darken-1 mt-2">
                        点击右上角"打开目录"加载日志文件
                      </div>
                    </div>
                  </template>

                </v-data-table-server>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>

<style scoped>
/* Font for timestamps */
.font-mono {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

/* Table row styles */
.cursor-pointer {
  cursor: pointer;
}

/* Selected row - bright yellow background */
.table-row-selected {
  background-color: rgba(255, 193, 7, 0.25) !important;
  border-left: 3px solid rgb(255, 193, 7) !important;
}

.table-row-selected:hover {
  background-color: rgba(255, 193, 7, 0.35) !important;
}

/* Highlighted row for bookmark jump */
.table-row-highlighted {
  background-color: rgba(33, 150, 243, 0.3) !important;
  border-left: 3px solid rgb(33, 150, 243) !important;
  animation: pulse-highlight 1.5s ease-in-out 3;
}

@keyframes pulse-highlight {
  0%, 100% { background-color: rgba(33, 150, 243, 0.3); }
  50% { background-color: rgba(33, 150, 243, 0.5); }
}

/* Table hover effect */
:deep(.v-data-table__tr:hover) {
  background-color: rgba(0, 0, 0, 0.04) !important;
}

/* Checkbox size */
:deep(.v-checkbox) {
  transform: scale(1.1);
}

/* Bookmark/Session list items */
.bookmark-item,
.session-item {
  transition: all 0.2s ease;
  margin: 2px 8px;
  border-radius: 8px;
}

.bookmark-item:hover {
  background-color: rgba(255, 193, 7, 0.15);
}

.session-item:hover {
  background-color: rgba(25, 118, 210, 0.1);
}

/* Card title adjustments */
:deep(.v-card-title) {
  font-size: 0.95rem;
}

/* Panel toggle button rotation animation */
.rotate-180 {
  transform: rotate(180deg);
  transition: transform 0.3s ease;
}

/* Responsive adjustments */
@media (max-width: 960px) {
  .v-col-3 {
    flex: 0 0 100%;
    max-width: 100%;
  }
  
  .v-col-9,
  .v-col-12 {
    flex: 0 0 100%;
    max-width: 100%;
  }
}
</style>
