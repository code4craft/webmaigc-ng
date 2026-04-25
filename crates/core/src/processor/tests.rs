use serde_json::json;

use super::*;
use crate::{HeaderMap, Page, Request};

#[test]
fn html_link_processor_extracts_http_links_and_resolves_relative_urls() {
    let request = Request::get("https://example.com/news");
    let page = Page {
        request,
        final_url: "https://example.com/news".to_string(),
        status_code: 200,
        headers: HeaderMap::new(),
        body: br##"
            <html>
              <body>
                <a href="/about">About</a>
                <a href="https://example.com/about#team">About Again</a>
                <a href="sub/page">Subpage</a>
                <a href="mailto:test@example.com">Mail</a>
                <a href="javascript:void(0)">JS</a>
                <a href="#fragment">Fragment</a>
              </body>
            </html>
        "##
        .to_vec(),
    };

    let result = HtmlLinkPageProcessor::default()
        .process(page)
        .expect("processor should succeed");

    let urls = result
        .requests
        .into_iter()
        .map(|request| request.url)
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        vec![
            "https://example.com/about".to_string(),
            "https://example.com/sub/page".to_string(),
        ]
    );
    assert_eq!(
        result.items[0].fields.get("links_discovered"),
        Some(&json!(2))
    );
}
