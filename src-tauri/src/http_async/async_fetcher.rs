use reqwest::Client;
use reqwest::Url;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use super::types::DownloadResult;
use super::progress_tracker::SpeedCalculator;
use crate::http_log_fetcher::HttpFetchError;

/// Chunk size for downloads (5MB per chunk)
const CHUNK_SIZE: u64 = 5 * 1024 * 1024;

/// Timeout per chunk (60 seconds should be enough for 5MB)
const CHUNK_TIMEOUT_SECS: u64 = 60;

/// Threshold for using chunked download (files larger than 10MB)
const CHUNKED_DOWNLOAD_THRESHOLD: u64 = 10 * 1024 * 1024;

/// Async HTTP log fetcher with retry and chunked download support
#[derive(Clone)]
pub struct AsyncHttpLogFetcher {
    client: Client,
    base_url: Url,
}

impl AsyncHttpLogFetcher {
    /// Create a new async HTTP log fetcher
    pub async fn new(base_url: &str) -> Result<Self, HttpFetchError> {
        let mut url = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

        // Ensure URL path ends with '/'
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }

        // Use longer timeout for chunk-based downloads
        let client = Client::builder()
            .timeout(Duration::from_secs(CHUNK_TIMEOUT_SECS))
            .build()
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        Ok(AsyncHttpLogFetcher {
            client,
            base_url: url,
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Fetch a single file with retry
    pub async fn fetch_file_with_retry(
        &self,
        url: &str,
        max_retries: u32,
        bytes_downloaded: Arc<AtomicU64>,
        speed_calculator: Arc<SpeedCalculator>,
    ) -> Result<DownloadResult, HttpFetchError> {
        let mut retry_count = 0;
        let mut last_error = None;
        let mut error_history = Vec::new();

        while retry_count <= max_retries {
            if retry_count > 0 {
                // Exponential backoff: 100ms, 200ms, 400ms...
                let backoff = 100 * (2_u64.pow(retry_count - 1));
                log::warn!("[ASYNC_DL] Retry {}/{} for {} after {}ms backoff",
                    retry_count, max_retries, url, backoff);
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }

            match self.fetch_file_once(url, &bytes_downloaded, &speed_calculator, retry_count).await {
                Ok(result) => {
                    if retry_count > 0 {
                        log::info!("[ASYNC_DL] Success on retry {} for {}", retry_count, url);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error_detail = format!("Retry {} failed: {}", retry_count, e);
                    log::error!("[ASYNC_DL] {}", error_detail);
                    error_history.push(error_detail);
                    last_error = Some(e);
                    retry_count += 1;
                }
            }
        }

        // Build comprehensive error message with history
        let final_error = format!(
            "Failed to download {} after {} retries.\nError history:\n  {}\nLast error: {}",
            url,
            max_retries,
            error_history.join("\n  "),
            last_error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "Unknown".to_string())
        );
        log::error!("[ASYNC_DL] {}", final_error);

        Err(last_error.unwrap())
    }

    /// Fetch file once (single attempt) - automatically chooses chunked or simple download
    async fn fetch_file_once(
        &self,
        url: &str,
        bytes_downloaded: &AtomicU64,
        speed_calculator: &SpeedCalculator,
        attempt_number: u32,
    ) -> Result<DownloadResult, HttpFetchError> {
        log::info!("[ASYNC_DL] [Attempt {}] Starting download: {}", attempt_number, url);

        // First, do a HEAD request to check file size and Range support
        let (content_length, supports_range) = self.check_file_support(url, attempt_number).await?;

        // Decide whether to use chunked download
        let use_chunked = content_length >= CHUNKED_DOWNLOAD_THRESHOLD && supports_range;

        if use_chunked {
            log::info!("[ASYNC_DL] [Attempt {}] Using chunked download: {} bytes ({} MB), Range support: yes",
                attempt_number, content_length, content_length / 1024 / 1024);
            self.fetch_file_chunked(url, content_length, bytes_downloaded, speed_calculator, attempt_number).await
        } else {
            if content_length >= CHUNKED_DOWNLOAD_THRESHOLD {
                log::warn!("[ASYNC_DL] [Attempt {}] File is large ({} MB) but server doesn't support Range requests, using simple download",
                    attempt_number, content_length / 1024 / 1024);
            } else {
                log::info!("[ASYNC_DL] [Attempt {}] Using simple download: {} bytes ({} MB)",
                    attempt_number, content_length, content_length / 1024 / 1024);
            }
            self.fetch_file_simple(url, bytes_downloaded, speed_calculator, attempt_number).await
        }
    }

    /// Check file size and Range request support using HEAD request
    async fn check_file_support(&self, url: &str, attempt_number: u32) -> Result<(u64, bool), HttpFetchError> {
        log::debug!("[ASYNC_DL] [Attempt {}] Checking file support: {}", attempt_number, url);

        let response = self.client.head(url).send().await
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(HttpFetchError::DownloadFailed {
                url: url.to_string(),
                reason: format!("HEAD request failed: HTTP {}", response.status()),
            });
        }

        let content_length = response.content_length().unwrap_or(0);
        let supports_range = response.headers().get("Accept-Ranges")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("bytes"))
            .unwrap_or(false);

        log::debug!("[ASYNC_DL] [Attempt {}] Content-Length: {}, Accept-Ranges: {}",
            attempt_number, content_length, if supports_range { "bytes" } else { "none" });

        Ok((content_length, supports_range))
    }

