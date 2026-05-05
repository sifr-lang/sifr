# `milestone_diag_4a` slice 2b.33 — Remove pre-1.0 retired diagnostic-code lifecycle and phase-derived `CompileError` fallbacks

Pass 2 review of the uncommitted working tree on branch
`codex/diag-4a-remove-retired-fallbacks`, following
[`reviews/semantic-diagnostic-code-taxonomy-diag-4a-remove-retired-fallbacks-review-pass-1.md`](semantic-diagnostic-code-taxonomy-diag-4a-remove-retired-fallbacks-review-pass-1.md).

## Verdict

**Approve.** Pass 1's two blocking findings are fully resolved.

Pass 1 blocked the slice because the chosen replacement semantics for the
deleted phase-bucket arm — routing every codeless `LoweringError` through
`DiagnosticCode::INTERNAL_COMPILER_PANIC` — both (a) violated the
issue-tracker hard rule that "known user-input failures must never be routed
through `SIFR-INTERNAL-*`" and (b) regressed the CLI exit code from
`EXIT_USER_DIAGNOSTIC` (1) to `EXIT_INTERNAL_COMPILER_FAILURE` (3) for ten
e2e fail fixtures. Pass 2 takes the migration path that pass 1 recommended
(option 1): the underlying HIR call sites are migrated to active codes,
the ten fixtures are re-keyed to those active codes, and the bridge becomes
a defensive *compiler-bug* surface rather than a production user-error
fallback. The empirical CLI check the user reports
(`cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/stdlib_wrong_type.sifr`
exits 1 with a normal type error) confirms the regression is gone.

The rest of the registry/`CompileError` surface change is unchanged from
pass 1 and remains internally consistent. Findings below are limited to
small follow-ups; none are blocking.

## What changed since pass 1

- Five HIR call sites migrated from `ctx.error(...)` to `ctx.error_with_code(...)`:
  - [`expressions.rs:686`](crates/sifr_hir/src/lower/expressions.rs:686) — `iter()` on a heterogeneous tuple → `TYPE_CONTAINER_ELEMENT_CONFLICT` (`SIFR-TYPE-0008`).
  - [`expressions.rs:1818`](crates/sifr_hir/src/lower/expressions.rs:1818), [`expressions.rs:1904`](crates/sifr_hir/src/lower/expressions.rs:1904), [`expressions.rs:1919`](crates/sifr_hir/src/lower/expressions.rs:1919) — argument-type mismatch (non-generic, generic-with-unresolved-typevar, generic-after-substitution) → `TYPE_MISMATCH` (`SIFR-TYPE-0002`).
  - [`builtin_calls.rs:192`](crates/sifr_hir/src/lower/builtin_calls.rs:192) — `tuple()` requires tuple/list-literal/string-literal → `STDLIB_UNSUPPORTED_SURFACE` (`SIFR-STDLIB-0001`).
  - [`builtin_calls.rs:932`](crates/sifr_hir/src/lower/builtin_calls.rs:932) — `reversed()` argument must be reversible → `PROTO_BOUND_NOT_SATISFIED` (`SIFR-PROTO-0001`).
- Three default-argument call sites migrated to a newly-added active code `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT` (`SIFR-TYPE-0011`):
  - module-level lowering, extracted into [`crates/sifr_hir/src/lower/default_args.rs:36`](crates/sifr_hir/src/lower/default_args.rs:36);
  - nested function lowering at [`typing_and_functions.rs:247`](crates/sifr_hir/src/lower/typing_and_functions.rs:247) and [`typing_and_functions.rs:264`](crates/sifr_hir/src/lower/typing_and_functions.rs:264);
  - `__init__` and method default-argument paths at [`classes.rs:474`](crates/sifr_hir/src/lower/classes.rs:474) and [`classes.rs:522`](crates/sifr_hir/src/lower/classes.rs:522).
