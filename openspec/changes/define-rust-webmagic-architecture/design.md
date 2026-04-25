## Context

本次 change 不是实现某个局部功能，而是为 Rust 版 webmagic-ng 定义总体架构蓝图。它承接了 Java 版 webmagic 的成熟经验，但不沿用传统的面向对象状态继承模型，而是重新适配 Rust 的 Trait、Arc、Tokio、Channel、CancellationToken 和 tracing 生态。

项目当前已经具备 Rust workspace 骨架，并已定义 crawler project model。下一步需要补齐“Spider 如何由哪些核心抽象构成、引擎如何调度、网络层如何榨取性能、双态部署如何共享代码、合规与观测如何作为原生能力进入系统”。

## Goals / Non-Goals

**Goals:**

- 明确定义 Rust 版 webmagic-ng 的四大核心 Trait 及其关系。
- 定义事件驱动引擎的调度、限流、反压和并发模型。
- 定义高性能下载层的网络优化原则与能力边界。
- 定义 Quick Start 与 Server 双态部署的共享模型与分工。
- 定义 Robots、Sitemap、Tracing 和 Metrics 的一等支持。

**Non-Goals:**

- 本 change 不直接实现所有运行时代码。
- 本 change 不定义完整 Admin UI 交互细节。
- 本 change 不承诺首版即达到 10 万+ QPS，但要求架构允许沿此方向演进。
- 本 change 不把 JS/TS 控制面的具体技术栈写死到实现层。

## Decisions

### 1. 核心抽象采用 Trait + 动态分发组合

Spider 的核心能力统一抽象为四个 Trait：

- `Downloader`：负责请求发送、重试、代理、压缩、协议细节。
- `PageProcessor`：无状态页面处理器，输入 `Page`，输出结构化 `ProcessResult`。
- `Scheduler`：统一封装去重与排队，不将二者暴露成上层必须手动编排的两个组件。
- `Pipeline`：负责结果落地和输出流向。

首个内置 `PageProcessor` 提供最基础的 HTML 链接发现能力：

- 解析页面中的 `href`
- 把相对路径解析为绝对 URL
- 过滤跨站与非 HTTP(S) 链接
- 将发现的链接回写给 `Scheduler`

这些能力在运行时通过 `Arc<dyn Trait>` 进行组合，由 `SpiderBuilder` 负责组装。

原因：

- Trait 更符合 Rust 的扩展方式，便于在不引入复杂继承层级的前提下开放扩展点。
- `Arc<dyn Trait>` 允许本地和服务端共用相同的抽象边界。

备选方案：

- 使用泛型把整个 Spider 参数化。优点是静态分发性能更好，但会放大类型复杂度，并增加双态部署与动态装配难度。

### 2. 引擎采用事件驱动，而不是轮询驱动

Spider 引擎不使用 `while true { poll() }` 主循环，而采用 Tokio 任务和有界 Channel 联动：

- 全局有界 `async_channel` MPMC Channel 负责把任务分发给固定数量 Worker。
- 每个活跃 Domain 派生独立 Dispatcher 协程，内部维护本地队列和限流器。
- Domain 协程在无令牌时挂起，不做忙等。
- Scheduler 不直接拥有一条“给单个消费者拉取”的总队列；它把通过去重的请求直接派发进 `SpiderEngine`，再由 Domain Dispatcher 汇入全局 MPMC worker 通道。

原因：

- 更自然地利用 Tokio 调度器。
- `async_channel` 原生支持 MPMC，允许 Scheduler 侧多路派发与 Spider 侧多 Worker 抢占消费天然闭环。
- 有界 Channel 天然提供反压基础。
- 按 Domain 派生调度协程可以把限流、robots、crawl-delay 和局部队列聚合到一起。

备选方案：

- 单全局队列 + 全局限流。缺点是不同域名之间相互干扰，且无法自然表达 per-domain robots 和 crawl-delay。

### 3. 采用三级反压作为系统自保护机制

反压链路定义为：

1. 下游 Worker 变慢。
2. 全局有界 Channel 被填满。
3. Domain Dispatcher 的发送被阻塞，本地队列积压。
4. 上游停止继续拉取或停止消费 Kafka。

原因：

- 避免在高峰期无限堆积内存。
- 让单机和分布式模式共享同一种流控语义。

同时，本地运行模型额外提供“单站点最多页面数”硬限制，作为比反压更强的业务级收敛阀门：

- 该限制按域名统计已接受请求数
- 命中上限后，`Scheduler` 直接丢弃同域名后续请求
- Spider 依旧通过已有的 in-flight 计数自然退出，不引入额外终止分支

### 4. 下载层优先做“协议与连接效率最大化”

下载器默认实现应基于 `reqwest + rustls`，同时明确支持：

- 激进 Keep-Alive 连接池。
- HTTP/2 多路复用。
- Brotli / Gzip / Deflate 解压。
- 异步 DNS。
- 代理能力扩展。

原因：

- Rust 版的性能优势必须首先体现在 I/O 层。
- 这些能力属于默认下载器应具备的工程基线，而不是后补优化。

### 5. 双态部署以 SpiderBuilder 为统一装配入口

SpiderBuilder 负责构建 Spider，无论目标是：

- CLI 单机模式：内存去重 + engine-backed scheduler + stdout/stderr 管道语义。
- Server 分布式模式：JS/TS 控制面 + Redis 配置广播 + Rust Daemon + Kafka 路由 + 全局去重中心。

原因：

- 保证“一套代码、双域运行”。
- 避免本地与服务端分叉成两套 Spider 组装逻辑。

### 6. 合规与观测是核心能力，不是外围插件

Robots、Sitemap、tracing、metrics、OpenTelemetry 都进入架构主线，而不是作为边缘插件。

原因：

- 爬虫框架天然面向长周期运行，合规与观测必须从一开始进入骨架。
- 如果后补，Domain 调度、请求上下文和日志结构都会返工。

## Risks / Trade-offs

- [动态分发引入少量运行时开销] → 用接口清晰性和可组合性换取微小损耗，热点路径后续可局部静态化。
- [Domain 协程数量过多] → 引入活跃域回收策略与空闲超时机制。
- [双态部署增加抽象复杂度] → 用 SpiderBuilder 和 shared core contract 保持边界一致。
- [Server 模式涉及 Redis/Kafka/JS 控制面，工程面复杂] → 首版先把协议和职责定义清楚，实现可分阶段落地。
- [合规逻辑可能拖慢吞吐] → 将 Robots 和 Sitemap 设计为可缓存、可复用、可独立限流的子流程。

## Migration Plan

由于当前仓库没有旧的 Rust 实现，本次迁移本质是“从空骨架进入架构成型阶段”：

1. 先调整 `core-module`，确立四大 Trait 与 SpiderBuilder 的 shared contract。
2. 再定义事件驱动引擎和默认下载层的运行时要求。
3. 随后实现 CLI 单机闭环，验证本地链路。
4. 最后逐步引入 Server 模式的控制面、Kafka 路由和全局去重中心。

回滚策略：

- 架构变更优先通过 spec/change 修正，不在实现层形成第二套运行模型。
- Server 模式依赖的外部组件保持可插拔，允许先退化为单机模式。

## Open Questions

- `PageProcessor` 首版是否只支持同步返回，还是直接定义异步处理接口。
- `Scheduler` trait 是否对外暴露去重命中原因和队列优先级。
- 分布式模式下全局去重首版优先 Redis Bloom、Redis Set 还是 Etcd。
- JS/TS 控制面与 Rust Daemon 之间的配置协议采用 JSON 还是更强约束的 schema。
