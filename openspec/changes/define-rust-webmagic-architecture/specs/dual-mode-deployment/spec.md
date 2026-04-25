## ADDED Requirements

### Requirement: SpiderBuilder must support quick-start local mode
SpiderBuilder SHALL 支持零外部依赖的本地 CLI 运行形态，并允许使用内存去重器、本地内存队列和标准输出管道进行抓取。

#### Scenario: 以 Unix 管道方式运行本地抓取
- **WHEN** 用户通过 CLI 在本地启动 Spider
- **THEN** 抓取结果输出到 stdout，日志输出到 stderr，并允许与其他 Unix 工具串联

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

