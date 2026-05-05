# Review: milestone_diag_4a — Renderer Integration (Pre-Implementation, Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a` (based on `origin/main` after merged [PR #1670](https://github.com/sifr-lang/sifr/pull/1670))
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Inventory: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md)
Prior wave reviews: pass-1..3 for `diag_2b`, pass-1..2 for `diag_3`, pass-1..3 for `diag_2a`, pass-1..3 for `diag_1`.

This is a pre-implementation review. No work has landed on the branch yet. The review focuses on what the code currently looks like, what `milestone_diag_4a` requires, and the concrete risks, code areas, test gaps, and sequencing decisions implementers should resolve before writing the first line of code.

## Verdict

**Proceed with explicit sequencing decisions.** The new diagnostic model from `diag_1`/`2a`/`2b` is in place — `crates/sifr_diagnostics` already exposes `SifrDiagnostic`, `DiagnosticBuilder`, `DiagnosticSink`, `SourceMap`/`SourceSpan`, the canonical ordering pipeline, and the human/compact/JSON envelope renderers ([crates/sifr_diagnostics/src/render/presentation.rs:10-32](../crates/sifr_diagnostics/src/render/presentation.rs:10)). What remains is the integration cliff: every `CompileError` construction site, the entire HIR `ctx.error(String)` surface, the workspace prefix classifier, `CompilePhase::TypeCheck => "SIFR-TYPE-0001"`, the `apply_diagnostic_recovery_limits` legacy grouper, and 91+ fail fixtures all live on the legacy path. The milestone's four-PR structure is the right shape, but several latent assumptions need to be made explicit before implementation, primarily around spans, workspace code timing, and the `Severity::Help` deletion.

## Scope-bounded summary

`milestone_diag_4a` covers (from [issues/…-diagnostics.md:837-867](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:837)):

1. Human/compact/JSON renderers consume `SifrDiagnostic`.
2. Remove workspace message-prefix code inference (`CompileError::workspace_diagnostic_code`).
3. All renderers share one deterministic canonical post-admission diagnostic stream; admission is a no-op pass in `diag_4a` (the cap activates in `diag_10`).
4. Compact grouping uses `(severity, code, message_template, primary display file)`.
5. Delete `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` (any unmigrated `TypeCheck` path must already use an inventory-assigned canonical code through `SifrDiagnostic` transport, or fail to compile).
6. Mechanical transport migration of every previously `CompilePhase::TypeCheck`-routed HIR/type-system call site to inventory-assigned `SifrDiagnostic` emission.
7. Replace user-facing `LoweringError { message, line, col }` with `LoweringOutcome` + `DiagnosticSink`.
8. Migrate parser adapters, workspace/project discovery, codegen boundaries, build/materialization/rustc, and test-runner emission paths covered by the inventory into `SifrDiagnostic` transport.

