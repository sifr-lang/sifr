# Rust Interop Certification 7 Review — Round 3

- Reviewed commit: `0c9b5f182`
- Base: `origin/main`
- Reviewer: agent, medium effort
- Verdict: findings; not satisfied

## Prior closure

The reviewer confirmed every round-1 and round-2 finding closed, the full
Rust-interop area green, all mandatory generated builds passing, exact
inventory counts, frozen features, safe Rust, and unrelated-path preservation.

## New findings

### 1. High — exact identity regresses generated-record view contracts

The new exact opaque-handle rendering was applied to all paired view
contracts. Existing contract-only advanced-data declarations intentionally
return generated record bridges, causing 24 driver library failures and a
blocking create-PR lane failure. Preserve generated-record contract handling,
enforce exact identity for opaque crate-backed handles, keep container/prefix
rejections, and run the entire driver library.

### 2. High — workspace Clippy failure

The recognized-probe note selection used an `Option::then(...).unwrap_or_default()`
chain rejected by `clippy::obfuscated-if-else`. Use a direct `if` expression
and rerun the documented workspace Clippy gate.

### 3. Low — canonical Rust target rendering duplicated

The driver duplicated codegen's sysroot absolute-target rule even though exact
string equality depends on codegen output. Expose and reuse the codegen
opaque-handle renderer so future canonical-root changes cannot silently drift.
