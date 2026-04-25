use crate::{Page, ProcessResult, SpiderError};

/// PageProcessor is a stateless page-to-result transform.
///
/// Implementations interpret a downloaded page and produce extracted data plus newly
/// discovered requests, but they do not own scheduling, deduplication, or pipeline concerns.
pub trait PageProcessor: Send + Sync {
    type Error: From<SpiderError>;

    fn process(&self, page: Page) -> Result<ProcessResult, Self::Error>;
}