The four expected sub-PRs from the issue are: (1) renderer integration + message-based code inference removal, (2) `LoweringError` replacement, (3) parser/workspace/codegen/build/rustc-boundary/test-runner transport, (4) `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport.

## Current-state inventory (pre-implementation snapshot)

Shared diagnostic infrastructure (shipped by prior milestones):

- [crates/sifr_diagnostics/src/lib.rs:9-17](../crates/sifr_diagnostics/src/lib.rs:9) re-exports `SifrDiagnostic`, `DiagnosticBuilder`, `DiagnosticSink`, `ErrorEmitted`, `SourceMap`, `SourceSpan`, `DiagnosticEnvelope`, plus `render_sink_human`, `render_sink_compact`, `render_sink_json`.
- The canonical ordering pipeline and template-based compact grouping already exist: [crates/sifr_diagnostics/src/render/mod.rs:88-204](../crates/sifr_diagnostics/src/render/mod.rs:88) sorts by `(path_rank, path, byte_start, byte_end, severity_rank, kind_rank, code, message_template, args, insertion_order)`, and [crates/sifr_diagnostics/src/render/presentation.rs:133-149](../crates/sifr_diagnostics/src/render/presentation.rs:133) builds the `(severity_rank, code, message_template, primary_display_file)` compact key.
- `Severity` is `Error | Warning | Note` only ([crates/sifr_diagnostics/src/model/mod.rs:9-16](../crates/sifr_diagnostics/src/model/mod.rs:9)). Top-level `Help` is intentionally absent. Help text flows through `help: Option<String>` or `ChildSeverity::Help` children.
- `DiagnosticBuilder::source(...)` requires a `SourceSpan`; `DiagnosticBuilder::internal(...)` does not. Severity must match the registry-declared severity ([model/mod.rs:312-332](../crates/sifr_diagnostics/src/model/mod.rs:312)). Drop discipline trips a `debug_assert!` and increments `UNEMITTED_DIAGNOSTIC_DROP_COUNT` ([model/mod.rs:268-290, 432-442](../crates/sifr_diagnostics/src/model/mod.rs:268)).
- Active codes the mechanical transport will need (sample): `SIFR-NAME-0001..0004`, `SIFR-IMPORT-0001..0002`, `SIFR-TYPE-0002..0009/0901/0902`, `SIFR-DECIMAL-0001..0008`, `SIFR-CALL-0001..0005`, `SIFR-OWN-0001..0004`, `SIFR-FLOW-0001..0003/0901`, `SIFR-MATCH-0001..0003`, `SIFR-PROTO-0001..0004`, `SIFR-CLASS-0001..0004`, `SIFR-RESULT-0001..0003`, `SIFR-STDLIB-0001..0004`, `SIFR-WORKSPACE-0001..0104`, `SIFR-CODEGEN-0002`, `SIFR-BUILD-0002..0006`, `SIFR-INTERNAL-0001` ([crates/sifr_diagnostics/src/codes.rs:10-119](../crates/sifr_diagnostics/src/codes.rs:10)).

Legacy surface still in place (must be migrated or wired through):

- [crates/sifr_driver/src/diagnostics.rs:25-160](../crates/sifr_driver/src/diagnostics.rs:25) — `CompileError`, `CompilePhase`, `CompilerDiagnostic`, `Severity` (with `Help`), `DiagnosticSpan` (line/col only, no byte offsets), and the prefix classifier `workspace_diagnostic_code` plus `CompilePhase => SIFR-…-0001` mapping.
- [crates/sifr_driver/src/diagnostics.rs:166-221](../crates/sifr_driver/src/diagnostics.rs:166) — `apply_diagnostic_recovery_limits` does both message-text grouping (key includes the rendered `message`) and the per-group cap of 5 with a synthetic "... +N more similar diagnostics" diagnostic. This grouper is incompatible with the new pipeline (message-template grouping + canonical sort + admission pass).
- [crates/sifr/src/main.rs:288-365](../crates/sifr/src/main.rs:288) — CLI compact rendering (uses `sifr_driver::DiagnosticSpan`/`Severity`), [main.rs:367-413](../crates/sifr/src/main.rs:367) — the legacy `render_compile_errors` dispatcher, [main.rs:215-231, 270-286](../crates/sifr/src/main.rs:215) — `Severity::Help` ranking and counting.
- HIR lowering's user-facing emission is entirely string-based: [crates/sifr_hir/src/lower/mod.rs:83-97](../crates/sifr_hir/src/lower/mod.rs:83) defines `LoweringError { message, line, col }` and [mod.rs:209-215](../crates/sifr_hir/src/lower/mod.rs:209) is the `ctx.error(String)` constructor. The inventory counts 489 raw HIR `ctx.error(...)` call sites across 22 files ([internal_docs/diagnostic_emission_inventory.md:7](../internal_docs/diagnostic_emission_inventory.md:7)); a quick `rg "ctx\.error\(" crates/sifr_hir/src` confirms the same order of magnitude (≈510 including `self.error` variants).
- HIR currently has no `SourceId`, no `SourceMap`, and no AST-range plumbing in `LowerCtx`. There is no `SourceMap` instance constructed anywhere outside `crates/sifr_diagnostics` tests (verified by grep — zero hits in driver/CLI/HIR).
- Decimal pseudo-codes are still embedded in HIR/type-system messages: 18 occurrences in `expressions.rs`/`decimal_methods.rs` and 2 in `sifr_type_system::check` ([sifr_hir/src/lower/expressions.rs:875-1097](../crates/sifr_hir/src/lower/expressions.rs:875), [decimal_methods.rs:42-198](../crates/sifr_hir/src/lower/decimal_methods.rs:42), [sifr_type_system/src/check.rs:31-45](../crates/sifr_type_system/src/check.rs:31)). Decimal migration formally lands in `diag_6`, but `diag_4a`'s mechanical transport will route these via the retired `SIFR-TYPE-0001` unless the routing decision is made early.
- `sifr_type_system::TypeError` and `TypeErrorKind` survive ([crates/sifr_type_system/src/lib.rs:30-65](../crates/sifr_type_system/src/lib.rs:30)). Their deletion is `diag_7`'s job, but `diag_4a` must decide how to forward `TypeError`-bearing call sites (e.g., 24 `TypeErrorKind::*` constructions per the inventory) into the canonical sink.
- `LoweringResult { reveal_types: Vec<String>, warnings: Vec<String> }` is currently the side-channel for non-error diagnostics ([sifr_hir/src/lower/mod.rs:430-437](../crates/sifr_hir/src/lower/mod.rs:430)) and is consumed via `emit_frontend_diagnostics` writing raw `stderr` lines ([sifr_driver/src/frontend/module_lowering.rs:44-51](../crates/sifr_driver/src/frontend/module_lowering.rs:44)). `LoweringOutcome` is defined ([sifr_hir/src/lowering_outcome.rs:1-7](../crates/sifr_hir/src/lowering_outcome.rs:1)) but unused.
- Test/baseline surface: 91 fail fixtures assert `SIFR-TYPE-0001` (95 total per inventory), 18 decimal fixtures embed `[E25xx]`, `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-{compact,json}.stderr.txt` lock the legacy code+message format ([decimal_invalid_literal/baselines/check-compact.stderr.txt:1-3](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:1)), and the e2e harness still accepts both `SIFR-TYPE-0001` and `[E2507]` substring matches ([crates/sifr/tests/e2e.rs:584-685](../crates/sifr/tests/e2e.rs:584)).

## Concrete implementation risks

### R1 — HIR has no SourceId/SourceSpan plumbing; mechanical transport cannot fabricate spans (high)

The mechanical transport migration in sub-PR 4 must produce `SifrDiagnostic` values for every HIR `ctx.error(String)` site. `DiagnosticBuilder::source(...)` requires a real `SourceSpan` ([model/mod.rs:312-323](../crates/sifr_diagnostics/src/model/mod.rs:312)), which requires a registered `SourceId`. Today, `LowerCtx::new()` knows nothing about a source map ([sifr_hir/src/lower/mod.rs:165-204](../crates/sifr_hir/src/lower/mod.rs:165)). Three options exist; one must be chosen explicitly in the implementation plan:

- **(a) Thread `SourceId` only.** `LowerCtx` holds a `SourceId` (assigned by the driver per module). `ctx.error(message)` becomes `ctx.emit_error(diag)` where the call site supplies the AST node range; sites without an AST range use `TextRange::default()` and the resulting `SourceSpan` covers byte 0..0 of the module file. This is the spec-conformant approach: the milestone says "`primary_span` populated where source exists" is a `diag_9` deliverable, not `diag_4a`, but the model still requires *some* `SourceSpan` per source diagnostic. **Caveat:** [issues/…-diagnostics.md:709](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:709) says "Diagnostics that truly have no real source mapping are internal compiler diagnostics and use `SIFR-INTERNAL-*`; do not fabricate a source span." A `0..0` span on a real source file is *not* a fabricated source span — it points at the start of the module — but reviewers must accept this transitional convention.
- **(b) Thread real AST ranges everywhere now.** Touch all ~510 call sites, accept the much larger PR, and finish what `diag_9` was scoped to do.
- **(c) Route all HIR diagnostics as `SifrDiagnostic::Internal` until `diag_9`.** Forbidden by the issue: `SIFR-INTERNAL-*` is reserved for compiler invariants and panic-boundary failures, not user input errors ([issues/…-diagnostics.md:1217-1224](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1217)).

**Recommendation:** Option (a). The implementation plan should explicitly say "every HIR/type-system source diagnostic carries a `SourceSpan(module_source_id, ast_range_or_TextRange::default())` in `diag_4a`; `diag_9` widens to AST-accurate ranges". This needs to be declared up front so `diag_9`'s span-completion pass has a clear "fix `0..0` placeholder" signal.

### R2 — Workspace prefix classifier deletion must follow, not precede, workspace construction-site migration (high)

`CompileError::workspace_diagnostic_code` ([sifr_driver/src/diagnostics.rs:96-128](../crates/sifr_driver/src/diagnostics.rs:96)) is currently the only source of `SIFR-WORKSPACE-0001..0103` codes. It infers them from message prefixes against `CompileError`s constructed with `CompilePhase::Build` in `crates/sifr_driver/src/build/workspace.rs`, `crates/sifr_driver/src/project/discovery.rs`, `crates/sifr_driver/src/workspace/mod.rs`, and similar sites (~7 sites in `build/workspace.rs`, 6 in `project/discovery.rs`, 2 in `workspace/mod.rs` per the inventory). If the classifier is deleted before those construction sites are converted to use `WORKSPACE_*` `DiagnosticCode` constants, every workspace error degrades to the retired `SIFR-BUILD-0001` and the `decimal_invalid_literal`-style verification baselines flip to wrong codes silently.

**Recommendation:** Within sub-PR 1 (renderer integration), delete the classifier *and* migrate the construction sites in the same PR. Alternatively, split sub-PR 1 into 1a (renderer scaffolding, classifier still present) and 1b (workspace construction-site migration + classifier deletion). The issue's sub-PR 3 ("Parser, workspace, codegen, build, rustc-boundary, and test-runner transport migration") is the cleaner home for the workspace migration — but then sub-PR 1 must keep the classifier alive. Pick one of those orderings and document it.

### R3 — `apply_diagnostic_recovery_limits` becomes dead code with semantic differences (medium)

The legacy grouper at [sifr_driver/src/diagnostics.rs:178-221](../crates/sifr_driver/src/diagnostics.rs:178) groups by *rendered message* and synthesizes a `... +N more similar diagnostics` summary diagnostic. Its replacement — the canonical sort + presentation compact renderer — groups by `message_template`, presents up to 5 representative *locations* per group, and never invents synthesized text-bearing diagnostics. The CLI tests at [main.rs:1268-1396](../crates/sifr/src/main.rs:1268) bake the legacy "x5 + ... +3 more similar diagnostics" shape into snapshots; those tests will fail or assert the wrong thing once the new pipeline is wired. The CLI also keys compact summary by message text and parses `message.starts_with("... +")` to detect summary groups ([main.rs:304-305](../crates/sifr/src/main.rs:304)) — that detection is no longer meaningful.

**Recommendation:** sub-PR 1 must delete `apply_diagnostic_recovery_limits` (or quarantine it behind `#[deprecated]` test-only) and rewrite the four `test_compact_renderer_*` tests to assert the new grouping shape. Do not preserve the old behavior in parallel; the milestone's "all renderers consume the same canonical stream" rule forbids two competing groupers. The CLI also imports `apply_diagnostic_recovery_limits` from `sifr_driver` ([sifr_driver/src/lib.rs:23-27](../crates/sifr_driver/src/lib.rs:23)) — that re-export must die in the same PR or `diag_4b`.

