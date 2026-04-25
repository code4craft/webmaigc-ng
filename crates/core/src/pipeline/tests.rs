use std::{
    collections::BTreeMap,
    io::{self, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::json;
use tokio::fs;

use super::*;
use crate::Item;

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
        .process(&Item::new(BTreeMap::from([
            ("url".to_string(), json!("https://example.com/a")),
            ("status".to_string(), json!(200)),
        ])))
        .await
        .expect("first item should be written");

    pipeline
        .process(&Item::new(BTreeMap::from([(
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

#[tokio::test]
async fn json_file_pipeline_appends_one_record_per_line() {
    let path = unique_path("json-file-pipeline");
    let pipeline = JsonFilePipeline::new(&path).expect("pipeline should build");
    let first = Item::new(BTreeMap::from([(
        "url".to_string(),
        json!("https://example.com/a"),
    )]));
    let second = Item::new(BTreeMap::from([(
        "url".to_string(),
        json!("https://example.com/b"),
    )]));

    pipeline
        .process(&first)
        .await
        .expect("first item should be written");
    pipeline
        .process(&second)
        .await
        .expect("second item should be written");

    let text = fs::read_to_string(&path)
        .await
        .expect("json lines file should exist");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"url\":\"https://example.com/a\""));
    assert!(lines[1].contains("\"url\":\"https://example.com/b\""));

    let _ = std::fs::remove_file(path);
}

fn unique_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}.jsonl"))
}
