use serde::{Deserialize, Serialize};

/// 下载进度状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressStatus {
    Connecting,
    Scanning { found: usize },
    Downloading {
        total_sessions: usize,
        current_session: usize,
        total_files: usize,
        completed_files: usize,
        failed_files: usize,
        speed: String,
        files: Vec<FileStatus>,
    },
    Parsing { session: String },
    Complete,
}

/// 文件下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub file_url: String,
    pub status: FileDownloadStatus,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

/// 文件下载状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileDownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Retrying,
}

/// 下载结果
#[derive(Debug)]
pub struct DownloadResult {
    pub url: String,
    pub content: String,
    pub bytes_downloaded: u64,
}

impl DownloadResult {
    pub fn new(url: String, content: String, bytes_downloaded: u64) -> Self {
        Self {
            url,
            content,
            bytes_downloaded,
        }
    }
}
