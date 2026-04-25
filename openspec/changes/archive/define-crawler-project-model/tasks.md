## 1. Project Contract

- [x] 1.1 定义 crawler project 的核心 Rust 类型，包括项目元数据、运行契约、输入参数和输出 schema
- [x] 1.2 定义项目版本标识与发布前引用语义，避免依赖本地路径
- [x] 1.3 定义静态校验和环境校验的结果类型与错误分类

## 2. Core Implementation

- [x] 2.1 在 `crates/core` 中实现 crawler project model 的模块划分与公开导出
- [x] 2.2 为项目 manifest 和 runtime contract 增加序列化与反序列化支持
- [x] 2.3 为校验结果和版本标识补充基础单元测试

## 3. Integration Contract

- [x] 3.1 补充文档，说明 CLI、server、worker 应如何依赖 shared crawler project model
- [x] 3.2 在总 roadmap 中标注 `1.2` 由该独立 change 承接
