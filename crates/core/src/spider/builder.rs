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
