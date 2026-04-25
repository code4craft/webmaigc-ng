use crate::{BoxFuture, Request, SpiderError};

/// Scheduler is the facade over deduplication and request submission.
///
/// In the integrated runtime model the scheduler routes accepted requests directly into
/// the `SpiderEngine` (per-domain dispatch + global worker channel). Callers therefore
/// only need to submit requests and signal shutdown — pulling work is the engine's job.
pub trait Scheduler: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn schedule(
        &self,
        requests: Vec<Request>,
    ) -> BoxFuture<'_, Result<ScheduleBatchResult, Self::Error>>;

    /// Stop accepting further submissions. Idempotent. After `close`, subsequent
    /// `schedule` calls SHALL return all requests as `QueueOutcome::Dropped`.
    fn close(&self) -> BoxFuture<'_, Result<(), Self::Error>>;
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