### R4 — `Severity::Help` is in the legacy CLI path but absent from the canonical model (medium)

[sifr_diagnostics::Severity](../crates/sifr_diagnostics/src/model/mod.rs:9) is exactly `Error | Warning | Note`. The legacy `sifr_driver::Severity` adds `Help` ([sifr_driver/src/diagnostics.rs:39-45](../crates/sifr_driver/src/diagnostics.rs:39)), and existing tests/snapshots emit `Severity::Help` as a top-level rank ([main.rs:215-231, 270-286, 1422-1445](../crates/sifr/src/main.rs:215)). The `test_compact_renderer_snapshot_multi_severity_group_order` snapshot explicitly asserts `help [SIFR-CODEGEN-0001] consider adding a type annotation (x1)` rendering, which has no counterpart in the canonical renderer.

**Recommendation:** sub-PR 1 deletes the legacy `Severity::Help` variant and the matching snapshot lines. The canonical model already handles "help" via `help: Option<String>` and `ChildSeverity::Help` children — there is no migration semantics to preserve, just snapshot deletion. Confirm no real emission site emits a top-level `Severity::Help`; a `rg "Severity::Help"` outside tests returns only the legacy enum and ranking infrastructure.

### R5 — `CompilePhase::TypeCheck` is referenced from many non-HIR sites used as panic boundaries and stderr labels (high)

`CompilePhase::TypeCheck` is *not* used only for code derivation — it is also passed to `run_with_panic_boundary` in `cmd_check` ([main.rs:471-479](../crates/sifr/src/main.rs:471)) and to `Display for CompileError` for the human-renderer label ([sifr_driver/src/diagnostics.rs:223-233](../crates/sifr_driver/src/diagnostics.rs:223)). Five non-HIR sites also construct `CompileError { phase: CompilePhase::TypeCheck, … }` for invariant/forwarding messages: [build/entrypoint.rs:221](../crates/sifr_driver/src/build/entrypoint.rs:221), [project/compile_order.rs:193](../crates/sifr_driver/src/project/compile_order.rs:193), [stdlib/bootstrap.rs:56](../crates/sifr_driver/src/stdlib/bootstrap.rs:56), [test_runner/orchestrator.rs:110](../crates/sifr_driver/src/test_runner/orchestrator.rs:110), and the CLI panic-boundary call. Deleting the `TypeCheck => "SIFR-TYPE-0001"` arm without addressing each of these collapses them onto a now-retired code or a fallthrough.

**Recommendation:** Each non-HIR `CompilePhase::TypeCheck` site must be reassigned in sub-PR 4 (or its preparatory sub-PR):

- The CLI `cmd_check` panic boundary → `SIFR-INTERNAL-0001` (it is an internal compiler panic).
- `run_codegen_with_boundary` ([sifr_driver/src/diagnostics.rs:255-267](../crates/sifr_driver/src/diagnostics.rs:255)) currently uses `CompilePhase::Codegen` → `SIFR-INTERNAL-0001` is the right home for the panic case; the non-panic "codegen failed" path is `SIFR-CODEGEN-0002`.
- `build/entrypoint.rs:221` is the "internal error: frontend lowering missing 'main' module" invariant → `SIFR-INTERNAL-0001`.
- `project/compile_order.rs:193` (dependency cycle) → per inventory line 93, `SIFR-WORKSPACE-0104` (workspace import cycle, which is now an `Active` registry entry per [codes.rs:109](../crates/sifr_diagnostics/src/codes.rs:109)) or `SIFR-IMPORT-*` depending on graph layer.
- `stdlib/bootstrap.rs:56` and `test_runner/orchestrator.rs:110` → forwarders that should now route the underlying frontend `SifrDiagnostic` directly rather than re-wrapping; if forwarding is impossible without losing identity, `SIFR-STDLIB-0003` for stdlib bootstrap failures and `SIFR-INTERNAL-0001` for orchestrator invariants are the inventory-assigned homes.

The implementation plan must enumerate these five sites explicitly. The `CompileError::Display` label `"type error"` ([diagnostics.rs:227](../crates/sifr_driver/src/diagnostics.rs:227)) becomes useless once the variant is gone; both human and compact paths should derive labels from severity/code via the canonical renderers.

### R6 — TypeError survives `diag_4a` but cannot leak into renderers (medium)

