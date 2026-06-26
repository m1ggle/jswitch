use super::Shell;

/// Generate the full shell integration script for `jswitch init <shell>`.
///
/// The script:
/// 1. Defines a `jswitch` wrapper function that auto-evals session exports.
/// 2. Installs a `cd` hook that auto-switches when a `.java-version` file is found.
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
    local output
    output=$(command jswitch "$@")
    local status=$?
    if [[ "$output" == "export "* ]]; then
        eval "$output"
    else
        printf '%s\n' "$output"
    fi
    return $status
}

__jswitch_auto_switch() {
    if [[ -f ".java-version" ]]; then
        eval "$(command jswitch switch "$(cat .java-version)" --session 2>/dev/null)"
    fi
}

__jswitch_prev_dir="$PWD"
__jswitch_cd() {
    builtin cd "$@" || return $?
    if [[ "$PWD" != "$__jswitch_prev_dir" ]]; then
        __jswitch_auto_switch
    fi
    __jswitch_prev_dir="$PWD"
}

alias cd='__jswitch_cd'
"#
    .to_owned()
}

fn zsh_script() -> String {
    r#"# jswitch shell integration for zsh
jswitch() {
    local output
    output=$(command jswitch "$@")
    local status=$?
    if [[ "$output" == "export "* ]]; then
        eval "$output"
    else
        printf '%s\n' "$output"
    fi
    return $status
}

__jswitch_auto_switch() {
    if [[ -f ".java-version" ]]; then
        eval "$(command jswitch switch "$(cat .java-version)" --session 2>/dev/null)"
    fi
}

chpwd_functions+=(__jswitch_auto_switch)
"#
    .to_owned()
}

fn fish_script() -> String {
    r#"# jswitch shell integration for fish
function jswitch
    set -l output (command jswitch $argv)
    if string match -q "set -gx *" -- $output
        eval $output
    else
        printf '%s\n' $output
    end
end

function __jswitch_auto_switch --on-variable PWD
    if test -f ".java-version"
        command jswitch switch (cat .java-version) --session 2>/dev/null | source
    end
end
"#
    .to_owned()
}

fn powershell_script() -> String {
    r#"# jswitch shell integration for PowerShell
function jswitch {
    $output = & jswitch.exe $args
    $status = $LASTEXITCODE
    if ($output -match '^\$env:') {
        Invoke-Expression ($output -join "`n")
    } else {
        $output
    }
    return $status
}

function Prompt {
    if (Test-Path ".java-version") {
        $version = Get-Content ".java-version" -ErrorAction SilentlyContinue
        if ($version) {
            & jswitch.exe switch $version --session 2>$null | Out-Null
        }
    }
    "PS > "
}
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
        assert!(script.contains("__jswitch_auto_switch"));
    }

    #[test]
    fn zsh_script_uses_chpwd_hook() {
        let script = init_script(Shell::Zsh);
        assert!(script.contains("jswitch()"));
        assert!(script.contains("chpwd_functions"));
    }

    #[test]
    fn fish_script_has_jswitch_function() {
        let script = init_script(Shell::Fish);
        assert!(script.contains("function jswitch"));
        assert!(script.contains("__jswitch_auto_switch"));
    }

    #[test]
    fn powershell_script_has_jswitch_function() {
        let script = init_script(Shell::PowerShell);
        assert!(script.contains("function jswitch"));
    }
}
