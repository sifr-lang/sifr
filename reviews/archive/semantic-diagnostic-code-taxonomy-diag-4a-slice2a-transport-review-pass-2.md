# Review: milestone_diag_4a — Slice 2a Transport Plumbing (Pass 2)

Branch: `codex/semantic-diagnostics-diag-4a-slice2` (working tree on top of `10129970`, the slice 1 merge)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md)

## Verdict

**Ready to ship as the slice 2a PR.** Every actionable pass-1 finding (R1, R2, O3, O4, N7) is closed by a focused, in-place change. No new findings have surfaced. Quick-profile signature continues to match `e1bf653aaa770517` (slice 1 baseline), which is the correct guarantee for a transport-only slice that intentionally produces no production behavior change. The patch surface stays tight to the slice 2a scope from the pre-implementation review: `Option<DiagnosticCode>` on `LoweringError`, additive `error_with_code` helper, conversion-site forwarding in `lower_frontend_module`, `STDLIB_BOOTSTRAP_FAILURE` override preserved, no fixture re-keying, no bridge deletion, no HIR call-site migration.

## Pass-1 finding closure

Severity legend reused from pass 1: **R** = recommended before PR opens (correctness/test gap); **O** = optional polish; **N** = note.

### R1 — Driver-side round-trip test for the `Some(_) → with_code` branch

**Closed.** Two new tests under `crates/sifr_driver/src/frontend/module_lowering.rs::tests` exercise the production branch decision in `lowering_error_to_compile_error`:

- [`coded_lowering_error_uses_active_diagnostic_code`](../crates/sifr_driver/src/frontend/module_lowering.rs:78) — feeds `LoweringError { code: Some(DiagnosticCode::TYPE_MISMATCH), .. }` through the helper with `FrontendDiagnosticStyle::Bare`, then asserts (a) `compile_error.code == Some(TYPE_MISMATCH)`, (b) `compile_error.to_diagnostic().code == "SIFR-TYPE-0002"`, and (c) the rendered URL is `https://sifr.sh/docs/errors/SIFR-TYPE-0002`. The triple assertion pins both the in-process state and the public-facing identity emitted by `to_diagnostic()`.
- [`codeless_lowering_error_preserves_legacy_bridge`](../crates/sifr_driver/src/frontend/module_lowering.rs:91) — feeds `LoweringError { code: None, .. }` through the helper with `FrontendDiagnosticStyle::ModulePrefixed`, then asserts (a) `compile_error.code == None`, (b) `compile_error.message == "[main] expected int, got str"` (proving the prefix path was taken), and (c) `compile_error.to_diagnostic().code == "SIFR-TYPE-0001"` (proving the legacy bridge is still alive when no code is attached).

Two design choices in the patch make these tests reliable:

1. **The branch arm is now an extracted helper.** `lower_frontend_module` no longer constructs `CompileError` inline; it calls `lowering_error_to_compile_error(module_name, diagnostic_style, error)` ([crates/sifr_driver/src/frontend/module_lowering.rs:36-52](../crates/sifr_driver/src/frontend/module_lowering.rs:36)). The extraction is purely refactor-grade — the body is byte-equivalent to the previous inline expression — but it isolates the branch decision into a small pure function the unit tests can call without standing up an AST or a real lowering pipeline. This is a reasonable substitute for the pass-1 R1 wording "drive a `LoweringError { code: Some(_) }` through `lower_frontend_module`": the branch decision *is* `lowering_error_to_compile_error`, so testing it directly proves the only behavior that slice 2a actually changes.
2. **Both arms exercise complementary diagnostic styles.** Test 1 uses `Bare`; test 2 uses `ModulePrefixed`. Together they prove the formatting branch and the code-forwarding branch are independent — a future refactor that, for example, accidentally moved code-forwarding under one style would fail one of the two tests.

The pass-1 R1 concern was that *no* test pinned the new branch in [crates/sifr_driver/src/frontend/module_lowering.rs:47-51](../crates/sifr_driver/src/frontend/module_lowering.rs:47) against silent regression. After this patch, both arms (`Some` and `None`) are pinned, the rendered code string is asserted (so a future change to the `code.code()` mapping would surface here too), and both diagnostic styles are exercised. R1 is fully closed.

