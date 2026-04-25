use std::sync::Arc;

use crate::{
    DefaultScheduler, Downloader, EngineConfig, MemoryDuplicateRemover, PageProcessor, Pipeline,
    Scheduler, Spider, SpiderEngine, SpiderError, SpiderParts, SpiderStage,
};

pub type DynDownloader = dyn Downloader<Error = SpiderError>;
pub type DynPageProcessor = dyn PageProcessor<Error = SpiderError>;
pub type DynScheduler = dyn Scheduler<Error = SpiderError>;
pub type DynPipeline = dyn Pipeline<Error = SpiderError>;
// `DynDuplicateRemover` is re-exported from `crate::scheduler`; do not redeclare here.
use crate::DynDuplicateRemover;

/// SchedulerWiring captures how the scheduler facade should be built at `build` time.
///
/// In the integrated runtime, the scheduler always sits on top of a `SpiderEngine`, so most
/// wiring options just choose how the deduplicator is provided. Custom schedulers that own
/// their own dispatch path can still be injected directly.
enum SchedulerWiring {
    DefaultMemoryDedup,
    DefaultWithDedup(Arc<DynDuplicateRemover>),
    Custom(Arc<DynScheduler>),
}

impl Default for SchedulerWiring {
    fn default() -> Self {
        Self::DefaultMemoryDedup
    }
}

#[derive(Default)]
pub struct SpiderBuilder {
    downloader: Option<Arc<DynDownloader>>,
    processor: Option<Arc<DynPageProcessor>>,
    scheduler: SchedulerWiring,
    pipeline: Option<Arc<DynPipeline>>,
    engine_config: Option<EngineConfig>,
}

