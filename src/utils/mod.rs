pub mod logging;

pub fn normalize_version(version: &str) -> String {
    version.trim().to_owned()
}
