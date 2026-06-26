/// Supported shells for environment integration.
#[derive(
    Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl Shell {
    /// Produce a shell-specific export statement for a single variable.
    pub fn export(&self, var: &str, value: &str) -> String {
        match self {
            Shell::Bash | Shell::Zsh => format!("export {var}=\"{value}\""),
            Shell::Fish => format!("set -gx {var} \"{value}\""),
            Shell::PowerShell => format!("$env:{var} = \"{value}\""),
        }
    }

    /// Produce a shell-specific statement that prepends `dir` to a path-style variable.
    pub fn prepend_path(&self, var: &str, dir: &str) -> String {
        match self {
            Shell::Bash | Shell::Zsh => format!("export {var}=\"{dir}:${var}\""),
            Shell::Fish => format!("set -gx {var} \"{dir}\" ${var}"),
            Shell::PowerShell => format!("$env:{var} = \"{dir};$env:{var}\""),
        }
    }

    /// Return the file extension for the init script of this shell.
    pub fn init_suffix(&self) -> &'static str {
        match self {
            Shell::Bash => "sh",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "ps1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_export_format() {
        assert_eq!(
            Shell::Bash.export("JAVA_HOME", "/path/to/jdk"),
            "export JAVA_HOME=\"/path/to/jdk\""
        );
    }

    #[test]
    fn fish_export_format() {
        assert_eq!(
            Shell::Fish.export("JAVA_HOME", "/path/to/jdk"),
            "set -gx JAVA_HOME \"/path/to/jdk\""
        );
    }

    #[test]
    fn powershell_export_format() {
        assert_eq!(
            Shell::PowerShell.export("JAVA_HOME", "/path/to/jdk"),
            "$env:JAVA_HOME = \"/path/to/jdk\""
        );
    }

    #[test]
    fn bash_prepend_path() {
        assert_eq!(
            Shell::Bash.prepend_path("PATH", "/jdk/bin"),
            "export PATH=\"/jdk/bin:$PATH\""
        );
    }

    #[test]
    fn fish_prepend_path() {
        assert_eq!(
            Shell::Fish.prepend_path("PATH", "/jdk/bin"),
            "set -gx PATH \"/jdk/bin\" $PATH"
        );
    }
}
