use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::{
    DynDownloader, DynPageProcessor, DynPipeline, DynScheduler, Request, SpiderEngine, SpiderError,
};

#[derive(Clone)]
pub struct SpiderParts {
    pub downloader: Arc<DynDownloader>,
    pub processor: Arc<DynPageProcessor>,
    pub scheduler: Arc<DynScheduler>,
    pub pipeline: Arc<DynPipeline>,
}

#[derive(Clone)]
pub struct Spider {
    pub parts: SpiderParts,
    pub engine: SpiderEngine,
}

/// Aggregate counters reported back from a single Spider run.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub processed: usize,
    pub items: usize,
    pub discovered: usize,
    pub errors: usize,
}

impl Spider {
    /// Drive the spider to completion.
    ///
    /// Wiring summary:
    /// - Seed requests are submitted via `Scheduler::schedule`, which deduplicates and then
    ///   pushes accepted requests directly into the engine's domain dispatcher.
    /// - Workers consume requests from `SpiderEngine::worker_receiver()` (the global MPMC
    ///   "main artery"), call `Downloader → PageProcessor → Pipeline`, and feed newly
    ///   discovered requests back into the same scheduler.
    /// - A single in-flight counter governs termination: when every accepted request has
    ///   been processed and no new ones are pending, the scheduler is closed (no further
    ///   submissions accepted) and the engine is shut down (workers observe end-of-stream
    ///   and exit cleanly).
    pub async fn run(self, seeds: Vec<Request>) -> Result<RunReport, SpiderError> {
        let parts = self.parts;
        let engine = self.engine;
        let worker_count = engine.worker_count();
        let worker_rx = engine.worker_receiver();

        let report = Arc::new(Mutex::new(RunReport::default()));
        let (done_tx, mut done_rx) = mpsc::unbounded_channel::<WorkerOutcome>();

        let mut worker_handles = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let rx = worker_rx.clone();
            let downloader = parts.downloader.clone();
            let processor = parts.processor.clone();
            let pipeline = parts.pipeline.clone();
            let scheduler = parts.scheduler.clone();
            let done_tx = done_tx.clone();

            worker_handles.push(tokio::spawn(async move {
                while let Ok(request) = rx.recv().await {
                    let outcome =
                        process_one(&downloader, &processor, &pipeline, &scheduler, request).await;
                    if done_tx.send(outcome).is_err() {
                        break;
                    }
                }
            }));
        }
        drop(done_tx);

        let initial = parts.scheduler.schedule(seeds).await?;
        let mut in_flight = initial.accepted;

        if in_flight == 0 {
            parts.scheduler.close().await?;
            engine.shutdown();
            for handle in worker_handles {
                let _ = handle.await;
            }
            let final_report = report.lock().await.clone();
            return Ok(final_report);
        }

        while in_flight > 0 {
            let Some(outcome) = done_rx.recv().await else {
                break;
            };
            {
                let mut rep = report.lock().await;
                rep.processed += 1;
                rep.items += outcome.items;
                rep.discovered += outcome.discovered_accepted;
                if outcome.failed {
                    rep.errors += 1;
                }
            }

            in_flight = in_flight + outcome.discovered_accepted - 1;
        }

        parts.scheduler.close().await?;
        engine.shutdown();

        for handle in worker_handles {
            let _ = handle.await;
        }

        let final_report = report.lock().await.clone();
        Ok(final_report)
    }
}

#[derive(Debug, Clone)]
struct WorkerOutcome {
    items: usize,
    discovered_accepted: usize,
    failed: bool,
}

async fn process_one(
    downloader: &Arc<DynDownloader>,
    processor: &Arc<DynPageProcessor>,
    pipeline: &Arc<DynPipeline>,
    scheduler: &Arc<DynScheduler>,
    request: Request,
) -> WorkerOutcome {
    let page = match downloader.download(request).await {
        Ok(page) => page,
        Err(_) => {
            return WorkerOutcome {
                items: 0,
                discovered_accepted: 0,
                failed: true,
            };
        }
    };

    let result = match processor.process(page) {
        Ok(result) => result,
        Err(_) => {
            return WorkerOutcome {
                items: 0,
                discovered_accepted: 0,
                failed: true,
            };
        }
    };

    let item_count = result.items.len();
    let mut failed = false;
    for item in result.items {
        if pipeline.process(item).await.is_err() {
            failed = true;
        }
    }

    let discovered_accepted = if result.requests.is_empty() {
        0
    } else {
        match scheduler.schedule(result.requests).await {
            Ok(batch) => batch.accepted,
            Err(_) => {
                failed = true;
                0
            }
        }
    };

    WorkerOutcome {
        items: item_count,
        discovered_accepted,
        failed,
    }
}
