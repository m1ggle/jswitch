use super::Shell;

/// Generate the full shell integration script for `jswitch init <shell>`.
///
/// The script:
/// 1. Defines a `jswitch` wrapper function that re-applies env after `switch`.
/// 2. Installs a hook that re-applies the global Java version on directory change.
/// 3. Applies the global version on shell startup.
pub fn init_script(shell: Shell) -> String {
    match shell {
        Shell::Bash => bash_script(),
        Shell::Zsh => zsh_script(),
        Shell::Fish => fish_script(),
        Shell::PowerShell => powershell_script(),
    }
}

fn bash_script() -> String {
    r#"# jswitch shell integration for bash
jswitch() {
    local cmd="${1:-}"; shift || true
    command jswitch "$cmd" "$@"
    local exit_code=$?
    if [[ "$cmd" == "switch" ]]; then
        eval "$(command jswitch env 2>/dev/null)"
    fi
    return $exit_code
}

__jswitch_apply_env() {
    eval "$(command jswitch env 2>/dev/null)"
}

if [[ -n "${PROMPT_COMMAND:-}" ]]; then
    export PROMPT_COMMAND="__jswitch_apply_env;${PROMPT_COMMAND}"
else
    export PROMPT_COMMAND="__jswitch_apply_env"
fi

__jswitch_apply_env
"#
    .to_owned()
}

fn zsh_script() -> String {
    r#"# jswitch shell integration for zsh
jswitch() {
    local cmd="${1:-}"; shift || true
    command jswitch "$cmd" "$@"
    local exit_code=$?
    if [[ "$cmd" == "switch" ]]; then
        eval "$(command jswitch env 2>/dev/null)"
    fi
    return $exit_code
}

__jswitch_apply_env() {
    eval "$(command jswitch env 2>/dev/null)"
}

autoload -Uz add-zsh-hook 2>/dev/null
add-zsh-hook chpwd __jswitch_apply_env 2>/dev/null || true

__jswitch_apply_env
"#
    .to_owned()
}

fn fish_script() -> String {
    r#"# jswitch shell integration for fish
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
"#
    .to_owned()
}

fn powershell_script() -> String {
    r#"# jswitch shell integration for PowerShell
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
"#
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_script_has_jswitch_function() {
        let script = init_script(Shell::Bash);
        assert!(script.contains("jswitch()"));
        assert!(script.contains("__jswitch_apply_env"));
    }

    #[test]
    fn zsh_script_uses_chpwd_hook() {
        let script = init_script(Shell::Zsh);
        assert!(script.contains("jswitch()"));
        assert!(script.contains("add-zsh-hook chpwd"));
    }

    #[test]
    fn fish_script_has_jswitch_function() {
        let script = init_script(Shell::Fish);
        assert!(script.contains("function jswitch"));
        assert!(script.contains("__jswitch_apply_env"));
    }

    #[test]
    fn powershell_script_has_jswitch_function() {
        let script = init_script(Shell::PowerShell);
        assert!(script.contains("function jswitch"));
    }
}
