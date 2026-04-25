use std::{
    io::{self, Stdout, Write},
    sync::Mutex,
};

use crate::{BoxFuture, Item, Pipeline, SpiderError, SpiderStage};

/// Pipeline that emits each `Item` as a single-line JSON record.
///
/// The default `stdout()` constructor binds the pipeline to the process stdout, which is the
/// quick-start CLI's data plane. Tests and embedders can supply any `Write + Send` sink via
/// `from_writer`.
pub struct JsonLinesPipeline {
    writer: Mutex<Box<dyn Write + Send>>,
}

impl JsonLinesPipeline {
    /// Bind the pipeline to the process stdout.
    pub fn stdout() -> Self {
        Self::from_writer(io::stdout())
    }

    /// Bind the pipeline to an arbitrary writer.
    pub fn from_writer<W>(writer: W) -> Self
    where
        W: Write + Send + 'static,
    {
        Self {
            writer: Mutex::new(Box::new(writer)),
        }
    }
}

impl Default for JsonLinesPipeline {
    fn default() -> Self {
        Self::stdout()
    }
}

impl Pipeline for JsonLinesPipeline {
    type Error = SpiderError;

    fn process(&self, item: Item) -> BoxFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let serialized = serde_json::to_string(&item)
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;
            let mut writer = self
                .writer
                .lock()
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;
            writeln!(writer, "{serialized}")
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;
            writer
                .flush()
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;
            Ok(())
        })
    }
}

/// `JsonLinesStdout` provides line-buffered access to `io::Stdout` for callers that want to
/// keep the type concrete (e.g. when storing the pipeline in a struct without a trait
/// object).
pub type JsonLinesStdout = Stdout;
