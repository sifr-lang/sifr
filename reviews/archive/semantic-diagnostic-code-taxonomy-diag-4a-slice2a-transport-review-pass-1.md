# Review: milestone_diag_4a — Slice 2a Transport Plumbing (Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a-slice2` (working tree on top of `10129970`, the slice 1 merge)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md)

Slice scope reviewed (claimed):

1. Add `code: Option<DiagnosticCode>` to `sifr_hir::LoweringError`.
2. Add `LowerCtx::error_with_code` as additive transport plumbing, with a narrow `#[allow(dead_code)]` because no HIR domain call sites are migrated in 2a.
3. Add focused HIR tests proving coded vs codeless lowering errors are recorded distinctly.
4. Update driver frontend module-lowering to forward `LoweringError.code` to `CompileError::with_code`, preserving the legacy `CompileError::new` fallback when absent.
5. Keep stdlib lowering errors collapsed to `STDLIB_BOOTSTRAP_FAILURE` with an explicit override comment.
6. Record slice-1 PR link, slice-2a status, pre-review pointer, and validation evidence in the issue tracker.

## Verdict

**The slice is scope-disciplined and structurally correct, and is ready to ship as a small slice-2a PR after addressing one missing-test gap (R1 below).** It implements exactly the slice 2a contract from the pre-implementation review (`Option<DiagnosticCode>` field, additive `error_with_code` helper, conversion-site forwarding, `STDLIB_BOOTSTRAP_FAILURE` override preserved, no fixture re-keying, no bridge deletion, no call-site migration). Quick-profile signature is unchanged from slice 1 (`e1bf653aaa770517`), which is the correct guarantee for a transport-only slice that intentionally produces no production behavior change.

The only material concern is that the pre-review explicitly required *two* tests for slice 2a — one in `sifr_hir` and one in `sifr_driver` — and only the `sifr_hir` half landed. The `sifr_driver` half (proving `compile_errors_to_diagnostics` emits the active code rather than the `SIFR-TYPE-0001` legacy bridge when `LoweringError.code` is `Some(_)`) is the test that actually pins the new branch in [crates/sifr_driver/src/frontend/module_lowering.rs:35](../crates/sifr_driver/src/frontend/module_lowering.rs:35) against silent regression. Recommendation: add it before opening the PR.

Everything else is minor polish.

## What was actually delivered (verified by code reading + local rebuild)

| Pre-review slice 2a deliverable | Status | Evidence |
| --- | --- | --- |
| `code: Option<DiagnosticCode>` field on `LoweringError` | ✅ | [crates/sifr_hir/src/lower/mod.rs:84-89](../crates/sifr_hir/src/lower/mod.rs:84) |
| `LowerCtx::error_with_code(code, message)` added alongside legacy `error(message)` | ✅ | [crates/sifr_hir/src/lower/mod.rs:220-231](../crates/sifr_hir/src/lower/mod.rs:220) |
| Legacy `LowerCtx::error(message)` initialises `code: None` | ✅ | [crates/sifr_hir/src/lower/mod.rs:211-218](../crates/sifr_hir/src/lower/mod.rs:211) |
| `frontend/module_lowering.rs` branches on `e.code` (Some → `with_code`; None → `new`) | ✅ | [crates/sifr_driver/src/frontend/module_lowering.rs:28-40](../crates/sifr_driver/src/frontend/module_lowering.rs:28) |
| `stdlib/bootstrap.rs` keeps `STDLIB_BOOTSTRAP_FAILURE` override | ✅ | [crates/sifr_driver/src/stdlib/bootstrap.rs:64-71](../crates/sifr_driver/src/stdlib/bootstrap.rs:64) |
| `STDLIB_BOOTSTRAP_FAILURE` override has documentation comment | ✅ | same hunk; see R3 below for tightening |
| HIR test: coded `error_with_code` records `Some(code)` | ✅ | [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:4-16](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:4) |
| HIR test: legacy `error` records `None` | ✅ | [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:18-27](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs:18) |
| `sifr_driver` test: round-trip from `LoweringError { code: Some(_) }` to `CompileError::with_code` ⇒ active code in `to_diagnostic()` | ❌ **missing** | see R1 below |
| Bridge `CompilePhase::TypeCheck => SIFR-TYPE-0001` left intact | ✅ | [crates/sifr_driver/src/diagnostics.rs:135-140](../crates/sifr_driver/src/diagnostics.rs:135) |
| 90 fixtures unchanged | ✅ | git diff shows no `crates/sifr/tests/e2e/**` modifications |
| 23 driver/CLI unit-test occurrences of `SIFR-TYPE-0001` unchanged | ✅ | git diff shows none of those files modified |
| Pre-implementation review checked-in | ✅ | `reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md` is untracked but present |
| Issue tracker: slice-1 PR link recorded | ✅ | [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34) |
| Issue tracker: slice-2a status item present | ✅ | [issues/...:35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:35) (unchecked, which is correct for an in-progress slice) |
| Issue tracker: pre-review pointer | ✅ | [issues/...:37](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:37) |
| Issue tracker: validation evidence for slice 2a | ✅ | [issues/...:75-82](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:75) |
| Quick-profile signature unchanged from slice 1 | ✅ | `e1bf653aaa770517` reported, matches slice 1 |

