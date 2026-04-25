use crate::{BoxFuture, Item, SpiderError};

/// Pipeline owns result sinks and persistence concerns.
///
/// Implementations may write to stdout, files, databases, or external systems, but they
/// should not parse pages or control crawl ordering.
pub trait Pipeline: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn process(&self, item: Item) -> BoxFuture<'_, Result<(), Self::Error>>;
}
