use std::{future::Future, pin::Pin};

use crate::{Page, Request, SpiderError};

/// Shared boxed future type for async crawler contracts without binding the core crate
/// to a specific async runtime implementation.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Downloader owns network I/O concerns.
///
/// Implementations are responsible for protocol details, retries, proxies, compression,
/// connection reuse, and transport-layer policies. They should not own crawl graph logic,
/// parsing logic, or persistence logic.
pub trait Downloader: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn download(&self, request: Request) -> BoxFuture<'_, Result<Page, Self::Error>>;
}