`sifr_type_system::TypeError` and `TypeErrorKind` are deleted in `diag_7` ([issues/…-diagnostics.md:942](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:942)). But ~24 construction sites and the public functions `type_check_binary_op`, `type_check_bool_op`, `type_check_comparison`, `type_check_unary_op` continue to return `Result<Type, TypeError>` ([crates/sifr_type_system/src/check.rs:26](../crates/sifr_type_system/src/check.rs:26)). HIR call sites that today do `ctx.error(error.message)` (the inventory's "type-error string forwarding" pattern, [internal_docs/diagnostic_emission_inventory.md:120](../internal_docs/diagnostic_emission_inventory.md:120)) must be rewritten in sub-PR 4 so that *the HIR caller* synthesizes the canonical `SifrDiagnostic` from the typed `TypeErrorKind` payload (using its `expected`/`actual`/`op`/`ty` fields) plus the AST span.

The issue [explicitly forbids](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:550) `impl From<TypeError> for SifrDiagnostic` as a long-term design but allows a "short-lived mechanical adapter" in a single migration PR. **Recommendation:** keep that mechanical adapter local to sub-PR 4, in the HIR call site (not as a `sifr_type_system` impl), and delete it in `diag_7`. Note that `TypeError` lacks a span — the HIR caller has the span; the adapter must take it as a parameter.

### R7 — Decimal pseudo-codes still emit `SIFR-TYPE-0001` content during `diag_4a` (medium)

Decimal migration to canonical `SIFR-DECIMAL-*` codes is `diag_6`'s deliverable. In `diag_4a`'s mechanical transport, the 18 decimal `[E25xx]`-bearing message strings in HIR will be assigned a code by inventory mapping. The inventory ([internal_docs/diagnostic_emission_inventory.md:312-319](../internal_docs/diagnostic_emission_inventory.md:312)) maps each pseudo-code 1:1 to a canonical decimal code, so the natural mechanical assignment in `diag_4a` is to *use the canonical decimal codes immediately*. That conflicts with `diag_6`'s framing as "decimal first migration" — but the only `diag_6` work that wouldn't be done by `diag_4a` is removing the literal `[E25xx]` substring from the rendered message and updating the 18 fixtures.

**Recommendation:** the `diag_4a` mechanical transport should *not* emit `SIFR-TYPE-0001`-tagged decimal diagnostics — it cannot, since `SIFR-TYPE-0001` is retired and the registry rejects it. It should emit `SIFR-DECIMAL-*` directly with the existing `[E25xx]`-bearing message strings; `diag_6` then strips the embedded pseudo-code from the message templates and updates the 18 decimal fixtures. Document this overlap so reviewers don't flag "decimal migration before decimal milestone".

### R8 — Fail-fixture annotation churn lands inside `diag_4a` whether the milestone admits it or not (medium)

91 fail fixtures assert `# expect-error: SIFR-TYPE-0001` (or `[SIFR-TYPE-0001] [E25xx]`). Once sub-PR 4 lands and HIR emits inventory-assigned codes (`SIFR-NAME-0001`, `SIFR-TYPE-0002`, `SIFR-OWN-0001`, `SIFR-CALL-0001`, etc.), every one of those fixtures will fail until its `# expect-error:` line is updated. Test-harness contract cleanup (`diag_5`) is sequenced *after* `diag_6`, so the harness still permits both legacy and canonical strings; that's fine. But the fixture annotations themselves *must* be updated within sub-PR 4 of `diag_4a`, simultaneous with the emission change, otherwise the e2e suite breaks.

**Recommendation:** sub-PR 4's PR description must call out the fixture annotation re-keying as a deliverable. The work is mechanical (the inventory maps each emission category to a code), but the volume is large (91 fixtures × 1 annotation each, plus the verification baselines under `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/`, plus the project verification baselines that already use canonical workspace codes and should not need changes). This is a real risk if the PR is cut without explicit fixture coverage.

### R9 — `LoweringResult.reveal_types` and `.warnings` are still `Vec<String>` (low/medium)

Replacing user-facing `LoweringError` with `LoweringOutcome` is sub-PR 2's job. But `LoweringResult` independently carries `reveal_types: Vec<String>` and `warnings: Vec<String>` ([sifr_hir/src/lower/mod.rs:430-437](../crates/sifr_hir/src/lower/mod.rs:430)) that the driver writes to stderr as raw lines ([frontend/module_lowering.rs:44-51](../crates/sifr_driver/src/frontend/module_lowering.rs:44)). The issue's [Non-Error Diagnostics section](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1226) says these must become structured `Severity::Note` (`SIFR-TYPE-0902`) and `Severity::Warning` (`SIFR-TYPE-0901`, `SIFR-FLOW-0901`) diagnostics in the canonical stream, *not* stderr side-channels. Phase Definition of Done line [issues/…-diagnostics.md:1260](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1260) says "warnings and `reveal_type` output are structured diagnostics in the canonical diagnostic stream".

`milestone_diag_4a` doesn't explicitly schedule that conversion, but the milestone's wording — "All renderers operate on `SifrDiagnostic` exclusively" — implies that any user-visible renderer text must come from `SifrDiagnostic`. **Recommendation:** sub-PR 2 (LoweringError replacement) should also fold `reveal_types`/`warnings` into the same `LoweringOutcome.diagnostics: Vec<SifrDiagnostic>` stream using `TYPE_REVEAL_TYPE`/`TYPE_ARITHMETIC_OVERFLOW_RISK`/`FLOW_UNREACHABLE_STATEMENT`. If implementers prefer to keep the side-channel until `diag_10` (when `SIFR-INTERNAL-0002` cap-summary work activates), call that out explicitly so the renderer milestone doesn't claim "all output via canonical stream" while warnings still escape to stderr.

### R10 — `LoweringError` symbol is used by 7 HIR unit-test files (low)

The HIR unit tests at [own_mut_param_tests.rs](../crates/sifr_hir/src/lower/own_mut_param_tests.rs), [expressions_tests.rs](../crates/sifr_hir/src/lower/expressions_tests.rs), [nested_function_tests.rs](../crates/sifr_hir/src/lower/nested_function_tests.rs), [guarded_index.rs:183-194](../crates/sifr_hir/src/lower/guarded_index.rs:183), [type_alias_tests.rs](../crates/sifr_hir/src/lower/type_alias_tests.rs), [numeric_sentinels.rs:314](../crates/sifr_hir/src/lower/numeric_sentinels.rs:314), [own_mut_semantics_tests.rs](../crates/sifr_hir/src/lower/own_mut_semantics_tests.rs) all destructure `Result<HirModule, Vec<LoweringError>>`. Sub-PR 2 must update each test helper to consume `LoweringOutcome { result, diagnostics }`. The milestone says `LoweringError` becomes "private transitional plumbing" in `diag_4a` and is fully deleted in `diag_11`. Confirm with the implementer whether `LoweringError` survives as a private struct or is deleted now: the issue text at [issues/…-diagnostics.md:734](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:734) says "`LoweringError` becomes private transitional plumbing only" in `diag_1` and "is removed from user-facing paths in `milestone_diag_4a`" — that allows it to stay as an internal type. The cleanest path: keep `LoweringError` private, used as the input form for the new `LowerCtx` -> `SifrDiagnostic` conversion, and delete in `diag_11`.

### R11 — Canonical sort uses display path, but driver doesn't yet feed display paths (medium)

[crates/sifr_diagnostics/src/render/mod.rs:179-204](../crates/sifr_diagnostics/src/render/mod.rs:179) sorts by `source_map.display_path(span.source_id)`. In project mode, display paths must be set per module so multi-file fixtures show the correct file ([issues/…-diagnostics.md:638-646, 700-714](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:638)). The driver currently has no `SourceMap`. Sub-PR 1 (renderer integration) must:

- Construct a `SourceMap` per compilation invocation (CLI level) and pass `&SourceMap` to the renderers.
- For each parsed module, register it via `SourceMap::register_source_with_metadata(display_path, canonical_path, module_name, text)`. Display paths must follow the diagnostic path-remapping policy (project-relative or `<stdin>`).
- Plumb the resulting `SourceId` into the parser adapter and HIR `LowerCtx` so emitted diagnostics carry valid spans.

**Recommendation:** define a `DriverSession` (or similar) that owns the `SourceMap` and `DiagnosticSink`, and route every `compile_*`/`check_*`/`emit_*` flow through it. Without this, sub-PR 4's HIR transport has no `SourceId` to attach to diagnostics — back to R1.

### R12 — Drop discipline trips a `debug_assert!` on every test path that abandons a builder (low/medium)

[model/mod.rs:268-290, 432-442](../crates/sifr_diagnostics/src/model/mod.rs:268) panics in debug builds if a `SifrDiagnostic` or `DiagnosticBuilder` is dropped without `build`/`emit`/`return`/`cancel`. The HIR mechanical transport must propagate the diagnostic through `LowerCtx::emit_error(...)` synchronously; any `?` early-return that drops a partially built diagnostic, or any `if let Some(span) = … { … } else { /* discard */ }` pattern, will trip the assertion. This is healthy discipline, but it means tests must be explicit about builder consumption, and the mechanical transport cannot use the previously-loose pattern of "build a string, decide later whether to push to errors".

**Recommendation:** introduce a small `LowerCtx::report(code, span, template, args)` helper that consumes the builder in one expression — every transport call site uses this helper, never raw `DiagnosticBuilder`. This both keeps the call sites short and avoids the drop-discipline trap.

## Required code areas (file-level checklist)

These are the files the implementation will need to touch, grouped by sub-PR:

**Sub-PR 1 — Renderer integration + message-prefix removal:**
- [crates/sifr/src/main.rs](../crates/sifr/src/main.rs) — replace `render_compile_errors` with a session-aware `render_sink_*` dispatcher; delete `render_compact_diagnostics`, `compact_severity_summary`, `compact_location_label`, `severity_rank`/`severity_label` for `Severity::Help`; rewrite `test_compact_renderer_*` tests against the canonical renderers; remove imports of `apply_diagnostic_recovery_limits`, `compile_errors_to_diagnostics`, `CompilerDiagnostic`, `Severity` from `sifr_driver`.
- [crates/sifr_driver/src/diagnostics.rs](../crates/sifr_driver/src/diagnostics.rs) — delete `workspace_diagnostic_code`, `apply_diagnostic_recovery_limits`, `Severity::Help`, the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` arm, the `to_diagnostic` method, and `compile_errors_to_diagnostics`. Either delete `CompilerDiagnostic`/`DiagnosticSpan`/`DiagnosticChild`/`DiagnosticSuggestion`/`RelatedSpan` outright or keep them as private bridge types until `diag_4b`. Decide explicitly.
- [crates/sifr_driver/src/lib.rs:23-27](../crates/sifr_driver/src/lib.rs:23) — update re-exports; the milestone allows transitional `sifr_driver` re-exports of `sifr_diagnostics` types but mandates removal in `diag_4b`.
- [crates/sifr_driver/src/tests/diagnostics.rs](../crates/sifr_driver/src/tests/diagnostics.rs) — delete or rewrite the four legacy tests (the `SIFR-TYPE-0001` recovery-limit fixtures explicitly named in the inventory).
- New: a session/source-map owner in `sifr_driver` (probably `crates/sifr_driver/src/session.rs`) that constructs `SourceMap` + `DiagnosticSink` per invocation.

**Sub-PR 2 — `LoweringError` → `LoweringOutcome`/`DiagnosticSink`:**
- [crates/sifr_hir/src/lower/mod.rs:83-215, 430-437, 470-501](../crates/sifr_hir/src/lower/mod.rs:83) — `LowerCtx` accepts a `SourceId`; `lower_module*` returns `LoweringOutcome`; `LoweringError` becomes `pub(crate)`; `reveal_types`/`warnings` become `Vec<SifrDiagnostic>` (per R9 decision).
- [crates/sifr_hir/src/lib.rs:17-22](../crates/sifr_hir/src/lib.rs:17) — update public exports.
- [crates/sifr_hir/src/lowering_outcome.rs](../crates/sifr_hir/src/lowering_outcome.rs) — add `has_errors()` accessor; consider whether `result` is `Option`-bearing on hard-fail paths.
- 7 HIR unit test files listed in R10 — update destructuring patterns.
- [crates/sifr_codegen/src/lib_codegen_tests.rs:15](../crates/sifr_codegen/src/lib_codegen_tests.rs:15) — update one call site.

**Sub-PR 3 — Parser/workspace/codegen/build/rustc-boundary/test-runner transport:**
- [crates/sifr_driver/src/frontend/api.rs:14-35](../crates/sifr_driver/src/frontend/api.rs:14) — parser adapter emits `SIFR-PARSE-0002..0009` with `parser_category` JSON arg via the registry. Per the inventory ([internal_docs/diagnostic_emission_inventory.md:51-61](../internal_docs/diagnostic_emission_inventory.md:51)), this is mechanical: map Ruff `ParseErrorType` variants to category codes.
- [crates/sifr_driver/src/build/workspace.rs](../crates/sifr_driver/src/build/workspace.rs) — 7 sites → `SIFR-WORKSPACE-0001..0004`, `SIFR-BUILD-0002..0006`.
- [crates/sifr_driver/src/build/entrypoint.rs](../crates/sifr_driver/src/build/entrypoint.rs) — 3 sites → `SIFR-BUILD-*` or `SIFR-INTERNAL-0001` for invariants.
- [crates/sifr_driver/src/build/materialize.rs](../crates/sifr_driver/src/build/materialize.rs) — 1 site → `SIFR-BUILD-0002`.
- [crates/sifr_driver/src/project/discovery.rs](../crates/sifr_driver/src/project/discovery.rs) — 6 sites split between `SIFR-WORKSPACE-*` and reachable `SIFR-PARSE-*` paths.
- [crates/sifr_driver/src/project/compile_order.rs:193](../crates/sifr_driver/src/project/compile_order.rs:193) → `SIFR-WORKSPACE-0104`.
- [crates/sifr_driver/src/project/frontend.rs:29](../crates/sifr_driver/src/project/frontend.rs:29) → `SIFR-WORKSPACE-*`.
- [crates/sifr_driver/src/stdlib/bootstrap.rs](../crates/sifr_driver/src/stdlib/bootstrap.rs) — 4 sites → `SIFR-STDLIB-0001..0003` / `SIFR-INTERNAL-0001`.
- [crates/sifr_driver/src/stdlib/cache.rs:55](../crates/sifr_driver/src/stdlib/cache.rs:55) → `SIFR-STDLIB-0004` or `SIFR-BUILD-*`.
- [crates/sifr_driver/src/workspace/mod.rs](../crates/sifr_driver/src/workspace/mod.rs) — 2 sites → `SIFR-WORKSPACE-0001..0004`.
- [crates/sifr_driver/src/test_runner/execution.rs](../crates/sifr_driver/src/test_runner/execution.rs) — 8 sites → `SIFR-BUILD-*`.
- [crates/sifr_driver/src/test_runner/orchestrator.rs](../crates/sifr_driver/src/test_runner/orchestrator.rs) — 2 sites; one forwards frontend diagnostics (must preserve identity), one is internal.
- [crates/sifr_driver/src/diagnostics.rs:255-267](../crates/sifr_driver/src/diagnostics.rs:255) — `run_codegen_with_boundary` → `SIFR-INTERNAL-0001` for panic, `SIFR-CODEGEN-0002` for non-panic codegen failures.

**Sub-PR 4 — `CompilePhase::TypeCheck` deletion + HIR/type-system mechanical transport:**
- [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs) — 205 sites (largest single file). Inventory categories: NAME/TYPE/CALL/STDLIB/PROTO/DECIMAL/FLOW.
- [crates/sifr_hir/src/lower/statements.rs](../crates/sifr_hir/src/lower/statements.rs) — 61 sites: FLOW/RESULT/PROTO/MATCH/TYPE/OWN/CALL.
- [crates/sifr_hir/src/lower/builtin_calls.rs](../crates/sifr_hir/src/lower/builtin_calls.rs) — 55 sites: CALL/TYPE/DECIMAL/STDLIB.
- [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs) — 24 sites: NAME/TYPE/RESULT/PROTO/CALL.
- [crates/sifr_hir/src/lower/mod.rs](../crates/sifr_hir/src/lower/mod.rs) — 20 sites: TYPE/IMPORT/NAME/STDLIB. Note the wrong-layer workspace import diagnostics that should move to driver layer (per inventory line 41-43).
- [crates/sifr_hir/src/lower/classes.rs](../crates/sifr_hir/src/lower/classes.rs) — 19 sites: CLASS/TYPE/NAME/PROTO.
- [crates/sifr_hir/src/lower/decimal_methods.rs](../crates/sifr_hir/src/lower/decimal_methods.rs) — 18 sites: DECIMAL/CALL/NAME (per R7, emit canonical `SIFR-DECIMAL-*` directly).
- [crates/sifr_hir/src/lower/aug_assign_lowering.rs](../crates/sifr_hir/src/lower/aug_assign_lowering.rs) — 17 sites: TYPE/OWN/NAME.
- [crates/sifr_hir/src/lower/bytes_methods.rs](../crates/sifr_hir/src/lower/bytes_methods.rs) — 16 sites: STDLIB/CALL/TYPE.
- [crates/sifr_hir/src/lower/method_call_args.rs](../crates/sifr_hir/src/lower/method_call_args.rs) — 13 sites: CALL/TYPE.
- [crates/sifr_hir/src/lower/tuple_unpack.rs](../crates/sifr_hir/src/lower/tuple_unpack.rs) — 13 sites: TYPE/FLOW.
- [crates/sifr_hir/src/lower/container_literal_specialization.rs](../crates/sifr_hir/src/lower/container_literal_specialization.rs) — 11 sites: TYPE.
- 11 smaller files with 1–3 sites each (see [internal_docs/diagnostic_emission_inventory.md:30-40](../internal_docs/diagnostic_emission_inventory.md:30)).
- `sifr_type_system::TypeError` forwarder bridging — short-lived adapter local to the HIR call sites that read `TypeErrorKind::*` payloads (not a `From` impl).
- [crates/sifr_driver/src/frontend/module_lowering.rs](../crates/sifr_driver/src/frontend/module_lowering.rs) — replace `LoweringError` -> `CompileError { phase: TypeCheck }` mapping with direct `SifrDiagnostic` forwarding through the sink. The `[main] ` module-prefix style must move to a structured `module` arg (see [tests/project_graph.rs:40-43](../crates/sifr_driver/src/tests/project_graph.rs:40)) or to a related-span on the diagnostic; do not embed module names in the rendered message.
- [crates/sifr/src/main.rs:471-479](../crates/sifr/src/main.rs:471) — `cmd_check` panic boundary → `SIFR-INTERNAL-0001`.
- 91 fail fixtures under `crates/sifr/tests/e2e/fail/*.sifr` — annotation re-keying.
- 2 verification baselines under `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/` — regenerate.
- Workspace verification baselines under `crates/sifr/tests/verification/project/*/baselines/` — regenerate against the new schema (byte ranges, span lines, `version: 1` envelope, `args` object, `message_template` field, `spans` array). Existing baselines do not have those fields.

## Likely test gaps

- **Schema/baseline gap:** existing JSON baselines ([decimal_invalid_literal/baselines/check-json.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt)) emit a flat array, not the versioned envelope `{ "version": 1, "diagnostics": [...] }`. They lack `message_template`, `args`, `spans`, and the new `RenderedDiagnostic` shape. All workspace and decimal verification baselines need full regeneration in `diag_4a`. Add a fixture-level test (the milestone explicitly requires this in `diag_5`, but it can be staged here) that proves JSON/compact/human consume the same `Vec<SifrDiagnostic>`.
- **No CLI test currently asserts that compact grouping uses `message_template`, not rendered `message`.** [presentation.rs:215-243](../crates/sifr_diagnostics/src/render/presentation.rs:215) covers it for the `sifr_diagnostics` crate-internal renderers, but there is no CLI-level integration test. Add one in sub-PR 1 that emits two diagnostics with the same template but different rendered text from two source files and asserts they appear under one compact group.
- **No driver-level test asserts deterministic ordering across hash-map iteration.** The milestone calls this out as a hard rule. The `sifr_diagnostics` test at [render/mod.rs:481-543](../crates/sifr_diagnostics/src/render/mod.rs:481) covers it within the diagnostics crate, but the driver/CLI also needs a test that two compilations of the same project on different machines produce byte-identical JSON output.
- **No test exercises the canonical-stream contract for `reveal_type(...)` and warnings.** Currently warnings/reveals go through stderr side-channels. Once they fold into the canonical stream, add a fixture that emits a `reveal_type(x)` plus a real error and asserts both appear in the same JSON envelope and are sorted correctly.
- **`ErrorEmitted` proof discipline is unenforced in HIR.** The milestone allows `LowerCtx::emit_error(...)` to discard the proof for now ([issues/…-diagnostics.md:443](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:443)), but the proof is needed for tainted HIR values in `diag_10`. Sub-PR 2's `LowerCtx::emit_error(...)` signature should already return `ErrorEmitted` so `diag_10` doesn't have to widen the API later.
- **Workspace-classifier deletion needs negative tests.** Add a test that asserts `CompileError` (or its replacement) does *not* attempt message-prefix code derivation, e.g., a workspace error whose message coincidentally starts with `"could not resolve import "` but is constructed via the canonical `SIFR-WORKSPACE-0101` constant — assert that the code is `0101` *because* of the constructor, not the message.
- **Snapshot rewrite plan:** the four `test_compact_renderer_*` tests in [main.rs:1268-1446](../crates/sifr/src/main.rs:1268) and the two `crates/sifr_driver/src/tests/diagnostics.rs` tests assert the legacy "x5 + ... +N more" shape. They should be deleted and replaced with snapshot tests over the new compact shape (`(severity, code, message_template, file)` grouping, locations aggregation, no synthesized text-bearing diagnostics).
- **`scripts/check_diagnostic_cancel_usage.py` is referenced by the issue's validation plan but does not exist** (verified by `ls scripts/`). It is required to land in `diag_1` per [issues/…-diagnostics.md:760](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:760), but does not appear in the scripts directory. Either it was missed in `diag_1` (a separate gap to flag) or it's hidden under a different name; confirm before claiming `diag_4a` validation. Same comment for `scripts/check_diagnostic_code_coverage.py` and `scripts/check_diagnostic_baseline_hygiene.py` — both referenced by the validation plan, both absent. (These are technically prerequisite gaps from earlier milestones, but they will block the `diag_4a` `Validation Plan` checklist if the implementer runs the full list.)

## Sequencing recommendations

The issue lists four sub-PRs; the order matters. Recommended sequencing:

1. **Sub-PR 1: Renderer integration + classifier+grouping deletion + session/source-map owner.** Deletes `apply_diagnostic_recovery_limits`, `workspace_diagnostic_code`, `Severity::Help`, the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` arm. Builds the `DriverSession` (or equivalent) that owns `SourceMap` + `DiagnosticSink`. CLI dispatches via `render_sink_human/compact/json`. **Critical:** this PR must keep the legacy `CompileError` -> `SifrDiagnostic` bridge alive for the workspace/parser/test-runner sites that haven't yet been migrated — otherwise it breaks every workspace verification baseline. The bridge maps `CompilePhase::Parse` to a temporary parser code, `CompilePhase::Build` to `SIFR-BUILD-0002` (no longer to `SIFR-BUILD-0001` since that's retired), and `CompilePhase::Codegen` to `SIFR-CODEGEN-0002`. Workspace codes still derive from the construction-site only after sub-PR 3. Document this transitional bridge as scoped to die in sub-PR 4.
2. **Sub-PR 3 (out of issue order): Parser/workspace/codegen/build/test-runner transport.** Lands before sub-PR 2/4 because it eliminates the workspace `CompilePhase::Build` constructions that the prefix classifier covered. After this PR, the only `CompileError` constructions left are HIR-frontend ones (`CompilePhase::TypeCheck`) and the panic boundaries (`Codegen`/`Build`).
3. **Sub-PR 2: `LoweringError` → `LoweringOutcome`/`DiagnosticSink`.** Adds `SourceId` plumbing into `LowerCtx`. Replaces `Vec<LoweringError>` returns with `LoweringOutcome`. Folds `reveal_types`/`warnings` into the diagnostic stream (R9). Updates 7 HIR unit-test files. Does *not* yet rewrite the 510 `ctx.error` call sites — they keep emitting `LoweringError` internally, which the new outer adapter converts to `SifrDiagnostic` with a temporary catch-all. This is a brief stepping stone — it cannot ship as the final state because the catch-all would emit `SIFR-TYPE-0001` (retired). The PR therefore must also assign at least one inventory code per file (e.g., a temporary `LowerCtx::error_with_code(code, span, msg)` helper) and cannot reasonably split from sub-PR 4.
4. **Sub-PR 4: HIR/type-system mechanical transport + 91 fixture re-keying + verification baseline regeneration.** The biggest PR. This is where the inventory's per-call-site mapping is realized, decimal pseudo-codes get canonical `SIFR-DECIMAL-*` codes, `SIFR-TYPE-0001` stops appearing in the wild, and the e2e suite stays green.

