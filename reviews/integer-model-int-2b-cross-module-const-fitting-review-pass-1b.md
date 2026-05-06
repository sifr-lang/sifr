# Review — INT-2B Cross-Module Const Fitting (pass 1b)

Branch: `int-2b-cross-module-const-fitting`
Reference: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](../issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), [internal_docs/integer_model.md](../internal_docs/integer_model.md)
Reviewer scope: correctness, regressions, missing tests, API contract, panic / no-user-path violations, PR readiness.

## Scope verified

The diff threads compile-time `BigInt` values for module-level integer constants through the producer-side export API and the consumer-side import resolution paths, so the existing `validate_fixed_width_initializer` path can fold imported constants:

- HIR-side wiring
  - [crates/sifr_hir/src/lower/mod.rs:455](../crates/sifr_hir/src/lower/mod.rs:455) — `LoweringResult.constant_integer_values: HashMap<String, BigInt>`.
  - [crates/sifr_hir/src/lower/mod.rs:475](../crates/sifr_hir/src/lower/mod.rs:475) — `ExternalDefs.constant_integer_values: HashMap<String, HashMap<String, BigInt>>`.
  - [crates/sifr_hir/src/lower/mod.rs:1166](../crates/sifr_hir/src/lower/mod.rs:1166) — final `LoweringResult` carries `ctx.const_integer_values.clone()`.
- Import resolution (three sites)
  - [crates/sifr_hir/src/lower/imports.rs:104](../crates/sifr_hir/src/lower/imports.rs:104) — early `from M import X [as Y]` pass, alias-aware via `local_name_for(name)`.
  - [crates/sifr_hir/src/lower/mod.rs:929](../crates/sifr_hir/src/lower/mod.rs:929) — stdlib full-import branch.
  - [crates/sifr_hir/src/lower/mod.rs:1083](../crates/sifr_hir/src/lower/mod.rs:1083) — local-module full-import branch.
  - [crates/sifr_hir/src/lower/compat_imports.rs:158](../crates/sifr_hir/src/lower/compat_imports.rs:158) — synthetic stdlib alias path used by compatibility shims.
- Producer-side gating
  - [crates/sifr_driver/src/project/exports.rs:93](../crates/sifr_driver/src/project/exports.rs:93) iterates `module.constants` (already filtered to non-`_`-prefix and only present when the producer module proved const-evaluability via `module_constants_lowering::collect_annotated_constant`/`collect_bare_constant`), and only writes through to `external_defs.constant_integer_values` when an integer value is present.
- Plumbing
  - [crates/sifr_driver/src/project/frontend.rs](../crates/sifr_driver/src/project/frontend.rs) threads the new field through the three destructure/reconstruct sites used by the test, single-file, and source-mode entry points.
  - [crates/sifr_driver/src/build/entrypoint.rs:285](../crates/sifr_driver/src/build/entrypoint.rs:285) re-emits an empty map for the resynthesized main `LoweringResult`. Codegen does not consume this field (`grep` shows no reads outside `lower/`/`project/`), so this is correct.
- Tests
  - [crates/sifr_driver/src/tests/project_graph.rs:588](../crates/sifr_driver/src/tests/project_graph.rs:588) extends the existing local-constants export test to assert `external_defs.constant_integer_values["constants_mod"]["ANSWER"] == 42`.
  - [crates/sifr_driver/src/tests/project_graph.rs:602](../crates/sifr_driver/src/tests/project_graph.rs:602) `test_project_lowering_fits_imported_integer_constants` proves the fitted let body collapses `LIMIT + 1` to `HirExpr::IntLiteral(255)` under the alias `LIMIT` for `BASE = 250 + 4`.
  - [crates/sifr_driver/src/tests/project_graph.rs:644](../crates/sifr_driver/src/tests/project_graph.rs:644) `test_project_lowering_does_not_fold_shadowed_imported_integer_constant` asserts that an inner local rebinding `BASE: int = 100` blocks the fold and surfaces `SIFR-TYPE-0002`.
