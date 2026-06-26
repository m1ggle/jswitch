use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    error::{JswitchError, jswitch_error::VersionError},
    version::{VersionManager, VersionResolver},
};

use super::shell::Shell;

/// Represents the environment variables needed to use a specific Java installation.
#[derive(Debug, Clone)]
pub struct JavaEnvironment {
    pub java_home: Option<String>,
    pub path_addition: Option<String>,
}

impl JavaEnvironment {
    /// Build environment variables for the currently active Java version.
    /// Returns `None` if no version is selected or the selected version is not installed.
    pub fn from_current_version() -> Result<Option<Self>, JswitchError> {
        let config = Config::load_or_default()?;
        let resolver = VersionResolver::new(config);

        match resolver.current()? {
            Some(current) => Ok(Self::for_version_id(&current.version)?),
            None => Ok(None),
        }
    }

    /// Build environment variables for a specific installed version.
    ///
    /// Looks up `~/.jswitch/versions/<version>/` and returns `None` if not installed.
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

    /// Produce shell-specific export statements for `JAVA_HOME` and `PATH`.
    ///
    /// Returns an empty `Vec` if no Java version is active.
    pub fn export_for_shell(&self, shell: Shell) -> Vec<String> {
        let mut statements = Vec::new();

        if let Some(ref home) = self.java_home {
            statements.push(shell.export("JAVA_HOME", home));
        }

        if let Some(ref bin) = self.path_addition {
            statements.push(shell.prepend_path("PATH", bin));
        }

        statements
    }

    /// Verify that the Java installation looks valid by checking for the
    /// `java` binary in its `bin` directory.
    pub fn verify(&self) -> Result<bool, VersionError> {
        let home = match &self.java_home {
            Some(h) => PathBuf::from(h),
            None => return Ok(false),
        };

        let java_bin = home
            .join("bin")
            .join(if cfg!(windows) { "java.exe" } else { "java" });

        Ok(java_bin.exists())
    }

    /// List all executables in the JDK `bin` directory, sorted alphabetically.
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
    fn bash_exports_java_home_and_path() {
        let env = JavaEnvironment {
            java_home: Some("/home/user/.jswitch/versions/17".to_owned()),
            path_addition: Some("/home/user/.jswitch/versions/17/bin".to_owned()),
        };

        let stmts = env.export_for_shell(Shell::Bash);
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            "export JAVA_HOME=\"/home/user/.jswitch/versions/17\""
        );
        assert_eq!(
            stmts[1],
            "export PATH=\"/home/user/.jswitch/versions/17/bin:$PATH\""
        );
    }

    #[test]
    fn fish_exports_java_home_and_path() {
        let env = JavaEnvironment {
            java_home: Some("/home/user/.jswitch/versions/17".to_owned()),
            path_addition: Some("/home/user/.jswitch/versions/17/bin".to_owned()),
        };

        let stmts = env.export_for_shell(Shell::Fish);
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0],
            "set -gx JAVA_HOME \"/home/user/.jswitch/versions/17\""
        );
        assert_eq!(
            stmts[1],
            "set -gx PATH \"/home/user/.jswitch/versions/17/bin\" $PATH"
        );
    }

    #[test]
    fn empty_environment_produces_no_statements() {
        let env = JavaEnvironment {
            java_home: None,
            path_addition: None,
        };
        assert!(env.export_for_shell(Shell::Bash).is_empty());
    }
}
