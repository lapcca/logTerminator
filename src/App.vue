<script setup>
import { ref, reactive, onMounted, onUnmounted, computed, watch, nextTick, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  ArrowRight,
  ArrowLeft,
  Document,
  FolderOpened,
  Link,
  Folder,
  Edit,
  Delete,
  Star,
  StarFilled,
  Search,
  Refresh,
  Collection,
  Expand,
  Fold,
  Setting,
  Check,
  Close,
  Plus,
  Minus,
  DArrowLeft,
  DArrowRight,
  MoreFilled,
  InfoFilled,
  WarningFilled
} from '@element-plus/icons-vue'
import MessageTooltip from './components/MessageTooltip.vue'
import TestSelectionDialog from './components/TestSelectionDialog.vue'
import BookmarkColorPicker from './components/BookmarkColorPicker.vue'
import { parsePythonStackTrace, getStackPreview, isPythonStackTrace } from './utils/stackParser.js'
import { formatMessage } from './utils/messageFormatter.js'

// Custom logTerminator icon component
const LogTerminatorIcon = {
  render() {
    return h('svg', {
      viewBox: '0 0 32 32',
      xmlns: 'http://www.w3.org/2000/svg',
      style: { width: '100%', height: '100%' }
    }, [
      // Definitions
      h('defs', [
        // Background gradient
        h('linearGradient', {
          id: 'bgGrad',
          x1: '0%',
          y1: '0%',
          x2: '100%',
          y2: '100%'
        }, [
          h('stop', { offset: '0%', 'stop-color': '#667eea' }),
          h('stop', { offset: '50%', 'stop-color': '#764ba2' }),
          h('stop', { offset: '100%', 'stop-color': '#f093fb' })
        ]),
        // Document gradient
        h('linearGradient', {
          id: 'docGrad',
          x1: '0%',
          y1: '0%',
          x2: '100%',
          y2: '100%'
        }, [
          h('stop', { offset: '0%', 'stop-color': '#ffffff', 'stop-opacity': '0.95' }),
          h('stop', { offset: '100%', 'stop-color': '#f0f0f5', 'stop-opacity': '0.95' })
        ]),
        // Accent gradient
        h('linearGradient', {
          id: 'accentGrad',
          x1: '0%',
          y1: '0%',
          x2: '100%',
          y2: '0%'
        }, [
          h('stop', { offset: '0%', 'stop-color': '#667eea' }),
          h('stop', { offset: '100%', 'stop-color': '#764ba2' })
        ])
      ]),
      // Background
      h('rect', {
        width: '32',
        height: '32',
        rx: '7',
        fill: 'url(#bgGrad)'
      }),
      // Inner glow
      h('ellipse', {
        cx: '16',
        cy: '16',
        rx: '11',
        ry: '11',
        fill: 'white',
        opacity: '0.08'
      }),
      // Document
      h('path', {
        d: 'M8 5C7.4 5 7 5.4 7 6V22C7 22.6 7.4 23 8 23H20C20.6 23 21 22.6 21 22V9L17 5H8Z',
        fill: 'url(#docGrad)'
      }),
      // Document fold
      h('path', {
        d: 'M17 5V9H21',
        fill: '#e0e0ea',
        opacity: '0.6'
      }),
      // Code lines
      h('rect', {
        x: '10',
        y: '11',
        width: '8',
        height: '1.5',
        rx: '0.75',
        fill: 'url(#accentGrad)'
      }),
      h('rect', {
        x: '10',
        y: '13.5',
        width: '6',
        height: '1.5',
        rx: '0.75',
        fill: 'url(#accentGrad)',
        opacity: '0.8'
      }),
      h('rect', {
        x: '10',
        y: '16',
        width: '7',
        height: '1.5',
        rx: '0.75',
        fill: 'url(#accentGrad)',
        opacity: '0.9'
      }),
      h('rect', {
        x: '10',
        y: '18.5',
        width: '5',
        height: '1.5',
        rx: '0.75',
        fill: 'url(#accentGrad)',
        opacity: '0.7'
      }),
      // Decorative corner
      h('path', {
        d: 'M23 20L27 24L27 27L23 27L23 20Z',
        fill: 'white',
        opacity: '0.9'
      }),
      h('circle', {
        cx: '25',
        cy: '23.5',
        r: '1',
        fill: 'url(#accentGrad)'
      }),
      // Accent dots
      h('circle', {
        cx: '12',
        cy: '7.5',
        r: '1',
        fill: 'white',
        opacity: '0.9'
      }),
      h('circle', {
        cx: '15',
        cy: '7.5',
        r: '1',
        fill: 'white',
        opacity: '0.7'
      })
    ])
  }
}

// Reactive data
const currentSession = ref('')
const logEntries = ref([])
const bookmarks = ref([])
const loading = ref(false)
const loadingMessage = ref('') // 显示加载状态信息
const searchTerm = ref('')
const levelFilter = ref([]) // Multi-select for log levels
const sessionLogLevels = ref([]) // Store all log levels for the current session
const selectAllLevels = ref(false) // Track select all checkbox state
const totalEntries = ref(0)
const sessions = ref([])
const showSidebar = ref(true) // 控制左侧面板显示/隐藏
const showBookmarksPanel = ref(true) // 控制书签面板展开/折叠
const selectedEntryIds = ref([]) // 选中的日志条目ID
const highlightedEntryId = ref(null) // 当前高亮的条目ID
const jumpToPage = ref(1) // 跳转到页码输入框的值
const levelSelectRef = ref(null) // Ref for level filter select dropdown

// Message expand/collapse state
const expandedMessageIds = ref(new Set()) // Set of message IDs that are expanded
const MESSAGE_LINE_LIMIT = 20 // Max lines to show before collapsing
const CHAR_WIDTH = 9 // Approximate character width in pixels (for monospace font)
const COLUMN_WIDTH = 400 // Approximate message column width in pixels

// Calculate visual line count for a message using the correct formula:
// 临时行数 = max((字符数 * 单个字符宽度 / 当前message列的宽度), 1)
// 行数 = 临时行数 + 字符里的换行符个数
function calculateVisualLines(message) {
  if (!message) return 0

  const charCount = message.length

  // 计算临时行数（确保最少有一行）
  const tempLines = Math.max(Math.ceil(charCount * CHAR_WIDTH / COLUMN_WIDTH), 1)

  // 计算换行符个数
  const newlineCount = (message.match(/\n/g) || []).length

  // 总行数 = 临时行数 + 换行符个数
  return tempLines + newlineCount
}

// State to track if hovering on expand button (to disable tooltip)
const hoveringExpandButton = ref(false)

// Handle hover events on expand button
function handleExpandHover() {
  hoveringExpandButton.value = true
}

function handleExpandLeave() {
  hoveringExpandButton.value = false
}

// Pinned tooltip state
const pinnedTooltip = ref({
  visible: false,
  rowId: null,
  message: '',
  hasJson: false,
  viewMode: 'raw',
  position: { x: 0, y: 0 },
  size: { width: 600, height: 400 }
})

// Sidebar width management
const sidebarWidth = ref(300)  // Default width in pixels
const isResizing = ref(false)  // Track if user is dragging
const resizeStartX = ref(0)    // Mouse X position when drag starts
const resizeStartWidth = ref(0) // Sidebar width when drag starts

// Log source dialog
const showSourceDialog = ref(false)
const logSourceInput = ref('') // Combined input for both URL and folder path
const logSourceInputRef = ref(null) // Ref for the input element

// Detect input type based on content
const inputSourceType = computed(() => {
  const input = logSourceInput.value.trim()
  if (input.startsWith('http://') || input.startsWith('https://')) {
    return 'url'
  }
  return 'folder'
})

