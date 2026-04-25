## Context

`build-dual-mode-crawler-platform` 已经把 crawler project model 识别为整个系统的共享底座，但那份 change 的范围过大，不适合细化和评审模型 contract。这里将其拆成独立 change，专门定义“什么是一个可运行、可发布、可追踪的 crawler project”。

当前仓库已经是 Rust workspace，最合理的模型承载位置是 `crates/core`。该模型需要同时满足本地 CLI、服务端发布链路和 Worker 执行，因此必须避免掺入具体传输层、存储层和 UI 层实现细节。

## Goals / Non-Goals

**Goals:**

- 定义 crawler project 的核心数据结构和字段边界。
- 定义本地运行与服务端发布共享的项目 contract。
- 定义项目版本、校验和运行前环境检查的语义。
- 约束模型优先沉淀到 `crates/core`，供后续模块复用。

**Non-Goals:**

- 不定义具体 HTTP API 形态。
- 不实现项目打包产物上传协议。
- 不实现实际浏览器执行逻辑。
- 不在本 change 中定义任务调度或日志存储 schema。

## Decisions

### 1. 采用“项目清单 + 运行契约”二层模型

crawler project 由两层组成：

- Project Manifest：声明项目标识、站点信息、输入参数、输出 schema、依赖和入口信息。
- Runtime Contract：声明执行需要的运行入口、环境前提、资源需求和校验规则。

原因：

- 既能表达“这个项目是什么”，也能表达“这个项目怎么被执行”。
- 适合本地 CLI 和服务端执行共享，不会把实现细节压缩成单一大对象。

备选方案：

- 只定义一个扁平结构体。问题是后续扩展环境要求、资源要求和发布元数据时会迅速失控。

### 2. 版本身份必须独立于代码路径

项目版本应该由显式版本标识承载，而不是依赖本地文件路径或临时目录名。

原因：

- 服务端任务实例必须能稳定引用某个项目版本。
- 日志、任务、回滚和审计都需要版本身份。

备选方案：

- 运行时临时生成匿名版本。问题是不可追踪，也不利于发布和回放。

### 3. 校验分为静态校验与环境校验

项目校验至少分两类：

- 静态校验：检查 manifest 字段完整性、入口合法性、参数 schema 合法性。
- 环境校验：检查浏览器依赖、必要配置、运行时能力是否满足。

原因：

- 静态错误和运行环境错误需要不同的反馈方式。
- 后续 CLI `validate` 与服务端 preflight 可以共享这套语义。

备选方案：

- 合并成单一“validate”结果。问题是错误定位粒度太粗，难以支持自动修复或按场景重试。

### 4. core 只承载稳定 contract，不承载执行细节

`crates/core` 只定义 project model、基础枚举、校验结果类型和版本标识，不直接依赖浏览器库、HTTP 框架或数据库驱动。

原因：

- 保持共享 crate 轻量，避免下游模块无谓耦合。
- 使 CLI、server、worker 可以安全复用。

## Risks / Trade-offs

- [模型设计过小导致后续字段不够用] → 先覆盖跨模式共享必需项，允许通过非破坏性字段扩展。
- [模型设计过大导致 core 过度抽象] → 明确 non-goals，不在此阶段塞入调度、存储和 UI 关切。
- [版本语义过早锁死] → 首版先定义身份和兼容边界，不预先承诺复杂升级机制。
- [校验职责散落到各模块] → 将校验结果类型和基础校验入口统一沉淀到 core。

## Migration Plan

当前没有旧实现需要迁移，本 change 的迁移重点是后续接入顺序：

1. 先在 `crates/core` 定义 manifest、runtime contract、version 和 validate 相关类型。
2. CLI 生成和校验命令改为消费该模型。
3. 服务端发布和任务实例模型引用该项目版本标识。
4. Worker 执行器根据 runtime contract 做执行前检查。

回滚策略：

- 若后续实现发现字段设计不合理，优先在 change/spec 层修正后再调整代码，不直接在实现层分叉新的 project model。

## Open Questions

- 首版 manifest 使用 TOML、YAML 还是 JSON 作为主序列化格式。
- 入口信息是否只支持单入口，还是提前支持多阶段 pipeline。
- 环境校验结果是否需要区分 fatal 与 warning。