- E2E fixture conversion
  - [crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr](../crates/sifr/tests/e2e/fail/fixed_width_const_expression_out_of_range.sifr) now uses canonical leading-line `# expect-error[col=N]: SIFR-INT-…` annotations.

## Correctness assessment

### Producer side

- Only public, const-evaluable module constants are exported. `collect_annotated_constant` only adds an entry to `module.constants` when `lower_integer_const_expr_simple` returns `Some` (literal/unary/binop tree over int literals) and the resulting initializer either fits the annotated type or is non-fixed-width [crates/sifr_hir/src/lower/module_constants_lowering.rs:36](../crates/sifr_hir/src/lower/module_constants_lowering.rs:36). `collect_bare_constant` follows the same gate. Because `remember_module_const_integer` runs the same `const_integer_value` evaluator that fitting uses, it only stores values within the 4096-decimal-digit budget — over-budget results never reach `ctx.const_integer_values`, so the export will not propagate something the consumer cannot legally fold either. This satisfies the contract in [internal_docs/integer_model.md:101](../internal_docs/integer_model.md:101).
- Producer-side ordering inside a module is sound. Imports are processed first (insert imported value), then `collect_module_constants` runs and may overwrite the same name with the local module-level value. So a redeclaration like `from M import BASE` followed by `BASE: int = 100` ends with `ctx.const_integer_values["BASE"] == 100`, which is also what gets exported. `module.constants` is the export key, so a stale import-time value is never re-exported as the local module's constant.

### Consumer side

- Each of the four insertion sites uses the alias-aware local name (`local_name_for(name)` / the `alias` parameter passed to `ensure_synthetic_stdlib_import`). The positive test exercises the alias case (`from constants_mod import BASE as LIMIT` → `value: uint8 = LIMIT + 1` folds to `255`).
- Lookup uses `externals.constant_integer_values.get(module_key).and_then(|m| m.get(name))`. When the producer is a non-int constant or stdlib module without integer values, the lookup is `None` and nothing changes — the existing scope/type wiring is unaffected.
- Shadowing safety is delegated to the existing `is_shadowed_by_inner_scope` check in `fixed_width_fitting::const_integer_value` ([crates/sifr_hir/src/lower/fixed_width_fitting.rs:109](../crates/sifr_hir/src/lower/fixed_width_fitting.rs:109)). When fitting `BASE + 1` while the function frame defines a local `BASE`, the lookup returns `Unsupported` and `validate_fixed_width_initializer` falls through to a regular type check, producing `SIFR-TYPE-0002`. The new shadow test confirms this end-to-end through the project lowering pipeline.

### Lifetime / staleness of the global map

`ctx.const_integer_values` is a flat per-module `HashMap`, not a frame-stacked structure. That is acceptable here because the only writers are: (a) import resolution (executed once, per-module) and (b) `remember_module_const_integer`, which runs only at module top level. Function-scope locals never insert into the map; they instead suppress folding via `is_shadowed_by_inner_scope`. So the map cannot accumulate stale per-function state across nested function bodies, and the absence of a "pop on scope exit" is intentional.

### Diagnostic / panic surface

No new `unwrap`/`expect` is added on user-reachable paths. All new lookups use `get(...).and_then(...)`. Cloning a `BigInt` is allocating but infallible. No new monolithic file or HIR guardrail risk — only field additions to existing structs.

### E2E fixture

