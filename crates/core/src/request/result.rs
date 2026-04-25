use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Request;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub fields: BTreeMap<String, Value>,
}

impl Item {
    pub fn new(fields: BTreeMap<String, Value>) -> Self {
        Self { fields }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessResult {
    pub items: Vec<Item>,
    pub requests: Vec<Request>,
}

impl ProcessResult {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            requests: Vec::new(),
        }
    }
}
