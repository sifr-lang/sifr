#!/usr/bin/env bash

set -euo pipefail

DEMO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${DEMO_ROOT}/../.." && pwd)"

cd "${DEMO_ROOT}"
uv sync --locked
cargo run --manifest-path "${REPO_ROOT}/Cargo.toml" -q -p sifr -- run src/main.sifr
