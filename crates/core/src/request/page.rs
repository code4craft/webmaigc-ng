use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Request;

pub type HeaderMap = BTreeMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Page {
    pub request: Request,
    pub final_url: String,
    pub status_code: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}
