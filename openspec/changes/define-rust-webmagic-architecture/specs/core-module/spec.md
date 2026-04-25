## MODIFIED Requirements

### Requirement: Core module owns the shared crawler project contract
`crates/core` 模块 SHALL 同时承载共享的 crawler project contract 与 Spider 运行时核心抽象，不再只限于项目清单模型。

#### Scenario: 在同一核心模块中组合项目模型与 Spider 抽象
- **WHEN** CLI、server 或 worker 需要构建 Spider 或读取项目定义
- **THEN** 它们从 `crates/core` 获取 crawler project model、核心 Trait 和共享 Spider contract，而不是在不同模块中重复定义

### Requirement: Core module defines manifest and runtime contract separately
`crates/core` 模块 SHALL 保留 crawler project 的 manifest 与 runtime contract 二层结构，并允许 SpiderBuilder 基于该结构装配运行时组件。

#### Scenario: 基于项目定义装配 Spider
- **WHEN** 某个运行时组件根据 crawler project 创建 Spider
- **THEN** manifest 提供项目身份和声明式元数据，runtime contract 提供运行入口与装配所需的执行约束

### Requirement: Core module exposes stable project version identity
`crates/core` 模块 SHALL 定义稳定的项目版本身份，并允许运行时使用该版本身份追踪 Spider 配置与执行实例。

#### Scenario: 通过版本标识关联 Spider 实例
- **WHEN** 某个 Spider 实例启动并产生日志、指标或任务状态
- **THEN** 系统可以通过共享核心定义中的版本标识回溯到对应项目版本

### Requirement: Core module defines preflight validation semantics
`crates/core` 模块 SHALL 保留静态校验与环境校验两类结果类型，并允许 Spider 装配前执行统一 preflight 检查。

#### Scenario: SpiderBuilder 在装配前执行 preflight
- **WHEN** 运行时准备根据 crawler project 构建 Spider
- **THEN** SpiderBuilder 可以基于核心模块定义的校验结果判断是否允许继续装配

### Requirement: Core module remains runtime-agnostic
`crates/core` 模块 SHALL 保持 runtime-agnostic，但可以定义 Downloader、PageProcessor、Scheduler、Pipeline 与 SpiderBuilder 所需的共享 Trait、结果类型和上下文对象。

#### Scenario: 核心模块暴露共享 Trait 而不绑定基础设施
- **WHEN** 应用模块依赖 `crates/core`
- **THEN** 它们可以复用核心 Trait 与共享 contract，而不会被迫引入 HTTP 框架、数据库驱动或消息队列实现

## ADDED Requirements

### Requirement: Core module defines four primary spider traits
`crates/core` 模块 SHALL 定义 `Downloader`、`PageProcessor`、`Scheduler`、`Pipeline` 四个核心 Trait，作为 Spider 运行时的主扩展点。

#### Scenario: 通过 Trait 组装 Spider
- **WHEN** 应用需要创建一个新的 Spider
- **THEN** 它可以通过实现或组合这四个 Trait 来定义下载、解析、调度与数据输出行为

### Requirement: PageProcessor is a stateless processing contract
`PageProcessor` SHALL 被定义为无状态页面处理接口，输入 `Page`，输出包含 `Items` 与新发现 `Requests` 的处理结果。

#### Scenario: 处理页面并产生结果
- **WHEN** Spider 把某个页面交给 `PageProcessor`
- **THEN** 处理结果同时包含结构化数据和后续待抓取请求集合

### Requirement: Scheduler is a facade over deduplication and queueing
`Scheduler` SHALL 对上层提供统一调度门面，并在内部封装 URL 去重与队列管理能力。

#### Scenario: 调度新发现的请求
- **WHEN** `PageProcessor` 产生新请求集合
- **THEN** Spider 通过 `Scheduler` 提交请求，而不是分别调用去重器和队列管理器

### Requirement: SpiderBuilder assembles spiders across deployment modes
`SpiderBuilder` SHALL 作为统一装配入口，将共享项目定义与核心 Trait 组合为可在 CLI 与 Server 两种模式下运行的 Spider。

#### Scenario: 复用同一装配逻辑运行不同部署形态
- **WHEN** 应用在本地单机模式与分布式模式下构建 Spider
- **THEN** 它们复用同一套 SpiderBuilder 装配逻辑，只替换具体运行时组件实现

