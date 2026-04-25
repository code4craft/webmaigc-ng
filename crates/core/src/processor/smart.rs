use std::collections::BTreeSet;

use crate::{
    HtmlLinkPageProcessor, Page, PageProcessor, ProcessResult, ScriptDataPageProcessor, SpiderError,
};

/// Composite processor that merges classic anchor extraction with script-data extraction.
pub struct SmartPageProcessor {
    html: HtmlLinkPageProcessor,
    script: ScriptDataPageProcessor,
}

impl SmartPageProcessor {
    pub fn new() -> Result<Self, SpiderError> {
        Ok(Self {
            html: HtmlLinkPageProcessor::new()?,
            script: ScriptDataPageProcessor::new()?,
        })
    }
}

impl Default for SmartPageProcessor {
    fn default() -> Self {
        Self::new().expect("default smart processor should build")
    }
}

impl PageProcessor for SmartPageProcessor {
    type Error = SpiderError;

    fn process(&self, page: Page) -> Result<ProcessResult, Self::Error> {
        let html_result = self.html.process(page.clone())?;
        let script_result = self.script.process(page)?;

        let mut seen = BTreeSet::new();
        let mut requests = Vec::new();
        for request in html_result
            .requests
            .into_iter()
            .chain(script_result.requests)
        {
            if seen.insert(request.url.clone()) {
                requests.push(request);
            }
        }

        Ok(ProcessResult {
            items: html_result.items,
            requests,
        })
    }
}
