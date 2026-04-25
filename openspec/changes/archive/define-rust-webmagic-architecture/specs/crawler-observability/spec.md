## ADDED Requirements

### Requirement: Framework must integrate tracing as the primary execution telemetry model
框架 SHALL 以 `tracing` 作为核心执行链路的日志与调用链抽象。

#### Scenario: 为请求生命周期记录 span
- **WHEN** Spider 发起下载、处理页面、调度请求和输出结果
- **THEN** 系统能够在统一 tracing 上下文中记录这些阶段的 span 和事件

### Requirement: Local mode must emit structured logs to stderr
本地模式 SHALL 将结构化日志定向输出到 stderr，而不污染 stdout 的抓取数据流。

#### Scenario: 本地模式输出数据与日志
- **WHEN** CLI 模式运行 Spider
- **THEN** JSON Lines 数据输出到 stdout，结构化日志输出到 stderr

### Requirement: Distributed mode must support trace propagation across components
分布式模式 SHALL 支持跨组件传递 Trace 上下文，以便将多节点执行链路拼接为完整调用链。

#### Scenario: 跨节点传递追踪上下文
- **WHEN** 抓取任务在不同节点、不同进程之间流转
- **THEN** 系统能够注入、提取并延续 Trace 标识，使链路追踪系统恢复完整请求路径

### Requirement: Framework must provide low-overhead runtime metrics
框架 SHALL 提供低开销运行时指标采集，并支持后台聚合与 Prometheus 风格暴露两类输出方式。

#### Scenario: 周期性上报指标
- **WHEN** Spider 长时间运行
- **THEN** 系统能够周期性汇总请求量、成功率、失败数、队列积压和活跃域等指标，而不在高频路径执行阻塞式上报

