# Ad Hoc Repository Architecture And Verification Surface Cleanup

status: draft

## Objective

Make the Sifr repository present as a serious, production-grade compiler codebase by giving every major repo surface a clear ownership contract: compiler source, public docs, internal architecture, planning, verification, workflow automation, release tooling, and external corpora.

This is an architectural cleanup phase, not a cosmetic sweep. The end state must be clean, explicit, and uncompromising: no fallback paths, no compatibility wrappers for old script names, no half-migrated folder layouts, no tracked process exhaust, and no machine-specific workflow assumptions.

## Problem

Sifr has strong compiler infrastructure, but the repository surface has accumulated historical artifacts from rapid milestone execution:

- planning docs, closed issues, and phase ledgers are not separated by lifecycle
- review transcripts and process history have been treated as active repo material
- Cursor workflows contain local-machine assumptions and model-specific command paths
- verification logic is split across `scripts/`, `verification/`, `audits/`, `crates/sifr/tests/verification/`, and internal docs
- some active script names encode stale phase numbers instead of stable compiler concerns
- audit fixtures and audit reports share one namespace even though they have different lifecycles
- verification inputs, inventories, reports, and policy docs are not consistently separated
- top-level repo intent is less obvious than the underlying compiler quality

The desired result is a repo where a new contributor can quickly answer:

1. Where is the compiler?
2. How do I validate it?
3. Which docs are current?
4. Which plans are active?
5. Which corpora are executable verification assets?
6. Which files are generated or historical and therefore not active engineering surface?

## Non-Goals

- No compiler feature work.
- No test coverage reduction.
- No CI-only validation behavior.
- No compatibility shims for moved script paths.
- No duplicate old and new verification layouts.
- No archive-in-place that leaves the active tree visually noisy.
- No tracking of point-in-time validation logs.
- No broad rewrite of compiler crates unless required by path or fixture ownership.

## Core Principles

- Active tree shows the product, not the process history.
- Humans read short, current docs and issue plans.
- Machines consume schemas, manifests, fixtures, and baselines.
- Generated reports live under `target/` and are not committed.
- Each fixture has one owning verification area.
- Lanes select verification areas; lanes do not own fixtures.
- Public validation has one entrypoint.
- Repo hygiene is enforced by guardrails, not convention.
- Historical material is either deleted, moved to git history, or placed in a separate archive outside the active tree.

## Target Top-Level Contract

The top-level tree should contain only intentional, load-bearing entries:

```text
.github/
.cursor/
crates/
demos/
docs/
editor_integrations/
internal_docs/
lib/
plans/
scripts/
third_party/
verification/
AGENTS.md
CLAUDE.md
Cargo.lock
Cargo.toml
LICENSE.md
README.md
logo.webp
sifr.toml
```

Every tracked top-level entry must be classified in the first implementation PR as one of:

- `keep`: active, intentional top-level surface
- `move`: active, but belongs under another owner
- `archive-external`: valuable process history, not active tree material
- `delete`: obsolete or generated
- `generate`: produced by a script and should not be committed unless it is a baseline

## Planning, Issues, And Reviews Structure

Target planning shape:

```text
plans/
  README.md
  roadmap.md

  phases/
    roadmap.md
    01_language_foundations.md
    02_type_system_power.md
    ...
    43_interoperability.md

  issues/
    active/
    completed/
    archive/

  reviews/
    active/
    archive/

internal_docs/
  architecture.md
  architecture/
  decisions/
```

Rules:

- `plans/roadmap.md` is the high-level execution roadmap.
- `plans/phases/roadmap.md` indexes phase status.
- Phase files stay flat under `plans/phases/` so phase numbers remain stable and easy to find.
- Phase status lives in each phase header and in `plans/phases/roadmap.md`; phases do not need active/completed/archive directories.
- Active ad hoc issue plans live in `plans/issues/active/`.
- Completed ad hoc issue plans move to `plans/issues/completed/`.
- Superseded, abandoned, or purely historical issue plans move to `plans/issues/archive/`.
- Active review artifacts live in `plans/reviews/active/`.
- Completed or historical review artifacts move to `plans/reviews/archive/`.
- Review archives are allowed under `plans/reviews/archive/`; they should not sit at the repository root.
- Issue files may include concise review summaries and PR links. Long transcript references should point into `plans/reviews/` only when the transcript is intentionally retained.
- Point-in-time validation logs are never committed.
- `internal_docs/` is reserved for durable architecture, accepted design decisions, and current technical references. It should not own execution planning.

