use crate::{BoxFuture, Request, SpiderError};

pub trait RequestQueue: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn push(&self, request: Request) -> BoxFuture<'_, Result<(), Self::Error>>;
}
