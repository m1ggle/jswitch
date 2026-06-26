#!/usr/bin/env bash
#
# jswitch release script
#
# Builds release binaries for macOS (Apple Silicon + Intel), packages them,
# generates checksums, and optionally uploads to a GitHub release.
#
# Usage:
#   ./scripts/release.sh <tag>           # build + package + checksum
#   ./scripts/release.sh <tag> --upload   # also upload to GitHub
#
# Prerequisites:
#   - Rust toolchain with cross-compile targets installed
#   - gh CLI authenticated (for --upload)

set -euo pipefail

REPO_OWNER="m1ggle"
REPO_NAME="jswitch"
BINARY_NAME="jswitch"

# ─── Targets ───────────────────────────────────────────────────────────────

TARGETS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
)

# ─── Helpers ──────────────────────────────────────────────────────────────

err() {
    echo "error: $*" >&2
    exit 1
}

info() {
    echo "release: $*"
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

# ─── Main ─────────────────────────────────────────────────────────────────

main() {
    [[ $# -ge 1 ]] || err "usage: $0 <tag> [--upload]"
    need_cmd cargo
    need_cmd tar
    need_cmd shasum

    local tag="$1"
    local do_upload=false
    [[ "${2:-}" == "--upload" ]] && do_upload=true

    # Strip leading 'v' for version checks
    local version="${tag#v}"

    info "building release for $tag (version $version)"

    # Verify working tree is clean
    if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
        err "working tree is not clean — commit or stash changes first"
    fi

    # Verify tag matches Cargo.toml version
    local cargo_version
    cargo_version="$(grep '^version' Cargo.toml | head -1 | sed -E 's/version = "([^"]+)"/\1/')"
    if [[ "$cargo_version" != "$version" ]]; then
        err "Cargo.toml version ($cargo_version) does not match tag ($version)"
    fi

    local dist_dir="dist"
    rm -rf "$dist_dir"
    mkdir -p "$dist_dir"

    # Build and package each target
    local target archive_name
    for target in "${TARGETS[@]}"; do
        info "building $target"

        # Ensure target is installed
        rustup target list --installed 2>/dev/null | grep -q "$target" \
            || rustup target add "$target"

        cargo build --release --target "$target"

        archive_name="${BINARY_NAME}-${target}.tar.gz"
        info "packaging $archive_name"

        tar -czf "$dist_dir/$archive_name" \
            -C "target/$target/release" \
            "$BINARY_NAME"

        info "built $archive_name"
    done

    # Generate checksums
    info "generating checksums"
    (
        cd "$dist_dir"
        shasum -a 256 *.tar.gz > checksums-sha256.txt
    )

    info "artifacts in $dist_dir/:"
    ls -lh "$dist_dir/"

    # Print checksums
    echo
    echo "─── SHA256 Checksums ───"
    cat "$dist_dir/checksums-sha256.txt"
    echo "────────────────────────"
    echo

    # Upload
    if $do_upload; then
        need_cmd gh
        info "uploading to GitHub release $tag"

        gh release create "$tag" \
            --repo "${REPO_OWNER}/${REPO_NAME}" \
            --title "$tag" \
            --generate-notes \
            --verify-tag \
            "$dist_dir"/*.tar.gz \
            "$dist_dir/checksums-sha256.txt"

        info "release $tag published"
        info "Homebrew tap update will trigger automatically via workflow"
    else
        info "skip upload (pass --upload to enable)"
        info "review artifacts in $dist_dir/ then run:"
        info "  gh release create $tag --repo ${REPO_OWNER}/${REPO_NAME} $dist_dir/*.tar.gz $dist_dir/checksums-sha256.txt"
    fi
}

main "$@"
