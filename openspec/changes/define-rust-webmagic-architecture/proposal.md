## Why

你已经有成熟的 Java 版 webmagic 经验，Rust 版不应该只是语法迁移，而应该借助 Trait、Tokio、Channel、rustls 和 tracing 生态，重构出一套真正适合高并发、低开销、双态部署的下一代爬虫架构。当前仓库已经有基础骨架，但缺少对“核心抽象、引擎控制流、网络层、双态部署、合规与观测”这一整套架构的统一定义。

## What Changes

- 定义 Rust 版 webmagic-ng 的核心架构哲学，明确以 Trait 抽象、事件驱动和反压控制作为一等设计原则。
- 修改 `core-module` 的要求，使其从单纯的 project contract 扩展为 Downloader、PageProcessor、Scheduler、Pipeline 四大核心 Trait 的共享边界。
- 新增事件驱动引擎规范，定义全局有界 Channel、按 Domain 派生 Dispatcher 协程、Worker 池和三级反压机制。
- 新增高性能网络层规范，定义基于 rustls、HTTP/2、异步 DNS、压缩解码和连接池优化的下载层要求。
- 新增双态部署规范，定义 Quick Start 本地 CLI 形态与 Server 分布式形态的架构边界和协作方式。
- 新增白帽协议支持与可观测性规范，覆盖 Robots、Sitemap、tracing、metrics 和集群调用链追踪。

## Capabilities

### New Capabilities
- `event-driven-engine`: 基于 Tokio 协程和有界 Channel 的事件驱动爬虫引擎，包括 Domain 级调度、限流和反压。
- `high-performance-network-layer`: 面向高吞吐抓取的下载层能力，包括 rustls、HTTP/2、多连接复用、压缩和异步 DNS。
- `dual-mode-deployment`: 同一套 SpiderBuilder 同时支持本地 CLI 与云原生分布式运行的部署模型。
- `crawler-compliance`: Robots.txt、Crawl-delay 和 Sitemap 的原生支持能力。
- `crawler-observability`: 面向单机和集群的 tracing、OpenTelemetry、metrics 和监控集成能力。

### Modified Capabilities
- `core-module`: 将核心模块扩展为 Rust 版 webmagic 的四大 Trait 抽象与共享 Spider contract，而不仅是 crawler project model。

## Impact

- 影响 `crates/core` 的模块边界和核心接口设计。
- 影响 `apps/cli`、`services/server`、`services/worker` 的职责划分与依赖关系。
- 影响后续下载器、调度器、Pipeline、SpiderBuilder、日志与指标实现方式。
- 影响未来 JS/TS 控制面与 Rust 数据面之间的协议设计。
