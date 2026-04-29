# Review: milestone_diag_4a — Renderer + Workspace-Inference Slice (Pass 3)

Branch: `codex/semantic-diagnostics-diag-4a` (uncommitted working tree on top of `73b4e32c`)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md)

## Verdict

**Ready to PR/merge.** The follow-up patch fully addresses the two pass-2 gating items (record the deferral honestly in the issue; tighten the transitional comment to name the owning slice) and additionally lands four of the five low-risk polish items pass 2 had marked optional (N2 forwarder comments, N3 internal-grouping test, N4 dead prefix branch, N5 `CompilePhase` cosmetic). R5's TODO markers are now in place at all six parse-failure spots. No correctness regressions, no scope drift, validation evidence is the right scope and signature matches the prior quick-profile run (`e1bf653aaa770517`, 79.70 s).

## Pass-2 gate items

| Pass-2 requirement | Status | Evidence |
| --- | --- | --- |
| Issue file records the `TypeCheck => SIFR-TYPE-0001` deletion deferral | ✓ **Recorded** | [issue lines 34-35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34) adds two new bullets: "Started `milestone_diag_4a` slice 1: canonical renderer presentation helpers plus explicit workspace diagnostic identity transport" and "Deferred `CompilePhase::TypeCheck => \"SIFR-TYPE-0001\"` bridge deletion and affected fixture re-keying to `milestone_diag_4a` slice 2, where HIR `LoweringError` transport will carry structured diagnostic codes." Reviewers comparing the merged slice against the issue plan ([issue line 1121](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1121) — "`diag_4a` deletes the `CompilePhase::TypeCheck => SIFR-TYPE-0001` public mapping…") will read this as a scope split rather than scope drift. |
| Transitional comment names the owning slice | ✓ **Tightened** | [diagnostics.rs:129-134](../crates/sifr_driver/src/diagnostics.rs:129) now reads "Transitional legacy bridge for unmigrated non-workspace paths in diag_4a slice 1. Slice 2 removes the TypeCheck fallback after HIR LoweringError carries structured DiagnosticCode values and affected fixtures are re-keyed. Workspace diagnostics must use `with_code`; this function no longer infers identity from rendered message text." Names the specific owning slice and re-states the workspace invariant. |

## Pass-2 polish items (N2-N5, R5)

| Pass-2 finding | Status | Evidence |
| --- | --- | --- |
| N2 — forwarder comments distinguishing inert vs live `code: error.code` | ✓ **Done** | [orchestrator.rs:113-115](../crates/sifr_driver/src/test_runner/orchestrator.rs:113) — `// Preserves None until HIR LoweringError carries a structured code in the next diag_4a slice.` The sibling forwarder at [bootstrap.rs:205-207](../crates/sifr_driver/src/stdlib/bootstrap.rs:205) — `// Preserves the internal panic code set by run_codegen_with_boundary.` Both now correctly distinguish the inert forwarder (TypeCheck path) from the live one (Codegen panic path). |
| N3 — compact-grouping test for two internals sharing `(severity, code, template)` | ✓ **Added** | [presentation.rs:338-360](../crates/sifr_diagnostics/src/render/presentation.rs:338) `compact_groups_internal_diagnostics_without_locations` emits two `INTERNAL_COMPILER_PANIC` errors with identical phase arg `"codegen"`, asserts the compact summary contains `"(x2)"`, and asserts `!compact.contains("  at ")` to pin the no-location grouping behavior. Pinning `"  at "` (with trailing spaces) avoids accidentally matching the `at` in any future help/url text. |
| N4 — `is_internal_compile_error` prefix fallback dead in production | ✓ **Deleted** | [main.rs:260-262](../crates/sifr/src/main.rs:260) is now a one-liner: `error.code == Some(DiagnosticCode::INTERNAL_COMPILER_PANIC)`. The synthetic test at [main.rs:1209-1221](../crates/sifr/src/main.rs:1209) constructs the user error via `CompileError::new("type mismatch", CompilePhase::TypeCheck)` (no code, falls through to the user exit) and the internal error via `CompileError::with_code(..., INTERNAL_COMPILER_PANIC)`, exercising the structured path. The downstream `test_run_with_panic_boundary_converts_panic_to_internal_compile_error` at [main.rs:1252-1266](../crates/sifr/src/main.rs:1252) still passes because `run_with_panic_boundary` itself sets the code. |
| N5 — `INTERNAL_COMPILER_PANIC` paired with `CompilePhase::TypeCheck` at [build/entrypoint.rs:215-221](../crates/sifr_driver/src/build/entrypoint.rs:215) | ✓ **Re-paired with `Build`** | [build/entrypoint.rs:217-221](../crates/sifr_driver/src/build/entrypoint.rs:217) now uses `CompilePhase::Build`, matching the sibling site at [build/entrypoint.rs:208-212](../crates/sifr_driver/src/build/entrypoint.rs:208). The legacy `Display` ([diagnostics.rs:226-232](../crates/sifr_driver/src/diagnostics.rs:226)) renders this as `"build error: internal error: frontend lowering missing 'main' module"` instead of the previously odd `"type error: internal error: …"`. JSON/structured-code behavior is unchanged because identity comes from `INTERNAL_COMPILER_PANIC`. |
| R5 — `TODO(diag_4a)` markers on the three single-bucket parse sites | ✓ **Added at all 6 spots** | Two markers each in [frontend/api.rs:18-19, 36-38](../crates/sifr_driver/src/frontend/api.rs:18), [project/discovery.rs:406-407, 424-425](../crates/sifr_driver/src/project/discovery.rs:406), and [stdlib/bootstrap.rs:30-31, 48-49](../crates/sifr_driver/src/stdlib/bootstrap.rs:30). All carry the same `TODO(diag_4a slice 2)` ownership tag, consistent with the comment in [diagnostics.rs:130-131](../crates/sifr_driver/src/diagnostics.rs:130). Pass-2 N6 (legacy CLI renderer prefix sniffing) is correctly left untouched — that surface is owned by a later wave. |