**Alternative (simpler but riskier) sequencing:** merge sub-PR 2 and sub-PR 4 into one. The HIR call sites cannot really be split from `LoweringError` removal because both must change atomically — the moment `LoweringError` is gone, every `ctx.error(String)` must already have become `ctx.emit_error(SifrDiagnostic)`. The issue's separation of (2) and (4) is artificial. Recommend explicitly merging them in the implementation plan.

**Hard-coded ordering constraints:**

- Sub-PR 1 must precede sub-PR 3 (renderer + session must exist before transport sites can call `session.sink.emit_error(...)`).
- Sub-PR 3 must precede the `workspace_diagnostic_code` deletion *or* the deletion must happen in sub-PR 1 alongside in-place workspace migration. Pick one.
- Sub-PR 4 must include 91 fixture annotation updates *and* regeneration of the decimal verification baseline. Splitting into "code first, fixtures later" leaves CI red between PRs.
- Verify `scripts/check_diagnostic_cancel_usage.py`, `scripts/check_diagnostic_code_coverage.py`, `scripts/check_diagnostic_baseline_hygiene.py` exist before this milestone closes; if missing, add them in sub-PR 1 since `diag_4a`'s validation plan depends on them.

## Decisions to make before the first commit

These are explicit choices the implementer should record in the PR description (or `internal_docs/`) so reviewers don't relitigate them mid-PR:

