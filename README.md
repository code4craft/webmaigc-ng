# webmagic-ng

`webmagic-ng` 是一个 Rust workspace，目标是逐步演进为同时支持 Quick Start 和 Server 模式的双模式爬虫平台。

## Workspace Layout

- `apps/cli`: 本地命令行入口，承接 Quick Start 模式。
- `crates/core`: 跨模式共享的领域模型、配置约束和通用能力。
- `services/server`: 服务端控制面，后续承接发布、任务、日志和鉴权 API。
- `services/worker`: 执行面，后续承接任务拉取、运行隔离和状态上报。
- `apps/admin`: Admin 入口预留目录，当前只承载文档和边界定义，后续可演进为 Web 客户端。
- `docs`: 架构和仓库规范文档。
- `openspec`: 需求、设计和实施任务。

## Current Status

当前已完成 Phase 0 的 `1.1`：

- 建立 Rust workspace 骨架
- 定义模块边界
- 定义仓库规范

后续任务请参考 `openspec/changes/build-dual-mode-crawler-platform/tasks.md`。