## Review Artifact Policy

The main repository should not contain `reviews/` as a top-level tracked tree. Review artifacts belong under `plans/reviews/`.

Allowed review references in active docs:

- reviewer name or model family when relevant
- pass/fail summary
- concise actionable findings
- PR link
- final disposition

Disallowed active-tree material:

- per-pass review logs
- `.claude.log` files
- stale transcript path lists
- validation evidence copied as long prose when a generated report exists

Long review transcripts may be retained under `plans/reviews/archive/` when they have ongoing planning value. Otherwise they should be deleted from the active tree and left to git history.

## Cursor Cleanup

Target `.cursor/` shape:

```text
.cursor/
  commands/
  references/
  skills/
```

Rules:

- No personal absolute paths.
- No local-machine paths such as `/Users/yaseralnajjar/...`.
- No embedded Obsidian state.
- No tracked `.DS_Store` or local editor files.
- Commands target the new `plans/issues/active/`, `plans/phases/`, and `plans/reviews/active/` layout.
- Review skills write to `plans/reviews/active/` by default.
- Keep `.cursor/skills/talk-to-claude-opus/` as the single Claude/Fable review workflow.
- Remove the other Claude review skill variants once their useful instructions are folded into `talk-to-claude-opus`.
- Skill names should describe workflow intent, not a transient model brand, unless the model is itself the workflow contract.
- Remove `.cursor/.rules/`.

Add a Cursor hygiene guardrail that checks:

- forbidden personal paths
- tracked local editor state
- tracked review-output paths
- stale issue or phase path references
- workflow commands that write generated artifacts into tracked locations

## Verification Architecture

Target verification shape:

```text
verification/
  README.md
  policy/                 # runner/data policy, not human conventions
  schemas/
    lane.schema.json
    area.schema.json
    suite.schema.json
    case.schema.json
    result.schema.json
  lanes/
    create-pr.json
    merge.json
    nightly.json
    release.json
  runner/
    sifr_verify/
      __main__.py
      lanes.py
      areas.py
      scheduler.py
      results.py
      schemas.py
  areas/
    core_language/
      manifest.json
      suites/
      fixtures/
      baselines/
      data/
      runner.py
    diagnostics/
    project_workspace/
    regression/
    fuzz_property/
    determinism/
    generated_code_quality/
    performance/
    developer_tooling/
    runtime_platform/
    stdlib_parity/
    algorithmic_compatibility/
    ecosystem_compatibility/
    package_management/
    distribution_release/
```

Rules:

- `verification/README.md` is the single human entrypoint for validation architecture.
- The verification runner is stdlib-only Python.
- No `pyproject.toml`, `uv`, or external Python package dependency is required for local validation.
- The runner discovers areas by `verification/areas/*/manifest.json`.
- The top-level verification concepts are only `runner`, `schemas`, `lanes`, `areas`, and `policy`.
- `areas` answer what is verified and who owns it.
- `lanes` answer when verification runs and with what resource budget.
- `schemas` answer what shape committed verification data must have.
- `runner` answers how verification is executed.
- `policy` answers machine-facing operational rules for baselines, artifacts, flakes, retention, and schema governance.
- Suites are area-local groupings, not a competing top-level taxonomy. A suite may live under `verification/areas/<area>/suites/` or inside the area manifest.
- Cases are area-owned. A lane may select a subset of an area or suite, but it never owns fixtures directly.
- Each area owns its fixtures, baselines, runner code, manifests, and area-local data.
- Each area manifest declares owner, suites, fixtures, baselines, resources, parallel safety, expected outputs, timeout policy, and result contract.
- Lane files select areas or area-local suites and define execution policy; they do not duplicate fixture lists.
- Generated reports go to `target/verification_reports/`.
- Markdown under `verification/` is limited to `README.md`, policy, and runbooks.
- Gate inputs are JSON, schemas, baselines, or fixtures.
- Parallelism is explicit per area.
- Performance-sensitive or shared-state areas must declare `parallel_safe: false`.
- Sequential/parallel equivalence is a runner self-check, not an ad hoc shell script.
- Determinism evidence is generated and schema-validated.

