<template>
  <el-popover
    ref="popoverRef"
    :width="popoverWidth"
    placement="auto"
    trigger="hover"
    :show-after="200"
    :hide-after="100"
    popper-class="message-tooltip-popover"
    :popper-options="popperOptions">
    <template #reference>
      <span
        ref="triggerRef"
        class="message-tooltip-trigger"
        :class="{ 'has-json': hasJson }"
        @mouseenter="handleMouseEnter">
        {{ truncatedMessage }}
      </span>
    </template>

    <div class="message-tooltip-content">
      <!-- Header with drag handle, toggle and copy buttons -->
      <div class="tooltip-header" @mousedown="handleDragStart">
        <div class="drag-handle">
          <el-icon><Rank /></el-icon>
        </div>

        <!-- Pin button (shown when not pinned) -->
        <el-button
          v-if="!isPinned"
          :icon="Paperclip"
          size="small"
          class="pin-btn"
          @click.stop="handlePin"
          title="固定悬浮框">
        </el-button>

        <!-- Close button (shown when pinned) -->
        <el-button
          v-if="isPinned"
          :icon="Close"
          size="small"
          class="close-btn"
          @click.stop="handleClose"
          title="关闭">
        </el-button>

        <div class="view-toggles">
          <el-button
            :type="viewMode === 'raw' ? 'primary' : 'default'"
            size="small"
            @click.stop="viewMode = 'raw'">
            Raw
          </el-button>
          <el-button
            :type="viewMode === 'json' ? 'primary' : 'default'"
            size="small"
            :disabled="!hasJson"
            @click.stop="viewMode = 'json'">
            JSON
          </el-button>
        </div>
        <div class="copy-buttons">
          <el-button
            size="small"
            @click.stop="copyRaw">
            Copy Raw
          </el-button>
          <el-button
            size="small"
            :disabled="!hasJson"
            @click.stop="copyJson">
            Copy JSON
          </el-button>
        </div>
      </div>

      <!-- Search bar (for JSON view) -->
      <div v-if="viewMode === 'json' && hasJson" class="search-bar">
        <div class="search-input-wrapper">
          <el-input
            v-model="searchTerm"
            placeholder="Search keys/values... (Enter for next match)"
            size="small"
            clearable
            @input="handleSearch"
            @keyup.enter="handleSearchEnter"
            @focus="handleInputFocus">
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        <div v-if="searchResults.length > 0" class="search-nav">
          <span class="search-results-info">
            Found {{ searchResults.length }} match(es)
          </span>
          <span v-if="searchResults.length > 0" class="search-counter">
            ({{ currentMatchIndex + 1 }}/{{ searchResults.length }})
          </span>
          <div class="nav-buttons">
            <el-button
              size="small"
              :disabled="searchResults.length === 0"
              @click="navigateMatch(-1)">
              <el-icon><ArrowUp /></el-icon>
            </el-button>
            <el-button
              size="small"
              :disabled="searchResults.length === 0"
              @click="navigateMatch(1)">
              <el-icon><ArrowDown /></el-icon>
            </el-button>
          </div>
        </div>
      </div>

      <!-- Content area -->
      <div class="tooltip-body" :style="{ maxHeight: popoverHeight }">
        <!-- Raw view -->
        <div v-if="viewMode === 'raw'" class="raw-view">
          <pre class="message-text">{{ message }}</pre>
        </div>

        <!-- JSON view -->
        <div v-else class="json-view" ref="jsonViewRef">
          <div v-if="jsonError" class="json-error">
            Invalid JSON: {{ jsonError }}
          </div>
          <div v-else class="json-content">
            <div v-html="displayJson" ref="jsonContentRef"></div>
          </div>
        </div>
      </div>

      <!-- Resize handles (only for pinned tooltips) -->
      <div v-if="isPinned" class="resize-handles">
        <!-- Edges -->
        <div class="resize-handle n" @mousedown="startResize($event, 'n')"></div>
        <div class="resize-handle s" @mousedown="startResize($event, 's')"></div>
        <div class="resize-handle e" @mousedown="startResize($event, 'e')"></div>
        <div class="resize-handle w" @mousedown="startResize($event, 'w')"></div>

        <!-- Corners -->
        <div class="resize-handle ne" @mousedown="startResize($event, 'ne')"></div>
        <div class="resize-handle nw" @mousedown="startResize($event, 'nw')"></div>
        <div class="resize-handle se" @mousedown="startResize($event, 'se')"></div>
        <div class="resize-handle sw" @mousedown="startResize($event, 'sw')"></div>
      </div>
    </div>
  </el-popover>
