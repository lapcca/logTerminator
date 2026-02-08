# HTTP 并行下载设计文档

## 概述

将当前串行的 HTTP 日志下载改为并行下载架构，使用 Async/Await 模型，每个 test session 内使用 6 个并发线程下载文件，同时支持最多 2 个 session 并行处理。系统会显示总下载速度和每个文件的详细状态，失败文件会自动重试。

## 目标

- **性能提升**: 从串行下载改为并行下载，预计加速 6-12 倍
- **可靠性**: 添加重试机制，自动处理临时网络故障
- **可见性**: 提供详细的下载进度和实时速度显示

## 技术栈变更

### 依赖更新

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }  # 移除 blocking
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

### 架构变更

从同步阻塞 (`reqwest::blocking`) 切换到异步非阻塞 (`reqwest::async` + `tokio`)。

## 架构设计

### 并发控制模型

```
全局并发控制：
├── Session 级别: Semaphore(2) - 最多 2 个 session 并行
│   ├── Session 1: 6 个文件并行下载
│   └── Session 2: 6 个文件并行下载
└── 文件下载: 每个 session 内部 Semaphore(6)
```

**最大并发数**: 2 sessions × 6 files = 12 个并发下载任务

### 数据流

```
HTTP URL → 扫描目录 → 分组为 sessions
    ↓
并行下载 (2 sessions × 6 files)
    ↓
收集结果 → 解析 HTML → 存入数据库
    ↓
更新进度和速度显示
```

## 核心组件

### 1. AsyncHttpLogFetcher

异步 HTTP 日志获取器，支持重试机制。

```rust
pub struct AsyncHttpLogFetcher {
    client: reqwest::Client,
}

impl AsyncHttpLogFetcher {
    pub async fn new(url: &str) -> Result<Self, HttpFetchError>;

    pub async fn fetch_file_with_retry(
        &self,
        url: &str,
        max_retries: u32,
        progress: Arc<AtomicU64>,
    ) -> Result<(String, u64), HttpFetchError>;
}
```

### 2. SessionDownloadCoordinator

Session 下载协调器，管理并发控制。

```rust
pub struct SessionDownloadCoordinator {
    max_sessions: usize,        // 2
    max_files_per_session: usize,  // 6
    max_retries: u32,           // 2
}

impl SessionDownloadCoordinator {
    pub async fn download_sessions(
        &self,
        url: String,
        selected_tests: Option<Vec<String>>,
        db_manager: Arc<Mutex<DatabaseManager>>,
        progress_callback: Arc<dyn Fn(ProgressStatus) + Send + Sync>,
    ) -> Result<Vec<String>, HttpFetchError>;
}
```

### 3. SpeedCalculator

速度计算器，跟踪下载速度。

```rust
pub struct SpeedCalculator {
    start_time: Instant,
    samples: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    sample_window: Duration,  // 2 秒窗口
}

impl SpeedCalculator {
    pub fn new() -> Self;

    pub fn add_sample(&self, bytes: u64);

    pub fn calculate_speed(&self) -> f64;

    pub fn format_speed(&self) -> String;
}
```

### 4. 进度状态

```rust
pub enum ProgressStatus {
    Connecting,
    Scanning { found: usize },
    Downloading {
        total_files: usize,
        completed: usize,
        failed: usize,
        speed: String,
        files: Vec<FileStatus>,
    },
    Parsing { session: String },
    Complete,
}

pub struct FileStatus {
    file_url: String,
    status: FileDownloadStatus,
    retry_count: u32,
    error_message: Option<String>,
}

pub enum FileDownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Retrying,
}
```

## 并发实现

### 文件下载并发

```rust
use tokio::sync::Semaphore;

pub async fn download_session(
    session_files: Vec<(String, usize)>,
    session_id: String,
    fetcher: Arc<AsyncHttpLogFetcher>,
    max_concurrent: usize,
    max_retries: u32,
) -> Result<Vec<DownloadResult>, HttpFetchError> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();

    for (file_url, file_index) in session_files {
        let permit = semaphore.clone();
        let fetcher_clone = fetcher.clone();

        let task = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            fetcher_clone.fetch_file_with_retry(&file_url, max_retries).await
        });

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;
    // 处理结果...
}
```

### Session 间并发

