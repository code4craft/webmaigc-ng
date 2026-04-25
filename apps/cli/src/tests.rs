use serde_json::json;
use std::path::PathBuf;

use webmagic_core::{HeaderMap, HtmlLinkPageProcessor, Page, PageProcessor, Request};

use crate::{parse_args, CliOptions};

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

#[test]
fn parse_args_supports_output_file_and_page_limit() {
    let options = parse_args([
        "--max-pages-per-site".to_string(),
        "10".to_string(),
        "--jsonl-out".to_string(),
        "data/fifa-news.jsonl".to_string(),
        "https://www.fifa.com/en/news".to_string(),
    ])
    .expect("args should parse");

    assert_eq!(
        options,
        CliOptions {
            seeds: vec![Request::get("https://www.fifa.com/en/news")],
            help_requested: false,
            max_pages_per_site: Some(10),
            jsonl_out: Some(PathBuf::from("data/fifa-news.jsonl")),
            stdout_too: false,
            quiet: false,
        }
    );
}

#[test]
fn parse_args_rejects_zero_page_limit() {
    let err = parse_args([
        "--max-pages-per-site".to_string(),
        "0".to_string(),
        "https://www.fifa.com/en/news".to_string(),
    ])
    .expect_err("zero page limit should be rejected");

    assert!(err.to_string().contains("greater than zero"));
}

#[test]
fn parse_args_supports_stdout_too_and_quiet() {
    let options = parse_args([
        "--jsonl-out".to_string(),
        "data/fifa-news.jsonl".to_string(),
        "--stdout-too".to_string(),
        "--quiet".to_string(),
        "https://www.fifa.com/en/news".to_string(),
    ])
    .expect("args should parse");

    assert_eq!(
        options,
        CliOptions {
            seeds: vec![Request::get("https://www.fifa.com/en/news")],
            help_requested: false,
            max_pages_per_site: None,
            jsonl_out: Some(PathBuf::from("data/fifa-news.jsonl")),
            stdout_too: true,
            quiet: true,
        }
    );
}
