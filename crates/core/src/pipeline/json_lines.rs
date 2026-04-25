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

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use serde_json::json;

    use super::*;

    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn json_lines_pipeline_writes_one_record_per_line() {
        let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
        let pipeline = JsonLinesPipeline::from_writer(SharedBuffer(buffer.clone()));

        pipeline
            .process(Item::new(BTreeMap::from([
                ("url".to_string(), json!("https://example.com/a")),
                ("status".to_string(), json!(200)),
            ])))
            .await
            .expect("first item should be written");

        pipeline
            .process(Item::new(BTreeMap::from([(
                "url".to_string(),
                json!("https://example.com/b"),
            )])))
            .await
            .expect("second item should be written");

        let written = buffer.lock().unwrap().clone();
        let text = String::from_utf8(written).expect("output should be utf-8");
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with('{') && lines[0].ends_with('}'));
        assert!(lines[0].contains("\"url\":\"https://example.com/a\""));
        assert!(lines[0].contains("\"status\":200"));
        assert!(lines[1].contains("\"url\":\"https://example.com/b\""));
    }
}
