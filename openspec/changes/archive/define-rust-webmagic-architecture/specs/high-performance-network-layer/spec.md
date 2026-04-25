## ADDED Requirements

### Requirement: Default downloader must use Rust-native secure transport
默认下载器 SHALL 基于 Rust 原生 TLS 栈实现安全传输，并避免依赖系统默认 TLS 作为唯一实现路径。

#### Scenario: 建立默认 HTTPS 连接
- **WHEN** Spider 使用默认下载器访问 HTTPS 目标
- **THEN** 下载器通过 Rust 原生 TLS 栈完成握手并复用该连接能力

### Requirement: Downloader must support aggressive connection reuse
下载器 SHALL 支持高效的 Keep-Alive 连接池和按 Host 的连接复用，以减少高并发下的握手与建连开销。

#### Scenario: 高频访问同一站点
- **WHEN** Spider 持续抓取同一 Host 的大量页面
- **THEN** 下载器优先复用已建立连接，而不是为每个请求重新创建 TCP 连接

### Requirement: Downloader must support HTTP/2 multiplexing and compression decoding
下载器 SHALL 支持 HTTP/2 多路复用，以及 Brotli、Gzip、Deflate 等常见压缩响应解码。

#### Scenario: 处理启用压缩和多路复用的站点
- **WHEN** 目标站点同时启用 HTTP/2 和压缩编码
- **THEN** 下载器能够在共享连接上并发请求并自动解压响应体

### Requirement: Downloader must support asynchronous DNS and proxy extensibility
下载器 SHALL 支持异步 DNS 解析，并为代理池或动态代理策略预留扩展能力。

#### Scenario: 在高并发场景下解析域名并切换代理
- **WHEN** Spider 运行在需要高并发解析和代理切换的环境中
- **THEN** 下载器不会因同步 DNS 阻塞而放大延迟，并且可以接入可替换的代理策略

