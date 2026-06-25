#[derive(Debug, Clone)]
pub struct DownloadClient {
    inner: reqwest::Client,
}

impl DownloadClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
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

pub mod fetcher;
pub mod installer;
pub mod progress;
pub mod verifier;

pub use fetcher::{RemoteVersion, VersionFetcher};
pub use installer::JavaInstaller;
pub use verifier::verify_sha256;
