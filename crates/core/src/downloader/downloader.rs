use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use dashmap::DashMap;
use reqwest::{Client, Method, Proxy};
use serde::{Deserialize, Serialize};

use crate::{HeaderMap, Page, Request, RequestMethod, SpiderError, SpiderStage};

/// Shared boxed future type for async crawler contracts without binding the core crate
/// to a specific async runtime implementation.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Downloader owns network I/O concerns.
///
/// Implementations are responsible for protocol details, retries, proxies, compression,
/// connection reuse, and transport-layer policies. They should not own crawl graph logic,
/// parsing logic, or persistence logic.
pub trait Downloader: Send + Sync {
    type Error: From<SpiderError> + Send + 'static;

    fn download(&self, request: Request) -> BoxFuture<'_, Result<Page, Self::Error>>;
}

pub trait ProxyProvider: Send + Sync {
    fn proxy_for(
        &self,
        request: &Request,
    ) -> BoxFuture<'_, Result<Option<ProxyTarget>, SpiderError>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyTarget {
    pub endpoint: String,
}

impl ProxyTarget {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticProxyProvider {
    target: ProxyTarget,
}

impl StaticProxyProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            target: ProxyTarget::new(endpoint),
        }
    }
}

impl ProxyProvider for StaticProxyProvider {
    fn proxy_for(
        &self,
        _request: &Request,
    ) -> BoxFuture<'_, Result<Option<ProxyTarget>, SpiderError>> {
        let target = self.target.clone();
        Box::pin(async move { Ok(Some(target)) })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TlsBackend {
    Rustls,
    NativeTls,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HttpProtocolPolicy {
    Http1Only,
    PreferHttp2,
    RequireHttp2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DnsResolverMode {
    AsyncHickory,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    Direct,
    Static,
    DynamicPool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThroughputProfile {
    Conservative,
    Balanced,
    HighThroughput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionPoolConfig {
    pub max_idle_per_host: usize,
    pub idle_timeout: Duration,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub tcp_keepalive: Option<Duration>,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 256,
            idle_timeout: Duration::from_secs(90),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            tcp_keepalive: Some(Duration::from_secs(60)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Http2Config {
    pub adaptive_window: bool,
    pub keep_alive_interval: Option<Duration>,
    pub keep_alive_timeout: Duration,
    pub keep_alive_while_idle: bool,
}

impl Default for Http2Config {
    fn default() -> Self {
        Self {
            adaptive_window: true,
            keep_alive_interval: Some(Duration::from_secs(30)),
            keep_alive_timeout: Duration::from_secs(10),
            keep_alive_while_idle: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressionConfig {
    pub brotli: bool,
    pub gzip: bool,
    pub deflate: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            brotli: true,
            gzip: true,
            deflate: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub endpoint: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Direct,
            endpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultDownloaderConfig {
    pub throughput_profile: ThroughputProfile,
    pub tls_backend: TlsBackend,
    pub http_protocol: HttpProtocolPolicy,
    pub connection_pool: ConnectionPoolConfig,
    pub http2: Http2Config,
    pub compression: CompressionConfig,
    pub dns_resolver: DnsResolverMode,
    pub proxy: ProxyConfig,
}

impl Default for DefaultDownloaderConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

impl DefaultDownloaderConfig {
    pub fn conservative() -> Self {
        Self {
            throughput_profile: ThroughputProfile::Conservative,
            tls_backend: TlsBackend::Rustls,
            http_protocol: HttpProtocolPolicy::PreferHttp2,
            connection_pool: ConnectionPoolConfig {
                max_idle_per_host: 64,
                idle_timeout: Duration::from_secs(30),
                connect_timeout: Duration::from_secs(10),
                request_timeout: Duration::from_secs(30),
                tcp_keepalive: Some(Duration::from_secs(30)),
            },
            http2: Http2Config {
                adaptive_window: false,
                keep_alive_interval: Some(Duration::from_secs(20)),
                keep_alive_timeout: Duration::from_secs(10),
                keep_alive_while_idle: false,
            },
            compression: CompressionConfig::default(),
            dns_resolver: DnsResolverMode::AsyncHickory,
            proxy: ProxyConfig::default(),
        }
    }

    pub fn balanced() -> Self {
        Self {
            throughput_profile: ThroughputProfile::Balanced,
            tls_backend: TlsBackend::Rustls,
            http_protocol: HttpProtocolPolicy::PreferHttp2,
            connection_pool: ConnectionPoolConfig::default(),
            http2: Http2Config::default(),
            compression: CompressionConfig::default(),
            dns_resolver: DnsResolverMode::AsyncHickory,
            proxy: ProxyConfig::default(),
        }
    }

    pub fn high_throughput() -> Self {
        Self {
            throughput_profile: ThroughputProfile::HighThroughput,
            tls_backend: TlsBackend::Rustls,
            http_protocol: HttpProtocolPolicy::PreferHttp2,
            connection_pool: ConnectionPoolConfig {
                max_idle_per_host: 1024,
                idle_timeout: Duration::from_secs(120),
                connect_timeout: Duration::from_secs(5),
                request_timeout: Duration::from_secs(20),
                tcp_keepalive: Some(Duration::from_secs(90)),
            },
            http2: Http2Config {
                adaptive_window: true,
                keep_alive_interval: Some(Duration::from_secs(15)),
                keep_alive_timeout: Duration::from_secs(10),
                keep_alive_while_idle: true,
            },
            compression: CompressionConfig::default(),
            dns_resolver: DnsResolverMode::AsyncHickory,
            proxy: ProxyConfig::default(),
        }
    }

    pub fn capabilities(&self) -> DownloaderCapabilities {
        DownloaderCapabilities {
            throughput_profile: self.throughput_profile,
            tls_backend: self.tls_backend,
            connection_reuse: self.connection_pool.max_idle_per_host > 0,
            max_idle_per_host: self.connection_pool.max_idle_per_host,
            tcp_keepalive: self.connection_pool.tcp_keepalive,
            http2_enabled: !matches!(self.http_protocol, HttpProtocolPolicy::Http1Only),
            http2_keep_alive_while_idle: self.http2.keep_alive_while_idle,
            compression: self.compression.clone(),
            async_dns: matches!(self.dns_resolver, DnsResolverMode::AsyncHickory),
            proxy_mode: self.proxy.mode,
        }
    }

    pub fn validate(&self) -> Result<(), SpiderError> {
        if self.connection_pool.max_idle_per_host == 0 {
            return Err(SpiderError::new(
                crate::SpiderStage::Build,
                "default downloader requires max_idle_per_host > 0",
            ));
        }

        if self.connection_pool.connect_timeout.is_zero() {
            return Err(SpiderError::new(
                crate::SpiderStage::Build,
                "default downloader requires connect_timeout > 0",
            ));
        }

        if self.connection_pool.request_timeout.is_zero() {
            return Err(SpiderError::new(
                crate::SpiderStage::Build,
                "default downloader requires request_timeout > 0",
            ));
        }

        if let Some(tcp_keepalive) = self.connection_pool.tcp_keepalive {
            if tcp_keepalive.is_zero() {
                return Err(SpiderError::new(
                    crate::SpiderStage::Build,
                    "default downloader requires tcp_keepalive > 0 when configured",
                ));
            }
        }

        if self.http2.keep_alive_timeout.is_zero() {
            return Err(SpiderError::new(
                crate::SpiderStage::Build,
                "default downloader requires http2 keep_alive_timeout > 0",
            ));
        }

        if matches!(self.proxy.mode, ProxyMode::Static) && self.proxy.endpoint.is_none() {
            return Err(SpiderError::new(
                crate::SpiderStage::Build,
                "static proxy mode requires proxy endpoint",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloaderCapabilities {
    pub throughput_profile: ThroughputProfile,
    pub tls_backend: TlsBackend,
    pub connection_reuse: bool,
    pub max_idle_per_host: usize,
    pub tcp_keepalive: Option<Duration>,
    pub http2_enabled: bool,
    pub http2_keep_alive_while_idle: bool,
    pub compression: CompressionConfig,
    pub async_dns: bool,
    pub proxy_mode: ProxyMode,
}

pub struct DefaultDownloader {
    config: DefaultDownloaderConfig,
    direct_client: Client,
    proxy_provider: Option<Arc<dyn ProxyProvider>>,
    proxied_clients: DashMap<String, Client>,
}

impl DefaultDownloader {
    pub fn new(config: DefaultDownloaderConfig) -> Result<Self, SpiderError> {
        Self::with_proxy_provider(config, None)
    }

    pub fn with_proxy_provider(
        config: DefaultDownloaderConfig,
        proxy_provider: Option<Arc<dyn ProxyProvider>>,
    ) -> Result<Self, SpiderError> {
        config.validate()?;

        if matches!(config.proxy.mode, ProxyMode::DynamicPool) && proxy_provider.is_none() {
            return Err(SpiderError::new(
                SpiderStage::Build,
                "dynamic proxy mode requires a proxy provider",
            ));
        }

        let direct_proxy = match config.proxy.mode {
            ProxyMode::Static => config.proxy.endpoint.clone(),
            _ => None,
        };

        let direct_client = build_client(&config, direct_proxy.as_deref())?;

        Ok(Self {
            config,
            direct_client,
            proxy_provider,
            proxied_clients: DashMap::new(),
        })
    }

    async fn resolve_client(&self, request: &Request) -> Result<Client, SpiderError> {
        if !matches!(self.config.proxy.mode, ProxyMode::DynamicPool) {
            return Ok(self.direct_client.clone());
        }

        let provider = self.proxy_provider.as_ref().ok_or_else(|| {
            SpiderError::new(
                SpiderStage::Build,
                "dynamic proxy mode requires a proxy provider",
            )
        })?;

        let target = provider.proxy_for(request).await?;
        let Some(target) = target else {
            return Ok(self.direct_client.clone());
        };

        if let Some(existing) = self.proxied_clients.get(&target.endpoint) {
            return Ok(existing.clone());
        }

        let client = build_client(&self.config, Some(target.endpoint.as_str()))?;
        self.proxied_clients
            .entry(target.endpoint)
            .or_insert_with(|| client.clone());
        Ok(client)
    }
}

impl Downloader for DefaultDownloader {
    type Error = SpiderError;

    fn download(&self, request: Request) -> BoxFuture<'_, Result<Page, Self::Error>> {
        Box::pin(async move {
            let client = self.resolve_client(&request).await?;
            let method = to_reqwest_method(&request.method);

            let mut builder = client.request(method, &request.url);
            for (name, value) in &request.headers {
                builder = builder.header(name, value);
            }

            if let Some(body) = request.body.clone() {
                builder = builder.body(body);
            }

            let response = builder
                .send()
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Download, err.to_string()))?;

            let final_url = response.url().to_string();
            let status_code = response.status().as_u16();
            let headers = to_header_map(response.headers());
            let body = response
                .bytes()
                .await
                .map_err(|err| SpiderError::new(SpiderStage::Download, err.to_string()))?
                .to_vec();

            Ok(Page {
                request,
                final_url,
                status_code,
                headers,
                body,
            })
        })
    }
}

fn build_client(
    config: &DefaultDownloaderConfig,
    proxy_endpoint: Option<&str>,
) -> Result<Client, SpiderError> {
    let mut builder = Client::builder()
        .connect_timeout(config.connection_pool.connect_timeout)
        .timeout(config.connection_pool.request_timeout)
        .pool_idle_timeout(config.connection_pool.idle_timeout)
        .pool_max_idle_per_host(config.connection_pool.max_idle_per_host)
        .tcp_keepalive(config.connection_pool.tcp_keepalive)
        .brotli(config.compression.brotli)
        .gzip(config.compression.gzip)
        .deflate(config.compression.deflate);

    builder = match config.tls_backend {
        TlsBackend::Rustls => builder.use_rustls_tls(),
        TlsBackend::NativeTls => builder,
    };

    builder = match config.http_protocol {
        HttpProtocolPolicy::Http1Only => builder.http1_only(),
        HttpProtocolPolicy::PreferHttp2 | HttpProtocolPolicy::RequireHttp2 => builder,
    };

    builder = match config.dns_resolver {
        DnsResolverMode::AsyncHickory => builder.hickory_dns(true),
        DnsResolverMode::System => builder.no_hickory_dns(),
    };

    if let Some(endpoint) = proxy_endpoint {
        let proxy = Proxy::all(endpoint)
            .map_err(|err| SpiderError::new(SpiderStage::Build, err.to_string()))?;
        builder = builder.proxy(proxy);
    }

    builder
        .build()
        .map_err(|err| SpiderError::new(SpiderStage::Build, err.to_string()))
}

fn to_reqwest_method(method: &RequestMethod) -> Method {
    match method {
        RequestMethod::Get => Method::GET,
        RequestMethod::Post => Method::POST,
        RequestMethod::Put => Method::PUT,
        RequestMethod::Patch => Method::PATCH,
        RequestMethod::Delete => Method::DELETE,
        RequestMethod::Head => Method::HEAD,
    }
}

fn to_header_map(headers: &reqwest::header::HeaderMap) -> HeaderMap {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}
