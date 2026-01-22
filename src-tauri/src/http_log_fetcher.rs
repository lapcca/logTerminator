use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use std::time::Duration;

/// Errors that can occur during HTTP log fetching
pub enum HttpFetchError {
    InvalidUrl(String),
    NetworkError(reqwest::Error),
    TimeoutError,
    DirectoryListingNotFound,
    InvalidDirectoryListingFormat,
    DownloadFailed { url: String, reason: String },
    ParseError(String),
}

impl std::fmt::Display for HttpFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpFetchError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            HttpFetchError::NetworkError(e) => write!(f, "Network error: {}", e),
            HttpFetchError::TimeoutError => write!(f, "Request timeout"),
            HttpFetchError::DirectoryListingNotFound => write!(f, "Directory listing not found"),
            HttpFetchError::InvalidDirectoryListingFormat => write!(f, "Invalid directory listing format"),
            HttpFetchError::DownloadFailed { url, reason } => {
                write!(f, "Failed to download {}: {}", url, reason)
            }
            HttpFetchError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::fmt::Debug for HttpFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for HttpFetchError {}

/// HTTP log fetcher for downloading logs from web servers
pub struct HttpLogFetcher {
    client: Client,
    base_url: Url,
}

impl HttpLogFetcher {
    /// Parse directory listing HTML and extract all file URLs
    pub fn parse_directory_listing(html: &str, base_url: &str) -> Result<Vec<String>, HttpFetchError> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a[href]").unwrap();

        let base = Url::parse(base_url)
            .map_err(|e| HttpFetchError::InvalidUrl(format!("{}: {}", base_url, e)))?;

        let mut urls = Vec::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Skip parent directory links
                if href == "../" || href.starts_with("?") {
                    continue;
                }

                // Skip directory links (ending with /)
                if href.ends_with('/') {
                    continue;
                }

                // Resolve relative URLs against base
                match base.join(href) {
                    Ok(full_url) => {
                        urls.push(full_url.to_string());
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not resolve URL '{}': {}", href, e);
                        continue;
                    }
                }
            }
        }

        Ok(urls)
    }
}
