//! Async HTTP log fetcher for parallel downloads
//!
//! This module provides async/await based HTTP downloading with:
//! - Concurrent file downloads (configurable limit)
//! - Automatic retry on failure
//! - Real-time progress tracking
//! - Speed calculation

mod types;
mod progress_tracker;
mod async_fetcher;
mod download_coordinator;

pub use types::*;
pub use progress_tracker::*;
pub use async_fetcher::*;
pub use download_coordinator::*;
