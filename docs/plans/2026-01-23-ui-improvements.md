# UI改进与Element Plus迁移实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** 迁移到Element Plus组件库并改进UI，包括Session删除、书签栏高度、移除复选框、分页高亮和按钮可见性。

**Architecture:** 逐步从Vuetify 3迁移到Element Plus，保持所有现有功能不变。使用Element Plus的对应组件替换Vuetify组件，同时添加Session删除功能和改进按钮样式。

**Tech Stack:** Vue 3 Composition API, Element Plus, Tauri, Vite, pnpm

---

## Task 1: 安装Element Plus依赖

**Files:**
- Modify: `package.json` (in worktree)

**Step 1: 安装Element Plus**

Run: `cd E:\work\logTerminator\.worktrees\ui-improvements && pnpm add element-plus @element-plus/icons-vue`

Expected: Package added to dependencies

**Step 2: 验证package.json**

Check `package.json` contains:
```json
"element-plus": "^latest",
"@element-plus/icons-vue": "^latest"
```

**Step 3: Commit**

```bash
git add package.json pnpm-lock.yaml
git commit -m "deps: add element-plus and icons"
```

---

## Task 2: 配置Element Plus

**Files:**
- Modify: `src/main.js`

**Step 1: 引入Element Plus**

Replace entire `src/main.js` with:

```javascript
import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import App from './App.vue'

const app = createApp(App)

// Register all Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(ElementPlus)
app.mount('#app')
```

**Step 2: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 3: Commit**

```bash
git add src/main.js
git commit -m "feat: configure element-plus"
```

---

## Task 3: 替换导航栏为Element Plus布局

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-app-bar为el-header**

Find `<v-app-bar>` (around line 700) and replace with:

```vue
<el-header class="app-header">
  <div class="header-content">
    <div class="header-left">
      <el-icon class="logo-icon" :size="32"><Document /></el-icon>
      <span class="app-title">LogTerminator</span>
    </div>

    <!-- Session Selector -->
    <el-select
      v-model="currentSession"
      :items="sessions"
      placeholder="选择会话"
      style="width: 300px; margin-left: 20px"
      @change="onSessionChange">
      <template #default="{ item }">
        <el-option
          v-for="session in sessions"
          :key="session.id"
          :label="session.name"
          :value="session.id">
          <div class="session-option">
            <el-icon><component :is="session.source_type === 'http' ? 'Link' : 'Folder'" /></el-icon>
            <span>{{ session.name }}</span>
            <span class="session-count">{{ session.total_entries }} 条记录</span>
          </div>
        </el-option>
      </template>
    </el-select>

    <div class="header-right">
      <!-- Open Directory Button -->
      <el-button type="primary" :icon="FolderOpened" @click="showSourceDialog = true">
        打开日志目录
      </el-button>
    </div>
  </div>

  <!-- Loading Progress -->
  <el-progress
    v-if="loading"
    :percentage="100"
    :indeterminate="true"
    :show-text="false"
    class="loading-progress" />
</el-header>
```

**Step 2: 添加CSS样式**

In `<style>` section, add:

```css
.app-header {
  background: white;
  border-bottom: 1px solid #e0e0e0;
  padding: 0;
  height: 60px;
  display: flex;
  align-items: center;
}

.header-content {
  width: 100%;
  display: flex;
  align-items: center;
  padding: 0 20px;
}

.header-left {
  display: flex;
  align-items: center;
}

.logo-icon {
  color: #409EFF;
  margin-right: 10px;
}

.app-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.header-right {
  margin-left: auto;
}

.session-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.session-count {
  color: #909399;
  font-size: 12px;
}

.loading-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
}
```

**Step 3: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace app-bar with el-header"
```

---

## Task 4: 替换布局容器为Element Plus

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-main和v-container**

Find `<v-main>` and `<v-container>` (around line 760), replace with:

```vue
<el-main class="main-content">
  <div class="content-wrapper">
    <div class="content-row">
      <!-- Left Sidebar -->
      <!-- (existing sidebar content) -->

      <!-- Main Content -->
      <!-- (existing main content) -->
    </div>
  </div>
</el-main>
```

**Step 2: 添加容器样式**

```css
.main-content {
  background: #f5f7fa;
  padding: 20px;
}

.content-wrapper {
  max-width: 100%;
  margin: 0 auto;
}

.content-row {
  display: flex;
  gap: 0;
  align-items: flex-start;
}
```

**Step 3: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace v-container with custom layout"
```

---

## Task 5: 替换书签面板为Element Plus Card

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-card为el-card**

Find the Bookmarks Panel section, replace with:

```vue
<el-card class="bookmarks-card" shadow="never">
  <template #header>
    <div class="bookmarks-header">
      <el-button
        :icon="showBookmarksPanel ? 'ArrowDown' : 'ArrowRight'"
        link
        @click="showBookmarksPanel = !showBookmarksPanel">
        {{ showBookmarksPanel ? '收起' : '展开' }}
      </el-button>
      <div class="bookmarks-title">
        <el-icon><Bookmark /></el-icon>
        <span>书签</span>
        <el-tag size="small" type="warning">{{ bookmarks.length }}</el-tag>
      </div>
    </div>
  </template>

  <div v-show="showBookmarksPanel" class="bookmarks-list" style="height: calc(100vh - 280px); overflow-y: auto;">
    <div v-if="bookmarks.length > 0">
      <div
        v-for="bookmark in bookmarks"
        :key="bookmark[0]?.id"
        class="bookmark-item"
        @click="jumpToBookmark(bookmark)">
        <el-icon class="bookmark-icon" color="#F59E0B"><StarFilled /></el-icon>
        <div class="bookmark-info">
          <div class="bookmark-title">{{ bookmark[0]?.title || '书签' }}</div>
          <div class="bookmark-meta">{{ bookmark[1]?.timestamp }}</div>
        </div>
        <div class="bookmark-actions" @click.stop>
          <el-button
            type="primary"
            :icon="Edit"
            size="small"
            link
            @click="showEditBookmarkTitleDialog(bookmark)">
            编辑
          </el-button>
          <el-button
            type="danger"
            :icon="Delete"
            size="small"
            link
            @click="removeBookmarkById(bookmark[0]?.id)">
            删除
          </el-button>
        </div>
      </div>
    </div>
    <el-empty v-else description="暂无书签">
      <el-text type="info">点击日志条目旁的星号添加书签</el-text>
    </el-empty>
  </div>
</el-card>
```

**Step 2: 添加书签面板样式**

```css
.bookmarks-card {
  margin-bottom: 20px;
}

.bookmarks-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bookmarks-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  flex: 1;
}

.bookmarks-list {
  padding: 0;
}

.bookmark-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: background 0.2s;
}

.bookmark-item:hover {
  background: #f5f7fa;
}

.bookmark-icon {
  margin-right: 12px;
}

.bookmark-info {
  flex: 1;
  min-width: 0;
}

.bookmark-title {
  font-weight: 500;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bookmark-meta {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.bookmark-actions {
  display: flex;
  gap: 8px;
}
```

**Step 3: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace bookmarks panel with el-card"
```

---

## Task 6: 添加Session删除功能

**Files:**
- Modify: `src/App.vue`

**Step 1: 在Session选择器中添加删除按钮**

Replace the session selector template with:

```vue
<el-select
  v-model="currentSession"
  placeholder="选择会话"
  style="width: 300px; margin-left: 20px"
  @change="onSessionChange">
  <template #default="{ item }">
    <el-option
      v-for="session in sessions"
      :key="session.id"
      :label="session.name"
      :value="session.id">
      <div class="session-option-item">
        <div class="session-info">
          <el-icon>
            <component :is="session.source_type === 'http' ? 'Link' : 'Folder'" />
          </el-icon>
          <span>{{ session.name }}</span>
          <el-tag size="small">{{ session.total_entries }} 条记录</el-tag>
        </div>
        <el-button
          type="danger"
          :icon="Delete"
          size="small"
          link
          @click.stop="confirmDeleteSession(session)" />
      </div>
    </el-option>
  </template>