Area-local shape:

```text
verification/areas/<area>/
  manifest.json           # owner, suites, resources, lane selectors, result contract
  README.md               # short area runbook only when needed
  runner.py               # optional area adapter called by the stdlib-only runner
  suites/                 # optional suite manifests for large areas
  fixtures/               # first-party source inputs and minimized cases
  corpora/                # external corpora and submodules owned by this area
  baselines/              # checked-in expected outputs
  data/                   # machine-readable inventories consumed by this area
```

Lane-local shape:

```text
verification/lanes/<lane>.json
```

Lane files may define:

- selected areas or area-local suites
- warm and cold time budgets
- resource policy
- shard policy
- retry and flake policy
- parallelism limits
- generated report requirements

Lane files may not define:

- fixture paths directly
- expected diagnostic text
- baselines
- domain ownership
- one-off shell commands

Permanent verification areas:

| Area | Owns |
| --- | --- |
| `core_language` | Broad compiler behavior fixtures for syntax, lowering, type checking, ownership, control flow, and pass/fail language behavior that is not more specifically owned elsewhere. |
| `diagnostics` | Diagnostic codes, renderer behavior, JSON/compact/human diagnostic contracts, docs/schema sync fixtures, and diagnostic presentation baselines. |
| `project_workspace` | Multi-file project behavior, imports, module graph, workspace resolution, graph isolation, cache identity, and project-mode command semantics. |
| `regression` | Fixed bugs, minimized crash reproducers, invariant failures, and promoted sentinel cases. |
| `fuzz_property` | Deterministic property tests, fuzz smoke seeds, sustained fuzz corpus metadata, minimization conventions, and seed provenance. |
| `determinism` | Deterministic report signatures, sequential/parallel equivalence, sharding behavior, flake/quarantine data, and runner determinism self-checks. |
| `generated_code_quality` | Emitted Rust quality, rustfmt/clippy/panic scans, generated-code determinism, generated binary-size checks, and codegen quality corpora. |
| `performance` | Benchmark manifests, budgets, waivers, baselines, performance fixtures, and budget-check runners. |
| `developer_tooling` | Formatter, linter, LSP, editor assets, editor query snapshots, completion quality, and tooling split-brain checks. |
| `runtime_platform` | Host/platform contracts, platform golden fixtures, process/runtime/IO/network host behavior that is platform substrate rather than stdlib parity. |
| `stdlib_parity` | CPython-adapted stdlib behavior, stdlib namespace contracts, complexity/resource parity, dependency snapshots, and stdlib parity evidence. |
| `algorithmic_compatibility` | Algorithmic compatibility corpora such as LeetCode, algorithmic scorecards, and external algorithm fixture projections. |
| `ecosystem_compatibility` | Curated OSS and broader ecosystem compatibility manifests, pinned external project outcomes, and non-blocking ecosystem signal. |
| `package_management` | Package graph, package demo repositories, package CLI alignment matrices, workspace/package fixtures, and package publishing/vendoring validation. |
| `distribution_release` | Installer, self-update, release-channel, artifact checksum, generated dispatcher, and preview/stable release validation. |

There is no permanent `audits` area. Existing audit material must move into the owning area above or be deleted/archived according to the audit cleanup rules.

## Scripts Cleanup

`scripts/` becomes repo operations and developer ergonomics only. It must not own verification implementation.

Keep in `scripts/`:

- `run_all_tests.sh` as the only public validation facade
- source and repository guardrails
- code generators
- release and publishing tools
- repository maintenance utilities

Move into `verification/areas/*`:

- e2e pass runner -> `core_language`
- validation contract matrices -> owning area, usually `core_language`, `project_workspace`, or `diagnostics`
- hardening runner -> `regression`, `fuzz_property`, and `ecosystem_compatibility` as appropriate
- platform golden checks -> `runtime_platform`
- generated-code quality checks -> `generated_code_quality`
- performance budget checks -> `performance`
- tooling and LSP verification checks -> `developer_tooling`
- distribution validation cases -> `distribution_release`
- stdlib namespace and corpus validation -> `stdlib_parity`
- fuzz and property checks -> `fuzz_property`
- determinism checks -> `determinism`
- sequential/parallel equivalence checks -> `determinism`

