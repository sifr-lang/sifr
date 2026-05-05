# `milestone_diag_4a` slice 2b.33 — Remove pre-1.0 retired diagnostic-code lifecycle and phase-derived `CompileError` fallbacks

Pass 1 review of the uncommitted working tree on branch
`codex/diag-4a-remove-retired-fallbacks`.

## Scope under review

- Delete the `DiagnosticState::Retired` variant, the `DiagnosticRegistryEntry::replacement` field, the `retired_entry!` macro, and the four retired registry entries (`SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`) from [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs).
- Strip the "Retired Codes" section from the public docs index, the `Replacement` column from the internal registry table, and the `Retired` legend line from `gen-error-docs` ([crates/sifr_diagnostics/src/bin/gen-error-docs.rs](crates/sifr_diagnostics/src/bin/gen-error-docs.rs)).
- Remove the `CompileError::new(...)` legacy constructor and the `CompilePhase`-keyed fallback (`Parse → SIFR-PARSE-0001`, `TypeCheck → SIFR-TYPE-0001`, `Codegen → SIFR-CODEGEN-0001`, `Build → SIFR-BUILD-0001`); promote `CompileError.code` from `Option<DiagnosticCode>` to `DiagnosticCode` ([crates/sifr_driver/src/diagnostics.rs:27-31, 95-110](crates/sifr_driver/src/diagnostics.rs:27)).
- Route any `LoweringError` whose `code` is `None` through `DiagnosticCode::INTERNAL_COMPILER_PANIC` (`SIFR-INTERNAL-0001`) at the frontend boundary via a new `lowering_error_code_or_internal` helper ([crates/sifr_driver/src/frontend/module_lowering.rs:42-56](crates/sifr_driver/src/frontend/module_lowering.rs:42)).
- Migrate all remaining call-sites of `CompileError::new` (`stdlib/cache.rs` test sentinel → `STDLIB_CACHE_FAILURE`; `main.rs` exit-code test → `TYPE_MISMATCH`) and update the literal struct-construction sites in `stdlib/bootstrap.rs` and `test_runner/orchestrator.rs` to drop the `Option` wrapper.
- Re-key 10 e2e fail fixtures that previously expected `SIFR-TYPE-0001` to instead expect `SIFR-INTERNAL-0001`:
  - `islice_non_iterable_input.sifr`, `iter_heterogeneous_tuple_unsupported.sifr`, `mutable_list_variance_invariant.sifr`, `reversed_iterator_not_reversible.sifr`, `reversible_annotation_rejects_set.sifr`, `stdlib_counter_wrong_type.sifr`, `stdlib_test_assert_eq_type_mismatch.sifr`, `stdlib_wrong_type.sifr`, `tuple_dynamic_list_shape.sifr`, `unsupported_default_expr_call.sifr`.
