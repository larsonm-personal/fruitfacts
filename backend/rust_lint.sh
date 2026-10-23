#!/usr/bin/env bash
# Check both the production server and optional support tools without upgrading dependencies
set -euo pipefail
cd -- "$(dirname -- "$0")"
case "${1:-check}" in
  check)
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings
    ;;
  fix)
    cargo fix --all-targets --all-features --allow-dirty
    cargo clippy --all-targets --all-features --fix --allow-dirty
    cargo fmt --all
    ;;
  deps)
    cargo +nightly udeps --all-targets --all-features
    cargo +nightly udeps --all-targets --no-default-features
    ;;
  *) echo "Usage: $0 [check|fix|deps]" >&2; exit 2 ;;
esac
