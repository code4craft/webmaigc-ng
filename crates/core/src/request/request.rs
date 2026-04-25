use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::HeaderMap;

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
