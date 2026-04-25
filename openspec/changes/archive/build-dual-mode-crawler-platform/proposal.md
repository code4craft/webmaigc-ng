## Why

webmagic-ng 需要同时覆盖两类截然不同的使用场景：一类是本地快速试抓、低门槛生成爬虫逻辑，另一类是云端长期运行、可观测、可治理的生产爬虫平台。当前仓库还处于空骨架阶段，必须先把“双模式”产品边界、实现顺序和能力拆分清楚，否则后续实现会在 CLI 体验、服务化架构和运维后台之间反复摇摆。

## What Changes

- 定义 Quick Start 模式的最小闭环：自然语言输入、页面分析、字段提取、代码生成、命令行运行和本地日志反馈。
- 定义 Server 模式的最小闭环：任务提交、鉴权、任务编排、定时调度、执行隔离、持久化日志和结果查询。
- 引入统一的爬虫项目模型，使 Quick Start 产出的逻辑可以无缝发布到 Server 模式执行，而不是维护两套能力。
- 定义 Admin 管理能力，包括任务大盘、运行状态监控、失败排查和按任务隔离的日志访问。
- 将整体建设拆分为多个阶段性里程碑，优先交付可工作的单机链路，再逐步补齐服务化与治理能力。

## Capabilities

### New Capabilities
- `quick-start-local-runner`: 面向本地和命令行的爬虫快速生成与运行能力，包括网页分析、代码固化、CLI 执行和调试日志。
- `crawler-project-model`: 统一 Quick Start 和 Server 模式的爬虫定义、配置、执行输入输出和打包发布约定。
- `server-task-execution`: 面向生产环境的任务接入、鉴权、调度、Worker 执行、Cron 触发和执行状态管理。
- `admin-observability-console`: 面向管理员和任务提交者的任务看板、运行日志、错误诊断和基础运维入口。

### Modified Capabilities

None.

## Impact

- 影响整体仓库结构，需要新增 CLI、Agent/Skill 集成、Server API、Scheduler/Worker、存储层和 Admin 前端等模块。
- 影响部署方式，需要定义本地运行、单机部署和生产集群部署的配置、镜像与环境变量规范。
- 影响依赖选择，需要引入浏览器自动化能力、任务队列/调度组件、鉴权机制、日志与指标基础设施。
- 影响后续研发流程，需要以统一 capability/spec 为中心推进实现和验收。
