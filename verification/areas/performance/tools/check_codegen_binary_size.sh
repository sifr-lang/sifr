#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: verification/areas/performance/tools/check_codegen_binary_size.sh <baseline_ref> [candidate_ref] [demo_path]

Compares release binary size for a Sifr demo between two git refs.
Exits non-zero if candidate binary is larger than baseline.
Both refs must support explicit --release and the finalized artifact layout.

Arguments:
  baseline_ref   Git ref/commit to compare from
  candidate_ref  Git ref/commit to compare to (default: HEAD)
  demo_path      Demo .sifr path relative to repo root
                 (default: demos/codegen_structural_passes/main.sifr)
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

BASE_REF="$1"
CANDIDATE_REF="${2:-HEAD}"
DEMO_PATH="${3:-demos/codegen_structural_passes/main.sifr}"

REPO_ROOT="$(git rev-parse --show-toplevel)"

BASE_SHA="$(git -C "$REPO_ROOT" rev-parse --verify "${BASE_REF}^{commit}")"
CANDIDATE_SHA="$(git -C "$REPO_ROOT" rev-parse --verify "${CANDIDATE_REF}^{commit}")"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-size-check.XXXXXX")"
WORKTREE="$TMP_DIR/checkout"

cleanup() {
  git -C "$REPO_ROOT" worktree remove --force "$WORKTREE" >/dev/null 2>&1 || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

git -C "$REPO_ROOT" worktree add --detach "$WORKTREE" "$BASE_SHA" >/dev/null

materialize_ruff_submodule() {
  local worktree="$1"
  local ruff_path="$worktree/third_party/ruff"
  local local_ruff_path="$REPO_ROOT/third_party/ruff"

  if [[ -f "$ruff_path/crates/ruff_text_size/Cargo.toml" ]]; then
    return
  fi

  if [[ -f "$local_ruff_path/crates/ruff_text_size/Cargo.toml" ]]; then
    mkdir -p "$worktree/third_party"
    rm -rf "$ruff_path"
    ln -s "$local_ruff_path" "$ruff_path"
    return
  fi

  git -C "$worktree" submodule update --init --recursive third_party/ruff >/dev/null
}

materialize_ruff_submodule "$WORKTREE"

materialize_local_workspace_roots() {
  local worktree="$1"

  # This local audit corpus is intentionally not required for the size demo,
  # but the repository sifr.toml names the directory as a workspace source root.
  mkdir -p "$worktree/verification/areas/algorithmic_compatibility/corpora/leetcode/src"
}

materialize_local_workspace_roots "$WORKTREE"

measure_size() {
  local worktree="$1"
  local label="$2"
  # Paths affect native crate identity and retained debug/string sections.
  # Keep source, sysroot and output locations identical for both real refs;
  # retain the whole-file assertion and the explicit release profile.
  local out_dir="$TMP_DIR/output"

  mkdir -p "$out_dir"
  (
    cd "$worktree"
    cargo run -p sifr -- build "$DEMO_PATH" --release --output "$out_dir" >/dev/null
  )

  local binary="$out_dir/sifr_output/target/final/sifr_output"
  if [[ ! -f "$binary" ]]; then
    binary="$out_dir/sifr_output/target/final/sifr_output.exe"
  fi

  if [[ ! -f "$binary" ]]; then
    echo "error: binary not found for $label: $binary" >&2
    exit 1
  fi

  if stat -f "%z" "$binary" >/dev/null 2>&1; then
    stat -f "%z" "$binary"
  else
    stat -c "%s" "$binary"
  fi
}

BASE_SIZE="$(measure_size "$WORKTREE" baseline)"
# This detached checkout belongs solely to this invocation. Switching normally
# preserves reusable Cargo storage and rejects conflicting source modifications.
git -C "$WORKTREE" switch --detach "$CANDIDATE_SHA" >/dev/null
materialize_ruff_submodule "$WORKTREE"
materialize_local_workspace_roots "$WORKTREE"
CAND_SIZE="$(measure_size "$WORKTREE" candidate)"
DELTA=$((CAND_SIZE - BASE_SIZE))
PCT="$(awk -v b="$BASE_SIZE" -v c="$CAND_SIZE" 'BEGIN { if (b == 0) { print "0.00" } else { printf "%.2f", ((c - b) / b) * 100 } }')"

printf 'baseline_ref=%s\n' "$BASE_REF"
printf 'candidate_ref=%s\n' "$CANDIDATE_REF"
printf 'demo=%s\n' "$DEMO_PATH"
printf 'baseline_size_bytes=%s\n' "$BASE_SIZE"
printf 'candidate_size_bytes=%s\n' "$CAND_SIZE"
printf 'delta_bytes=%s\n' "$DELTA"
printf 'delta_percent=%s\n' "$PCT"

if (( DELTA > 0 )); then
  echo "result=FAIL (candidate binary is larger)"
  exit 2
fi

echo "result=PASS (candidate binary is not larger)"
