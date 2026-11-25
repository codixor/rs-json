#!/usr/bin/env bash
# Generate large synthetic inputs (deterministic) with the crate's own example.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p data
cargo run --release --example gen_corpus -- data/gen_mixed_16mb.json   mixed  16
cargo run --release --example gen_corpus -- data/gen_array_256mb.json  array  256
cargo run --release --example gen_corpus -- data/gen_lines_256mb.ndjson ndjson 256
