use std::{collections::BTreeMap, num::NonZeroU32, sync::Arc, time::Duration};

use serde_json::json;

use super::engine::{BackpressureLevel, PullDecision, RobotsPolicy};
use super::*;
use crate::{
    BoxFuture, DedupOutcome, Downloader, DuplicateRemover, HeaderMap, Item, Page, PageProcessor,
    Pipeline, ProcessResult, QueueOutcome, Request, ScheduleBatchResult, ScheduledRequest,
    Scheduler,
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

#[tokio::test]
async fn engine_dispatches_request_to_global_worker_queue() {
    let engine = SpiderEngine::new(EngineConfig::new(4, 8, 8, NonZeroU32::new(50).unwrap()));
    let worker_rx = engine.worker_receiver();

    engine
        .dispatch(Request::get("https://example.com"))
        .await
        .expect("dispatch should succeed");

    let received = worker_rx
        .recv()
        .await
        .expect("worker should receive request");
    assert_eq!(received.url, "https://example.com");
    assert_eq!(engine.active_domains(), 1);
}

#[tokio::test]
async fn engine_creates_independent_domain_dispatchers() {
    let engine = SpiderEngine::new(EngineConfig::new(4, 8, 8, NonZeroU32::new(50).unwrap()));

    engine
        .dispatch(Request::get("https://example.com/a"))
        .await
        .expect("dispatch should succeed");
    engine
        .dispatch(Request::get("https://example.org/b"))
        .await
        .expect("dispatch should succeed");

    assert_eq!(engine.active_domains(), 2);
}

#[tokio::test]
async fn engine_updates_domain_policy_and_robots_cache() {
    let engine = SpiderEngine::new(EngineConfig::new(4, 8, 8, NonZeroU32::new(50).unwrap()));

    engine
        .dispatch(Request::get("https://example.com/a"))
        .await
        .expect("dispatch should succeed");

    engine
        .update_domain_rate_limit("example.com", NonZeroU32::new(5).unwrap())
        .expect("rate limit should update");
    engine
        .update_domain_crawl_delay("example.com", Duration::from_millis(250))
        .expect("crawl delay should update");
    engine
        .cache_domain_robots(
            "example.com",
            RobotsPolicy {
                raw: "User-agent: *".to_string(),
                crawl_delay: Some(Duration::from_millis(250)),
            },
        )
        .await
        .expect("robots cache should update");

    let handle = engine
        .domain_handle("example.com")
        .expect("domain handle should exist");

    assert_eq!(handle.rate_limit().qps, NonZeroU32::new(5).unwrap());
    assert_eq!(
        handle.rate_limit().crawl_delay,
        Some(Duration::from_millis(250))
    );
    assert!(handle.robots().await.is_some());
}

#[tokio::test]
async fn engine_reports_backpressure_and_pull_decision() {
    let engine = SpiderEngine::new(EngineConfig::new(4, 1, 1, NonZeroU32::new(50).unwrap()));

    engine
        .dispatch(Request::get("https://example.com/a"))
        .await
        .expect("dispatch should succeed");

    let mut snapshot = engine.backpressure_snapshot();
    for _ in 0..10 {
        if snapshot.level == BackpressureLevel::Saturated {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        snapshot = engine.backpressure_snapshot();
    }

    assert_eq!(snapshot.global.capacity, 1);
    assert!(snapshot.global.queued == 1 || snapshot.domains[0].queue.queued == 1);
    assert_eq!(snapshot.level, BackpressureLevel::Saturated);
    assert_eq!(engine.should_pull_more(), PullDecision::Stop);
}
