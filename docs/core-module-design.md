# Core Module Design

## Context

`crates/core` 是 webmagic-ng 双模式架构的共享底座。它必须同时服务 Quick Start 本地链路、Server 发布链路和 Worker 执行链路，因此只能承载稳定、可复用、与运行时无关的核心 contract。

当前最关键的职责是定义 crawler project model，也就是“一个爬虫项目如何被描述、校验、版本化并被不同运行时一致解释”。

## Goals

- 定义 crawler project 的核心数据结构和字段边界。
- 定义本地运行与服务端发布共享的 project contract。
- 定义项目版本、静态校验和环境校验的统一语义。
- 保持 `crates/core` 轻量、稳定、可复用。

## Non-Goals

- 不定义 HTTP API、数据库 schema 或前端协议。
- 不实现浏览器执行逻辑、任务调度或日志存储。
- 不让 `core` 直接依赖具体运行时框架或重型基础设施库。

## Core Decisions

### 1. 使用“Project Manifest + Runtime Contract”二层结构

核心模块中的 crawler project 由两部分组成：

- Project Manifest：项目身份、目标站点元数据、输入参数、输出 schema、依赖声明。
- Runtime Contract：执行入口、环境前提、资源要求、执行前校验约束。

这样做的原因是：

- Manifest 负责表达“项目是什么”。
- Runtime Contract 负责表达“项目如何被执行”。
- 这种拆分能同时满足本地 CLI、服务端发布和 Worker 执行，不会把所有关切压进一个巨型结构体。

### 2. 版本身份独立于本地路径

项目版本必须由显式版本标识承载，不能依赖目录名、临时路径或进程内匿名引用。

这样做的原因是：

- Server 任务实例需要稳定绑定到具体项目版本。
- 日志、审计、回滚和重放都需要同一个可解析的版本身份。

### 3. 校验分为静态校验与环境校验

核心模块需要定义两类前置校验：

- 静态校验：检查字段完整性、入口定义、schema 合法性。
- 环境校验：检查运行时依赖、配置项和执行前条件是否满足。

这样做的原因是：

- 两类错误需要不同的修复路径。
- CLI 的 `validate` 和服务端 preflight 可以共享同一套结果类型。

### 4. core 只承载稳定 contract

`crates/core` 只定义共享领域模型、版本标识、校验结果、基础枚举和错误分类，不直接引入浏览器、HTTP、数据库或任务队列依赖。

这样做的原因是：

- 保持 workspace 依赖方向清晰。
- 避免 CLI、server、worker 因共享 crate 被迫耦合到不相关基础设施。

## Integration Guidance

- `apps/cli` 通过 `core` 读取和校验 crawler project。
- `services/server` 通过 `core` 解析发布请求中的项目定义，并为任务绑定项目版本。
- `services/worker` 通过 `core` 读取 runtime contract 并执行 preflight 校验。

任何跨模式共享的数据结构，优先先进入 `core`，再由其他模块消费；不要在各模块里复制定义。

## Current Module Surface

当前 `crates/core` 中，crawler project model 应按如下边界组织：

- `crawler::project`: `CrawlerProject`、`ProjectManifest`、`RuntimeContract`、`ProjectVersion` 等稳定 contract。
- `crawler::validation`: 静态校验、环境校验、错误分类和校验报告。
- `module`: 仓库级模块描述辅助类型。

依赖方向约束如下：

- `apps/cli` 只消费 `crawler::project` 和 `crawler::validation`，不在 CLI 中重新定义 manifest 或 version 类型。
- `services/server` 通过 `ProjectReference` 绑定项目版本，通过 `ValidationReport` 暴露发布前检查结果。
- `services/worker` 只读取 `RuntimeContract` 和环境校验结果，不在 worker 私有定义执行前 contract。

## Open Questions

- 首版 manifest 的主序列化格式采用 TOML、YAML 还是 JSON。
- 入口 contract 首版是否仅支持单入口。
- 环境校验是否需要区分 `fatal` 和 `warning`。
