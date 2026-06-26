#!/usr/bin/env bash
#
# jswitch install script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/m1ggle/jswitch/main/scripts/install.sh | bash
#
# Downloads the latest release binary for the current platform and installs
# it to ~/.jswitch/bin (or /usr/local/bin if run as root).

set -euo pipefail

REPO_OWNER="m1ggle"
REPO_NAME="jswitch"
BINARY_NAME="jswitch"
INSTALL_DIR="${JSWITCH_INSTALL_DIR:-$HOME/.jswitch/bin}"

# ─── Detect platform ──────────────────────────────────────────────────────

detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            case "$arch" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                x86_64)        echo "x86_64-apple-darwin" ;;
                *) err "unsupported macOS architecture: $arch" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
                x86_64)        echo "x86_64-unknown-linux-gnu" ;;
                *) err "unsupported Linux architecture: $arch" ;;
            esac
            ;;
        *)
            err "unsupported OS: $os (only macOS and Linux are supported)"
            ;;
    esac
}

# ─── Helpers ──────────────────────────────────────────────────────────────

err() {
    echo "error: $*" >&2
    exit 1
}

info() {
    echo "info: $*"
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

# ─── Main ─────────────────────────────────────────────────────────────────

main() {
    need_cmd curl
    need_cmd tar

    local target archive_name download_url tmpdir

    target="$(detect_platform)"
    archive_name="${BINARY_NAME}-${target}.tar.gz"
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    # Resolve latest release tag
    local latest_tag
    latest_tag="$(curl -fsSL \
        "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed -E 's/.*"([^"]+)".*/\1/')"

    if [[ -z "$latest_tag" ]]; then
        err "could not determine latest release tag"
    fi

    info "latest release: $latest_tag"
    info "target: $target"

    download_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${latest_tag}/${archive_name}"

    # Download
    info "downloading $download_url"
    curl -fsSL -o "$tmpdir/$archive_name" "$download_url" \
        || err "download failed"

    # Extract
    info "extracting"
    tar -xzf "$tmpdir/$archive_name" -C "$tmpdir"

    # Install
    mkdir -p "$INSTALL_DIR"

    if [[ -w /usr/local/bin ]] && [[ "$EUID" -ne 0 ]]; then
        # User has write access to /usr/local/bin — symlink there too
        cp "$tmpdir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        ln -sf "$INSTALL_DIR/$BINARY_NAME" /usr/local/bin/$BINARY_NAME
        info "installed to $INSTALL_DIR/$BINARY_NAME (symlinked in /usr/local/bin)"
    else
        cp "$tmpdir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        info "installed to $INSTALL_DIR/$BINARY_NAME"
    fi

    # Verify
    "$INSTALL_DIR/$BINARY_NAME" --version || true

    echo
    echo "✅ jswitch installed successfully!"
    echo
    echo "Add $INSTALL_DIR to your PATH if not already:"
    echo
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo
    echo "Then initialize shell integration:"
    echo
    echo "  eval \"\$(jswitch init bash)\"   # or zsh/fish"
    echo
}

main "$@"
