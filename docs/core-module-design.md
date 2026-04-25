# Core Module Design

## Context

`crates/core` 是 webmagic-ng 双模式架构的共享底座。它必须同时服务 Quick Start 本地链路、Server 发布链路和 Worker 执行链路，因此只能承载稳定、可复用、与运行时无关的核心 contract。

当前最关键的职责是定义 Spider 运行时共享 contract，也就是“请求、页面、处理结果、组件接口和引擎装配方式如何被不同运行时一致解释”。

## Goals

- 定义 Spider 运行时共享数据结构和字段边界。
- 定义本地运行与服务端执行共享的运行时 contract。
- 保持组件接口与引擎装配方式简洁稳定。
- 保持 `crates/core` 轻量、稳定、可复用。

## Non-Goals

- 不定义 HTTP API、数据库 schema 或前端协议。
- 不实现浏览器执行逻辑、任务调度或日志存储。
- 不让 `core` 直接依赖具体运行时框架或重型基础设施库。

## Core Decisions

### 1. core 只承载运行时最小 contract

`crates/core` 只定义共享领域模型、版本标识、校验结果、基础枚举和错误分类，不直接引入浏览器、HTTP、数据库或任务队列依赖。

这样做的原因是：

- 保持 workspace 依赖方向清晰。
- 避免 CLI、server、worker 因共享 crate 被迫耦合到不相关基础设施。

## Integration Guidance

- `apps/cli` 通过 `core` 复用请求、页面、结果对象与组件接口。
- `services/server` 当前只应消费最小运行时共享类型，不在 `core` 内承载项目管理语义。
- `services/worker` 通过 `core` 复用引擎装配和运行时共享对象。

任何跨模式共享的数据结构，优先先进入 `core`，再由其他模块消费；不要在各模块里复制定义。

## Current Module Surface

当前 `crates/core` 中，应按如下边界组织：

- `request`: `Request`、`Page`、`ProcessResult`、`Item` 等抓取流转对象。
- `dedup`: 去重接口与默认内存实现。
- `queue`: 请求队列接口与默认内存实现。
- `downloader` / `processor` / `scheduler` / `pipeline`: 四大核心 Trait 的独立组件模块。
- `spider::builder`: `SpiderBuilder` 与动态装配相关类型。
- `spider::engine`: 事件驱动引擎骨架，包括全局工作通道和 Domain Dispatcher 注册表。
- `spider::error`: `SpiderError` 与 `SpiderStage`。
- `spider::types`: `Spider`、`SpiderParts` 等聚合对象。
- `module`: 仓库级模块描述辅助类型。

依赖方向约束如下：

- `apps/cli` 只消费 `request` 中的共享 contract，不在 CLI 中重新定义 request 或 page 类型。
- `apps/cli` 和后续运行时实现通过 `downloader / processor / scheduler / pipeline` 注入组件，而不是绕开 shared core 自定义接口。
- `SpiderBuilder` 默认装配内存去重 + 内存队列；如果用户提供分布式去重器和队列实现，则切换为分布式调度门面。
- `services/server` 若未来需要项目定义或发布协议，应放在更上层模块，不直接塞回当前最小 core。
- `services/worker` 只读取运行时共享对象，不在 worker 私有定义执行期 contract。

当前四大 Trait 的职责边界如下：

- `Downloader`: 只负责网络 I/O，包括协议、代理、重试、压缩和连接复用。
- `PageProcessor`: 只负责把页面转换为结构化结果和新请求，不拥有调度与持久化。
- `Scheduler`: 统一封装去重和排队，对上层暴露单一调度门面，并返回逐请求调度反馈。
- `Pipeline`: 只负责结果落地，不参与页面解析与抓取顺序控制。

当前默认下载器的能力边界是：

- `downloader::DefaultDownloaderConfig` 用来表达默认下载器的工程基线，而不是把具体 HTTP 客户端实现细节散落到运行时各处。
- TLS 后端默认使用 `TlsBackend::Rustls`，保留 `NativeTls` 作为兼容型备选，而不是默认路径。
- 协议策略通过 `HttpProtocolPolicy` 表达，当前默认是 `PreferHttp2`，从 contract 层明确 HTTP/2 是默认能力的一部分。
- 连接池策略通过 `ConnectionPoolConfig` 表达，覆盖 `max_idle_per_host`、空闲超时、建连超时和请求超时。
- 连接复用与吞吐调优通过 `ThroughputProfile` 暴露为统一入口，当前提供：
  - `Conservative`
  - `Balanced`
  - `HighThroughput`
