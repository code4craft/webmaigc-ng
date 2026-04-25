use std::{
    collections::BTreeMap,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use serde_json::json;
use webmagic_core::{
    BoxFuture, Downloader, EngineConfig, HeaderMap, Item, Page, PageProcessor, Pipeline,
    ProcessResult, Request, SpiderBuilder, SpiderError,
};

struct CountingDownloader {
    calls: AtomicUsize,
}

impl Downloader for CountingDownloader {
    type Error = SpiderError;

    fn download(&self, request: Request) -> BoxFuture<'_, Result<Page, Self::Error>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
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

struct LinkProcessor {
    seed_followups: Mutex<Option<Vec<Request>>>,
}

impl PageProcessor for LinkProcessor {
    type Error = SpiderError;

    fn process(&self, page: Page) -> Result<ProcessResult, Self::Error> {
        let mut item = BTreeMap::new();
        item.insert("url".to_string(), json!(page.request.url));

        let followups = if page.request.url == "https://example.com/seed" {
            self.seed_followups
                .lock()
                .unwrap()
                .take()
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(ProcessResult {
            items: vec![Item::new(item)],
            requests: followups,
        })
    }
}

struct CollectingPipeline {
    items: Mutex<Vec<Item>>,
}

impl Pipeline for CollectingPipeline {
    type Error = SpiderError;

    fn process(&self, item: Item) -> BoxFuture<'_, Result<(), Self::Error>> {
        self.items.lock().unwrap().push(item);
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn spider_run_processes_seeds_and_followups_to_completion() {
    let downloader = Arc::new(CountingDownloader {
        calls: AtomicUsize::new(0),
    });
    let processor = Arc::new(LinkProcessor {
        seed_followups: Mutex::new(Some(vec![
            Request::get("https://example.com/a"),
            Request::get("https://example.com/b"),
            Request::get("https://example.com/a"),
        ])),
    });
    let pipeline = Arc::new(CollectingPipeline {
        items: Mutex::new(Vec::new()),
    });

    let config = EngineConfig::new(4, 16, 16, NonZeroU32::new(100).unwrap());
    let spider = SpiderBuilder::new()
        .downloader(downloader.clone())
        .page_processor(processor.clone())
        .engine_config(config)
        .pipeline(pipeline.clone())
        .build()
        .expect("builder should succeed");

    let report = spider
        .run(vec![Request::get("https://example.com/seed")])
        .await
        .expect("spider should complete");

    assert_eq!(report.processed, 3);
    assert_eq!(report.items, 3);
    assert_eq!(report.discovered, 2);
    assert_eq!(report.errors, 0);

    assert_eq!(downloader.calls.load(Ordering::SeqCst), 3);
    assert_eq!(pipeline.items.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn spider_run_returns_immediately_when_no_seeds_accepted() {
    let downloader = Arc::new(CountingDownloader {
        calls: AtomicUsize::new(0),
    });
    let processor = Arc::new(LinkProcessor {
        seed_followups: Mutex::new(None),
    });
    let pipeline = Arc::new(CollectingPipeline {
        items: Mutex::new(Vec::new()),
    });

    let spider = SpiderBuilder::new()
        .downloader(downloader.clone())
        .page_processor(processor)
        .pipeline(pipeline)
        .build()
        .expect("builder should succeed");

    let report = spider.run(vec![]).await.expect("empty run should succeed");

    assert_eq!(report.processed, 0);
    assert_eq!(report.items, 0);
    assert_eq!(report.discovered, 0);
    assert_eq!(downloader.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn spider_run_stops_accepting_requests_after_site_page_limit() {
    let downloader = Arc::new(CountingDownloader {
        calls: AtomicUsize::new(0),
    });
    let processor = Arc::new(LinkProcessor {
        seed_followups: Mutex::new(Some(vec![
            Request::get("https://example.com/a"),
            Request::get("https://example.com/b"),
            Request::get("https://example.com/c"),
        ])),
    });
    let pipeline = Arc::new(CollectingPipeline {
        items: Mutex::new(Vec::new()),
    });

    let config =
        EngineConfig::new(4, 16, 16, NonZeroU32::new(100).unwrap()).with_max_pages_per_site(2);
    let spider = SpiderBuilder::new()
        .downloader(downloader.clone())
        .page_processor(processor)
        .engine_config(config)
        .pipeline(pipeline.clone())
        .build()
        .expect("builder should succeed");

    let report = spider
        .run(vec![Request::get("https://example.com/seed")])
        .await
        .expect("spider should complete");

    assert_eq!(report.processed, 2);
    assert_eq!(report.items, 2);
    assert_eq!(report.discovered, 1);
    assert_eq!(report.errors, 0);
    assert_eq!(downloader.calls.load(Ordering::SeqCst), 2);
    assert_eq!(pipeline.items.lock().unwrap().len(), 2);
}
