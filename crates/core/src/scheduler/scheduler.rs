use crate::{BoxFuture, Request, SpiderError};

/// Scheduler is the facade over deduplication and queueing.
///
/// Callers submit requests through this boundary instead of separately invoking a
/// deduplicator and a queue implementation.
pub trait Scheduler: Send + Sync {
    type Error: From<SpiderError>;

    fn schedule(
        &self,
        requests: Vec<Request>,
    ) -> BoxFuture<'_, Result<ScheduleBatchResult, Self::Error>>;
}

/// Batch-level scheduling feedback returned by the scheduler facade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleBatchResult {
    pub accepted: usize,
    pub dropped: usize,
    pub results: Vec<ScheduledRequest>,
}

impl ScheduleBatchResult {
    pub fn empty() -> Self {
        Self {
            accepted: 0,
            dropped: 0,
            results: Vec::new(),
        }
    }
}

/// Per-request scheduling feedback that merges deduplication and queueing results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRequest {
    pub request: Request,
    pub dedup: DedupOutcome,
    pub queue: QueueOutcome,
}

impl ScheduledRequest {
    pub fn accepted(request: Request, dedup: DedupOutcome, queue: QueueOutcome) -> Self {
        Self {
            request,
            dedup,
            queue,
        }
    }
}

/// Deduplication outcome is intentionally exposed so callers can distinguish
/// duplicate drops from queueing failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupOutcome {
    New,
    Duplicate,
}

/// Queue outcome captures whether the scheduler actually accepted the request into its queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueOutcome {
    Enqueued,
    Dropped,
}
