<script setup>
import { ref } from 'vue'

const emit = defineEmits(['color-selected', 'close'])

// Preset colors with hex values
const presetColors = [
  { name: 'Yellow', hex: '#F59E0B' },
  { name: 'Orange', hex: '#E6A23C' },
  { name: 'Green', hex: '#67C23A' },
  { name: 'Cyan', hex: '#17B2E2' },
  { name: 'Purple', hex: '#9C27B0' },
  { name: 'Pink', hex: '#F48FB1' }
]

const customColor = ref('#F59E0B')
const showColorPicker = ref(false)

function selectColor(hex) {
  emit('color-selected', hex)
  emit('close')
}

function selectCustomColor() {
  emit('color-selected', customColor.value)
  emit('close')
}
</script>

<template>
  <div class="bookmark-color-picker">
    <div class="picker-header">
      <span class="picker-title">选择书签颜色</span>
    </div>

    <!-- Preset colors -->
    <div class="preset-colors">
      <button
        v-for="color in presetColors"
        :key="color.hex"
        class="color-button"
        :style="{ backgroundColor: color.hex }"
        :title="color.name"
        @click="selectColor(color.hex)"
      >
        <span class="color-name">{{ color.name }}</span>
      </button>
    </div>

    <!-- Custom color picker -->
    <div class="custom-color-section">
      <el-button
        text
        size="small"
        @click="showColorPicker = !showColorPicker"
      >
        更多颜色...
      </el-button>

      <div v-if="showColorPicker" class="custom-color-picker">
        <el-color-picker
          v-model="customColor"
          show-alpha
          size="small"
        />
        <el-button
          type="primary"
          size="small"
          @click="selectCustomColor"
        >
          确定
        </el-button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bookmark-color-picker {
  padding: 12px;
  min-width: 200px;
}

.picker-header {
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
}

.picker-title {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.preset-colors {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.color-button {
  width: 36px;
  height: 36px;
  border-radius: 6px;
  border: 2px solid #dcdfe6;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.color-button:hover {
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  border-color: #409EFF;
}

.color-button:active {
  transform: scale(0.95);
}

.color-name {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  font-size: 10px;
  padding: 2px 0;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.color-button:hover .color-name {
  opacity: 1;
}

.custom-color-section {
  border-top: 1px solid #ebeef5;
  padding-top: 8px;
}

.custom-color-picker {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

:deep(.el-color-picker) {
  width: 100%;
}

:deep(.el-color-picker__trigger) {
  width: 100%;
  height: 32px;
}
</style>