Rules:

- No active script filename may encode stale phase numbers.
- No duplicate entrypoints for the same gate.
- No compatibility wrapper remains for an old script name after migration.
- `AGENTS.md`, CI workflows, README, internal docs, and Cursor commands update atomically with script moves.
- Shell is allowed only for tiny public facades or release scripts where POSIX shell is the artifact under test.
- Verification scripts moved out of `scripts/` should be rewritten as stdlib-only Python area runners unless the checked artifact is itself POSIX shell.
- Shell-to-Python rewrites must preserve command semantics, exit-code behavior, timeout behavior, output normalization, and generated report shape.
- Rewrites must land with side-by-side equivalence evidence before the shell implementation is deleted.

Script migration classification:

- `repo-guardrail`: stays in `scripts/`
- `code-generator`: stays in `scripts/`
- `release-tool`: stays in `scripts/` unless it is only a validation case
- `verification-runner`: move to `verification/areas/<area>/` and rewrite in Python
- `verification-case`: move to the owning area as a fixture, manifest row, or area-local helper
- `obsolete-phase-tool`: delete after proving no active reference remains

## Audits Cleanup

`audits/` should not survive as a top-level test system. Verification is the new home for executable audit value.

Initial disposition:

| Current path | Disposition |
| --- | --- |
| `audits/.DS_Store` | Delete. |
| `audits/run_audit.sh`, `audits/run_audit_fast.sh`, `audits/run_borrowing_audit.sh` | Delete after equivalent verification area manifests and runners exist. Do not keep wrappers. |
| `audits/lint_panic_patterns.sh` | Delete or replace with an owned guardrail/verification check under `generated_code_quality` or `scripts/` depending on whether it scans generated output or source. |
| `audits/STDLIB_PARITY_MASTER_REPORT.md` | Archive or delete after any current state is reflected in `stdlib_parity` manifests/data. It must not remain an active report. |
| `audits/*/REPORT.md`, `audits/*/POST_HARDENING_REPORT.md` | Archive or delete. Keep only concise current state in area manifests, area README files, or `plans/` if the history matters. |
| `audits/stdlib/cpython_parity_fixture_format.md` | Keep as a convention, but move to the owning location: `internal_docs/verification/` if it is a human convention, or `verification/areas/stdlib_parity/README.md` if it is area-specific runner guidance. |
| `audits/borrowing/*.sifr` | Promote into `verification/areas/core_language/fixtures/ownership/` or delete duplicates once coverage is proven elsewhere. |
| `audits/lexical_and_syntax/*.sifr` | Promote into `verification/areas/core_language/fixtures/syntax/` or delete duplicates once covered by syntax/parser suites. |
| `audits/type_inference/*.sifr` | Promote into `verification/areas/core_language/fixtures/type_inference/` or delete duplicates once covered by type-system suites. |
| `audits/type_system/*.sifr` | Promote into `verification/areas/core_language/fixtures/type_system/` or delete duplicates once covered by type-system suites. |
| `audits/iteration_protocol/*.sifr` | Promote into `verification/areas/core_language/fixtures/iteration/` unless a case is specifically stdlib parity. |
| `audits/object_model/*.sifr` | Promote into `verification/areas/core_language/fixtures/object_model/` unless a case is specifically stdlib parity. |
| `audits/modules_and_imports/*.sifr` | Promote into `verification/areas/project_workspace/fixtures/imports/`. |
| `audits/python_basics/*.sifr` | Split by ownership: syntax/type/control-flow cases to `core_language`; stdlib behavior to `stdlib_parity`; redundant smoke examples delete after coverage proof. |
| `audits/stdlib/*.sifr` | Promote into `verification/areas/stdlib_parity/fixtures/`. |
| `audits/leetcode/` | Move to `verification/areas/algorithmic_compatibility/corpora/leetcode/` and make it either a submodule or clone-restored corpus, not both. |

Migration rules:

