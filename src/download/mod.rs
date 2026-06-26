// DownloadClient has moved to the `network` module so that proxy and mirror
// resolution live alongside the HTTP client.  We re-export it here for
// backward compatibility with existing call sites that import from
// `download::DownloadClient`.
pub use crate::network::DownloadClient;

pub mod fetcher;
pub mod installer;
pub mod progress;
pub mod verifier;

pub use fetcher::{RemoteVersion, VersionFetcher};
pub use installer::JavaInstaller;
pub use verifier::verify_sha256;
