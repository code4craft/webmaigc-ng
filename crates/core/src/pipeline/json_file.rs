use std::path::PathBuf;

use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::{mpsc, oneshot},
};

use crate::{BoxFuture, Item, Pipeline, SpiderError, SpiderStage};

struct WriteCommand {
    line: String,
    ack: oneshot::Sender<Result<(), SpiderError>>,
}

/// Pipeline that appends one JSON object per line into a local file.
///
/// Real disk I/O is owned by a dedicated background task so multiple workers can enqueue
/// writes without interleaving file contents.
pub struct JsonFilePipeline {
    tx: mpsc::Sender<WriteCommand>,
}

impl JsonFilePipeline {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, SpiderError> {
        Self::with_buffer(path, 1024)
    }

    pub fn with_buffer(path: impl Into<PathBuf>, buffer: usize) -> Result<Self, SpiderError> {
        if buffer == 0 {
            return Err(SpiderError::new(
                SpiderStage::Build,
                "json file pipeline buffer must be greater than zero",
            ));
        }

        let path = path.into();
        let (tx, mut rx) = mpsc::channel::<WriteCommand>(buffer);

        tokio::spawn(async move {
            let mut file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
            {
                Ok(file) => file,
                Err(err) => {
                    let message = format!("failed to open json file pipeline sink: {err}");
                    while let Some(command) = rx.recv().await {
                        let _ = command.ack.send(Err(SpiderError::new(
                            SpiderStage::Pipeline,
                            message.clone(),
                        )));
                    }
                    return;
                }
            };

            while let Some(command) = rx.recv().await {
                let result = async {
                    file.write_all(command.line.as_bytes()).await?;
                    file.write_all(b"\n").await?;
                    file.flush().await
                }
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()));

                let _ = command.ack.send(result);
            }
        });

        Ok(Self { tx })
    }
}

impl Pipeline for JsonFilePipeline {
    type Error = SpiderError;

    fn process<'a>(&'a self, item: &'a Item) -> BoxFuture<'a, Result<(), Self::Error>> {
        Box::pin(async move {
            let line = serde_json::to_string(item)
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;
            let (ack_tx, ack_rx) = oneshot::channel();

            self.tx
                .send(WriteCommand { line, ack: ack_tx })
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?;

            ack_rx
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Pipeline, err.to_string()))?
        })
    }
}
