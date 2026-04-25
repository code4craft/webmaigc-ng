## Why

统一爬虫项目模型是 webmagic-ng 双模式架构的共享底座。若不先稳定定义项目 contract，Quick Start 生成物、Server 发布链路、任务执行和日志追踪都会各自形成一套不兼容的数据结构，后续集成成本会持续放大。

## What Changes

- 定义 crawler project 的最小可用 contract，包括项目元数据、输入参数、输出 schema、运行入口和依赖声明。
- 定义 crawler project 的可移植性要求，确保本地生成物可直接发布到服务端执行。
- 定义 crawler project 的版本语义和发布前校验要求。
- 约束该模型在 Rust 代码中的落点，优先收敛到 `crates/core`。

## Capabilities

### New Capabilities
- `crawler-project-model`: 统一 Quick Start 与 Server 模式的爬虫项目定义、版本化、校验与执行契约。

### Modified Capabilities

None.

## Impact

- 影响 `crates/core` 的核心领域模型设计。
- 影响后续 CLI 项目生成、项目校验、项目打包和服务端项目发布接口。
- 影响任务实例如何绑定项目版本，以及运行日志如何回溯到具体项目定义。