// Helper for dialog
const canOpen = computed(() => {
  const input = logSourceInput.value.trim()
  if (input === '') return false
  if (inputSourceType.value === 'url') {
    return input.match(/^https?:\/\//) !== null
  }
  return true // Any non-empty input is considered valid for folder
})

// Bookmark editing
const showEditBookmarkDialog = ref(false) // 控制编辑书签对话框显示
const editingBookmark = ref(null) // 当前编辑的书签
const editingBookmarkTitle = ref('') // 编辑时的临时标题

// Bookmark color picker
const bookmarkColorPickerVisible = ref(false) // 控制颜色选择器显示
const bookmarkColorPickerEntryId = ref(null) // 当前正在添加书签的条目ID

// Test selection dialog
const showTestSelectionDialog = ref(false)
const testScanResults = ref([])
const testScanLoading = ref(false)
const pendingSourcePath = ref('')
const pendingSourceIsHttp = ref(false)

// Toggle functions
function toggleBookmarksPanel() {
  showBookmarksPanel.value = !showBookmarksPanel.value
}

// Data table options
const options = reactive({
  page: 1,
  itemsPerPage: 100,
  sortBy: ['timestamp'],
  sortDesc: [false]
})

// 跟踪当前页码（用于分页滚动）
const currentPageForScroll = ref(1)

// Items per page options
const itemsPerPageOptions = [
  { title: '25 条/页', value: 25 },
  { title: '50 条/页', value: 50 },
  { title: '100 条/页', value: 100 },
  { title: '200 条/页', value: 200 },
  { title: '500 条/页', value: 500 },
]

// Priority order for log levels (higher priority first)
const levelPriority = {
  'ERROR': 5,
  'WARNING': 4,
  'INFO': 3,
  'DEBUG': 2,
  'TRACE': 1
}

// Computed property for sorted log levels
const sortedLogLevels = computed(() => {
  if (sessionLogLevels.value.length === 0) {
    return []
  }
  // Sort by priority (higher priority first)
  return [...sessionLogLevels.value].sort((a, b) => {
    const priorityA = levelPriority[a] || 0
    const priorityB = levelPriority[b] || 0
    return priorityB - priorityA
  })
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
  // Don't clear logSourceInput - keep the last loaded directory if available
  showSourceDialog.value = true
  // Focus will be handled by the @opened event handler
}

// Fetch all log levels for the current session
async function fetchSessionLogLevels() {
  if (!currentSession.value) {
    sessionLogLevels.value = []
    levelFilter.value = []
    return
  }

  try {
    sessionLogLevels.value = await invoke('get_session_log_levels', { sessionId: currentSession.value })
    // Default to selecting all levels except TRACE
    levelFilter.value = sessionLogLevels.value.filter(level => level !== 'TRACE')
    // Update selectAll checkbox: only true when ALL levels (including TRACE) are selected
    selectAllLevels.value = false
  } catch (error) {
    console.error('Error fetching session log levels:', error)
    sessionLogLevels.value = []
  }
}

// Handle session change from selector
async function onSessionChange() {
  // Close pinned tooltip when switching sessions
  closePinnedTooltip()

  await fetchSessionLogLevels() // Fetch all log levels for this session
  refreshLogs()
}

// Toggle all log levels selection
function toggleAllLevels() {
  if (selectAllLevels.value) {
    // Select all levels
    levelFilter.value = [...sessionLogLevels.value]
  } else {
    // Clear selection
    levelFilter.value = []
  }
}

// Start dragging the resizer
function startResize(event) {
  isResizing.value = true
  resizeStartX.value = event.clientX
  resizeStartWidth.value = sidebarWidth.value
  event.preventDefault()
}

// Handle mouse move during resize
function onMouseMove(event) {
  if (!isResizing.value) return

  const deltaX = event.clientX - resizeStartX.value
  const newWidth = resizeStartWidth.value + deltaX
  const minWidth = 200
  const maxWidth = window.innerWidth / 2

  sidebarWidth.value = Math.max(minWidth, Math.min(maxWidth, newWidth))
}

// Handle mouse up to stop resizing
function onMouseUp() {
  isResizing.value = false
}

// Load sidebar width from localStorage
function loadSidebarWidth() {
  const saved = localStorage.getItem('sidebarWidth')
  if (saved) {
    sidebarWidth.value = parseInt(saved)
  }
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
      logSourceInput.value = selected
    }
  } catch (error) {
    console.error('Error selecting folder:', error)
  }
}

// Handle Enter key in source dialog
async function handleSourceDialogEnter() {
  if (canOpen.value) {
    await openLogSource()
  }
}

// Open log source (folder or URL)
async function openLogSource() {
  showSourceDialog.value = false

  const input = logSourceInput.value.trim()
  pendingSourcePath.value = input
  pendingSourceIsHttp.value = inputSourceType.value === 'url'

  await scanTests(input, inputSourceType.value === 'url')
}

// Scan for available tests
async function scanTests(path, isHttp) {
  testScanLoading.value = true
  testScanResults.value = []

  try {
    let results
    if (isHttp) {
      results = await invoke('scan_log_http_url', { url: path })
    } else {
      results = await invoke('scan_log_directory', { directoryPath: path })
    }
    testScanResults.value = results

    if (results.length === 0) {
      alert(isHttp
        ? '未找到符合格式的test日志文件'
        : '未找到符合格式的test日志文件')
      return
    }

    // Show test selection dialog
    showTestSelectionDialog.value = true
  } catch (error) {
    console.error('Error scanning for tests:', error)
    let userMsg = error
    if (error.includes('InvalidUrl')) {
      userMsg = 'URL格式无效，请输入有效的HTTP/HTTPS地址'
    } else if (error.includes('Timeout')) {
      userMsg = '请求超时，服务器可能响应缓慢'
    } else if (error.includes('No test log files found')) {
      userMsg = '未找到符合格式的test日志文件'
    }
    alert(`扫描失败：${userMsg}`)
  } finally {
    testScanLoading.value = false
  }
}

// Handle test selection confirmation
async function handleTestSelectionConfirm(selectedTests) {
  console.log('[App] handleTestSelectionConfirm called with:', selectedTests)
  console.log('[App] pendingSourcePath:', pendingSourcePath.value)
  console.log('[App] pendingSourceIsHttp:', pendingSourceIsHttp.value)

  showTestSelectionDialog.value = false

  if (pendingSourceIsHttp.value) {
    await loadFromHttpUrl(pendingSourcePath.value, selectedTests)
  } else {
    await loadFromDirectory(pendingSourcePath.value, selectedTests)
  }
}

// Load from HTTP URL
async function loadFromHttpUrl(url, selectedTests) {
  console.log('[loadFromHttpUrl] url:', url)
  console.log('[loadFromHttpUrl] selectedTests:', selectedTests)
  console.log('[loadFromHttpUrl] selectedTests type:', typeof selectedTests)
  console.log('[loadFromHttpUrl] selectedTests isArray:', Array.isArray(selectedTests))

  loading.value = true
  loadingMessage.value = 'Connecting to server...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_http_url_async', { url, selectedTests })
    console.log('[loadFromHttpUrl] sessionIds returned:', sessionIds)
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s) from server`

      // Save the last used directory/URL
      try {
        await invoke('save_last_directory', { directory: url })
        console.log('[App] Saved last directory:', url)
      } catch (error) {
        console.warn('Failed to save last directory:', error)
      }
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
async function loadFromDirectory(directoryPath, selectedTests) {
  console.log('[loadFromDirectory] directoryPath:', directoryPath)
  console.log('[loadFromDirectory] selectedTests:', selectedTests)
  console.log('[loadFromDirectory] selectedTests type:', typeof selectedTests)
  console.log('[loadFromDirectory] selectedTests isArray:', Array.isArray(selectedTests))

  loading.value = true
  loadingMessage.value = 'Scanning directory...'
  selectedEntryIds.value = []

  try {
    const sessionIds = await invoke('parse_log_directory', { directoryPath, selectedTests })
    console.log('[loadFromDirectory] sessionIds returned:', sessionIds)
    loadingMessage.value = `Found ${sessionIds.length} test session(s)`

    await loadSessions()

    if (sessionIds.length > 0) {
      currentSession.value = sessionIds[0]
      await refreshLogs()
      loadingMessage.value = `Loaded ${sessionIds.length} test session(s)`

      // Save the last used directory/URL
      try {
        await invoke('save_last_directory', { directory: directoryPath })
        console.log('[App] Saved last directory:', directoryPath)
      } catch (error) {
        console.warn('Failed to save last directory:', error)
      }
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

// Confirm delete session
async function confirmDeleteSession(session) {
  try {
    await ElMessageBox.confirm(
      `确定要删除测试会话 "${session.name}" 及其所有日志吗？此操作不可撤销。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
        confirmButtonClass: 'el-button--danger'
      }
    )
    await deleteSession(session.id)
  } catch {
    // User cancelled
  }
}

