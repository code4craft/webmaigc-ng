use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dashmap::DashMap;

use crate::{
    BoxFuture, DedupOutcome, DuplicateRemover, QueueOutcome, Request, ScheduleBatchResult,
    ScheduledRequest, Scheduler, SpiderEngine, SpiderError,
};

pub type DynDuplicateRemover = dyn DuplicateRemover<Error = SpiderError>;

/// Default scheduler that pairs an injectable deduplicator with the engine's "main artery".
///
/// Once a request passes deduplication it is dispatched directly into `SpiderEngine`, which
/// owns the per-domain rate-limited dispatcher and the global MPMC worker channel. There is
/// no intermediate request queue — backpressure flows naturally from the workers up through
/// the engine's bounded channels back into `schedule`.
pub struct DefaultScheduler {
    deduplicator: Arc<DynDuplicateRemover>,
    engine: SpiderEngine,
    closed: AtomicBool,
    accepted_pages_per_site: DashMap<String, usize>,
    max_pages_per_site: Option<usize>,
}

impl DefaultScheduler {
    pub fn new(deduplicator: Arc<DynDuplicateRemover>, engine: SpiderEngine) -> Self {
        Self {
            deduplicator,
            max_pages_per_site: engine.config().max_pages_per_site,
            engine,
            closed: AtomicBool::new(false),
            accepted_pages_per_site: DashMap::new(),
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
}

impl Scheduler for DefaultScheduler {
    type Error = SpiderError;

    fn schedule(
        &self,
        requests: Vec<Request>,
    ) -> BoxFuture<'_, Result<ScheduleBatchResult, Self::Error>> {
        Box::pin(async move {
            let mut accepted = 0;
            let mut dropped = 0;
            let mut results = Vec::with_capacity(requests.len());

            for request in requests {
                if self.is_closed() {
                    dropped += 1;
                    results.push(ScheduledRequest {
                        request,
                        dedup: DedupOutcome::New,
                        queue: QueueOutcome::Dropped,
                    });
                    continue;
                }

                if self.deduplicator.is_duplicate(&request).await? {
                    dropped += 1;
                    results.push(ScheduledRequest {
                        request,
                        dedup: DedupOutcome::Duplicate,
                        queue: QueueOutcome::Dropped,
                    });
                    continue;
                }

                let reserved_domain = match self.max_pages_per_site {
                    Some(max_pages) => {
                        let domain = request.domain_key()?;
                        let mut accepted = self
                            .accepted_pages_per_site
                            .entry(domain.clone())
                            .or_insert(0);
                        if *accepted >= max_pages {
                            dropped += 1;
                            results.push(ScheduledRequest {
                                request,
                                dedup: DedupOutcome::New,
                                queue: QueueOutcome::Dropped,
                            });
                            continue;
                        }
                        *accepted += 1;
                        Some(domain)
                    }
                    None => None,
                };

                let dispatched = self.engine.dispatch(request.clone()).await;
                match dispatched {
                    Ok(()) => {
                        accepted += 1;
                        results.push(ScheduledRequest::accepted(
                            request,
                            DedupOutcome::New,
                            QueueOutcome::Enqueued,
                        ));
                    }
                    Err(err) => {
                        if let Some(domain) = reserved_domain {
                            if let Some(mut accepted) =
                                self.accepted_pages_per_site.get_mut(&domain)
                            {
                                *accepted = accepted.saturating_sub(1);
                            }
                        }
                        // Dispatch failure (engine shut down or domain channel rejected the
                        // send) aborts the batch immediately; callers only receive the error,
                        // so partial queue accounting here would be lost anyway.
                        let _ = request;
                        return Err(err);
                    }
                }
            }

            Ok(ScheduleBatchResult {
                accepted,
                dropped,
                results,
            })
        })
    }

    fn close(&self) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            self.closed.store(true, Ordering::Release);
            Ok(())
        })
    }
}
