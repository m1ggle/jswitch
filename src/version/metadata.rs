use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionMetadata {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub source: Option<String>,
}
