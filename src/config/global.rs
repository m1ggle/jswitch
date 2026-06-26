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
    pub plugins: PluginsConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub default_version: Option<String>,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub check_updates: bool,
    #[serde(default)]
    pub quiet_mode: bool,
}

/// Overridable download source URLs.  When a field is `None`, the fetcher
/// uses its built-in default.  Users can set these via `jswitch config set`
/// when an upstream changes its URL structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesConfig {
    /// Adoptium API base URL (e.g. `https://api.adoptium.net/v3/assets/feature_releases`).
    #[serde(default)]
    pub adoptopenjdk: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginsConfig {
    #[serde(default)]
    pub maven: bool,
    #[serde(default)]
    pub gradle: bool,
}