1. **Span placeholder convention for HIR transport (R1):** confirm "every HIR source diagnostic carries `SourceSpan(module_source_id, ast_range_or TextRange::default())` in `diag_4a`; `diag_9` widens to AST-accurate ranges". State this explicitly.
2. **Workspace classifier removal timing (R2):** confirm sub-PR 1 deletes the classifier *and* keeps a transitional `CompilePhase::Build => SIFR-BUILD-0002` bridge (not `0001`) until sub-PR 3 completes the workspace site migration.
3. **`apply_diagnostic_recovery_limits` fate (R3):** confirm it is deleted in sub-PR 1, not preserved as a parallel grouping path. Confirm the "x5 + ... +N more" snapshot tests are deleted, not migrated.
4. **`Severity::Help` removal (R4):** confirm sub-PR 1 deletes the legacy variant and updates all snapshots.
5. **`CompilePhase::TypeCheck` site reassignments (R5):** record the per-site code for each of the 5 non-HIR `TypeCheck` sites.
6. **`TypeError` adapter scope (R6):** confirm the adapter lives at HIR call sites, not as `impl From` on `TypeError`, and is deleted in `diag_7`.
7. **Decimal codes in `diag_4a` vs `diag_6` (R7):** confirm `SIFR-DECIMAL-*` is emitted directly in `diag_4a`'s mechanical transport; `diag_6` only strips the `[E25xx]` prefix from message templates and updates fixtures.
8. **Fixture annotation re-keying (R8):** confirm sub-PR 4 includes 91 fixture updates plus 2 decimal verification baseline regenerations plus 5 workspace verification baseline format-only regenerations.
9. **`reveal_types`/`warnings` migration (R9):** decide whether to fold into the canonical stream in sub-PR 2 or defer to `diag_10`.
10. **Sub-PR 2/4 merge (Sequencing):** decide whether to ship `LoweringError` removal and HIR transport in one PR or two. The issue lists two; the dependency graph argues for one.

