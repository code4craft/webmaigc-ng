use std::sync::Arc;

use crate::{
    BoxFuture, DedupOutcome, DuplicateRemover, QueueOutcome, Request, RequestQueue,
    ScheduleBatchResult, ScheduledRequest, Scheduler, SpiderError,
};

pub type DynDuplicateRemover = dyn DuplicateRemover<Error = SpiderError>;
pub type DynRequestQueue = dyn RequestQueue<Error = SpiderError>;

pub struct DefaultScheduler {
    deduplicator: Arc<DynDuplicateRemover>,
    queue: Arc<DynRequestQueue>,
}

impl DefaultScheduler {
    pub fn new(deduplicator: Arc<DynDuplicateRemover>, queue: Arc<DynRequestQueue>) -> Self {
        Self {
            deduplicator,
            queue,
        }
    }
}

impl Scheduler for DefaultScheduler {
    type Error = SpiderError;

    fn schedule(
        &self,
        requests: Vec<Request>,
    ) -> BoxFuture<'_, Result<ScheduleBatchResult, Self::Error>> {
        let deduplicator = self.deduplicator.clone();
        let queue = self.queue.clone();

        Box::pin(async move {
            let mut accepted = 0;
            let mut dropped = 0;
            let mut results = Vec::with_capacity(requests.len());

            for request in requests {
                if deduplicator.is_duplicate(&request).await? {
                    dropped += 1;
                    results.push(ScheduledRequest {
                        request,
                        dedup: DedupOutcome::Duplicate,
                        queue: QueueOutcome::Dropped,
                    });
                    continue;
                }

                queue.push(request.clone()).await?;
                accepted += 1;
                results.push(ScheduledRequest::accepted(
                    request,
                    DedupOutcome::New,
                    QueueOutcome::Enqueued,
                ));
            }

            Ok(ScheduleBatchResult {
                accepted,
                dropped,
                results,
            })
        })
    }
}
