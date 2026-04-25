use std::collections::{BTreeMap, BTreeSet};

use reqwest::Url;
use scraper::{Html, Selector};
use serde_json::json;

use crate::{Item, Page, PageProcessor, ProcessResult, Request, SpiderError, SpiderStage};

/// Baseline HTML processor that emits page metadata and discovers linked pages.
///
/// It resolves relative `href` values against `Page::final_url`, keeps only same-host
/// HTTP(S) targets, strips fragments, and deduplicates links found within the same page
/// before handing them back to the scheduler for global deduplication.
pub struct HtmlLinkPageProcessor {
    href_selector: Selector,
}

impl HtmlLinkPageProcessor {
    pub fn new() -> Result<Self, SpiderError> {
        let href_selector = Selector::parse("a[href], link[href]").map_err(|err| {
            SpiderError::new(
                SpiderStage::Process,
                format!("failed to compile href selector: {err}"),
            )
        })?;

        Ok(Self { href_selector })
    }
}

impl Default for HtmlLinkPageProcessor {
    fn default() -> Self {
        Self::new().expect("static selector should compile")
    }
}

impl PageProcessor for HtmlLinkPageProcessor {
    type Error = SpiderError;

    fn process(&self, page: Page) -> Result<ProcessResult, Self::Error> {
        let Page {
            request,
            final_url,
            status_code,
            headers: _,
            body,
        } = page;

        let base_url = Url::parse(&final_url).map_err(|err| {
            SpiderError::new(
                SpiderStage::Process,
                format!("page final_url is not a valid url: {err}"),
            )
        })?;
        let base_host = base_url
            .host_str()
            .ok_or_else(|| SpiderError::new(SpiderStage::Process, "page final_url has no host"))?;

        let body_str = String::from_utf8_lossy(&body);
        let document = Html::parse_document(&body_str);
        let mut links: BTreeSet<String> = BTreeSet::new();
        for element in document.select(&self.href_selector) {
            let Some(raw_href) = element.value().attr("href") else {
                continue;
            };
            let href = raw_href.trim();
            if href.is_empty()
                || href.starts_with('#')
                || href.starts_with("javascript:")
                || href.starts_with("mailto:")
                || href.starts_with("tel:")
            {
                continue;
            }

            let Ok(mut resolved) = base_url.join(href) else {
                continue;
            };
            if !matches!(resolved.scheme(), "http" | "https") {
                continue;
            }
            if resolved.host_str() != Some(base_host) {
                continue;
            }
            resolved.set_fragment(None);
            links.insert(resolved.into());
        }

        let mut fields = BTreeMap::new();
        fields.insert("url".to_string(), json!(request.url));
        fields.insert("final_url".to_string(), json!(final_url));
        fields.insert("status".to_string(), json!(status_code));
        fields.insert("body_bytes".to_string(), json!(body.len()));
        fields.insert("links_discovered".to_string(), json!(links.len()));

        Ok(ProcessResult {
            items: vec![Item::new(fields)],
            requests: links.into_iter().map(Request::get).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HeaderMap;

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
}