## Open questions for the implementer

- The issue's validation plan references `scripts/check_diagnostic_cancel_usage.py` and `scripts/check_diagnostic_code_coverage.py` and `scripts/check_diagnostic_baseline_hygiene.py`. These do not exist in `scripts/` today. Were they punted from `diag_1`? If so, `diag_4a` should add at least the cancel-usage and code-coverage checks since this is the milestone that introduces real `DiagnosticBuilder` usage and the first family of active emission sites.
- The `[main] ` module-prefix style at [tests/project_graph.rs:40-43](../crates/sifr_driver/src/tests/project_graph.rs:40) and [frontend/module_lowering.rs:31-33](../crates/sifr_driver/src/frontend/module_lowering.rs:31) is currently a string concatenation. The milestone implies module identity should be a structured field, not message text. Confirm whether `module_name` is added as a JSON-only declared arg to all family codes or attached via `RelatedSpan`/source-map module metadata. The cleaner shape is "diagnostic spans carry source IDs whose source-map metadata includes the module name; renderers may prefix the human label with `[module]` based on source-map lookup, never on string concatenation".
- Confirm whether the ordering policy's `severity_rank` matches the canonical `Severity` enum. [model/mod.rs:9-16](../crates/sifr_diagnostics/src/model/mod.rs:9) defines `Error | Warning | Note` with `derive(PartialOrd, Ord)`, which gives the same total order as the `severity_rank` function at [render/mod.rs:220-226](../crates/sifr_diagnostics/src/render/mod.rs:220). Either remove the redundant function or document why it exists (it does protect against future enum reordering).