- Ten `expect-error:` markers in `crates/sifr/tests/e2e/fail/` re-keyed away from `SIFR-INTERNAL-0001` to the active codes above.
- New active registry entry `SIFR-TYPE-0011` ([`codes.rs:39`](crates/sifr_diagnostics/src/codes.rs:39), [`codes.rs:628`](crates/sifr_diagnostics/src/codes.rs:628)), `ACTIVE_DIAGNOSTIC_CODES` membership at [`codes.rs:1335`](crates/sifr_diagnostics/src/codes.rs:1335), generated public-index row in [`docs/errors/diagnostic-codes.md:58`](docs/errors/diagnostic-codes.md:58), generated internal-reference row in [`internal_docs/diagnostic_codes.md:82`](internal_docs/diagnostic_codes.md:82), and per-code page [`docs/errors/SIFR-TYPE-0011.md`](docs/errors/SIFR-TYPE-0011.md:1) (untracked, will be added by the slice).
- Module-level default-argument collection extracted into a new submodule [`crates/sifr_hir/src/lower/default_args.rs`](crates/sifr_hir/src/lower/default_args.rs:1) (45 lines) registered at [`mod.rs:21`](crates/sifr_hir/src/lower/mod.rs:21) and [`mod.rs:82`](crates/sifr_hir/src/lower/mod.rs:82). The original ~36-line inline block at the old module-level call site is replaced by a single `collect_function_defaults(&mut ctx, &function_name, func)` call at [`mod.rs:689`](crates/sifr_hir/src/lower/mod.rs:689).
- `lowering_error_to_compile_error` ([`module_lowering.rs:37-58`](crates/sifr_driver/src/frontend/module_lowering.rs:37)) now wraps codeless emissions with an explicit `internal compiler error: HIR lowering emitted a diagnostic without canonical code: ...` prefix, *and* still routes them through `INTERNAL_COMPILER_PANIC`. The `codeless_lowering_error_is_internal_compiler_diagnostic` test ([`module_lowering.rs:104-117`](crates/sifr_driver/src/frontend/module_lowering.rs:104)) is updated to assert the new prefixed message.
- The corresponding entry in the issue tracker was added at [`issues/...md:68`](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:68) and the prior deferral note marked `[x]`-superseded at [`issues/...md:69`](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:69).

## How pass 1's blockers map to pass 2

### Blocker 1 — codeless `LoweringError → SIFR-INTERNAL-0001` masquerading as user error: **resolved**

- All ten of the previously-affected fixtures now expect *active* user-error codes, not `SIFR-INTERNAL-0001`. Verified by `git diff` of `crates/sifr/tests/e2e/fail/`:
  - 6 of 10 → `SIFR-TYPE-0002` (`stdlib_wrong_type`, `stdlib_test_assert_eq_type_mismatch`, `stdlib_counter_wrong_type`, `mutable_list_variance_invariant`, `reversible_annotation_rejects_set`, `islice_non_iterable_input`),
  - 1 → `SIFR-TYPE-0008` (`iter_heterogeneous_tuple_unsupported`),
  - 1 → `SIFR-TYPE-0011` (`unsupported_default_expr_call`),
  - 1 → `SIFR-STDLIB-0001` (`tuple_dynamic_list_shape`),
  - 1 → `SIFR-PROTO-0001` (`reversed_iterator_not_reversible`).
- Each emission site now uses `ctx.error_with_code(<DiagnosticCode>, <msg>)`. Spot-traced each fixture's surface text against the corresponding HIR source line; the literal message strings match the existing fixture expectations character-for-character, so `test_e2e_fail`'s strict `failure.code == expected.code` and `failure.message.contains(...)` checks are upheld.
- The `lowering_error_code_or_internal` helper ([`module_lowering.rs:60-64`](crates/sifr_driver/src/frontend/module_lowering.rs:60)) still exists, but no e2e fixture exercises it. Its remaining role is the *defensive* one pass 1 sketched as option 2: any *future* unmigrated codeless emission becomes a clearly-prefixed `SIFR-INTERNAL-0001` "compiler bug" diagnostic rather than a silent fallback. The new prefix string makes the implementation-bug intent explicit at the user-visible message layer.

### Blocker 2 — CLI exit-code regression for legitimate user programs: **resolved**

- Because every fixture now reports an active `TYPE`/`STDLIB`/`PROTO` code instead of `INTERNAL_COMPILER_PANIC`, `is_internal_compile_error` ([`crates/sifr/src/main.rs:260-262`](crates/sifr/src/main.rs:260)) returns `false` for these fixtures, and `compile_error_exit_code` returns `EXIT_USER_DIAGNOSTIC` (1).
- The user's manual run (`cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/stdlib_wrong_type.sifr` → exit 1) confirms the empirical CLI exit-code regression that pass 1 reproduced is gone.
- `test_compile_error_exit_code_contract_user_vs_internal` is unchanged in shape; the user-error half is now constructed with `DiagnosticCode::TYPE_MISMATCH` ([`main.rs:1210-1214`](crates/sifr/src/main.rs:1210)), and the internal-error half remains `INTERNAL_COMPILER_PANIC` ([`main.rs:1217-1221`](crates/sifr/src/main.rs:1217)). The split is preserved.

