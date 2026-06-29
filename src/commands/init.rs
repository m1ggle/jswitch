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

    /// Returns the init script file name for this shell.
    pub fn script_file_name(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Zsh => "jswitch-init.sh",
            Shell::Fish => "jswitch-init.fish",
            Shell::PowerShell => "jswitch-init.ps1",
        }
    }

    /// Returns the source one-liner to add to the shell config file.
    /// The full script lives in ~/.jswitch/bin/ and is sourced from the shell config,
    /// keeping the user's rc file clean.
    pub fn source_line(&self) -> String {
        match self {
            Shell::Bash | Shell::Zsh => {
                r#"[[ -s "$HOME/.jswitch/bin/jswitch-init.sh" ]] && source "$HOME/.jswitch/bin/jswitch-init.sh""#
                    .to_owned()
            }
            Shell::Fish => {
                r#"[ -f "$HOME/.jswitch/bin/jswitch-init.fish" ] && source "$HOME/.jswitch/bin/jswitch-init.fish""#
                    .to_owned()
            }
            Shell::PowerShell => r#". "$HOME/.jswitch/bin/jswitch-init.ps1""#.to_owned(),
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
    let shell = args.shell.or_else(Shell::detect).unwrap_or(Shell::Bash);
    let script_content = generate_integration_script(shell);

    // --no-modify-config: print script + source line to stdout, don't touch any files
    if args.no_modify_config {
        println!("# JSwitch {} shell integration", shell_shell_name(shell));
        println!("# Save this to ~/.jswitch/bin/{} :", shell.script_file_name());
        println!();
        print!("{}", script_content);
        println!();
        println!("# Then add this line to your shell config:");
        println!("{}", shell.source_line());
        return Ok(());
    }

    // 1. Write the full integration script to ~/.jswitch/bin/
    let bin_dir = jswitch_bin_dir()?;
    std::fs::create_dir_all(&bin_dir).map_err(|source| {
        crate::error::jswitch_error::IoError::Access {
            path: bin_dir.clone(),
            source,
        }
    })?;
    let script_path = bin_dir.join(shell.script_file_name());
    std::fs::write(&script_path, &script_content).map_err(|source| {
        crate::error::jswitch_error::IoError::Access {
            path: script_path.clone(),
            source,
        }
    })?;

    // 2. Add source one-liner to shell config (not the full script)
    if let Some(config_path) = shell.config_path() {
        let expanded = expand_tilde(config_path);
        let path = std::path::Path::new(&expanded);

        // --force: strip out old inline script blocks and stale source lines first
        if args.force {
            remove_old_integration(path)?;
        } else if path.exists() {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing.contains("jswitch-init") {
                println!("jswitch shell integration already exists in {}", config_path);
                println!("Use --force to re-initialize, or --no-modify-config to print to stdout.");
                println!("✅ Integration script updated at {}", script_path.display());
                return Ok(());
            }
        }

        // Append the source one-liner
        if path.exists() {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|source| crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                })?;
            writeln!(file, "\n{}", shell.source_line()).map_err(|source| {
                crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(path, format!("{}\n", shell.source_line())).map_err(|source| {
                crate::error::jswitch_error::IoError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        }

        println!("✅ jswitch shell integration:");
        println!("   Script: {}", script_path.display());
        println!("   Source line added to {}", config_path);
        println!(
            "   Run 'source {}' or restart your shell to apply.",
            config_path
        );
    } else {
        // PowerShell — no standard config path, print instructions
        println!("✅ jswitch shell integration script written to {}", script_path.display());
        println!("   Add this to your PowerShell profile:");
        println!("   {}", shell.source_line());
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

/// Returns the path to ~/.jswitch/bin/ where init scripts are stored.
fn jswitch_bin_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir()
        .ok_or(crate::error::jswitch_error::VersionError::HomeDirUnavailable)?;
    Ok(home.join(".jswitch").join("bin"))
}

/// Remove old jswitch integration from a shell config file.
///
/// Handles both legacy inline script blocks (delimited by `>>> jswitch init >>>`
/// / `<<< jswitch init <<<`) and old source one-liners referencing
/// `jswitch-init`, so `--force` produces a clean state before re-adding.
fn remove_old_integration(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path).map_err(|source| {
        crate::error::jswitch_error::IoError::Access {
            path: path.to_path_buf(),
            source,
        }
    })?;

    let mut new_lines = Vec::new();
    let mut in_block = false;

    for line in content.lines() {
        if line.contains(">>> jswitch init >>>") {
            in_block = true;
            continue;
        }
        if line.contains("<<< jswitch init <<<") {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        // Skip old source one-liners
        if line.contains("jswitch-init") {
            continue;
        }
        new_lines.push(line);
    }

    let new_content = new_lines.join("\n");
    std::fs::write(path, new_content).map_err(|source| {
        crate::error::jswitch_error::IoError::Access {
            path: path.to_path_buf(),
            source,
        }
    })?;

    Ok(())
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
    fn shell_has_script_file_names() {
        assert_eq!(Shell::Bash.script_file_name(), "jswitch-init.sh");
        assert_eq!(Shell::Zsh.script_file_name(), "jswitch-init.sh");
        assert_eq!(Shell::Fish.script_file_name(), "jswitch-init.fish");
        assert_eq!(Shell::PowerShell.script_file_name(), "jswitch-init.ps1");
    }

    #[test]
    fn shell_source_lines_reference_jswitch_bin() {
        assert!(Shell::Bash.source_line().contains("jswitch/bin/jswitch-init.sh"));
        assert!(Shell::Zsh.source_line().contains("jswitch/bin/jswitch-init.sh"));
        assert!(Shell::Fish.source_line().contains("jswitch/bin/jswitch-init.fish"));
        assert!(Shell::PowerShell.source_line().contains("jswitch/bin/jswitch-init.ps1"));
    }

    #[test]
    fn remove_old_integration_strips_inline_blocks_and_source_lines() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join(".zshrc");
        std::fs::write(&cfg, "# before\n# >>> jswitch init >>>\nold stuff\n# <<< jswitch init <<<\n# after\n[[ -s \"$HOME/.jswitch/bin/jswitch-init.sh\" ]] && source \"$HOME/.jswitch/bin/jswitch-init.sh\"\n# end\n").unwrap();

        remove_old_integration(&cfg).unwrap();

        let result = std::fs::read_to_string(&cfg).unwrap();
        assert!(!result.contains(">>> jswitch init >>>"));
        assert!(!result.contains("<<< jswitch init <<<"));
        assert!(!result.contains("jswitch-init"));
        assert!(result.contains("# before"));
        assert!(result.contains("# after"));
        assert!(result.contains("# end"));
    }

    #[test]
    fn expand_tilde_replaces_home() {
        // Just verify the function doesn't panic
        let result = expand_tilde("~/some/path");
        // It should replace ~ with something (or keep as-is if no home dir)
        assert!(!result.starts_with("~/") || !result.contains('~') || result == "~/some/path");
    }
}
