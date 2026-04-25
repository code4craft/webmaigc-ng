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

当前共享运行时 contract 放在以下领域模块中：

- `request`: `Request`、`Page`、`Item`、`ProcessResult`
- `dedup`: `DuplicateRemover`、`MemoryDuplicateRemover`
- `queue`: `RequestQueue`、`MemoryRequestQueue`
- `spider::builder`: `SpiderBuilder`
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

## Open Questions

- 如果未来重新引入项目定义层，应该放在 `core` 还是更上层 crate。
- SpiderBuilder 是否需要继续保持纯运行时装配，不承载任何发布或配置协议。