### Hard-rule consistency at the policy layer: **resolved**

- Pass 1 flagged the apparent contradiction between the line 68 plan and the lines 1331/1338 hard rules. Pass 2 doesn't change the rule text — and it doesn't need to, because the slice no longer routes a known-user-input failure through `SIFR-INTERNAL-*`. The bridge that remains fires only for genuine compiler bugs (an emission site that hasn't been given a `DiagnosticCode`), which is exactly what `SIFR-INTERNAL-*` is for. The two rules and the new line 68 plan now coexist coherently.

## What I checked

### A. Registry and generated docs — clean and unchanged from pass 1's "approve-on-sight" envelope

- [`crates/sifr_diagnostics/src/codes.rs`](crates/sifr_diagnostics/src/codes.rs) cleanly drops `DiagnosticState::Retired`, `DiagnosticRegistryEntry::replacement`, the four retired registry entries, and the `retired_entry!` macro. Adds `TYPE_UNSUPPORTED_DEFAULT_ARGUMENT` constant, the corresponding `active_entry!` row, and the `ACTIVE_DIAGNOSTIC_CODES` slot. Match arms in `DiagnosticState::as_str` and the registry-validation test at [`codes.rs:1494`](crates/sifr_diagnostics/src/codes.rs:1494) collapse correctly to the new closed `{Active, Reserved}` set.
- `gen-error-docs.rs` no longer emits the "Retired Codes" public-index section, no longer prints the `Retired` legend line, and no longer carries the `Replacement` column in the internal registry table. The format-string column count matches the new header. `--check` mode reportedly green per the user's local run.
- The generated [`docs/errors/diagnostic-codes.md`](docs/errors/diagnostic-codes.md) and [`internal_docs/diagnostic_codes.md`](internal_docs/diagnostic_codes.md) match the registry change byte-for-byte (verified by spot-checking row counts and the new `SIFR-TYPE-0011` entry).
- Repo-wide grep for `SIFR-PARSE-0001|SIFR-TYPE-0001|SIFR-CODEGEN-0001|SIFR-BUILD-0001|retired_entry|DiagnosticState::Retired` against `crates/sifr_diagnostics`, `crates/sifr_driver`, `crates/sifr_hir`, `crates/sifr/src`, `crates/sifr/tests`, `docs/`, and `scripts/` returns zero hits in source. The remaining historical references in [`internal_docs/diagnostic_emission_inventory.md:49`](internal_docs/diagnostic_emission_inventory.md:49), [`:120`](internal_docs/diagnostic_emission_inventory.md:120), [`:142-143`](internal_docs/diagnostic_emission_inventory.md:142), and [`:262`](internal_docs/diagnostic_emission_inventory.md:262) are all phrased as past state ("removed before public stability", "Removed catch-all"), not current expectations. No orphan `docs/errors/SIFR-{PARSE,TYPE,CODEGEN,BUILD}-0001.md` files exist.

### B. `CompileError` surface — `Option<DiagnosticCode>` removal is complete

- `CompileError.code` is `DiagnosticCode` (no `Option`) at [`diagnostics.rs:30`](crates/sifr_driver/src/diagnostics.rs:30). `CompileError::with_code` is the only public constructor; `CompileError::new` is gone. `to_diagnostic` reads `self.code.code()` directly.
- Every direct construction site has been migrated:
  - [`stdlib/bootstrap.rs:206`](crates/sifr_driver/src/stdlib/bootstrap.rs:206) — propagates `e.code` unwrapped, drops the explanatory comment.
  - [`stdlib/cache.rs:54`](crates/sifr_driver/src/stdlib/cache.rs:54) — sentinel switched to `with_code(..., DiagnosticCode::STDLIB_CACHE_FAILURE)`.
  - [`test_runner/orchestrator.rs:108-114`](crates/sifr_driver/src/test_runner/orchestrator.rs:108) — `CompileError { code: error.code, ... }` propagates the boundary-classified code; the prior comment about legacy `None` codes is removed.
  - [`tests/project_graph.rs:385`](crates/sifr_driver/src/tests/project_graph.rs:385) — assertion updated to compare against bare `DiagnosticCode` (not `Some(...)`).
  - [`tests/diagnostics.rs:84-129`](crates/sifr_driver/src/tests/diagnostics.rs:84) compact-renderer tests re-keyed `0001` → `0002` mechanically; tests don't depend on the meaning of the codes.
- Repo-wide grep for `Option<DiagnosticCode>|code: None|code: Some(` in `crates/sifr_driver/src` and `crates/sifr/src` returns one result, the test helper at [`module_lowering.rs:81`](crates/sifr_driver/src/frontend/module_lowering.rs:81), which intentionally still constructs `LoweringError` (not `CompileError`) with `Option<DiagnosticCode>`. Correct.

### C. The remaining bridge in `lowering_error_to_compile_error`

The bridge is still in place at [`module_lowering.rs:37-58`](crates/sifr_driver/src/frontend/module_lowering.rs:37). Its current contract:

- If the `LoweringError.code` is `Some(_)`: pass it through unchanged (modulo the `Bare`/`ModulePrefixed` style transform).
- If the `LoweringError.code` is `None`: classify as `INTERNAL_COMPILER_PANIC` *and* prepend the user-visible message with `internal compiler error: HIR lowering emitted a diagnostic without canonical code: `.

I am satisfied with this contract for two reasons:

1. **No e2e fail fixture exercises the codeless arm.** The user's reported `cargo test -p sifr --test e2e test_e2e_fail` green run, combined with the strict `failure.code == expected.code` semantics of `test_e2e_fail` ([`tests/e2e.rs:2562`](crates/sifr/tests/e2e.rs:2562)), is a strong signal that every currently exercised user-input failure is on a coded path. Any future regression that re-introduces a codeless emission on a user-input path will be caught at test-time by a fixture mismatch (active code expected vs. `SIFR-INTERNAL-0001` observed) rather than silently shipping an exit-code regression.

2. **The user-visible message is now self-describing as a compiler bug.** A user who hits the bridge sees `internal compiler error: HIR lowering emitted a diagnostic without canonical code: ...`, which is the right framing for an unintended fallthrough. This is materially different from pass 1's behavior, where the user saw a normal-looking type-error message under a `SIFR-INTERNAL-0001` code.

That said, 342 raw `ctx.error(` sites remain across `crates/sifr_hir/src/lower/`. None are reached by current fail fixtures, but each is a latent risk for the same blocker pattern if a *new* fixture ever exercises one before its emission site is migrated. See finding 4 below.

### D. The new `default_args.rs` extraction is correct

- [`crates/sifr_hir/src/lower/default_args.rs`](crates/sifr_hir/src/lower/default_args.rs:1) is registered as `mod default_args;` at [`mod.rs:21`](crates/sifr_hir/src/lower/mod.rs:21), with the public-to-parent helper imported at [`mod.rs:82`](crates/sifr_hir/src/lower/mod.rs:82). The single call site in `lower_module_impl` ([`mod.rs:689`](crates/sifr_hir/src/lower/mod.rs:689)) replaces the prior 36-line inline block.
- Iteration shape is preserved exactly: positional args, then keyword-only args, with `regular_count = args.len() + usize::from(vararg.is_some())`. Insertion is gated on `!defaults.is_empty()` — same as before. Inner per-param helper `collect_param_default` is private to the module; outer `collect_function_defaults` is `pub(super)`.
- Privacy: `LowerCtx` is `pub(super)` in `crates/sifr_hir/src/lower/mod.rs`; `default_args` is a submodule of `crates/sifr_hir/src/lower`, which by Rust's privacy rules can read/write the private fields `function_defaults` and call the private `error_with_code` method. No new visibility leakage. No new public-API surface.
- The diagnostic is emitted via `ctx.error_with_code(DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT, format!(...))`. Message string is unchanged from the pre-refactor version, so the fixture continues to match by `expect-error` substring.
- File line count: 45 lines, well under any guardrail. `mod.rs` is now 1168 lines (limit 1200 per `MAX_LINES_BY_FILE`); the extraction was the right move to keep that file under bound.

### E. `SIFR-TYPE-0011` is coherent

- Constant: `DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT = SIFR-TYPE-0011` (severity Error) at [`codes.rs:39-40`](crates/sifr_diagnostics/src/codes.rs:39).
- Registry row: family `TYPE`, summary "Unsupported default argument expression.", representative fixture `crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr`, message template `function {function}: unsupported default argument expression for parameter {parameter}`, owner `sifr_hir::lower::typing_and_functions`, declared/dedupe args `function`, `parameter` ([`codes.rs:628-638`](crates/sifr_diagnostics/src/codes.rs:628)).
- Active-set membership at [`codes.rs:1335`](crates/sifr_diagnostics/src/codes.rs:1335).
- Generated public index row at [`docs/errors/diagnostic-codes.md:58`](docs/errors/diagnostic-codes.md:58); generated internal-reference row at [`internal_docs/diagnostic_codes.md:82`](internal_docs/diagnostic_codes.md:82); per-code page [`docs/errors/SIFR-TYPE-0011.md`](docs/errors/SIFR-TYPE-0011.md:1).
- Emission sites: [`default_args.rs:36-43`](crates/sifr_hir/src/lower/default_args.rs:36), [`typing_and_functions.rs:247-253`](crates/sifr_hir/src/lower/typing_and_functions.rs:247), [`typing_and_functions.rs:264-270`](crates/sifr_hir/src/lower/typing_and_functions.rs:264), [`classes.rs:474-480`](crates/sifr_hir/src/lower/classes.rs:474), [`classes.rs:522-528`](crates/sifr_hir/src/lower/classes.rs:522). All use `ctx.error_with_code(DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT, ...)`. Message templates are: `function '{}': unsupported default argument expression for parameter '{}'` for top-level/nested functions and `class '{}.__init__': ...` / `class '{}.{}': ...` for class init/method paths. The fixture expects the function-shape string and is satisfied by the module-level path. The class-shape variants don't have a representative fixture in this slice, which is acceptable because the registry's "representative fixture" requirement is per-code, not per-emission-site.
- Fixture line `# expect-error: SIFR-TYPE-0011: function 'pick': unsupported default argument expression for parameter 'x'` matches the runtime message exactly.
- Owner-module field declares `sifr_hir::lower::typing_and_functions` but the actual owner span is now four files: `default_args.rs`, `typing_and_functions.rs`, and two sites in `classes.rs`. The registry stores a single owner — listing `typing_and_functions` is reasonable since that file historically owned the policy, but it isn't strictly accurate after the extraction. See finding 1.

### F. Local validation envelope

The user reports clean runs of: `cargo test -p sifr_diagnostics -p sifr_driver --lib --tests`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check`, `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo test -p sifr --test e2e test_e2e_fail`, `cargo clippy --workspace -- -D warnings`, and the manual CLI exit-code check on `stdlib_wrong_type.sifr`. I traced each gate against the diff and confirmed the assertions/structures it tests are not broken by the change shape. AGENTS.md still asks for `scripts/run_all_tests.sh --profile quick` before PR; if this hasn't run yet, run it before merging — it's the gate that covers `check_diagnostic_schema_sync.py` and `check_diagnostic_docs_sync.py` together.

## Findings

### Non-blocking

1. **`SIFR-TYPE-0008` re-use spans two surface forms with different message templates.** The fixture `iter_heterogeneous_tuple_unsupported.sifr` now expects `SIFR-TYPE-0008` for the message `iter() tuple argument must have one statically provable element type`, but the registry's `SIFR-TYPE-0008` summary, message template, owner module, and representative fixture all describe the container-literal-conflict shape (`container literal has conflicting {element_kind} types: {expected} and {actual}` from `sifr_hir::lower::container_literal_specialization`). The two are conceptually adjacent (both about element-type uniformity in a container) but the registry metadata is single-shape. The slice doesn't add a per-emission-shape secondary fixture or generalize the template, so the JSON-arg-aware future renderer will see one diagnostic code with two distinct surface message shapes. Action (post-merge): either generalize `SIFR-TYPE-0008`'s template/summary to cover both shapes, or split a `SIFR-TYPE-0012` (or similar) for the iter-tuple element-type case and re-key that fixture. Not a blocker — `test_e2e_fail` matches by code+substring, and the registry validation only requires *a* representative fixture exists per active code, not that every emission site is registered.

2. **`SIFR-TYPE-0011` owner-module is single-valued but emission spans four files.** Same shape of comment as #1: registry says `sifr_hir::lower::typing_and_functions` owns it, but the call sites are also in `default_args.rs` and two places in `classes.rs`. This is a pre-existing pattern (other codes with multiple owners take the same single-valued shortcut). Not a regression.

3. **`collect_function_defaults` now exists in two places.** `default_args::collect_function_defaults` (module-level, mutates `ctx.function_defaults`) and `typing_and_functions::collect_function_defaults` (nested-function, returns `Vec<(usize, HirExpr)>` for the caller to insert) have nearly identical inner loops. The duplication is small and the two functions have different return contracts, but a follow-up could unify them into a `collect_function_defaults_into(&mut Vec<(usize, HirExpr)>, ...)` helper plus thin wrappers. Out of scope for this slice; flagging as a maintainability note.

4. **No automated guardrail prevents future codeless `ctx.error(` regressions on user-input paths.** Pass 1 suggested a `scripts/check_no_codeless_lowering_emissions.py`-style check to track the count down monotonically; pass 2 does not add it. With 342 raw `ctx.error(` sites still present in `crates/sifr_hir/src/lower/`, the bridge's defensive role is the only safety net for unmigrated paths. The risk is bounded by `test_e2e_fail`'s strict-code semantics — a future fixture exercising a codeless path would fail loudly — but a build-time guardrail would catch the regression *before* the fixture exists. Recommend tracking this as a follow-up under the same milestone, particularly since the issue tracker policy at [`issues/...md:1175`](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1175) already says "HIR user diagnostics must not be emitted through raw `ctx.error(String)`" — a script can mechanize this rule.

5. **The `codeless_lowering_error_is_internal_compiler_diagnostic` test now locks in the bridge's exact output.** The assertion checks `compile_error.message ==
"internal compiler error: HIR lowering emitted a diagnostic without canonical code: [main] expected int, got str"` ([`module_lowering.rs:111-115`](crates/sifr_driver/src/frontend/module_lowering.rs:111)). This is fine for pass 2's "compiler bug" framing but ties the bridge's user-visible string to a specific format. If you later move to pass 1 suggestion #3 (make `LoweringError.code` mandatory at the type-system level), this test should be replaced by a *type-level* invariant (the codeless arm becomes unrepresentable), not just a different string assertion. Tracking as a follow-up.

6. **`internal_docs/diagnostic_emission_inventory.md` retains historical references to `SIFR-TYPE-0001`** at [`:120`](internal_docs/diagnostic_emission_inventory.md:120), [`:142`](internal_docs/diagnostic_emission_inventory.md:142), [`:143`](internal_docs/diagnostic_emission_inventory.md:143), and [`:262`](internal_docs/diagnostic_emission_inventory.md:262). All four are correctly framed as past state ("Removed catch-all", "Removed; harness samples now use active parser codes", "[describes prior state]: `SIFR-TYPE-0001` plus message-embedded `[E2501]`"). Acceptable as historical references; no change needed.

7. **Issue-tracker line 68 wording still says "in progress."** Mechanical: the slice is implementation-complete and reviewer-satisfied at this pass; flip to the standard "implementation complete and reviewer-satisfied" wording on PR open per the convention used in slices 2b.30/2b.31/2b.32. Pre-existing nit from pass 1.

### Out of scope (correctly carved out)

- Migration of the remaining ~342 raw `ctx.error(` sites in `crates/sifr_hir/src/lower/` whose error paths are not exercised by current fail fixtures. The slice intentionally narrows on the ten exercised paths plus the related class/function default-argument variants.
- Promoting `LoweringError.code: Option<DiagnosticCode>` to a non-`Option` field (pass 1 suggestion #3). Out of scope for this slice; eliminating `legacy_error_records_no_structured_identity` is contingent on the 342-site migration.
- Adding an exit-code contract to `verification/validation_contracts/manifest.json` (pass 1 suggestion #5). Worth doing in a follow-up to lock in the user-vs-internal exit-code split for these specific 10 fixtures, but not required for the current slice to be correct.
- Changes to `LoweringError` span/line/col, renderer behavior, or compact-grouping logic — unchanged.

## Summary

Pass 2 takes pass 1's recommended option-1 path: it migrates the HIR call sites and re-keys the ten affected fixtures back to active semantic codes, leaving the codeless-arm bridge as a defensive *compiler-bug* surface. The change set is internally consistent, the registry/`CompileError` surface is unchanged from pass 1's "approve-on-sight" envelope, the new `SIFR-TYPE-0011` code is coherent, the `default_args.rs` extraction is privacy-clean and respects the HIR maintainability guardrails, and the empirical CLI exit-code regression is gone. The remaining items are non-blocking follow-ups (multi-shape code re-use, cross-file owner accuracy, helper de-duplication, optional guardrail script, optional manifest exit-code contract, and the issue-tracker wording flip on PR open).

**Recommend merge after `scripts/run_all_tests.sh --profile quick` runs clean locally.**
