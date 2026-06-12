# M5 Runtime Diagnostics Wave — Review Pass 1

Branch: `codex/concurrency-runtime-m5-diagnostics-next`
Scope reviewed: first Sifr-owned structured runtime diagnostics surface in `sifr.runtime` — Sifr value types (`DiagnosticLevel`, `DiagnosticEvent`, `DiagnosticError`), `diagnostic_event(...)`, `emit_diagnostic(event)`, and lowering of `_sifr.runtime.runtime_emit_diagnostic` to gated `tracing::event!` emission with `target: "sifr.runtime"`. Metrics explicitly deferred. Warnings global filter rejection preserved.

## Result

FAIL — concrete validation-evidence mismatch in the traceability artifact. The wave itself is otherwise well shaped (correct dependency lock, no tracing-attribute pull-in, Sifr-native public types, honest metrics deferral), but the documented lane coverage does not match what was actually validated. Fix the lane manifests (or correct the claim), re-run the relevant lane, then this is shippable.

## Blocking findings

### 1. Validation-lane traceability overclaim for the new e2e fixture
The traceability artifact claims `runtime_diagnostics_tracing` is part of both lane manifests:

- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:48` — Create PR row includes `runtime_diagnostics_tracing`.
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:49` — Merge row includes `runtime_diagnostics_tracing`.

Neither manifest contains the fixture:

- `verification/validation_lanes/create_pr_e2e_manifest.json` — 120 fixture names listed; `runtime_diagnostics_tracing` absent. The reported `e2e 120 passed, 0 failed` matches the existing manifest exactly, which independently confirms the new fixture was not exercised by `scripts/run_all_tests.sh --profile create-pr`.
- `verification/validation_lanes/merge_e2e_manifest.json` — 120 fixture names listed; `runtime_diagnostics_tracing` absent.

Required fix (either, not both):

- Preferred: add `"runtime_diagnostics_tracing"` to the `fixture_names` array in both `verification/validation_lanes/create_pr_e2e_manifest.json` and `verification/validation_lanes/merge_e2e_manifest.json`, then re-run `scripts/run_all_tests.sh --profile create-pr` and record the refreshed `e2e` totals/report-signature/cache-hit numbers before opening the PR. That refresh is also necessary because the current report-signature (`293aaf3695dc42f8`) was produced against an unchanged manifest; adding a fixture must invalidate it.
- Alternative: if the new fixture is intentionally not in either lane (e.g., the lane policy excludes single-feature stdlib smoke tests), remove `runtime_diagnostics_tracing` from the Create PR and Merge rows in the traceability doc and explicitly state that the e2e fixture is validated ad-hoc only.

The ad-hoc `cargo run -q -p sifr -- check` and `... -- run` invocations are not equivalent to a lane entry — they exercise the binary but they are not gated by any future lane regression.

## Non-blocking observations

- `reviews/ad-hoc-production-concurrency-runtime-m5-runtime-diagnostics-review-pass-1.md` was a 0-byte placeholder when the review packet was assembled. That is fine as a stub for this pass's review output to be written into, but make sure it carries this review's content before the PR ships so the M5 pass-1 review row in the execution doc has substance to cite.
- Branch is "behind `origin/main` by 2 commits" per `git status`. Rebase or merge `origin/main` before opening the PR so the diff the reviewer sees matches the diff CI will validate (you already flagged this in the packet).
- Note: the `runtime_emit_diagnostic` intrinsic returns `Result[None, DiagnosticError]`, but the user-facing Sifr constructors of `DiagnosticLevel(...)` do not gate the level string. The lowering correctly rejects unknown levels with `DiagnosticError::new("unsupported diagnostic level: <name>")`, and the fixture covers that path — but consider whether a future wave should additionally provide a typed level constructor (e.g., `try_level("verbose")` returning `Result[DiagnosticLevel, DiagnosticError]`) so the rejection is push-able to compile time when the level is a literal. Not required for this wave; recording for the next observability slice.

## Items verified clean

