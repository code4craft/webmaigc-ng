use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use webmagic_core::{
    DefaultDownloader, DefaultDownloaderConfig, DynPipeline, EngineConfig, JsonFilePipeline,
    JsonLinesPipeline, Request, SmartPageProcessor, SpiderBuilder,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Default, PartialEq, Eq)]
struct CliOptions {
    seeds: Vec<Request>,
    help_requested: bool,
    max_pages_per_site: Option<usize>,
    jsonl_out: Option<PathBuf>,
    stdout_too: bool,
    quiet: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let options = parse_args(std::env::args().skip(1))?;

    if options.help_requested || options.seeds.is_empty() {
        eprintln!("webmagic-cli — quick-start crawler");
        eprintln!();
        eprintln!("usage: webmagic-cli [OPTIONS] URL [URL ...]");
        eprintln!();
        eprintln!("Each seed URL is fetched and same-site links are discovered recursively.");
        eprintln!("By default, items are written as JSON Lines to stdout.");
        eprintln!("Progress and errors are written to stderr.");
        eprintln!();
        eprintln!("options:");
        eprintln!(
            "  --max-pages-per-site N  Stop scheduling new same-site pages after N accepted pages"
        );
        eprintln!(
            "  --jsonl-out PATH        Append crawl items to a local .jsonl file instead of stdout"
        );
        eprintln!(
            "  --stdout-too           Keep writing JSON Lines to stdout when --jsonl-out is enabled"
        );
        eprintln!("  --quiet                Suppress start/done progress logs on stderr");
        eprintln!("  -h, --help              Show this help");
        eprintln!();
        eprintln!("example:");
        eprintln!(
            "  cargo run -p webmagic-cli -- --max-pages-per-site 10 --jsonl-out data/fifa-news.jsonl https://www.fifa.com/en/news"
        );
        std::process::exit(if options.help_requested { 0 } else { 2 });
    }

    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default())
        .map_err(|err| anyhow!("downloader init failed: {err}"))?;
    let mut builder = SpiderBuilder::new()
        .downloader(Arc::new(downloader))
        .page_processor(Arc::new(SmartPageProcessor::default()));

    if let Some(max_pages_per_site) = options.max_pages_per_site {
        builder = builder
            .engine_config(EngineConfig::default().with_max_pages_per_site(max_pages_per_site));
    }

    let mut pipelines: Vec<Arc<DynPipeline>> = Vec::new();
    match options.jsonl_out {
        Some(path) => {
            pipelines
                .push(Arc::new(JsonFilePipeline::new(path).map_err(|err| {
                    anyhow!("json file pipeline init failed: {err}")
                })?));
            if options.stdout_too {
                pipelines.push(Arc::new(JsonLinesPipeline::stdout()));
            }
        }
        None => pipelines.push(Arc::new(JsonLinesPipeline::stdout())),
    }

    let spider = builder
        .pipelines(pipelines)
        .build()
        .map_err(|err| anyhow!("spider build failed: {err}"))?;

    if !options.quiet {
        eprintln!(
            "webmagic-cli: starting with {} seed(s)",
            options.seeds.len()
        );
    }
    let report = spider
        .run(options.seeds)
        .await
        .map_err(|err| anyhow!("spider run failed: {err}"))?;
    if !options.quiet {
        eprintln!(
            "webmagic-cli: done processed={} items={} discovered={} errors={}",
            report.processed, report.items, report.discovered, report.errors
        );
    }

    if report.errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliOptions> {
    let mut options = CliOptions::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => options.help_requested = true,
            "--max-pages-per-site" => {
                let raw = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --max-pages-per-site"))?;
                let value = raw
                    .parse::<usize>()
                    .map_err(|err| anyhow!("invalid --max-pages-per-site value `{raw}`: {err}"))?;
                if value == 0 {
                    return Err(anyhow!("--max-pages-per-site must be greater than zero"));
                }
                options.max_pages_per_site = Some(value);
            }
            "--jsonl-out" => {
                let raw = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --jsonl-out"))?;
                options.jsonl_out = Some(PathBuf::from(raw));
            }
            "--stdout-too" => options.stdout_too = true,
            "--quiet" => options.quiet = true,
            other if other.starts_with('-') => {
                return Err(anyhow!("unknown argument: {other}"));
            }
            other => options.seeds.push(Request::get(other.to_string())),
        }
    }

    Ok(options)
}
