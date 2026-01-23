<script setup>
import { ref, reactive, onMounted, computed, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'

// Reactive data
const currentSession = ref('')
const logEntries = ref([])
const bookmarks = ref([])
const loading = ref(false)
const loadingMessage = ref('') // 显示加载状态信息
const searchTerm = ref('')
const levelFilter = ref('ALL')
const totalEntries = ref(0)
const sessions = ref([])
const showSidebar = ref(true) // 控制左侧面板显示/隐藏
const showBookmarksPanel = ref(true) // 控制书签面板展开/折叠
const showSessionsPanel = ref(true) // 控制测试会话面板展开/折叠
const selectedEntryIds = ref([]) // 选中的日志条目ID
const highlightedEntryId = ref(null) // 当前高亮的条目ID
const jumpToPage = ref(1) // 跳转到页码输入框的值

// Sidebar width management
const sidebarWidth = ref(300)  // Default width in pixels
const isResizing = ref(false)  // Track if user is dragging

// Log source dialog
const showSourceDialog = ref(false)
const sourceType = ref('folder') // 'folder' or 'url'
const httpUrl = ref('')
const selectedFolderPath = ref('')

// Helper for dialog
const canOpen = computed(() => {
  if (sourceType.value === 'folder') {
    return selectedFolderPath.value !== ''
  } else {
    return httpUrl.value !== '' && httpUrl.value.match(/^https?:\/\//)
  }
})

// Bookmark editing
const showEditBookmarkDialog = ref(false) // 控制编辑书签对话框显示
const editingBookmark = ref(null) // 当前编辑的书签
const editingBookmarkTitle = ref('') // 编辑时的临时标题

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

// Items per page options
const itemsPerPageOptions = [
  { title: '25 条/页', value: 25 },
  { title: '50 条/页', value: 50 },
  { title: '100 条/页', value: 100 },
  { title: '200 条/页', value: 200 },
  { title: '500 条/页', value: 500 },
]

// Log levels for filtering
const logLevels = ['ALL', 'ERROR', 'WARNING', 'INFO', 'DEBUG', 'TRACE']

// Priority order for log levels (higher priority first)
const levelPriority = {
  'ERROR': 5,
  'WARNING': 4,
  'INFO': 3,
  'DEBUG': 2,
  'TRACE': 1
}

// Dynamic log levels based on current session data
const dynamicLogLevels = computed(() => {
  if (!currentSession.value || logEntries.value.length === 0) {
    return ['ALL']
  }
  // Get unique levels from current log entries
  const uniqueLevels = [...new Set(logEntries.value.map(entry => entry.level))]
  // Sort by priority (higher priority first)
  const sortedLevels = uniqueLevels.sort((a, b) => {
    const priorityA = levelPriority[a] || 0
    const priorityB = levelPriority[b] || 0
    return priorityB - priorityA
  })
  // Add ALL option at the beginning
  return ['ALL', ...sortedLevels]
})

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

// Computed page count
const totalPages = computed(() => Math.ceil(totalEntries.value / options.itemsPerPage))

// Show source dialog
async function openDirectory() {
  showSourceDialog.value = true
  sourceType.value = 'folder'
  selectedFolderPath.value = ''
  httpUrl.value = ''
}

// Select local folder
async function selectLocalFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Log Directory'
    })
    if (selected) {
      selectedFolderPath.value = selected
    }
  } catch (error) {
    console.error('Error selecting folder:', error)
  }
}

// Open log source (folder or URL)
async function openLogSource() {
  showSourceDialog.value = false

  if (sourceType.value === 'url' && httpUrl.value) {
    await loadFromHttpUrl(httpUrl.value)
  } else if (sourceType.value === 'folder' && selectedFolderPath.value) {
    await loadFromDirectory(selectedFolderPath.value)
  }
}