- `fixture-corpus`: promote into the owning verification area
- `external-corpus`: move under the owning verification area and resolve submodule ownership
- `historical-report`: delete from active tree or archive externally
- `obsolete-experiment`: delete
- `policy-spec`: move to `verification/policy/` or an area runbook

Rules:

- No audit report markdown remains active unless it is a timeless policy or spec.
- Every retained audit fixture is referenced by exactly one owning area manifest.
- Each promoted fixture must either become a manifest-owned verification case or be deleted as duplicate/obsolete.
- Stale fixtures should be updated to current Sifr syntax and expected behavior before promotion.
- Duplicate fixtures should be deleted only after the coverage map shows the behavior is already owned by another verification area.
- The LeetCode corpus must have one ownership model: submodule or clone script, not both.
- The LeetCode corpus belongs under `verification/areas/algorithmic_compatibility/corpora/leetcode/`.
- Audit fixtures must be executable by the verification runner.
- Audit output reports are generated under `target/`, not committed.

## Verification Material Cleanup

Classify every existing file under `verification/` before moving it:

- `area-runner`: move under `verification/areas/<area>/runner` or equivalent area-owned code
- `area-fixture`: move under the owning area's fixtures or cases
- `area-baseline`: move under the owning area's baselines
- `area-manifest`: convert to the new area manifest schema
- `lane-policy`: move under `verification/lanes/`
- `global-policy`: move under `verification/policy/`
- `schema`: move under `verification/schemas/`
- `generated-report`: delete from active tree and regenerate under `target/`
- `historical-inventory`: convert to JSON if consumed by a gate, otherwise archive externally or delete
- `obsolete`: delete

Rules:

- `verification/` should not contain loose domain files at the root after cleanup.
- Domain material must live under one area, policy, schema, lane, or runner owner.
- Point-in-time markdown inventories must either become generated reports or compact policy docs.
- JSON inventories are retained only when a gate consumes them or a current design doc references them as canonical data.
- Any retained markdown in `verification/` must explain a stable policy or runbook, not record a past result.

## Internal Docs Cleanup

`internal_docs/` is only for decided technical state and conventions. It should describe how Sifr works now, the contracts the implementation currently upholds, accepted engineering conventions, and accepted architecture/decision records.

It should not contain implementation instructions, execution planning, active issues, review transcripts, validation ledgers, phase checklists, or point-in-time status reports.

Shape `internal_docs/` around a small set of durable topics:

- `architecture.md`
- `syntax_architecture.md`
- `integer_model.md`
- `async_concurrency_model.md`
- `network_http_architecture.md`
- `text_i18n_architecture.md`
- `frontend_query_architecture.md`
- `frontend_cache_invalidation.md`
- `narrowing_flow_facts_design.md`
- `sifr_workspace_design.md`
- `structured_runtime_work_model.md`
- `dependency_policy.md`
- `generated_code_quality.md`
- `performance_budgets.md`
- `lsp_server.md`
- `editor_integrations.md`
- `vscode_extension.md`
- `distribution_pipeline.md`

Move execution material out of `internal_docs/`:

- `roadmap.md` -> `plans/roadmap.md`
- `phases/*.md` -> `plans/phases/`
- phase checklists and execution ledgers -> `plans/`
- active or future implementation instructions -> `plans/`
- repeated validation evidence -> generated reports under `target/`
- review transcript references -> `plans/reviews/`

Clean up docs in place instead of over-classifying them:

- consolidate scattered docs when one current design document would be clearer
- remove stale status sections once the current state is described elsewhere
- rewrite procedural "how to implement" sections into state/contract descriptions, or move them to `plans/`
- keep verification conventions in `internal_docs/verification/` when they describe human-facing conventions
- move runner mechanics, schemas, lane data, baseline retention, and generated-artifact policy under `verification/`

Rules:

- Internal docs should explain current architecture, accepted decisions, implemented behavior, durable state, and accepted engineering conventions.
- Internal docs should not tell contributors how to implement a future change. Those instructions belong in `plans/`.
- Internal docs should not contain long validation transcripts, repeated pass/fail logs, or AI-review path ledgers.
- Phase-numbered architecture-transfer notes should either consolidate into a small design narrative or move to an archive directory.
- Absolute personal reference paths must become environment-variable based references, repo-relative references, or explicit external-reference notes.
- Docs that describe planned work but not accepted design belong under `plans/`, not `internal_docs/`.
- Verification convention docs may live in `internal_docs/verification/` when they describe human-facing conventions.
- Verification docs that are part of runner mechanics, schema contracts, lane data, baseline retention, or generated-artifact policy belong under `verification/policy/`, `verification/schemas/`, or a verification area.

