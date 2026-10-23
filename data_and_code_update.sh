#!/usr/bin/env bash
# Compatibility entry point. See server_admin/README.md for required arguments.
set -euo pipefail
exec "$(dirname "${BASH_SOURCE[0]}")/scripts/deploy.sh" "$@"