// Load from HTTP URL
async function loadFromHttpUrl(url) {
  loading.value = true
  loadingMessage.value = 'Connecting to server...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_http_url', { url })
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s) from server`
    }
  } catch (error) {
    console.error('Error loading from HTTP:', error)
    let userMsg = error
    if (error.includes('InvalidUrl')) {
      userMsg = 'Invalid URL format. Please enter a valid HTTP/HTTPS URL.'
    } else if (error.includes('Timeout')) {
      userMsg = 'Request timed out. The server may be slow or unreachable.'
    } else if (error.includes('DirectoryListingNotFound')) {
      userMsg = 'Could not find directory listing. Check that the URL points to a directory.'
    }
    alert(`Failed to load logs: ${userMsg}`)
    loadingMessage.value = ''
  } finally {
    setTimeout(() => {
      loading.value = false
      loadingMessage.value = ''
    }, 500)
  }
}

// Load from local directory
async function loadFromDirectory(directoryPath) {
  loading.value = true
  loadingMessage.value = 'Scanning directory...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_directory', { directoryPath })
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s)`
    }
  } catch (error) {
    console.error('Error loading from directory:', error)
    alert(`Error loading directory: ${error}`)
    loadingMessage.value = ''
  } finally {
    setTimeout(() => {
      loading.value = false
      loadingMessage.value = ''
    }, 500)
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

// Delete test session
async function deleteSession(sessionId, event) {
  // Stop event propagation to prevent parent click handlers
  if (event) {
    event.stopPropagation()
  }

  const confirmed = confirm('确定要删除此测试会话及其所有日志吗？此操作不可撤销。')
  console.log('Delete confirmation result:', confirmed, 'for session:', sessionId)

  if (!confirmed) {
    console.log('Session deletion cancelled by user')
    return
  }

  try {
    console.log('Proceeding to delete session:', sessionId)
    await invoke('delete_session', { sessionId })
    console.log('Session deleted successfully:', sessionId)

    // If deleting current session, clear it
    if (currentSession.value === sessionId) {
      currentSession.value = ''
      logEntries.value = []
      totalEntries.value = 0
      bookmarks.value = []
    }
    await loadSessions()
  } catch (error) {
    console.error('Error deleting session:', error)
    alert(`删除会话时出错：${error}`)
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
    console.log('Total entries from backend:', total, 'Items per page:', options.itemsPerPage, 'Calculated totalPages:', Math.ceil(total / options.itemsPerPage))

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

// Truncate text for display
function truncateText(text, maxLength = 80) {
  if (!text) return '书签'
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength).trim() + '...'
}

// Add bookmark
async function addBookmark(entry) {
  try {
    await invoke('add_bookmark', {
      logEntryId: entry.id,
      title: truncateText(entry.message),
      color: 'amber'
    })
    await loadBookmarks()
  } catch (error) {
    console.error('Error adding bookmark:', error)
    alert(`添加书签时出错：${error}`)
  }
}

// Remove bookmark by ID
async function removeBookmarkById(bookmarkId) {
  try {
    await invoke('delete_bookmark', { bookmarkId })
    await loadBookmarks()
  } catch (error) {
    console.error('Error removing bookmark:', error)
    alert(`删除书签时出错：${error}`)
  }
}

// Remove bookmark
async function removeBookmark(entryId) {
  const bookmark = bookmarks.value.find(b => b[0].log_entry_id === entryId)
  if (bookmark && bookmark[0].id) {
    await removeBookmarkById(bookmark[0].id)
  }
}

// Show edit bookmark title dialog
function showEditBookmarkTitleDialog(bookmark) {
  editingBookmark.value = bookmark
  editingBookmarkTitle.value = bookmark[0]?.title || ''
  showEditBookmarkDialog.value = true
}

// Update bookmark title
async function updateBookmarkTitle(bookmarkId, newTitle) {
  try {
    await invoke('update_bookmark_title', { bookmarkId, title: newTitle })
    await loadBookmarks()
  } catch (error) {
    console.error('Error updating bookmark:', error)
    alert(`更新书签时出错：${error}`)
  }
}

// Confirm edit bookmark
function confirmEditBookmark() {
  if (editingBookmark.value) {
    const bookmarkId = editingBookmark.value[0]?.id
    if (bookmarkId) {
      updateBookmarkTitle(bookmarkId, editingBookmarkTitle.value)
    }
    showEditBookmarkDialog.value = false
    editingBookmark.value = null
    editingBookmarkTitle.value = ''
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
          title: truncateText(entry.message),
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

// Format stack trace for display (convert newlines to spaces)
function formatStack(stack) {
  if (!stack) return '-'
  return stack.replace(/\n/g, ' ').trim()
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

// Go to specific page
function goToPage(page) {
  if (page >= 1 && page <= totalPages.value) {
    options.page = page
    refreshLogs()
  }
}

// Execute page jump from input field
function executeJump() {
  let targetPage = jumpToPage.value
  if (targetPage < 1) targetPage = 1
  if (targetPage > totalPages.value) targetPage = totalPages.value
  goToPage(targetPage)
  jumpToPage.value = targetPage
}

// Calculate visible page numbers for pagination
function visiblePageNumbers(currentPage, totalPages) {
  const pages = []
  const delta = 2 // Number of pages to show around current page
  
  if (totalPages <= 7) {
    // Show all pages if total is small
    for (let i = 2; i < totalPages; i++) {
      pages.push(i)
    }
  } else {
    // Show pages around current page
    const start = Math.max(2, currentPage - delta)
    const end = Math.min(totalPages - 1, currentPage + delta)
    
    for (let i = start; i <= end; i++) {
      pages.push(i)
    }
  }
  
  return pages
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
  const bookmarkInfo = bookmark[0] // Bookmark object
  const bookmarkData = bookmark[1] // LogEntry object

  if (!bookmarkInfo || !bookmarkData || !bookmarkData.id) return

  try {
    // Get the page number for this entry
    const entryPage = await invoke('get_entry_page', {
      entryId: bookmarkData.id,
      itemsPerPage: options.itemsPerPage,
      levelFilter: levelFilter.value,
      searchTerm: searchTerm.value
    })

    if (entryPage === null) {
      alert('书签对应的日志条目不存在')
      return
    }

    // Navigate to the correct page
    if (entryPage !== options.page) {
      options.page = entryPage
      await refreshLogs()
      // Wait for render then highlight
      await nextTick()
      setTimeout(() => highlightAndScroll(bookmarkData.id), 150)
    } else {
      // Already on correct page, just highlight
      highlightAndScroll(bookmarkData.id)
    }
  } catch (error) {
    console.error('Error jumping to bookmark:', error)
    alert('跳转到书签时出错')
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

  // Listen for HTTP progress events
  listen('http-progress', (event) => {
    loadingMessage.value = event.payload
  })
})

// Watch dynamicLogLevels and reset levelFilter if current selection is not available
watch(dynamicLogLevels, (newLevels) => {
  if (levelFilter.value !== 'ALL' && !newLevels.includes(levelFilter.value)) {
    levelFilter.value = 'ALL'
  }
})
</script>

<template>
  <v-app>
    <!-- Log Source Dialog -->
    <v-dialog v-model="showSourceDialog" max-width="500">
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon class="mr-2">mdi-folder-open</v-icon>
          Open Log Source
        </v-card-title>
        <v-card-text class="pt-4">
          <v-radio-group v-model="sourceType">
            <v-radio label="Local Folder" value="folder"></v-radio>
            <v-btn
              v-if="sourceType === 'folder'"
              @click="selectLocalFolder"
              variant="outlined"
              prepend-icon="mdi-folder"
              class="ml-8 mb-4">
              {{ selectedFolderPath || 'Browse...' }}
            </v-btn>

            <v-radio label="HTTP Server" value="url" class="mt-4"></v-radio>
            <v-text-field
              v-if="sourceType === 'url'"
              v-model="httpUrl"
              label="Enter URL"
              variant="outlined"
              placeholder="http://logs.example.com/"
              prepend-inner-icon="mdi-web"
              class="ml-8"
              clearable>
            </v-text-field>
          </v-radio-group>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn variant="text" @click="showSourceDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            variant="flat"
            :disabled="!canOpen"
            @click="openLogSource">
            Open
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Edit Bookmark Dialog -->
    <v-dialog v-model="showEditBookmarkDialog" max-width="400">
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon class="mr-2">mdi-bookmark-edit</v-icon>
          编辑书签名称
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="editingBookmarkTitle"
            label="书签名称"
            variant="outlined"
            autofocus
            prepend-inner-icon="mdi-form-textbox"
            hide-details
            class="mb-4"
            @keyup.enter="confirmEditBookmark">
          </v-text-field>
          <div v-if="editingBookmark" class="text-caption text-grey">
            <v-icon size="14" class="mr-1">mdi-clock-outline</v-icon>
            日志时间: {{ editingBookmark[1]?.timestamp }}
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn variant="text" @click="showEditBookmarkDialog = false">取消</v-btn>
          <v-btn color="primary" variant="flat" @click="confirmEditBookmark">确定</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

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
      
      <!-- Loading indicator -->
      <v-progress-linear
        v-if="loading"
        indeterminate
        color="white"
        class="mr-4"
        style="width: 200px;">
      </v-progress-linear>
      <span v-if="loadingMessage" class="text-body-2 mr-4 text-white">{{ loadingMessage }}</span>
      
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
                        <template v-slot:append>
                          <v-btn
                            icon
                            variant="text"
                            size="small"
                            color="grey"
                            @click.stop="showEditBookmarkTitleDialog(bookmark)"
                            title="编辑书签名称">
                            <v-icon size="small">mdi-pencil</v-icon>
                          </v-btn>
                          <v-btn
                            icon
                            variant="text"
                            size="small"
                            color="grey"
                            @click.stop="removeBookmarkById(bookmark[0]?.id)"
                            title="删除书签">
                            <v-icon size="small">mdi-close</v-icon>
                          </v-btn>
                        </template>
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
                        @click="currentSession = session.id; levelFilter = 'ALL'; refreshLogs()"
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
                              {{ session.source_type === 'http' ? 'mdi-web' : 'mdi-folder' }}
                            </v-icon>
                          </v-avatar>
                        </template>
                        <v-list-item-title class="text-body-2 font-weight-medium">
                          {{ session.name }}
                        </v-list-item-title>
                        <v-list-item-subtitle class="text-caption">
                          {{ session.total_entries }} 条记录
                        </v-list-item-subtitle>
                        <template v-slot:append>
                          <v-btn
                            icon
                            variant="text"
                            size="small"
                            color="grey"
                            @click.stop="deleteSession(session.id, $event)"
                            title="删除会话">
                            <v-icon size="small">mdi-delete</v-icon>
                          </v-btn>
                        </template>
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
                  <v-col cols="12" md="2">
                    <v-select
                      v-model="options.itemsPerPage"
                      :items="itemsPerPageOptions"
                      label="每页显示"
                      variant="outlined"
                      density="comfortable"
                      prepend-inner-icon="mdi-format-list-numbered"
                      bg-color="white"
                      @update:model-value="options.page = 1; refreshLogs()">
                    </v-select>
                  </v-col>
                  <v-col cols="12" md="2">
                    <v-select
                      v-model="levelFilter"
                      :items="dynamicLogLevels"
                      label="日志级别"
                      variant="outlined"
                      density="comfortable"
                      prepend-inner-icon="mdi-filter"
                      bg-color="white"
                      @update:model-value="refreshLogs">
                    </v-select>
                  </v-col>
                  <v-col cols="12" md="4">
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


             <v-data-table-server
               v-model="selectedEntryIds"
               :headers="headers"
               :items="logEntries"
               :loading="loading"
               :items-length="totalEntries"
               :options.sync="options"
               @update:options="handlePagination"
               show-select
               show-pagination
               item-value="id"
               class="log-table"
               density="comfortable"
               fixed-header
               hover
               :height="showSidebar ? 'calc(100vh - 280px)' : 'calc(100vh - 160px)'">

               <!-- Pagination info slot -->
               <template v-slot:bottom="{ page, pageCount, prevPage, nextPage }">
                 <div class="d-flex align-center pa-3" style="width: 100%;">
                   <v-btn
                     icon
                     variant="text"
                     size="small"
                     :disabled="options.page === 1"
                     @click="goToPage(options.page - 1)"
                     class="mr-2">
                     <v-icon>mdi-chevron-left</v-icon>
                   </v-btn>

                   <!-- Page number buttons -->
                   <div class="d-flex align-center mx-1">
                     <v-btn
                       variant="text"
                       size="small"
                       :class="{ 'bg-primary-lighten-5': options.page === 1 }"
                       min-width="32"
                       @click="goToPage(1)"
                       :disabled="options.page === 1">
                       1
                     </v-btn>

                     <v-btn
                       v-if="options.page > 3"
                       variant="text"
                       size="small"
                       disabled
                       min-width="32"
                       class="px-1">
                       ...
                     </v-btn>

                     <v-btn
                       v-for="p in visiblePageNumbers(options.page, totalPages)"
                       :key="p"
                       variant="text"
                       size="small"
                       :class="{ 'bg-primary-lighten-5': options.page === p }"
                       min-width="32"
                       @click="goToPage(p)">
                       {{ p }}
                     </v-btn>

                     <v-btn
                       v-if="options.page < totalPages - 2"
                       variant="text"
                       size="small"
                       disabled
                       min-width="32"
                       class="px-1">
                       ...
                     </v-btn>

                     <v-btn
                       v-if="totalPages > 1"
                       variant="text"
                       size="small"
                       :class="{ 'bg-primary-lighten-5': options.page === totalPages }"
                       min-width="32"
                       @click="goToPage(totalPages)"
                       :disabled="options.page === totalPages">
                       {{ totalPages }}
                     </v-btn>
                   </div>

                   <v-btn
                     icon
                     variant="text"
                     size="small"
                     :disabled="options.page === totalPages"
                     @click="goToPage(options.page + 1)"
                     class="mr-4 ml-2">
                     <v-icon>mdi-chevron-right</v-icon>
                   </v-btn>

                   <v-divider vertical class="mx-2"></v-divider>

                   <!-- Page jump input -->
                   <div class="d-flex align-center mx-3">
                     <span class="text-body-2 mr-2">跳转到</span>
                     <v-text-field
                       v-model.number="jumpToPage"
                       @keyup.enter="executeJump()"
                       variant="outlined"
                       density="compact"
                       hide-details
                       single-line
                       style="width: 60px;"
                       type="number"
                       :min="1"
                       :max="totalPages">
                     </v-text-field>
                     <span class="text-body-2 ml-2">页</span>
                   </div>

                   <v-divider vertical class="mx-2"></v-divider>
                   <span class="text-body-2 text-grey ml-2">
                     共 {{ totalEntries }} 条记录
                   </span>
                 </div>
               </template>

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
                     <v-tooltip
                       location="top"
                       open-on-hover
                       close-on-content-click="false"
                       :open-delay="200"
                       :close-delay="300"
                       transition="fade-transition"
                       :disabled="!item.stack"
                       interactive>
                       <template v-slot:activator="{ props }">
                         <span
                           v-bind="props"
                           class="text-truncate d-block font-mono text-caption stack-cell"
                           :class="{ 'has-stack': item.stack }"
                           style="max-width: 240px; color: #666; display: block; min-height: 20px;">
                           {{ formatStack(item.stack) }}
                         </span>
                       </template>
                       <v-card v-if="item.stack" class="stack-tooltip-card" max-width="600" max-height="400" style="background: white;">
                         <v-card-title class="tooltip-header py-2 px-4" style="font-size: 13px; font-weight: 500; border-bottom: 1px solid rgba(0,0,0,0.08);">
                           <v-icon size="14" class="mr-2" style="color: #ffa726;">mdi-layers-stack</v-icon>
                           Stack Trace
                         </v-card-title>
                         <v-card-text class="font-mono text-caption tooltip-content pa-4" style="color: #424242; white-space: pre-wrap; overflow-y: auto; max-height: 340px;">
                           {{ item.stack }}
                         </v-card-text>
                       </v-card>
                     </v-tooltip>
                   </td>
                   <!-- Message -->
                   <td>
                     <v-tooltip
                       location="bottom"
                       max-width="800"
                       :open-delay="200"
                       :close-delay="300"
                       transition="fade-transition"
                       interactive>
                       <template v-slot:activator="{ props }">
                         <div
                           v-bind="props"
                           class="text-body-2 text-wrap message-cell"
                           style="word-break: break-word; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;">
                           {{ item.message }}
                         </div>
                       </template>
                       <v-card class="message-tooltip-card" max-width="800" max-height="500" style="background: white;">
                         <v-card-title class="tooltip-header py-2 px-4" style="font-size: 13px; font-weight: 500; border-bottom: 1px solid rgba(0,0,0,0.08);">
                           <v-icon size="14" class="mr-2" style="color: #42a5f5;">mdi-message-text</v-icon>
                           Message
                         </v-card-title>
                         <v-card-text class="text-body-2 tooltip-content pa-4" style="color: #424242; white-space: pre-wrap; overflow-y: auto; max-height: 440px;">
                           {{ item.message }}
                         </v-card-text>
                       </v-card>
                     </v-tooltip>
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

/* Stack cell styling */
.stack-cell {
  cursor: default;
  line-height: 1.4;
  padding: 2px 0;
}

.stack-cell.has-stack {
  cursor: help;
}

.stack-cell.has-stack:hover {
  background-color: rgba(158, 158, 158, 0.15);
  border-radius: 4px;
}

/* Stack tooltip card styling */
.stack-tooltip-card {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12) !important;
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 12px !important;
  overflow: hidden;
}

/* Message tooltip card styling */
.message-tooltip-card {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12) !important;
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 12px !important;
  overflow: hidden;
}

/* Tooltip header styling */
.tooltip-header {
  background: linear-gradient(135deg, #e3f2fd 0%, #f5f5f5 100%);
  color: #424242;
  letter-spacing: 0.3px;
}

/* Tooltip content styling */
.tooltip-content {
  line-height: 1.6;
  padding: 16px;
  background: #fafafa;
}

/* Custom scrollbar for tooltip content */
.tooltip-content::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.tooltip-content::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

.tooltip-content::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 4px;
  transition: background 0.2s ease;
}

.tooltip-content::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
}

/* Ensure tooltip content is readable */
.stack-tooltip-card .v-card-text,
.message-tooltip-card .v-card-text {
  line-height: 1.5;
}

/* Fade transition for tooltip */
.fade-transition-enter-active,
.fade-transition-leave-active {
  transition: opacity 0.2s ease;
}

.fade-transition-enter-from,
.fade-transition-leave-to {
  opacity: 0;
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