// Delete test session
async function deleteSession(sessionId, event) {
  // Stop event propagation to prevent parent click handlers
  if (event) {
    event.stopPropagation()
  }

  try {
    await invoke('delete_session', { sessionId })
    ElMessage.success('会话已删除')

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
    ElMessage.error(`删除会话时出错：${error}`)
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

    // Ensure auto-bookmarks are created for this session (only on page 1, i.e., when loading a new session)
    if (options.page === 1) {
      try {
        const autoBookmarks = await invoke('ensure_auto_bookmarks', { sessionId: currentSession.value })
        console.log('Auto-bookmarked', autoBookmarks.length, 'entries')
      } catch (error) {
        console.error('Error ensuring auto-bookmarks:', error)
        // Don't alert - this is not critical
      }
    }

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
async function addBookmark(entry, color = '#F59E0B') {
  try {
    await invoke('add_bookmark', {
      logEntryId: entry.id,
      title: truncateText(entry.message),
      color: color
    })
    await loadBookmarks()
  } catch (error) {
    console.error('Error adding bookmark:', error)
    alert(`添加书签时出错：${error}`)
  }
}

// Show bookmark color picker
function showBookmarkColorPicker(entry) {
  bookmarkColorPickerEntryId.value = entry.id
  bookmarkColorPickerVisible.value = true
}

// Handle bookmark color selection
async function handleBookmarkColorSelected(color) {
  const entry = logEntries.value.find(e => e.id === bookmarkColorPickerEntryId.value)
  if (entry) {
    await addBookmark(entry, color)
  }
  bookmarkColorPickerVisible.value = false
  bookmarkColorPickerEntryId.value = null
}

// Close bookmark color picker
function closeBookmarkColorPicker() {
  bookmarkColorPickerVisible.value = false
  bookmarkColorPickerEntryId.value = null
}

// Handle click outside to close bookmark color picker
function handleBookmarkPickerClickOutside(event) {
  // Only handle if the picker is visible
  if (!bookmarkColorPickerVisible.value) return

  // Find the popover element
  const popover = document.querySelector('.el-popover.bookmark-color-picker-popover')
  const reference = event.target.closest('.el-popover__reference')

  // Close if click is outside both popover and reference button
  if (popover && !popover.contains(event.target) && !reference) {
    closeBookmarkColorPicker()
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
    showBookmarkColorPicker(entry)
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
          color: '#E6A23C' // Orange for batch bookmarks
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

// Parse stack trace for display
function parseStack(stackRaw) {
  if (!stackRaw) return []
  return parsePythonStackTrace(stackRaw)
}

// Check if stack is Python format
function isPythonStack(stackRaw) {
  return isPythonStackTrace(stackRaw)
}

// Extract file name from full path
function getFileName(filePath) {
  if (!filePath) return ''
  // Handle both Unix (/) and Windows (\) path separators
  const parts = filePath.split(/[\/\\]/)
  return parts[parts.length - 1] || filePath
}

// Format stack trace for display (convert newlines to spaces)
function formatStack(stack) {
  if (!stack) return '-'
  return stack.replace(/\n/g, ' ').trim()
}

// Check if a message should be collapsed (visual lines > 20)
function needsCollapse(message) {
  const visualLines = calculateVisualLines(message)
  return visualLines > MESSAGE_LINE_LIMIT
}

// Check if a message is currently expanded
function isExpanded(entryId) {
  return expandedMessageIds.value.has(entryId)
}

// Toggle expand/collapse for a message
function toggleExpand(entryId, event) {
  // Stop event propagation immediately
  if (event) {
    event.stopPropagation()
    event.stopImmediatePropagation()
  }

  if (expandedMessageIds.value.has(entryId)) {
    expandedMessageIds.value.delete(entryId)
  } else {
    expandedMessageIds.value.add(entryId)
  }
  // Force reactivity by creating a new Set
  expandedMessageIds.value = new Set(expandedMessageIds.value)
}

// Get display message (truncated if needed)
// Uses the same line calculation logic as calculateVisualLines
function getDisplayMessage(row) {
  const message = row.message
  if (!message) return ''

  const visualLines = calculateVisualLines(message)
  if (visualLines <= MESSAGE_LINE_LIMIT) {
    return message
  }

  // Message needs to be collapsed
  // Calculate approximate max characters for MESSAGE_LINE_LIMIT lines
  // Each line can fit approximately (COLUMN_WIDTH / CHAR_WIDTH) characters
  const charsPerLine = Math.floor(COLUMN_WIDTH / CHAR_WIDTH)
  const maxChars = MESSAGE_LINE_LIMIT * charsPerLine

  // Truncate to max characters
  let truncated = message.substring(0, maxChars)

  // If the truncated message has newlines, try to end at a complete line
  const lastNewline = truncated.lastIndexOf('\n')
  if (lastNewline > maxChars * 0.7) {
    // Last newline is reasonably close to maxChars (within 70%), use it
    truncated = message.substring(0, lastNewline)
  }

  return truncated
}

// Extract time from timestamp (HH:mm:ss format)
function extractTimeFromTimestamp(timestamp) {
  if (!timestamp) return '--:--:--'
  // Match formats like "2026-01-27 16:12:39" or "2026/01/22 07:58:29,723 UTC"
  // Extract the HH:mm:ss part
  const match = timestamp.match(/(\d{2}:\d{2}:\d{2})/)
  return match ? match[1] : '--:--:--'
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

// Get bookmark background color from hex color (adds opacity)
function getBookmarkBackgroundColor(color) {
  if (!color) return 'transparent' // No background for auto-bookmarks

  // Convert hex to rgba with 0.15 opacity
  const hex = color.replace('#', '')
  if (hex.length === 6) {
    const r = parseInt(hex.substring(0, 2), 16)
    const g = parseInt(hex.substring(2, 4), 16)
    const b = parseInt(hex.substring(4, 6), 16)
    return `rgba(${r}, ${g}, ${b}, 0.15)`
  }
  return 'transparent'
}

// Handle pagination changes
function handlePagination() {
  refreshLogs()
}

// Go to specific page
function goToPage(page) {
  if (page >= 1 && page <= totalPages.value) {
    options.page = page
    currentPageForScroll.value = page
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
    currentPageForScroll.value = 1
    refreshLogs()
  }, 300)
}

// Handle level select dropdown visibility change
function handleLevelSelectVisibleChange(visible) {
  if (visible) {
    // When dropdown opens, add close listeners
    nextTick(() => {
      addDropdownCloseListeners()
    })
  } else {
    // When dropdown closes, remove the listeners
    removeDropdownCloseListeners()
  }
}

// Track mouse position for dropdown close detection
let dropdownMouseTracker = {
  isTracking: false,
  dropdownRect: null,
  selectRect: null,
  timer: null,
  checkInterval: null
}

// Add listeners to close dropdown
function addDropdownCloseListeners() {
  // Remove any existing listeners first
  removeDropdownCloseListeners()

  const dropdownEl = findDropdownElement()
  const selectEl = levelSelectRef.value?.$el

  if (!dropdownEl || !selectEl) {
    // If dropdown not found yet, try again after a short delay
    setTimeout(() => {
      const retryDropdown = findDropdownElement()
      if (retryDropdown) {
        addDropdownCloseListeners()
      }
    }, 100)
    return
  }

  // Store the rects for tracking
  dropdownMouseTracker.dropdownRect = dropdownEl.getBoundingClientRect()
  dropdownMouseTracker.selectRect = selectEl.getBoundingClientRect()
  dropdownMouseTracker.isTracking = true

  // Add mousemove listener to track cursor position
  document.addEventListener('mousemove', handleDropdownMouseMove, { passive: true })

  // Add click-outside listener
  document.addEventListener('click', handleLevelSelectClickOutside, { capture: true, passive: true })
}

// Remove listeners
function removeDropdownCloseListeners() {
  dropdownMouseTracker.isTracking = false
  dropdownMouseTracker.dropdownRect = null
  dropdownMouseTracker.selectRect = null

  if (dropdownMouseTracker.timer) {
    clearTimeout(dropdownMouseTracker.timer)
    dropdownMouseTracker.timer = null
  }

  if (dropdownMouseTracker.checkInterval) {
    clearInterval(dropdownMouseTracker.checkInterval)
    dropdownMouseTracker.checkInterval = null
  }

  document.removeEventListener('mousemove', handleDropdownMouseMove)
  document.removeEventListener('click', handleLevelSelectClickOutside, { capture: true })
}

// Find the dropdown element (Element Plus renders it as a separate popper)
function findDropdownElement() {
  const selectEl = levelSelectRef.value?.$el
  if (!selectEl) return null

  // The dropdown has class 'el-select-dropdown' and is usually appended to body
  // Find the one that belongs to this select by checking the 'x-placement' attribute
  // or by finding one that's visible
  const dropdowns = document.querySelectorAll('.el-select-dropdown')
  for (const dropdown of dropdowns) {
    // Check if this dropdown is visible and belongs to our select
    const rect = dropdown.getBoundingClientRect()
    if (rect.width > 0 && rect.height > 0) {
      return dropdown
    }
  }
  return null
}

// Handle mouse move to detect when cursor leaves dropdown area
function handleDropdownMouseMove(event) {
  if (!dropdownMouseTracker.isTracking) return

  const x = event.clientX
  const y = event.clientY

  const dropdownRect = dropdownMouseTracker.dropdownRect
  const selectRect = dropdownMouseTracker.selectRect

  // Check if cursor is outside both dropdown and select
  const outsideDropdown = !dropdownRect || (
    x < dropdownRect.left ||
    x > dropdownRect.right ||
    y < dropdownRect.top ||
    y > dropdownRect.bottom
  )

  const outsideSelect = !selectRect || (
    x < selectRect.left ||
    x > selectRect.right ||
    y < selectRect.top ||
    y > selectRect.bottom
  )

  if (outsideDropdown && outsideSelect) {
    // Mouse is outside, set a timer to close
    if (!dropdownMouseTracker.timer) {
      dropdownMouseTracker.timer = setTimeout(() => {
        closeDropdown()
      }, 150)
    }
  } else {
    // Mouse is back inside, cancel the timer
    if (dropdownMouseTracker.timer) {
      clearTimeout(dropdownMouseTracker.timer)
      dropdownMouseTracker.timer = null
    }
  }
}

// Close the dropdown
function closeDropdown() {
  levelSelectRef.value?.blur()
  removeDropdownCloseListeners()
}

// Handle click outside to close level select dropdown
function handleLevelSelectClickOutside(event) {
  const selectEl = levelSelectRef.value?.$el
  if (!selectEl) return

  const dropdownEl = findDropdownElement()

  // Check if click is outside both the select and the dropdown
  const clickedOutsideSelect = !selectEl.contains(event.target)
  const clickedOutsideDropdown = dropdownEl && !dropdownEl.contains(event.target)

  if (clickedOutsideSelect && clickedOutsideDropdown) {
    // Close the dropdown
    closeDropdown()
  }
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
      currentPageForScroll.value = entryPage
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

// Handle JSON detected in message
function handleJsonDetected(entryId, data) {
  console.log('JSON detected in entry', entryId, ':', data)
  // Can be used for statistics or future features
}

// Highlight and scroll to entry
function highlightAndScroll(entryId) {
  highlightedEntryId.value = entryId

  // Scroll to element using the entry-id class
  // Wait for nextTick to ensure DOM is updated
  nextTick(() => {
    const element = document.querySelector(`.entry-id-${entryId}`)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  })

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
onMounted(async () => {
  // Load last used directory
  try {
    const lastDir = await invoke('get_last_directory')
    if (lastDir && lastDir.length > 0) {
      console.log('[App] Last directory loaded:', lastDir)
      logSourceInput.value = lastDir
    }
  } catch (error) {
    console.warn('Failed to load last directory:', error)
  }

  loadSessions()
  loadSidebarWidth()

  // Listen for HTTP progress events with new async format
  listen('http-progress', (event) => {
    try {
      const progress = JSON.parse(event.payload)

      // ProgressStatus enum serializes with variant name as key
      if (progress.Connecting !== undefined) {
        loadingMessage.value = 'Connecting to server...'
      } else if (progress.Scanning !== undefined) {
        loadingMessage.value = `Scanning... Found ${progress.Scanning.found} log files`
      } else if (progress.Downloading !== undefined) {
        const dl = progress.Downloading
        const totalProgress = dl.total_files > 0
          ? Math.round((dl.completed_files / dl.total_files) * 100)
          : 0
        loadingMessage.value = `Downloading... ${totalProgress}% (${dl.completed_files}/${dl.total_files} files) - ${dl.speed}`
      } else if (progress.Parsing !== undefined) {
        loadingMessage.value = `Parsing ${progress.Parsing.session}...`
      } else if (progress.Complete !== undefined) {
        loadingMessage.value = 'Complete!'
      } else {
        loadingMessage.value = event.payload // Fallback for unknown format
      }
    } catch {
      // Fallback for old string format
      loadingMessage.value = event.payload
    }
  })

  // Add resize event listeners
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
  window.addEventListener('resize', handleWindowResize)

  // Add click outside listener for bookmark color picker
  document.addEventListener('click', handleBookmarkPickerClickOutside)
})

onUnmounted(() => {
  // Remove resize event listeners
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  window.removeEventListener('resize', handleWindowResize)
  // Remove dropdown close listeners
  removeDropdownCloseListeners()
  // Remove click outside listener for bookmark color picker
  document.removeEventListener('click', handleBookmarkPickerClickOutside)
})

// Handle window resize - clamp pinned tooltip to new bounds
function handleWindowResize() {
  if (!pinnedTooltip.value.visible) return

  const maxX = window.innerWidth - pinnedTooltip.value.size.width
  const maxY = window.innerHeight - pinnedTooltip.value.size.height - 64

  if (pinnedTooltip.value.position.x > maxX) {
    pinnedTooltip.value.position.x = Math.max(0, maxX)
  }
  if (pinnedTooltip.value.position.y > maxY) {
    pinnedTooltip.value.position.y = Math.max(64, maxY)
  }
}

// Watch currentSession and fetch log levels when it changes
watch(currentSession, async (newSessionId) => {
  if (newSessionId) {
    await fetchSessionLogLevels()
  } else {
    sessionLogLevels.value = []
  }
})

// Watch levelFilter for changes and sync selectAllLevels state
watch(levelFilter, (newValues, oldValues) => {
  // Avoid infinite loop - only process if actually changed
  if (JSON.stringify(newValues) === JSON.stringify(oldValues)) {
    return
  }

  // Sync select all checkbox state
  if (newValues.length === sessionLogLevels.value.length) {
    selectAllLevels.value = true
  } else {
    selectAllLevels.value = false
  }

  // Refresh logs when filter changes
  refreshLogs()
})

// Persist sidebar width to localStorage
watch(sidebarWidth, (newWidth) => {
  localStorage.setItem('sidebarWidth', newWidth)
})

// Handle dialog opened event - focus input when dialog is fully opened
function handleSourceDialogOpened() {
  nextTick(() => {
    if (logSourceInputRef.value) {
      logSourceInputRef.value.focus()
    }
  })
}

// Element Plus Table helpers
function getLevelType(level) {
  const types = {
    'ERROR': 'danger',
    'WARNING': 'warning',
    'INFO': 'primary',
    'DEBUG': 'info',
    'TRACE': ''
  }
  return types[level] || ''
}

// Computed table height
const tableHeight = computed(() => {
  return showSidebar.value ? 'calc(100vh - 208px)' : 'calc(100vh - 88px)'
})

// Handle page change with smart scroll behavior
async function handlePageChange(page) {
  // 先保存旧页码（使用独立的 ref 来跟踪，因为 Element Plus 会在触发事件前更新 options.page）
  const oldPage = currentPageForScroll.value
  console.log('[handlePageChange] oldPage:', oldPage, 'newPage:', page, 'page type:', typeof page)

  const isNewPageNext = page > oldPage
  const isNewPagePrev = page < oldPage

  console.log('[handlePageChange] Comparison:', page, '>', oldPage, '=', isNewPageNext, page, '<', oldPage, '=', isNewPagePrev)

  // 更新页码
  options.page = page
  // 同步更新跟踪页码
  currentPageForScroll.value = page

  console.log('[handlePageChange] Updated options.page to:', options.page)

  // 先加载新页数据
  await refreshLogs()

  console.log('[handlePageChange] Data loaded, waiting for DOM update...')

  // 等待DOM更新后滚动
  nextTick(() => {
    console.log('[handlePageChange] nextTick executed, options.page is now:', options.page)

    requestAnimationFrame(() => {
      // 尝试多种方式查找滚动容器
      let scrollContainer = document.querySelector('.el-table__body-wrapper')
      if (scrollContainer) {
        const innerWrap = scrollContainer.querySelector('.el-scrollbar__wrap')
        const scrollBar = scrollContainer.querySelector('.el-scrollbar')
        if (innerWrap) {
          scrollContainer = innerWrap
        } else if (scrollBar) {
          scrollContainer = scrollBar
        }
      }

      console.log('[handlePageChange] scrollContainer:', scrollContainer)

      if (!scrollContainer) {
        console.log('[handlePageChange] No scroll container found!')
        return
      }

      const scrollHeight = scrollContainer.scrollHeight

      if (isNewPageNext) {
        // 下一页：始终滚动到顶部
        console.log('[handlePageChange] Next page - scrolling to top, scrollHeight:', scrollHeight)
        scrollContainer.scrollTop = 0
      } else if (isNewPagePrev) {
        // 上一页：始终滚动到底部
        console.log('[handlePageChange] Previous page - scrolling to bottom, scrollHeight:', scrollHeight)
        scrollContainer.scrollTop = scrollHeight
      } else {
        console.log('[handlePageChange] Same page - no scroll needed')
      }
    })
  })
}

// Handle page size change
function handleSizeChange(size) {
  options.itemsPerPage = size
  options.page = 1
  currentPageForScroll.value = 1
  refreshLogs()
}

// Get table row class name
function getTableRowClassName({ row }) {
  const classes = []
  // Add entry-id class for scroll-to functionality
  classes.push(`entry-id-${row.id}`)
  if (selectedEntryIds.value.includes(row.id)) {
    classes.push('table-row-selected')
  }
  if (isHighlighted(row.id)) {
    classes.push('table-row-highlighted')
  }
  return classes.join(' ')
}

// Handle pin request from MessageTooltip
function handleTooltipPin(rowId, data) {
  // Close existing pinned tooltip if any
  if (pinnedTooltip.value.visible) {
    closePinnedTooltip()
  }

  // Set new pinned tooltip
  pinnedTooltip.value = {
    visible: true,
    rowId,
    message: data.message,
    hasJson: data.hasJson,
    viewMode: data.viewMode || 'raw',
    position: data.position,
    size: data.size
  }
}

// Close pinned tooltip
function closePinnedTooltip() {
  pinnedTooltip.value.visible = false
  pinnedTooltip.value.rowId = null
}

// Update pinned tooltip position
function updatePinnedPosition(pos) {
  if (!pinnedTooltip.value.visible) return
  pinnedTooltip.value.position = pos
}

// Update pinned tooltip size
function updatePinnedSize(size) {
  if (!pinnedTooltip.value.visible) return
  pinnedTooltip.value.size = size
}
</script>

<template>
  <div class="app-container">
    <!-- Log Source Dialog -->
    <el-dialog
      v-model="showSourceDialog"
      title="打开日志源"
      width="600px"
      :close-on-click-modal="false"
      @opened="handleSourceDialogOpened">
      <div class="log-source-content">
        <div class="content-description">
          <el-icon :size="20"><Link v-if="inputSourceType === 'url'" /><Folder v-else /></el-icon>
          <span v-if="inputSourceType === 'url'">检测到 HTTP 服务器 URL</span>
          <span v-else>输入文件夹路径或 HTTP URL</span>
        </div>
        <div class="source-input-wrapper">
          <el-input
            ref="logSourceInputRef"
            v-model="logSourceInput"
            :placeholder="inputSourceType === 'url' ? '例如: http://logs.example.com/test-logs/' : '选择或输入本地文件夹路径，或输入 HTTP URL'"
            :prefix-icon="inputSourceType === 'url' ? Link : Folder"
            clearable
            size="large"
            class="log-source-input"
            @keyup.enter="handleSourceDialogEnter" />
          <el-button
            v-if="inputSourceType === 'folder'"
            @click="selectLocalFolder"
            :icon="FolderOpened"
            size="large"
            class="browse-btn">
            浏览
          </el-button>
        </div>
        <div class="input-hint">
          <el-text type="info" size="small">
            <template v-if="logSourceInput.trim()">
              <el-icon><InfoFilled /></el-icon>
              <span v-if="inputSourceType === 'url'">将作为 HTTP 服务器 URL 处理</span>
              <span v-else>将作为本地文件夹路径处理</span>
            </template>
            <template v-else>
              输入 http:// 或 https:// 开头的地址将作为 HTTP URL，否则作为本地文件夹路径
            </template>
          </el-text>
        </div>
      </div>

      <template #footer>
        <el-button @click="showSourceDialog = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="!canOpen"
          @click="openLogSource"
          size="large">
          打开日志源
        </el-button>
      </template>
    </el-dialog>

    <!-- Edit Bookmark Dialog -->
    <el-dialog
      v-model="showEditBookmarkDialog"
      title="编辑书签名称"
      width="400px"
      :close-on-click-modal="false">
      <el-input
        v-model="editingBookmarkTitle"
        placeholder="书签名称"
        autofocus
        @keyup.enter="confirmEditBookmark">
      </el-input>
      <div v-if="editingBookmark" class="bookmark-info">
        <el-icon :size="14"><InfoFilled /></el-icon>
        日志时间: {{ editingBookmark[1]?.timestamp }}
      </div>

      <template #footer>
        <el-button @click="showEditBookmarkDialog = false">取消</el-button>
        <el-button type="primary" @click="confirmEditBookmark">确定</el-button>
      </template>
    </el-dialog>

    <!-- Test Selection Dialog -->
    <TestSelectionDialog
      v-model:visible="showTestSelectionDialog"
      :scan-results="testScanResults"
      :loading="testScanLoading"
      :directory-path="pendingSourcePath"
      @confirm="handleTestSelectionConfirm" />

    <!-- App Header -->
    <el-header class="app-header" :class="{ 'sidebar-collapsed': !showSidebar }">
      <div class="header-content">
        <div class="header-left">
          <el-button
            :icon="showSidebar ? Fold : Expand"
            circle
            @click="showSidebar = !showSidebar"
            :title="showSidebar ? '收起左侧面板' : '展开左侧面板'"
            class="sidebar-toggle" />
          <div class="logo-icon">
            <LogTerminatorIcon />
          </div>
          <span class="app-title">logTerminator</span>

          <!-- Session Selector -->
          <el-select
            v-model="currentSession"
            placeholder="选择会话"
            class="session-select"
            @change="onSessionChange">
            <el-option
              v-for="session in sessions"
              :key="session.id"
              :label="session.name"
              :value="session.id">
              <el-tooltip
                :content="session.directory_path"
                placement="right"
                :show-after="300">
                <div class="session-option-item">
                  <div class="session-info">
                    <el-icon><component :is="session.source_type === 'http' ? 'Link' : 'Folder'" /></el-icon>
                    <span class="session-name">{{ session.name }}</span>
                    <span class="session-count">{{ session.total_entries }} 条记录</span>
                  </div>
                  <el-icon
                    class="delete-icon"
                    :size="16"
                    @click.stop="confirmDeleteSession(session)">
                    <Delete />
                  </el-icon>
                </div>
              </el-tooltip>
            </el-option>
          </el-select>

          <!-- Open Directory Button -->
          <el-button
            type="primary"
            :icon="FolderOpened"
            :loading="loading"
            @click="openDirectory"
            class="open-dir-btn">
            打开目录
          </el-button>

          <!-- Search Input -->
          <el-input
            v-model="searchTerm"
            placeholder="搜索日志内容..."
            :prefix-icon="Search"
            clearable
            style="width: 240px"
            @input="debouncedSearch">
          </el-input>

          <!-- Log Level Filter -->
          <el-select
            ref="levelSelectRef"
            v-model="levelFilter"
            placeholder="日志级别"
            multiple
            class="level-filter-select"
            :popper-options="{
              strategy: 'fixed',
              modifiers: [{ name: 'flip', enabled: false }]
            }">
            <template #header>
              <div style="padding: 8px 12px; border-bottom: 1px solid #ebeef5;">
                <el-checkbox
                  v-model="selectAllLevels"
                  @change="toggleAllLevels">
                  全选
                </el-checkbox>
              </div>
            </template>
            <el-option
              v-for="level in sortedLogLevels"
              :key="level"
              :label="level"
              :value="level" />
          </el-select>

        </div>

        <div class="header-right">
          <span v-if="loadingMessage" class="loading-message">{{ loadingMessage }}</span>
        </div>
      </div>

      <el-progress
        v-if="loading"
        :percentage="100"
        :indeterminate="true"
        :show-text="false"
        class="loading-progress" />
    </el-header>

    <!-- Main Content -->
    <el-main class="main-content">
      <div class="content-wrapper">
        <div class="content-row">
          <!-- Left Sidebar -->
          <transition name="slide-fade">
            <div v-if="showSidebar" class="sidebar-container">
              <div class="sidebar" :style="{ width: sidebarWidth + 'px' }">
                <!-- Bookmarks Panel -->
                <el-card class="bookmarks-panel" shadow="hover">
                  <template #header>
                    <div class="panel-header">
                      <div class="panel-title">
                        <el-icon
                          :class="{ 'rotated': !showBookmarksPanel }"
                          @click="showBookmarksPanel = !showBookmarksPanel">
                          <ArrowRight />
                        </el-icon>
                        <el-icon class="bookmark-icon"><Collection /></el-icon>
                        <span>书签</span>
                        <el-tag size="small" type="warning" class="bookmark-count">
                          {{ bookmarks.length }}
                        </el-tag>
                      </div>
                    </div>
                  </template>

                  <transition name="panel-slide">
                    <div v-show="showBookmarksPanel" class="bookmarks-list">
                      <el-scrollbar :height="'calc(100vh - 280px)'">
                        <div v-if="bookmarks.length > 0" class="bookmark-items">
                          <div
                            v-for="bookmark in bookmarks"
                            :key="bookmark[0]?.id"
                            class="bookmark-item"
                            :style="{ backgroundColor: getBookmarkBackgroundColor(bookmark[0]?.color) }"
                            @click="jumpToBookmark(bookmark)">
                            <el-avatar :size="20" class="bookmark-avatar">
                              <el-icon><StarFilled /></el-icon>
                            </el-avatar>
                            <span class="bookmark-time">{{ extractTimeFromTimestamp(bookmark[1]?.timestamp) }}</span>
                            <span class="bookmark-title">{{ bookmark[0]?.title || '书签' }}</span>
                            <div class="bookmark-actions">
                              <el-button
                                link
                                type="primary"
                                :icon="Edit"
                                size="small"
                                @click.stop="showEditBookmarkTitleDialog(bookmark)"
                                title="编辑书签名称" />
                              <el-button
                                link
                                type="danger"
                                :icon="Close"
                                size="small"
                                @click.stop="removeBookmarkById(bookmark[0]?.id)"
                                title="删除书签" />
                            </div>
                          </div>
                        </div>
                        <el-empty
                          v-else
                          description="暂无书签"
                          :image-size="80">
                          <template #description>
                            <p class="empty-text">点击日志条目旁的星号添加书签</p>
                          </template>
                        </el-empty>
                      </el-scrollbar>
                    </div>
                  </transition>
                </el-card>
              </div>

              <!-- Resizer -->
              <div
                class="resizer"
                :class="{ 'is-resizing': isResizing }"
                @mousedown="startResize">
              </div>
            </div>
          </transition>

          <!-- Main Content Area -->
          <div class="main-area">
            <!-- Log Table -->
            <el-card class="table-card" shadow="hover">
              <el-table
                :data="logEntries"
                :height="tableHeight"
                v-loading="loading"
                stripe
                :row-class-name="getTableRowClassName"
                :row-key="(row) => row.id">
                <el-table-column type="selection" width="50" />
                <el-table-column prop="timestamp" label="时间戳" width="180">
                  <template #default="{ row }">
                    <span @click="toggleRowSelection(row)" style="cursor: pointer; display: block;">{{ row.timestamp }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="level" label="级别" width="90">
                  <template #default="{ row }">
                    <el-tag :type="getLevelType(row.level)" size="small" @click="toggleRowSelection(row)" style="cursor: pointer;">
                      {{ row.level }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="stack" label="调用栈" width="60" align="center">
                  <template #default="{ row }">
                    <div @click="toggleRowSelection(row)" style="cursor: pointer; display: flex; justify-content: center; align-items: center; height: 100%;">
                      <el-tooltip
                        v-if="row.stack"
                        placement="auto"
                        :show-after="2000"
                        :hide-after="500"
                        effect="light"
                        popper-class="stack-tooltip-large"
                        :offset="10"
                        :popper-options="{
                          modifiers: [
                            { name: 'flip', options: { fallbackPlacements: ['top', 'bottom', 'left', 'right'] } },
                            { name: 'preventOverflow', options: { boundary: 'viewport' } }
                          ]
                        }"
                        raw-content>
                        <template #content>
                          <div class="tooltip-content">
                            <div class="tooltip-header">
                              <el-icon><MoreFilled /></el-icon>
                              Stack Trace
                            </div>
                            <!-- Python stack trace with tabs -->
                            <template v-if="isPythonStack(row.stack)">
                              <el-tabs model-value="parsed" class="stack-tabs">
                                <el-tab-pane label="解析表格" name="parsed">
                                  <el-table
                                    :data="parseStack(row.stack)"
                                    size="small"
                                    :header-cell-style="{ background: '#f5f7fa', color: '#606266', fontWeight: '600' }"
                                    class="tooltip-stack-table"
                                    max-height="300">
                                    <el-table-column label="文件" min-width="150">
                                      <template #default="{ row: stackRow }">
                                        <span :title="stackRow.file">{{ getFileName(stackRow.file) }}</span>
                                      </template>
                                    </el-table-column>
                                    <el-table-column prop="line" label="行号" width="60" align="center" />
                                    <el-table-column prop="function" label="函数" min-width="120" show-overflow-tooltip />
                                    <el-table-column prop="code" label="代码" min-width="180" show-overflow-tooltip />
                                  </el-table>
                                </el-tab-pane>
                                <el-tab-pane label="原始栈" name="raw">
                                  <div class="tooltip-text">{{ row.stack }}</div>
                                </el-tab-pane>
                              </el-tabs>
                            </template>
                            <!-- Non-Python stack trace: show raw text only -->
                            <div v-else class="tooltip-text">{{ row.stack }}</div>
                          </div>
                        </template>
                        <el-icon :size="18" class="stack-icon">
                          <WarningFilled />
                        </el-icon>
                      </el-tooltip>
                      <span v-else class="stack-placeholder">-</span>
                    </div>
                  </template>
                </el-table-column>
                <el-table-column
                  prop="message"
                  label="消息"
                  class-name="message-column">
                  <template #default="{ row }">
                    <div class="message-cell-wrapper">
                      <span
                        v-if="needsCollapse(row.message)"
                        class="expand-toggle"
                        @click.stop.prevent="toggleExpand(row.id, $event)"
                        @mousedown.stop.prevent
                        @mouseup.stop.prevent
                        @mouseenter="handleExpandHover"
                        @mouseleave="handleExpandLeave"
                        style="display: inline-flex; align-items: center; justify-content: center; cursor: pointer; user-select: none; -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none;">
                        {{ isExpanded(row.id) ? '-' : '+' }}
                      </span>
                      <MessageTooltip
                        ref="messageTooltipRefs"
                        :message="formatMessage(row.message)"
                        :display-message="isExpanded(row.id) ? formatMessage(row.message) : getDisplayMessage(row)"
                        :is-pinned="pinnedTooltip.visible && pinnedTooltip.rowId === row.id"
                        :initial-position="pinnedTooltip.position"
                        :initial-view-mode="pinnedTooltip.viewMode"
                        :initial-size="pinnedTooltip.size"
                        :useDialogForLargeJson="true"
                        :largeJsonThreshold="2"
                        :disable-hover="hoveringExpandButton"
                        @json-detected="(data) => handleJsonDetected(row.id, data)"
                        @pin="(data) => handleTooltipPin(row.id, data)"
                        @close="closePinnedTooltip"
                        @position-update="updatePinnedPosition"
                        @size-update="updatePinnedSize" />
                    </div>
                  </template>
                </el-table-column>
                <el-table-column label="书签" width="80" align="center">
                  <template #default="{ row }">
                    <el-popover
                      v-if="!isBookmarked(row.id)"
                      :visible="bookmarkColorPickerVisible && bookmarkColorPickerEntryId === row.id"
                      placement="left"
                      width="200"
                      trigger="manual"
                      popper-class="bookmark-color-picker-popover">
                      <template #reference>
                        <el-button
                          :icon="Star"
                          circle
                          size="small"
                          @click.stop="showBookmarkColorPicker(row)" />
                      </template>
                      <BookmarkColorPicker
                        @color-selected="handleBookmarkColorSelected"
                        @close="closeBookmarkColorPicker" />
                    </el-popover>
                    <el-button
                      v-else
                      :icon="StarFilled"
                      type="warning"
                      circle
                      size="small"
                      @click.stop="toggleBookmark(row)" />
                  </template>
                </el-table-column>
              </el-table>

              <!-- Pagination -->
              <div class="pagination-container">
                <el-pagination
                  v-model:current-page="options.page"
                  v-model:page-size="options.itemsPerPage"
                  :page-sizes="[25, 50, 100, 200, 500]"
                  :total="totalEntries"
                  layout="prev, pager, next, jumper, ->, sizes, total"
                  background
                  @current-change="handlePageChange"
                  @size-change="handleSizeChange" />
              </div>
            </el-card>
          </div>
        </div>
      </div>
    </el-main>

  </div>
</template>

<style scoped>
/* App Container */
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
}

/* App Header Styles */
.app-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 0;
  height: 64px;
  display: flex;
  flex-direction: column;
  align-items: center;
  position: relative;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  z-index: 1000;
}

.header-content {
  width: 100%;
  display: flex;
  align-items: center;
  padding: 0 28px;
  height: 64px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 0 1 auto;
  min-width: 0;
}

.sidebar-toggle {
  background: rgba(255, 255, 255, 0.15);
  border: none;
  color: white;
  transition: all 0.3s ease;
  flex-shrink: 0;
}

.sidebar-toggle:hover {
  background: rgba(255, 255, 255, 0.25);
  transform: scale(1.05);
}

.logo-icon {
  width: 32px;
  height: 32px;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.app-title {
  font-size: 20px;
  font-weight: 700;
  color: white;
  letter-spacing: 1px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.session-select {
  width: 280px;
  margin-left: 20px;
  flex-shrink: 0;
}

.open-dir-btn {
  margin-left: 12px;
  flex-shrink: 0;
}

/* Log level filter - dynamic width to align with table */
.level-filter-select {
  flex: 0 0 auto !important;
  width: calc(100vw - 1030px) !important;
  min-width: 200px !important;
  max-width: none !important;
}

/* When sidebar is collapsed, log level filter can be wider */
.sidebar-collapsed .level-filter-select {
  width: calc(100vw - 430px) !important;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
  min-width: fit-content;
}

.loading-message {
  color: white;
  font-size: 14px;
  white-space: nowrap;
}

.loading-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
}

/* Main Content */
.main-content {
  flex: 1;
  padding: 16px;
  overflow: hidden;
}

.content-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.content-row {
  display: flex;
  gap: 0;
  height: 100%;
}

/* Sidebar */
.sidebar-container {
  display: flex;
  flex-shrink: 0;
}

.sidebar {
  flex-shrink: 0;
  padding-right: 16px;
}

.bookmarks-panel {
  height: fit-content;
}

.panel-header {
  padding: 0;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.panel-title .el-icon {
  cursor: pointer;
  transition: transform 0.3s ease;
}

.panel-title .el-icon.rotated {
  transform: rotate(-90deg);
}

.bookmark-icon {
  color: #f59e0b;
}

.bookmark-count {
  margin-left: auto;
}

.bookmarks-list {
  margin-top: 12px;
}

.bookmark-items {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bookmark-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  min-height: 28px;
}

.bookmark-item:hover {
  filter: brightness(0.9);
}

.bookmark-avatar {
  background-color: #fef3c7;
  color: #f59e0b;
  flex-shrink: 0;
}

.bookmark-time {
  font-size: 12px;
  color: #909399;
  white-space: nowrap;
  flex-shrink: 0;
  margin: 0;
  padding: 0;
  line-height: 1;
}

.bookmark-title {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1;
  margin: 0;
  padding: 0;
}

.bookmark-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.bookmark-actions :deep(.el-button) {
  padding: 2px 4px;
  height: 20px;
  margin: 0;
}

.bookmark-actions :deep(.el-button .el-icon) {
  font-size: 12px;
}

.bookmark-item:hover .bookmark-actions {
  opacity: 1;
}

.empty-text {
  color: #909399;
  font-size: 12px;
  margin-top: 8px;
}

/* Main Area */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0 28px;
}

.table-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.table-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0;
}

/* Table Styles */
.el-table {
  flex: 1;
}

/* Remove default cursor pointer from rows */
:deep(.el-table__row) {
  cursor: default;
}

:deep(.el-table__row.table-row-selected) {
  background-color: rgba(255, 193, 7, 0.15) !important;
}

:deep(.el-table__row.table-row-selected td) {
  border-left: 3px solid rgb(255, 193, 7);
}

:deep(.el-table__row.table-row-highlighted) {
  background-color: rgba(33, 150, 243, 0.3) !important;
  animation: pulse-highlight 1.5s ease-in-out 3;
}

:deep(.el-table__row.table-row-highlighted td) {
  border-left: 3px solid rgb(33, 150, 243);
}

@keyframes pulse-highlight {
  0%, 100% { background-color: rgba(33, 150, 243, 0.3); }
  50% { background-color: rgba(33, 150, 243, 0.5); }
}

.stack-cell {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #666;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 240px;
}

/* Stack column icon styles */
.stack-icon {
  cursor: pointer;
  color: var(--el-text-color-secondary);
  transition: color 0.2s ease;
}

.stack-icon:hover {
  color: var(--el-color-warning);
}

.stack-placeholder {
  color: #c0c4cc;
  font-size: 14px;
}

.message-cell {
  font-size: 14px;
  width: 100%;
  overflow: hidden;
  /* Preserve newlines from original logs */
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
}

/* Message column wrapper for proper overflow handling */
.message-cell-wrapper {
  width: 100%;
  overflow: hidden;
  display: block;
}

/* Element Plus table cell structure - ensure proper overflow */
:deep(.el-table__body-wrapper .el-table__body .message-column) {
  overflow: visible;
}

:deep(.el-table__body-wrapper .el-table__body .message-column .cell) {
  width: 100% !important;
  overflow: visible !important;
  /* Preserve newlines from original logs */
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
}

:deep(.el-table__body-wrapper .el-table__body .message-column .cell .message-cell-wrapper) {
  width: 100%;
  display: flex;
  align-items: flex-start;
  gap: 4px;
}

.expand-toggle {
  display: inline-block;
  min-width: 24px;
  padding: 2px 6px;
  margin-right: 4px;
  cursor: pointer;
  color: #409EFF;
  font-weight: bold;
  font-size: 16px;
  line-height: 1.4;
  user-select: none;
  flex-shrink: 0;
  pointer-events: auto !important;
  position: relative;
  z-index: 10;
  border: 1px solid #409EFF;
  border-radius: 4px;
  background-color: #ecf5ff;
}

.expand-toggle:hover {
  color: #409EFF;
  background-color: #d9ecff;
  border-color: #409EFF;
}

:deep(.el-table__body-wrapper .el-table__body .message-column .cell .message-cell-wrapper .message-tooltip-trigger) {
  flex: 1;
  display: block;
  /* Preserve newlines from original logs */
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  pointer-events: auto !important;
}

/* Pagination */
.pagination-container {
  flex-shrink: 0;
  padding: 16px;
  border-top: 1px solid #ebeef5;
}

/* Resizer */
.resizer {
  width: 4px;
  cursor: col-resize;
  background: #e0e0e0;
  transition: background 0.2s, width 0.2s;
  flex-shrink: 0;
}

.resizer:hover,
.resizer.is-resizing {
  background: #1976d2;
  width: 6px;
}

/* Session Option */
.session-option-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 8px 0;
}

.session-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.session-name {
  font-size: 14px;
}

.session-count {
  color: #909399;
  font-size: 12px;
  margin-left: 8px;
}

.delete-icon {
  color: #909399;
  cursor: pointer;
  transition: color 0.2s ease;
  padding: 4px;
}

.delete-icon:hover {
  color: #f56c6c;
}

/* Dialog Styles */
.log-source-content {
  padding: 0 8px;
}

.content-description {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 16px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e8eef5 100%);
  border-radius: 8px;
  border-left: 4px solid #409EFF;
  color: #606266;
  font-size: 14px;
}

.content-description .el-icon {
  color: #409EFF;
  flex-shrink: 0;
}

.content-description span {
  line-height: 1.6;
}

.source-input-wrapper {
  display: flex;
  gap: 12px;
  align-items: stretch;
}

.log-source-input {
  flex: 1;
  min-width: 0;
}

.log-source-input :deep(.el-input__wrapper) {
  width: 100%;
}

.log-source-input :deep(.el-input__inner) {
  width: 100%;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.browse-btn {
  flex-shrink: 0;
  width: 100px;
  font-weight: 500;
}

.input-hint {
  margin-top: 12px;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.input-hint .el-icon {
  font-size: 14px;
}

.bookmark-info {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  color: #909399;
  font-size: 12px;
}

/* Transitions */
.slide-fade-enter-active {
  transition: all 0.3s ease-out;
}

.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(1, 0.5, 0.8, 1);
}

.slide-fade-enter-from {
  transform: translateX(-20px);
  opacity: 0;
}

.slide-fade-leave-to {
  transform: translateX(-20px);
  opacity: 0;
}

.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: all 0.3s ease;
}

.panel-slide-enter-from,
.panel-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Tooltip Styles */
:deep(.stack-tooltip),
:deep(.message-tooltip) {
  max-width: 600px;
}

.tooltip-content {
  padding: 16px;
}

.tooltip-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 12px;
}

.tooltip-text {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  max-height: 340px;
  overflow-y: auto;
}

/* Scrollbar Styles */
.tooltip-text::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.tooltip-text::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
}

