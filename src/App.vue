<script setup>
import { ref, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Reactive data
const currentSession = ref('')
const logEntries = ref([])
const bookmarks = ref([])
const loading = ref(false)
const searchTerm = ref('')
const levelFilter = ref(null)
const totalEntries = ref(0)
const sessions = ref([])

// Data table options
const options = reactive({
  page: 1,
  itemsPerPage: 100,
  sortBy: ['timestamp'],
  sortDesc: [false]
})

// Log levels for filtering
const logLevels = ['ERROR', 'WARNING', 'INFO', 'DEBUG', 'TRACE']

// Table headers
const headers = [
  { text: '时间戳', value: 'timestamp', width: '180px' },
  { text: '级别', value: 'level', width: '80px' },
  { text: '调用栈', value: 'stack', width: '250px' },
  { text: '消息', value: 'message' },
  { text: '操作', value: 'actions', width: '80px', sortable: false }
]

// Open log directory
async function openDirectory() {
  try {
    console.log('Attempting to open directory dialog...')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择日志目录'
    })
    console.log('Directory selected:', selected)

    if (selected) {
      loading.value = true
      console.log('Opening directory:', selected)

      try {
        currentSession.value = await invoke('parse_log_directory', { directoryPath: selected })
        console.log('Session created:', currentSession.value)

        await loadSessions()
        await refreshLogs()

        console.log('Directory processed successfully')
      } catch (error) {
        console.error('Error processing directory:', error)
        alert(`处理目录时出错：${error}\n\n请检查目录路径是否正确，并确保有读取权限。`)
      } finally {
        loading.value = false
      }
    }
  } catch (error) {
    console.error('Error opening directory:', error)
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

// Refresh log entries
async function refreshLogs() {
  if (!currentSession.value) return

  loading.value = true
  try {
    console.log('Fetching logs for session:', currentSession.value)
    const result = await invoke('get_log_entries', {
      sessionId: currentSession.value,
      offset: (options.page - 1) * options.itemsPerPage,
      limit: options.itemsPerPage,
      levelFilter: levelFilter.value,
      searchTerm: searchTerm.value
    })

    console.log('Received result:', result)
    // Rust returns (Vec<LogEntry>, total_count)
    const [entries, total] = result
    logEntries.value = entries || []
    totalEntries.value = total || 0

    await loadBookmarks()
  } catch (error) {
    console.error('Error fetching logs:', error)
    alert(`获取日志时出错：${error}\n\n请检查会话是否有效，或重试刷新操作。`)
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

// Add bookmark
async function addBookmark(entry) {
  try {
    await invoke('add_bookmark', {
      logEntryId: entry.id,
      title: `标记 ${entry.timestamp}`,
      color: 'yellow'
    })
    await loadBookmarks()
  } catch (error) {
    console.error('Error adding bookmark:', error)
    alert(`添加书签时出错：${error}\n\n请稍后重试。`)
  }
}

// Jump to bookmark
function jumpToBookmark(bookmark) {
  console.log('Jump to bookmark:', bookmark)
  // TODO: Implement jump functionality
}

// Get level color for chips
function getLevelColor(level) {
  const colors = {
    'ERROR': 'red',
    'WARNING': 'orange',
    'INFO': 'blue',
    'DEBUG': 'grey',
    'TRACE': 'purple'
  }
  return colors[level] || 'grey'
}

// Handle pagination changes
function handlePagination() {
  refreshLogs()
}

// Debounced search
let searchTimeout
function debouncedSearch() {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    options.page = 1 // Reset to first page
    refreshLogs()
  }, 300)
}

// Load sessions on mount
onMounted(() => {
  loadSessions()
})
</script>

<template>
  <v-app>
    <v-app-bar app color="primary" dark>
      <v-toolbar-title>
        <v-icon class="mr-2">mdi-file-document-outline</v-icon>
        LogTerminator - 日志查看器
      </v-toolbar-title>
      <v-spacer></v-spacer>
      <v-btn @click="openDirectory" :loading="loading" color="accent" prepend-icon="mdi-folder-open">
        打开日志目录
      </v-btn>
    </v-app-bar>

    <v-main>
      <v-container fluid>
        <v-row>
          <!-- Bookmarks panel -->
          <v-col cols="3">
            <v-card class="mb-4" elevation="2">
              <v-card-title class="bg-blue-grey-lighten-5">
                <v-icon class="mr-2">mdi-bookmark-multiple</v-icon>
                书签/标记
                <v-chip size="small" color="primary" variant="flat" class="ml-auto">
                  {{ bookmarks.length }}
                </v-chip>
              </v-card-title>
              <v-card-text class="pa-0">
                <v-list dense>
                  <v-list-item
                    v-for="bookmark in bookmarks"
                    :key="bookmark[0].id"
                    @click="jumpToBookmark(bookmark)"
                    class="bookmark-item">
                    <template v-slot:prepend>
                      <v-icon :color="bookmark[0].color" size="small">mdi-bookmark</v-icon>
                    </template>
                    <v-list-item-content>
                      <v-list-item-title class="text-body-2">{{ bookmark[0].title || '标记' }}</v-list-item-title>
                      <v-list-item-subtitle class="text-caption">{{ bookmark[1].timestamp }}</v-list-item-subtitle>
                    </v-list-item-content>
                  </v-list-item>
                  <v-list-item v-if="bookmarks.length === 0">
                    <v-list-item-content>
                      <v-icon class="mr-2 text-disabled">mdi-bookmark-outline</v-icon>
                      <v-list-item-title class="text--secondary text-body-2">暂无书签</v-list-item-title>
                    </v-list-item-content>
                  </v-list-item>
                </v-list>
              </v-card-text>
            </v-card>

            <!-- Sessions panel -->
            <v-card elevation="2">
              <v-card-title class="bg-green-lighten-5">
                <v-icon class="mr-2">mdi-test-tube</v-icon>
                测试会话
                <v-chip size="small" color="success" variant="flat" class="ml-auto">
                  {{ sessions.length }}
                </v-chip>
              </v-card-title>
              <v-card-text class="pa-0">
                <v-list dense>
                  <v-list-item
                    v-for="session in sessions"
                    :key="session.id"
                    :class="{ 'bg-primary-lighten-5': currentSession === session.id }"
                    @click="currentSession = session.id; refreshLogs()"
                    class="session-item">
                    <template v-slot:prepend>
                      <v-icon size="small" :color="currentSession === session.id ? 'primary' : 'grey'">mdi-folder</v-icon>
                    </template>
                    <v-list-item-content>
                      <v-list-item-title class="text-body-2">{{ session.name }}</v-list-item-title>
                      <v-list-item-subtitle class="text-caption">{{ session.total_entries }} 条记录</v-list-item-subtitle>
                    </v-list-item-content>
                  </v-list-item>
                </v-list>
              </v-card-text>
            </v-card>
          </v-col>

          <!-- Main content -->
          <v-col cols="9">
            <!-- Filters -->
            <v-card class="mb-4" elevation="2">
              <v-card-title class="bg-grey-lighten-4">
                <v-icon class="mr-2">mdi-filter-variant</v-icon>
                过滤和搜索
              </v-card-title>
              <v-card-text>
                <v-row>
                  <v-col cols="3">
                    <v-select
                      v-model="levelFilter"
                      :items="logLevels"
                      label="日志级别"
                      clearable
                      prepend-inner-icon="mdi-filter"
                      variant="outlined"
                      density="comfortable"
                      @change="refreshLogs">
                    </v-select>
                  </v-col>
                  <v-col cols="6">
                    <v-text-field
                      v-model="searchTerm"
                      label="搜索内容"
                      prepend-inner-icon="mdi-magnify"
                      variant="outlined"
                      density="comfortable"
                      clearable
                      @input="debouncedSearch">
                    </v-text-field>
                  </v-col>
                  <v-col cols="3">
                    <v-btn @click="refreshLogs" color="primary" :loading="loading" prepend-icon="mdi-refresh" size="large">
                      刷新
                    </v-btn>
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>

            <!-- Log entries table -->
            <v-card elevation="2">
              <v-card-title class="bg-blue-lighten-5">
                <v-icon class="mr-2">mdi-file-table-outline</v-icon>
                日志条目
                <v-chip size="small" color="info" variant="flat" class="ml-auto">
                  {{ totalEntries }} 条记录
                </v-chip>
              </v-card-title>
              <v-card-text class="pa-0">
                <v-data-table
                  :headers="headers"
                  :items="logEntries"
                  :loading="loading"
                  :server-items-length="totalEntries"
                  :options.sync="options"
                  @update:options="handlePagination"
                  class="elevation-0"
                  density="compact"
                  fixed-header
                  height="calc(100vh - 280px)">

                  <template v-slot:item.level="{ item }">
                    <v-chip
                      :color="getLevelColor(item.level)"
                      dark
                      small>
                      {{ item.level }}
                    </v-chip>
                  </template>

                  <template v-slot:item.stack="{ item }">
                    <v-tooltip bottom>
                      <template v-slot:activator="{ on }">
                        <span v-on="on" class="text-truncate" style="max-width: 200px;">
                          {{ item.stack }}
                        </span>
                      </template>
                      <span>{{ item.stack }}</span>
                    </v-tooltip>
                  </template>

                  <template v-slot:item.actions="{ item }">
                    <v-tooltip text="添加书签">
                      <template v-slot:activator="{ props }">
                        <v-btn
                          v-bind="props"
                          icon
                          size="small"
                          color="warning"
                          variant="text"
                          @click="addBookmark(item)">
                          <v-icon size="small">mdi-bookmark-plus-outline</v-icon>
                        </v-btn>
                      </template>
                    </v-tooltip>
                  </template>

                </v-data-table>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>

<style scoped>
/* 自定义样式用于日志查看器 */
.log-level-chip {
  font-weight: 500;
  font-size: 0.75rem;
}

.bookmark-item:hover {
  background-color: rgba(255, 193, 7, 0.1);
}

.session-item:hover {
  background-color: rgba(25, 118, 210, 0.1);
}

/* 数据表样式优化 */
:deep(.v-data-table__wrapper) {
  border-radius: 4px;
}

:deep(.v-data-table-header th) {
  background-color: #f5f5f5;
  font-weight: 600;
}

/* 卡片标题样式 */
.v-card-title {
  font-size: 1rem;
  font-weight: 500;
}

/* 响应式调整 */
@media (max-width: 960px) {
  .v-col-3 {
    flex: 0 0 100%;
    max-width: 100%;
  }

  .v-col-9 {
    flex: 0 0 100%;
    max-width: 100%;
  }
}
</style>
