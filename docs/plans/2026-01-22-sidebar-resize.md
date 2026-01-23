# Sidebar Resize and Session Selector Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move session selection to navbar dropdown and make sidebar resizable for more log viewing space.

**Architecture:**
- Remove Sessions panel from sidebar, move to v-select dropdown in app-bar
- Replace fixed col-based layout with pixel-based dynamic width for sidebar
- Add draggable resizer between sidebar and main content
- Persist sidebar width to localStorage

**Tech Stack:** Vue 3 Composition API, Vuetify 3, localStorage API

---

## Task 1: Add sidebar width state management

**Files:**
- Modify: `src/App.vue`

**Step 1: Add new reactive state variables**

In `<script setup>` section after line 21 (after `jumpToPage`), add:

```javascript
// Sidebar width management
const sidebarWidth = ref(300)  // Default width in pixels
const isResizing = ref(false)  // Track if user is dragging
```

**Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: add sidebar width state variables"
```

---

## Task 2: Remove Sessions Panel from sidebar

**Files:**
- Modify: `src/App.vue`

**Step 1: Remove showSessionsPanel state**

Find and remove this line (around line 18):
```javascript
const showSessionsPanel = ref(true) // 控制测试会话面板展开/折叠
```

**Step 2: Remove toggleSessionsPanel function**

Find and remove this function (around line 35):
```javascript
function toggleSessionsPanel() {
  showSessionsPanel.value = !showSessionsPanel.value
}
```

**Step 3: Remove Sessions Panel template**

Find the Sessions Panel section in template (starts with `<!-- Sessions Panel -->` around line 796) and remove the entire v-card element until before `</v-expand-x-transition>` (about 80 lines).

The section to remove looks like:
```vue
            <!-- Sessions Panel -->
            <v-card elevation="2">
              <v-card-title class="d-flex align-center py-2 px-4 bg-blue-grey-lighten-5">
                ...
            </v-card>
```

Remove everything until:
```vue
          </v-col>
          </v-expand-x-transition>
```

**Step 4: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 5: Commit**

```bash
git add src/App.vue
git commit -m "refactor: remove sessions panel from sidebar"
```

---

## Task 3: Add Session selector to navbar

**Files:**
- Modify: `src/App.vue`

**Step 1: Add v-select to app-bar**

In the `<template>` section, find `<v-app-bar>` (around line 695). Add the v-select component after the title and before the spacer.

Find:
```vue
    <v-app-bar>
      <v-app-bar-title class="d-flex align-center">
        ...
      </v-app-bar-title>

      <v-progress-linear ... />
```

Replace with:
```vue
    <v-app-bar>
      <v-app-bar-title class="d-flex align-center">
        ...
      </v-app-bar-title>

      <!-- Session Selector -->
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
        style="max-width: 300px; margin-left: 20px"
        @update:model-value="onSessionChange">
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

      <v-progress-linear ... />
```

**Step 2: Add onSessionChange handler**

In `<script setup>` section, add this function after `openDirectory` function (around line 135):

```javascript
// Handle session change from selector
function onSessionChange() {
  levelFilter.value = 'ALL'
  refreshLogs()
}
```

**Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 4: Manual test**

Run: `pnpm tauri dev`
Expected:
- Session selector appears in navbar
- Can switch between sessions
- Log entries update correctly

**Step 5: Commit**

```bash
git add src/App.vue
git commit -m "feat: add session selector to navbar"
```

---

## Task 4: Convert sidebar to dynamic pixel width

**Files:**
- Modify: `src/App.vue`

**Step 1: Change sidebar column from cols to style**

Find the sidebar column (around line 722):
```vue
          <v-col v-if="showSidebar" cols="3" class="pr-4">
```

Replace with:
```vue
          <v-col v-if="showSidebar" :style="{ width: sidebarWidth + 'px', flexShrink: 0 }" class="pr-4">
```

**Step 2: Change main content column to fill remaining space**

Find the main content column (around line 870):
```vue
          <v-col :cols="showSidebar ? 9 : 12">
```

Replace with:
```vue
          <v-col :style="{ flex: 1 }">
```

**Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "refactor: convert sidebar to pixel-based width"
```

---

## Task 5: Add resizer component

**Files:**
- Modify: `src/App.vue`

**Step 1: Add resizer div between sidebar and main content**

Find the end of the sidebar column (after `</v-expand-x-transition>` and before `<v-col :style="{ flex: 1 }">`). Add the resizer:

```vue
          </v-col>
          </v-expand-x-transition>

          <!-- Resizer -->
          <div
            class="resizer"
            :class="{ 'is-resizing': isResizing }"
            @mousedown="startResize">
          </div>

          <!-- Main Content -->
          <v-col :style="{ flex: 1 }">
```

**Step 2: Add resizer CSS styles**

In `<style>` section (at end of file), add:

```css
.resizer {
  width: 4px;
  cursor: col-resize;
  background: #e0e0e0;
  transition: background 0.2s;
  flex-shrink: 0;
}
.resizer:hover,
.resizer.is-resizing {
  background: #1976d2;
}
.resizer:hover {
  width: 6px;
}
```

**Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: add resizer component"
```

---

## Task 6: Implement resize logic

**Files:**
- Modify: `src/App.vue`

**Step 1: Add resize functions**

Add these functions after `onSessionChange` (around line 140):

```javascript
// Start dragging the resizer
function startResize(event) {
  isResizing.value = true
  event.preventDefault()
}

// Handle mouse move during resize
function onMouseMove(event) {
  if (!isResizing.value) return

  const newWidth = event.clientX
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
```

**Step 2: Add event listeners in onMounted**

Find `onMounted` (around line 585) and add event listeners:

```javascript
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
```

**Step 3: Add cleanup in onUnmounted**

Add `onUnmounted` import and add cleanup function. First, update the import:

```javascript
import { ref, reactive, onMounted, onUnmounted, computed, watch, nextTick } from 'vue'
```

Then add after `onMounted`:

```javascript
onUnmounted(() => {
  // Remove resize event listeners
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})
```

**Step 4: Add watch to persist width changes**

Add after the `watch(dynamicLogLevels, ...)` (around line 590):

```javascript
// Persist sidebar width to localStorage
watch(sidebarWidth, (newWidth) => {
  localStorage.setItem('sidebarWidth', newWidth)
})
```

**Step 5: Verify frontend compiles**

Run: `pnpm build`
Expected: Successful build

**Step 6: Commit**

```bash
git add src/App.vue
git commit -m "feat: implement resize logic with localStorage persistence"
```

---

## Task 7: Test drag functionality

**Files:**
- None (manual testing)

**Step 1: Manual test - drag resizer**

Run: `pnpm tauri dev`

Tests:
- [ ] Hover over resizer - cursor changes to col-resize
- [ ] Drag resizer left - sidebar shrinks
- [ ] Drag resizer right - sidebar expands
- [ ] Minimum width 200px enforced
- [ ] Maximum width 50% of screen enforced

**Step 2: Manual test - localStorage persistence**

Tests:
- [ ] Adjust sidebar width
- [ ] Close and reopen app
- [ ] Width is restored to last setting

**Step 3: Manual test - session selector**

Tests:
- [ ] Session selector shows in navbar
- [ ] All sessions appear in dropdown
- [ ] Icons show correctly (folder vs web)
- [ ] Switching sessions updates log table
- [ ] Filter resets when switching sessions

**Step 4: Manual test - responsive behavior**

Tests:
- [ ] Toggle sidebar off/on
- [ ] Layout adjusts correctly
- [ ] Bookmarks still functional

**Step 5: Document any issues found**

If any issues found, create fix tasks and commit.

---

## Task 8: Final verification and cleanup

**Files:**
- None (verification)

**Step 1: Run full production build**

Run: `pnpm tauri build`
Expected: Successful build with no errors

**Step 2: Build verification**

Expected output:
- Frontend builds successfully
- Backend compiles successfully
- Installers created at `src-tauri/target/release/bundle/`

**Step 3: Run tests**

Run: `cd src-tauri && cargo test --test '*'`
Expected: 10 tests passing (same as baseline)

**Step 4: Check for unused code**

Search for any remaining references to removed features:
- `showSessionsPanel` - should be gone
- `toggleSessionsPanel` - should be gone
- Sessions Panel template - should be gone

**Step 5: Final commit if needed**

If any cleanup needed:
```bash
git add .
git commit -m "chore: final cleanup for sidebar resize feature"
```

---

## Summary

This plan implements sidebar resize and session selector redesign in 8 tasks:

1. ✅ Add sidebar width state management
2. ✅ Remove Sessions panel from sidebar
3. ✅ Add Session selector to navbar
4. ✅ Convert sidebar to pixel-based width
5. ✅ Add resizer component
6. ✅ Implement resize logic
7. ✅ Manual testing
8. ✅ Final verification

Each task follows TDD where applicable, has complete code snippets, exact commands, and frequent commits.