- `Cargo.toml:97` adds `tracing = { version = "0.1.44", default-features = false, features = ["std"] }`, matching the phase-doc lock (`issues/ad-hoc-production-concurrency-runtime-platform-substrate.md:280`) verbatim. `default-features = false` disables `tracing-attributes`, so `#[instrument]` etc. cannot be used, as the phase requires.
- The workspace-level `tracing` declaration is not consumed by any first-party crate; it is only pulled in by generated runtime code through `STDLIB_FEATURE_SPECS` / `generated_cargo_dependencies`. No tracing type or subscriber is installed by Sifr, and `lib/sifr/runtime.sifr` does not reference `tracing` — the Sifr public surface owns the value types and `Result[None, DiagnosticError]`. No leakage.
- `crates/sifr_stdlib/src/lib.rs` registers `_sifr.runtime` and updates the `warnings` comment to reflect the now-existing diagnostics surface. The `unsupported_legacy_stdlib_module` / `legacy_stdlib_module_info` paths still reject `sifr.warnings` and route it to `sifr.runtime` with the "Python global warning filters are replaced by typed diagnostics" reason. Warnings filter parity remains rejected.
- `crates/sifr_stdlib/src/features.rs` adds `StdlibFeature::Tracing`, the `TRACING_DEPS` row, the `STDLIB_FEATURE_SPECS` entry, the `feature_for_codegen_requirement("tracing")` mapping, and `features_for_stdlib_module("sifr.runtime" | "_sifr.runtime") => &[Tracing]`. Cargo spec strings are stable.
- `crates/sifr_codegen/src/intrinsics/registry.rs` wires `additional_required_features("runtime_emit_diagnostic") => &[Tracing]` and dispatches lowering to `runtime::lower_runtime_emit_diagnostic`, mirroring the established intrinsic registry pattern.
- `crates/sifr_codegen/src/intrinsics/registry/runtime.rs` lowers to a self-contained Rust block: it extracts `.as_str()` views on the four input strings, matches the level, fires `tracing::event!(target: "sifr.runtime", tracing::Level::{TRACE|DEBUG|INFO|WARN|ERROR}, diagnostic_target = ..., diagnostic_name = ..., diagnostic_message = ...)`, returns `Ok(())`, and on an unknown level returns `Err(DiagnosticError::new(format!("unsupported diagnostic level: {}", ...)))`. The arity guard (`args.len() != 4`) is defensive against codegen plumbing changes. The `tracing::event!` macro syntax (`target:` first, then level, then key=value fields) is correct for `tracing` 0.1.x.
- Unit tests (`crates/sifr_codegen/src/intrinsics/registry_core_tests.rs`):
  - `lowers_runtime_diagnostic_intrinsic_with_tracing_metadata` asserts `Tracing` is registered as an `additional_required_features` entry and checks the rendered Rust contains the expected `tracing::event!`, `target: "sifr.runtime"`, `tracing::Level::INFO`, `diagnostic_target = __sifr_diagnostic_target`, `DiagnosticError::new`, and `unsupported diagnostic level` substrings.
  - `runtime_diagnostic_intrinsic_rejects_wrong_arity` exercises the defensive arity guard.
  - `runtime_module_dependency_metadata_includes_tracing_only` pins that loading `sifr.runtime` (with no extra required features) renders exactly the locked `tracing = { version = "0.1.44", default-features = false, features = ["std"] }` dependency string. This is the right shape to catch a future drift that would silently re-introduce `default-features = true` or `attributes`.
- `lib/sifr/runtime.sifr` exposes only Sifr-owned types and helpers. `DiagnosticError(Error)` uses the built-in error base class (matches `SignalError`/`ResourceError` pattern). Module-level `TRACE/DEBUG/INFO/WARN/ERROR: DiagnosticLevel = DiagnosticLevel(...)` follows the existing `SIGINT`/`SIGTERM` module-constant pattern proven by `sifr.signal`.
- `crates/sifr/tests/e2e/pass/runtime_diagnostics_tracing.sifr` exercises all five valid levels through `emit_diagnostic(...)` (their match arms in the lowered block) and the unknown-level rejection path, asserting the exact error message string `"unsupported diagnostic level: verbose"` produced by the lowering. The success-path `except` block intentionally cannot fire under a no-subscriber runtime because `tracing::event!` is a no-op without a dispatcher, which is the documented invariant for this wave.
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md` status block is correct in spirit (in-progress, with the diagnostics row added, follow-up section calling out metrics deferral with the policy reason). The metrics deferral language — "the phase requires concrete metric names, label/cardinality policy, emission points, redaction policy, and deterministic tests before selecting the `metrics` crate" — is consistent with the phase's "metrics after M5 metric schema approval" rule. The only doc problem is the Validation Coverage row covered under the blocking finding above.
- No Python warning filter parity reappeared anywhere: `bare_cpython_warnings_import`/`legacy_sifr_warnings_removed` are still in the fail-suite row.

## Required to pass next review

- Add `runtime_diagnostics_tracing` to both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json`, re-run `scripts/run_all_tests.sh --profile create-pr`, and update the wave's recorded validation evidence (e2e totals, report-signature, cache-hits) accordingly — or remove the lane claims from the traceability artifact and demote coverage to ad-hoc.
- Reconfirm `cargo fmt --check`, `cargo test -p sifr_codegen runtime_diagnostic`, and `cargo test -p sifr_codegen runtime_module_dependency_metadata_includes_tracing_only` after the manifest change (these should be untouched, but include them in the rerun snapshot for the record).
