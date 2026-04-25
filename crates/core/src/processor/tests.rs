use std::collections::BTreeSet;

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
                <a href="/favicon-16x16.png">Icon</a>
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

#[test]
fn script_data_processor_extracts_same_site_urls_from_inline_state() {
    let request = Request::get("https://example.com/news");
    let page = Page {
        request,
        final_url: "https://example.com/news".to_string(),
        status_code: 200,
        headers: HeaderMap::new(),
        body: br##"
            <html>
              <body>
                <script type="application/json">
                  {"items":["https://example.com/articles/1","/articles/2","https://cdn.example.com/logo.svg"]}
                </script>
                <script>
                  window.__STATE__ = {"next":"https://example.com/articles/3","asset":"/favicon.ico"};
                </script>
              </body>
            </html>
        "##
        .to_vec(),
    };

    let result = ScriptDataPageProcessor::default()
        .process(page)
        .expect("processor should succeed");
    let urls = result
        .requests
        .into_iter()
        .map(|request| request.url)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        urls,
        BTreeSet::from([
            "https://example.com/articles/1".to_string(),
            "https://example.com/articles/2".to_string(),
            "https://example.com/articles/3".to_string(),
        ])
    );
}

#[test]
fn smart_page_processor_merges_html_and_script_links() {
    let request = Request::get("https://example.com/news");
    let page = Page {
        request,
        final_url: "https://example.com/news".to_string(),
        status_code: 200,
        headers: HeaderMap::new(),
        body: br##"
            <html>
              <body>
                <a href="/articles/1">A1</a>
                <script>
                  window.__STATE__ = {"next":"https://example.com/articles/2","dup":"/articles/1"};
                </script>
              </body>
            </html>
        "##
        .to_vec(),
    };

    let result = SmartPageProcessor::default()
        .process(page)
        .expect("processor should succeed");
    let urls = result
        .requests
        .into_iter()
        .map(|request| request.url)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        urls,
        BTreeSet::from([
            "https://example.com/articles/1".to_string(),
            "https://example.com/articles/2".to_string(),
        ])
    );
    assert_eq!(result.items.len(), 1);
}