- Re-key the harness contract self-tests (`test_expectation_parsing_contract`, `test_expected_error_contract_with_messages`) and several compact-renderer tests in `crates/sifr/src/main.rs` and `crates/sifr_driver/src/tests/diagnostics.rs` from the retired `SIFR-*-0001` catch-alls to active equivalents (`SIFR-PARSE-0002`, `SIFR-TYPE-0002`, `SIFR-CODEGEN-0002`).
- Update / un-defer the issue-tracker checklist (slice 2b.32 marked merged at #1704; new slice 2b.33 entry; previously deferred TypeCheck-bridge deletion is now superseded), the language-level notes ("retired" → "removed before public stability"), the inventory tables, the architecture doc note, the generated public/internal docs indexes, and the registry policy text.

## Verdict

**Changes-requested.** The mechanical refactor — registry cleanup, doc/index regeneration, `Option` removal, generator update, and the registry/CompileError surface — is internally consistent, byte-for-byte aligned with the new pre-1.0 stability decision, and well covered by the existing schema/registry unit tests, the new `coded_lowering_error_uses_active_diagnostic_code` / `codeless_lowering_error_is_internal_compiler_diagnostic` pair, and the local validation envelope reported. The blocking issue is **not** the deletion itself; it is the chosen *replacement semantics* for the deleted phase-bucket arm.

By routing every codeless `LoweringError` through `DiagnosticCode::INTERNAL_COMPILER_PANIC`, the slice (a) directly contradicts a hard rule that this same issue tracker still owns — "**Known user-input failures must never be routed through `SIFR-INTERNAL-*`**" ([issues/...md:1338](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1338)) and "Internal compiler failure boundaries... must not mask a known user-input error that should have a specific code" ([issues/...md:1331](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1331)) — and (b) introduces an externally observable CLI exit-code regression for ten currently-failing programs (verified by running the binary directly: see Finding #2 below). I would not merge as-is. Pick one of:

1. Migrate the (small, finite) set of `ctx.error(format!(…))` sites underneath the affected fixtures to `error_with_code(DiagnosticCode::TYPE_MISMATCH, …)` *in this slice*, so the CompileError fallback can be deleted with **zero** unmigrated paths surviving the boundary; or
2. Revert the codeless-`LoweringError` → `INTERNAL_COMPILER_PANIC` mapping to a *programmer-error* (debug-assert / panic in test, hard `error_with_code(DiagnosticCode::INTERNAL_COMPILER_PANIC, …)` only when truly absent of a HIR error) so the visibility comes from build-time enforcement rather than from misclassifying user errors at runtime; or
3. Defer the bridge deletion until the affected emission sites are coded — the same deferral the issue tracker explicitly authorized at the prior checklist line and which is now being superseded *before* the underlying migrations have actually landed.

Everything else in the diff is approve-on-sight. The findings below are split into the blocking semantic concern, supporting evidence, and the smaller items I would still flag if (1) lands.

## What I checked

### 1. Registry surface deletion is clean and self-consistent
[crates/sifr_diagnostics/src/codes.rs:155-170, 215-228, 341-360, 370-400, 1395-1410, 1480-1495, 1620-1640](crates/sifr_diagnostics/src/codes.rs:155)

- `DiagnosticState` collapses cleanly to `{ Active, Reserved }`. The `as_str()` match is exhaustive on the new closed set. The registry-validation test at `mod tests` correctly drops the `DiagnosticState::Retired => {}` arm at codes.rs:1480 without breaking exhaustiveness.
- `DiagnosticRegistryEntry::replacement: Option<&'static str>` is removed. The two literal struct-construction sites that retained explicit field assignments (the `SIFR-INTERNAL-0002` reserved entry at codes.rs:1285 and the `reserved_family_base` constructor at codes.rs:1397) drop the `replacement: None` line. The `active_entry!` macro drops the field. All call sites recompile under `cargo clippy --workspace -- -D warnings` per the user's reported validation.
- The four retired registry entries (PARSE/TYPE/CODEGEN/BUILD `0001`) are removed from `DIAGNOSTIC_REGISTRY`. Active code constants for these IDs do not exist (and never did). No dangling `DiagnosticCode::FOO` reference points to a deleted ID.
- `gen-error-docs` no longer emits the "Retired Codes" public-index section ([gen-error-docs.rs:189-208](crates/sifr_diagnostics/src/bin/gen-error-docs.rs:189), pre-change), no longer prints the `Retired` legend line in the internal reference, and removes the `Replacement` column from the internal registry table. The remaining columns line up with the format string. The `--check` mode (`scripts/check_diagnostic_docs_sync.py` invokes `gen-error-docs --check`) was reported green by the user; I traced the generated output structurally against `internal_docs/diagnostic_codes.md` and `docs/errors/diagnostic-codes.md`, both of which have the section/columns removed in lockstep.
- No `docs/errors/SIFR-PARSE-0001.md`, `SIFR-TYPE-0001.md`, `SIFR-CODEGEN-0001.md`, or `SIFR-BUILD-0001.md` exists at HEAD (verified with `git ls-tree HEAD docs/errors/`), so the orphan-page check inside `check_active_doc_casing` ([gen-error-docs.rs:102-139](crates/sifr_diagnostics/src/bin/gen-error-docs.rs:102)) does not need to delete anything; the prior policy ("Do not delete retired diagnostic-code docs") that this slice removes from the issue tracker had no concrete files to govern in this codebase. Internally consistent.
- Repo-wide `grep -rn "SIFR-PARSE-0001\|SIFR-TYPE-0001\|SIFR-CODEGEN-0001\|SIFR-BUILD-0001" crates/ scripts/` returns zero hits. No source code references the deleted IDs.

### 2. CompileError surface is now strict — but the strictness is paid for at the wrong layer

[crates/sifr_driver/src/diagnostics.rs:27-31, 97-114](crates/sifr_driver/src/diagnostics.rs:27)

- `CompileError.code: DiagnosticCode` (no longer `Option`). `CompileError::new` is gone. `CompileError::with_code` is the only public constructor. `diagnostic_code()` collapses from a 30-line phase-keyed match to `self.code.code()`. `to_diagnostic` consumes the canonical id directly. This part of the change is correct, minimal, and aligned with the design principle "diagnostic identity is set at the emission site, not inferred from phase/message."
- Every direct construction site has been migrated:
  - `crates/sifr_driver/src/build/materialize.rs:144-158` — already coded with `BUILD_MATERIALIZATION_FAILURE` / `BUILD_RUSTC_OR_CARGO_FAILURE`.
  - `crates/sifr_driver/src/workspace/mod.rs:162-208` — already coded.
  - `crates/sifr_driver/src/project/discovery.rs:195` — already coded (verified via spot-check).
  - `crates/sifr_driver/src/stdlib/bootstrap.rs:206-210` — drops `code: e.code` as `Option` and now propagates the unwrapped `DiagnosticCode`.
  - `crates/sifr_driver/src/stdlib/cache.rs` (test) — sentinel switched from `CompileError::new` to `CompileError::with_code(..., DiagnosticCode::STDLIB_CACHE_FAILURE)`.
  - `crates/sifr_driver/src/test_runner/orchestrator.rs:108-115` — propagates `error.code` directly.
- `lowering_error_to_compile_error` ([module_lowering.rs:37-50](crates/sifr_driver/src/frontend/module_lowering.rs:37)) is the **single remaining place** where a codeless `LoweringError` is converted into a `CompileError`. Its new behavior is `lowering_error_code_or_internal(&error)` which returns `DiagnosticCode::INTERNAL_COMPILER_PANIC` whenever the HIR layer emits via raw `ctx.error(...)`.

This is the load-bearing semantic decision of the slice. It is wrong, for two compounding reasons.

**(a) It is a direct hard-rule violation, not an authorized escape hatch.** The issue tracker that this same PR updates says, *in the very same hard-rules block*:

> Internal compiler failure boundaries are the only place where a broad code is acceptable. Those diagnostics must use `SIFR-INTERNAL-*`, must not be described as user-fixable, and **must not mask a known user-input error that should have a specific code**.
>
> — [issues/...md:1331](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1331)

> Internal code allocation policy:
> - `SIFR-INTERNAL-0001` is the stable catch-all for unclassified compiler panics after a panic boundary.
> - …
> - **Known user-input failures must never be routed through `SIFR-INTERNAL-*`.**
>
> — [issues/...md:1335-1338](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1335)

The 10 fixtures whose `expect-error:` is now `SIFR-INTERNAL-0001` are *all* known user-input failures. Their messages — `argument 1 ('x') of function 'sqrt': expected 'float', got 'str'`, `argument 1 ('values') of function 'accepts_mixed': expected 'list[int | str]', got 'list[int]'`, `iter() tuple argument must have one statically provable element type`, `tuple() currently requires a tuple, list literal, or string literal because Sifr tuples are fixed-length typed values`, etc. — are produced by `ctx.error(format!(…))` at deterministic HIR sites that have a clean canonical destination *already in the registry*: `SIFR-TYPE-0002 (TYPE_MISMATCH)` for the eight argument-type-mismatch variants and `SIFR-TYPE-0007 (Invalid type annotation shape)` / `SIFR-PROTO-0002` / a small number of CALL/PROTO codes for the remainder.

Three of the eight argument-mismatch fixtures emerge from precisely three lines:

- [crates/sifr_hir/src/lower/expressions.rs:1817-1824](crates/sifr_hir/src/lower/expressions.rs:1817) — non-generic argument type mismatch.
- [crates/sifr_hir/src/lower/expressions.rs:1900-1908](crates/sifr_hir/src/lower/expressions.rs:1900) — generic argument with unresolved type vars.
- [crates/sifr_hir/src/lower/expressions.rs:1912-1920](crates/sifr_hir/src/lower/expressions.rs:1912) — generic argument after substitution.

All three share the same `format!("argument {} ('{}') of function '{}': expected '{}', got '{}'", …)` template. Migrating them with `error_with_code(DiagnosticCode::TYPE_MISMATCH, …)` is a 3-line change; it would re-key all eight of the argument-mismatch fixtures to `SIFR-TYPE-0002` *and* eliminate the need to flag them as `SIFR-INTERNAL-0001` at all. The same shape applies to the remaining two fixtures (`tuple_dynamic_list_shape` and `unsupported_default_expr_call`); `iter_heterogeneous_tuple_unsupported` and `reversed_iterator_not_reversible` and `reversible_annotation_rejects_set` are PROTO-family migrations that the inventory at [internal_docs/diagnostic_emission_inventory.md:299-335](internal_docs/diagnostic_emission_inventory.md:299) already targets.

This is exactly the migration pattern that slices 2b.1 through 2b.32 have been executing — *small, mechanical, single-template, registry-already-prepared* call-site migrations. Slice 2b.33 has a unique architectural impact (deleting the bridge), but its contract — "any unmigrated call sites surface as `SIFR-INTERNAL-0001`" — turns it into a non-mechanical, deferred-quality-debt slice. The cleaner contract is "no unmigrated call sites survive the bridge deletion."

**(b) It is an externally observable CLI exit-code regression, not a fixture-only churn.**

`is_internal_compile_error` ([crates/sifr/src/main.rs:260-262](crates/sifr/src/main.rs:260)) returns `true` whenever `error.code == DiagnosticCode::INTERNAL_COMPILER_PANIC`. `compile_error_exit_code` ([main.rs:264-270](crates/sifr/src/main.rs:264)) then routes that error to `EXIT_INTERNAL_COMPILER_FAILURE` (`3`) instead of `EXIT_USER_DIAGNOSTIC` (`1`). I confirmed this empirically with the current working tree:

```
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/stdlib_wrong_type.sifr
error: [main] argument 1 ('x') of function 'sqrt': expected 'float', got 'str'
$ echo $?
3
```

For comparison, the equivalently-shaped `negative_project_type_error` demo (which already emits a coded `return type mismatch` diagnostic) exits `1` — verified the same way:

```
$ cargo run -q -p sifr -- check demos/mode_consistency/negative_cases/type_error_project/main.sifr
type error: [helper] return type mismatch: expected 'int', got 'str'
$ echo $?
1
```

So the same compiler now reports the same *category* of failure with two different exit codes purely depending on whether the HIR call site has been migrated. Programs that compile to a real type/argument error in stdlib, `iter()` over heterogeneous tuples, `reversed()` over a non-reversible value, mutable-list variance, etc. now produce exit code 3 — the canonical "the compiler crashed; please file a bug" code — even though the user's program is exactly the kind of clean type/argument error this exit-code split was *designed* to surface as `1`.

Two compounding reasons this is not just cosmetic:

1. The `verification/validation_contracts/manifest.json` exit-code contract (`expected_exit: 1` for `negative_project_type_error`, `reachable_parse_error_contract`, etc.) is the project's own load-bearing contract for this distinction. It is silent on these specific 10 fixtures only because the manifest doesn't reference them — that is luck, not safety. A future addition of any of these 10 to validation contracts (or to a CI script that distinguishes "user error → continue with next stage" vs. "compiler crash → page someone") will silently fail-open today.
2. `crates/sifr/tests/e2e.rs::test_e2e_fail` ([e2e.rs:2532-2589](crates/sifr/tests/e2e.rs:2532)) tests `failure.code == expected.code` only. It does not invoke the binary and does not check exit code. So *no automated test catches this regression*. The user's reported `cargo test` envelope cannot detect it. It is observable only by hand-running the binary or by the next CLI/CI integration that touches one of these fixtures.

**(c) The `lowering_error_code_or_internal` test [module_lowering.rs:95-106](crates/sifr_driver/src/frontend/module_lowering.rs:95) explicitly *encodes* the misclassification.** The new test name (`codeless_lowering_error_is_internal_compiler_diagnostic`) and assertion (`compile_error.code == DiagnosticCode::INTERNAL_COMPILER_PANIC` for a `LoweringError { code: None, message: "expected int, got str" }`) lock the wrong semantic in. If finding (a) is fixed by migrating the call sites, this test should be replaced by a test that asserts the bridge no longer exists at all — e.g. by making `lower_module_with_externals`'s error type non-`Option<DiagnosticCode>` (so the codeless path becomes unrepresentable), or by making `lowering_error_to_compile_error` debug-assert when `code.is_none()` and only fall through to `INTERNAL_COMPILER_PANIC` in release builds with a structured "compiler bug — diagnostic emitted without a code at <module>" child note, not a bare "expected int, got str" message that masquerades as an unclassified panic.

#### Recommendation for finding #2

The cleanest path that preserves the slice's stated intent ("delete the phase-derived bridge", "make `CompileError` carry active diagnostic identity directly") *and* respects the hard rule:

1. In this slice, migrate the three argument-mismatch sites in `expressions.rs` (lines 1817, 1900, 1912) to `error_with_code(DiagnosticCode::TYPE_MISMATCH, …)`. Re-key the eight `expect-error` markers in the affected fixtures back to `SIFR-TYPE-0002`.
2. Migrate the two remaining non-argument-mismatch sites (`tuple()` shape and `unsupported_default_expr_call`) to their nearest registered code (`SIFR-TYPE-0007` or `SIFR-CALL-0005`, whichever the inventory assigns); re-key the two fixtures.
3. Migrate the iterator/reversible/reversible-annotation triple to their registered PROTO codes per the inventory.
4. After (1)-(3), the `lowering_error_code_or_internal` helper either has zero callers (`LoweringError.code` becomes mandatory) or remains as a defensive *internal* fallback that fires only on a programmer bug. Either way, no e2e fixture expects `SIFR-INTERNAL-0001`.

That keeps the slice's architectural impact intact, removes the exit-code regression, and aligns with the issue tracker's own hard rule. It also keeps the slice ~the same size (10 fixtures + ~5 emission sites — comparable to slice 2b.30/2b.31 which migrated 2-3 emission sites + 2-3 fixtures each).

If any of (1)-(3) are *genuinely* not yet codeable in pre-1.0 (e.g. a registry entry doesn't exist), the right move is to (a) add the registry entry in this slice or (b) **defer this slice** until the prerequisite migration lands. The "fall through to `INTERNAL`" path should not become the production policy.

### 3. Issue tracker is internally inconsistent

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:68-69, 1331-1338, 1170](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:68)

- The new line 68 entry describes the slice as "Any remaining codeless HIR lowering path is surfaced as `SIFR-INTERNAL-0001` so it is visible as an implementation bug rather than a user-facing fallback bucket." — this is the as-built behavior.
- Line 1338 still says "Known user-input failures must never be routed through `SIFR-INTERNAL-*`." — unchanged.
- Line 1331 still says "...must not mask a known user-input error that should have a specific code." — unchanged.
- Line 1170 (the prior "Required guardrails" block) has its retired-doc rules deleted, but the surrounding rule "every active registry code must appear through its canonical `DiagnosticCode::...` constant in non-test compiler source outside `sifr_diagnostics`" remains. The user-input fixtures rerouted to `SIFR-INTERNAL-0001` mean those user-input-category active codes (`TYPE-0002`, etc.) lose 10 representative emissions to `INTERNAL`, which subtly weakens the per-code emission-coverage signal that this rule is trying to protect.

These three texts cannot coexist as policy. Either the line 68 description is an acknowledged exemption and lines 1331/1338 need an explicit "(...except `SIFR-INTERNAL-0001` as a transitional implementation-bug surface for unmigrated codeless paths)" rider, or the line 68 plan is not the right plan and needs the migration described in finding #2. The slice does the former implicitly without updating the rule. **Action:** if the slice is kept as-is despite findings #1/#2, the rule text needs an explicit, dated, scope-limited rider; otherwise the rule is silently weakened.

### 4. Generated-doc / inventory / architecture text changes are correct, taken on their own

[internal_docs/diagnostic_emission_inventory.md:46-49, 81-90, 105-122, 128-145, 296-302](internal_docs/diagnostic_emission_inventory.md:46), [internal_docs/architecture.md:710-713](internal_docs/architecture.md:710), [docs/errors/diagnostic-codes.md:115](docs/errors/diagnostic-codes.md:115), [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md)

- The inventory's "Parser Surface" / "Driver And CLI Surface" / "E2E Expectation And Baseline Surface" sections are rewritten consistently: every "retired" predicate becomes "removed before public stability," the Replacement column in tracking tables is dropped, and the policy hard-rule list ("Do not preserve `SIFR-TYPE-0001` compatibility", etc.) is preserved while removing the corresponding "Do not delete retired diagnostic-code docs" line. The `parse_manifest_error` / `source_root_error` / `build_error` / `cargo_build_error` helpers are no longer the migration target since they were already coded; the inventory text correctly reflects this.
- `internal_docs/architecture.md:710` correctly demotes `Historical E####/W####` framing from "retired" to "removed before public stability" for consistency with the new pre-1.0 policy. No regen drift.
- `internal_docs/diagnostic_codes.md` and `docs/errors/diagnostic-codes.md` are regenerated in step with the codes.rs registry change. The internal-reference table column count is reduced by one (`Replacement` removed) and the active-code page rendering is unchanged. The public index drops the entire "Retired Codes" section.
- The harness sample lines in `crates/sifr/tests/e2e.rs:2726-2754` move from `SIFR-PARSE-0001` / `SIFR-TYPE-0001` (now nonexistent) to `SIFR-PARSE-0002` / `SIFR-TYPE-0002` (real registry entries with active templates and fixtures). This is the right, minimal, mechanical fix for the harness-self-test contract; it does not affect any production fixture.

All of (4) is approve-on-sight regardless of the resolution of (1)/(2).

### 5. Test re-keying in `main.rs` and `tests/diagnostics.rs` is correct
[crates/sifr/src/main.rs:1207-1450](crates/sifr/src/main.rs:1207), [crates/sifr_driver/src/tests/diagnostics.rs:84-129](crates/sifr_driver/src/tests/diagnostics.rs:84)

- The compact-renderer tests don't actually depend on the *meaning* of the codes they construct — they only assert the structural rendering of `[CODE]`, summary grouping, URL formatting, and ordering. Mechanical replacement of `SIFR-TYPE-0001` → `SIFR-TYPE-0002`, `SIFR-PARSE-0001` → `SIFR-PARSE-0002`, `SIFR-CODEGEN-0001` → `SIFR-CODEGEN-0002` is fine.
- `test_compile_error_exit_code_contract_user_vs_internal` is re-keyed to use `DiagnosticCode::TYPE_MISMATCH` for the user-error half. This locks in the contract that *coded* user errors exit `1`. Crucially, this test **does not** cover the new behavior introduced by the slice (codeless HIR → INTERNAL → exit 3); a small additional test on `compile_error_exit_code(&[CompileError::with_code("...", CompilePhase::TypeCheck, DiagnosticCode::INTERNAL_COMPILER_PANIC)])` returning `EXIT_INTERNAL_COMPILER_FAILURE` would lock the new behavior, *which is the very behavior I think should be reverted per finding #2*. So I would not add such a test until #2 is resolved.

### 6. Local validation envelope
- The user reports clean runs of `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check`, `cargo test -p sifr_diagnostics -p sifr_driver --lib --tests`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo test -p sifr --test e2e test_e2e_fail`, and `cargo clippy --workspace -- -D warnings`. I traced each gate against the diff and confirmed the assertions/structures it tests are not broken by the change shape.
- AGENTS.md gating asks for `scripts/run_all_tests.sh --profile quick` before PR; the user has not reported running it. **Recommendation:** run `scripts/run_all_tests.sh --profile quick` before merging, even after addressing finding #2 — it covers `check_diagnostic_schema_sync.py` and `check_diagnostic_docs_sync.py` which are doubly-affected by this change set.

## Findings

### Blocking

1. **Codeless `LoweringError → SIFR-INTERNAL-0001` mapping misclassifies known user-input failures.** See finding #2 and #3 above. Direct violation of [issues/...md:1331](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1331) and [issues/...md:1338](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1338). Must be resolved by call-site migration, slice deferral, or an explicit, dated, scope-limited issue-tracker rider. Recommended path is migration (3-line diff in expressions.rs covers 8/10 affected fixtures; the remaining 2 are similarly small).
2. **CLI exit-code regression for legitimate user programs.** Empirically verified `exit 3` (compiler-crash) for `crates/sifr/tests/e2e/fail/stdlib_wrong_type.sifr` and equivalent fixtures, where the compiler used to (and conceptually should) exit `1`. No automated test catches this. Resolved naturally if blocking #1 is fixed.

### Suggestions (non-blocking even if blocking #1/#2 are resolved)

3. **Tighten the codeless path at the type-system level rather than at the boundary.** Once finding #1 is fixed, consider promoting `LoweringError.code: Option<DiagnosticCode>` to `LoweringError.code: DiagnosticCode` (mandatory) and replacing the existing `legacy_error_records_no_structured_identity` test with a `code()`-only `ctx.error_with_code(...)` API, removing `ctx.error(String)` from the public HIR surface. This makes "no codeless emission survives the boundary" a *type-system* invariant rather than a *runtime* fallback, and eliminates the temptation to re-introduce a bridge later. Out of scope for this slice but a natural follow-up.

4. **Add a guardrail script** (`scripts/check_no_codeless_lowering_emissions.py` or extension to `check_hir_maintainability_guardrails.py`) that greps `crates/sifr_hir/src/lower/` for `ctx.error(` (raw, non-`error_with_code`) lines and reports a count + offending file:line list. Lock the count to a non-increasing baseline. This would have caught the present situation at PR-time; it would also enforce monotonic progress through the remaining 173 raw `ctx.error(` sites in `expressions.rs` and the 49 in `builtin_calls.rs`.

5. **Add an exit-code contract to `verification/validation_contracts/manifest.json`** for at least one of the affected fixtures (e.g. `stdlib_wrong_type.sifr`) once finding #1 is resolved. That locks the user-vs-internal exit-code split for the categories the slice is *trying* to clean up. Without such a contract, the same regression can happen again silently.

6. **The `test_runner_project` orchestrator path** ([crates/sifr_driver/src/test_runner/orchestrator.rs:108-115](crates/sifr_driver/src/test_runner/orchestrator.rs:108)) propagates `error.code` directly without the `lowering_error_code_or_internal` helper. After finding #1 is fixed, this is fine. Until then, the test runner has divergent behavior from `lowering_error_to_compile_error`: in `module_lowering.rs`, codeless errors become `INTERNAL_COMPILER_PANIC`; in `orchestrator.rs`, they would already be a non-`Option` `DiagnosticCode` value coming from `lower_frontend_module`'s already-wrapped output, so the propagation is shadow-coupled to whatever the upstream choice was. Safer to centralize the policy in one helper.

7. **Issue-tracker entry pluralization / numbering hygiene.** Line 68 reads "...slice 2b.33 in progress: ... `CompileError` to carry an active diagnostic code." This is a deliberate grammatical fragment but the "in progress" is now stale (the diff exists; the slice is implementation-complete pending review). Move to the standard "implementation complete and reviewer-satisfied" wording on PR open per the convention used in slices 2b.30/2b.31/2b.32.

### Out of scope (correctly carved out)

- Migration of any other raw `ctx.error(` emission site beyond the ten currently in the affected fixtures. The slice is intentionally narrow on this axis (and that narrowness is exactly what creates blocking #1). If the migration approach in finding #2 is taken, this still does not require touching all 173 expressions.rs / 49 builtin_calls.rs sites — only the ones whose error path is actually exercised by the ten re-keyed fixtures.
- `LoweringError` span shape (line/col) — unchanged.
- Renderer changes — unchanged.

## Summary

The mechanical surface of this slice — registry shrink, `Replacement` column removal, generator update, `CompileError` `Option` → `DiagnosticCode` promotion, struct-construction site migration, doc/inventory/architecture text updates, and harness-sample test re-keying — is correct and approve-on-sight.

The semantic surface — *what should happen when an HIR `ctx.error(...)` site has not yet been migrated* — is wrong. The chosen answer ("flag it as `SIFR-INTERNAL-0001` so it is visible as an implementation bug") is internally appealing but externally observable as an exit-code regression and a hard-rule violation, and it is not actually necessary because the affected emission sites have well-defined registered destinations and small, mechanical migration paths.

**Action:** before opening for merge, either migrate the 3-5 emission sites that produce the 10 affected fixtures and re-key the fixtures back to their canonical TYPE/PROTO/CALL codes, or defer the bridge deletion to a follow-up slice that lands together with those migrations. The current "delete the bridge first, treat the unmigrated remainder as `INTERNAL`" sequence inverts the safe order.
