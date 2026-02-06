<script setup>
import { ref, computed } from 'vue'
import { ElCheckbox, ElCheckboxGroup, ElTag } from 'element-plus'

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  scanResults: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  },
  directoryPath: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:visible', 'confirm'])

const selectedTests = ref([])

// Computed properties
const hasSelection = computed(() => selectedTests.value.length > 0)

const testCount = computed(() => props.scanResults.length)

const selectedCount = computed(() => selectedTests.value.length)

// Methods
function handleConfirm() {
  console.log('[TestSelectionDialog] Confirm clicked, selectedTests:', selectedTests.value)
  console.log('[TestSelectionDialog] selectedTests type:', typeof selectedTests.value)
  console.log('[TestSelectionDialog] selectedTests isArray:', Array.isArray(selectedTests.value))

  // Ensure we always emit an array
  const testsToEmit = Array.isArray(selectedTests.value) ? selectedTests.value : [selectedTests.value]
  console.log('[TestSelectionDialog] testsToEmit:', testsToEmit)

  // Emit confirm event without closing dialog - parent will handle closing
  emit('confirm', testsToEmit)
}

function closeDialog() {
  emit('update:visible', false)
}

function handleClosed() {
  // Reset selection when dialog closes
  console.log('[TestSelectionDialog] Dialog closed, resetting selection')
  selectedTests.value = []
}

function toggleAll() {
  if (selectedTests.value.length === props.scanResults.length) {
    selectedTests.value = []
  } else {
    selectedTests.value = props.scanResults.map(r => r.test_name)
  }
}

function isAllSelected() {
  return props.scanResults.length > 0 && selectedTests.value.length === props.scanResults.length
}

function isSomeSelected() {
  return selectedTests.value.length > 0 && selectedTests.value.length < props.scanResults.length
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="val => !val && closeDialog()"
    title="选择要加载的Test会话"
    width="600px"
    :close-on-click-modal="false"
    @closed="handleClosed"
    class="test-selection-dialog">
    <div v-loading="loading" class="dialog-content">
      <!-- Top hint -->
      <div class="top-hint">
        <el-icon class="hint-icon"><InfoFilled /></el-icon>
        <span class="hint-text">找到 {{ testCount }} 个test会话，请选择要加载的项目</span>
      </div>

      <!-- Select all checkbox -->
      <div v-if="scanResults.length > 0" class="select-all-row">
        <el-checkbox
          :model-value="isAllSelected()"
          :indeterminate="isSomeSelected()"
          @change="toggleAll">
          全选
        </el-checkbox>
        <span class="selected-count">{{ selectedCount }} / {{ testCount }} 已选择</span>
      </div>

      <!-- Test list -->
      <el-checkbox-group v-model="selectedTests" class="test-list">
        <div
          v-for="result in scanResults"
          :key="result.test_name"
          :class="['test-item', { 'is-loaded': result.is_loaded }]">
          <el-checkbox :label="result.test_name" :disabled="loading">
            <div class="test-item-content">
              <div class="test-info">
                <span class="test-name" :title="result.test_name">{{ result.test_name }}</span>
                <el-tag size="small" type="info" class="file-count">
                  {{ result.file_count }} 个文件
                </el-tag>
                <el-tag v-if="result.is_loaded" size="small" type="success" class="loaded-tag">
                  已加载
                </el-tag>
              </div>
              <div v-if="result.is_loaded && result.estimated_entries" class="existing-info">
                {{ result.estimated_entries }} 条记录
              </div>
            </div>
          </el-checkbox>
        </div>
      </el-checkbox-group>

      <!-- Empty state -->
      <el-empty
        v-if="!loading && scanResults.length === 0"
        description="未找到任何test会话"
        :image-size="80" />
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-hint">
          <el-icon><WarningFilled /></el-icon>
          <span>已选择的test将覆盖现有数据</span>
        </div>
        <div class="footer-buttons">
          <el-button @click="closeDialog">取消</el-button>
          <el-button
            type="primary"
            :disabled="!hasSelection || loading"
            @click="handleConfirm">
            加载选中的 Test ({{ selectedCount }})
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.test-selection-dialog :deep(.el-dialog__header) {
  padding: 16px 20px;
  border-bottom: 1px solid #ebeef5;
}

.test-selection-dialog :deep(.el-dialog__body) {
  padding: 16px 20px;
}

.test-selection-dialog :deep(.el-dialog__footer) {
  padding: 12px 20px;
  border-top: 1px solid #ebeef5;
}

.dialog-content {
  min-height: 200px;
}

.top-hint {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e8eef5 100%);
  border-radius: 8px;
  border-left: 4px solid #409EFF;
  margin-bottom: 16px;
  color: #606266;
  font-size: 14px;
}

.hint-icon {
  color: #409EFF;
  flex-shrink: 0;
}

.hint-text {
  line-height: 1.6;
}

.select-all-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f5f7fa;
  border-radius: 6px;
  margin-bottom: 12px;
}

.selected-count {
  font-size: 13px;
  color: #909399;
}

.test-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
  padding: 4px;
}

.test-item {
  padding: 12px;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.test-item:hover {
  background-color: #f5f7fa;
  border-color: #c0c4cc;
}

.test-item.is-loaded {
  background-color: #f0f9ff;
  border-color: #b3d8ff;
}

.test-item :deep(.el-checkbox__label) {
  width: 100%;
}

.test-item-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.test-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.test-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-count {
  font-size: 12px;
}

.loaded-tag {
  font-size: 12px;
}

.existing-info {
  font-size: 12px;
  color: #909399;
  margin-left: 24px;
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.footer-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #e6a23c;
}

.footer-buttons {
  display: flex;
  gap: 12px;
}

/* Scrollbar styling for test list */
.test-list::-webkit-scrollbar {
  width: 8px;
}

.test-list::-webkit-scrollbar-track {
  background: #f5f7fa;
  border-radius: 4px;
}

.test-list::-webkit-scrollbar-thumb {
  background: #dcdfe6;
  border-radius: 4px;
  transition: background 0.2s ease;
}

.test-list::-webkit-scrollbar-thumb:hover {
  background: #c0c4cc;
}
</style>
