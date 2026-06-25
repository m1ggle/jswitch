# Stability and Diagnostics

This page covers the phase 5 user-experience and stability features: logging, download progress, diagnostics, tests, docs, and completion.

## Logging

`jswitch` initializes structured logging on startup and writes logs to stderr.

Enable debug logs with either environment variable:

```bash
JSWITCH_DEBUG=true jswitch current
RUST_LOG=jswitch=debug jswitch install 17 --source corretto
```

Use quiet mode to suppress interactive download progress:

```bash
JSWITCH_QUIET=true jswitch install 17 --source corretto
```

## Download Progress

`jswitch install` streams archives to the cache and displays a progress bar when the terminal is not in quiet mode. Checksums are fetched and verified before extraction when the selected source provides a checksum URL.

## Doctor Checks

Run diagnostics with:

```bash
jswitch doctor
```

The command checks:

- global configuration readability
- `~/.jswitch/versions` state
- installed Java version count
- whether the selected current version is installed
- whether `java -version` is available on `PATH`

Example output:

```text
[ok] config - configuration is readable
[ok] versions-dir - /Users/me/.jswitch/versions exists
[ok] installed-versions - 2 versions installed
[ok] current-version - 17 is active (global)
[ok] java - java version "17.0.10"
doctor summary: 0 failed, 0 warnings
```

Warnings indicate actionable setup issues, such as selecting a version that has not been installed yet.

## Verification Commands

Before committing changes, run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-features -- -D warnings
```

## Completion

Generate shell completion scripts with:

```bash
jswitch completion zsh > ~/.zfunc/_jswitch
jswitch completion bash > ~/.local/share/bash-completion/completions/jswitch
jswitch completion fish > ~/.config/fish/completions/jswitch.fish
```

See [`completion.md`](completion.md) for shell-specific installation steps.
