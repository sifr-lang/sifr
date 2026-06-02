Reviewed the working tree end-to-end after pass-1 fixes. Confirmed both residuals are addressed and no new regressions appeared. Re-ran `cargo test -p sifr_analysis` locally (23 pass) and `cargo clippy -p sifr_analysis -p sifr -- -D warnings` (clean). M1 guardrails and file-size guardrails still pass.

## SATISFIED

**Residual #1 — helper_export marker is now genuinely cross-file**
- `crates/sifr_analysis/src/host/m17_tests.rs:189-212` now resolves the `helper_export` marker (helper.sifr line 0 char 2, anchored on `helper_value`) and exercises every query family it declares:
  - `definition` on `helper_file` returns a non-empty result.
  - `references` returns a vector that contains at least one `Location` whose `file == main_file` AND one whose `file == helper_file`, so the corpus is now actually multi-file at the query level (main.sifr imports `helper_value` and uses it in `value: int = helper_value`, which the references query finds).
  - `semantic_tokens` on `helper_file` returns a non-empty token stream.
- The pass-1 concern that "every asserted lookup targets `main.sifr` locals" no longer applies — the file boundary is traversed by an actual query, not just by existing on disk.

**Residual #2 — `SnapshotHandleAnchor::kind` is checked at resolve time**
- `crates/sifr_analysis/src/handles.rs:154-177` adds an `expected_kind: SnapshotHandleKind` parameter to `ensure_handle_current` and compares `anchor.kind == expected_kind` alongside `snapshot_id` and `revision`.
- Each `resolve_*` entry point passes its own kind (`handles.rs:107,115,123,131,139`), so the field is exercised on every call. The kind no longer is dead at runtime — it provides a defense-in-depth assertion that a handle cannot be silently routed to the wrong resolver in a future refactor.
- All five handle kinds plus snapshot-level `hover` still produce `AnalysisErrorKind::StaleSnapshot` after `host.update_document`, verified by `snapshot_handles_are_internal_and_reject_wrong_snapshot_resolution` (`m17_tests.rs:263-341`, still green).

**Other contracts re-verified**
- Snapshot-handle privacy unchanged: `handles.rs:1` keeps `#![allow(dead_code)]`, types/methods are `pub(crate)`, and `lib.rs:10` declares `mod handles;` with no `pub use`. No symbol leaks outside the crate.
- Package diagnostic non-duplication asserts in `verification/tooling/check_diagnostic_source_canonicalization_contract.py` and the M1 guardrail pins in `verification/tooling/check_typescript_go_m1_guardrails.py` remain intact; `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → PASS.
- Doc/tracker fit unchanged from pass 1 (`internal_docs/architecture.md`, `frontend_query_architecture.md`, `typescript_go_architecture_transfer_m1_guardrails.md`, the new M17 doc, and the M17 row in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`).

## Residual (non-blocking)

1. **Temp-dir cleanup remains best-effort.** `m17_tests.rs:259` still calls `let _ = std::fs::remove_dir_all(&dir);` on the happy path only — a panicking assertion above it still leaves `/tmp/sifr_analysis_m17_…` behind. Not raised as blocking in pass 1, still not blocking now; `tempfile::TempDir` would tidy it whenever someone next touches the file.
2. **Kind-mismatch reported as `StaleSnapshot`.** `handles.rs:165` returns `AnalysisErrorKind::StaleSnapshot` even when only the `kind` differs (e.g. a future caller routes a `TypeHandle` through `resolve_symbol_handle`). In current code that path is unreachable because handles are `pub(crate)` and each kind is set at construction, so this is purely cosmetic — but the error message wording ("snapshot handle is stale") would mislead a future debugger. Optional: introduce a dedicated `HandleKindMismatch` kind or assert in debug builds.

## Verdict

M17 remains **SATISFIED**. No blocking findings.
