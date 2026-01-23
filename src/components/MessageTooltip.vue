<template>
  <el-popover
    :width="popoverWidth"
    :placement="placement"
    trigger="click"
    popper-class="message-tooltip-popover">
    <template #reference>
      <span
        class="message-tooltip-trigger"
        :class="{ 'has-json': hasJson }">
        {{ truncatedMessage }}
      </span>
    </template>

    <div class="message-tooltip-content">
      <!-- Header with toggle and copy buttons -->
      <div class="tooltip-header">
        <div class="view-toggles">
          <el-button
            :type="viewMode === 'raw' ? 'primary' : 'default'"
            size="small"
            @click="viewMode = 'raw'">
            Raw
          </el-button>
          <el-button
            :type="viewMode === 'json' ? 'primary' : 'default'"
            size="small"
            :disabled="!hasJson"
            @click="viewMode = 'json'">
            JSON
          </el-button>
        </div>
        <div class="copy-buttons">
          <el-button
            size="small"
            @click="copyRaw">
            Copy Raw
          </el-button>
          <el-button
            size="small"
            :disabled="!hasJson"
            @click="copyJson">
            Copy JSON
          </el-button>
        </div>
      </div>

      <!-- Content area -->
      <div class="tooltip-body">
        <!-- Raw view -->
        <div v-if="viewMode === 'raw'" class="raw-view">
          <pre class="message-text">{{ message }}</pre>
        </div>

        <!-- JSON view -->
        <div v-else class="json-view">
          <div v-if="jsonError" class="json-error">
            Invalid JSON: {{ jsonError }}
          </div>
          <div v-else class="json-content" v-html="highlightedJson"></div>
        </div>
      </div>
    </div>
  </el-popover>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { detectJson, syntaxHighlightJson, prettifyJson, getJsonSize } from '../utils/jsonViewer.js'

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

// State
const viewMode = ref('raw')
const hasJson = ref(false)
const parsedJson = ref(null)
const jsonError = ref(null)
const highlightedJson = ref('')

// Computed
const popoverWidth = computed(() => {
  if (!hasJson.value) {
    return '400px'
  }
  const size = getJsonSize(props.message)
  return size > props.largeJsonThreshold ? '80%' : '600px'
})

const placement = computed(() => {
  return props.useDialogForLargeJson && hasJson.value ? 'bottom' : 'bottom'
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
  if (!hasJson.value) {
    return
  }

  const prettified = prettifyJson(props.message)

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

// Lifecycle
watch(() => props.message, () => {
  detectJsonInMessage()
  if (hasJson.value) {
    generateHighlightedJson()
  }
}, { immediate: true })

watch(viewMode, (newMode) => {
  if (newMode === 'json' && !highlightedJson.value) {
    generateHighlightedJson()
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
  max-height: 400px;
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

.json-content > div {
  margin-left: 20px;
}

.json-content > div:first-child {
  margin-left: 0;
}
</style>
