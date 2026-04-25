use serde_json::json;
use webmagic_core::{HeaderMap, HtmlLinkPageProcessor, Page, PageProcessor, Request};

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
