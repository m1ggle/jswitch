use reqwest::Proxy;

/// Resolved proxy configuration: environment variables override config-file values.
#[derive(Debug, Clone, Default)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
}

impl ProxyConfig {
    /// Merge `[proxy]` config with `JSWITCH_*` environment variables.
    /// Env vars win when both are present.
    pub fn from_config_and_env(config: &crate::config::Config) -> Self {
        Self {
            http_proxy: std::env::var("JSWITCH_HTTP_PROXY")
                .ok()
                .or_else(|| config.proxy.http_proxy.clone()),
            https_proxy: std::env::var("JSWITCH_HTTPS_PROXY")
                .ok()
                .or_else(|| config.proxy.https_proxy.clone()),
            no_proxy: std::env::var("JSWITCH_NO_PROXY")
                .ok()
                .or_else(|| config.proxy.no_proxy.clone()),
        }
    }

    /// Apply proxy settings to a reqwest client builder.
    pub fn apply(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        let mut builder = builder;
        if let Some(url) = &self.http_proxy
            && let Ok(proxy) = Proxy::http(url)
        {
            builder = builder.proxy(proxy);
        }
        if let Some(url) = &self.https_proxy
            && let Ok(proxy) = Proxy::https(url)
        {
            builder = builder.proxy(proxy);
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn falls_back_to_config_when_env_unset() {
        // Env vars are unlikely to be set in CI, but clear them for safety
        unsafe {
            std::env::remove_var("JSWITCH_HTTP_PROXY");
            std::env::remove_var("JSWITCH_HTTPS_PROXY");
        }

        let mut config = Config::default();
        config.proxy.http_proxy = Some("http://config-proxy:8080".to_owned());

        let proxy = ProxyConfig::from_config_and_env(&config);

        assert_eq!(
            proxy.http_proxy.as_deref(),
            Some("http://config-proxy:8080")
        );
    }

    #[test]
    fn defaults_to_empty_when_nothing_configured() {
        unsafe {
            std::env::remove_var("JSWITCH_HTTP_PROXY");
            std::env::remove_var("JSWITCH_HTTPS_PROXY");
        }

        let config = Config::default();
        let proxy = ProxyConfig::from_config_and_env(&config);

        assert!(proxy.http_proxy.is_none());
        assert!(proxy.https_proxy.is_none());
    }
}
