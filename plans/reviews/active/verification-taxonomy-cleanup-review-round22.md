# Review: Verification taxonomy / codebase hygiene cleanup

**SATISFIED** — the cleanup achieves its stated goal. Active verification surfaces, profile routing, crate identifiers, and renamed fixtures are internally consistent; the taxonomy gate passes; no stale references survive outside `plans/`. Findings below are notes for follow-up consideration, none block this round.

---

## Findings (severity ordered)

### Medium — formatter manifest checker lost an invariant

`verification/areas/developer_tooling/check_formatter_rules_manifests.py` is the replacement for the deleted `check_formatter_contract_manifests.py`. The old check cross-referenced every capability row, CLI surface, and required AST phrase against `plans/issues/.../ad-hoc-production-grade-sifr-formatter*.md` so the manifests could not silently drift from the reviewed formatter design. The new version drops that entire cross-check and keeps only:

- shape/uniqueness of capability rows
- shape of CLI rows + Ruff `args.rs` marker presence
- AST row IDs (set equality only) and field presence
- Ruff submodule baseline/commit/branch

The hardcoded row counts (`35` capabilities, `22` CLI rows) still anchor against accidental removal, but content drift is no longer detected anywhere. Dropping the dependency on transient `plans/` docs was the right move, but the invariant ("manifest content matches the reviewed source of truth") was not relocated. Consider promoting the relevant capability/CLI/AST sentences into a small in-repo `formatter_rules.md` under `verification/areas/developer_tooling/` and re-asserting against that, so the source of truth lives with the verification it backs. Not blocking.

### Low — residual "contract" wording in active surfaces (same code smell category)

The taxonomy regex deliberately allows free-text "contract", schema field names, and identifiers like `contract_id`/`contract_version`/`contract_check`/`contract_path`. Given the user's note that "wave → contract is the same code smell," several spots may still warrant a sweep:

- **Area manifest descriptions** (read by anyone listing areas):
  - `verification/areas/developer_tooling/manifest.json:5` — "Developer tooling **contracts** for LSP, formatter, linter…"
  - `verification/areas/core_language/manifest.json:5` — "Core compiler language semantics, **contract matrices**, and e2e language behavior."
  - `verification/areas/diagnostics/manifest.json:5` — "…presentation baseline **contracts**."
  - `verification/areas/project_workspace/manifest.json:5` — "multi-module baseline **contracts**."
- **Case IDs** in the new `validation_suites` manifests:
  - `verification/areas/project_workspace/data/validation_suites/manifest.json:315` — `reachable_parse_error_contract` (and its `closure_neg_check`/`closure_neg_build` command IDs reuse the older "closure" smell)
  - `verification/areas/core_language/data/validation_suites/manifest.json:460` — `matrix_backed_parser_and_lexer_contracts`
- **Internal identifiers** that match other renamed siblings:
  - `verification/areas/performance/runner.py:129,161` — `run_contract_variants()` (escapes the regex because of the `run_` prefix breaking the word boundary; sibling case IDs and the suite were renamed to `rules`)
  - `verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py:527` — local variable `diagnostic_contract = DIAGNOSTIC_CANONICALIZATION.read_text(...)` reads the renamed `check_diagnostic_source_canonicalization_rules.py`
- **Free text** in active rule docs:
  - `verification/areas/core_language/data/integer_model/readiness_hardening.md:18` — "performance benchmark **contract** is not active yet … long-term target remains within 2x … once the performance benchmark **contract** owns…"
  - `verification/areas/core_language/data/integer_model/serialization_boundary_rules.md` — "INT-5 **contract** lock", "Schema **contract**", "Required Sifr **contract**"
  - `verification/areas/core_language/data/integer_dtype_rules.md:42` — "Required default **contract**"
  - `verification/areas/stdlib_parity/tools/generate_concurrency_runtime_inventory.py:583` — return literal `"contract evidence"` (synthesized into reports)
- **Schema field uniform across 16 area manifests**: `baseline_metadata_contract` block. Renaming would be a coordinated schema change but is a clean candidate (e.g. `baseline_metadata_policy`).
- **Inconsistency inside one file**: `verification/areas/developer_tooling/lsp_protocol_matrix.json:86` describes formatting edits as matching `sifr fmt`; line 87 (rangeFormatting) still says "match formatter **contract**". Pure cosmetic asymmetry.

None of these break verification today and all pass the taxonomy gate as written. They are listed so a future pass can decide whether the cleanup intends to reach them.

### Low — `performance:rules` suite is defined but unrouted

`verification/areas/performance/manifest.json` defines a `rules` suite (replacing the prior `contracts` suite). No profile (`create-pr`, `merge`, `nightly`, `release`) selects `performance:rules`, and `profile_assignment_matrix.json` only references `performance:{smoke,representative,full}`. The runner's `run_contract_variants()` is invoked unconditionally inside `smoke`/`representative`/`full`, so the suite's intended cases (`benchmark-manifest`, `budget-policy`, `trend-policy`, …) still execute via those profiles. The standalone `rules` suite is thus reachable only by direct `--area performance --suite rules` invocation. Not a regression — just verify whether the suite was meant to stay exposed for ad-hoc use or be removed.

---

## Direct answers to the four questions

1. **Process-taxonomy leaks outside `plans/`?** None that the taxonomy gate would catch. `phase`/`wave`/`milestone`/`world-class`/`closeout`/`ad-hoc`/`work-item` are gone from active surfaces (and from crate sources). The only `phase` usages that remain — `compiler phase`, `record_compiler_phase_trace`, `WorkspaceTracePhase`, `LintPhase`, etc. — are explicit allowlist entries for legitimate compiler-pipeline terminology.
2. **Broken suite routing, stale command strings, stale fixture references?** No. Every renamed fixture (`presentation_rules_cases`, `frontend_internal_diagnostic_rules_cases`, `package_library_diagnostic_rules_cases`) has its baselines and metadata updated. `diagnostic_rendering_harness` is consistently referenced from `cargo_metadata_classification.json` and `check_diagnostic_source_canonicalization_rules.py`. `validation_suites` env vars (`SIFR_VALIDATION_SUITE_MANIFEST`/`_FILTER`) line up between `area_adapter.py` and `validation_suite_support/manifest.rs`. `harness_behavior_tests` is wired into `e2e_support/mod.rs:21`. No `*_contract_*` paths remain in active code.
3. **Formatter manifest checker replacement acceptable?** Acceptable for unblocking, but it dropped a real invariant (manifest ↔ reviewed-doc cross-check). See Medium finding for a low-cost relocation suggestion.
4. **Crate-level delivery-flavored leaks?** None remaining in `crates/sifr_*` Rust sources. The only `phase` hits there (`workspace_trace.rs`, `workspace_session_tests.rs`) are compiler-phase tracing terminology and are allowlisted. `contract` is absent from the touched crate sources entirely.
