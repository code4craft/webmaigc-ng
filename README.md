# webmagic-ng

`webmagic-ng` 是一个 Rust workspace，目标是逐步演进为同时支持 Quick Start 和 Server 模式的双模式爬虫平台。

## Why webmagic-ng

当前这个仓库已经不是“只有骨架”的状态，核心价值主要在这几件事：

- 双模式共享同一套 runtime contract：CLI、后续 Server 和 Worker 复用 `crates/core`，避免各跑各的模型。
- 默认链路可直接跑通：`DefaultDownloader + SmartPageProcessor + JSON Lines pipeline` 已经形成最小可用闭环。
- 对部分前端页面有额外兜底：除了 HTML 锚点，也会尝试从内联脚本状态数据里补充同站点链接，但当前效果仍在持续打磨。
- 对页面和静态资源有基础区分：默认处理器会过滤明显不是页面的资源链接，降低误抓噪音。
- 调度侧已经有工程化基础：内置域名级 dispatcher、限速/`crawl-delay`、robots 缓存和多级反压快照，而不是单纯串行 demo。
- 输出链路适合 Unix 风格工作流：结果默认走 stdout，也支持 `.jsonl` 文件落盘以及“落盘同时继续保留 stdout 数据流”。
- 下载器配置边界已经明确：TLS、HTTP 协议策略、连接池、压缩、DNS、代理模式和吞吐 profile 都有稳定 contract，可继续往生产链路扩展。

## Toolchain

- 仓库当前锁定 Rust `1.93.0`，见 [rust-toolchain.toml](/Users/yihua/coding/code4craft/webmaigc-ng/rust-toolchain.toml)。
- 首次进入仓库前，建议先确认本机已安装对应工具链：`rustup toolchain install 1.93.0`
- 之后直接在仓库根目录运行 `cargo test`、`cargo fmt`、`cargo clippy`

## Development Rules

- 开发规范见 [docs/repo-conventions.md](/Users/yihua/coding/code4craft/webmaigc-ng/docs/repo-conventions.md)。
- CLI 使用说明见 [docs/cli-quick-start.md](/Users/yihua/coding/code4craft/webmaigc-ng/docs/cli-quick-start.md)。
- 面向 Agent/协作式开发的额外约束见 [AGENTS.md](/Users/yihua/coding/code4craft/webmaigc-ng/AGENTS.md)。
- 结构性实现优先走 `openspec`，不要跳过 spec 直接做大改。

## Workspace Layout

- `apps/cli`: 本地命令行入口，承接 Quick Start 模式。
- `crates/core`: 跨模式共享的领域模型、配置约束和通用能力。
- `services/server`: 服务端控制面，后续承接发布、任务、日志和鉴权 API。
- `services/worker`: 执行面，后续承接任务拉取、运行隔离和状态上报。
- `apps/admin`: Admin 入口预留目录，当前只承载文档和边界定义，后续可演进为 Web 客户端。
- `docs`: 架构和仓库规范文档。
- `openspec`: 需求、设计和实施任务。

## Current Status

当前仓库已完成的重点不再只是 workspace 骨架，而是已经具备一条可运行的本地抓取链路：

- `apps/cli` 可以直接接收 seed URL 并运行单站点抓取。
- `crates/core` 已经落下共享 request/page/item/process contract。
- 默认下载器基于 `reqwest + rustls`，具备最小可用真实抓取能力。
- 默认处理器 `SmartPageProcessor` 已覆盖 HTML 抽链，并带有基础的脚本状态数据补链能力。
- 默认调度链路已经切到事件驱动引擎，支持域名级限速、页数上限和自然收敛退出。
- 默认 pipeline 已支持 stdout JSON Lines 和并发安全的本地 `.jsonl` 文件输出。

如果要看更细的核心边界和运行模型，直接看 [docs/core-module-design.md](/Users/yihua/coding/code4craft/webmaigc-ng/docs/core-module-design.md)。

## CLI Quick Start

当前 `apps/cli` 已经可以直接跑一个最小可用的同站点爬虫：

- 默认下载器：`DefaultDownloader`
- 默认处理器：`SmartPageProcessor`（HTML 锚点为主，附带基础脚本状态数据补链）
- 默认输出：JSON Lines 到 stdout
- 支持限制单站点最多抓取页面数
- 支持直接把抓取结果落到本地 `.jsonl` 文件
- 支持“落盘同时继续保留 stdout 数据流”
- 支持静默进度日志

先看帮助：

```bash
cargo run -p webmagic-cli -- --help
```

抓取 `https://webmagic.io/`，并把结果写到磁盘：

```bash
mkdir -p data
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  https://webmagic.io/
```

运行完成后：

- 抓取数据在 `data/webmagic-home.jsonl`
- 运行进度和汇总信息在 stderr
- 默认处理器会优先从 HTML 页面里抽取同站点链接，并尽量避免把 `.png/.css/.woff2` 这类静态资源误判成页面

如果你希望“既写文件，又继续把数据流交给下游命令”，可以再加 `--stdout-too`：

```bash
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  --stdout-too \
  https://webmagic.io/ | head
```

如果你只关心结果文件，不想看启动/汇总日志，可以再加 `--quiet`：

```bash
cargo run -p webmagic-cli -- \
  --jsonl-out data/webmagic-home.jsonl \
  --quiet \
  https://webmagic.io/
```

查看前几行结果：

```bash
head -n 3 data/webmagic-home.jsonl
```

每一行都是一个 JSON 对象，当前默认会包含：

- `url`
- `final_url`
- `status`
- `body_bytes`
- `links_discovered`

## Roadmap

当前 roadmap 维持“四阶段递进式交付”，但结合现在的实现状态，可以更具体地看成下面这些里程碑：

### Phase 0: Foundation

已基本完成：

- Rust workspace、模块边界和仓库规范
- `crates/core` 共享 contract
- Spider builder / engine / scheduler / pipeline 基础骨架
- 默认下载器、默认处理器、默认 stdout/file pipeline

### Phase 1: Quick Start MVP

当前已完成大半，接下来重点补齐“更好用”而不只是“能跑”：

- 完善 CLI 参数和 profile 组合，减少手工装配成本
- 补更多真实站点 smoke test，覆盖前端壳页面和常见内容站
- 增强结果输出与调试体验，例如更清晰的汇总、失败原因和运行配置可见性
- 继续收敛默认处理器策略，提升页面识别和抽链质量

### Phase 2: Server MVP

打通从“本地试抓”到“服务端长期运行”的第一条闭环：

- 项目发布与版本管理
- 任务提交、定时调度和 Worker 执行
- 运行日志回查、状态查询和基础鉴权
- 复用 core runtime contract，避免 CLI 和服务端出现两套执行语义

### Phase 3: Admin & Production Hardening

面向真正可运维的生产能力补齐控制面和治理能力：

- Admin 控制台
- 监控告警、失败重试和运行诊断
- 更完整的部署文档、灰度和回滚方案
- 面向长期运行场景的稳定性、可观测性和治理能力补强
