# M5 Runtime Diagnostics Wave — Review Pass 2

Branch: `codex/concurrency-runtime-m5-diagnostics-next`
Scope reviewed: same wave as pass 1 (Sifr-owned `sifr.runtime` diagnostics surface, `_sifr.runtime.runtime_emit_diagnostic` intrinsic, `tracing::event!` lowering, fixture + lane wiring) plus the fixes applied in response to pass-1 blockers.

## Result

PASS — the pass-1 blocker is fully resolved (the new fixture is now in both lane manifests and was actually exercised by the create-pr lane), and the additional batch-Cargo-toml wiring uncovered by re-running that lane is correctly closed with both a module-driven and a `required_crates`-driven path plus a pinned contract test. No new correctness, panic, or feature-wiring concerns surfaced.

## Pass-1 blocker verification

Prior blocker: traceability doc claimed `runtime_diagnostics_tracing` in both lanes while neither manifest contained it.

Verified fixed:

- `verification/validation_lanes/create_pr_e2e_manifest.json:116` — `"runtime_diagnostics_tracing"` is now present in `fixture_names`. The lane file went from 120 to 121 fixture names.
- `verification/validation_lanes/merge_e2e_manifest.json:131` — `"runtime_diagnostics_tracing"` is now present in `fixture_names`. The merge lane file went from 131 to 132 fixture names.
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md:48,49` — Create PR and Merge rows still list `runtime_diagnostics_tracing`; with the manifest fix, the claim is now truthful, not aspirational.
- The reported `report_signature` changed from the pass-1 `293aaf3695dc42f8` to `d760194c89dbc954`, which is consistent with the manifest mutation (a passing rerun on an unchanged manifest would have re-produced the old signature). Reported e2e count is 121 — exactly `120 + 1` for the newly added fixture, which is the right delta.
- The user-reported `scripts/run_all_tests.sh --profile create-pr` rerun is a PASS at the new manifest; the warm-wall-time advisory note is non-blocking by design.

The alternative remediation path (demote the doc claim to ad-hoc) was correctly *not* taken — the preferred manifest-add path was taken, which is the stronger validation contract.

## New surface added since pass 1 (batch Cargo.toml wiring)

Adding the fixture to the create-pr lane forced the e2e harness to compile a generated batch crate that imports `tracing::event!`. Pass 1's surface intentionally only wired tracing at the workspace level and through `STDLIB_FEATURE_SPECS::Tracing`, so the e2e batch builder didn't see it. The pass-2 fixes close that hole through three coherent points:

- `crates/sifr/tests/e2e_support/harness_model.rs:457` — `infer_dependencies` now adds `"tracing"` to the inferred required crate set when generated Rust source contains `tracing::` or `use tracing`. This matches the existing inference pattern used for `bigdecimal`, `rust_decimal`, etc. False positives are benign (additive dep), and the new diagnostic lowering deterministically emits `tracing::event!(...)`, so the substring match is reliable for the new surface.
- `crates/sifr/tests/e2e_support/fixture_compilation.rs:308,429` — the batch Cargo.toml generator now emits the locked `tracing = { version = "0.1.44", default-features = false, features = ["std"] }` spec from both the `sifr.runtime` / `_sifr.runtime` stdlib-module branch and the `required_crates` branch. The spec string is byte-identical to the workspace `Cargo.toml:97` and `crates/sifr_stdlib/src/features.rs:198` `TRACING_DEPS`, so there is no risk of `default-features = true` or `tracing-attributes` leaking through this path.
- `crates/sifr/tests/e2e_support/harness_behavior_tests.rs:526` — `test_generate_cargo_toml_runtime_diagnostics_use_locked_tracing_spec` independently pins both code paths (module-driven and required-crate-driven) to the exact locked spec. This is the right level: it would catch a future drift on either side of the OR.

The three-point shape (inference → spec → contract pin) mirrors how `tokio` is wired, so the diff fits the established convention rather than introducing a new pattern.

## Items independently verified in this pass

- `lib/sifr/runtime.sifr` is unchanged from pass 1's accepted shape: Sifr-owned `DiagnosticLevel`, `DiagnosticEvent`, `DiagnosticError(Error)` with `message: str`, the five module constants, `diagnostic_event(...)`, and `emit_diagnostic(event) -> Result[None, DiagnosticError]`. Mirrors the `SignalError(Error) { message: str }` shape at `lib/sifr/signal.sifr:5-6` — proven pattern.
- `crates/sifr_stdlib/src/runtime.rs` (new file, 24 lines) registers exactly one intrinsic `runtime_emit_diagnostic` with `FunctionType::all_borrow` over four `Type::Str` arguments and `Result[None, DiagnosticError]` return. No constants. Matches the Sifr-side call site verbatim, with no extra surface area.
- `crates/sifr_stdlib/src/lib.rs:18,38,108` adds `mod runtime`, `use runtime::intrinsic_runtime`, and the `"_sifr.runtime" => Some(intrinsic_runtime())` arm. `_sifr.runtime` is `pub` through `STDLIB_SOURCES` / `intrinsic_modules`. The legacy-warning routing comment at `lib.rs:246-247` was updated to remove the "until the M5 diagnostics surface lands" caveat now that the surface exists.
- `crates/sifr_codegen/src/intrinsics/registry.rs:30` declares the `runtime` submodule. `registry.rs:74` registers `"runtime_emit_diagnostic" => &[StdlibFeature::Tracing]` in `additional_required_features`. `registry.rs:715` dispatches `"runtime_emit_diagnostic"` to `runtime::lower_runtime_emit_diagnostic` with `required_feature = None`. The `Tracing` dep is therefore pulled in only via `additional_required_features`, which is the correct shape — `required_feature` is reserved for single-feature intrinsics and `Tracing` is the appropriate `additional_required_features` companion.
- `crates/sifr_codegen/src/intrinsics/registry/runtime.rs:5-82` lowers to a self-contained Rust block expression. Inputs are normalized via `(level).as_str()` etc. so user-supplied `String` and `&str` shapes both match. The `match` arms emit `tracing::event!(target: "sifr.runtime", tracing::Level::{TRACE|DEBUG|INFO|WARN|ERROR}, diagnostic_target = …, diagnostic_name = …, diagnostic_message = …)` and return `Ok(())`. The wildcard arm returns `Err(DiagnosticError::new(format!("unsupported diagnostic level: {}", __sifr_diagnostic_level)))`. The `{{}}` → `{}` and named-arg interpolation are correct. No `.unwrap()` / `.expect()` is reachable from user input. The block returns `Result<(), DiagnosticError>`, matching the registered Sifr intrinsic signature in `crates/sifr_stdlib/src/runtime.rs:17`.
- `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:97-149` adds three tests:
  - `lowers_runtime_diagnostic_intrinsic_with_tracing_metadata` pins the rendered `tracing::event!`, `target: "sifr.runtime"`, `tracing::Level::INFO`, `diagnostic_target = __sifr_diagnostic_target`, `DiagnosticError::new`, and `unsupported diagnostic level` substrings. This is the right grain — it will fail on attribute-pull-in regressions or static-target regressions.
  - `runtime_diagnostic_intrinsic_rejects_wrong_arity` pins the defensive arity guard.
  - `runtime_module_dependency_metadata_includes_tracing_only` calls `generated_cargo_dependencies({"sifr.runtime"}, {})` and asserts the result is *exactly* `[tracing = { version = "0.1.44", default-features = false, features = ["std"] }]`. This is the strongest possible pin against silently re-introducing `default-features = true`, `attributes`, or any additional transitive dep through this module.
- `crates/sifr_stdlib/src/features.rs:32,68,197-200,323-326,377,424` adds the `Tracing` variant, the `tracing` crate-name string, the `TRACING_DEPS` row, the `STDLIB_FEATURE_SPECS` entry, the `feature_for_codegen_requirement("tracing") => Tracing` mapping, and the `"sifr.runtime" | "_sifr.runtime" => &[StdlibFeature::Tracing]` row. The literal spec string is byte-identical to the workspace `Cargo.toml:97` and to the batch-harness branches in `fixture_compilation.rs:310,431`, so there is exactly one canonical spec value.
- `crates/sifr_stdlib/src/sources.rs:97-100` registers `sifr.runtime` against `lib/sifr/runtime.sifr` so `from sifr.runtime import …` resolves at compile time.
- `crates/sifr/tests/e2e/pass/runtime_diagnostics_tracing.sifr` covers all five valid levels through `emit_diagnostic(...)` and the unknown-level rejection via `DiagnosticLevel("verbose")`. The success-path `except DiagnosticError as e: assert e.message == ""` is effectively dead under a no-subscriber runtime (tracing without dispatcher is a no-op, so the success arm always returns `Ok(())`), but it's a useful tripwire if any future change starts surfacing happy-path errors. The failure-path assertion `failed = e2.message == "unsupported diagnostic level: verbose"` exactly matches what `format!("unsupported diagnostic level: {}", "verbose")` produces in the lowering.
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md` is internally consistent: the production surface row (line 23) cites three concrete validation evidence items (the lane fixture and the two new unit tests); the host matrix (line 42) correctly records diagnostics as host-independent and notes "Sifr does not install a global subscriber or Python warning filter"; the follow-up boundary (line 60) explicitly defers metrics with the policy reason ("concrete metric names, label/cardinality policy, emission points, redaction policy, and deterministic tests before selecting the `metrics` crate"). The metrics deferral language matches the phase-doc rule.
- `Cargo.toml:97` continues to hold the locked `tracing = { version = "0.1.44", default-features = false, features = ["std"] }`. No first-party crate consumes this declaration; it is the workspace-level lock referenced by the literal spec strings in the codegen path.
- `crates/sifr_stdlib/src/lib.rs:243-248` keeps `warnings` mapped to `sifr.runtime` with the legacy-rejection diagnostic. No Python global-warning-filter parity reappears.
- File-size guardrail: largest touched file is `crates/sifr/tests/e2e_support/fixture_compilation.rs` at 897 lines, still under the 900-line cap. All other touched files remain well below the limit; new files (`registry/runtime.rs` 82 lines, `stdlib/runtime.rs` 24 lines, `lib/sifr/runtime.sifr` 60 lines) are small and single-responsibility.

