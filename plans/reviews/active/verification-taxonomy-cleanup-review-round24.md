## Review: Verification/Codebase Terminology Hygiene Cleanup

### Concern 1 — Delivery-taxonomy leakage outside `plans/`

**Clean.** I ran direct greps for every stale slug and term across `verification/`, `.cursor/`, `.github/workflows/`, `crates/`, `demos/`, `docs/`, `internal_docs/`, `lib/`, `scripts/`, and root-level files.

- Stale command slugs (`/add-ticket`, `/add-work-item`, `/work-on-ticket`, `/work-on-item`, `/create-prds`): no hits outside `plans/` and `verification_taxonomy.py` (the guard intentionally references the strings inside string-concatenated patterns).
- Stale skill name `phase-closure-loop`: zero hits anywhere in active surfaces.
- Stale renamed test paths (`build_output_contracts`, `harness_contract_tests`, `validation_contracts`, `validation_contract_support`, `network_http_dependency_contract_tests`, `query_diagnostics_contract_tests`, `task_runtime_contract_tests`, `diagnostic_contract_harness`, `prd-solution-design-template`): zero hits.
- "phase/milestone/wave/roadmap" mentions in active markdown: only legitimate compiler/runtime hits (`docs/cli_command_semantics.md:86` "phase labels" for build output, `internal_docs/typescript_go_architecture_transfer_guardrails.md:193` "service phase traces", `internal_docs/typescript_go_architecture_transfer_trace_status.md:20` "trace phase vocabulary", `internal_docs/tooling_analysis.md:183` "phase execution state"). All four are compiler-pipeline language, not delivery taxonomy.
- `contract_xxx` snake_case identifiers in active surfaces: every hit is either on the explicit allowlist (`contract_suites`, `contract_errors`, `contract_id`, `contract_version`, `contract_check`, `contract_path`) or only occurs inside `verification_taxonomy.py` itself (which is excluded from its own scan at line 226).
- Narrative "contract" usage in docs (`self-update contract`, `package-management contract`, `planner contract`, `language contract`, `runtime contract`) is correctly preserved — these are language/runtime semantic terms, not delivery aliases.

### Concern 2 — Taxonomy guard scope

The guard is well-calibrated:

- `ACTIVE_ROOTS` (lines 16–29) correctly excludes `plans/`, `tmp/`, `target/`, `third_party/`, `.git/` while covering every authoritative surface.
- The `ALLOW_TEXT_PATTERNS` allowlist (lines 48–59) preserves legitimate compiler identifiers (`WorkspaceTracePhase`, `SingleOwnerCompilerPhase`, `LintPhase`, `PhaseExecution`, `ProgressPhase`, `phase_plan`, `record_compiler_phase_trace`, `compiler phase`, `build phase`, `lint phase`) and narrative `phase=` usage.
- Self-test covers 47+ failure cases and one positive case, so any new pattern addition has a guard against accidental regression.
- `validate_text` correctly skips the guard file itself (line 226).

**Non-blocking observation:** The pattern at line 84 `re.compile(r"\`[a-z_][a-z0-9_/-]* [^\`]*\.md\`", re.IGNORECASE)` would reject any backticked filename reference containing a space (e.g., `` `concurrency_runtime_structured-task capability_traceability.md` ``). This is consistent with the cleanup goal and is exercised by the self-test, but is the broadest pattern — could over-fire on a legitimate "see `path/to file.md`" reference if such a path is ever introduced. None exist today.

### Concern 3 — Task command rename consistency

**Clean.** Verified `.cursor/commands/` contains exactly the seven expected files: `add-task.md`, `create-design-doc.md`, `create-new-version.md`, `create-task.md`, `refinement.md`, `review-pr.md`, `work-on-task.md`.

`project-workflow/SKILL.md` references `/create-design-doc`, `/create-task`, `/add-task`, `/refinement`, `/work-on-task`, `/review-pr` — every one of these has a matching command file. `add-task.md`, `work-on-task.md`, `refinement.md`, `review-pr.md` were all internally re-pointed to the new task vocabulary (e.g. `add-task.md` now uses "Task" and references `task-template.md` / `design-document-template.md`).

**Minor discrepancy with the prompt's framing, not a code issue:** the prompt described the rename as `add-work-item.md → add-task.md` and `work-on-item.md → work-on-task.md`, but `git log` shows the actual rename in this branch is `add-ticket.md → add-task.md` and `work-on-ticket.md → work-on-task.md`. The end state is correct either way; the prompt narrative just misnamed the source filenames.

### Concern 4 — Formatter rules invariant correctness

**Clean.** I cross-validated the three manifests against `formatter_rules.md`:

- `capability_matrix.json`: 35 rows, every `capability` and `implementation_requirement` string appears verbatim in the rules doc.
- `cli_parity.json`: 22 rows, every `ruff_surface` and `required_fixture` appears verbatim.
- `ast_coverage.json`: 11 rows matching the `REQUIRED_AST_ROWS` set exactly; every `id`, `syntax`, and `required_formatter_behavior` appears verbatim.

`check_formatter_rules_manifests.py` enforces row counts (35/22/11) and the doc-presence invariant on every row field (lines 96-104, 126-134, 156-168). The Ruff FormatCommand markers (lines 33-50) cross-check against the vendored Ruff `args.rs` so CLI-parity rows can't silently drift from the upstream binding. `check_baseline` validates the Ruff submodule pin, branch, commit subject, and required seed paths. The self-test (lines 224-252) verifies that an invalid classification is rejected.

The reference-level invariant is genuinely closed: any manifest edit that doesn't simultaneously update `formatter_rules.md` will fail the check.

### Concern 5 — Non-blocking nit

`.github/workflows/preview-release.yml:50` and `:174` were updated to "until the a stable channel is supported" — the article is doubled ("the a"). Grammar nit only, not a hygiene issue. Worth fixing in passing but not blocking.

### Verdict

**SATISFIED.**

The cleanup is internally consistent, the taxonomy guard is comprehensive without over-reach, the cursor command renames resolve cleanly across the workflow skill, and the formatter-rules manifest checker now enforces a true reference invariant (manifests → rules doc → upstream Ruff binding). The only finding is the cosmetic "the a stable channel" typo in the preview-release workflow, which I'm flagging as non-blocking.
