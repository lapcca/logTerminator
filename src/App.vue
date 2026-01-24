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
  InfoFilled
} from '@element-plus/icons-vue'
import MessageTooltip from './components/MessageTooltip.vue'
import { parsePythonStackTrace, getStackPreview, isPythonStackTrace } from './utils/stackParser.js'

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

// Sidebar width management
const sidebarWidth = ref(300)  // Default width in pixels
const isResizing = ref(false)  // Track if user is dragging
const resizeStartX = ref(0)    // Mouse X position when drag starts
const resizeStartWidth = ref(0) // Sidebar width when drag starts

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

// Data table options
const options = reactive({
  page: 1,
  itemsPerPage: 100,
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
  showSourceDialog.value = true
  sourceType.value = 'folder'
  selectedFolderPath.value = ''
  httpUrl.value = ''
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
onMounted(() => {
  loadSessions()
  loadSidebarWidth()

  // Listen for HTTP progress events
  listen('http-progress', (event) => {
    loadingMessage.value = event.payload
  })

  // Add resize event listeners
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
})

onUnmounted(() => {
  // Remove resize event listeners
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  // Remove dropdown close listeners
  removeDropdownCloseListeners()
})

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

// Handle page change
function handlePageChange(page) {
  options.page = page
  refreshLogs()
}

// Handle page size change
function handleSizeChange(size) {
  options.itemsPerPage = size
  options.page = 1
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
</script>

<template>
  <div class="app-container">
    <!-- Log Source Dialog -->
    <el-dialog
      v-model="showSourceDialog"
      title="打开日志源"
      width="600px"
      :close-on-click-modal="false">
      <el-tabs v-model="sourceType" class="source-tabs">
        <el-tab-pane label="本地文件夹" name="folder">
          <div class="tab-content">
            <div class="content-description">
              <el-icon :size="20"><Folder /></el-icon>
              <span>选择本地计算机上的日志文件夹路径</span>
            </div>
            <div class="folder-input-wrapper">
              <el-input
                v-model="selectedFolderPath"
                placeholder="选择或输入本地文件夹路径"
                :prefix-icon="Folder"
                clearable
                size="large"
                class="folder-input" />
              <el-button
                @click="selectLocalFolder"
                :icon="FolderOpened"
                size="large"
                class="browse-btn">
                浏览
              </el-button>
            </div>
          </div>
        </el-tab-pane>

        <el-tab-pane label="HTTP 服务器" name="url">
          <div class="tab-content">
            <div class="content-description">
              <el-icon :size="20"><Link /></el-icon>
              <span>输入 HTTP 服务器上的日志目录 URL 地址</span>
            </div>
            <el-input
              v-model="httpUrl"
              placeholder="例如: http://logs.example.com/test-logs/"
              :prefix-icon="Link"
              clearable
              size="large"
              class="url-input" />
          </div>
        </el-tab-pane>
      </el-tabs>

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

    <!-- App Header -->
    <el-header class="app-header">
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
        </div>

        <!-- Log Level Filter - separate element to align with table -->
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
                            @click="jumpToBookmark(bookmark)">
                            <el-avatar :size="32" class="bookmark-avatar">
                              <el-icon><StarFilled /></el-icon>
                            </el-avatar>
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
                highlight-current-row
                @row-click="toggleRowSelection"
                :row-class-name="getTableRowClassName"
                :row-key="(row) => row.id">
                <el-table-column type="selection" width="50" />
                <el-table-column prop="timestamp" label="时间戳" width="180" />
                <el-table-column prop="level" label="级别" width="90">
                  <template #default="{ row }">
                    <el-tag :type="getLevelType(row.level)" size="small">
                      {{ row.level }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="stack" label="调用栈" width="280">
                  <template #default="{ row }">
                    <el-tooltip
                      v-if="row.stack"
                      placement="top"
                      :show-after="200"
                      :hide-after="500"
                      effect="light"
                      popper-class="stack-tooltip-large"
                      :offset="10"
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
                      <span class="stack-cell">{{ formatStack(row.stack) }}</span>
                    </el-tooltip>
                    <span v-else class="stack-cell">-</span>
                  </template>
                </el-table-column>
                <el-table-column
                  prop="message"
                  label="消息"
                  min-width="300">
                  <template #default="{ row }">
                    <MessageTooltip
                      :message="row.message"
                      :useDialogForLargeJson="true"
                      :largeJsonThreshold="2"
                      @json-detected="(data) => handleJsonDetected(row.id, data)" />
                  </template>
                </el-table-column>
                <el-table-column label="书签" width="80" align="center">
                  <template #default="{ row }">
                    <el-button
                      :icon="isBookmarked(row.id) ? StarFilled : Star"
                      :type="isBookmarked(row.id) ? 'warning' : 'default'"
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
  justify-content: space-between;
  padding: 0 28px;
  height: 64px;
  position: relative;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.sidebar-toggle {
  background: rgba(255, 255, 255, 0.15);
  border: none;
  color: white;
  transition: all 0.3s ease;
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
}

.app-title {
  font-size: 20px;
  font-weight: 700;
  color: white;
  letter-spacing: 1px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.session-select {
  width: 280px;
  margin-left: 20px;
}

.open-dir-btn {
  margin-left: 12px;
}

.level-filter-select {
  margin-left: 12px;
  flex: 1;
  min-width: 200px;
  box-sizing: border-box;
  /* Add right margin to match table-card content area (28px padding + 1px border = 29px) */
  margin-right: 1px;
}

/* When sidebar is hidden, adjust log level position */
.sidebar-collapsed .level-filter-select {
  /* No special handling needed with flex layout */
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
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
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.bookmark-item:hover {
  background-color: rgba(255, 193, 7, 0.15);
}

.bookmark-avatar {
  background-color: #fef3c7;
  color: #f59e0b;
  flex-shrink: 0;
}

.bookmark-title {
  flex: 1;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bookmark-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s ease;
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

:deep(.el-table__row) {
  cursor: pointer;
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

.message-cell {
  font-size: 14px;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
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
.source-tabs {
  margin-top: -12px;
}

.source-tabs :deep(.el-tabs__header) {
  margin-bottom: 24px;
}

.source-tabs :deep(.el-tabs__nav-wrap::after) {
  display: none;
}

.source-tabs :deep(.el-tabs__item) {
  font-size: 15px;
  padding: 0 24px;
}

.source-tabs :deep(.el-tabs__active-bar) {
  height: 3px;
}

.tab-content {
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

.folder-input-wrapper {
  display: flex;
  gap: 12px;
  align-items: stretch;
}

.folder-input {
  flex: 1;
  min-width: 0;
}

.folder-input :deep(.el-input__wrapper) {
  width: 100%;
}

.folder-input :deep(.el-input__inner) {
  width: 100%;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.browse-btn {
  flex-shrink: 0;
  width: 100px;
  font-weight: 500;
}

.url-input {
  width: 100%;
}

.url-input :deep(.el-input__wrapper) {
  width: 100%;
}

.url-input :deep(.el-input__inner) {
  width: 100%;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
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
</style>

<style>
/* Message Tooltip Popover Styles */
.message-tooltip-popover {
  max-width: 80%;
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
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
}
</style>
