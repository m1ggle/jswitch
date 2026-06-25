use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
// Keep this structure aligned with the public ~/.jswitch/config.toml section layout.
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub sources: SourcesConfig,
    #[serde(default)]
    pub proxy: ProxyConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub default_version: Option<String>,
    #[serde(default)]
    pub mirror_url: Option<String>,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub check_updates: bool,
    #[serde(default)]
    pub quiet_mode: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesConfig {
    #[serde(default)]
    pub openjdk: Option<String>,
    #[serde(default)]
    pub corretto: Option<String>,
    #[serde(default)]
    pub adoptopenjdk: Option<String>,
    #[serde(default)]
    pub oracle: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    #[serde(default)]
    pub http_proxy: Option<String>,
    #[serde(default)]
    pub https_proxy: Option<String>,
    #[serde(default)]
    pub no_proxy: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginsConfig {
    #[serde(default)]
    pub maven: bool,
    #[serde(default)]
    pub gradle: bool,
}
