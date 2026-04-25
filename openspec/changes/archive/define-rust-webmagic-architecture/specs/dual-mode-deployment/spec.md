## ADDED Requirements

### Requirement: SpiderBuilder must support quick-start local mode
SpiderBuilder SHALL 支持零外部依赖的本地 CLI 运行形态，并允许使用内存去重器、engine-backed scheduler 和标准输出管道进行抓取。

#### Scenario: 以 Unix 管道方式运行本地抓取
- **WHEN** 用户通过 CLI 在本地启动 Spider
- **THEN** 抓取结果输出到 stdout，日志输出到 stderr，并允许与其他 Unix 工具串联

### Requirement: Spider must drive an end-to-end run loop in local mode
Spider SHALL 提供单机模式的端到端运行入口，串联 Downloader、PageProcessor、Pipeline 和 Scheduler，并在所有任务完成后自然终止。

#### Scenario: 从 seed 跑到所有派生请求处理完毕
- **WHEN** 调用方传入 seed 请求并触发运行
- **THEN** 系统先通过 `Scheduler::schedule` 把 seed 注入 `SpiderEngine` 的 domain dispatcher，再汇入全局 `async_channel` worker 通道，按域名级调度处理所有 seed 与派生请求，并在 in-flight 任务归零后关闭调度器与全局工作通道，返回运行汇总

#### Scenario: 单站点抓取达到页面上限
- **WHEN** 调用方为某次本地抓取配置单站点最大页面数
- **THEN** 系统在该域名接受到指定页数后停止继续调度同域名新请求，并在已接受任务处理完成后自然退出

#### Scenario: 本地模式同时挂载多个结果输出
- **WHEN** 调用方在构建 Spider 时挂载多个 pipeline
- **THEN** 单个 `Item` 会被广播给所有 pipeline 并发处理，而不是只选择其中一个输出通道

#### Scenario: 没有可用 seed
- **WHEN** 调用方传入空 seed 列表或所有 seed 被去重过滤
- **THEN** 系统不阻塞等待，立刻清理资源并返回空运行汇总

### Requirement: Quick-start CLI must load seeds from positional arguments
Quick-start CLI SHALL 把命令行位置参数视为 seed URL 列表，并在缺省情况下使用核心默认装配（DefaultDownloader + JsonLinesPipeline + engine-backed 默认调度器）启动 Spider。

#### Scenario: 通过命令行启动抓取
- **WHEN** 用户执行 `webmagic-cli URL [URL ...]`
- **THEN** CLI 把每个 URL 作为 GET seed 注入 SpiderBuilder 默认装配并触发 `Spider::run`

#### Scenario: 缺少 seed 或请求帮助
- **WHEN** 用户没有传入任何 URL，或显式传入 `-h` / `--help`
- **THEN** CLI 在 stderr 输出 usage 信息，并以非零（缺 seed）或 0（请求帮助）退出，不产生 stdout 输出

### Requirement: Local mode must keep stdout reserved for crawl data
Quick-start CLI SHALL 仅向 stdout 写入抓取数据流（每行一个 JSON 对象的 Item），日志、运行进度和错误必须写入 stderr。

#### Scenario: CLI 与 Unix 工具串联
- **WHEN** 用户把 `webmagic-cli` 的 stdout 通过管道接给 `jq` 或 `grep`
- **THEN** 下游工具看到的数据流不受运行进度、警告或错误干扰，运行汇总信息在 stderr 出现

### Requirement: SpiderBuilder must support distributed server mode
SpiderBuilder SHALL 支持分布式运行形态，并允许 Rust 数据面与外部控制面协同完成 Spider 生命周期管理。

#### Scenario: 由控制面动态下发 Spider 配置
- **WHEN** 外部控制面广播某个 Spider 的期望配置或生命周期指令
- **THEN** Rust 数据面可以根据配置动态拉起、更新或平滑终止本地 Spider 实例

### Requirement: Distributed routing must preserve domain affinity
分布式模式 SHALL 允许基于 URL 的 Domain 维持任务路由亲和性，以便节点退化为本地单机限流模型。

#### Scenario: 按域名路由抓取任务
- **WHEN** 分布式系统把抓取任务写入消息系统
- **THEN** 相同域名的任务被稳定路由到同一逻辑分区或节点，以减少跨节点分布式锁竞争

### Requirement: Distributed deployment must support global deduplication
分布式模式 SHALL 提供独立于单机内存结构之外的全局去重中心，用于跨节点防止 URL 重复消费。

#### Scenario: 多节点同时接收同一链接
- **WHEN** 多个节点有机会处理同一 URL
- **THEN** 系统通过全局去重中心保证该 URL 不被重复调度为有效新任务
