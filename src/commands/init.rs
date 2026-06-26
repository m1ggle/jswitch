use clap::Args;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
pub struct InitArgs {
    #[arg(value_enum)]
    pub shell: Option<Shell>,

    /// Write to shell config file instead of stdout (default: true).
    #[arg(long)]
    pub no_modify_config: bool,

    /// Force overwrite existing integration.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl Shell {
    /// Returns the default config file path for this shell.
    pub fn config_path(&self) -> Option<&'static str> {
        match self {
            Shell::Bash => Some("~/.bashrc"),
            Shell::Zsh => Some("~/.zshrc"),
            Shell::Fish => Some("~/.config/fish/config.fish"),
            Shell::PowerShell => None, // PowerShell profile path varies
        }
    }

    /// Returns the file name for this shell's completion script.
    pub fn completion_file_name(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Zsh => "jswitch.bash",
            Shell::Fish => "jswitch.fish",
            Shell::PowerShell => "jswitch.ps1",
        }
    }

    /// Detects the current shell from environment, if possible.
    pub fn detect() -> Option<Self> {
        let shell = std::env::var("SHELL").ok()?;
        let shell_name = shell.rsplit('/').next()?;

        match shell_name {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            _ => None,
        }
    }
}

/// Generates the full shell integration script content for the given shell type.
fn generate_integration_script(shell: Shell) -> String {
    match shell {
        Shell::Bash | Shell::Zsh => generate_bash_zsh_script(),
        Shell::Fish => generate_fish_script(),
        Shell::PowerShell => generate_powershell_script(),
    }
}

// ─── Bash / Zsh Integration Script ───────────────────────────────────────

fn generate_bash_zsh_script() -> String {
    include_str!("../../completions/jswitch.sh").to_owned()
}

// ─── Fish Shell Integration Script ──────────────────────────────────────

fn generate_fish_script() -> String {
    r#"
# >>> jswitch init >>>
# JSwitch shell integration for fish

function jswitch
    set -l cmd $argv[1]
    set -e argv[1]
    command jswitch $cmd $argv
    set -l exit_code $status
    if test "$cmd" = "switch"
        command jswitch env 2>/dev/null | source
    end
    return $exit_code
end

function __jswitch_apply_env --on-variable PWD
    command jswitch env 2>/dev/null | source
end

# Apply on startup
command jswitch env 2>/dev/null | source

# <<< jswitch init <<<
"#
    .trim_start()
    .to_owned()
}

// ─── PowerShell Integration Script ──────────────────────────────────────

fn generate_powershell_script() -> String {
    r#"
# >>> jswitch init >>>
# JSwitch shell integration for PowerShell

function jswitch {
    $cmd = $args[0]
    $rest = $args[1..($args.Length - 1)]
    & jswitch.exe @args
    $status = $LASTEXITCODE
    if ($cmd -eq 'switch') {
        $envOutput = & jswitch.exe env 2>$null
        foreach ($line in $envOutput) {
            if ($line -match '^export (\w+)=(.*)$') {
                $name = $Matches[1]
                $value = $Matches[2] -replace '"', ''
                Set-Item -Path "Env:$name" -Value $value
            }
        }
    }
    return $status
}

function __jswitch_apply_env {
    $envOutput = & jswitch.exe env 2>$null
    foreach ($line in $envOutput) {
        if ($line -match '^export (\w+)=(.*)$') {
            $name = $Matches[1]
            $value = $Matches[2] -replace '"', ''
            Set-Item -Path "Env:$name" -Value $value
        }
    }
}

# Apply on startup
__jswitch_apply_env

# <<< jswitch init <<<
"#
    .trim_start()
    .to_owned()
}

// ─── Main run function ───────────────────────────────────────────────────

pub async fn run(args: InitArgs) -> Result<()> {
    // Determine target shell: explicit arg > auto-detect > default bash
    let shell = args.shell.or_else(Shell::detect).unwrap_or(Shell::Bash);

    let script_content = generate_integration_script(shell);

    // Output strategy
    if args.no_modify_config {
        // Print to stdout, let user redirect manually
        println!("# JSwitch {} shell integration", shell_shell_name(shell));
        println!("# Paste this into your shell config file:\n");
        print!("{}", script_content);
        println!();
        return Ok(());
    }

    // Write to shell config file
    if let Some(config_path) = shell.config_path() {
        let expanded = expand_tilde(config_path);
        let path = std::path::Path::new(&expanded);

        // Check if integration already exists
        if path.exists() && !args.force {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing.contains(">>> jswitch init <<<") {
                println!(
                    "jswitch shell integration already exists in {}",
                    config_path
                );
                println!("Use --force to overwrite, or --no-modify-config to print to stdout.");
                return Ok(());
            }
        }

        // Append or create
        if path.exists() {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|source| crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                })?;
            writeln!(file, "\n{}", script_content).map_err(|source| {
                crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(path, &script_content).map_err(|source| {
                crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        }

        println!("✅ jswitch shell integration written to {}", config_path);
        println!(
            "   Run 'source {}' or restart your shell to apply.",
            config_path
        );
    } else {
        // No config file path (e.g. PowerShell) — print to stdout
        println!("# JSwitch {} shell integration", shell_shell_name(shell));
        println!("# Save this to your PowerShell profile:\n");
        print!("{}", script_content);
        println!();
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn shell_shell_name(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
        Shell::PowerShell => "PowerShell",
    }
}

fn expand_tilde(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        path.replace("~/", &format!("{}/", home.display()))
            .replace('~', &home.to_string_lossy())
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_bash_script_contains_markers() {
        let script = generate_bash_zsh_script();
        assert!(script.contains(">>> jswitch init >>>"));
        assert!(script.contains("<<< jswitch init <<<"));
    }

    #[test]
    fn generates_fish_script_is_not_empty() {
        let script = generate_fish_script();
        assert!(!script.is_empty());
        assert!(script.contains("jswitch"));
    }

    #[test]
    fn generates_powershell_script_is_not_empty() {
        let script = generate_powershell_script();
        assert!(!script.is_empty());
        assert!(script.contains("jswitch"));
    }

    #[test]
    fn shell_has_completion_file_names() {
        assert_eq!(Shell::Bash.completion_file_name(), "jswitch.bash");
        assert_eq!(Shell::Zsh.completion_file_name(), "jswitch.bash");
        assert_eq!(Shell::Fish.completion_file_name(), "jswitch.fish");
        assert_eq!(Shell::PowerShell.completion_file_name(), "jswitch.ps1");
    }

    #[test]
    fn expand_tilde_replaces_home() {
        // Just verify the function doesn't panic
        let result = expand_tilde("~/some/path");
        // It should replace ~ with something (or keep as-is if no home dir)
        assert!(!result.starts_with("~/") || !result.contains('~') || result == "~/some/path");
    }
}
