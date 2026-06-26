use serde::Deserialize;

use crate::{config::global::SourcesConfig, error::jswitch_error::NetworkError};

const DEFAULT_ADOPTIUM_BASE_URL: &str = "https://api.adoptium.net/v3/assets/feature_releases";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteVersion {
    pub version: String,
    pub archive_name: String,
    pub download_url: String,
    pub checksum_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VersionFetcher {
    client: reqwest::Client,
    sources: SourcesConfig,
}

impl VersionFetcher {
    pub fn new(client: reqwest::Client, sources: SourcesConfig) -> Self {
        Self { client, sources }
    }

    fn adoptium_base_url(&self) -> &str {
        self.sources
            .adoptopenjdk
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_ADOPTIUM_BASE_URL)
    }

    pub async fn fetch(&self, requested: &str) -> Result<RemoteVersion, NetworkError> {
        let major = resolve_feature_version(requested)?;
        self.fetch_adoptium(major).await
    }

    async fn fetch_adoptium(&self, major: u32) -> Result<RemoteVersion, NetworkError> {
        let image_type = "jdk";
        let base = self.adoptium_base_url();
        let url = format!(
            "{base}/{major}/ga?architecture={}&heap_size=normal&image_type={image_type}&jvm_impl=hotspot&os={}&page_size=1&project=jdk&sort_method=DEFAULT&sort_order=DESC&vendor=eclipse",
            architecture(),
            operating_system(),
        );

        let assets = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<AdoptiumAsset>>()
            .await?;

        assets
            .into_iter()
            .find_map(|asset| asset.into_remote_version())
            .ok_or_else(|| NetworkError::RemoteVersionNotFound(major.to_string()))
    }
}

fn resolve_feature_version(requested: &str) -> Result<u32, NetworkError> {
    match requested {
        "lts" | "stable" => Ok(21),
        "latest" => Ok(25),
        value => {
            let parts = value.split('.').collect::<Vec<_>>();
            let major = parts
                .first()
                .and_then(|major| major.parse::<u32>().ok())
                .ok_or_else(|| NetworkError::UnsupportedVersion(value.to_owned()))?;

            if major == 1 && matches!(parts.get(1), Some(&"8")) {
                Ok(8)
            } else {
                Ok(major)
            }
        }
    }
}

// --- Platform helpers ---

fn operating_system() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mac",
        "windows" => "windows",
        _ => "linux",
    }
}

fn architecture() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x64",
    }
}

// --- Adoptium API response types ---

#[derive(Debug, Deserialize)]
struct AdoptiumAsset {
    // Some API entries (e.g. source-only releases) omit the `binary` field.
    binary: Option<AdoptiumBinary>,
    version_data: AdoptiumVersionData,
}

impl AdoptiumAsset {
    fn into_remote_version(self) -> Option<RemoteVersion> {
        let package = self.binary?.package;
        Some(RemoteVersion {
            version: self.version_data.semver,
            archive_name: package.name?,
            download_url: package.link?,
            checksum_url: package.checksum_link,
        })
    }
}

#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    name: Option<String>,
    link: Option<String>,
    checksum_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdoptiumVersionData {
    semver: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_major_versions() {
        assert_eq!(resolve_feature_version("17").unwrap(), 17);
        assert_eq!(resolve_feature_version("17.0.10").unwrap(), 17);
        assert_eq!(resolve_feature_version("1.8").unwrap(), 8);
        assert_eq!(resolve_feature_version("1.8.0_402").unwrap(), 8);
        assert_eq!(resolve_feature_version("lts").unwrap(), 21);
    }

    #[test]
    fn maps_platform_names() {
        assert!(!operating_system().is_empty());
        assert!(!architecture().is_empty());
    }

    #[test]
    fn uses_config_overridden_adoptium_url() {
        let sources = SourcesConfig {
            adoptopenjdk: Some("https://my-mirror.example.com/adoptium".to_owned()),
        };

        let fetcher = VersionFetcher::new(reqwest::Client::new(), sources);

        assert_eq!(
            fetcher.adoptium_base_url(),
            "https://my-mirror.example.com/adoptium"
        );
    }

    #[test]
    fn uses_default_adoptium_url() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());

        assert_eq!(fetcher.adoptium_base_url(), DEFAULT_ADOPTIUM_BASE_URL);
    }
}
