use serde::Deserialize;

use crate::{
    commands::install::JavaSource, config::global::SourcesConfig,
    error::jswitch_error::NetworkError,
};

const DEFAULT_ADOPTIUM_BASE_URL: &str = "https://api.adoptium.net/v3/assets/feature_releases";
const DEFAULT_CORRETTO_BASE_URL: &str = "https://corretto.aws/downloads/latest";
const DEFAULT_CORRETTO_CHECKSUM_BASE_URL: &str = "https://corretto.aws/downloads/latest_sha256";
const DEFAULT_ORACLE_BASE_URL: &str = "https://download.oracle.com/java";
const DEFAULT_OPENJDK_BASE_URL: &str = "https://download.java.net/java/GA";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteVersion {
    pub version: String,
    pub source: JavaSource,
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

    fn corretto_base_url(&self) -> &str {
        self.sources
            .corretto
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_CORRETTO_BASE_URL)
    }

    fn corretto_checksum_base_url(&self) -> &str {
        self.sources
            .corretto_checksum
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_CORRETTO_CHECKSUM_BASE_URL)
    }

    fn oracle_base_url(&self) -> &str {
        self.sources
            .oracle
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_ORACLE_BASE_URL)
    }

    fn openjdk_base_url(&self) -> &str {
        self.sources
            .openjdk
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_OPENJDK_BASE_URL)
    }

    pub async fn fetch(
        &self,
        requested: &str,
        source: JavaSource,
    ) -> Result<RemoteVersion, NetworkError> {
        let major = resolve_feature_version(requested)?;

        match source {
            JavaSource::Corretto => Ok(self.corretto_remote_version(major)),
            JavaSource::Oracle => Ok(self.oracle_remote_version(major)),
            JavaSource::OpenJdk => Ok(self.openjdk_remote_version(major)),
            JavaSource::Adoptopenjdk => self.fetch_adoptium(major, source).await,
        }
    }

    // --- Corretto: direct URL construction (no API call needed) ---

    fn corretto_remote_version(&self, major: u32) -> RemoteVersion {
        let archive_name = corretto_archive_name(major);
        let base = self.corretto_base_url();
        let download_url = format!("{base}/{archive_name}");
        let checksum_base = self.corretto_checksum_base_url();
        let checksum_url = Some(format!("{checksum_base}/{archive_name}"));

        RemoteVersion {
            version: major.to_string(),
            source: JavaSource::Corretto,
            archive_name,
            download_url,
            checksum_url,
        }
    }

    // --- Oracle: direct URL, no checksum endpoint available ---

    fn oracle_remote_version(&self, major: u32) -> RemoteVersion {
        let archive_name = oracle_archive_name(major);
        let base = self.oracle_base_url();
        let download_url = format!("{base}/{major}/latest/{archive_name}");

        RemoteVersion {
            version: major.to_string(),
            source: JavaSource::Oracle,
            archive_name,
            download_url,
            // Oracle does not expose a standalone checksum endpoint.
            checksum_url: None,
        }
    }

    // --- OpenJDK (download.java.net): direct URL, no checksum endpoint ---

    fn openjdk_remote_version(&self, major: u32) -> RemoteVersion {
        let archive_name = openjdk_archive_name(major);
        let base = self.openjdk_base_url();
        let download_url = format!("{base}/jdk{major}/latest/GPL/{archive_name}");

        RemoteVersion {
            version: major.to_string(),
            source: JavaSource::OpenJdk,
            archive_name,
            download_url,
            checksum_url: None,
        }
    }

    // --- Adoptium (Eclipse Temurin): API-based discovery ---

    async fn fetch_adoptium(
        &self,
        major: u32,
        source: JavaSource,
    ) -> Result<RemoteVersion, NetworkError> {
        let image_type = "jdk";
        let base = self.adoptium_base_url();
        let url = format!(
            "{base}/{major}/ga?architecture={}&heap_size=normal&image_type={image_type}&jvm_impl=hotspot&os={}&page_size=1&project=jdk&sort_method=DEFAULT&sort_order=DESC&vendor={}",
            architecture(),
            operating_system(),
            vendor(source),
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
            .find_map(|asset| asset.into_remote_version(source))
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

// --- Archive name builders ---

fn corretto_archive_name(major: u32) -> String {
    let extension = if std::env::consts::OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    format!(
        "amazon-corretto-{major}-{}-{}-jdk.{extension}",
        architecture(),
        corretto_operating_system(),
    )
}

fn oracle_archive_name(major: u32) -> String {
    let extension = if std::env::consts::OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    format!(
        "jdk-{major}_{}-{}_bin.{extension}",
        operating_system_for_archive(),
        architecture(),
    )
}

fn openjdk_archive_name(major: u32) -> String {
    let extension = if std::env::consts::OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };
    format!(
        "openjdk-{major}_{}-{}_bin.{extension}",
        operating_system_for_archive(),
        architecture(),
    )
}

// --- Platform helpers ---

fn vendor(source: JavaSource) -> &'static str {
    match source {
        JavaSource::OpenJdk | JavaSource::Adoptopenjdk => "eclipse",
        JavaSource::Corretto => "amazon",
        JavaSource::Oracle => "oracle",
    }
}

