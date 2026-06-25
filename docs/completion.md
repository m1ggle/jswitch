# Shell Completion Usage

`jswitch completion` generates shell completion scripts from the CLI definition.

## Generate Completion Script

```bash
jswitch completion <shell>
```

Supported shells:

```text
bash
zsh
fish
powershell
elvish
```

## Zsh

```bash
mkdir -p ~/.zfunc
jswitch completion zsh > ~/.zfunc/_jswitch
```

Ensure `~/.zshrc` contains:

```bash
fpath=(~/.zfunc $fpath)
autoload -Uz compinit
compinit
```

Reload the shell:

```bash
source ~/.zshrc
```

## Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
jswitch completion bash > ~/.local/share/bash-completion/completions/jswitch
```

Reload completion for the current shell:

```bash
source ~/.local/share/bash-completion/completions/jswitch
```

## Fish

```bash
mkdir -p ~/.config/fish/completions
jswitch completion fish > ~/.config/fish/completions/jswitch.fish
```

Reload Fish:

```bash
exec fish
```

## PowerShell

```powershell
jswitch completion powershell > jswitch.ps1
```

Load it in the current session:

```powershell
. .\jswitch.ps1
```

For persistent usage, add the script content to your PowerShell profile:

```powershell
notepad $PROFILE
```

## Elvish

```bash
mkdir -p ~/.elvish/lib
jswitch completion elvish > ~/.elvish/lib/jswitch.elv
```

Then load the generated script from your Elvish configuration.

## Verify

After installation, run:

```bash
jswitch <TAB>
```

Expected command completions include:

```text
init install switch list current config completion doctor
```

Shell values can also be completed:

```bash
jswitch completion <TAB>
```

Expected shell completions include:

```text
bash zsh fish powershell elvish
```