## Validation expectations

For `milestone_diag_4a` to satisfy the milestone's definition of done, local validation must include at minimum:

- `cargo test -p sifr_diagnostics` (existing).
- `cargo test -p sifr_hir` (existing — must still pass after `LoweringOutcome` migration).
- `cargo test -p sifr_driver` (existing — must still pass after CLI/driver renderer migration).
- `cargo test -p sifr -- test_e2e_fail` (the 91-fixture suite, including all re-keyed annotations).
- `scripts/run_e2e_pass.sh` (existing).
- `scripts/run_all_tests.sh --profile quick`.
- `python3 scripts/check_diagnostic_docs_sync.py`.
- `python3 scripts/check_diagnostic_schema_sync.py`.
- `cargo run -p sifr_diagnostics --bin gen-error-docs -- --check`.
- `cargo run -q -p sifr -- --diagnostic-format json check crates/sifr/tests/e2e/fail/type_mismatch.sifr` and the equivalent compact/human invocations as smoke tests.

The full validation set in [issues/…-diagnostics.md:1140-1170](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1140) names additional scripts that don't exist yet — see open questions above.

## Out of scope (will surface as `diag_4b`/later milestones)

- `CompilePhase` and `CompileError` deletion are formally `diag_4b`, not `diag_4a`. `CompilePhase::Display` remains; the public `CompileError` abstraction remains. Only the public *code mapping* dies.
- Per-fixture span coverage is `diag_9`. `diag_4a` only requires *some* `SourceSpan` per source diagnostic, not a precise AST range.
- `TypeError`/`TypeErrorKind` deletion is `diag_7`.
- Recovery cap (50 top-level) and `SIFR-INTERNAL-0002` cap-omission summaries activate in `diag_10`. The admission pass added in `diag_4a` is a no-op pass that exists so `diag_10` has a clear hook.
- `reveal_type(...)` and warnings *may* fold into the canonical stream in `diag_4a` (R9) or wait for `diag_10`. Either is defensible; record the choice.
- Test harness contract cleanup (rejecting `[Edddd]` and message-substring expectations) is `diag_5`. `diag_4a` keeps the harness backwards-compatible.

## Summary

The shared diagnostic infrastructure is fully built; `diag_4a` is the integration cliff. The four sub-PR structure in the issue is approximately right but should be re-ordered (1, 3, then merged 2+4) and each sub-PR should record its sequencing decisions explicitly. The largest hidden cost is span plumbing through `LowerCtx` — the implementation cannot avoid it because the canonical `SourceDiagnostic` requires a `SourceSpan`. The largest fixture cost is the 91 annotation re-keys, which are mechanical given the inventory but must be co-merged with the HIR emission change to avoid red CI between PRs. Confirm the existence (or schedule the addition) of `scripts/check_diagnostic_cancel_usage.py`, `scripts/check_diagnostic_code_coverage.py`, and `scripts/check_diagnostic_baseline_hygiene.py` before the implementation claims `diag_4a` validation.
