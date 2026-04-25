use async_channel::{bounded, Receiver, Sender};

use crate::{BoxFuture, Request, RequestQueue, SpiderError, SpiderStage};

#[derive(Clone)]
pub struct MemoryRequestQueue {
    sender: Sender<Request>,
}

impl MemoryRequestQueue {
    pub fn bounded(capacity: usize) -> (Self, Receiver<Request>) {
        let (sender, receiver) = bounded(capacity);
        (Self { sender }, receiver)
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
