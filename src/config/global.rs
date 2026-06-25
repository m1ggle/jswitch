use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
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
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}