impl SpiderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn downloader(mut self, downloader: Arc<DynDownloader>) -> Self {
        self.downloader = Some(downloader);
        self
    }

    pub fn page_processor(mut self, processor: Arc<DynPageProcessor>) -> Self {
        self.processor = Some(processor);
        self
    }

    pub fn pipeline(mut self, pipeline: Arc<DynPipeline>) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Override the engine configuration. When omitted, `EngineConfig::default()` is used.
    pub fn engine_config(mut self, config: EngineConfig) -> Self {
        self.engine_config = Some(config);
        self
    }

    /// Inject a custom deduplicator while keeping the default engine-backed scheduler.
    pub fn deduplicator(mut self, deduplicator: Arc<DynDuplicateRemover>) -> Self {
        self.scheduler = SchedulerWiring::DefaultWithDedup(deduplicator);
        self
    }

    /// Inject a fully custom scheduler. The supplied scheduler is responsible for routing
    /// accepted requests to whatever execution backend it represents.
    pub fn scheduler(mut self, scheduler: Arc<DynScheduler>) -> Self {
        self.scheduler = SchedulerWiring::Custom(scheduler);
        self
    }

    pub fn build(self) -> Result<Spider, SpiderError> {
        let engine = SpiderEngine::new(self.engine_config.unwrap_or_default());

        let scheduler = match self.scheduler {
            SchedulerWiring::DefaultMemoryDedup => {
                let deduplicator: Arc<DynDuplicateRemover> =
                    Arc::new(MemoryDuplicateRemover::new());
                Arc::new(DefaultScheduler::new(deduplicator, engine.clone())) as Arc<DynScheduler>
            }
            SchedulerWiring::DefaultWithDedup(deduplicator) => {
                Arc::new(DefaultScheduler::new(deduplicator, engine.clone())) as Arc<DynScheduler>
            }
            SchedulerWiring::Custom(scheduler) => scheduler,
        };

        Ok(Spider {
            parts: SpiderParts {
                downloader: self
                    .downloader
                    .ok_or_else(|| SpiderError::new(SpiderStage::Build, "missing downloader"))?,
                processor: self.processor.ok_or_else(|| {
                    SpiderError::new(SpiderStage::Build, "missing page processor")
                })?,
                scheduler,
                pipeline: self
                    .pipeline
                    .ok_or_else(|| SpiderError::new(SpiderStage::Build, "missing pipeline"))?,
            },
            engine,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use serde_json::json;

    use super::*;
    use crate::{
        BoxFuture, DedupOutcome, DuplicateRemover, HeaderMap, Item, Page, ProcessResult,
        QueueOutcome, Request, ScheduleBatchResult, ScheduledRequest,
    };

    struct NoopDownloader;
    struct NoopProcessor;
    struct NoopScheduler;
    struct NoopPipeline;

    impl Downloader for NoopDownloader {
        type Error = SpiderError;

        fn download(&self, request: Request) -> BoxFuture<'_, Result<Page, Self::Error>> {
            Box::pin(async move {
                Ok(Page {
                    final_url: request.url.clone(),
                    request,
                    status_code: 200,
                    headers: HeaderMap::new(),
                    body: Vec::new(),
                })
            })
        }
    }

    impl PageProcessor for NoopProcessor {
        type Error = SpiderError;

        fn process(&self, _page: Page) -> Result<ProcessResult, Self::Error> {
            Ok(ProcessResult {
                items: vec![Item::new(BTreeMap::from([("ok".to_string(), json!(true))]))],
                requests: vec![],
            })
        }
    }

    impl Scheduler for NoopScheduler {
        type Error = SpiderError;

        fn schedule(
            &self,
            requests: Vec<Request>,
        ) -> BoxFuture<'_, Result<ScheduleBatchResult, Self::Error>> {
            Box::pin(async move {
                Ok(ScheduleBatchResult {
                    accepted: requests.len(),
                    dropped: 0,
                    results: requests
                        .into_iter()
                        .map(|request| {
                            ScheduledRequest::accepted(
                                request,
                                DedupOutcome::New,
                                QueueOutcome::Enqueued,
                            )
                        })
                        .collect(),
                })
            })
        }

        fn close(&self) -> BoxFuture<'_, Result<(), Self::Error>> {
            Box::pin(async { Ok(()) })
        }
    }

    impl Pipeline for NoopPipeline {
        type Error = SpiderError;

        fn process(&self, _item: Item) -> BoxFuture<'_, Result<(), Self::Error>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[test]
    fn spider_builder_requires_all_components() {
        let err = match SpiderBuilder::new().build() {
            Ok(_) => panic!("builder should fail without components"),
            Err(err) => err,
        };

        assert_eq!(err.stage, SpiderStage::Build);
    }

    #[test]
    fn spider_builder_builds_with_all_components() {
        let spider = SpiderBuilder::new()
            .downloader(Arc::new(NoopDownloader))
            .page_processor(Arc::new(NoopProcessor))
            .pipeline(Arc::new(NoopPipeline))
            .build()
            .expect("builder should succeed");

        let _ = spider.parts.scheduler.clone();
    }

    #[test]
    fn spider_builder_accepts_custom_scheduler_override() {
        let spider = SpiderBuilder::new()
            .downloader(Arc::new(NoopDownloader))
            .page_processor(Arc::new(NoopProcessor))
            .scheduler(Arc::new(NoopScheduler))
            .pipeline(Arc::new(NoopPipeline))
            .build()
            .expect("builder should succeed");

        let _ = spider.parts.scheduler.clone();
    }

    #[test]
    fn spider_builder_accepts_custom_deduplicator() {
        struct NoopDedup;

        impl DuplicateRemover for NoopDedup {
            type Error = SpiderError;

            fn is_duplicate(&self, _request: &Request) -> BoxFuture<'_, Result<bool, Self::Error>> {
                Box::pin(async { Ok(false) })
            }
        }

        let spider = SpiderBuilder::new()
            .downloader(Arc::new(NoopDownloader))
            .page_processor(Arc::new(NoopProcessor))
            .deduplicator(Arc::new(NoopDedup))
            .pipeline(Arc::new(NoopPipeline))
            .build()
            .expect("builder should succeed");

        let _ = spider.parts.scheduler.clone();
    }
}
