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

    /// Fetch all available GA releases from Adoptium.
    ///
    /// Calls the `/info/available_releases` endpoint to discover which major
    /// versions exist, then fetches the latest GA binary for each in parallel.
    /// Versions that fail to resolve are silently skipped.
    pub async fn list_remote(&self) -> Result<Vec<RemoteVersion>, NetworkError> {
        let releases = self.fetch_available_releases().await?;

        let futures: Vec<_> = releases
            .available_releases
            .into_iter()
            .rev() // newest major first
            .map(|major| self.fetch_adoptium(major))
            .collect();

        let results = futures_util::future::join_all(futures).await;
        Ok(results.into_iter().filter_map(|r| r.ok()).collect())
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

    /// Derive the `/info/available_releases` URL from the configured base URL.
    ///
    /// The base URL ends with `/v3/assets/feature_releases`; stripping that
    /// suffix yields the API root (`…/v3`) to which `/info/available_releases`
    /// is appended.
    fn adoptium_info_url(&self) -> String {
        let base = self.adoptium_base_url();
        let root = base
            .strip_suffix("/assets/feature_releases")
            .unwrap_or(base);
        format!("{root}/info/available_releases")
    }

    async fn fetch_available_releases(&self) -> Result<AvailableReleases, NetworkError> {
        let url = self.adoptium_info_url();
        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<AvailableReleases>()
            .await?)
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
    // The API returns a `binaries` array; some entries (e.g. source-only
    // releases) may have an empty array or binaries without packages.
    binaries: Vec<AdoptiumBinary>,
    version_data: AdoptiumVersionData,
}

impl AdoptiumAsset {
    fn into_remote_version(self) -> Option<RemoteVersion> {
        let package = self
            .binaries
            .into_iter()
            .find_map(|binary| binary.package)?;
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
    // Some binaries have only an `installer` and no `package` (e.g. .pkg on macOS).
    package: Option<AdoptiumPackage>,
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

/// Response from the `/info/available_releases` endpoint.
#[derive(Debug, Deserialize)]
struct AvailableReleases {
    available_releases: Vec<u32>,
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

    #[test]
    fn derives_info_url_from_default_base() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());

        assert_eq!(
            fetcher.adoptium_info_url(),
            "https://api.adoptium.net/v3/info/available_releases"
        );
    }

    #[test]
    fn derives_info_url_from_overridden_base() {
        let sources = SourcesConfig {
            adoptopenjdk: Some(
                "https://my-mirror.example.com/v3/assets/feature_releases".to_owned(),
            ),
        };
        let fetcher = VersionFetcher::new(reqwest::Client::new(), sources);

        assert_eq!(
            fetcher.adoptium_info_url(),
            "https://my-mirror.example.com/v3/info/available_releases"
        );
    }
}