.tooltip-text::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.15);
  border-radius: 4px;
  transition: background 0.2s ease;
}

.tooltip-text::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.25);
}

/* Monospace font for timestamps */
:deep(.el-table__body-wrapper .el-table__row td:first-child) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

/* Large Tooltip for Stack Trace */
:deep(.stack-tooltip-large) {
  max-width: 800px !important;
}

/* Stack Tabs in Tooltip */
.stack-tabs {
  margin-top: 8px;
}

.stack-tabs :deep(.el-tabs__header) {
  margin: 0 0 8px 0;
}

.stack-tabs :deep(.el-tabs__item) {
  font-size: 13px;
  padding: 0 16px;
}

.stack-tabs :deep(.el-tabs__content) {
  padding: 0;
}

.tooltip-stack-table {
  font-size: 12px;
}

.tooltip-stack-table :deep(.el-table__header th) {
  padding: 6px 0;
  font-size: 12px;
  background: #f5f7fa !important;
}

.tooltip-stack-table :deep(.el-table__body td) {
  padding: 4px 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 11px;
}

.tooltip-stack-table :deep(.el-table__body tr:hover > td) {
  background-color: #f5f7fa !important;
}

/* Pinned Tooltip Container */
.pinned-tooltip-container {
  position: fixed;
  z-index: 2000;
  pointer-events: auto;
}
</style>

<style>
/* Message Tooltip Popover Styles */
.message-tooltip-popover {
  max-width: 80%;
  z-index: 9999 !important;
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
  background-color: #000000;
  color: #e6edf3;
  padding: 12px 14px;
  border-radius: 4px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
}

/* Pinned Tooltip Popover Styles */
.message-tooltip-popover.is-pinned {
  position: fixed !important;
  transform: none !important;
}
</style>