    /// Fetch file using chunked download with Range requests
    async fn fetch_file_chunked(
        &self,
        url: &str,
        content_length: u64,
        bytes_downloaded: &AtomicU64,
        speed_calculator: &SpeedCalculator,
        attempt_number: u32,
    ) -> Result<DownloadResult, HttpFetchError> {
        let start_time = std::time::Instant::now();
        let total_chunks = (content_length + CHUNK_SIZE - 1) / CHUNK_SIZE;

        log::info!("[ASYNC_DL] [Attempt {}] Chunked download: {} bytes in {} chunks ({} MB/chunk)",
            attempt_number, content_length, total_chunks, CHUNK_SIZE / 1024 / 1024);

        let mut chunks = Vec::new();
        let mut total_downloaded = 0u64;

        for chunk_index in 0..total_chunks {
            let start = chunk_index * CHUNK_SIZE;
            let end = std::cmp::min(start + CHUNK_SIZE - 1, content_length - 1);

            match self.fetch_chunk(url, chunk_index, start, end, attempt_number).await {
                Ok(chunk_data) => {
                    let chunk_len = chunk_data.len() as u64;
                    total_downloaded += chunk_len;
                    bytes_downloaded.fetch_add(chunk_len, Ordering::Relaxed);
                    speed_calculator.add_sample(bytes_downloaded.load(Ordering::Relaxed));

                    let progress = (total_downloaded * 100) / content_length;
                    log::debug!("[ASYNC_DL] [Attempt {}] Chunk {}/{} downloaded: {} bytes ({}% complete)",
                        attempt_number, chunk_index + 1, total_chunks, chunk_len, progress);

                    chunks.push(chunk_data);
                }
                Err(e) => {
                    log::error!("[ASYNC_DL] [Attempt {}] Failed to download chunk {}/{}: {}",
                        attempt_number, chunk_index + 1, total_chunks, e);
                    return Err(HttpFetchError::DownloadFailed {
                        url: url.to_string(),
                        reason: format!("Failed to download chunk {}/{}: {}", chunk_index + 1, total_chunks, e),
                    });
                }
            }
        }

        // Merge all chunks
        let total_size = chunks.iter().map(|c| c.len()).sum();
        let mut buffer = Vec::with_capacity(total_size);
        for chunk in chunks {
            buffer.extend_from_slice(&chunk);
        }

        // Convert to String
        let final_content = match String::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => {
                log::error!("[ASYNC_DL] [Attempt {}] UTF-8 conversion error at byte {}",
                    attempt_number, e.utf8_error().valid_up_to());
                let recovered = String::from_utf8_lossy(e.as_bytes()).to_string();
                log::warn!("[ASYNC_DL] [Attempt {}] Recovered with lossy conversion", attempt_number);
                recovered
            }
        };

        let elapsed = start_time.elapsed().as_secs();
        log::info!("[ASYNC_DL] [Attempt {}] Chunked download completed in {}s: {} bytes, {} chars",
            attempt_number, elapsed, total_downloaded, final_content.len());

