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

#### Scenario: 基础 HTML 处理器抽取页面链接
- **WHEN** 运行时使用框架内置的基础 HTML `PageProcessor`
- **THEN** 它解析页面中的 `href` 链接，解析相对 URL 为绝对 URL，过滤跨站与非 HTTP(S) 链接，并把结果作为新请求集合返回给 `Scheduler`

#### Scenario: 脚本状态处理器从内联数据中抽取页面链接
- **WHEN** 页面是 SPA 壳页面，链接主要存在于内联 `script` 的 JSON 或 JS 状态对象中
- **THEN** 框架内置的脚本数据 `PageProcessor` 会从这些内联数据里提取同站点页面 URL，并把结果作为新请求集合返回给 `Scheduler`

### Requirement: Scheduler is a facade over deduplication and queueing
`Scheduler` SHALL 对上层提供统一调度门面，并在内部封装 URL 去重与队列管理能力。

#### Scenario: 调度新发现的请求
- **WHEN** `PageProcessor` 产生新请求集合
- **THEN** Spider 通过 `Scheduler` 提交请求，而不是分别调用去重器和队列管理器

#### Scenario: 站点达到页面上限后停止继续接受请求
- **WHEN** 某个域名已经达到配置中的最大页面数
- **THEN** `Scheduler` 丢弃该域名后续的新请求，并保持 Spider 通过 in-flight 计数自然收敛退出

### Requirement: Pipeline must consume items as immutable shared inputs
`Pipeline` SHALL 以不可变借用方式接收 `Item`，避免 pipeline 对抓取结果进行回写，并允许 Spider 将同一个 `Item` 广播给多个 pipeline 共享处理。

#### Scenario: Spider 把一个 Item 广播给多个 Pipeline
- **WHEN** 某个页面处理结果包含一个或多个 `Item`
- **THEN** Spider 对每个 `Item` 并发调用所有已挂载 pipeline，并保持 pipeline 之间互不影响

#### Scenario: Pipeline 处理失败不污染原始 Item
- **WHEN** 某个 pipeline 在处理 `Item` 时失败
- **THEN** 失败只体现在该 pipeline 的返回结果与 Spider 错误统计中，不允许通过可变引用修改原始 `Item`

### Requirement: Core module must provide a baseline json-file pipeline
`crates/core` 模块 SHALL 提供 `JsonFilePipeline`，把 `Item` 以 JSON Lines 形式异步追加写入本地文件，作为本地调试与极简落盘的基线实现。

#### Scenario: 多个 worker 并发写同一个结果文件
- **WHEN** 多个 worker 几乎同时把不同 `Item` 交给同一个 `JsonFilePipeline`
- **THEN** pipeline 通过串行化的后台写入机制保证文件内容按行追加，不发生交错损坏

### Requirement: SpiderBuilder assembles spiders across deployment modes
`SpiderBuilder` SHALL 作为统一装配入口，将共享项目定义与核心 Trait 组合为可在 CLI 与 Server 两种模式下运行的 Spider。

#### Scenario: 复用同一装配逻辑运行不同部署形态
- **WHEN** 应用在本地单机模式与分布式模式下构建 Spider
- **THEN** 它们复用同一套 SpiderBuilder 装配逻辑，只替换具体运行时组件实现

#### Scenario: 组合多个页面处理策略
- **WHEN** 调用方希望同时覆盖传统 HTML 页面和 SPA 壳页面
- **THEN** 它可以装配一个组合型 `PageProcessor`，合并锚点抽链与脚本数据抽链结果
