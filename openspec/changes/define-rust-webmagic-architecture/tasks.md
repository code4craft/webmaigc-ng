## 1. 核心抽象

- [x] 1.1 扩展 `core-module` 规范，定义 Downloader、PageProcessor、Scheduler、Pipeline 四大 Trait 的职责边界
- [x] 1.2 定义 SpiderBuilder、Page、Request、ProcessResult 等共享运行时 contract
- [x] 1.3 明确 `Scheduler` 对去重与排队的统一门面语义

## 2. 引擎与运行时

- [ ] 2.1 定义事件驱动引擎的任务流，包括全局有界 Channel、Worker 池和 Domain Dispatcher
- [ ] 2.2 定义 Domain 级限流、robots 缓存和 crawl-delay 更新机制
- [ ] 2.3 定义多级反压行为和上游停止拉取语义

## 3. 下载层

- [ ] 3.1 定义默认下载器能力边界，包括 rustls、连接池、HTTP/2、压缩和异步 DNS
- [ ] 3.2 明确代理池与下载器扩展点的集成方式
- [ ] 3.3 定义高吞吐场景下的连接复用和配置项暴露策略

## 4. 双态部署

- [ ] 4.1 定义 Quick Start 本地模式的运行模型、配置加载方式和 stdout/stderr 约束
- [ ] 4.2 定义 Server 分布式模式中 JS/TS 控制面、Rust Daemon、Redis 广播和 Kafka 路由的职责划分
- [ ] 4.3 定义全局去重中心与按域名路由亲和性的协作关系

## 5. 合规与观测

- [ ] 5.1 定义 Robots.txt 获取、缓存、拦截与 Crawl-delay 映射策略
- [ ] 5.2 定义 SitemapProcessor 的递归解析语义
- [ ] 5.3 定义 tracing、OpenTelemetry、metrics 与本地/分布式输出形态

## 6. 落地计划

- [ ] 6.1 将本 change 与现有 `build-dual-mode-crawler-platform` roadmap 对齐，标明架构前置关系
- [ ] 6.2 输出后续实现顺序建议，先 core，再单机引擎，再分布式控制面
- [ ] 6.3 预留性能压测、合规验证和观测接入的实施检查项
