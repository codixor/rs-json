#!/usr/bin/env bash
# Download benchmark corpus + JSONTestSuite. Run on a machine with internet.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p data tests/conformance/cases
base=https://raw.githubusercontent.com/miloyip/nativejson-benchmark/master/data
for f in twitter.json citm_catalog.json canada.json; do
  [ -s "data/$f" ] || curl -fsSL "$base/$f" -o "data/$f"
done
if [ ! -d tests/conformance/cases/.git ]; then
  git clone --depth 1 https://github.com/nst/JSONTestSuite /tmp/jts
  cp /tmp/jts/test_parsing/*.json tests/conformance/cases/
fi
echo "corpus ready: $(ls data | wc -l) data files, $(ls tests/conformance/cases | wc -l) conformance cases"
