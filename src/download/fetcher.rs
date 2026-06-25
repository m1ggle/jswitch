use serde::Deserialize;

use crate::{commands::install::JavaSource, error::jswitch_error::NetworkError};

use super::DownloadClient;

const ADOPTIUM_BASE_URL: &str = "https://api.adoptium.net/v3/assets/feature_releases";
const CORRETTO_BASE_URL: &str = "https://corretto.aws/downloads/latest";
const CORRETTO_CHECKSUM_BASE_URL: &str = "https://corretto.aws/downloads/latest_sha256";

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
    client: DownloadClient,
}

impl VersionFetcher {
    pub fn new(client: DownloadClient) -> Self {
        Self { client }
    }

    pub async fn fetch(
        &self,
        requested: &str,
        source: JavaSource,
    ) -> Result<RemoteVersion, NetworkError> {
        let major = resolve_feature_version(requested)?;

        if source == JavaSource::Corretto {
            return Ok(corretto_remote_version(major));
        }

        let image_type = match source {
            JavaSource::OpenJdk | JavaSource::Adoptopenjdk => "jdk",
            JavaSource::Corretto | JavaSource::Oracle => "jdk",
        };
        let url = format!(
            "{ADOPTIUM_BASE_URL}/{major}/ga?architecture={}&heap_size=normal&image_type={image_type}&jvm_impl=hotspot&os={}&page_size=1&project=jdk&sort_method=DEFAULT&sort_order=DESC&vendor={}",
            architecture(),
            operating_system(),
            vendor(source),
        );

        let assets = self
            .client
            .inner()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<AdoptiumAsset>>()
            .await?;

        assets
            .into_iter()
            .find_map(|asset| asset.into_remote_version(source))
            .ok_or_else(|| NetworkError::RemoteVersionNotFound(requested.to_owned()))
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

fn corretto_remote_version(major: u32) -> RemoteVersion {
    let archive_name = corretto_archive_name(major);
    RemoteVersion {
        version: major.to_string(),
        source: JavaSource::Corretto,
        archive_name: archive_name.clone(),
        download_url: format!("{CORRETTO_BASE_URL}/{archive_name}"),
        checksum_url: Some(format!("{CORRETTO_CHECKSUM_BASE_URL}/{archive_name}")),
    }
}

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

fn corretto_operating_system() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    }
}

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

fn architecture() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x64",
    }
}

#[derive(Debug, Deserialize)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
    version_data: AdoptiumVersionData,
}

impl AdoptiumAsset {
    fn into_remote_version(self, source: JavaSource) -> Option<RemoteVersion> {
        let package = self.binary.package;
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
    fn builds_corretto_remote_version_from_direct_urls() {
        let remote = corretto_remote_version(17);

        assert_eq!(remote.version, "17");
        assert_eq!(remote.source, JavaSource::Corretto);
        assert!(remote.archive_name.starts_with("amazon-corretto-17-"));
        assert!(remote.download_url.starts_with(CORRETTO_BASE_URL));
        assert_eq!(
            remote.checksum_url,
            Some(format!(
                "{CORRETTO_CHECKSUM_BASE_URL}/{}",
                remote.archive_name
            ))
        );
    }

    #[test]
    fn builds_corretto_eight_url_for_legacy_java_version() {
        let major = resolve_feature_version("1.8").unwrap();
        let remote = corretto_remote_version(major);

        assert_eq!(remote.version, "8");
        assert!(remote.archive_name.starts_with("amazon-corretto-8-"));
        assert!(remote.download_url.contains("amazon-corretto-8-"));
    }
}
