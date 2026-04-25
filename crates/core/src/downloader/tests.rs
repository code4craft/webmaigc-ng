use std::time::Duration;

use super::downloader::ThroughputProfile;
use super::*;
use crate::SpiderStage;

#[test]
fn default_downloader_config_exposes_expected_capabilities() {
    let config = DefaultDownloaderConfig::default();
    let capabilities = config.capabilities();

    assert_eq!(capabilities.throughput_profile, ThroughputProfile::Balanced);
    assert_eq!(capabilities.tls_backend, TlsBackend::Rustls);
    assert!(capabilities.connection_reuse);
    assert_eq!(capabilities.max_idle_per_host, 256);
    assert_eq!(capabilities.tcp_keepalive, Some(Duration::from_secs(60)));
    assert!(capabilities.http2_enabled);
    assert!(capabilities.http2_keep_alive_while_idle);
    assert!(capabilities.compression.brotli);
    assert!(capabilities.compression.gzip);
    assert!(capabilities.compression.deflate);
    assert!(capabilities.async_dns);
    assert_eq!(capabilities.proxy_mode, ProxyMode::Direct);
}

#[test]
fn default_downloader_config_requires_valid_pool_and_proxy_settings() {
    let invalid_pool = DefaultDownloaderConfig {
        connection_pool: ConnectionPoolConfig {
            max_idle_per_host: 0,
            ..ConnectionPoolConfig::default()
        },
        ..DefaultDownloaderConfig::default()
    };
    assert!(invalid_pool.validate().is_err());

    let invalid_proxy = DefaultDownloaderConfig {
        proxy: ProxyConfig {
            mode: ProxyMode::Static,
            endpoint: None,
        },
        ..DefaultDownloaderConfig::default()
    };
    assert!(invalid_proxy.validate().is_err());

    let invalid_keepalive = DefaultDownloaderConfig {
        connection_pool: ConnectionPoolConfig {
            tcp_keepalive: Some(Duration::ZERO),
            ..ConnectionPoolConfig::default()
        },
        ..DefaultDownloaderConfig::default()
    };
    assert!(invalid_keepalive.validate().is_err());

    assert!(DefaultDownloaderConfig::default().validate().is_ok());
}

#[test]
fn high_throughput_profile_exposes_more_aggressive_reuse_settings() {
    let balanced = DefaultDownloaderConfig::balanced();
    let throughput = DefaultDownloaderConfig::high_throughput();

    assert_eq!(
        throughput.throughput_profile,
        ThroughputProfile::HighThroughput
    );
    assert!(
        throughput.connection_pool.max_idle_per_host > balanced.connection_pool.max_idle_per_host
    );
    assert!(throughput.connection_pool.connect_timeout < balanced.connection_pool.connect_timeout);
    assert!(throughput.http2.adaptive_window);
    assert!(throughput.validate().is_ok());
}

#[test]
fn default_downloader_requires_provider_for_dynamic_proxy_mode() {
    let config = DefaultDownloaderConfig {
        proxy: ProxyConfig {
            mode: ProxyMode::DynamicPool,
            endpoint: None,
        },
        ..DefaultDownloaderConfig::default()
    };

    let err = match DefaultDownloader::new(config) {
        Ok(_) => panic!("dynamic mode should require provider"),
        Err(err) => err,
    };
    assert_eq!(err.stage, SpiderStage::Build);
}

#[test]
fn default_downloader_builds_with_direct_mode() {
    let downloader = DefaultDownloader::new(DefaultDownloaderConfig::default());
    assert!(downloader.is_ok());
}
