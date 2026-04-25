# Repository Conventions

## 1. 模块边界

- `apps/*`: 用户直接调用的入口程序。
- `services/*`: 长生命周期服务进程。
- `crates/*`: 可复用的共享库，不承载独立部署职责。
- `apps/admin`: 当前为管理端占位目录，不在 Rust workspace 内；前端实现成熟后再单独引入构建链路。

## 2. Rust 代码约定

- Rust edition 统一使用 `2021`。
- 共享模型、错误类型、配置协议优先放在 `crates/core`。
- `apps/cli` 只依赖编排层，不直接持有服务端专属存储或调度实现。
- `services/server` 负责控制面职责，不直接内嵌重型执行逻辑。
- `services/worker` 负责执行职责，不承担 API 接入或管理后台职责。

## 3. 依赖管理

- 公共依赖优先提升到 root `Cargo.toml` 的 `[workspace.dependencies]`。
- 新 crate 默认通过 workspace 继承版本、edition、license。
- 引入外部重型依赖前，必须先说明它属于 `cli`、`server`、`worker` 还是 `core`。

## 4. 目录演进规则

- 新增运行时模块时，优先判断它是 `app`、`service` 还是 `crate`，不要平铺在仓库根目录。
- Quick Start 和 Server 共用的数据结构必须先沉淀到 `crates/core`，避免重复定义。
- 管理端需要消费的 API 契约，优先由 `server` 暴露并在文档中声明，等需求稳定后再决定是否抽成独立 crate。

## 5. 开发流程

- 结构性变更先更新 `openspec`，再改代码。
- 完成 OpenSpec task 后，立即回写 `tasks.md` 复选框。
- 提交前至少运行 `cargo check`；涉及格式变更时运行 `cargo fmt --all`。

