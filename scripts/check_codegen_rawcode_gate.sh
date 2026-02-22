#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "[rawcode-gate] checking intrinsics/methods for RawCode constructors..."
if rg -n "RawCode\(" crates/sifr_codegen/src/intrinsics crates/sifr_codegen/src/methods -g '*.rs' >/tmp/sifr_rawcode_gate_hits.txt; then
  echo "[rawcode-gate] FAIL: RawCode constructor usage found in intrinsics/methods:" >&2
  cat /tmp/sifr_rawcode_gate_hits.txt >&2
  exit 2
fi

echo "[rawcode-gate] validating preamble RawCode count is zero..."
cargo test -p sifr_codegen preamble_rawcode_is_zero -- --nocapture

echo "[rawcode-gate] PASS"
