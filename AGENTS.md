# AGENTS.md

## 基本约束

- 默认使用中文交流。
- 代码、文档、测试都优先保持直接、简洁，不引入不必要抽象。
- 先读现有结构，再决定新增模块归属；不要凭感觉平铺新目录。

## 仓库工作方式

- 结构性变更先改 `openspec`，再改实现。
- 完成某个 OpenSpec task 后，立即回写对应 `tasks.md` 复选框。
- 修改公共 contract、核心配置或模块边界后，同步更新 `docs/core-module-design.md` 或 `docs/repo-conventions.md`。

## Rust 约定

- 工具链跟随仓库根目录的 `rust-toolchain.toml`，当前固定为 `1.93.0`。
- 公共依赖优先放到 workspace 根 `Cargo.toml` 的 `[workspace.dependencies]`。
- `crates/core` 只承载共享 contract 和最小运行时实现，不轻易塞入 CLI、Server 专属语义。
- `spider` 表示引擎和装配；`request`、`downloader`、`scheduler`、`pipeline`、`dedup`、`queue` 保持独立模块。

## 代码风格

- 默认保持 ASCII。
- 优先小步修改，不做无关重构。
- 新增公开配置对象时，优先提供 `Default`、命名 profile 或显式校验函数。
- 新增错误信息时，直接说明失败原因，不写空泛文案。

## 测试要求

- 提交前至少运行 `cargo test -p webmagic-core`。
- 涉及格式调整时运行 `cargo fmt --all`。
- 涉及默认下载器、网络协议或真实抓取链路时，尽量保留一条真实 smoke test。
- 异步调度/反压测试要考虑时序波动，避免脆弱断言。