fn operating_system() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mac",
        "windows" => "windows",
        _ => "linux",
    }
}

/// OS name as used in archive file names (e.g. `jdk-17_macos-aarch64_bin.tar.gz`).
fn operating_system_for_archive() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    }
}

fn corretto_operating_system() -> &'static str {
    operating_system_for_archive()
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
    fn into_remote_version(self, source: JavaSource) -> Option<RemoteVersion> {
        let package = self.binary?.package;
        Some(RemoteVersion {
            version: self.version_data.semver,
            source,
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
    fn builds_corretto_remote_version_from_default_urls() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());
        let remote = fetcher.corretto_remote_version(17);

        assert_eq!(remote.version, "17");
        assert_eq!(remote.source, JavaSource::Corretto);
        assert!(remote.archive_name.starts_with("amazon-corretto-17-"));
        assert!(remote.download_url.starts_with(DEFAULT_CORRETTO_BASE_URL));
        assert_eq!(
            remote.checksum_url,
            Some(format!(
                "{DEFAULT_CORRETTO_CHECKSUM_BASE_URL}/{}",
                remote.archive_name
            ))
        );
    }

    #[test]
    fn builds_oracle_remote_version_without_checksum() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());
        let remote = fetcher.oracle_remote_version(17);

        assert_eq!(remote.version, "17");
        assert_eq!(remote.source, JavaSource::Oracle);
        assert!(remote.archive_name.starts_with("jdk-17_"));
        assert!(remote.download_url.starts_with(DEFAULT_ORACLE_BASE_URL));
        assert!(remote.download_url.contains("/17/latest/"));
        assert!(remote.checksum_url.is_none());
    }

    #[test]
    fn builds_openjdk_remote_version_without_checksum() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());
        let remote = fetcher.openjdk_remote_version(21);

        assert_eq!(remote.version, "21");
        assert_eq!(remote.source, JavaSource::OpenJdk);
        assert!(remote.archive_name.starts_with("openjdk-21_"));
        assert!(
            remote
                .download_url
                .starts_with("https://download.java.net/java/GA/jdk21/latest/GPL/")
        );
        assert!(remote.download_url.contains("/jdk21/latest/"));
        assert!(remote.checksum_url.is_none());
    }

    #[test]
    fn builds_corretto_eight_url_for_legacy_java_version() {
        let fetcher = VersionFetcher::new(reqwest::Client::new(), SourcesConfig::default());
        let major = resolve_feature_version("1.8").unwrap();
        let remote = fetcher.corretto_remote_version(major);

        assert_eq!(remote.version, "8");
        assert!(remote.archive_name.starts_with("amazon-corretto-8-"));
        assert!(remote.download_url.contains("amazon-corretto-8-"));
    }

    #[test]
    fn uses_config_overridden_urls() {
        let sources = SourcesConfig {
            corretto: Some("https://my-mirror.example.com/corretto".to_owned()),
            corretto_checksum: Some("https://my-mirror.example.com/corretto-checksums".to_owned()),
            oracle: Some("https://my-mirror.example.com/oracle".to_owned()),
            openjdk: Some("https://my-mirror.example.com/openjdk".to_owned()),
            adoptopenjdk: Some("https://my-mirror.example.com/adoptium".to_owned()),
        };

        let fetcher = VersionFetcher::new(reqwest::Client::new(), sources);

        let corretto = fetcher.corretto_remote_version(17);
        assert!(
            corretto
                .download_url
                .starts_with("https://my-mirror.example.com/corretto/")
        );
        assert!(
            corretto
                .checksum_url
                .unwrap()
                .starts_with("https://my-mirror.example.com/corretto-checksums/")
        );

        let oracle = fetcher.oracle_remote_version(17);
        assert!(
            oracle
                .download_url
                .starts_with("https://my-mirror.example.com/oracle/17/latest/")
        );

        let openjdk = fetcher.openjdk_remote_version(21);
        assert!(
            openjdk
                .download_url
                .starts_with("https://my-mirror.example.com/openjdk/jdk21/latest/GPL/")
        );

        assert_eq!(
            fetcher.adoptium_base_url(),
            "https://my-mirror.example.com/adoptium"
        );
    }
}
