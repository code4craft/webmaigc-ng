mod downloader;

pub use downloader::{
    BoxFuture, CompressionConfig, ConnectionPoolConfig, DefaultDownloader, DefaultDownloaderConfig,
    DnsResolverMode, Downloader, DownloaderCapabilities, HttpProtocolPolicy, ProxyConfig,
    ProxyMode, ProxyProvider, ProxyTarget, StaticProxyProvider, TlsBackend,
};
