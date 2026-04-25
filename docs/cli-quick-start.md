# CLI Quick Start

`webmagic-cli` 是当前仓库里最直接的本地抓取入口，适合快速验证：

- seed URL 是否可抓
- `SmartPageProcessor` 是否能覆盖 HTML 锚点，并在部分页面上补充脚本状态数据里的同站点链接
- `max_pages_per_site` 是否能让抓取自然收敛
- 输出是否能直接落到本地 `.jsonl` 文件

## Help

```bash
cargo run -p webmagic-cli -- --help
```

当前支持的关键参数：

- `--max-pages-per-site N`：单站点最多接受多少页面
- `--jsonl-out PATH`：把抓取结果以 JSON Lines 追加写入本地文件
- `--stdout-too`：在启用 `--jsonl-out` 时，继续把 JSON Lines 同步写到 stdout
- `--quiet`：关闭启动和最终汇总这类 stderr 进度日志

## Example

抓取 `https://webmagic.io/`，并最终落到磁盘：

```bash
mkdir -p data
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  https://webmagic.io/
```

运行预期：

- stderr 会输出启动和最终汇总信息
- `data/webmagic-home.jsonl` 至少会包含 seed 页面本身的一行结果
- 默认处理器会优先抽取 HTML 页面里的同站点链接，并尽量避免把 `favicon`、字体、CSS 这类静态资源误判成页面链接

如果你希望“既写文件，又继续让 stdout 保持数据流”：

```bash
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  --stdout-too \
  https://webmagic.io/ | head
```

如果你只关心结果文件，不想看进度日志：

```bash
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  --quiet \
  https://webmagic.io/
```

查看结果：

```bash
wc -l data/webmagic-home.jsonl
head -n 5 data/webmagic-home.jsonl
```

当前每个 `Item` 默认包含这些字段：

- `url`：原始请求 URL
- `final_url`：最终落地 URL
- `status`：HTTP 状态码
- `body_bytes`：响应体字节数
- `links_discovered`：当前页面抽取出的同站点链接数量

## Notes

- CLI 默认使用 `SmartPageProcessor`，会先做 HTML 锚点抽链，并在部分页面上尝试脚本状态数据补链。
- 当前基础处理器只抽取页面锚点链接，不会把 `link rel="icon"`、`manifest`、字体、CSS 这类资源当成页面继续抓。
- 如果不传 `--jsonl-out`，结果会写到 stdout，适合接 `jq`、`head`、`tee`。
- 如果同时传 `--jsonl-out --stdout-too`，Spider 会同时挂载文件输出和 stdout 输出两个 pipeline。
- `--max-pages-per-site` 是按域名统计的 accepted 页面数，不是按最终写盘成功数统计。