</template>

<script setup>
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, ArrowUp, ArrowDown, Rank, Paperclip, Close } from '@element-plus/icons-vue'
import { detectJson, syntaxHighlightJson, prettifyJson, getJsonSize, searchInJson } from '../utils/jsonViewer.js'

const props = defineProps({
  message: {
    type: String,
    default: ''
  },
  useDialogForLargeJson: {
    type: Boolean,
    default: true
  },
  largeJsonThreshold: {
    type: Number,
    default: 2 // KB
  },
  isPinned: {
    type: Boolean,
    default: false
  },
  initialPosition: {
    type: Object,
    default: () => ({ x: 0, y: 0 })
  },
  initialSize: {
    type: Object,
    default: () => ({ width: 600, height: 400 })
  }
})

const emit = defineEmits(['json-detected', 'pin', 'close', 'position-update', 'size-update'])

// Refs
const popoverRef = ref(null)
const triggerRef = ref(null)
const jsonViewRef = ref(null)
const jsonContentRef = ref(null)

// State
const viewMode = ref('raw')
const hasJson = ref(false)
const parsedJson = ref(null)
const jsonError = ref(null)
const highlightedJson = ref('')  // Original syntax-highlighted JSON (without search highlights)
const displayJson = ref('')      // Currently displayed JSON (may include search highlights)
const searchTerm = ref('')
const searchResults = ref([])
const currentMatchIndex = ref(0)

// Pinned state
const currentSize = ref({ width: 600, height: 400 })
const currentPosition = ref({ x: 0, y: 0 })

// Drag state - use plain variables for better performance
let dragState = {
  isDragging: false,
  hasDragged: false,
  startX: 0,
  startY: 0,
  currentX: 0,
  currentY: 0,
  popperElement: null,
  positionLockObserver: null,
  rafId: null
}

// Resize state
const MIN_SIZE = { width: 300, height: 200 }
const HEADER_HEIGHT = 64
let resizeState = {
  isResizing: false,
  direction: null,
  startX: 0,
  startY: 0,
  startWidth: 0,
  startHeight: 0,
  startLeft: 0,
  startTop: 0
}

// Allow popper auto-positioning to avoid overflow
const popperOptions = {
  modifiers: [
    {
      name: 'preventOverflow',
      enabled: true,
      options: {
        boundary: 'viewport',
        padding: 8,
      },
    },
    {
      name: 'flip',
      enabled: true,
      options: {
        fallbackPlacements: ['top', 'bottom', 'left', 'right'],
        boundary: 'viewport',
        padding: 8,
      },
    },
    {
      name: 'computeStyles',
      options: {
        gpuAcceleration: false, // Disable to prevent transform-based positioning
      },
    },
  ],
}

// Computed
const popoverWidth = computed(() => {
  if (!hasJson.value) {
    return '400px'
  }
  const size = getJsonSize(props.message)
  return size > props.largeJsonThreshold ? '80%' : '600px'
})

const popoverHeight = computed(() => {
  if (viewMode.value === 'raw') {
    const lines = props.message.split('\n').length
    const estimatedHeight = lines * 22 + 100 // 22px per line + padding
    if (estimatedHeight < 400) return 'auto'
    if (estimatedHeight < 800) return `${estimatedHeight}px`
    return '800px'
  } else if (hasJson.value && parsedJson.value) {
    // Estimate JSON height
    const height = estimateJsonHeight(parsedJson.value)
    if (height < 400) return 'auto'
    if (height < 800) return `${height}px`
    return '800px'
  }
  return '400px'
})

const truncatedMessage = computed(() => {
  if (!props.message) {
    return '-'
  }
  // Return full message - let CSS handle truncation with text-overflow: ellipsis
  return props.message
})

