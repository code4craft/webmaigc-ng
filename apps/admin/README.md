# Admin Placeholder

`apps/admin` 是管理端预留目录。

当前阶段不在 Rust workspace 内实现前端构建链路，原因如下：

- `1.1` 的目标是先明确模块边界，而不是过早绑定具体前端框架。
- 管理端首先消费 `services/server` 暴露的 API 和运维视图契约。
- 等 `5.x` 任务开始时，再根据需求决定采用 Web-only 还是桌面包装方案。

在此之前，所有 Admin 相关约束统一写入 `docs/` 和 `openspec/`。