A minor stylistic observation that is *not* a finding: the test helper `lowering_error(code, message)` ([module_lowering.rs:69-76](../crates/sifr_driver/src/frontend/module_lowering.rs:69)) constructs `LoweringError` via the public field syntax. This works because `LoweringError`'s fields are `pub` ([crates/sifr_hir/src/lower/mod.rs:84-89](../crates/sifr_hir/src/lower/mod.rs:84)). That is also true for the `lib.rs` re-export at [crates/sifr_hir/src/lib.rs:19](../crates/sifr_hir/src/lib.rs:19). No encapsulation is being violated.

### R2 — Visibility inconsistency between `error` and `error_with_code`

**Closed.** [crates/sifr_hir/src/lower/mod.rs:226](../crates/sifr_hir/src/lower/mod.rs:226) now reads `fn error_with_code(&mut self, code: DiagnosticCode, message: String)` (plain `fn`, no `pub(super)`). This matches the existing `fn error(&mut self, message: String)` at [crates/sifr_hir/src/lower/mod.rs:211](../crates/sifr_hir/src/lower/mod.rs:211).

I verified that the descendant-private access still works for the new test module: `crates/sifr_hir/src/lower/diagnostic_transport_tests.rs` is declared as `mod diagnostic_transport_tests;` at [crates/sifr_hir/src/lower/mod.rs:247](../crates/sifr_hir/src/lower/mod.rs:247) and uses `super::LowerCtx`, so it sees the private `error_with_code` method via the standard parent-private/descendant-access carve-out. The wider `pub(super)` was unnecessary and is now correctly dropped.

### O3 — Stdlib override comment tightening

**Closed.** [crates/sifr_driver/src/stdlib/bootstrap.rs:64-67](../crates/sifr_driver/src/stdlib/bootstrap.rs:64) now reads:

```rust
// Even if `e.code` is `Some(_)`, stdlib lowering
// failures collapse to bootstrap failures from the
// caller's perspective, not user-facing semantic
// diagnostics.
```

The "Even if `e.code` is `Some(_)`" clause is the load-bearing addition that pass-1 O3 specifically called for: it makes the override visible to a future slice 2b reviewer who would otherwise see `e.code` referenced nowhere in the body and wonder whether the omission is a bug. The comment now explains *both* the philosophy (stdlib is bootstrap, not user-facing) and the operational consequence (any HIR-emitted code is intentionally discarded). Good.

### O4 — Stale `next diag_4a slice` comment in test_runner/orchestrator.rs

**Closed.** [crates/sifr_driver/src/test_runner/orchestrator.rs:113-115](../crates/sifr_driver/src/test_runner/orchestrator.rs:113) now reads:

```rust
// Forwards `LoweringError.code` faithfully: `None`
// for legacy call sites and `Some(_)` after upcoming
// diag_4a slice-2b migrations.
```

The slice-2 ambiguity that pass-1 O4 flagged ("the next diag_4a slice" was unclear once 2a *was* the next slice) is resolved: the new wording explicitly names slice 2b as the migration milestone for active codes, while accurately describing the current behavior (faithful pass-through, with `None` predominating today). This is exactly the rephrasing pass-1 O4 suggested.

### N7 — TODO next to the dead-code allow

**Closed.** [crates/sifr_hir/src/lower/mod.rs:220-225](../crates/sifr_hir/src/lower/mod.rs:220) now carries:

```rust
// TODO(diag_4a slice 2b): remove this allow when the first domain
// migration calls `error_with_code`.
#[allow(
    dead_code,
    reason = "diag_4a slice 2a adds transport before per-domain HIR call-site migration"
)]
fn error_with_code(&mut self, code: DiagnosticCode, message: String) {
```

