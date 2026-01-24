<template>
  <el-popover
    ref="popoverRef"
    :width="popoverWidth"
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
      <div
        class="tooltip-header"
        @mousedown="handleDragStart">
        <div class="drag-handle">
          <el-icon><Rank /></el-icon>
        </div>
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
            placeholder="Search keys/values..."
            size="small"
            clearable
            @input="handleSearch"
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
    </div>
  </el-popover>
</template>

<script setup>
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Search, ArrowUp, ArrowDown, Rank } from '@element-plus/icons-vue'
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
  }
})

const emit = defineEmits(['json-detected'])

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

// Disable popper auto-positioning completely
const popperOptions = {
  modifiers: [
    {
      name: 'preventOverflow',
      enabled: false,
    },
    {
      name: 'flip',
      enabled: false,
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
  const maxLen = 60
  if (props.message.length > maxLen) {
    return props.message.substring(0, maxLen) + '...'
  }
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

  dragState.currentX = e.clientX - dragState.startX
  dragState.currentY = e.clientY - dragState.startY
  dragState.hasDragged = true

  // Apply position directly using fixed positioning
  const popper = dragState.popperElement

  // Disable all transitions and animations
  popper.style.transition = 'none'
  popper.style.animation = 'none'

  // Set fixed position with !important to override popper.js
  popper.style.setProperty('position', 'fixed', 'important')
  popper.style.setProperty('left', `${dragState.currentX}px`, 'important')
  popper.style.setProperty('top', `${dragState.currentY}px`, 'important')
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

onUnmounted(() => {
  document.removeEventListener('mousemove', handleDragMove)
  document.removeEventListener('mouseup', handleDragEnd)
  unlockPosition()
  if (dragState.positionLockObserver) {
    dragState.positionLockObserver.disconnect()
  }
})
</script>

<style scoped>
.message-tooltip-trigger {
  cursor: pointer;
  display: inline-block;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
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
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
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
  color: #409EFF;
  font-weight: bold;
  font-size: 12px;
}

:deep(.json-toggle:hover) {
  color: #66b1ff;
}

:deep(.json-item) {
  margin-left: 40px; /* 4-space indentation */
  border-left: 1px solid #444;
  padding-left: 8px;
}

:deep(.json-item[data-collapsed="true"]) {
  display: none;
}

/* Search highlighting */
:deep(mark.json-search-highlight) {
  background-color: rgba(255, 200, 0, 0.4);
  color: #000;
  border-radius: 2px;
  padding: 1px 2px;
  font-weight: bold;
  transition: background-color 0.2s;
}

:deep(mark.json-search-current) {
  background-color: rgba(255, 100, 0, 0.6) !important;
  outline: 2px solid #ff6600;
  outline-offset: -2px;
  border-radius: 2px;
}
</style>
