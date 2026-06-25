# Homebrew Distribution

This guide summarizes how to make this CLI installable with Homebrew:

```bash
brew install yourusername/tap/jswitch
```

## Overview

Homebrew installs third-party CLI tools from a tap repository. For `yourusername/tap`, Homebrew resolves the tap to:

```text
https://github.com/yourusername/homebrew-tap
```

The tap repository contains a Ruby formula at:

```text
Formula/jswitch.rb
```

The formula points to release artifacts from the main `jswitch` repository.

## 1. Publish GitHub Release Artifacts

Build macOS release binaries for Apple Silicon and Intel Macs:

```bash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

Package each binary:

```bash
tar -czf jswitch-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release jswitch
tar -czf jswitch-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release jswitch
```

Calculate checksums:

```bash
shasum -a 256 jswitch-aarch64-apple-darwin.tar.gz
shasum -a 256 jswitch-x86_64-apple-darwin.tar.gz
```

Upload the archives to a GitHub Release, for example:

```text
https://github.com/yourusername/jswitch/releases/download/v0.1.0/jswitch-aarch64-apple-darwin.tar.gz
https://github.com/yourusername/jswitch/releases/download/v0.1.0/jswitch-x86_64-apple-darwin.tar.gz
```

## 2. Create a Homebrew Tap Repository

Create a GitHub repository named:

```text
homebrew-tap
```

Expected URL:

```text
https://github.com/yourusername/homebrew-tap
```

This naming convention lets users install with:

```bash
brew install yourusername/tap/jswitch
```

or:

```bash
brew tap yourusername/tap
brew install jswitch
```

## 3. Add the Formula

Create this file in the tap repository:

```text
Formula/jswitch.rb
```

Example formula:

```ruby
class Jswitch < Formula
  desc "Fast Java version switcher written in Rust"
  homepage "https://github.com/yourusername/jswitch"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yourusername/jswitch/releases/download/v0.1.0/jswitch-aarch64-apple-darwin.tar.gz"
      sha256 "PUT_AARCH64_SHA256_HERE"
    else
      url "https://github.com/yourusername/jswitch/releases/download/v0.1.0/jswitch-x86_64-apple-darwin.tar.gz"
      sha256 "PUT_X86_64_SHA256_HERE"
    end
  end

  def install
    bin.install "jswitch"
    generate_completions_from_executable(bin/"jswitch", "completion")
  end

  test do
    assert_match "jswitch", shell_output("#{bin}/jswitch --help")
  end
end
```

Replace:

- `yourusername` with the GitHub user or organization.
- `v0.1.0` with the release tag.
- `PUT_AARCH64_SHA256_HERE` with the Apple Silicon archive SHA256.
- `PUT_X86_64_SHA256_HERE` with the Intel archive SHA256.

Commit and push the tap repository:

```bash
git add Formula/jswitch.rb
git commit -m "Add jswitch formula"
git push
```

## 4. Verify Installation

Install from the tap:

```bash
brew install yourusername/tap/jswitch
```

Check the installed binary:

```bash
jswitch --help
jswitch completion zsh
```

Run Homebrew's formula test:

```bash
brew test yourusername/tap/jswitch
```

## 5. Recommended Automation

Manual release and formula updates work for the first version, but releases should eventually be automated.

Recommended options:

1. `cargo-dist`: best fit for Rust CLI projects; can build release artifacts and help maintain installer metadata.
2. `goreleaser`: mature release automation with Homebrew tap support, also works for Rust binaries.
3. Custom GitHub Actions: maximum control, but more maintenance.

For this project, `cargo-dist` is the preferred long-term path because it is Rust-native and can automate release artifacts, checksums, and Homebrew distribution metadata.

## Release Checklist

Before publishing a Homebrew release:

1. Update the package version in `Cargo.toml`.
2. Create and push a version tag such as `v0.1.0`.
3. Build macOS release archives for `aarch64-apple-darwin` and `x86_64-apple-darwin`.
4. Upload archives to the GitHub Release.
5. Calculate SHA256 checksums.
6. Update `Formula/jswitch.rb` in `homebrew-tap`.
7. Run `brew install yourusername/tap/jswitch` and `brew test yourusername/tap/jswitch`.
