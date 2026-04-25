use std::sync::Arc;

use crate::{
    DefaultScheduler, Downloader, DuplicateRemover, MemoryDuplicateRemover, MemoryRequestQueue,
    PageProcessor, Pipeline, RequestQueue, Scheduler, Spider, SpiderError, SpiderParts,
    SpiderStage,
};

pub type DynDownloader = dyn Downloader<Error = SpiderError>;
pub type DynPageProcessor = dyn PageProcessor<Error = SpiderError>;
pub type DynScheduler = dyn Scheduler<Error = SpiderError>;
pub type DynPipeline = dyn Pipeline<Error = SpiderError>;
pub type DynDuplicateRemover = dyn DuplicateRemover<Error = SpiderError>;
pub type DynRequestQueue = dyn RequestQueue<Error = SpiderError>;

const DEFAULT_MEMORY_QUEUE_CAPACITY: usize = 1024;

enum SchedulerConfig {
    Memory {
        queue_capacity: usize,
    },
    Distributed {
        deduplicator: Arc<DynDuplicateRemover>,
        queue: Arc<DynRequestQueue>,
    },
    Custom {
        scheduler: Arc<DynScheduler>,
    },
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self::Memory {
            queue_capacity: DEFAULT_MEMORY_QUEUE_CAPACITY,
        }
    }
}

#[derive(Default)]
pub struct SpiderBuilder {
    downloader: Option<Arc<DynDownloader>>,
    processor: Option<Arc<DynPageProcessor>>,
    scheduler: SchedulerConfig,
    pipeline: Option<Arc<DynPipeline>>,
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

    pub fn scheduler(mut self, scheduler: Arc<DynScheduler>) -> Self {
        self.scheduler = SchedulerConfig::Custom { scheduler };
        self
    }

    pub fn pipeline(mut self, pipeline: Arc<DynPipeline>) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn with_memory_scheduler(mut self, queue_capacity: usize) -> Self {
        self.scheduler = SchedulerConfig::Memory { queue_capacity };
        self
    }

    pub fn with_distributed_scheduler(
        mut self,
        deduplicator: Arc<DynDuplicateRemover>,
        queue: Arc<DynRequestQueue>,
    ) -> Self {
        self.scheduler = SchedulerConfig::Distributed {
            deduplicator,
            queue,
        };
        self
    }

    pub fn build(self) -> Result<Spider, SpiderError> {
        let scheduler = match self.scheduler {
            SchedulerConfig::Memory { queue_capacity } => {
                let deduplicator = Arc::new(MemoryDuplicateRemover::new());
                let (queue, _receiver) = MemoryRequestQueue::bounded(queue_capacity);
                Arc::new(DefaultScheduler::new(deduplicator, Arc::new(queue))) as Arc<DynScheduler>
            }
            SchedulerConfig::Distributed {
                deduplicator,
                queue,
            } => Arc::new(DefaultScheduler::new(deduplicator, queue)) as Arc<DynScheduler>,
            SchedulerConfig::Custom { scheduler } => scheduler,
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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use serde_json::json;

    use super::*;
    use crate::{
        BoxFuture, DedupOutcome, HeaderMap, Item, Page, ProcessResult, QueueOutcome, Request,
        ScheduleBatchResult, ScheduledRequest,
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
    fn spider_builder_accepts_distributed_scheduler_parts() {
        struct NoopDedup;
        struct NoopQueue;

        impl DuplicateRemover for NoopDedup {
            type Error = SpiderError;

            fn is_duplicate(&self, _request: &Request) -> BoxFuture<'_, Result<bool, Self::Error>> {
                Box::pin(async { Ok(false) })
            }
        }

        impl RequestQueue for NoopQueue {
            type Error = SpiderError;

            fn push(&self, _request: Request) -> BoxFuture<'_, Result<(), Self::Error>> {
                Box::pin(async { Ok(()) })
            }
        }

        let spider = SpiderBuilder::new()
            .downloader(Arc::new(NoopDownloader))
            .page_processor(Arc::new(NoopProcessor))
            .with_distributed_scheduler(Arc::new(NoopDedup), Arc::new(NoopQueue))
            .pipeline(Arc::new(NoopPipeline))
            .build()
            .expect("builder should succeed");

        let _ = spider.parts.scheduler.clone();
    }
}
