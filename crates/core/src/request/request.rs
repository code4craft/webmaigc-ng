use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{HeaderMap, SpiderError, SpiderStage};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
    pub labels: BTreeMap<String, String>,
}

impl Request {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: RequestMethod::Get,
            headers: HeaderMap::new(),
            body: None,
            labels: BTreeMap::new(),
        }
    }

    pub fn domain_key(&self) -> Result<String, SpiderError> {
        extract_host(&self.url)
            .ok_or_else(|| SpiderError::new(SpiderStage::Schedule, "request url has no host"))
    }
}

fn extract_host(url: &str) -> Option<String> {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host_port = without_scheme.split('/').next()?;
    let host = host_port.split('@').next_back()?;
    let host = if host.starts_with('[') {
        host.split(']')
            .next()
            .map(|segment| format!("{segment}]"))?
    } else {
        host.split(':').next()?.to_string()
    };

    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
}