## Non-blocking observations

- Branch is still behind `origin/main` by 2 commits (different 2 commits than pass 1, since main has advanced). Same housekeeping note as pass 1: rebase or merge `origin/main` before opening the PR so the diff the reviewer sees matches the diff CI will validate.
- `fixture_compilation.rs` is at 897/900 lines. Not blocking, but the very next dep added through this generator will trip the guardrail; the next wave that touches this file should plan to split the dep-spec emitter by responsibility (stdlib-module branch vs `required_crates` branch is the obvious cleavage) rather than adding more arms in place.
- The accidental broad `cargo test -p sifr_codegen runtime_ -- --nocapture` filter — flagged in the prompt as failing against unrelated legacy async-runtime tests — is a known pre-existing surface unrelated to this wave. The new runtime diagnostics tests pass under the precise filter, and the authoritative create-pr lane passes end-to-end, so this does not block the wave. The legacy `runtime_*` test names in unrelated areas are a future-cleanup item if naming collisions become a recurring annoyance.
- `infer_dependencies` substring-matches `"tracing::"` and `"use tracing"` to pull in the `tracing` crate. The match is intentionally broad and additive (a false positive only over-includes a benign dep that would otherwise be unused), so this is fine for now. If any future first-party-crate-derived generated Rust starts emitting `tracing::` in comments only, the existing pattern is robust enough.
- The fixture's success-path `except DiagnosticError as e: assert e.message == ""` arm is effectively unreachable under a no-subscriber runtime (which is the documented invariant for this wave). It is harmless and slightly defensive against future surprises, so it can stay; a future wave that installs a subscriber may want to add a more deliberate assertion here.
- Future observability slice (carried forward from pass 1): consider a `try_level("verbose") -> Result[DiagnosticLevel, DiagnosticError]` constructor so unknown-level rejection can move to compile time when the level is a literal. Not required for this wave.

## Required to pass next review

Nothing required. Recommend rebasing on `origin/main` and opening the PR.