- `parse_expect_error_line` only matches lines that *start* with `# expect-error:` or `# expect-error[`. The original trailing form (`too_wide: uint8 = 2 ** 8  # expect-error: SIFR-INT-0001 col=23`) was therefore silently ignored by the harness — converting to leading-comment form makes the fixture actually enforce the codes. This is a real (pre-existing) fix.
- Column counts verified against the source: column 23 is the `2` in `2 ** 8`, column 24 is the `1` in `10 ** 5000`. Both diagnostic codes (`SIFR-INT-0001` for fixed-width out-of-range; `SIFR-INT-0004` for the budget-exceeded `10 ** 5000`) are active in the registry [crates/sifr_diagnostics/src/codes.rs:62](../crates/sifr_diagnostics/src/codes.rs:62), [crates/sifr_diagnostics/src/codes.rs:64](../crates/sifr_diagnostics/src/codes.rs:64).

## Notes / suggested follow-ups (non-blocking)

1. **Stdlib bootstrap does not propagate `constant_integer_values`.** [crates/sifr_driver/src/stdlib/bootstrap.rs](../crates/sifr_driver/src/stdlib/bootstrap.rs) collects exports through a bespoke loop that *does not* call `collect_module_exports`, and it never populates `stdlib_defs.constant_integer_values`. As a result, importing a const-evaluable integer constant from any stdlib module will not fold into a fixed-width target — only user-project modules participate. This is consistent with the slice description ("…through the existing sifr_driver frontend/project API"), but it leaves a real asymmetry: a user module exposing `BASE: int = 254` participates, while the same constant exposed from a stdlib module does not. Worth tracking as a follow-up so stdlib integer constants get the same treatment (or worth an explicit decision that stdlib stays opted-out).
2. **Shadowing test only covers declare-before-use.** The new shadow test verifies the local rebinding precedes the use site. It does not cover the case where the local rebinding occurs *after* a use of `BASE` in the same function body. Whether Sifr's resolver hoists the local binding (Python-style "function scope name capture") or treats the pre-declaration use as the imported binding is a deeper resolver question and probably out of scope for this slice — but it is a latent edge case that the current test does not pin down.
3. **Fixture style divergence.** The sibling `fixed_width_literal_out_of_range.sifr` places both `expect-error` annotations at the top of file before the `def main():`. The new fixture uses inline-leading comments adjacent to each erroring line. Both forms are accepted by `parse_expect_error_line` and both are enforced; this is a stylistic note rather than a correctness issue.
4. **Plumbing duplication.** Three sites in `project/frontend.rs` destructure-then-reconstruct the `LoweringResult` only to clone `module`. The new field threads through cleanly, but a one-line `let lowering_result = result.clone();` would be simpler. Pure style; not in scope.
5. **Re-export semantics.** The current export rule is "export only constants the producer itself defined." Transitive re-export (e.g., `consumer.sifr` doing `from constants_mod import ANSWER` and then importing into `main`) is not honored — `main` must import `ANSWER` directly from `constants_mod`. That matches `internal_docs/integer_model.md:101` ("Imported immutable module constants may carry const-evaluable status across module boundaries…"), which is silent on transitive re-export. Worth noting for future docs but not a defect.

## Validation review

The reported local validation set covers the relevant surfaces:
- `cargo test -p sifr_driver project_lowering` exercises the three new project-graph tests.
- `cargo test -p sifr_hir fixed_width` exercises the fitting evaluator.
- `cargo test -p sifr --test e2e test_e2e_fail` exercises the converted fixture.
- `cargo clippy -p sifr_hir -p sifr_driver -- -D warnings` with workspace pedantic lints.
- `scripts/run_all_tests.sh --profile quick` (signature `e1bf653aaa770517`, ~73s) provides whole-tree signal.

No additional validation gaps were identified for the slice's stated scope.

## Readiness

The slice is focused, the data flow is sound, the producer-side gating is conservative (tied to `module.constants`, the same set that already passed const-evaluability), the consumer-side preserves shadowing safety via the existing mechanism, and the tests cover both the happy path (with alias) and the negative path (shadow). The e2e fixture conversion is correct and converts a previously-silent annotation form into one the harness actually enforces. The stdlib propagation gap is the only substantive open question and is best handled as a follow-up rather than reopening this slice.

VERDICT: SATISFIED
