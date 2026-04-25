## 1. 核心抽象

- [x] 1.1 扩展 `core-module` 规范，定义 Downloader、PageProcessor、Scheduler、Pipeline 四大 Trait 的职责边界
- [x] 1.2 定义 SpiderBuilder、Page、Request、ProcessResult 等共享运行时 contract
- [x] 1.3 明确 `Scheduler` 对去重与排队的统一门面语义

## 2. 引擎与运行时

- [x] 2.1 定义事件驱动引擎的任务流，包括全局有界 Channel、Worker 池和 Domain Dispatcher
- [x] 2.2 定义 Domain 级限流、robots 缓存和 crawl-delay 更新机制
- [x] 2.3 定义多级反压行为和上游停止拉取语义

## 3. 下载层

- [x] 3.1 定义默认下载器能力边界，包括 rustls、连接池、HTTP/2、压缩和异步 DNS
- [x] 3.2 明确代理池与下载器扩展点的集成方式
- [x] 3.3 定义高吞吐场景下的连接复用和配置项暴露策略

## 4. 双态部署

- [x] 4.1 定义 Quick Start 本地模式的运行模型、配置加载方式和 stdout/stderr 约束
  - [x] 4.1.1 定义端到端运行模型（Spider::run、in-flight 反馈、终止顺序）
  - [x] 4.1.2 定义 CLI 配置加载方式（位置参数收 seed、`-h/--help` 输出 usage）
  - [x] 4.1.3 定义 stdout 数据流与 stderr 日志的具体管道约束（JsonLinesPipeline + stderr-only 进度日志）
- [x] 4.2 提供最基础的 HTML 通用 `PageProcessor`，解析页面所有 link 并回写 `Scheduler`
- [x] 4.3 在抓取配置中增加单站点最多页面数限制，并在调度链路中强制收敛
  - [x] 4.3.1 增加真实抓取测试，验证 `https://www.fifa.com/en/news` 不会把静态资源误判为页面链接
- [x] 4.4 定义 Pipeline 只读契约与 Spider 对多 Pipeline 的 fan-out 广播执行语义
- [x] 4.5 提供 `JsonFilePipeline`，支持本地 JSON Lines 异步追加落盘
- [x] 4.6 增加脚本 JSON/状态数据抽链处理器，并与基础 HTML 抽链处理器组合使用
- [ ] 4.7 规划后续持久化与流式输出 Pipeline
  - [ ] 4.7.1 定义 `MysqlPipeline` 的宽表 JSON 落盘方案与连接池约束
  - [ ] 4.7.2 定义 `KafkaPipeline` 的 Topic 输出与可选 routing key 约束

## 5. 合规与观测

- [ ] 5.1 定义 Robots.txt 获取、缓存、拦截与 Crawl-delay 映射策略
- [ ] 5.2 定义 SitemapProcessor 的递归解析语义

## 6. 落地计划

- [ ] 6.1 将本 change 与现有 `build-dual-mode-crawler-platform` roadmap 对齐，标明架构前置关系
- [ ] 6.2 输出后续实现顺序建议，先 core，再单机引擎，再分布式控制面

## 7.后续版本
- [ ] 7.1 定义 Server 分布式模式中 JS/TS 控制面、Rust Daemon、Redis 广播和 Kafka 路由的职责划分
- [ ] 7.2 定义全局去重中心与按域名路由亲和性的协作关系
- [ ] 7.3 定义 tracing、OpenTelemetry、metrics 与本地/分布式输出形态
- [ ] 7.4 预留性能压测、合规验证和观测接入的实施检查项
