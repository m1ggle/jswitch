use std::process::Command;

use crate::{
    Result,
    config::{Config, manager::default_config_path},
    version::{VersionManager, VersionResolver},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorCheck {
    name: &'static str,
    status: CheckStatus,
    message: String,
}

pub async fn run() -> Result<()> {
    let mut checks = Vec::new();
    checks.push(check_config());

    let manager = VersionManager::from_default_root()?;
    checks.push(check_versions_dir(&manager));
    checks.push(check_installed_versions(&manager)?);
    checks.push(check_current_version(&manager)?);
    checks.push(check_java_on_path());

    for check in &checks {
        println!(
            "{} {} - {}",
            status_label(check.status),
            check.name,
            check.message
        );
    }

    let failed = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Warn)
        .count();

    println!("doctor summary: {failed} failed, {warnings} warnings");
    Ok(())
}

fn check_config() -> DoctorCheck {
    match Config::load_or_default() {
        Ok(_) => DoctorCheck::ok("config", "configuration is readable"),
        Err(error) => {
            let path = default_config_path()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "~/.jswitch/config.toml".to_owned());
            DoctorCheck::fail("config", format!("failed to read {path}: {error}"))
        }
    }
}

fn check_versions_dir(manager: &VersionManager) -> DoctorCheck {
    let path = manager.versions_dir();
    if path.is_dir() {
        DoctorCheck::ok("versions-dir", format!("{} exists", path.display()))
    } else if manager.root_dir().exists() {
        DoctorCheck::warn(
            "versions-dir",
            format!("{} does not exist yet", path.display()),
        )
    } else {
        DoctorCheck::warn(
            "versions-dir",
            format!("{} will be created on first install", path.display()),
        )
    }
}

fn check_installed_versions(manager: &VersionManager) -> Result<DoctorCheck> {
    let versions = manager.list_installed()?;
    if versions.is_empty() {
        Ok(DoctorCheck::warn(
            "installed-versions",
            "no Java versions installed".to_owned(),
        ))
    } else {
        Ok(DoctorCheck::ok(
            "installed-versions",
            format!("{} versions installed", versions.len()),
        ))
    }
}

fn check_current_version(manager: &VersionManager) -> Result<DoctorCheck> {
    let config = Config::load_or_default()?;
    let resolver = VersionResolver::new(config);

    match resolver.current()? {
        Some(current) if manager.is_installed(&current.version) => Ok(DoctorCheck::ok(
            "current-version",
            format!("{} is active", current.version),
        )),
        Some(current) => Ok(DoctorCheck::warn(
            "current-version",
            format!("{} is selected but not installed", current.version),
        )),
        None => Ok(DoctorCheck::warn(
            "current-version",
            "no Java version selected".to_owned(),
        )),
    }
}

fn check_java_on_path() -> DoctorCheck {
    match Command::new("java").arg("-version").output() {
        Ok(output) if output.status.success() => {
            DoctorCheck::ok("java", parse_java_version(&output.stderr))
        }
        Ok(output) => DoctorCheck::warn(
            "java",
            format!("java -version exited with status {}", output.status),
        ),
        Err(error) => DoctorCheck::warn("java", format!("java not available on PATH: {error}")),
    }
}

fn parse_java_version(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .next()
        .map(str::to_owned)
        .unwrap_or_else(|| "java is available on PATH".to_owned())
}

fn status_label(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Ok => "[ok]",
        CheckStatus::Warn => "[warn]",
        CheckStatus::Fail => "[fail]",
    }
}

impl DoctorCheck {
    fn ok(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Ok,
            message: message.into(),
        }
    }

    fn warn(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Warn,
            message: message.into(),
        }
    }

    fn fail(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Fail,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels_are_stable() {
        assert_eq!(status_label(CheckStatus::Ok), "[ok]");
        assert_eq!(status_label(CheckStatus::Warn), "[warn]");
        assert_eq!(status_label(CheckStatus::Fail), "[fail]");
    }

    #[test]
    fn parses_java_version_first_line() {
        let line = parse_java_version(b"java version \"17.0.10\"\nother output");

        assert_eq!(line, "java version \"17.0.10\"");
    }
}