// Methods
function estimateJsonHeight(obj, depth = 0) {
  if (obj === null || typeof obj !== 'object') {
    return 22
  }

  const keys = Object.keys(obj)
  const isArr = Array.isArray(obj)
  const childCount = isArr ? obj.length : keys.length

  if (childCount === 0) {
    return 22
  }

  let totalHeight = 22
  const itemsPerLine = depth === 0 ? 1 : 2

  if (isArr) {
    obj.forEach((item, idx) => {
      const childHeight = estimateJsonHeight(item, depth + 1)
      if (idx % itemsPerLine === 0) {
        totalHeight += childHeight
      }
    })
  } else {
    keys.forEach((key, idx) => {
      const childHeight = estimateJsonHeight(obj[key], depth + 1)
      if (idx % itemsPerLine === 0) {
        totalHeight += childHeight
      }
    })
  }

  return totalHeight + depth * 40 // 40px indentation per level
}

function detectJsonInMessage() {
  const result = detectJson(props.message)
  hasJson.value = result.success
  parsedJson.value = result.parsed
  jsonError.value = result.error

  if (hasJson.value) {
    emit('json-detected', { parsed: result.parsed })
  }
}

function generateHighlightedJson() {
  if (hasJson.value && parsedJson.value !== null) {
    highlightedJson.value = syntaxHighlightJson(parsedJson.value)
    // Initialize displayJson with the original highlighted version
    displayJson.value = highlightedJson.value
  }
}

async function copyRaw() {
  try {
    await navigator.clipboard.writeText(props.message)
    ElMessage.success('Raw message copied to clipboard')
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = props.message
    textArea.style.position = 'fixed'
    textArea.style.opacity = '0'
    document.body.appendChild(textArea)
    textArea.select()
    try {
      document.execCommand('copy')
      ElMessage.success('Raw message copied to clipboard')
    } catch (e) {
      ElMessage.error('Copy failed: ' + e.message)
    }
    document.body.removeChild(textArea)
  }
}

async function copyJson() {
  if (!hasJson.value || !parsedJson.value) {
    return
  }

  // Stringify the parsed JSON with 2-space indentation
  const prettified = JSON.stringify(parsedJson.value, null, 2)

  try {
    await navigator.clipboard.writeText(prettified)
    ElMessage.success('JSON copied to clipboard')
  } catch (err) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = prettified
    textArea.style.position = 'fixed'
    textArea.style.opacity = '0'
    document.body.appendChild(textArea)
    textArea.select()
    try {
      document.execCommand('copy')
      ElMessage.success('JSON copied to clipboard')
    } catch (e) {
      ElMessage.error('Copy failed: ' + e.message)
    }
    document.body.removeChild(textArea)
  }
}

function handleSearch() {
  // Always restore original first
  displayJson.value = highlightedJson.value
  searchResults.value = []
  currentMatchIndex.value = 0

  if (!searchTerm.value || !parsedJson.value) {
    return
  }

  searchResults.value = searchInJson(parsedJson.value, searchTerm.value)
  currentMatchIndex.value = 0

  if (searchResults.value.length > 0) {
    nextTick(() => {
      applyHighlight()
      scrollToMatch(0)
    })
  }
}

function handleSearchEnter() {
  // Navigate to next match on Enter
  if (searchResults.value.length > 0) {
    navigateMatch(1)
  }
}

function applyHighlight() {
  if (!searchTerm.value || !highlightedJson.value) return

  // Create a new HTML string with search highlights from the original
  const lowerSearchTerm = searchTerm.value.toLowerCase()
  const regex = new RegExp(`(${escapeRegex(searchTerm.value)})`, 'gi')

  // Parse the original HTML and add highlights
  const tempDiv = document.createElement('div')
  tempDiv.innerHTML = highlightedJson.value

  // Walk through all text nodes and highlight matches
  const walker = document.createTreeWalker(
    tempDiv,
    NodeFilter.SHOW_TEXT,
    null,
    false
  )

  const textNodes = []
  let node
  while ((node = walker.nextNode())) {
    textNodes.push(node)
  }

  // Apply highlights
  textNodes.forEach(textNode => {
    if (textNode.textContent.toLowerCase().includes(lowerSearchTerm)) {
      const text = textNode.textContent
      const highlightedHtml = text.replace(regex, '<mark class="json-search-highlight">$1</mark>')

      const newFragment = document.createRange()
      newFragment.setStartBefore(textNode)
      newFragment.setEndAfter(textNode)
      newFragment.deleteContents()

      const span = document.createElement('span')
      span.innerHTML = highlightedHtml
      newFragment.insertNode(span)
    }
  })

  displayJson.value = tempDiv.innerHTML
}