## Submodule Policy

Submodules inside a move zone require dedicated handling.

Known submodule categories:

- parser and compiler reference dependencies under `third_party/`
- editor integration repositories
- verification demo repositories
- large verification corpora
- LeetCode or external audit corpora

Rules:

- Every submodule has one owning top-level area.
- Submodule moves update `.gitmodules`, scripts, CI, docs, and manifests in the same PR.
- A corpus is either a submodule or restored by a clone script, not both.
- Optional heavy corpora are excluded from default create-pr validation unless explicitly selected by lane policy.

## Cargo Lock Policy

`Cargo.lock` is a load-bearing file for this binary-producing compiler workspace and should be tracked.

Cleanup must:

- remove any ignore rule that conflicts with tracking `Cargo.lock`
- keep `Cargo.lock` in the root tracked set
- document why the compiler workspace tracks the lockfile
- validate that CI and local builds use the same dependency graph

This policy should land as its own PR or isolated commit because it affects every contributor.

## Guardrails

Add repo-hygiene guardrails for:

- personal absolute paths in active workflow/code surfaces
- tracked local editor artifacts
- tracked generated validation reports
- tracked Python bytecode and cache directories
- active phase-numbered verification script names
- verification fixtures not owned by any area manifest
- verification fixtures owned by more than one area manifest
- markdown reports under `verification/` that are not README, policy, or runbooks
- stale references to removed review transcript paths
- stale references to removed script entrypoints

The personal-path guardrail must allow intentional example paths in tests when they are explicitly part of fixture data.

## Proposed PR Sequence

### PR 1: Repository Surface Contract

Add the full tracked-entry disposition tables:

- top-level entries
- `scripts/` entries
- `verification/` entries
- `audits/` entries
- `internal_docs/` entries
- submodules
- Cursor workflow files
- plan, issue, phase, and review artifact locations

No moves in this PR.

Validation:

- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`

### PR 1B: Relevance Audit

Produce reviewable keep/move/rewrite/delete tables for stale or outdated material before any deletion-heavy PR:

- `audits/`
- `verification/`
- `internal_docs/`
- `scripts/`
- `.cursor/`

Each row must include:

- current path
- classification
- destination or deletion rationale
- whether a gate consumes it
- whether a current doc references it
- validation required after changing it

Validation:

- `git diff --check`
- no file moves
- no deletions

### PR 2: Cargo Lock Policy

Make `Cargo.lock` tracking explicit and fix ignore rules.

Validation:

- `cargo check --workspace`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 3: Cursor Portability Cleanup

Remove local-machine assumptions from `.cursor/` and update commands for the planned `plans/` layout.

Validation:

- Cursor hygiene guardrail
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 4: Review Tree Normalization

Move intentionally retained review artifacts under `plans/reviews/active/` or `plans/reviews/archive/`. Delete review artifacts that have no ongoing planning value and replace direct transcript ledgers in active docs with concise summaries.

Validation:

- no top-level tracked `reviews/`
- retained review artifacts live under `plans/reviews/`
- no active doc depends on review transcript paths
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 5: Planning Tree Normalization

Move active, completed, and archived phase and issue docs into the new lifecycle directories.

Validation:

- all roadmap links resolve
- all AGENTS/Cursor references resolve
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 6: Verification Runner Foundation

Add the stdlib-only verification runner, schemas, area discovery, and result format.

No existing runner migration yet.

Validation:

- schema self-tests
- runner discovery self-test
- `scripts/run_all_tests.sh --profile create-pr`

### PR 7: Verification Lane Normalization

Split lane configuration into `verification/lanes/*.json` and validate with schemas.

Validation:

- all lanes schema-valid
- old lane manifest removed
- `scripts/run_all_tests.sh --profile create-pr`

### PR 8-N: Verification Area Corpus Migration

Migrate one verification domain per PR into `verification/areas/<area>/`.

Candidate order:

1. `diagnostics`
2. `project_workspace`
3. `core_language`
4. `regression`
5. `fuzz_property`
6. `determinism`
7. `generated_code_quality`
8. `performance`
9. `developer_tooling`
10. `runtime_platform`
11. `distribution_release`
12. `package_management`
13. `stdlib_parity`
14. `algorithmic_compatibility`
15. `ecosystem_compatibility`

Validation for each area migration:

- area manifest schema-valid
- moved fixtures are owned by exactly one area
- shell verification helpers rewritten to stdlib-only Python unless shell is the artifact under test
- old verification script names deleted, not wrapped
- runner can execute the area
- determinism check for affected area
- sequential/parallel equivalence where applicable
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+1: Submodule Normalization

Resolve submodule ownership and move any submodule paths that belong under verification areas.

Validation:

- `.gitmodules` is correct
- clone/restoration scripts are correct
- CI checkout still initializes required submodules
- affected area runner executes
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+2: Scripts Verification Migration

Move remaining verification implementation out of `scripts/` and into verification areas.

Delete old entrypoints. Do not add compatibility wrappers.

Validation:

- no verification implementation remains in `scripts/`
- no stale script references remain
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh --profile merge`

### PR N+3: Audits Normalization

Promote retained audit fixtures into verification manifests and remove historical report markdown from the active tree.

Validation:

- no top-level `audits/`
- every retained audit fixture is manifest-owned
- audit area executes through the runner
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+4: Internal Docs Relevance Cleanup

Move, consolidate, or delete outdated internal docs according to the relevance audit.

Validation:

- roadmap links resolve
- phase roadmap links resolve
- verification policy docs live under `verification/policy/`
- no active internal doc contains long validation transcript ledgers
- `git diff --check`

### PR N+5: Docs And Guardrails Closeout

Update public docs, internal docs, AGENTS, Cursor commands, and CI docs to reflect the final structure.

Add all hygiene guardrails.

Validation:

- guardrails pass
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh --profile merge`

## Acceptance Criteria

- Fresh clone top-level tree matches the top-level contract.
- No top-level tracked `reviews/` tree exists.
- Any retained review artifacts live under `plans/reviews/`.
- No top-level `audits/` tree exists.
- `.cursor/` contains only portable workflow assets.
- No active workflow/code surface contains personal `/Users/yaseralnajjar/...` paths.
- `Cargo.lock` is tracked and intentionally not ignored.
- `scripts/run_all_tests.sh --profile create-pr` is the only public create-PR validation command.
- `scripts/run_all_tests.sh` delegates directly to the verification runner.
- No active verification implementation remains in `scripts/`.
- Every verification area has a schema-valid manifest.
- Every retained fixture has exactly one owning area manifest.
- Lane files reference areas instead of duplicating fixture ownership.
- No committed validation result logs exist outside explicit baselines.
- Markdown under `verification/` is limited to README, policy, and runbooks.
- CI and local validation use the same runner and lane files.
- Migration PRs prove determinism and sequential/parallel equivalence where applicable.
- Final closeout passes local merge validation.

## Risks

- Moving fixtures can reorder e2e discovery or invalidate snapshots.
- Removing review transcripts can break historical issue references unless summaries replace them.
- Moving submodules requires atomic `.gitmodules`, script, CI, docs, and manifest updates.
- A Python runner can drift if it gains external dependencies; keep it stdlib-only.
- Parallel scheduling can hide resource contention unless every area declares resource policy.
- Historical docs may contain the only explanation for a design decision; archive by value, not by age.

## Review Notes

This plan was reviewed with the Fable high model before being written. The main findings incorporated here were:

- the top-level contract must include load-bearing entries such as `lib/`, `.github/`, `sifr.toml`, and `logo.webp`
- corpus migration and submodule migration must be explicit PRs, not risks hidden inside script movement
- the personal-path guardrail must distinguish workflow/code leaks from intentional test fixture paths
- `Cargo.lock` tracking deserves its own policy step
- the runner should commit to stdlib-only Python
- `internal_docs/verification/` should move to `verification/policy/` instead of remaining as a temporary half-state
