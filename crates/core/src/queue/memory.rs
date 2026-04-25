use async_channel::{bounded, Receiver, Sender};

use crate::{BoxFuture, Request, RequestQueue, SpiderError, SpiderStage};

/// In-memory bounded request queue retained as an extension seam.
///
/// The default `SpiderBuilder` no longer wires this into the local pipeline; the
/// `SpiderEngine` global channel and per-domain dispatchers already own the in-process
/// bounded buffering. `MemoryRequestQueue` is still exposed so embedders can inject it as
/// a `RequestQueue` implementation (e.g. for tests against the trait or to stage requests
/// into a custom scheduler).
#[derive(Clone)]
pub struct MemoryRequestQueue {
    sender: Sender<Request>,
    receiver: Receiver<Request>,
}

impl MemoryRequestQueue {
    pub fn bounded(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }

    pub fn receiver(&self) -> Receiver<Request> {
        self.receiver.clone()
    }

    pub fn len(&self) -> usize {
        self.sender.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }
}

impl RequestQueue for MemoryRequestQueue {
    type Error = SpiderError;

    fn push(&self, request: Request) -> BoxFuture<'_, Result<(), Self::Error>> {
        let sender = self.sender.clone();
        Box::pin(async move {
            sender
                .send(request)
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Schedule, err.to_string()))
        })
    }
}
