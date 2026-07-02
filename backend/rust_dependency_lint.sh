#!/usr/bin/env bash

# Exit trap: always pause unless called from parent script
trap 'if [[ -z "${SKIP_PAUSE:-}" ]]; then echo ""; echo "Press any key to exit"; read -r -n1 -s; fi' EXIT

set -euo pipefail
cd -- "$(dirname -- "$(readlink -f -- "$0")")"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required"
    exit 1
fi

if ! cargo upgrade --version >/dev/null 2>&1; then
    echo "cargo-upgrade is required"
    echo "Run cargo install cargo-edit --locked"
    exit 1
fi

if ! cargo +nightly udeps --version >/dev/null 2>&1; then
    echo "nightly cargo-udeps is required"
    echo "Run rustup toolchain install nightly"
    echo "Run cargo install cargo-udeps --locked"
    exit 1
fi

read -r -p "Fix cargo registry for cargo upgrade? (y/N): " answer
if [[ $answer == [Yy] ]]; then
    echo "Fixing cargo registry for cargo upgrade"
    rm -rf "${CARGO_HOME:-$HOME/.cargo}/registry"
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo fetch
    use_git_registry=1
else
    echo "Skipping cargo registry fix"
    use_git_registry=0
fi

rm -f ./Cargo.lock
if [[ $use_git_registry -eq 1 ]]; then
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo upgrade --pinned --recursive false --exclude libsqlite3-sys
else
    cargo upgrade --pinned --recursive false --exclude libsqlite3-sys
fi
cargo +nightly udeps --all-targets