</el-select>
```

**Step 2: 添加删除确认函数**

In `<script setup>`, add after `onSessionChange`:

```javascript
// Confirm and delete session
async function confirmDeleteSession(session) {
  try {
    await ElMessageBox.confirm(
      `确定要删除会话 "${session.name}" 吗？这将同时删除所有相关日志和书签。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await deleteSession(session.id)
    ElMessage.success('会话已删除')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败: ' + error)
    }
  }
}

// Delete session
async function deleteSession(sessionId) {
  try {
    await invoke('delete_session', { sessionId })
    await loadSessions()

    // If deleted session was current, clear selection
    if (currentSession.value === sessionId) {
      currentSession.value = null
      logEntries.value = []
    }
  } catch (error) {
    throw error
  }
}
```

**Step 3: 添加必要的导入**

At the top of `<script setup>`, add:

```javascript
import { ElMessageBox, ElMessage } from 'element-plus'
```

**Step 4: 添加Session选项样式**

```css
.session-option-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.session-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}
```

**Step 5: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 6: Commit**

```bash
git add src/App.vue
git commit -m "feat: add session delete functionality"
```

---

## Task 7: 替换Log表格为Element Plus Table

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-data-table为el-table**

Find the log table section and replace with:

```vue
<el-card shadow="never">
  <el-table
    :data="logEntries"
    :height="tableHeight"
    stripe
    v-loading="loading">

    <el-table-column prop="timestamp" label="时间戳" width="180" fixed />

    <el-table-column prop="level" label="级别" width="100">
      <template #default="{ row }">
        <el-tag :type="getLevelType(row.level)" size="small">
          {{ row.level }}
        </el-tag>
      </template>
    </el-table-column>

    <el-table-column prop="message" label="消息" min-width="300">
      <template #default="{ row }">
        <el-tooltip :content="row.message" placement="top" :disabled="row.message.length < 100">
          <div class="log-message">{{ row.message }}</div>
        </el-tooltip>
      </template>
    </el-table-column>

    <el-table-column prop="stack" label="堆栈" min-width="200">
      <template #default="{ row }">
        <el-text
          v-if="row.stack"
          type="info"
          size="small"
          style="font-family: monospace;">
          {{ row.stack }}
        </el-text>
      </template>
    </el-table-column>

    <el-table-column label="操作" width="120" align="center" fixed="right">
      <template #default="{ row }">
        <el-button
          :icon="isBookmarked(row.id) ? StarFilled : Star"
          :type="isBookmarked(row.id) ? 'warning' : 'default'"
          link
          @click="toggleBookmark(row)">
          {{ isBookmarked(row.id) ? '已收藏' : '收藏' }}
        </el-button>
      </template>
    </el-table-column>
  </el-table>

  <!-- Pagination -->
  <div class="pagination-container">
    <el-pagination
      v-model:current-page="options.page"
      :page-size="options.itemsPerPage"
      :page-sizes="[10, 20, 50, 100]"
      :total="totalEntries"
      layout="total, sizes, prev, pager, next, jumper"
      background
      @current-change="handlePageChange"
      @size-change="handleSizeChange" />
  </div>
</el-card>
```

**Step 2: 添加getLevelType函数**

In `<script setup>`, add:

```javascript
// Get log level tag type
function getLevelType(level) {
  const levelMap = {
    'ERROR': 'danger',
    'WARN': 'warning',
    'INFO': 'info',
    'DEBUG': 'info',
    'TRACE': 'info',
  }
  return levelMap[level] || 'info'
}
```

**Step 3: 添加handlePageChange和handleSizeChange**

Add these functions:

```javascript
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
```

**Step 4: 添加表格和分页样式**

```css
.log-message {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pagination-container {
  display: flex;
  justify-content: center;
  padding: 20px 0;
  border-top: 1px solid #e0e0e0;
}
```

**Step 5: 计算tableHeight**

Add computed property:

```javascript
const tableHeight = computed(() => {
  return window.innerHeight - 320
})
```

**Step 6: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 7: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace v-data-table with el-table"
```

---

## Task 8: 更新过滤器区域为Element Plus

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-select为el-select**

Replace the filter card with:

```vue
<el-card shadow="never" class="filters-card">
  <el-row :gutter="20" align="middle">
    <el-col :span="6">
      <el-select
        v-model="options.itemsPerPage"
        placeholder="每页显示"
        style="width: 100%"
        @change="options.page = 1; refreshLogs()">
        <el-option
          v-for="size in itemsPerPageOptions"
          :key="size"
          :label="size"
          :value="size" />
      </el-select>
    </el-col>

    <el-col :span="6">
      <el-select
        v-model="levelFilter"
        placeholder="日志级别"
        style="width: 100%"
        @change="refreshLogs">
        <el-option
          v-for="level in dynamicLogLevels"
          :key="level"
          :label="level"
          :value="level" />
      </el-select>
    </el-col>

    <el-col :span="12">
      <el-input
        v-model="searchTerm"
        placeholder="搜索日志内容..."
        :prefix-icon="Search"
        clearable
        @input="refreshLogsDebounced" />
    </el-col>
  </el-row>
</el-card>
```

**Step 2: 添加过滤器样式**

```css
.filters-card {
  margin-bottom: 20px;
}
```

**Step 3: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace filter controls with element-plus"
```

---

## Task 9: 替换对话框组件为Element Plus

**Files:**
- Modify: `src/App.vue`

**Step 1: 替换v-dialog为el-dialog**

Find and replace all dialogs with el-dialog:

```vue
<!-- Log Source Dialog -->
<el-dialog
  v-model="showSourceDialog"
  title="打开日志源"
  width="500px">
  <el-radio-group v-model="sourceType" style="margin-bottom: 20px;">
    <el-radio value="folder">本地文件夹</el-radio>
    <el-radio value="url">HTTP URL</el-radio>
  </el-radio-group>

  <div v-if="sourceType === 'folder'">
    <el-button type="primary" :icon="FolderOpened" @click="selectLocalFolder" :disabled="!selectedFolderPath">
      选择文件夹
    </el-button>
    <div v-if="selectedFolderPath" class="mt-2">
      <el-text type="info">{{ selectedFolderPath }}</el-text>
    </div>
  </div>

  <div v-else>
    <el-input
      v-model="httpUrl"
      placeholder="https://example.com/logs"
      :prefix-icon="Link" />
  </div>

  <template #footer>
    <el-button @click="showSourceDialog = false">取消</el-button>
    <el-button
      type="primary"
      :disabled="!canOpen"
      @click="openLogSource"
      :loading="loading">
      打开
    </el-button>
  </template>
</el-dialog>

<!-- Edit Bookmark Dialog -->
<el-dialog
  v-model="showEditBookmarkDialog"
  title="编辑书签"
  width="400px">
  <el-input
    v-model="editingBookmarkTitle"
    placeholder="书签标题"
    :prefix-icon="Edit" />

  <template #footer>
    <el-button @click="showEditBookmarkDialog = false">取消</el-button>
    <el-button type="primary" @click="saveBookmarkTitle">
      保存
    </el-button>
  </template>
</el-dialog>
```

**Step 2: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "refactor: replace dialogs with el-dialog"
```

---

## Task 10: 移除Vuetify依赖

**Files:**
- Modify: `package.json`
- Modify: `src/App.vue`

**Step 1: 移除Vuetify相关代码**

Remove from `src/App.vue`:
- All `v-` prefix components (should already be replaced)
- Any remaining Vuetify-specific styles

**Step 2: 卸载Vuetify依赖**

Run: `cd E:\work\logTerminator\.worktrees\ui-improvements && pnpm remove vuetify`

**Step 3: 清理index.html**

Check `index.html` and remove any Vuetify-related CDN links or styles

**Step 4: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 5: Commit**

```bash
git add package.json pnpm-lock.yaml src/App.vue index.html
git commit -m "chore: remove vuetify dependency"
```

---

## Task 11: 清理未使用的CSS和代码

**Files:**
- Modify: `src/App.vue`

**Step 1: 移除Vuetify相关样式**

Remove from `<style>` section:
- Any `.v-` class selectors
- Vuetify override styles
- Unused CSS variables

**Step 2: 优化现有样式**

Clean up and organize remaining CSS

**Step 3: 验证前端编译**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "chore: cleanup unused css and code"
```

---

## Task 12: 最终测试和验证

**Files:**
- None (verification)

**Step 1: 运行完整生产构建**

Run: `cd E:\work\logTerminator\.worktrees\ui-improvements && pnpm tauri build`
Expected: Successful build

**Step 2: 功能测试清单**

手动测试所有功能：
- [ ] Session选择器正常工作
- [ ] Session删除功能正常
- [ ] 删除当前session时正确处理
- [ ] 书签栏高度与log table一致
- [ ] Log table无复选框
- [ ] 星标按钮可见且可用
- [ ] 分页当前页高亮正确
- [ ] 所有文字按钮可见且功能正常
- [ ] 侧边栏调整宽度功能正常
- [ ] 所有过滤器正常工作
- [ ] 搜索功能正常
- [ ] 对话框正常工作

**Step 3: 运行后端测试**

Run: `cd src-tauri && cargo test --test '*'`
Expected: 10 tests passing

**Step 4: 修复发现的问题**

如果发现任何问题，创建修复任务并提交

**Step 5: 最终commit**

如果所有测试通过：
```bash
git add .
git commit -m "chore: final cleanup and verification complete"
```

---

## Summary

This plan implements UI improvements and Element Plus migration in 12 tasks:

1. ✅ Install Element Plus dependencies
2. ✅ Configure Element Plus in main.js
3. ✅ Replace navigation bar
4. ✅ Replace layout containers
5. ✅ Replace bookmarks panel
6. ✅ Add session delete functionality
7. ✅ Replace log table with el-table
8. ✅ Replace filter controls
9. ✅ Replace dialog components
10. ✅ Remove Vuetify dependency
11. ✅ Cleanup unused CSS
12. ✅ Final testing and verification

Each task follows TDD where applicable, has complete code snippets, exact commands, and frequent commits.
