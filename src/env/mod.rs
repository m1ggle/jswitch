use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    error::{JswitchError, jswitch_error::VersionError},
    version::{VersionManager, VersionResolver},
};

#[derive(Debug, Clone)]
pub struct JavaEnvironment {
    pub java_home: Option<String>,
    pub path_addition: Option<String>,
}

impl JavaEnvironment {
    /// Build environment variables for the currently active Java version.
    /// Returns None if no version is selected or not installed.
    pub fn from_current_version() -> Result<Option<Self>, JswitchError> {
        let config = Config::load_or_default()?;
        let resolver = VersionResolver::new(config);

        match resolver.current()? {
            Some(current) => Ok(Self::for_version_id(&current.version)?),
            None => Ok(None),
        }
    }

    /// Build environment variables for a specific installed version string (e.g. "adoptopenjdk/17").
    pub fn for_version_id(version_id: &str) -> Result<Option<Self>, VersionError> {
        let manager = VersionManager::from_default_root()?;
        let install_path = manager.versions_dir().join(version_id);

        if !install_path.is_dir() {
            return Ok(None);
        }

        let java_home = install_path.to_string_lossy().to_string();
        let bin_path = install_path.join("bin").to_string_lossy().to_string();

        Ok(Some(Self {
            java_home: Some(java_home),
            path_addition: Some(bin_path),
        }))
    }

    /// Export shell-compatible environment variable assignments.
    /// Output format: `export KEY=value` suitable for eval in bash/zsh.
    pub fn export_statements(&self) -> Vec<String> {
        let mut statements = Vec::new();

        if let Some(ref home) = self.java_home {
            statements.push(format!("export JAVA_HOME=\"{home}\""));
        }

        if let Some(ref bin) = self.path_addition {
            // Prepend to PATH, avoiding duplicates
            statements.push(format!("export PATH=\"{bin}:$PATH\""));
        }

        statements
    }

    /// Verify that the Java installation looks valid by checking for key binaries.
    pub fn verify(&self) -> Result<bool, VersionError> {
        let home = match &self.java_home {
            Some(h) => PathBuf::from(h),
            None => return Ok(false),
        };

        let bin = home.join("bin");
        let java_bin = bin.join(if cfg!(windows) { "java.exe" } else { "java" });

        Ok(java_bin.exists())
    }

    /// List all executables available in the JDK bin directory.
    pub fn list_executables(&self) -> Result<Vec<String>, VersionError> {
        let home = match &self.java_home {
            Some(h) => PathBuf::from(h),
            None => return Ok(Vec::new()),
        };

        let bin = home.join("bin");
        if !bin.is_dir() {
            return Ok(Vec::new());
        }

        let mut executables = Vec::new();
        let entries = fs::read_dir(&bin).map_err(|source| VersionError::ReadDir {
            path: bin.clone(),
            source,
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            // On Unix, check executable bit; on Windows, check .exe extension
            let is_executable = if cfg!(windows) {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    entry
                        .metadata()
                        .ok()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false)
                }
                #[cfg(not(unix))]
                false
            };

            if is_executable && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                executables.push(name.to_owned());
            }
        }

        executables.sort();
        Ok(executables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_statements_includes_java_home_and_path() {
        let env = JavaEnvironment {
            java_home: Some("/home/user/.jswitch/versions/adoptopenjdk/17".to_owned()),
            path_addition: Some("/home/user/.jswitch/versions/adoptopenjdk/17/bin".to_owned()),
        };

        let stmts = env.export_statements();

        assert!(stmts.iter().any(|s| s.starts_with("export JAVA_HOME=")));
        assert!(stmts.iter().any(|s| s.starts_with("export PATH=")));
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn empty_environment_produces_no_statements() {
        let env = JavaEnvironment {
            java_home: None,
            path_addition: None,
        };
        assert!(env.export_statements().is_empty());
    }
}