## New observations from pass 3

### O1 — Out-of-scope identity assignment for `compile_order` cycle (low, intentional)

[compile_order.rs:189-195](../crates/sifr_driver/src/project/compile_order.rs:189) now emits the cycle diagnostic with `CompilePhase::Build` (was `TypeCheck` in pass 2's draft) plus `WORKSPACE_IMPORT_CYCLE`. This is internally consistent with the rest of the workspace family (all `SIFR-WORKSPACE-01xx` codes route through `CompilePhase::Build`) and matches the structural classification — an import cycle is a workspace-resolution failure, not a typecheck failure. The legacy `Display` will render as `"build error: module dependency cycle detected: …"`, which is consistent with the unresolved-import / ambiguous-import siblings. No fixtures depend on the old `"type error: module dependency cycle …"` rendering — `grep` for "module dependency cycle" turns up only this site and unit tests that look at the structured fields. Acceptable.

### O2 — `STDLIB_BOOTSTRAP_FAILURE` pairs with a parse-classification TODO marker (cosmetic)

[stdlib/bootstrap.rs:30-39](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) and [bootstrap.rs:48-55](../crates/sifr_driver/src/stdlib/bootstrap.rs:48) attach a `TODO(diag_4a slice 2): classify Ruff parse failures into the precise active parse-code buckets.` marker but emit `STDLIB_BOOTSTRAP_FAILURE` (`SIFR-STDLIB-0003`). The other two parse sites emit `PARSE_EXPECTED_TOKEN_OR_RECOVERY` (`SIFR-PARSE-0002`) where the marker matches the intent. For the stdlib bootstrap site, classifying a stdlib parse failure as `STDLIB_BOOTSTRAP_FAILURE` is arguably *already* the correct identity — a malformed stdlib source is a compiler-bootstrap failure rather than a user-facing parse error — so the TODO marker may be misleading on this specific site. A minor doc nit: in slice 2 the bootstrap markers should either be deleted (if the stdlib identity is final) or rewritten to read "if stdlib parse failures should ever be reclassified into parse-family codes". Not a blocker for this slice.

### O3 — `parse_manifest_error` takes `impl std::fmt::Display` but is single-call (style nit, low)

[workspace/mod.rs:162](../crates/sifr_driver/src/workspace/mod.rs:162) is the only constructor for `WORKSPACE_MALFORMED_MANIFEST`, called from `parse_manifest_schema_error` and from the toml deserialization path. The `impl std::fmt::Display` parameter is genuinely useful (toml errors are `Display` rather than `String`), so this is fine as-is. Flagged only because pass 2 listed `source_root_error` polish; the manifest sibling is structurally the same shape after the typed-kind refactor at [workspace/mod.rs:177-204](../crates/sifr_driver/src/workspace/mod.rs:177). No action.

### O4 — `is_internal_compile_error` is now structurally tied to a single code (intended, but worth flagging)

The simplification at [main.rs:260-262](../crates/sifr/src/main.rs:260) means the exit-code classifier now treats *only* `INTERNAL_COMPILER_PANIC` as internal. If a future internal-error code is introduced (e.g. `INTERNAL_INVARIANT_VIOLATION` or `INTERNAL_LOWERING_BUG`), the classifier must be updated in the same change-set, otherwise the new code's errors would route to `EXIT_USER_DIAGNOSTIC`. This is a property of the simplification, not a bug — but the next contributor adding an `SIFR-INTERNAL-` family member needs to update [main.rs:260](../crates/sifr/src/main.rs:260) and the `EXIT_INTERNAL_COMPILER_FAILURE` test at [main.rs:1209](../crates/sifr/src/main.rs:1209). Consider, in slice 2, replacing the equality check with a "code starts with `SIFR-INTERNAL-`" predicate — but that decision is correctly out of scope here.

### O5 — Issue heading rename `"the current wave" → "milestone_diag_2a"` (positive)

[issue line 39](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:39) re-titles the historical validation block from `Validation evidence for the current wave:` to `Validation evidence for milestone_diag_2a:`. The bullet inside the block already names the `milestone_diag_2a` branch, so the new heading matches. No phantom "current wave" label remains, which makes the document self-consistent now that three waves have validation blocks.

## Out-of-scope drift check (negative findings — *good*)

- `crates/sifr_hir/` — untouched.
- `LoweringError` / `LoweringResult` / `LoweringOutcome` — untouched.
- `CompilePhase::TypeCheck` enum variant and its `Display` arm at [diagnostics.rs:36, 228](../crates/sifr_driver/src/diagnostics.rs:36) — retained; the deferral note in the issue and the in-code transitional comment are the audit trail for keeping it.
- The TypeCheck fallback `"SIFR-TYPE-0001"` at [diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) — still in place, as the slicing decision requires.
- `compile_errors_to_diagnostics` and `apply_diagnostic_recovery_limits` signatures — unchanged.
- 91 fail fixtures and the `decimal_invalid_literal` baselines — untouched. The transitional `"SIFR-TYPE-0001"` they rely on still routes through the bridge.
- Help-aware compact renderer in `main.rs` ([main.rs:302-417](../crates/sifr/src/main.rs:302)) — unchanged. The new presentation-layer compact renderer is parallel infrastructure for slice 2 wiring.
- The legacy CLI's `SIFR-PARSE-` / `SIFR-TYPE-` / `SIFR-CODEGEN-` / `SIFR-BUILD-` prefix sniffer at [main.rs:374-385](../crates/sifr/src/main.rs:374) — unchanged. Pass-2 N6 explicitly out of scope.

## Behavioral / correctness check

I traced every site that this slice changes from a struct literal to `with_code`:

- [build/entrypoint.rs:163-170, 207-211, 214-221](../crates/sifr_driver/src/build/entrypoint.rs:163) — three sites; all carry `BUILD_MATERIALIZATION_FAILURE` or `INTERNAL_COMPILER_PANIC`.
- [build/materialize.rs:115, 124, 144-158](../crates/sifr_driver/src/build/materialize.rs:115) — split helpers (`build_error` → `BUILD_MATERIALIZATION_FAILURE`, `cargo_build_error` → `BUILD_RUSTC_OR_CARGO_FAILURE`) preserved from pass 2.
- [build/workspace.rs:30-44, 124-130, 163-170, 210-217, 294-307](../crates/sifr_driver/src/build/workspace.rs:30) — 6 build-error literals replaced with codes (`BUILD_TEMP_WORKSPACE_FAILURE`, `BUILD_ARTIFACT_MISSING`, `BUILD_MATERIALIZATION_FAILURE`).
- [diagnostics.rs:260-266](../crates/sifr_driver/src/diagnostics.rs:260) — `run_codegen_with_boundary` pairs `Codegen` + `INTERNAL_COMPILER_PANIC` (✓ correct).
- [frontend/api.rs:21-44](../crates/sifr_driver/src/frontend/api.rs:21) — both Ruff parse paths pair `Parse` + `PARSE_EXPECTED_TOKEN_OR_RECOVERY`.
- [frontend/module_lowering.rs:25-44](../crates/sifr_driver/src/frontend/module_lowering.rs:25) — uses `CompileError::new(...)` (no code) by design — this is the inert forwarder for HIR lowering errors. Slice-2 work.
- [project/compile_order.rs:189-195](../crates/sifr_driver/src/project/compile_order.rs:189) — `Build` + `WORKSPACE_IMPORT_CYCLE` (was `TypeCheck` + retired bucket).
- [project/discovery.rs:194-225, 393-426](../crates/sifr_driver/src/project/discovery.rs:194) — typed dispatch on `ResolutionFailureKind`, parse failures re-coded.
- [project/frontend.rs:25-32](../crates/sifr_driver/src/project/frontend.rs:25) — `Build` + `INTERNAL_COMPILER_PANIC` for unparsed-module invariant.
- [stdlib/bootstrap.rs:30-77, 195-209](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) — parse failures and lowering errors paired with `STDLIB_BOOTSTRAP_FAILURE`; codegen-panic forwarder preserves `INTERNAL_COMPILER_PANIC` from `run_codegen_with_boundary`.
- [stdlib/cache.rs:50-58](../crates/sifr_driver/src/stdlib/cache.rs:50) — synthetic test uses `CompileError::new(...)` (no code) since the test exercises the cache, not the bridge. Correct.
- [test_runner/execution.rs:39-138](../crates/sifr_driver/src/test_runner/execution.rs:39) — eight `Build` literals coded (`BUILD_MATERIALIZATION_FAILURE`, `BUILD_CARGO_MANIFEST_FAILURE`, `BUILD_RUSTC_OR_CARGO_FAILURE`).
- [test_runner/orchestrator.rs:88-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:88) — invariant violation gets `INTERNAL_COMPILER_PANIC`; HIR-error forwarder retains `code: error.code` (inert today, ready for slice 2).
- [tests/diagnostics.rs:7-85](../crates/sifr_driver/src/tests/diagnostics.rs:7) — three positive code-stability tests rewritten to active codes (`SIFR-PARSE-0002`, `SIFR-TYPE-0002`, `SIFR-CODEGEN-0002`, `SIFR-BUILD-0003`). New negative test `test_workspace_codes_do_not_derive_from_message_prefixes` proves the message-prefix workspace inference is gone — emits a workspace-shaped message but with `BUILD_TEMP_WORKSPACE_FAILURE`, asserts `SIFR-BUILD-0003` wins. Excellent regression guard.
- [workspace/mod.rs:120-204](../crates/sifr_driver/src/workspace/mod.rs:120) — typed `SourceRootErrorKind` enum from pass 2 retained; manifest-parse error coded with `WORKSPACE_MALFORMED_MANIFEST`.

I did not find any path where a previously-coded site has been silently downgraded to `CompileError::new` in this follow-up; every site that has access to a meaningful `DiagnosticCode` uses `with_code`, and every site that does not (HIR lowering forwarder, single sentinel test) uses `new` with a comment.

## Validation

The implementer's reported run is the right scope:

- `cargo fmt --check`
- `python3 scripts/check_diagnostic_schema_sync.py`
- `python3 scripts/check_diagnostic_docs_sync.py`
- `cargo test -p sifr_diagnostics`
- `cargo test -p sifr_driver diagnostics`
- `cargo test -p sifr --no-run`
- `cargo clippy -p sifr_diagnostics -p sifr_driver -p sifr -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=79.70s`

The signature matches the published baseline for the branch and is consistent with the value reported on `milestone_diag_2b`. No baseline regeneration is required because the renderer is not yet wired into the CLI; the JSON/compact outputs still come from `compile_errors_to_diagnostics` and the workspace fixtures continue to assert the active workspace codes.

Pre-PR I would still verify that the workspace verification fixtures (`workspace_unresolved_import`, `workspace_ambiguous_import`, `workspace_malformed_manifest`, `workspace_namespace_collision`) match — but `scripts/run_all_tests.sh --profile quick` covers them, so the green run is sufficient evidence.

## Sequencing recommendations carried forward to slice 2

These are unchanged from pass 2's recommendations and should land in slice 2:

1. Repoint the three remaining bridge fallbacks (`Parse → PARSE_EXPECTED_TOKEN_OR_RECOVERY`, `Codegen → CODEGEN_BACKEND_FAILURE`, `Build → BUILD_MATERIALIZATION_FAILURE`) and **delete** `TypeCheck => SIFR-TYPE-0001` once HIR `LoweringError` carries codes. Re-key the affected `decimal_*` and 80+ e2e fail fixtures in the same slice.
2. Migrate HIR `LoweringError` to carry a `DiagnosticCode`. Then the inert `code: error.code` forward in [orchestrator.rs:113-115](../crates/sifr_driver/src/test_runner/orchestrator.rs:113) and the `CompileError::new(...)` site at [module_lowering.rs:29](../crates/sifr_driver/src/frontend/module_lowering.rs:29) become live and can be migrated to `with_code` directly.
3. Reconsider the parse-classification TODOs at [stdlib/bootstrap.rs:30, 48](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) — either delete (if `STDLIB_BOOTSTRAP_FAILURE` is the canonical identity) or change the marker text.
4. Optionally generalize `is_internal_compile_error` (O4) to recognize any `SIFR-INTERNAL-*` family code, anticipating future internal-error families.

## Summary

Pass 3 confirms the slice is mergeable. The two pass-2 gating items are in place: the deferral is recorded in [issues/ad-hoc-…md:34-35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:34) with explicit slice-2 ownership, and the transitional comment at [diagnostics.rs:129-134](../crates/sifr_driver/src/diagnostics.rs:129) names the owning slice. The implementer additionally landed N2-N5 polish and the R5 TODO markers, which makes the slice substantially cleaner than the pass-2 baseline. New observations O1-O5 are cosmetic or deferred-by-design; none block this PR. Validation evidence is the right scope and signature matches.