        Ok(DownloadResult::new(url.to_string(), final_content, total_downloaded))
    }

    /// Fetch a single chunk using Range request
    async fn fetch_chunk(
        &self,
        url: &str,
        chunk_index: u64,
        start: u64,
        end: u64,
        attempt_number: u32,
    ) -> Result<Vec<u8>, HttpFetchError> {
        let range_header = format!("bytes={}-{}", start, end);
        log::trace!("[ASYNC_DL] [Attempt {}] Fetching chunk {}: Range: {}",
            attempt_number, chunk_index, range_header);

        let response = self.client
            .get(url)
            .header("Range", range_header)
            .send()
            .await
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        // Check for 206 Partial Content
        if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(HttpFetchError::DownloadFailed {
                url: url.to_string(),
                reason: format!("Expected 206 Partial Content, got {}", response.status()),
            });
        }

        // Get the actual bytes
        let bytes = response.bytes().await
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        Ok(bytes.to_vec())
    }

    /// Fetch file using simple (non-chunked) download
    async fn fetch_file_simple(
        &self,
        url: &str,
        bytes_downloaded: &AtomicU64,
        speed_calculator: &SpeedCalculator,
        attempt_number: u32,
    ) -> Result<DownloadResult, HttpFetchError> {
        log::info!("[ASYNC_DL] [Attempt {}] Starting simple download: {}", attempt_number, url);

        let start_time = std::time::Instant::now();

        let response = match self.client.get(url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs();
                log::error!("[ASYNC_DL] [Attempt {}] Network error after {}s: {}", attempt_number, elapsed, e);
                return Err(HttpFetchError::NetworkError(e));
            }
        };

        let status = response.status();
        if !status.is_success() {
            return Err(HttpFetchError::DownloadFailed {
                url: url.to_string(),
                reason: format!("HTTP {}", status),
            });
        }

        let content_length = response.content_length().unwrap_or(0);
        log::info!("[ASYNC_DL] [Attempt {}] Content-Length: {} bytes ({} MB)",
            attempt_number, content_length, content_length / 1024 / 1024);

        let mut downloaded = 0u64;
        let mut buffer = Vec::with_capacity(content_length as usize);

        use futures::stream::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(ch) => ch,
                Err(e) => {
                    let elapsed = start_time.elapsed().as_secs();
                    log::error!("[ASYNC_DL] [Attempt {}] Stream error after {}s at byte {}: {}",
                        attempt_number, elapsed, downloaded, e);
                    return Err(HttpFetchError::NetworkError(e));
                }
            };

            downloaded += chunk.len() as u64;
            bytes_downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
            speed_calculator.add_sample(bytes_downloaded.load(Ordering::Relaxed));

            buffer.extend_from_slice(&chunk);

            // Log progress every 5MB
            if downloaded % (5 * 1024 * 1024) == 0 || downloaded == content_length {
                let progress = if content_length > 0 {
                    format!("{}%", downloaded * 100 / content_length)
                } else {
                    format!("{} bytes", downloaded)
                };
                log::debug!("[ASYNC_DL] [Attempt {}] Progress: {} / {} bytes ({})",
                    attempt_number, downloaded, content_length, progress);
            }
        }

        // Convert to String
        let final_content = match String::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => {
                let elapsed = start_time.elapsed().as_secs();
                log::error!("[ASYNC_DL] [Attempt {}] UTF-8 conversion error after {}s at byte {}",
                    attempt_number, elapsed, e.utf8_error().valid_up_to());
                let recovered = String::from_utf8_lossy(e.as_bytes()).to_string();
                log::warn!("[ASYNC_DL] [Attempt {}] Recovered with lossy conversion", attempt_number);
                recovered
            }
        };

        let elapsed = start_time.elapsed().as_secs();
        log::info!("[ASYNC_DL] [Attempt {}] Simple download completed in {}s: {} bytes, {} chars",
            attempt_number, elapsed, downloaded, final_content.len());

        Ok(DownloadResult::new(url.to_string(), final_content, downloaded))
    }

    /// Parse directory listing HTML and extract all file URLs
    pub async fn parse_directory_listing(
        &self,
        html: &str,
    ) -> Result<Vec<String>, HttpFetchError> {
        Ok(crate::http_log_fetcher::HttpLogFetcher::parse_directory_listing(
            html,
            self.base_url.as_str(),
        )?)
    }

    /// Fetch directory listing
    pub async fn fetch_directory_listing(&self) -> Result<String, HttpFetchError> {
        let response = self.client
            .get(self.base_url.clone())
            .send()
            .await
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(HttpFetchError::DownloadFailed {
                url: self.base_url.to_string(),
                reason: format!("HTTP status: {}", response.status()),
            });
        }

        response.text().await
            .map_err(|e| HttpFetchError::NetworkError(e))
    }
}