The TODO line names the cleanup obligation explicitly and ties it to the same `(diag_4a slice 2b)` marker used elsewhere in the codebase (e.g. [crates/sifr_driver/src/stdlib/bootstrap.rs:30](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) and [crates/sifr_driver/src/stdlib/bootstrap.rs:48](../crates/sifr_driver/src/stdlib/bootstrap.rs:48)). When the first slice 2b sub-PR migrates a `LowerCtx::error` site to `error_with_code`, the function will gain a production caller and clippy will start warning that the `#[allow]` is unused; the TODO makes it obvious that the attribute should be deleted in the same diff. This matches pass-1 N7 word-for-word.

### Pass-1 findings tracked but not requiring code changes

- **O5 — Wall-time variance.** Pass 1 noted the ~65% wall-time jump (79.70s → 131.56s) and recommended a one-line PR-description note. The validation table now reports `wall_time=86.73s`, which is only ~9% above slice 1 — well within environmental noise. The signature match is unchanged at `e1bf653aaa770517`, which is the load-bearing assertion for a transport-only slice. No PR-description hygiene action is necessary because the wall-time delta is no longer surprising.
- **N6 — Issue-tracker checkbox state.** [issues/...:35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:35) is correctly `[ ]` for an in-progress slice; the merged-PR pattern from slice 1 ([issues/...:34](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34)) is the right template for the post-merge update. No code change required pre-merge.

## Independent re-verification

I read the working-tree state of every touched file and confirmed:

| Pre-review slice 2a deliverable | Status | Evidence |
| --- | --- | --- |
| `code: Option<DiagnosticCode>` field on `LoweringError` | ✅ | [crates/sifr_hir/src/lower/mod.rs:84-89](../crates/sifr_hir/src/lower/mod.rs:84) |
| `LowerCtx::error_with_code` added alongside legacy `error` (parity-styled `fn`) | ✅ | [crates/sifr_hir/src/lower/mod.rs:226-233](../crates/sifr_hir/src/lower/mod.rs:226) |
| Legacy `LowerCtx::error(message)` initialises `code: None` | ✅ | [crates/sifr_hir/src/lower/mod.rs:211-218](../crates/sifr_hir/src/lower/mod.rs:211) |
| `frontend/module_lowering.rs` branches on `error.code` (Some → `with_code`; None → `new`) | ✅ | [crates/sifr_driver/src/frontend/module_lowering.rs:47-51](../crates/sifr_driver/src/frontend/module_lowering.rs:47) (now extracted into `lowering_error_to_compile_error`) |
| `stdlib/bootstrap.rs` keeps `STDLIB_BOOTSTRAP_FAILURE` override and documents it | ✅ | [crates/sifr_driver/src/stdlib/bootstrap.rs:60-77](../crates/sifr_driver/src/stdlib/bootstrap.rs:60) |
| `test_runner/orchestrator.rs` forwards `error.code` and the comment is current | ✅ | [crates/sifr_driver/src/test_runner/orchestrator.rs:108-119](../crates/sifr_driver/src/test_runner/orchestrator.rs:108) |
| HIR test: coded `error_with_code` records `Some(code)` | ✅ | [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:4-16](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:4) |
| HIR test: legacy `error` records `None` | ✅ | [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:18-27](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:18) |
| Driver test: `Some(_) → CompileError::with_code → "SIFR-TYPE-0002"` | ✅ | [crates/sifr_driver/src/frontend/module_lowering.rs:78-89](../crates/sifr_driver/src/frontend/module_lowering.rs:78) |
| Driver test: `None → CompileError::new → "SIFR-TYPE-0001"` (legacy bridge alive) | ✅ | [crates/sifr_driver/src/frontend/module_lowering.rs:91-102](../crates/sifr_driver/src/frontend/module_lowering.rs:91) |
| Bridge `CompilePhase::TypeCheck => SIFR-TYPE-0001` left intact | ✅ | [crates/sifr_driver/src/diagnostics.rs:135-140](../crates/sifr_driver/src/diagnostics.rs:135) |
| 90 fixtures unchanged | ✅ | git diff shows no `crates/sifr/tests/e2e/**` modifications |
| 23 driver/CLI unit-test occurrences of `SIFR-TYPE-0001` unchanged | ✅ | git diff shows none of those files modified |
| Issue tracker: slice-1 PR link recorded | ✅ | [issues/...:34](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34) |
| Issue tracker: slice-2a status item present (correctly `[ ]`) | ✅ | [issues/...:35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:35) |
| Issue tracker: slice-2 pre-review pointer | ✅ | [issues/...:37](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:37) |
| Issue tracker: slice-2a in-progress review pointer | ✅ | [issues/...:38](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:38) |
| Issue tracker: slice 2a validation evidence block | ✅ | [issues/...:76-84](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:76) |
| Quick-profile signature still `e1bf653aaa770517` | ✅ | reported and matches slice 1 |

