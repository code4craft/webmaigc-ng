## ADDED Requirements

### Requirement: Engine must use bounded channel based event dispatch
引擎 SHALL 使用有界 Channel 将待执行请求分发给固定数量的 Worker 协程，而不是依赖主动轮询主循环。

#### Scenario: 通过有界通道分发任务
- **WHEN** Spider 产生待执行请求
- **THEN** 引擎通过有界 Channel 将请求发送给 Worker 池，并在容量耗尽时施加反压

### Requirement: Engine must maintain per-domain dispatchers
引擎 SHALL 为每个活跃域名维护独立的 Dispatcher 协程，用于承载该域名的本地队列、限流器和协议状态。

#### Scenario: 隔离不同域名的调度状态
- **WHEN** 系统同时抓取多个域名
- **THEN** 每个域名的 Crawl-delay、robots 缓存和本地积压状态彼此独立

### Requirement: Engine must support token-bucket rate limiting without busy waiting
引擎 SHALL 在 Domain Dispatcher 内使用令牌桶限流，并在无令牌时挂起协程而不是执行忙等轮询。

#### Scenario: 域名达到限流上限
- **WHEN** 某个域名在当前时间窗口内没有可用令牌
- **THEN** 其 Dispatcher 挂起等待下一次令牌可用，而不是持续占用 CPU

### Requirement: Engine must provide multi-stage backpressure
引擎 SHALL 提供从 Worker、全局 Channel、Domain 本地队列到上游拉取源的多级反压链路。

#### Scenario: 下游处理速度下降
- **WHEN** Worker 处理速度下降导致全局分发通道趋于饱和
- **THEN** Domain Dispatcher 的发送被阻塞，本地队列积压，并进一步阻止系统继续无界拉取新任务