- `ConnectionPoolConfig` 现在还暴露 `tcp_keepalive`，用于高并发长连接场景的连接保活策略。
- `Http2Config` 作为默认下载器的协议调优 contract 暴露，允许上层统一表达 HTTP/2 keepalive 与窗口相关诉求。
- 响应解压能力通过 `CompressionConfig` 表达，当前默认启用 Brotli、Gzip、Deflate。
- DNS 解析策略通过 `DnsResolverMode` 表达，当前默认是 `AsyncHickory`，从边界上明确“异步 DNS 是默认下载器的标准能力”。
- 代理扩展通过 `ProxyConfig + ProxyMode` 表达，允许默认直连、固定代理和动态代理池三种模式。
- `DefaultDownloaderConfig::validate()` 用来保证默认下载器配置在进入具体 runtime 实现前已经满足基本约束。
- `DefaultDownloader` 当前已经提供最小可用实现，基于 `reqwest + rustls` 将 `Request` 下载成 `Page`。
- `ProxyProvider` 是默认下载器的代理扩展点：
  - `Direct` 模式直接使用基础 client
  - `Static` 模式在构建期绑定固定代理
  - `DynamicPool` 模式通过 `ProxyProvider` 按请求选择代理，并按代理地址缓存 client
- `DownloaderCapabilities` 会把当前 profile、连接复用预算、TCP keepalive 和代理模式显式暴露出来，方便 CLI、Server 或后续管理面读取运行配置。
- 当前默认下载器实现已经把连接池、TCP keepalive、压缩、DNS 和代理模式落到了 `reqwest` builder。
- `Http2Config` 中的细粒度字段目前先作为 core contract 保留；由于当前 `reqwest 0.12` builder 没有暴露对应调优接口，默认实现暂未将这些字段一一映射到底层。

当前共享运行时 contract 放在以下领域模块中：

- `request`: `Request`、`Page`、`Item`、`ProcessResult`
- `dedup`: `DuplicateRemover`、`MemoryDuplicateRemover`
- `queue`: `RequestQueue`、`MemoryRequestQueue`
- `spider::builder`: `SpiderBuilder`
- `spider::engine`: `SpiderEngine`、`EngineConfig`
- `spider::types`: `Spider`、`SpiderParts`
- `spider::error`: `SpiderError`、`SpiderStage`
- 顶层组件模块：`downloader`、`processor`、`scheduler`、`pipeline`

当前 `Scheduler` 的门面语义是：

- 输入一个请求批次
- 内部完成去重判断和入队动作
- 输出 `ScheduleBatchResult`
- 逐请求反馈以 `ScheduledRequest` 表达，其中明确区分：
  - `DedupOutcome`
  - `QueueOutcome`

当前 `SpiderBuilder` 的装配策略是：

- 默认：单机模式，使用 `MemoryDuplicateRemover + MemoryRequestQueue + DefaultScheduler`
- 分布式：用户注入 `DuplicateRemover + RequestQueue`，由 `DefaultScheduler` 组合成统一门面
- 自定义：用户直接注入自己的 `Scheduler`

当前事件驱动引擎骨架的任务流是：

- `SpiderEngine` 持有一个全局有界 `async-channel`
- 每个域名第一次出现时，在 `DomainDispatcherRegistry` 中注册一个独立 dispatcher
- 每个 dispatcher 持有自己的 `tokio::sync::mpsc` 队列
- dispatcher 从域名本地队列取出请求，再转发到全局 Worker 通道

当前 Domain Dispatcher 的控制面语义是：

- 每个域名持有独立的 `DomainRateLimit`
- `DomainRateLimit` 支持 `qps + crawl_delay`
- dispatcher 在每次转发前按照最小间隔 `sleep`，而不是忙等
- 每个域名还持有独立的 `RobotsPolicy` 缓存
- `SpiderEngine` 可以在运行时更新域名的 `qps`、`crawl_delay` 和 robots 缓存

当前多级反压语义是：

- `SpiderEngine` 维护全局工作队列深度和每个域名本地队列深度。
- `DomainDispatcherHandle` 可输出域名级 `DomainBackpressureSnapshot`，包含队列容量、当前堆积和压力等级。
- `SpiderEngine` 可输出 `EngineBackpressureSnapshot`，聚合全局队列和所有域名队列状态。
- 压力等级分为 `Healthy / Constrained / Saturated`，用于表达继续拉取、减速拉取和停止拉取三种上游决策。
- `SpiderEngine::should_pull_more()` 返回 `PullDecision`，供本地 seed 注入器或分布式 MQ/Kafka 消费器决定是否继续拉取。

这一步已经把任务流骨架、域名级控制面和多级反压反馈放进 core，后续任务继续补充下载层与分布式协议联动。

## Open Questions

- 如果未来重新引入项目定义层，应该放在 `core` 还是更上层 crate。
- SpiderBuilder 是否需要继续保持纯运行时装配，不承载任何发布或配置协议。