I rebuilt `sifr_hir` in both `dev` and `release` profiles locally (clean build for the latter): both finish without warnings, which confirms the `#[allow(dead_code, reason = "…")]` annotation on `error_with_code` is correctly scoped — without it, a release build would warn (the function has no production caller yet) and clippy would fail the workspace's `-D warnings` gate.

## Answers to the user's review questions

### Q1. Is `Option<DiagnosticCode>` the right additive shape for slice 2a?

**Yes.** It is the only shape that lets slice 2a land as a small additive PR without simultaneously migrating all 489 `LowerCtx::error(...)` call sites. The pre-review ([§"Concrete pre-implementation actions before slice 2a opens", line 228](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md:228)) explicitly recommends `Option` for slice 2a and a tightening to non-`Option` in slice 2c, after every call site is migrated; the implementation matches that recommendation.

The alternative — making `code: DiagnosticCode` non-optional from day one — would force every existing `ctx.error(...)` call site to either pick a code or be temporarily routed through a transitional bucket. Picking a code per site is the correct end state but is the slice 2b workload (489 sites, ~90 fixtures); collapsing into a transitional bucket is exactly the "label-laundering" anti-pattern the issue forbids and which slice 1 just removed. The `Option` shape preserves the correct compile-time behavior of *every existing call site* (they continue to record `code: None`, fall through the bridge arm, and remain `SIFR-TYPE-0001`) while giving 2b sub-PRs a place to plug in active codes one domain at a time.

A subtler reason this is the right shape: with `Option`, the bridge arm in [crates/sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) becomes the *exact* signal of "this site has not been migrated yet." Slice 2c's deletion of that arm is then equivalent to the type-system enforcement of "every HIR error carries a code" — which is the issue's [line 863](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:863) "must use an inventory-assigned canonical code … or fail to compile" requirement. The slice plan composes cleanly toward that gate.

### Q2. Is the `#[allow(dead_code)]` allowance on `error_with_code` acceptable as a transitional marker?

