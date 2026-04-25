use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpiderError {
    pub stage: SpiderStage,
    pub message: String,
}

impl SpiderError {
    pub fn new(stage: SpiderStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }
}

impl fmt::Display for SpiderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.stage, self.message)
    }
}

impl Error for SpiderError {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpiderStage {
    Build,
    Download,
    Process,
    Schedule,
    Pipeline,
}
