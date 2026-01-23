# 侧边栏可拖拽调整 + Session选择器重构设计

**Date:** 2026-01-22
**Status:** Design Approved
**Author:** Collaborative Design Session

## 概述

优化界面布局以增加日志表格显示空间：
1. 将Session会话列表从侧边栏移至顶部导航栏下拉选择器
2. 侧边栏只保留书签面板
3. 实现可拖拽调整侧边栏宽度功能
4. 持久化用户设置的宽度

## 动机

当前侧边栏占用25%宽度（cols="3"），包含书签和会话两个面板，导致日志表格显示区域受限。用户希望有更多空间查看日志内容。

## 设计方案

### 界面布局重构

**侧边栏简化：**
- 左侧边栏只保留**书签面板**
- 移除测试会话面板
- 书签面板可拖拽调整宽度
- 侧边栏可完全隐藏（通过拖拽到最小宽度以下）

**Session会话选择器：**
- 位置：顶部导航栏（app-bar）
- 形式：下拉选择器（v-select）
- 显示当前选中的会话名称和图标
- 点击展开所有可用会话列表

**导航栏布局：**
```
[Logo] [会话选择器 v]  [进度条/消息]  [打开目录按钮]
```

### 数据流和状态管理

**新增状态变量：**
```javascript
const sidebarWidth = ref(300)      // 侧边栏宽度（像素）
const isResizing = ref(false)       // 是否正在拖拽
```

**数据持久化：**
```javascript
// 保存
localStorage.setItem('sidebarWidth', width)

// 读取（onMounted时）
const savedWidth = localStorage.getItem('sidebarWidth') || '300'
```

**Session切换流程：**
1. 用户在下拉选择器中选择新会话
2. 更新 `currentSession`
3. 调用 `refreshLogs()` 获取新会话的日志
4. 重置筛选条件（levelFilter = 'ALL'）

**拖拽流程：**
1. 用户按住分隔条（mousedown）
2. `isResizing = true`，监听鼠标移动
3. 计算新宽度，限制在最小200px和最大50%之间
4. 实时更新 `sidebarWidth`
5. 鼠标松开时 `isResizing = false`，保存到 localStorage

### 组件设计

**Session选择器（v-select配置）：**
```vue
<v-select
  v-model="currentSession"
  :items="sessions"
  item-title="name"
  item-value="id"
  prepend-inner-icon="mdi-folder-multiple"
  density="comfortable"
  variant="solo"
  flat
  hide-details
  style="max-width: 300px">
  <template v-slot:selection="{ item }">
    <v-icon :color="currentSession === item.id ? 'primary' : 'grey'" size="small" class="mr-2">
      {{ item.source_type === 'http' ? 'mdi-web' : 'mdi-folder' }}
    </v-icon>
    <span class="text-truncate">{{ item.name }}</span>
  </template>
  <template v-slot:item="{ props, item }">
    <v-list-item v-bind="props">
      <template v-slot:prepend>
        <v-icon size="small">{{ item.raw.source_type === 'http' ? 'mdi-web' : 'mdi-folder' }}</v-icon>
      </template>
      <v-list-item-title>{{ item.raw.name }}</v-list-item-title>
      <v-list-item-subtitle>{{ item.raw.total_entries }} 条记录</v-list-item-subtitle>
    </v-list-item>
  </template>
</v-select>
```

**可拖拽分隔条：**
```vue
<div
  class="resizer"
  :class="{ 'is-resizing': isResizing }"
  @mousedown="startResize">
</div>
```

**样式（CSS）：**
```css
.resizer {
  width: 4px;
  cursor: col-resize;
  background: #e0e0e0;
  transition: background 0.2s;
}
.resizer:hover, .resizer.is-resizing {
  background: #1976d2;
}
```

### 实现细节

**文件修改：**
- `src/App.vue` - 唯一需要修改的文件

**需要移除的代码：**
- `showSessionsPanel` 状态变量
- `toggleSessionsPanel` 函数
- 整个 Sessions Panel 模板部分（约80行代码）

**需要添加的代码：**
- `sidebarWidth` 状态变量
- `isResizing` 状态变量
- `startResize(event)` 函数
- `onMouseMove(event)` 函数
- `onMouseUp()` 函数
- `loadSidebarWidth()` 函数

**侧边栏布局变化：**
```vue
<!-- 从固定 cols 改为动态宽度 -->
<v-col
  v-if="showSidebar"
  :style="{ width: sidebarWidth + 'px', flexShrink: 0 }"
  class="pr-4">
  <!-- 只保留书签面板 -->
</v-col>

<!-- 分隔条 -->
<div class="resizer" @mousedown="startResize"></div>

<!-- 主内容区自动填充 -->
<v-col :style="{ flex: 1 }">
```

**Session选择器位置：**
```vue
<v-app-bar>
  <v-app-bar-title>LogTerminator</v-app-bar-title>

  <!-- Session选择器 -->
  <v-select v-model="currentSession" ... />

  <v-spacer></v-spacer>

  <!-- 加载状态和按钮 -->
</v-app-bar>
```

**事件监听（生命周期）：**
```javascript
onMounted(() => {
  loadSessions()
  loadSidebarWidth()  // 新增
  listen('http-progress', ...)

  // 拖拽事件
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
})

onUnmounted(() => {
  // 清理拖拽事件
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})
```

**宽度保存监听：**
```javascript
watch(sidebarWidth, (newWidth) => {
  localStorage.setItem('sidebarWidth', newWidth)
})
```

### 拖拽逻辑实现

```javascript
function startResize(event) {
  isResizing.value = true
  event.preventDefault()
}

function onMouseMove(event) {
  if (!isResizing.value) return

  const newWidth = event.clientX
  const minWidth = 200
  const maxWidth = window.innerWidth / 2

  sidebarWidth.value = Math.max(minWidth, Math.min(maxWidth, newWidth))
}

function onMouseUp() {
  isResizing.value = false
}

function loadSidebarWidth() {
  const saved = localStorage.getItem('sidebarWidth')
  if (saved) {
    sidebarWidth.value = parseInt(saved)
  }
}
```

### 测试清单

- [ ] 拖拽分隔条可以调整侧边栏宽度
- [ ] 最小宽度限制生效（200px）
- [ ] 最大宽度限制生效（50%屏幕）
- [ ] 宽度保存到 localStorage
- [ ] 刷新后恢复上次设置的宽度
- [ ] Session选择器正确显示所有会话
- [ ] Session切换后日志正确刷新
- [ ] Session图标正确显示（folder/web）
- [ ] 侧边栏隐藏后，日志表格占满全宽
- [ ] 书签功能不受影响

### 兼容性

- 浏览器：localStorage 需要现代浏览器支持
- 移动端：拖拽操作对触摸屏需要额外处理（future work）

## 实现计划

1. 移除 Sessions Panel 相关代码
2. 添加 Session 选择器到导航栏
3. 实现侧边栏宽度状态管理
4. 实现拖拽逻辑和事件监听
5. 添加 CSS 样式
6. 测试各项功能

## 预期效果

- 用户体验：更大面积显示日志内容
- 灵活性：用户可自定义侧边栏宽度
- 简洁性：界面更简洁，Session选择更直观
