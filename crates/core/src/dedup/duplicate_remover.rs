use crate::{BoxFuture, Request, SpiderError};

pub trait DuplicateRemover: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn is_duplicate(&self, request: &Request) -> BoxFuture<'_, Result<bool, Self::Error>>;
}