// Helper to escape regex special characters
function escapeRegex(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function removeHighlight() {
  // Simply restore the original highlighted JSON
  displayJson.value = highlightedJson.value
}

function handlePin() {
  const popper = getPopperElement()
  if (!popper) return

  const rect = popper.getBoundingClientRect()
  emit('pin', {
    message: props.message,
    hasJson: hasJson.value,
    position: { x: rect.left, y: rect.top },
    size: { width: rect.width, height: rect.height }
  })
}

function handleClose() {
  emit('close')
}

function navigateMatch(direction) {
  if (searchResults.value.length === 0) return

  // First, re-apply highlights to ensure we have the current DOM state
  applyHighlight()

  currentMatchIndex.value += direction

  // Wrap around
  if (currentMatchIndex.value < 0) {
    currentMatchIndex.value = searchResults.value.length - 1
  } else if (currentMatchIndex.value >= searchResults.value.length) {
    currentMatchIndex.value = 0
  }

  nextTick(() => {
    scrollToMatch(currentMatchIndex.value)
  })
}

function scrollToMatch(index) {
  if (!jsonViewRef.value || index < 0 || index >= searchResults.value.length) return

  // Find all highlighted marks
  nextTick(() => {
    const highlights = jsonViewRef.value.querySelectorAll('.json-search-highlight')
    if (highlights[index]) {
      highlights[index].scrollIntoView({ behavior: 'smooth', block: 'center' })

      // Remove current class from all
      jsonViewRef.value.querySelectorAll('.json-search-current').forEach(el => {
        el.classList.remove('json-search-current')
      })

      // Add current class to this highlight
      highlights[index].classList.add('json-search-current')
    }
  })
}

function handleMouseEnter() {
  // Reset drag state when hovering trigger (new tooltip instance)
  resetDragState()
}

function resetDragState() {
  dragState.hasDragged = false
  dragState.popperElement = null
  dragState.currentX = 0
  dragState.currentY = 0

  // Stop position lock observer
  if (dragState.positionLockObserver) {
    dragState.positionLockObserver.disconnect()
    dragState.positionLockObserver = null
  }

  // Stop RAF
  if (dragState.rafId) {
    cancelAnimationFrame(dragState.rafId)
    dragState.rafId = null
  }
}

// Resize handlers
function startResize(e, direction) {
  if (!props.isPinned) return

  e.preventDefault()
  e.stopPropagation()

  const popper = getPopperElement()
  if (!popper) return

  const rect = popper.getBoundingClientRect()

  resizeState.isResizing = true
  resizeState.direction = direction
  resizeState.startX = e.clientX
  resizeState.startY = e.clientY
  resizeState.startWidth = rect.width
  resizeState.startHeight = rect.height
  resizeState.startLeft = rect.left
  resizeState.startTop = rect.top

  document.addEventListener('mousemove', handleResizeMove, { passive: false })
  document.addEventListener('mouseup', handleResizeEnd, { passive: false })
}

function handleResizeMove(e) {
  if (!resizeState.isResizing || !props.isPinned) return

  const deltaX = e.clientX - resizeState.startX
  const deltaY = e.clientY - resizeState.startY

  let newWidth = resizeState.startWidth
  let newHeight = resizeState.startHeight
  let newX = currentPosition.value.x
  let newY = currentPosition.value.y

  // Calculate max bounds based on current position before applying changes
  const maxWidth = window.innerWidth - currentPosition.value.x
  const maxHeight = window.innerHeight - currentPosition.value.y - HEADER_HEIGHT

  // Calculate new size based on direction
  if (resizeState.direction.includes('e')) {
    newWidth = resizeState.startWidth + deltaX
  }
  if (resizeState.direction.includes('w')) {
    newWidth = resizeState.startWidth - deltaX
    // Use current position as starting point, not startLeft
    newX = currentPosition.value.x + deltaX
  }
  if (resizeState.direction.includes('s')) {
    newHeight = resizeState.startHeight + deltaY
  }
  if (resizeState.direction.includes('n')) {
    newHeight = resizeState.startHeight - deltaY
    // Use current position as starting point, not startTop
    newY = currentPosition.value.y + deltaY
  }

  // Clamp to min/max
  newWidth = Math.max(MIN_SIZE.width, Math.min(newWidth, maxWidth))
  newHeight = Math.max(MIN_SIZE.height, Math.min(newHeight, maxHeight))

  // Clamp position
  newX = Math.max(0, Math.min(newX, window.innerWidth - newWidth))
  newY = Math.max(HEADER_HEIGHT, Math.min(newY, window.innerHeight - newHeight))

  currentSize.value = { width: newWidth, height: newHeight }
  currentPosition.value = { x: newX, y: newY }

  emit('size-update', currentSize.value)
  emit('position-update', currentPosition.value)

  applyPinnedPosition()
}

function handleResizeEnd() {
  resizeState.isResizing = false
  document.removeEventListener('mousemove', handleResizeMove)
  document.removeEventListener('mouseup', handleResizeEnd)
}

function applyPinnedPosition() {
  if (!props.isPinned) return

  const popper = getPopperElement()
  if (!popper) return

  popper.style.setProperty('position', 'fixed', 'important')
  popper.style.setProperty('left', `${currentPosition.value.x}px`, 'important')
  popper.style.setProperty('top', `${currentPosition.value.y}px`, 'important')
  popper.style.setProperty('width', `${currentSize.value.width}px`, 'important')
  popper.style.setProperty('height', `${currentSize.value.height}px`, 'important')
  popper.style.setProperty('right', 'auto', 'important')
  popper.style.setProperty('bottom', 'auto', 'important')
  popper.style.setProperty('transform', 'none', 'important')
  popper.style.setProperty('margin', '0', 'important')
}

function getPopperElement() {
  if (dragState.popperElement) {
    // Check if still valid
    if (document.body.contains(dragState.popperElement)) {
      return dragState.popperElement
    }
  }

  // Find by class name - get the visible one
  const poppers = document.querySelectorAll('.message-tooltip-popover')
  for (const popper of poppers) {
    const rect = popper.getBoundingClientRect()
    // Check if visible (width > 0 and height > 0 and on screen)
    if (rect.width > 0 && rect.height > 0 && rect.top < window.innerHeight) {
      dragState.popperElement = popper
      return popper
    }
  }
  return null
}

// Lock position using RAF and MutationObserver
function lockPosition() {
  if (!dragState.hasDragged || !dragState.popperElement) return

  // Cancel any existing RAF
  if (dragState.rafId) {
    cancelAnimationFrame(dragState.rafId)
  }

  const applyLockedPosition = () => {
    if (!dragState.hasDragged || !dragState.popperElement) return

    const popper = dragState.popperElement

    // Force fixed position
    popper.style.setProperty('position', 'fixed', 'important')
    popper.style.setProperty('left', `${dragState.currentX}px`, 'important')
    popper.style.setProperty('top', `${dragState.currentY}px`, 'important')
    popper.style.setProperty('right', 'auto', 'important')
    popper.style.setProperty('bottom', 'auto', 'important')
    popper.style.setProperty('transform', 'none', 'important')
    popper.style.setProperty('margin', '0', 'important')

    // Continue locking
    dragState.rafId = requestAnimationFrame(applyLockedPosition)
  }

  applyLockedPosition()
}

function unlockPosition() {
  if (dragState.rafId) {
    cancelAnimationFrame(dragState.rafId)
    dragState.rafId = null
  }
}

function handleInputFocus() {
  // Prevent Element Plus from repositioning by locking immediately
  const popper = getPopperElement()
  if (popper && dragState.hasDragged) {
    // Lock the position immediately before any Element Plus updates
    popper.style.setProperty('position', 'fixed', 'important')
    popper.style.setProperty('left', `${dragState.currentX}px`, 'important')
    popper.style.setProperty('top', `${dragState.currentY}px`, 'important')
    popper.style.setProperty('right', 'auto', 'important')
    popper.style.setProperty('bottom', 'auto', 'important')
    popper.style.setProperty('transform', 'none', 'important')
    popper.style.setProperty('margin', '0', 'important')
  }
  // Start the RAF lock to maintain position
  nextTick(() => {
    lockPosition()
  })
}

// Drag handlers
function handleDragStart(e) {
  // Only allow dragging from header
  if (!e.target.closest('.tooltip-header')) return

  const popper = getPopperElement()
  if (!popper) {
    console.warn('Could not find popper element')
    return
  }

  dragState.isDragging = true
  dragState.popperElement = popper

  const rect = popper.getBoundingClientRect()
  dragState.startX = e.clientX - rect.left
  dragState.startY = e.clientY - rect.top
  dragState.currentX = rect.left
  dragState.currentY = rect.top

  document.addEventListener('mousemove', handleDragMove, { passive: false })
  document.addEventListener('mouseup', handleDragEnd)

  e.preventDefault()
}

function handleDragMove(e) {
  if (!dragState.isDragging || !dragState.popperElement) return

  let newX = e.clientX - dragState.startX
  let newY = e.clientY - dragState.startY
  dragState.hasDragged = true

  // Clamp to window bounds
  const popper = dragState.popperElement
  const rect = popper.getBoundingClientRect()
  const maxX = window.innerWidth - rect.width
  const maxY = window.innerHeight - rect.height

  newX = Math.max(0, Math.min(newX, maxX))
  newY = Math.max(64, Math.min(newY, maxY)) // 64 is header height

  dragState.currentX = newX
  dragState.currentY = newY

  // Update position state for pinned tooltips
  if (props.isPinned) {
    currentPosition.value = { x: newX, y: newY }
    emit('position-update', currentPosition.value)
  }

  // Disable all transitions and animations
  popper.style.transition = 'none'
  popper.style.animation = 'none'

  // Set fixed position with !important to override popper.js
  popper.style.setProperty('position', 'fixed', 'important')
  popper.style.setProperty('left', `${newX}px`, 'important')
  popper.style.setProperty('top', `${newY}px`, 'important')
  popper.style.setProperty('right', 'auto', 'important')
  popper.style.setProperty('bottom', 'auto', 'important')
  popper.style.setProperty('transform', 'none', 'important')
  popper.style.setProperty('margin', '0', 'important')
}

function handleDragEnd() {
  dragState.isDragging = false

  document.removeEventListener('mousemove', handleDragMove)
  document.removeEventListener('mouseup', handleDragEnd)

  // Start position lock after drag
  if (dragState.hasDragged) {
    lockPosition()
  }
}

// Lifecycle
watch(() => props.message, () => {
  detectJsonInMessage()
  if (hasJson.value) {
    generateHighlightedJson()
  }
  // Reset drag state when message changes
  resetDragState()
}, { immediate: true })

watch(viewMode, (newMode) => {
  if (newMode === 'json' && !highlightedJson.value) {
    generateHighlightedJson()
  }
  // Ensure displayJson is synced when switching to JSON view
  if (newMode === 'json' && highlightedJson.value && displayJson.value !== highlightedJson.value) {
    displayJson.value = highlightedJson.value
  }
  // Lock position after mode change if dragged
  nextTick(() => {
    if (dragState.hasDragged) {
      lockPosition()
    }
  })
})

// Watch for position/size changes in pinned mode
watch([currentPosition, currentSize], () => {
  if (props.isPinned) {
    nextTick(() => {
      applyPinnedPosition()
    })
  }
}, { deep: true })

// Initialize position/size for pinned tooltips
watch(() => props.isPinned, (isPinned) => {
  if (isPinned) {
    currentPosition.value = { ...props.initialPosition }
    currentSize.value = { ...props.initialSize }
  }
}, { immediate: true })

onUnmounted(() => {
  document.removeEventListener('mousemove', handleDragMove)
  document.removeEventListener('mouseup', handleDragEnd)
  document.removeEventListener('mousemove', handleResizeMove)
  document.removeEventListener('mouseup', handleResizeEnd)
  resizeState.isResizing = false
  unlockPosition()
  if (dragState.positionLockObserver) {
    dragState.positionLockObserver.disconnect()
  }
})
</script>

<style scoped>
.message-tooltip-trigger {
  cursor: pointer;
  display: block;
  width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.message-tooltip-trigger.has-json {
  color: #409EFF;
  font-weight: 500;
}

.tooltip-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #dcdfe6;
  margin-bottom: 8px;
  cursor: move;
  user-select: none;
  gap: 8px;
}

.drag-handle {
  color: #909399;
  cursor: grab;
  display: flex;
  align-items: center;
  padding: 4px;
  flex-shrink: 0;
}

.drag-handle:hover {
  color: #409EFF;
}

.drag-handle:active {
  cursor: grabbing;
}

.view-toggles {
  display: flex;
  gap: 8px;
}

.copy-buttons {
  display: flex;
  gap: 8px;
}

.tooltip-body {
  overflow-y: auto;
  padding: 12px;
}

.raw-view .message-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.json-view {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.json-error {
  color: #f56c6c;
  padding: 12px;
}

.json-content {
  background-color: #282c34;
  color: #abb2bf;
  padding: 16px;
  border-radius: 6px;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.2);
}

.search-bar {
  padding: 8px 12px;
  border-bottom: 1px solid #dcdfe6;
}

.search-input-wrapper {
  margin-bottom: 8px;
}

.search-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.search-results-info {
  background-color: #e6f7ff;
  color: #409EFF;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 4px;
}

.search-counter {
  font-size: 12px;
  color: #606266;
  padding: 4px 8px;
}

.nav-buttons {
  display: flex;
  gap: 4px;
  margin-left: auto;
}

/* Pin and Close buttons */
.pin-btn {
  color: #909399;
  transition: all 0.2s;
}

.pin-btn:hover {
  color: #409EFF;
  background: rgba(64, 158, 255, 0.1);
}

.close-btn {
  color: #909399;
}

.close-btn:hover {
  color: #f56c6c;
  background: rgba(245, 108, 108, 0.1);
}

/* JSON styles - use :deep() to apply to v-html content */
:deep(.json-array),
:deep(.json-object) {
  position: relative;
}

:deep(.json-toggle) {
  position: absolute;
  left: -40px;
  cursor: pointer;
  user-select: none;
  color: #61afef;
  font-weight: bold;
  font-size: 12px;
}

:deep(.json-toggle:hover) {
  color: #98c379;
}

:deep(.json-item) {
  margin-left: 40px; /* 4-space indentation */
  border-left: 1px solid #4b5263;
  padding-left: 8px;
}

:deep(.json-item[data-collapsed="true"]) {
  display: none;
}

/* Search highlighting */
:deep(mark.json-search-highlight) {
  background-color: rgba(234, 166, 56, 0.5);
  color: #fff;
  border-radius: 3px;
  padding: 2px 4px;
  font-weight: bold;
  transition: background-color 0.2s;
  box-shadow: 0 0 0 1px rgba(234, 166, 56, 0.3);
}

:deep(mark.json-search-current) {
  background-color: rgba(234, 166, 56, 0.8) !important;
  outline: 2px solid #e6a555;
  outline-offset: 1px;
  border-radius: 3px;
  box-shadow: 0 0 8px rgba(234, 166, 56, 0.5);
}

/* Resize Handles */
.resize-handles {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 10;
}

.resize-handle {
  position: absolute;
  background: transparent;
  pointer-events: auto;
  transition: background 0.2s;
}

.resize-handle:hover {
  background: rgba(64, 158, 255, 0.2);
}

/* Edges */
.resize-handle.n,
.resize-handle.s {
  left: 4px;
  right: 4px;
  height: 4px;
  cursor: ns-resize;
}

.resize-handle.n {
  top: 0;
}

.resize-handle.s {
  bottom: 0;
}

.resize-handle.e,
.resize-handle.w {
  top: 4px;
  bottom: 4px;
  width: 4px;
  cursor: ew-resize;
}

.resize-handle.e {
  right: 0;
}

.resize-handle.w {
  left: 0;
}

/* Corners */
.resize-handle.ne,
.resize-handle.nw,
.resize-handle.se,
.resize-handle.sw {
  width: 12px;
  height: 12px;
}

.resize-handle.ne {
  top: 0;
  right: 0;
  cursor: nesw-resize;
}

.resize-handle.nw {
  top: 0;
  left: 0;
  cursor: nwse-resize;
}

.resize-handle.se {
  bottom: 0;
  right: 0;
  cursor: nwse-resize;
}

.resize-handle.sw {
  bottom: 0;
  left: 0;
  cursor: nesw-resize;
}
</style>