**Yes, with caveats.** The allow-attribute is necessary in non-test builds (no production caller exists yet — the function would otherwise trip the workspace's `unused` lints under `-D warnings` for clippy), and the explicit `reason = "diag_4a slice 2a adds transport before per-domain HIR call-site migration"` text makes the transitional intent legible to future readers. I rebuilt `release` locally (no test cfg) and confirmed the function compiles cleanly because of the allow.

Two caveats worth tracking, neither blocking:

- **N7 (auto-removal as a slice-2b reminder).** When the first slice 2b sub-PR migrates a call site to `error_with_code`, the `#[allow(dead_code)]` becomes unnecessary; clippy will warn that the allow is itself unused. The reason text already serves as a passive hint, but a one-line `TODO(diag_4a slice 2b): drop this allow when the first call site migrates` next to the attribute would make the cleanup obligation explicit. Optional polish — not a blocker.
- **R2 (visibility inconsistency).** `error_with_code` is `pub(super)` while the existing `error` is plain `fn`. The existing `error` is callable from every `lower::*` submodule via Rust's descendant-private access, so plain `fn` is sufficient. Either both should be `pub(super)` or both should be plain `fn`; mixing is a small inconsistency. The minimum-scope and stylistic-parity choice is plain `fn error_with_code`. Not a correctness issue, and the wider visibility does not leak symbols outside the crate (still crate-internal because `LowerCtx` itself is `pub(super)`). Drop `pub(super)` for parity, or accept the wider visibility deliberately.

### Q3. Does module-lowering forwarding preserve behavior for `None` and correctly use active codes for `Some`?

**Yes.** The branch in [crates/sifr_driver/src/frontend/module_lowering.rs:28-40](../crates/sifr_driver/src/frontend/module_lowering.rs:28) is structurally correct:

```rust
let message = match diagnostic_style {
    FrontendDiagnosticStyle::Bare => e.message,
    FrontendDiagnosticStyle::ModulePrefixed => {
        format!("[{}] {}", module_name, e.message)
    }
};
if let Some(code) = e.code {
    CompileError::with_code(message, CompilePhase::TypeCheck, code)
} else {
    CompileError::new(message, CompilePhase::TypeCheck)
}
```

- **None branch** constructs `CompileError::new(...)`, which sets `code: None`. `to_diagnostic()` then falls through to the bridge arm `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` — identical to slice 1 behavior. No fixture, baseline, or driver/CLI test is affected.
- **Some branch** constructs `CompileError::with_code(_, _, code)`, which sets `code: Some(code)`. `to_diagnostic()` short-circuits at [diagnostics.rs:126-127](../crates/sifr_driver/src/diagnostics.rs:126) and emits `code.code()` — the active inventory string. This is the path that will start carrying production load in slice 2b once HIR call sites migrate to `error_with_code`.

Two transitive checks that this slice gets right by virtue of *not* touching them:

- The test-runner orchestrator path at [crates/sifr_driver/src/test_runner/orchestrator.rs:108-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:108) takes the `Vec<CompileError>` *already produced by* `lower_frontend_module` and re-wraps it; it does not re-classify or strip codes. So whatever code `module_lowering.rs` set is preserved. The pre-review listed this as one of the three forwarding sites; in practice the slice 2a transport plumbing is satisfied without modifying it because the conversion already lives upstream. (See O4 below for a stale-comment nit at this site.)
- The stdlib path at [crates/sifr_driver/src/stdlib/bootstrap.rs:60-75](../crates/sifr_driver/src/stdlib/bootstrap.rs:60) intentionally collapses to `STDLIB_BOOTSTRAP_FAILURE`; that is the documented override and is the right behavior (see Q4).

### Q4. Is the stdlib bootstrap override documented clearly enough?

**Adequately, but the comment can be tightened.** The current text:

```rust
// User-facing HIR codes are intentionally collapsed
// here: stdlib lowering failures are compiler
// bootstrap failures from the caller's perspective.
```

This explains the *philosophy* (stdlib is a compiler-internal concern, not a user-facing semantic error). What it does not say explicitly is the *operational* consequence: when slice 2b lands and `e.code` is `Some(SIFR-TYPE-0002)` etc., this site **discards** that code and substitutes `STDLIB_BOOTSTRAP_FAILURE`. A future reader looking at the code will not see `e.code` referenced anywhere in the body, which is the actual signal that the override is happening, but the comment doesn't make the override explicit.

Recommendation (R3, optional polish): tighten to something like:

> Even if `e.code` is `Some(_)`, this collapses to `STDLIB_BOOTSTRAP_FAILURE`: stdlib lowering failures are compiler bootstrap failures from the caller's perspective, not user-facing semantic diagnostics.

The `e.code` mention is the load-bearing piece — it tells a 2b reviewer that this site is a deliberate exception to the "forward HIR code" contract that `lower_frontend_module` follows.

A separate observation: [crates/sifr_driver/src/stdlib/bootstrap.rs:30-31, 48-49](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) already carries `TODO(diag_4a slice 2): classify Ruff parse failures …` markers on the stdlib *parse* failure paths. Those TODOs survive slice 2a unchanged and are correctly scoped (slice 2a does not touch parse classification — that is owned by [milestone_diag_7 line 941-943](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:941) per the pre-review). The stdlib lowering site does not have an analogous TODO because the override is the intended permanent behavior, not a transitional state. Consistent.

### Q5. Is validation sufficient for this no-production-behavior slice?

**Sufficient with one caveat.** The validation budget the pre-review prescribed for slice 2a ([line 239](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md:239)) is exactly:

- `cargo test -p sifr_hir`, `cargo test -p sifr_driver`, `cargo clippy --workspace -- -D warnings`, `scripts/run_all_tests.sh --profile quick`
- "Quick-profile should match `e1bf653aaa770517` (no behavior change)."

The reported evidence (issue tracker line 75-82) covers:

- `cargo fmt --check` ✅
- `python3 scripts/check_hir_maintainability_guardrails.py` ✅ (extra, slice-appropriate because the patch adds an HIR test file)
- `cargo test -p sifr_hir diagnostic_transport_tests` ✅ (subset; see caveat)
- `cargo test -p sifr_driver` ✅
- `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` ✅ (narrower than the pre-review's `--workspace`; see caveat)
- `scripts/run_all_tests.sh --profile quick` ✅, signature `e1bf653aaa770517` matches slice 1 — this is the headline evidence that no production behavior changed.

I re-ran `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr_driver`, and `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` locally; all pass. The signature match for `--profile quick` is the single strongest signal for a transport-only slice and is the right gate.

Caveats:

- **The reported `cargo test -p sifr_hir` is filtered to `diagnostic_transport_tests` only.** The user's note about the two unrelated pre-existing HIR assertion failures (`test_empty_dict_literal_conflicting_write_reports_deterministic_error`, `test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`) explains why a full `cargo test -p sifr_hir` was not the gate. That is acceptable here because (a) those failures reproduce on `origin/main` and are not introduced by this diff, (b) the workspace `--profile quick` lane is the authoritative gate per `AGENTS.md`, and (c) it ran clean. The slice-2a PR description should call this out explicitly so reviewers don't assume the partial filter is hiding new failures: "the two failing HIR tests reproduce on `origin/main` at `<sha>` and are unrelated to this diff; quick-profile is the authoritative gate." Optional but recommended for PR hygiene.
- **Clippy was run on `-p sifr_hir -p sifr_driver`, not `--workspace`.** That is fine for slice 2a because no other crate changed, but `cargo clippy --workspace -- -D warnings` is cheap and matches the workspace-level lint gate exercised by CI. Recommend running once before opening the PR — should be a no-op given the touched crates, but it is the gate the issue's `AGENTS.md` defaults to.

For a transport-only slice with no fixture or baseline changes, the quick-profile signature match is the load-bearing evidence. Slice 2b PRs will need the full e2e suite (`scripts/run_e2e_pass.sh`) per the pre-review's [§"Validation budget" 2b row](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md:240); slice 2a does not.

## Findings

Severity legend: **R** = recommended before PR opens (test or correctness gap); **O** = optional polish; **N** = note for the PR description or future slice.

### R1. Missing driver-side round-trip test for the new `Some(code)` branch

The pre-review explicitly required ([§"Slice 2a — Transport plumbing", step 4, line 162](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md:162)):

> Add a unit test in `sifr_hir` proving `LoweringError { code: Some(SIFR-TYPE-0002), .. }` round-trips to a `CompileError::with_code`, and a sibling test in `sifr_driver` proving `compile_errors_to_diagnostics` emits the active code (not `SIFR-TYPE-0001`) when `error.code` is `Some(_)`.

The HIR side landed (the two tests in [diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) cover the field-level invariants), but the driver-side counterpart is absent. The HIR tests prove `LowerCtx::error_with_code` populates `LoweringError.code`; they do **not** prove that `lower_frontend_module` correctly forwards that code into a `CompileError::with_code` rather than the legacy `CompileError::new` fallback.

Why this matters concretely: the new branch in [module_lowering.rs:35-39](../crates/sifr_driver/src/frontend/module_lowering.rs:35) is the *only* production code path that this slice changes. Without a unit test exercising both arms (`Some` → `with_code`, `None` → `new`), a future refactor could collapse the branch back to the legacy single-arm form and the whole test suite would still pass — because no migrated HIR call site exists yet to fail integration. The test is cheap (a small synthetic call to `lower_frontend_module` with a hand-crafted source that triggers a known HIR error path is overkill; a more targeted test that constructs a `CompileError` directly from a `LoweringError` is sufficient — but the cleanest version uses the public `lower_frontend_module` entry point).

Recommended shapes (any one suffices):

1. **Targeted unit test in `crates/sifr_driver/src/tests/diagnostics.rs`.** Construct a `CompileError::with_code` with `DiagnosticCode::TYPE_MISMATCH` and assert `to_diagnostic().code == "SIFR-TYPE-0002"` (already covered transitively by `test_compile_errors_to_diagnostics_preserves_order`); and a paired `CompileError::new(_, CompilePhase::TypeCheck)` whose `to_diagnostic().code == "SIFR-TYPE-0001"` (also already covered transitively because the bridge arm is exercised by every legacy site). What is *not* covered today and is the actual missing test is the **`lower_frontend_module` branch decision** — a test that drives a `LoweringError { code: Some(_) }` through `lower_frontend_module` and asserts the resulting `CompileError` carries `code: Some(_)`, plus a sibling `LoweringError { code: None }` case asserting the resulting `CompileError` carries `code: None`. This is a `crates/sifr_driver/src/frontend/module_lowering.rs` `#[cfg(test)]` test or a sibling in `crates/sifr_driver/src/tests/`.
2. **End-to-end-ish test using a real HIR error site that already exists.** Pick any current `ctx.error(...)` site (still produces `code: None`) and any test source that triggers it; assert the `to_diagnostic().code == "SIFR-TYPE-0001"` (the legacy fallback is alive). For the `Some` arm, *temporarily* call `error_with_code` from within a test-only scaffolding helper. This is heavier than option 1 and probably not worth it for slice 2a.

Option 1 is the smaller, slice-2a-appropriate choice. The literal one-line behavior to pin is "`if let Some(code) = e.code` correctly routes to `with_code`" — anything that mechanically asserts that is sufficient.

### R2. Visibility inconsistency between `error` and `error_with_code`

[crates/sifr_hir/src/lower/mod.rs:211 vs 220-224](../crates/sifr_hir/src/lower/mod.rs:211)

```rust
fn error(&mut self, message: String) { ... }                         // crate-private + descendant access

#[allow(dead_code, reason = "...")]
pub(super) fn error_with_code(&mut self, code: DiagnosticCode, message: String) { ... }
```

`pub(super)` from `lower::mod.rs` widens the visibility to `sifr_hir`'s root module, while `fn error` (no qualifier) is private to `lower` with the usual descendant-access carve-out — that's why every `lower::*` submodule can call `ctx.error(...)`. Both forms reach the same set of slice-2b call sites (which all live under `lower::*`).

Two ways to fix:

- **Drop `pub(super)`** to plain `fn error_with_code` for parity with the existing `error`. This is the minimum-scope choice and matches the existing codebase style. **Recommended.**
- Keep `pub(super)` and *also* widen `error` to `pub(super)` in a same-PR consistency pass. Bigger blast radius for a stylistic fix; not recommended for slice 2a.

This is a style-and-consistency finding, not a correctness one. The wider visibility does not leak symbols outside the crate (`LowerCtx` is `pub(super)`).

### O3. Stdlib override comment can be tightened

[crates/sifr_driver/src/stdlib/bootstrap.rs:64-66](../crates/sifr_driver/src/stdlib/bootstrap.rs:64)

The current comment explains why stdlib lowering errors collapse to `STDLIB_BOOTSTRAP_FAILURE` (philosophy: stdlib is bootstrap, not user-facing). It does not state explicitly that `e.code` is discarded. When slice 2b lands and HIR call sites start emitting structured codes for stdlib lowering errors, a 2b reviewer reading this site will see no reference to `e.code` at all and may wonder whether the omission is a bug.

Recommended tightening:

> Even if `e.code` is `Some(_)`, this collapses to `STDLIB_BOOTSTRAP_FAILURE`: stdlib lowering failures are compiler bootstrap failures from the caller's perspective, not user-facing semantic diagnostics.

The "Even if `e.code` is `Some(_)`" clause is the load-bearing addition.

Optional. The current comment is already adequate as transitional documentation.

### O4. Stale comment in test_runner/orchestrator.rs

[crates/sifr_driver/src/test_runner/orchestrator.rs:113-115](../crates/sifr_driver/src/test_runner/orchestrator.rs:113)

```rust
// Preserves None until HIR LoweringError carries a
// structured code in the next diag_4a slice.
code: error.code,
```

This comment was written in slice 1 to document a forward-reference to slice 2. Slice 2a now lands the *plumbing* but no production HIR call site emits a code yet — that is slice 2b. So strictly the comment is still accurate (the field is still typically `None` for production paths), but the phrase "the next diag_4a slice" is now ambiguous: slice 2a *is* the next slice and it did add the plumbing but didn't fill in any active codes. A 2b reviewer reading this might misinterpret it.

Recommended phrasing (optional polish):

> Forwards `LoweringError.code` faithfully — `None` for legacy unmigrated call sites, `Some(_)` once HIR call sites migrate in upcoming slice 2b sub-PRs.

Slice 2a does not need to repaint this comment, but a one-line touch-up here is a low-cost win and keeps the slice-2a PR scope honest about what changed in adjacent files.

### O5. Wall-time variance in `--profile quick`

Slice 1 reported `79.70s`; slice 2a reports `131.56s` for the same `e1bf653aaa770517` signature. The signature match is the load-bearing assertion (no production behavior change); the wall-time delta is environmental (machine load, cold artifact cache, parallel jobs). Worth a one-line PR-description note ("wall-time variance is environmental; signature is the gate") so reviewers don't read the 65% wall-time jump as a regression.

### N6. Issue tracker checkbox state for slice 2a

[issues/...:35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:35) reads:

```
- [ ] Started `milestone_diag_4a` slice 2a: additive HIR `LoweringError` structured diagnostic-code transport plumbing.
```

This is the right state for an in-progress slice. Slice 1's bullet at line 34 is `[x]` with merged-PR link; slice 2a should follow the same pattern once merged. Just a note: when the slice 2a PR opens, leave the bullet `[ ]` until merge; on merge, flip to `[x]` and append `PR: https://github.com/sifr-lang/sifr/pull/<n>.` Consistent with the slice-1 bullet style.

The deferred-bridge-deletion bullet at line 36 is now `[x]` with the rationale "deferred to later milestone_diag_4a slice-2 sub-PRs, where HIR call sites will be migrated by domain before the bridge is removed." That is correct framing and matches the pre-review's three-sub-slice plan.

### N7. The `#[allow(dead_code)]` becomes redundant in the first slice 2b sub-PR

When slice 2b's first migration PR (decimal, per the pre-review's recommended ordering) lands, `error_with_code` will gain a production caller and the `#[allow]` will be unused. Clippy's `unused_attributes` will flag it. Slice 2b's first sub-PR should remove the attribute as part of the same diff. A passive `TODO(diag_4a slice 2b): drop this allow when the first call site migrates` next to the attribute would make the obligation explicit; the current `reason = "diag_4a slice 2a adds transport before per-domain HIR call-site migration"` already implies it but is not as actionable. Optional polish; not a blocker.

## Scope discipline check

The slice does **not** do any of the things the pre-review flagged as out-of-scope for slice 2a:

- ✅ No HIR call sites migrated (489 sites untouched).
- ✅ No fixtures re-keyed (~90 untouched).
- ✅ No verification baselines re-keyed (2 untouched).
- ✅ No driver/CLI hard-coded `"SIFR-TYPE-0001"` test occurrences re-keyed (23 untouched).
- ✅ No bridge arm deleted (`CompilePhase::TypeCheck => "SIFR-TYPE-0001"` still live).
- ✅ No legacy `CompileError::new` removal.
- ✅ No `LoweringError → LoweringOutcome/DiagnosticSink` migration (separate slice 2d per pre-review).
- ✅ No parser bucket splitting (owned by `milestone_diag_7`).
- ✅ No decimal `[E25xx]` removal from message templates (owned by `milestone_diag_6`; slice 2a only adds the *transport* shape).
- ✅ No centralized message-prefix dispatcher reintroduced (slice 1's R5 regression-guard test at [crates/sifr_driver/src/tests/diagnostics.rs:71-81](../crates/sifr_driver/src/tests/diagnostics.rs:71) is unchanged).

The patch surface is exactly the four files the pre-review's slice 2a section specified, plus the issue tracker and the new HIR test module. No incidental refactors, no commented-out code, no introduced abstractions. This is the right size for a slice-2a PR (~30-line diff in the production source plus the test module).

## Readiness

The slice is ready to ship as a small slice-2a PR after addressing **R1** (the missing driver-side round-trip test). Without R1, a future change to `lower_frontend_module` could silently regress code-forwarding because no unit test exercises the new branch. R2 (visibility inconsistency) is recommended for stylistic parity but does not affect correctness; O3, O4, O5, N6, N7 are polish.

Suggested PR description bullets:

- Adds `Option<DiagnosticCode>` transport to `sifr_hir::LoweringError`, additive `LowerCtx::error_with_code`, and forwarding in `frontend/module_lowering.rs`. No production HIR call sites migrate yet; that is slice 2b.
- `STDLIB_BOOTSTRAP_FAILURE` override for stdlib lowering errors is preserved by design; documented inline.
- Quick-profile signature `e1bf653aaa770517` matches slice 1, confirming no production behavior change.
- The two pre-existing HIR assertion failures (`test_empty_dict_literal_conflicting_write_reports_deterministic_error`, `test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`) reproduce on `origin/main` and are not introduced by this diff; the authoritative gate is `scripts/run_all_tests.sh --profile quick`.

Suggested follow-up scoping (re-asserts the pre-review's three-sub-slice plan):

- Slice 2b: per-domain migration of `LowerCtx::error → error_with_code`, ordered decimal → ownership/flow/match/result → class/protocol/import → call/tuple/container/annotation → type/name. Each domain sub-PR re-keys its fixtures and runs the full e2e suite.
- Slice 2c: delete `LowerCtx::error` (codeless overload) and `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge arm; tighten `CompileError.code` to non-`Option` if parse/codegen/build paths are coded by then; re-key the 2 verification baselines and 23 driver/CLI unit-test occurrences; add the "bridge is gone" regression test.

## Summary

Slice 2a is a tight, scope-disciplined transport-plumbing slice that lands exactly the data-structure changes the pre-implementation review prescribed for sub-slice 2a, with the stdlib override correctly preserved and an explicit dead-code allowance making the additive nature visible. Quick-profile signature parity with slice 1 confirms no production behavior change, which is the correct guarantee for this slice. The single material gap is the absent driver-side round-trip test (R1) for the new `Some(_) → CompileError::with_code` forwarding branch — adding it before the PR opens closes the regression-guard hole and brings the test surface fully in line with the pre-review's spec. Beyond that, R2 (visibility inconsistency), O3 (stdlib comment tightening), O4 (stale orchestrator comment), O5 (wall-time PR note), N6 (issue checkbox flip on merge), and N7 (TODO next to the dead-code allow) are minor polish that the implementer can address inline or defer.
