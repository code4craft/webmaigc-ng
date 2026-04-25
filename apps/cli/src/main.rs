use std::sync::Arc;

use anyhow::{anyhow, Result};
use webmagic_core::{
    DefaultDownloader, DefaultDownloaderConfig, HtmlLinkPageProcessor, JsonLinesPipeline, Request,
    SpiderBuilder,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut seeds: Vec<Request> = Vec::new();
    let mut help_requested = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => help_requested = true,
            other => seeds.push(Request::get(other.to_string())),
        }
    }

    if help_requested || seeds.is_empty() {
        eprintln!("webmagic-cli — quick-start crawler");
        eprintln!();
        eprintln!("usage: webmagic-cli URL [URL ...]");
        eprintln!();
        eprintln!("Each seed URL is fetched and same-site links are discovered recursively.");
        eprintln!("Items (one JSON object per fetched page) are written to stdout.");
        eprintln!("Progress and errors are written to stderr.");
        std::process::exit(if help_requested { 0 } else { 2 });
    }

    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .map_err(|err| anyhow!("downloader init failed: {err}"))?;

    let spider = SpiderBuilder::new()
        .downloader(Arc::new(downloader))
        .page_processor(Arc::new(HtmlLinkPageProcessor::default()))
        .pipeline(Arc::new(JsonLinesPipeline::stdout()))
        .build()
        .map_err(|err| anyhow!("spider build failed: {err}"))?;

    eprintln!("webmagic-cli: starting with {} seed(s)", seeds.len());
    let report = spider
        .run(seeds)
        .await
        .map_err(|err| anyhow!("spider run failed: {err}"))?;
    eprintln!(
        "webmagic-cli: done processed={} items={} discovered={} errors={}",
        report.processed, report.items, report.discovered, report.errors
    );

    if report.errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use webmagic_core::{HeaderMap, Page, PageProcessor};

    #[test]
    fn html_link_page_processor_emits_metadata_item_and_discovered_links() {
        let request = Request::get("https://example.com/a");
        let page = Page {
            request: request.clone(),
            final_url: "https://example.com/a/redirect".to_string(),
            status_code: 200,
            headers: HeaderMap::new(),
            body: br#"<a href="/b">b</a><a href="https://example.com/c">c</a>"#.to_vec(),
        };

        let result = HtmlLinkPageProcessor::default()
            .process(page)
            .expect("process should succeed");
        assert_eq!(result.requests.len(), 2);
        assert_eq!(result.items.len(), 1);

        let fields = &result.items[0].fields;
        assert_eq!(fields.get("url"), Some(&json!("https://example.com/a")));
        assert_eq!(
            fields.get("final_url"),
            Some(&json!("https://example.com/a/redirect"))
        );
        assert_eq!(fields.get("status"), Some(&json!(200)));
        assert_eq!(fields.get("links_discovered"), Some(&json!(2)));
    }
}
