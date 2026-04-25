use crate::{BoxFuture, Request, SpiderError};

/// `RequestQueue` is a minimal write-side abstraction over an external request queue.
///
/// The default in-process scheduler does not depend on this trait; the `SpiderEngine`'s
/// per-domain dispatcher already plays the role of "bounded queue with backpressure".
/// `RequestQueue` is retained as an extension seam so a distributed scheduler can plug a
/// remote queue (Kafka, Redis Streams, etc.) into the submit side without touching the
/// rest of the runtime.
pub trait RequestQueue: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn push(&self, request: Request) -> BoxFuture<'_, Result<(), Self::Error>>;
}
