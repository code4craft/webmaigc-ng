use std::sync::Arc;

use crate::{DynDownloader, DynPageProcessor, DynPipeline, DynScheduler};

#[derive(Clone)]
pub struct SpiderParts {
    pub downloader: Arc<DynDownloader>,
    pub processor: Arc<DynPageProcessor>,
    pub scheduler: Arc<DynScheduler>,
    pub pipeline: Arc<DynPipeline>,
}

#[derive(Clone)]
pub struct Spider {
    pub parts: SpiderParts,
}
