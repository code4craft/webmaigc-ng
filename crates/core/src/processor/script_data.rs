use std::collections::BTreeSet;

use regex::Regex;
use reqwest::Url;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::{Page, PageProcessor, ProcessResult, Request, SpiderError, SpiderStage};

/// Extract same-site page URLs from inline script data.
///
/// This is intended for SPA shell pages that expose routes or related URLs in JSON blobs
/// or JS state assignments instead of rendered anchor tags.
pub struct ScriptDataPageProcessor {
    script_selector: Selector,
    quoted_url_pattern: Regex,
}

impl ScriptDataPageProcessor {
    pub fn new() -> Result<Self, SpiderError> {
        let script_selector = Selector::parse("script:not([src])").map_err(|err| {
            SpiderError::new(
                SpiderStage::Process,
                format!("failed to compile script selector: {err}"),
            )
        })?;
        let quoted_url_pattern = Regex::new(r#"["'](?P<url>https?://[^"'\\\s]+|/[^"'\\\s]+)["']"#)
            .map_err(|err| {
                SpiderError::new(
                    SpiderStage::Process,
                    format!("failed to compile script url pattern: {err}"),
                )
            })?;

        Ok(Self {
            script_selector,
            quoted_url_pattern,
        })
    }
}

impl Default for ScriptDataPageProcessor {
    fn default() -> Self {
        Self::new().expect("static script selector and regex should compile")
    }
}

impl PageProcessor for ScriptDataPageProcessor {
    type Error = SpiderError;

    fn process(&self, page: Page) -> Result<ProcessResult, Self::Error> {
        let base_url = Url::parse(&page.final_url).map_err(|err| {
            SpiderError::new(
                SpiderStage::Process,
                format!("page final_url is not a valid url: {err}"),
            )
        })?;
        let base_host = base_url
            .host_str()
            .ok_or_else(|| SpiderError::new(SpiderStage::Process, "page final_url has no host"))?;

        let body_str = String::from_utf8_lossy(&page.body);
        let document = Html::parse_document(&body_str);
        let mut links = BTreeSet::new();

        for element in document.select(&self.script_selector) {
            let content = element.text().collect::<String>();
            if content.trim().is_empty() {
                continue;
            }

            if let Ok(value) = serde_json::from_str::<Value>(&content) {
                collect_urls_from_json(&value, &base_url, base_host, &mut links);
            }

            for captures in self.quoted_url_pattern.captures_iter(&content) {
                let Some(raw) = captures.name("url") else {
                    continue;
                };
                if let Some(url) = normalize_candidate(raw.as_str(), &base_url, base_host) {
                    links.insert(url);
                }
            }
        }

        Ok(ProcessResult {
            items: Vec::new(),
            requests: links.into_iter().map(Request::get).collect(),
        })
    }
}

fn collect_urls_from_json(
    value: &Value,
    base_url: &Url,
    base_host: &str,
    links: &mut BTreeSet<String>,
) {
    match value {
        Value::String(raw) => {
            if let Some(url) = normalize_candidate(raw, base_url, base_host) {
                links.insert(url);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_urls_from_json(item, base_url, base_host, links);
            }
        }
        Value::Object(fields) => {
            for value in fields.values() {
                collect_urls_from_json(value, base_url, base_host, links);
            }
        }
        _ => {}
    }
}

fn normalize_candidate(raw: &str, base_url: &Url, base_host: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.starts_with('#')
        || raw.starts_with("javascript:")
        || raw.starts_with("mailto:")
        || raw.starts_with("tel:")
    {
        return None;
    }

    let mut resolved = if raw.starts_with("http://") || raw.starts_with("https://") {
        Url::parse(raw).ok()?
    } else if raw.starts_with('/') {
        base_url.join(raw).ok()?
    } else {
        return None;
    };

    if resolved.host_str() != Some(base_host) {
        return None;
    }
    if !matches!(resolved.scheme(), "http" | "https") {
        return None;
    }

    resolved.set_fragment(None);
    if !looks_like_html_page(&resolved) {
        return None;
    }

    Some(resolved.into())
}

fn looks_like_html_page(url: &Url) -> bool {
    let path = url.path().to_ascii_lowercase();
    let Some(last_segment) = path.rsplit('/').next() else {
        return true;
    };

    if !last_segment.contains('.') {
        return true;
    }

    !matches!(
        last_segment.rsplit('.').next(),
        Some(
            "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "svg"
                | "webp"
                | "ico"
                | "css"
                | "js"
                | "mjs"
                | "json"
                | "xml"
                | "txt"
                | "pdf"
                | "zip"
                | "gz"
                | "woff"
                | "woff2"
                | "ttf"
                | "eot"
                | "mp4"
                | "webm"
                | "mp3"
                | "webmanifest"
        )
    )
}
