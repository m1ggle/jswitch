#!/usr/bin/env bash
#
# jswitch cross-compile script
#
# Compiles release binaries for all supported targets and optionally
# produces distributable archives. Used by release.sh internally, but
# can also be run standalone for quick cross-target verification.
#
# Usage:
#   ./scripts/cross-compile.sh                # build all targets
#   ./scripts/cross-compile.sh --package      # build + create tar.gz archives
#   ./scripts/cross-compile.sh aarch64-apple-darwin  # build single target

set -euo pipefail

BINARY_NAME="jswitch"

# All supported cross-compile targets
DEFAULT_TARGETS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "x86_64-unknown-linux-gnu"
)

# ─── Helpers ──────────────────────────────────────────────────────────────

err() {
    echo "error: $*" >&2
    exit 1
}

info() {
    echo "x-compile: $*"
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

# ─── Functions ────────────────────────────────────────────────────────────

# Ensure a rustup target is installed
ensure_target() {
    local target="$1"
    if ! rustup target list --installed 2>/dev/null | grep -q "$target"; then
        info "installing target $target"
        rustup target add "$target"
    fi
}

# Build a single target
build_target() {
    local target="$1"
    ensure_target "$target"
    info "building $target"
    cargo build --release --target "$target"
}

# Package a single target into a tar.gz
package_target() {
    local target="$1"
    local archive_name="${BINARY_NAME}-${target}.tar.gz"
    local dist_dir="dist"

    mkdir -p "$dist_dir"

    if [[ ! -f "target/$target/release/$BINARY_NAME" ]]; then
        err "binary not found for $target — run build first"
    fi

    info "packaging $archive_name"
    tar -czf "$dist_dir/$archive_name" \
        -C "target/$target/release" \
        "$BINARY_NAME"
}

# ─── Main ─────────────────────────────────────────────────────────────────

main() {
    need_cmd cargo
    need_cmd rustup

    local do_package=false
    local targets=()

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --package|-p)
                do_package=true
                shift
                ;;
            --help|-h)
                echo "usage: $0 [--package] [target ...]"
                echo
                echo "targets:"
                for t in "${DEFAULT_TARGETS[@]}"; do
                    echo "  $t"
                done
                exit 0
                ;;
            *)
                targets+=("$1")
                shift
                ;;
        esac
    done

    # Use provided targets or default to all
    if [[ ${#targets[@]} -eq 0 ]]; then
        targets=("${DEFAULT_TARGETS[@]}")
    fi

    info "targets: ${targets[*]}"
    echo

    local target
    local failed=()

    for target in "${targets[@]}"; do
        if build_target "$target"; then
            info "✓ $target"
            if $do_package; then
                package_target "$target"
            fi
        else
            info "✗ $target (build failed)"
            failed+=("$target")
        fi
        echo
    done

    # Summary
    info "─── summary ───"
    for target in "${targets[@]}"; do
        if [[ ${#failed[@]} -gt 0 && " ${failed[*]} " == *" $target "* ]]; then
            echo "  ✗ $target"
        else
            echo "  ✓ $target"
        fi
    done

    if [[ ${#failed[@]} -gt 0 ]]; then
        err "${#failed[@]} target(s) failed: ${failed[*]}"
    fi

    info "all targets built successfully"
}

main "$@"
