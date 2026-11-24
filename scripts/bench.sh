#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cargo bench -p rs-json@0.1.0 --bench json_core --manifest-path "$ROOT/Cargo.toml" -- --sample-size 20 "$@"
