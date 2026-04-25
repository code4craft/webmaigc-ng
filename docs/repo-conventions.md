# Repository Conventions

## 1. 模块边界

- `apps/*`: 用户直接调用的入口程序。
- `services/*`: 长生命周期服务进程。
- `crates/*`: 可复用的共享库，不承载独立部署职责。
- `apps/admin`: 当前为管理端占位目录，不在 Rust workspace 内；前端实现成熟后再单独引入构建链路。

## 2. Rust 代码约定

- Rust edition 统一使用 `2021`。
- 工具链统一跟随仓库根目录的 `rust-toolchain.toml`，当前固定为 `1.93.0`。
- 共享模型、错误类型、配置协议优先放在 `crates/core`。
- `apps/cli` 只依赖编排层，不直接持有服务端专属存储或调度实现。
- `services/server` 负责控制面职责，不直接内嵌重型执行逻辑。
- `services/worker` 负责执行职责，不承担 API 接入或管理后台职责。
- `crates/core` 优先承载稳定 contract，不要轻易把 CLI、Server 专属语义塞回 core。
- 对外暴露的配置对象优先提供 `Default` 或显式 profile 构造函数，避免把大量魔法值散落在调用方。
- 新增公开类型时，优先补最小单元测试，至少覆盖默认值、校验逻辑和关键分支。

## 3. 依赖管理

- 公共依赖优先提升到 root `Cargo.toml` 的 `[workspace.dependencies]`。
- 新 crate 默认通过 workspace 继承版本、edition、license。
- 引入外部重型依赖前，必须先说明它属于 `cli`、`server`、`worker` 还是 `core`。
- `core` 中引入网络、存储、消息队列类依赖时，要先判断它是“共享 contract 需要”还是“运行时实现需要”。
- 若某个依赖受工具链版本影响明显，先验证与当前 `rust-toolchain.toml` 兼容，再合入。

## 4. 目录演进规则

- 新增运行时模块时，优先判断它是 `app`、`service` 还是 `crate`，不要平铺在仓库根目录。
- Quick Start 和 Server 共用的数据结构必须先沉淀到 `crates/core`，避免重复定义。
- 管理端需要消费的 API 契约，优先由 `server` 暴露并在文档中声明，等需求稳定后再决定是否抽成独立 crate。
- `spider` 目录表示执行引擎与装配相关语义；下载、调度、去重、请求等独立组件优先保持顶层模块，不再塞回 `spider`。

## 5. 开发流程

- 结构性变更先更新 `openspec`，再改代码。
- 完成 OpenSpec task 后，立即回写 `tasks.md` 复选框。
- 提交前至少运行 `cargo check`；涉及格式变更时运行 `cargo fmt --all`。
- 涉及核心行为变更时，优先运行 `cargo test -p webmagic-core`。
- 能做真实联网 smoke test 的地方，优先保留一条最小真实测试链路，避免 contract 和真实运行脱节。
- 修改设计边界或公共配置后，同步更新 `docs/core-module-design.md` 或其他对应文档。

## 6. 测试约定

- 默认优先写单元测试；涉及默认下载器、协议联通等真实路径时，可补充 integration test。
- 集成测试文件放在 crate 下的 `tests/` 目录，文件名直接表达场景。
- 测试名称优先描述行为，不写成模糊的 `test_xxx`。
- 对异步调度与反压场景，断言应考虑时序波动，避免把实现锁死为脆弱的瞬时状态。

## 7. 文档约定

- 架构边界写进 `docs/`，不要只留在对话里。
- 长期有效的规则写进 `docs/repo-conventions.md` 或 `AGENTS.md`。
- 阶段性实现计划写进 `openspec`，不要拿 `README` 代替实施清单。