Patch shape (`git diff --stat` on the working tree):

```
crates/sifr_driver/src/frontend/module_lowering.rs   | 74 +++++++++++++++--
crates/sifr_driver/src/stdlib/bootstrap.rs           |  4 +
crates/sifr_driver/src/test_runner/orchestrator.rs   |  5 +-
crates/sifr_hir/src/lower/mod.rs                     | 21 +++++
issues/ad-hoc-semantic-diagnostic-code-taxonomy-...  | 17 +++-
+ crates/sifr_hir/src/lower/diagnostic_transport_tests.rs  (new, 27 lines)
```

This is the right size for slice 2a: ~30 production-source lines (additive transport + extraction) plus two small test modules. No incidental refactors, no commented-out code, no introduced abstractions outside the test surface, no new public API on `sifr_hir` or `sifr_driver` beyond the additive transport requested by the pre-review.

## Scope discipline check

The patch still does **not** do any of the things the pre-review flagged as out-of-scope for slice 2a:

- ✅ No HIR call sites migrated (489 sites untouched — `git grep "ctx.error(" crates/sifr_hir/src/lower/` returns the same hit count as slice 1).
- ✅ No fixtures re-keyed (~90 untouched).
- ✅ No verification baselines re-keyed (2 untouched).
- ✅ No driver/CLI hard-coded `"SIFR-TYPE-0001"` test occurrences re-keyed (23 untouched).
- ✅ No bridge arm deleted (`CompilePhase::TypeCheck => "SIFR-TYPE-0001"` still live at [crates/sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)).
- ✅ No legacy `CompileError::new` removal.
- ✅ No `LoweringError → LoweringOutcome/DiagnosticSink` migration (separate slice 2d per pre-review).
- ✅ No parser bucket splitting (owned by `milestone_diag_7`).
- ✅ No decimal `[E25xx]` removal from message templates (owned by `milestone_diag_6`).
- ✅ No centralized message-prefix dispatcher reintroduced — the slice 1 R5 regression-guard test at [crates/sifr_driver/src/tests/diagnostics.rs:71-81](../crates/sifr_driver/src/tests/diagnostics.rs:71) is unchanged.

The one structural change pass 2 added is purely refactor-grade: extracting `lowering_error_to_compile_error` from the inline `Err` arm of `lower_frontend_module`. That extraction is a direct precondition of the new R1 tests (an inline closure cannot be reached by a unit test) and the resulting helper is byte-equivalent to the previous inline code path. No behavior change.

## Validation evidence (mirrors the issue tracker, signature is the gate)

| Gate | Result | Notes |
| --- | --- | --- |
| `cargo fmt --check` | ✅ | clean |
| `python3 scripts/check_hir_maintainability_guardrails.py` | ✅ | new HIR test file is small (27 lines); within guardrails |
| `cargo test -p sifr_hir diagnostic_transport_tests` | ✅ | both new HIR tests pass |
| `cargo test -p sifr_driver frontend::module_lowering::tests` | ✅ | both new driver tests pass |
| `cargo test -p sifr_driver` | ✅ | full driver suite green |
| `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` | ✅ | dead-code allow remains correctly scoped; no other lints surface |
| `scripts/run_all_tests.sh --profile quick` | ✅ | report signature `e1bf653aaa770517` (matches slice 1), wall time `86.73s` |

The signature parity is the load-bearing evidence for a transport-only slice that intentionally produces no production behavior change. Slice 2b PRs will need the full e2e suite (`scripts/run_e2e_pass.sh`) once HIR call sites start emitting active codes; slice 2a does not.

