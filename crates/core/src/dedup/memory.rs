use dashmap::DashSet;

use crate::{BoxFuture, DuplicateRemover, Request, SpiderError};

#[derive(Default)]
pub struct MemoryDuplicateRemover {
    seen_urls: DashSet<String>,
}

impl MemoryDuplicateRemover {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DuplicateRemover for MemoryDuplicateRemover {
    type Error = SpiderError;

    fn is_duplicate(&self, request: &Request) -> BoxFuture<'_, Result<bool, Self::Error>> {
        let is_duplicate = !self.seen_urls.insert(request.url.clone());
        Box::pin(async move { Ok(is_duplicate) })
    }
}
