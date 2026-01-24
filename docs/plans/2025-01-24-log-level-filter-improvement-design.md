# Log Level Filter Improvement Design

**Date:** 2025-01-24
**Status:** Design Approved

## Overview

Improve the log level filter interaction by removing the "ALL" option, defaulting to selecting all available log levels, and adding a "Select All" checkbox for quick bulk operations.

## Requirements

1. Remove the "ALL" option from the log level filter
2. Default to selecting all available log levels when loading a session
3. Add a "Select All" checkbox at the top of the dropdown for quick select/deselect all

## Design

### 1. Overall Behavior

- **Default State**: When loading a new test session, automatically select all available log levels (INFO, ERROR, WARN, etc.)
- **Select All Checkbox**: Added at the top of the dropdown with two states:
  - Checked = All log levels are selected
  - Unchecked = At least one log level is not selected
- **Independent Options**: Each log level can be toggled independently, with the select all checkbox syncing automatically

When user clicks "Select All" checkbox:
- If currently all selected → Deselect all (clear selection)
- If currently not all selected → Select all log levels

### 2. Component Structure and Data Flow

**Data Changes:**
- `levelFilter` initial value: `[]` (changed from `['ALL']`)
- `sortedLogLevels` computed property: Remove 'ALL' handling, return sorted levels directly
- `selectAllLevels`: New boolean ref to track select all checkbox state

**Core Functions:**
- `toggleAllLevels()` - Handle select all checkbox click
- `updateSelectAllState()` - Update select all checkbox when individual levels change

**Element Plus Select Configuration:**
```vue
<el-select v-model="levelFilter" multiple>
  <template #header>
    <el-checkbox v-model="selectAllLevels" @change="toggleAllLevels">
      全选
    </el-checkbox>
  </template>
  <el-option v-for="level in sortedLogLevels" ... />
</el-select>
```

### 3. Key Interaction Logic

**Scenario A: Loading New Session**
1. User opens log directory and selects a session
2. `watch(currentSession)` triggers `fetchSessionLogLevels()`
3. Backend returns available levels: `['INFO', 'ERROR', 'WARN']`
4. **New**: Automatically set these levels as selected
5. `levelFilter.value = sessionLogLevels.value` (select all)
6. `selectAllLevels.value = true` (sync checkbox)
7. `refreshLogs()` displays all logs

**Scenario B: Click Select All Checkbox**
1. User clicks "Select All" checkbox
2. `toggleAllLevels()` is called:
   - If `selectAllLevels` is `true` → set `levelFilter.value = []` (clear)
   - If `selectAllLevels` is `false` → set `levelFilter.value = sessionLogLevels.value` (select all)
3. `refreshLogs()` triggers automatically via `watch(levelFilter)`

**Scenario C: Toggle Individual Level**
1. User checks/unchecks a specific level (e.g., ERROR)
2. `watch(levelFilter)` triggers
3. **New logic**: Check if all levels are selected
   - If `levelFilter.length === sessionLogLevels.length` → `selectAllLevels = true`
   - Otherwise → `selectAllLevels = false`
4. `refreshLogs()` continues execution

### 4. Backend Changes

**No backend changes required.** The existing `get_session_log_levels` command returns correct data:
- Input: `session_id`
- Output: `Vec<String>` containing all unique log levels

### 5. Edge Cases

**Case 1: Session with no logs**
- `sessionLogLevels.value = []`
- `levelFilter.value = []`
- Dropdown shows empty, select all checkbox in disabled state
- No impact on other features

**Case 2: User clears all selections**
- `levelFilter.value = []`
- `selectAllLevels = false`
- `refreshLogs()` called with empty array
- Backend logic: Empty array means no filtering, returns all logs
- Select all checkbox remains unchecked

**Case 3: Switching sessions with different level sets**
- Old session: `['INFO', 'ERROR', 'WARN']`
- New session: `['INFO', 'DEBUG']`
- Switch auto-resets: `levelFilter.value = ['INFO', 'DEBUG']`
- Invalid levels (ERROR, WARN) automatically cleared

## Implementation

### File Changes

**src/App.vue**

1. **Initialization data (line ~182):**
   ```javascript
   const levelFilter = ref([])  // Changed from ['ALL']
   const selectAllLevels = ref(false)  // New
   ```

2. **sortedLogLevels computed property (line ~251):**
   ```javascript
   const sortedLogLevels = computed(() => {
     if (sessionLogLevels.value.length === 0) {
       return []
     }
     const sortedLevels = [...sessionLogLevels.value].sort((a, b) => {
       const priorityA = levelPriority[a] || 0
       const priorityB = levelPriority[b] || 0
       return priorityB - priorityA
     })
     return sortedLevels  // No longer adds 'ALL'
   })
   ```

3. **fetchSessionLogLevels function (line ~302):**
   ```javascript
   async function fetchSessionLogLevels() {
     if (!currentSession.value) {
       sessionLogLevels.value = []
       levelFilter.value = []
       return
     }
     try {
       sessionLogLevels.value = await invoke('get_session_log_levels', { sessionId: currentSession.value })
       // New: Default to selecting all levels
       levelFilter.value = [...sessionLogLevels.value]
       selectAllLevels.value = true
     } catch (error) {
       console.error('Error fetching session log levels:', error)
       sessionLogLevels.value = []
     }
   }
   ```

4. **New toggleAllLevels function:**
   ```javascript
   function toggleAllLevels() {
     if (selectAllLevels.value) {
       levelFilter.value = [...sessionLogLevels.value]
     } else {
       levelFilter.value = []
     }
   }
   ```

5. **Enhanced watch(levelFilter) (line ~1032):**
   ```javascript
   watch(levelFilter, (newValues, oldValues) => {
     if (JSON.stringify(newValues) === JSON.stringify(oldValues)) {
       return
     }

     // New: Sync select all checkbox state
     if (newValues.length === sessionLogLevels.value.length) {
       selectAllLevels.value = true
     } else {
       selectAllLevels.value = false
     }

     refreshLogs()
   })
   ```

6. **Template update (line ~1362):**
   ```vue
   <el-select
     ref="levelSelectRef"
     v-model="levelFilter"
     placeholder="日志级别"
     multiple
     clearable>
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
   ```

7. **Remove old watch(sortedLogLevels)** - No longer needed without 'ALL' option

## Testing Checklist

- [ ] Loading a new session selects all log levels by default
- [ ] Select all checkbox shows checked when all levels selected
- [ ] Clicking select all deselects all levels
- [ ] Clicking select all again reselects all levels
- [ ] Toggling individual levels updates select all checkbox correctly
- [ ] Switching sessions resets filter to new session's levels
- [ ] Clearing all selections via clear button works correctly
- [ ] Empty session (no logs) handles gracefully