```rust
pub async fn download_all_sessions(
    sessions: Vec<SessionInfo>,
    coordinator: &SessionDownloadCoordinator,
) -> Result<Vec<String>, HttpFetchError> {
    let session_semaphore = Arc::new(Semaphore::new(coordinator.max_sessions));
    let mut session_tasks = Vec::new();

    for session in sessions {
        let permit = session_semaphore.clone();
        let task = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            coordinator.download_single_session(session).await
        });
        session_tasks.push(task);
    }

    let results = futures::future::join_all(session_tasks).await;
    // 处理结果...
}
```

## 错误处理

### 重试策略

```
文件下载失败
    │
    ├─→ retry_count < max_retries (2)?
    │   ├─→ Yes: 等待 100ms → 重试
    │   └─→ No: 记录失败，继续
    │
    ├─→ 更新 failed_files 计数
    └─→ 在进度中显示: "[重试] 文件名失败，正在重试..."
```

### 错误传播

- 单个文件失败不影响其他文件
- 单个 session 失败不影响其他 session
- 所有结果收集后统一返回

## 用户界面

### 进度显示格式

```javascript
{
  "type": "downloading",
  "totalSessions": 3,
  "currentSession": 1,
  "totalFiles": 24,
  "completedFiles": 8,
  "failedFiles": 0,
  "speed": "15.2 MB/s",
  "files": [
    { "name": "TestABC_ID_1---0.html", "status": "completed", "size": "2.3 MB" },
    { "name": "TestABC_ID_1---1.html", "status": "downloading", "progress": "45%" },
    { "name": "TestABC_ID_1---2.html", "status": "retrying", "retry": "1/2" }
  ]
}
```

### 前端进度条

```
┌─────────────────────────────────────────────────┐
│ 加载日志文件 - Session 1/3                       │
├─────────────────────────────────────────────────┤
│ 下载中... 15.2 MB/s                              │
│ ████████████░░░░░░░░░░░░░░ 33% (8/24)          │
├─────────────────────────────────────────────────┐
│ 文件状态:                                        │
│ ✓ TestABC_ID_1---0.html (2.3 MB)               │
│ ↻ TestABC_ID_1---1.html (45%)                  │
│ ⚠ TestABC_ID_1---2.html [重试 1/2]             │
│ ⏳ TestABC_ID_1---3.html                         │
│ ⏳ TestABC_ID_1---4.html                         │
│ ⏳ TestABC_ID_1---5.html                         │
│ ⏳ TestABC_ID_1---6.html                         │
└─────────────────────────────────────────────────┘
```

## 性能预期

| 方面 | 当前（串行） | 改进后（并行） |
|------|-------------|---------------|
| 文件下载 | 逐个下载，50ms 延迟 | 6 个并发，无延迟 |
| Session 处理 | 逐个处理 | 2 个 session 并行 |
| 错误处理 | 失败即中断 | 自动重试 2 次 |
| 进度显示 | 简单文本 | 详细状态 + 速度 |
| 下载速度 | ~1-2 MB/s | ~10-20 MB/s |
| 预计加速 | 1x | 6-12x |

## 文件结构

```
src-tauri/src/
├── http_log_fetcher.rs       # 新增：异步 HTTP 日志获取器
│   ├── mod.rs                # 模块导出
│   ├── async_fetcher.rs      # AsyncHttpLogFetcher
│   ├── download_coordinator.rs  # SessionDownloadCoordinator
│   ├── progress_tracker.rs   # DownloadProgress, SpeedCalculator
│   └── types.rs              # DownloadTask, FileStatus 等
```

## Tauri 命令变更

### 新增异步命令

```rust
#[tauri::command]
async fn parse_log_http_url_async(
    _state: State<'_, AppState>,
    window: tauri::Window,
    url: String,
    selected_tests: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    let db_path = "logterminator.db".to_string();
    let progress_callback = Arc::new(|status: ProgressStatus| {
        let msg = serde_json::to_string(&status).unwrap();
        let _ = window.emit("http-progress", msg);
    });

    parse_log_http_url_async_impl(db_path, url, selected_tests, progress_callback)
        .await
        .map_err(|e| e.to_string())
}
```

## 风险和缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 服务器过载 | 连接被拒绝 | 限制并发数为 12 |
| 内存占用高 | 多个大文件同时下载 | 使用流式传输 |
| 数据库锁 | 并发写入冲突 | 每个 session 完成后再写入 |

## 实现优先级

1. **高优先级** - 核心并发下载逻辑
2. **中优先级** - 速度计算和详细进度
3. **低优先级** - 单元测试和性能基准测试
