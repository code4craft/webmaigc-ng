use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use webmagic_core::{
    BoxFuture, DefaultDownloader, DefaultDownloaderConfig, Downloader, EngineConfig,
    HtmlLinkPageProcessor, Item, PageProcessor, Pipeline, Request, ScriptDataPageProcessor,
    SmartPageProcessor, SpiderBuilder, SpiderError,
};

#[tokio::test]
async fn real_fetch_fifa_homepage_over_https() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .expect("default downloader should build");

    let page = downloader
        .download(Request::get("https://www.fifa.com/"))
        .await
        .expect("default downloader should fetch fifa homepage");
    assert_eq!(page.request.url, "https://www.fifa.com/");
    assert!((200..400).contains(&page.status_code));
    assert!(page.final_url.starts_with("https://"));
    assert!(!page.body.is_empty());

    let body = String::from_utf8_lossy(&page.body).to_lowercase();
    assert!(body.contains("fifa"));
}

struct CollectingPipeline {
    items: Mutex<Vec<Item>>,
}

impl Pipeline for CollectingPipeline {
    type Error = SpiderError;

    fn process<'a>(&'a self, item: &'a Item) -> BoxFuture<'a, Result<(), Self::Error>> {
        self.items.lock().unwrap().push(item.clone());
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn real_fetch_fifa_news_spider_does_not_treat_static_assets_as_pages() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .expect("default downloader should build");
    let pipeline = Arc::new(CollectingPipeline {
        items: Mutex::new(Vec::new()),
    });
    let config =
        EngineConfig::new(4, 32, 64, NonZeroU32::new(20).unwrap()).with_max_pages_per_site(10);

    let spider = SpiderBuilder::new()
        .downloader(Arc::new(downloader))
        .page_processor(Arc::new(HtmlLinkPageProcessor::default()))
        .engine_config(config)
        .pipeline(pipeline.clone())
        .build()
        .expect("spider should build");

    let report = spider
        .run(vec![Request::get("https://www.fifa.com/en/news")])
        .await
        .expect("spider should complete");

    assert_eq!(report.processed, 1);
    assert_eq!(report.items, 1);
    assert_eq!(report.errors, 0);
    assert_eq!(pipeline.items.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn real_fetch_fifa_news_script_data_processor_extracts_same_site_page_url() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .expect("default downloader should build");

    let page = downloader
        .download(Request::get("https://www.fifa.com/en/news"))
        .await
        .expect("default downloader should fetch fifa news page");

    let result = ScriptDataPageProcessor::default()
        .process(page)
        .expect("script data processor should succeed");
    let urls = result
        .requests
        .into_iter()
        .map(|request| request.url)
        .collect::<Vec<_>>();

    assert!(urls.contains(&"https://www.fifa.com/auth".to_string()));
}

#[tokio::test]
async fn real_fetch_fifa_news_smart_processor_discovers_more_than_plain_html() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .expect("default downloader should build");

    let page = downloader
        .download(Request::get("https://www.fifa.com/en/news"))
        .await
        .expect("default downloader should fetch fifa news page");

    let html_only = HtmlLinkPageProcessor::default()
        .process(page.clone())
        .expect("html processor should succeed");
    let smart = SmartPageProcessor::default()
        .process(page)
        .expect("smart processor should succeed");

    assert!(html_only.requests.is_empty());
    assert!(!smart.requests.is_empty());
}
