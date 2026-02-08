use reqwest::Client;
use reqwest::Url;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use super::types::DownloadResult;
use super::progress_tracker::SpeedCalculator;
use crate::http_log_fetcher::HttpFetchError;

/// Async HTTP log fetcher with retry support
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

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
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

        while retry_count <= max_retries {
            if retry_count > 0 {
                // Exponential backoff: 100ms, 200ms, 400ms...
                let backoff = 100 * (2_u64.pow(retry_count - 1));
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }

            match self.fetch_file_once(url, &bytes_downloaded, &speed_calculator).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Fetch file once (single attempt)
    async fn fetch_file_once(
        &self,
        url: &str,
        bytes_downloaded: &AtomicU64,
        speed_calculator: &SpeedCalculator,
    ) -> Result<DownloadResult, HttpFetchError> {
        println!("[ASYNC_DL] Starting download: {}", url);
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| HttpFetchError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(HttpFetchError::DownloadFailed {
                url: url.to_string(),
                reason: format!("HTTP status: {}", response.status()),
            });
        }

        // Get content length for progress tracking
        let content_length = response.content_length().unwrap_or(0);
        println!("[ASYNC_DL] Content-Length: {} bytes", content_length);

        // Download with progress tracking - use Vec<u8> to handle large files
        let mut downloaded = 0u64;
        let mut buffer = Vec::with_capacity(content_length as usize);

        use futures::stream::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| HttpFetchError::NetworkError(e))?;

            downloaded += chunk.len() as u64;
            bytes_downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
            speed_calculator.add_sample(bytes_downloaded.load(Ordering::Relaxed));

            // Append chunk to buffer - NO SIZE LIMIT
            buffer.extend_from_slice(&chunk);

            // Log progress every 5MB
            if downloaded % (5 * 1024 * 1024) == 0 || downloaded == content_length {
                println!("[ASYNC_DL] Download progress: {} / {} bytes ({}%)",
                    downloaded, content_length,
                    if content_length > 0 { (downloaded * 100 / content_length) } else { 0 });
            }
        }

        // Convert to String at the end
        let final_content = String::from_utf8_lossy(&buffer).to_string();
        println!("[ASYNC_DL] Completed download: {} ({} bytes, {} chars in string)",
            url, downloaded, final_content.len());

        // Verify content完整性
        if final_content.is_empty() {
            log::error!("[ASYNC_DL] WARNING: Downloaded content is empty for {}", url);
        } else {
            log::info!("[ASYNC_DL] Successfully downloaded {} with {} characters", url, final_content.len());
        }

        Ok(DownloadResult::new(url.to_string(), final_content, downloaded))
    }

    /// Parse directory listing HTML and extract all file URLs
    pub async fn parse_directory_listing(
        &self,
        html: &str,
    ) -> Result<Vec<String>, HttpFetchError> {
        // Use the synchronous parser from existing code
        // This is OK because parsing is CPU-bound, not I/O bound
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
