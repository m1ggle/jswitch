use crate::config::Config;

use super::ProxyConfig;

/// HTTP client wrapper that applies proxy settings from config and environment.
#[derive(Debug, Clone)]
pub struct DownloadClient {
    inner: reqwest::Client,
}

impl DownloadClient {
    /// Bare constructor with no proxy — kept for tests and simple use cases.
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    /// Build a client from `Config`, automatically applying proxy settings.
    /// Environment variables `JSWITCH_HTTP_PROXY` / `JSWITCH_HTTPS_PROXY` take
    /// precedence over `[proxy]` entries in config.toml.
    pub fn from_config(config: &Config) -> Self {
        let proxy = ProxyConfig::from_config_and_env(config);
        let builder = proxy.apply(reqwest::Client::builder());
        Self {
            inner: builder.build().unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }
}

impl Default for DownloadClient {
    fn default() -> Self {
        Self::new()
    }
}