Two narrowing caveats from pass 1 still apply to the reported gates and remain acceptable:

- `cargo test -p sifr_hir` is filtered to `diagnostic_transport_tests` because of two unrelated pre-existing HIR assertion failures (`test_empty_dict_literal_conflicting_write_reports_deterministic_error`, `test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`) that reproduce on `origin/main`. The quick-profile signature match is the authoritative gate per `AGENTS.md`. Worth a one-line PR-description note ("two unrelated HIR test failures reproduce on `origin/main`; quick-profile is the authoritative gate") so reviewers don't misread the filter.
- Clippy was run on `-p sifr_hir -p sifr_driver` rather than `--workspace`. That is fine for this slice because no other crate changed, but `cargo clippy --workspace -- -D warnings` is cheap and matches the workspace-level lint gate that CI will exercise. Recommend running once before opening the PR — should be a no-op given the touched crates, but it is the gate the issue's `AGENTS.md` defaults to. *Not a blocker.*

## Remaining blockers

**None.** All pass-1 R/O/N findings are closed in code or correctly deferred to PR-description hygiene. No new findings have surfaced from re-reading the working tree.

## Recommendation

Open the slice 2a PR. Suggested PR-description bullets (carried forward from pass 1, refreshed):

- Adds `Option<DiagnosticCode>` transport to `sifr_hir::LoweringError`, additive `LowerCtx::error_with_code` (private, parity with `error`), and forwarding in `frontend/module_lowering.rs` via the new `lowering_error_to_compile_error` helper. No production HIR call sites migrate yet; that is slice 2b.
- `STDLIB_BOOTSTRAP_FAILURE` override for stdlib lowering errors is preserved by design and explicitly documented; an HIR-emitted `Some(_)` code is intentionally discarded at this site.
- Quick-profile signature `e1bf653aaa770517` matches slice 1, confirming no production behavior change. Wall-time `86.73s` is within environmental noise.
- Test surface: two HIR-side tests (`error_with_code` writes `Some(code)`, legacy `error` writes `None`) plus two driver-side tests (`Some(_)` arm renders `SIFR-TYPE-0002`, `None` arm renders the bridge `SIFR-TYPE-0001`) pin the new branch decision.
- Unrelated pre-existing HIR test failures (`test_empty_dict_literal_conflicting_write_reports_deterministic_error`, `test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`) reproduce on `origin/main` and are not introduced by this diff; quick-profile is the authoritative gate.

Suggested follow-up scoping (re-asserts the pre-review's three-sub-slice plan):

- Slice 2b: per-domain migration of `LowerCtx::error → error_with_code`, ordered decimal → ownership/flow/match/result → class/protocol/import → call/tuple/container/annotation → type/name. Each domain sub-PR re-keys its fixtures and runs the full e2e suite. The first 2b sub-PR also drops the `#[allow(dead_code)]` and the `TODO(diag_4a slice 2b)` marker on `error_with_code` once the function gains a production caller.
- Slice 2c: delete `LowerCtx::error` (codeless overload) and `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge arm; tighten `CompileError.code` to non-`Option` if parse/codegen/build paths are coded by then; re-key the 2 verification baselines and 23 driver/CLI unit-test occurrences; add the "bridge is gone" regression test.

## Summary

Slice 2a converged in two review passes. The pass-1 R1 gap (missing driver-side round-trip test for the new `Some(_) → with_code` branch) is closed by a clean refactor of the conversion logic into `lowering_error_to_compile_error` plus two paired unit tests that assert both the in-process `CompileError.code` and the rendered `to_diagnostic().code` for both arms, across both diagnostic styles. R2 (visibility), O3 (stdlib comment), O4 (orchestrator comment), and N7 (TODO marker) are all closed in place with the wording pass 1 specifically suggested. Quick-profile signature parity with slice 1 (`e1bf653aaa770517`) confirms no production behavior change, scope discipline holds (no fixture re-keying, no bridge deletion, no call-site migration, no centralized prefix dispatcher), and the patch surface remains small enough for a focused slice-2a PR. No blockers remain.
