# Ad Hoc Repository Architecture And Verification Surface Cleanup

status: implementation-ready

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
- No duplicate old and new verification layouts in the end state. Transitional coexistence during migration is allowed only when tracked by an explicit migration-status table.
- No archive-in-place that leaves the active tree visually noisy.
- No tracking of point-in-time validation logs.
- No broad rewrite of compiler crates unless required by path or fixture ownership.

## Core Principles

- Active tree shows the product, not the process history.
- Humans read short, current docs and issue plans.
- Machines consume schemas, manifests, fixtures, and baselines.
- Generated reports live under `target/` and are not committed.
- Each fixture has one owning verification area.
- Profiles select verification areas; profiles do not own fixtures.
- Public validation has one entrypoint.
- Repo hygiene is enforced by guardrails, not convention.
- Historical material is either deleted, moved to git history, or placed in a separate archive outside the active tree.

## Reference Compiler Lessons

This cleanup should learn from mature compiler and runtime repositories without copying their exact folder names. The useful lesson is the ownership invariant behind each layout.

Reference observations:

- TypeScript keeps compiler cases and baselines as first-class test assets under `tests/cases/` and `tests/baselines/reference/`, with harness code separated from the cases. Sifr should keep fixtures and baselines area-owned and reviewable, not hidden in scripts or markdown reports.
- typescript-go uses a strong split between compiler implementation, repo tools, and `testdata/` compatibility baselines. Sifr should preserve the same conceptual split through `crates/`, `scripts/`, and `verification/areas/`, rather than creating a generic top-level `testdata/` tree.
- Rust separates compiler implementation, standard library, test runner tooling, and behavior-mode test families such as UI, codegen, run-make, incremental, rustdoc, and debuginfo. Sifr should let areas own suites by execution mode when useful, instead of forcing every case into one flat pass/fail bucket.
- Bun separates tests, integration fixtures, regression material, and benchmarks. Sifr should keep performance validation distinct from correctness verification and avoid mixing benchmark artifacts with regular create-PR gates.
- CPython separates public docs, maintainer-facing internal docs, test runner implementation, resource-heavy tests, platform data, and tools. Sifr should keep `internal_docs/` as current maintainer knowledge and make profile resource policy explicit in the verification runner.

Sifr-specific conclusions:

- Keep `verification/` as the top-level validation contract rather than adding a generic `tests/` or `testdata/` tree.
- Keep `plans/` separate from `internal_docs/`; mature repos do not make active execution plans look like compiler architecture.
- Treat baselines as source-controlled contracts only when they are intentional expected outputs. Generated run logs still belong under `target/`.
- Support area-local case directives or metadata when they make a single fixture self-describing, but the area manifest remains the ownership source of truth.
- Resource classes, flake policy, retry policy, shard policy, and resume behavior belong in profiles and runner policy, not in one-off scripts.
- Benchmarks and performance budgets should have their own area and profile policy so normal validation stays deterministic and appropriately fast.

## Target Top-Level Contract

The top-level tree should contain only intentional, load-bearing entries:

```text
.github/
.cursor/
.gitignore
.gitmodules
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

Implementation rule: every tracked file or directory move must use `git mv`, followed by the intended content/reference edits, validation, and an explicit commit for that PR slice. Do not move tracked files with plain filesystem commands and leave Git to infer renames later.

## Planning, Issues, And Reviews Structure

Target planning shape:

```text
plans/
  README.md
  roadmap.md

  phases/
    index.md
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
- `plans/phases/index.md` indexes phase status.
- Phase files stay flat under `plans/phases/` so phase numbers remain stable and easy to find.
- Phase status lives in each phase header and in `plans/phases/index.md`; phases do not need active/completed/archive directories.
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

## Implementation Progress

| Milestone | Status | PR |
| --- | --- | --- |
| PR 1 Repository Surface And Relevance Inventory | Merged | <https://github.com/sifr-lang/sifr/pull/2506> |
| PR 2 Cargo Lock Policy | Merged | <https://github.com/sifr-lang/sifr/pull/2507> |
| PR 3 Cursor Portability Cleanup | Merged | <https://github.com/sifr-lang/sifr/pull/2508> |
| PR 4 Review Tree Normalization | Merged | <https://github.com/sifr-lang/sifr/pull/2509> |
| PR 5 Planning Tree Normalization | Merged | <https://github.com/sifr-lang/sifr/pull/2510> |
| PR 6 Verification Runner Foundation | Merged | <https://github.com/sifr-lang/sifr/pull/2511> |
| PR 7 Verification Profile Normalization | Merged | <https://github.com/sifr-lang/sifr/pull/2512> |
| PR 8 Diagnostics Area Migration | Merged | <https://github.com/sifr-lang/sifr/pull/2513> |
| PR 9 Project Workspace Area Migration | Merged | <https://github.com/sifr-lang/sifr/pull/2514> |
| PR 10 Core Language Contract Migration | Merged | <https://github.com/sifr-lang/sifr/pull/2515> |
| PR 11 Regression Area Migration | In progress | This branch |

## Verification Migration Status

This table is the required source of truth while legacy validation surfaces and
the new `sifr_verify` runner coexist. `scripts/run_all_tests.sh` remains the
authoritative public facade until the facade cutover PR.

| Area | Legacy path | New area path | Current authoritative gate | Equivalence evidence | Cutover status |
| --- | --- | --- | --- | --- | --- |
| Runner foundation | `scripts/run_all_tests.sh`, `scripts/validation_lane.py`, `scripts/validation_lane_report.py` | `verification/runner/sifr_verify/`, `verification/schemas/`, `verification/policy/`, `verification/pyproject.toml`, `verification/uv.lock` | Existing bash facade plus `sifr_verify` self-tests/profile helpers | `uv lock --project verification --check`; `uv run --project verification --locked python -m sifr_verify --self-test`; `scripts/run_all_tests.sh --profile create-pr` | Foundation merged; profile helpers now own shell/report policy |
| Profiles | Deleted `verification/validation_lanes/manifest.json`; retained `verification/validation_lanes/*_e2e_manifest.json` until `core_language` migration | `verification/profiles/{create-pr,merge,nightly,release}.json` | `uv run --project verification --locked python -m sifr_verify profiles shell --profile <profile>` feeding the legacy bash facade | `uv run --project verification --locked python -m sifr_verify profiles check`; old-vs-new shell export diff for all four profiles; report summarization payload equivalence | Merged in PR 7; profile source cut over, fixture manifests not migrated |
| `diagnostics` | Deleted diagnostics row from `verification/suites/manifest.json`; moved diagnostic sync/coverage/hygiene scripts and `crates/sifr/tests/verification/diagnostics/` fixtures | `verification/areas/diagnostics/` | `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite contracts`; merge/nightly/release hardening dispatches diagnostics baselines through `--suite baselines` | `uv run --project verification --locked python -m sifr_verify areas check`; diagnostics contracts and baselines area execution; legacy facade `scripts/run_all_tests.sh --profile create-pr` | Merged in PR 8; diagnostics source cut over, presentation synthetic baselines validated by diagnostics contract checker |
| `project_workspace` | Deleted project row from `verification/suites/manifest.json`; moved `crates/sifr/tests/verification/project/` fixtures; deleted project/workspace contract shell wrappers | `verification/areas/project_workspace/` | `uv run --project verification --locked python -m sifr_verify areas run --area project_workspace --suite baselines`; `uv run --project verification --locked python -m sifr_verify areas run --area project_workspace --suite frontend_mode_parity --suite phase23_graph_isolation`; merge/nightly/release hardening dispatches project baselines through this area | Area schema validation, project area baseline execution, exact project workspace contract suite execution, legacy hardening dispatch equivalence; `scripts/run_all_tests.sh --profile create-pr`; Opus PASS after exact-suite filter review | Baselines merged in PR 9; exact contract suite migration merged in PR 10 |
| `core_language` | `scripts/run_e2e_pass.sh`, `verification/validation_lanes/*_e2e_manifest.json`, core contract rows, `verification/validation_contracts/` | `verification/areas/core_language/` | `uv run --project verification --locked python -m sifr_verify areas run --area core_language --suite integer_dtype_contract --suite phase24_hir_analysis --suite phase25_cfg_flow`; legacy e2e runner remains authoritative for e2e pass fixtures | Exact core language contract suite execution; profile/facade contract dispatch through `sifr_verify areas run`; Opus PASS after exact-suite filter review; e2e migration pending | Contract matrix source cut over in PR 10; e2e pass runner and fixture manifests still pending |
| `regression` | Deleted `verification/fixedbugs/index.json`, `verification/crashes/index.json`, and crate-local crash reproducers | `verification/areas/regression/` | `uv run --project verification --locked python -m sifr_verify areas run --area regression --suite fixedbugs --suite crashes`; merge/nightly/release hardening dispatches these suites through the regression area | Area schema validation, fixedbugs/crashes area execution, legacy hardening dispatch equivalence; `scripts/run_all_tests.sh --profile create-pr` | In progress in PR 11 |
| `fuzz_property` | `verification/fuzz_property/`, `scripts/run_smoke_fuzz_property.sh` | `verification/areas/fuzz_property/` | Legacy smoke/property scripts | Pending area migration PR | Not started |
| `generated_code_quality` | `verification/generated_code_quality/` | `verification/areas/generated_code_quality/` | Legacy generated-code shell helpers | Pending area migration PR | Not started |
| `performance` | `verification/performance/`, `verification/perf/`, performance scripts | `verification/areas/performance/` | Legacy performance budget scripts | Pending area migration PR | Not started |
| `developer_tooling` | `verification/tooling/`, LSP corpus submodule | `verification/areas/developer_tooling/` | Legacy tooling scripts | Pending area migration PR | Not started |
| `runtime_platform` | `verification/platform/`, `scripts/run_platform_golden.sh` | `verification/areas/runtime_platform/` | Legacy platform golden runner | Pending area migration PR | Not started |
| `distribution_release` | `verification/distribution/`, `scripts/run_distribution_validation.sh` | `verification/areas/distribution_release/` | Legacy distribution validation script | Pending area migration PR | Not started |
| `package_management` | `verification/package_management/` | `verification/areas/package_management/` | Legacy package guardrails and facade | Pending area migration PR | Not started |
| `stdlib_parity` | `verification/stdlib/`, stdlib corpus scripts | `verification/areas/stdlib_parity/` | Legacy stdlib scripts and facade | Pending area migration PR | Not started |
| `algorithmic_compatibility` | LeetCode audit/subrepo material and corpus taxonomy scripts | `verification/areas/algorithmic_compatibility/` | Legacy audit/corpus tooling | Pending area migration PR | Not started |
| `ecosystem_compatibility` | `verification/oss/` | `verification/areas/ecosystem_compatibility/` | Legacy hardening runner | Pending area migration PR | Not started |

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
- Commands target the new `plans/issues/active/`, `plans/phases/`, and `plans/reviews/active/` layout only after the planning tree exists.
- Review skills write to `plans/reviews/active/` by default.
- Keep `.cursor/skills/talk-to-claude-opus/` as the single Claude/Fable review workflow.
- Remove the other Claude review skill variants once their useful instructions are folded into `talk-to-claude-opus`.
- Do not introduce additional model-branded review skill variants; `talk-to-claude-opus` remains the existing workflow contract.
- Remove `.cursor/.rules/`.
- Move useful `.cursor/plans/` content into `plans/`; delete embedded Obsidian/local state and obsolete workflow notes.

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
  pyproject.toml          # uv-managed verification tooling project
  uv.lock                 # tracked verification tooling lockfile
  policy/                 # runner/data policy, not human conventions
  schemas/
    profile.schema.json
    area.schema.json
    suite.schema.json
    case.schema.json
    result.schema.json
  profiles/
    create-pr.json
    merge.json
    nightly.json
    release.json
  runner/
    sifr_verify/
      __main__.py
      profiles.py
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
- Python verification tooling is managed by `uv`.
- The runner should keep dependencies small and explicit. If verification needs Python packages, they are declared in `verification/pyproject.toml`, locked in `verification/uv.lock`, and invoked through `uv run --project verification`.
- No ad hoc `pip install`, unmanaged virtualenv, user-site package, or machine-local Python package assumption is allowed.
- Schemas are a committed data contract, and the runner validates only an explicit supported subset: object shape, required keys, primitive scalar types, arrays of objects or strings, enums, and repo-relative path strings. No `$ref`, `allOf`, `anyOf`, `patternProperties`, or unreviewed schema feature expansion is allowed.
- The runner must reject any committed schema that uses keywords outside the supported subset. Silent ignoring of unsupported schema features is forbidden.
- The runner discovers areas by `verification/areas/*/manifest.json`.
- The top-level verification concepts are only `runner`, `schemas`, `profiles`, `areas`, and `policy`.
- `areas` answer what is verified and who owns it.
- `profiles` answer when verification runs and with what resource budget. The name deliberately matches the public `--profile` flag of `scripts/run_all_tests.sh`; the flag value resolves directly to `verification/profiles/<name>.json`.
- `schemas` answer what shape committed verification data must have.
- `runner` answers how verification is executed.
- `policy` answers machine-facing operational rules for baselines, artifacts, flakes, retention, and schema governance.
- `toolchain` steps are runner-executed cargo/rustfmt/clippy/test steps selected by profiles through schema-defined names. Toolchain step names are an enum in `profile.schema.json`, not a separate registry. They cover workspace build and Rust test gates without making raw shell commands part of profile data.
- `guardrail` steps are runner-executed entries selected by profiles through a committed registry at `verification/policy/guardrails.json`. The registry maps stable guardrail names to `scripts/` entrypoints, arguments, timeout policy, and expected report shape.
- Suites are area-local groupings, not a competing top-level taxonomy. A suite may live under `verification/areas/<area>/suites/` or inside the area manifest.
- Cases are area-owned. A profile may select a subset of an area or suite, but it never owns fixtures directly.
- A case is the unit of verification: a manifest or suite entry plus the fixture files and expected outputs it references. A fixture is on-disk input material only; fixtures are never executed except as part of a case.
- A golden fixture is not a separate top-level material kind. It is a case whose purpose is to protect a stable, cross-cutting expected behavior contract. Golden cases live inside their owning area as ordinary cases with `kind: "golden"` or an area-local `golden` suite.
- Each area owns its fixtures, baselines, runner code, manifests, and area-local data.
- Each area manifest declares owner, suites, fixtures, baselines, resources, parallel safety, expected outputs, timeout policy, and result contract.
- Area-local fixture directives are allowed when they make compiler expectations readable next to the source file, but manifests remain the ownership and discovery source of truth.
- An area `runner.py` implements only the schema-defined adapter interface: discover, execute, and report results for its cases. Scheduling, parallelism, retries, resource classes, and report generation belong exclusively to `sifr_verify`; area adapters may not implement their own framework.
- Profile files select areas or area-local suites and define execution policy; they do not duplicate fixture lists.
- Generated reports go to `target/verification_reports/`.
- Markdown under `verification/` is limited to `README.md`, policy, and runbooks.
- Gate inputs are JSON, schemas, baselines, or fixtures.
- Parallelism is explicit per area.
- Performance-sensitive or shared-state areas must declare `parallel_safe: false`.
- Sequential/parallel equivalence is a runner self-check, not an ad hoc shell script or permanent verification area.
- Runner determinism, report-signature checks, and profile equivalence live under `verification/runner/` self-tests and `verification/policy/` data.

Area-local shape:

```text
verification/areas/<area>/
  manifest.json           # only mandatory area file
  README.md               # short area runbook only when needed
  runner.py               # optional area adapter called by the verification runner
  suites/                 # optional suite manifests for large areas
  fixtures/               # first-party source inputs and minimized cases
  corpora/                # external corpora and submodules owned by this area
  baselines/              # checked-in expected outputs
  data/                   # machine-readable inventories consumed by this area
```

Area subdirectories are created only on first use. Small areas should not carry empty ceremony.

Golden fixture normalization:

- The word `golden` is allowed because it is established testing vocabulary, but it must describe case intent, not folder ownership outside the area model.
- A golden case still has one owning area, one manifest entry, declared expected output or baseline references, and normal profile selection.
- Full expected stdout/stderr/diagnostic snapshots belong in `baselines/`; compact assertions such as expected exit code and required output substrings may live in the case manifest.
- Current platform golden fixtures move from `verification/platform/golden/` into `verification/areas/runtime_platform/` as a `platform_contract` or `golden` suite.
- `scripts/run_platform_golden.sh` is deleted after the `runtime_platform` area runner and selected profiles execute the same cases.

Profile-local shape:

```text
verification/profiles/<profile>.json
```

Profile files may define:

- selected areas or area-local suites
- selected toolchain step sets
- selected guardrail step sets
- warm and cold time budgets
- resource policy
- resource classes such as network, large-memory, platform-specific, long-running, or external-corpus
- shard policy
- retry and flake policy
- resume and failure-reproduction policy
- parallelism limits
- generated report requirements

Profile files may not define:

- fixture paths directly
- expected diagnostic text
- baselines
- area ownership
- raw shell commands or one-off command strings

Naming convention:

- Profile names are kebab-case because they are CLI-facing, such as `--profile create-pr`.
- Area and suite names are snake_case because they are identifier-facing: directories, manifest keys, and Python modules.

Canonical runner invocation:

```bash
uv run --project verification python -m sifr_verify --profile create-pr
```

`verification/pyproject.toml` owns package discovery for `verification/runner/sifr_verify/`, pins `requires-python`, and declares any needed verification dependencies. `verification/README.md` documents the minimum supported `uv` version. `scripts/run_all_tests.sh` fail-fasts with an actionable message when `uv` is missing or below that minimum. CI may install the pinned `uv` version before invoking the same local validation facade; the test behavior itself must remain identical between local and CI execution.

Permanent verification areas:

Ownership follows the contract being asserted, not the feature being exercised. A fixture asserting CPython-observable behavior belongs to `stdlib_parity`; a fixture asserting compile-time Sifr semantics belongs to `core_language`; a minimized fixed bug belongs to `regression`; an external project compatibility signal belongs to `ecosystem_compatibility`.

| Area | Owns |
| --- | --- |
| `core_language` | Broad compiler behavior fixtures for syntax, lowering, type checking, ownership, control flow, and pass/fail language behavior that is not more specifically owned elsewhere. |
| `diagnostics` | Diagnostic codes, renderer behavior, JSON/compact/human diagnostic contracts, docs/schema sync fixtures, and diagnostic presentation baselines. |
| `project_workspace` | Multi-file project behavior, imports, module graph, workspace resolution, graph isolation, cache identity, and project-mode command semantics. |
| `regression` | Fixed bugs, minimized crash reproducers, invariant failures, and promoted sentinel cases. |
| `fuzz_property` | Deterministic property tests, fuzz smoke seeds, sustained fuzz corpus metadata, minimization conventions, and seed provenance. |
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

`crates/sifr/tests/verification/` is repo-level verification material unless a fixture is truly private to a crate unit test. Repo-level fixtures currently under that path move into the owning `verification/areas/<area>/fixtures/`; Rust test harnesses may remain under `crates/sifr/tests/` but should reference area-owned fixtures. Crate-local unit-test fixtures may remain next to crate tests only when the crate test itself owns the data and no repo-level manifest references it.

## Scripts Cleanup

`scripts/` becomes repo operations and developer ergonomics only. It must not own verification implementation.

Boundary rule: a check that gates first-party source or repo hygiene and needs no compiled artifact is a guardrail in `scripts/`; a check that executes or inspects compiler behavior, generated output, diagnostics, or binaries is verification and belongs to an area.

Guardrails remain implemented in `scripts/`, but the end-state validation runner invokes them through `verification/policy/guardrails.json` so `scripts/run_all_tests.sh` can stay a thin dispatcher while profiles avoid raw shell commands.

Keep in `scripts/`:

- `run_all_tests.sh` as the only public validation facade
- source and repository guardrails
- code generators
- release and publishing tools
- repository maintenance utilities

Move into `verification/areas/*`:

- e2e pass runner -> `core_language`
- validation contract matrices -> owning areas listed in the discovered verification disposition table: `core_language` and `project_workspace`
- hardening runner -> split into `regression`, `fuzz_property`, `ecosystem_compatibility`, and runner self-tests
- platform golden checks -> `runtime_platform`
- generated-code quality checks -> `generated_code_quality`
- performance budget checks -> `performance`
- tooling and LSP verification checks -> `developer_tooling`
- distribution validation cases -> `distribution_release`
- stdlib namespace and corpus validation -> `stdlib_parity`
- fuzz and property checks -> `fuzz_property`
- report determinism and sequential/parallel equivalence checks -> `verification/runner/` self-tests and `verification/policy/` data

Rules:

- No active script filename may encode stale phase numbers.
- No duplicate entrypoints for the same gate.
- No compatibility wrapper remains for an old script name after migration.
- `AGENTS.md`, CI workflows, README, internal docs, and Cursor commands update atomically with script moves.
- Shell is allowed only for tiny public facades or release scripts where POSIX shell is the artifact under test.
- Verification scripts moved out of `scripts/` should be rewritten as Python area runners managed by the verification `uv` project unless the checked artifact is itself POSIX shell.
- Shell-to-Python rewrites must preserve command semantics, exit-code behavior, timeout behavior, output normalization, and generated report shape.
- Rewrites must land with side-by-side equivalence evidence before the shell implementation is deleted.
- `run_all_tests.sh` is a facade, not a second runner. During migration it may dispatch to both legacy and new implementations only when the migration-status table says which path is authoritative. In the end state it must be a thin profile dispatcher over `sifr_verify`.

Script migration classification:

- `repo-guardrail`: stays in `scripts/`
- `code-generator`: stays in `scripts/`
- `release-tool`: stays in `scripts/` unless it is only a validation case
- `verification-runner`: move to `verification/areas/<area>/` and rewrite in Python
- `verification-case`: move to the owning area as a fixture, manifest row, or area-local helper
- `obsolete-phase-tool`: delete after proving no active reference remains

Initial script disposition:

Stay in `scripts/` as public facades, repo guardrails, repo maintenance, code generators, or release tooling:

- `scripts/run_all_tests.sh` -> keep as the only public validation facade; end state is a thin profile dispatcher over `sifr_verify`.
- `scripts/check_file_size_guardrails.py`, `scripts/check_hir_maintainability_guardrails.py`, `scripts/check_sifr_driver_maintainability_guardrails.py`, `scripts/check_source_crate_dependency_direction.py` -> keep as source/repo guardrails.
- `scripts/check_codegen_rawcode_gate.sh` -> keep as a source guardrail unless it grows generated-output inspection; if it does, move the generated-output portion to `generated_code_quality`.
- `scripts/check_diagnostic_cancel_usage.py`, `scripts/check_diagnostic_transport_cleanup.py` -> keep as diagnostics source hygiene guardrails.
- `scripts/check_integer_dtype_contract.py` -> move to `core_language`; it validates the active integer dtype contract sentinel under verification data.
- `scripts/check_package_manager_guardrails.py` -> move to `package_management`; it validates package-manager source boundaries and package-management verification matrices together.
- `scripts/clone_subrepos.sh` -> keep as repository maintenance.
- `scripts/generate_unicode_tables.py` -> keep as a code generator.
- `scripts/distribution/build_preview_artifacts.sh`, `scripts/distribution/create_new_version.sh`, `scripts/distribution/generate_dispatchers.sh`, `scripts/distribution/generate_version_installer.sh` -> keep as release tooling.

Move into verification areas or runner-owned policy:

- `scripts/run_e2e_pass.sh` -> `verification/areas/core_language/` as an area suite runner or delete after the `create-pr` profile owns the same cases.
- `scripts/run_validation_contract_matrix.sh` -> split by contract ownership: `core_language`, `project_workspace`, and `diagnostics`; shared harness pieces belong in `verification/runner/`.
- `scripts/run_phase23_graph_isolation_matrix.sh` -> `project_workspace`.
- `scripts/run_phase24_hir_analysis_consolidation_matrix.sh`, `scripts/run_phase25_cfg_flow_activation_matrix.sh` -> `core_language`.
- `scripts/run_frontend_mode_parity_matrix.sh` -> `project_workspace`; it is a wrapper over the validation contract matrix for project/compiler frontend mode parity.
- `scripts/run_platform_golden.sh` -> `runtime_platform`.
- `scripts/run_smoke_fuzz_property.sh` -> `fuzz_property`.
- `scripts/run_integer_model_closure_perf.py`, `scripts/ci_e2e_throughput.sh` -> `performance` with explicit budget/profile policy.
- `scripts/check_codegen_binary_size.sh` -> `performance`; binary size is treated as an explicit performance budget.
- `scripts/check_diagnostic_baseline_hygiene.py`, `scripts/check_diagnostic_code_coverage.py`, `scripts/check_diagnostic_docs_sync.py`, `scripts/check_diagnostic_schema_sync.py` -> `diagnostics`.
- `scripts/run_distribution_validation.sh`, `scripts/distribution/validate_self_update_metadata.sh` -> `distribution_release` if used as validation gates; keep only release-mutating operations in `scripts/distribution/`.
- `scripts/run_stdlib_namespace_corpus_validation.py`, `scripts/check_phase30_complexity_resource_inventory.py` -> `stdlib_parity`.
- `scripts/build_full_corpus_failure_taxonomy.py` -> `algorithmic_compatibility`; it builds failure taxonomy artifacts for full algorithmic corpus result JSON.
- `scripts/generate_concurrency_runtime_inventory.py` -> `stdlib_parity` as an area-local data generator if the output is verification inventory.
- `scripts/run_verification_hardening.py` and `scripts/run_verification_hardening/` -> split across `regression`, `fuzz_property`, `ecosystem_compatibility`, and runner self-tests; do not keep a generic hardening runner after areas own the suites.
- `scripts/check_e2e_report_determinism.sh`, `scripts/check_e2e_sequential_parallel_equivalence.sh` -> `verification/runner/` self-tests and `verification/policy/` evidence.
- `scripts/validation_lane.py`, `scripts/validation_lane_report.py` -> replaced with `verification/runner/sifr_verify/profiles.py` and profile report handling; deleted in PR 7.

Delete or replace after references are removed:

- `scripts/__pycache__/` and `scripts/run_verification_hardening/__pycache__/` -> delete and guard against tracked Python bytecode.
- `scripts/validate_phase15_backlog.py` -> delete; it protects a historical phase-15 backlog path and should not survive the planning-tree move.
- `scripts/phase_contract_gate_check.py` -> replace with a current `plans/phases/index.md` consistency guardrail if still useful; delete the phase-numbered path assumptions.
- `scripts/archive_issues.sh`, `scripts/archive_reviews.sh`, `scripts/archive_reviews_and_issues.sh` -> update for `plans/issues/{active,completed,archive}` and `plans/reviews/{active,archive}` if still needed; otherwise delete in favor of direct `git mv` during planning PRs.

## Audits Cleanup

`audits/` should not survive as a top-level test system. Verification is the new home for executable audit value.

Initial disposition:

| Current path | Disposition |
| --- | --- |
| `audits/.DS_Store` | Delete. |
| `audits/run_audit.sh`, `audits/run_audit_fast.sh`, `audits/run_borrowing_audit.sh` | Delete after equivalent verification area manifests and runners exist. Do not keep wrappers. |
| `audits/lint_panic_patterns.sh` | Replace under `generated_code_quality`; it enforces user-facing generated-code panic policy for `sifr_codegen` and should be owned with the other generated-code quality gates. |
| `audits/STDLIB_PARITY_MASTER_REPORT.md` | Archive or delete after any current state is reflected in `stdlib_parity` manifests/data. It must not remain an active report. |
| `audits/*/REPORT.md`, `audits/*/POST_HARDENING_REPORT.md` | Archive or delete. Keep only concise current state in area manifests, area README files, or `plans/` if the history matters. |
| `audits/stdlib/cpython_parity_fixture_format.md` | Keep as a convention, but move to the owning location: `internal_docs/verification/` if it is a human convention, or `verification/areas/stdlib_parity/README.md` if it is area-specific runner guidance. |
| `audits/borrowing/*.sifr` | Promote into `verification/areas/core_language/fixtures/ownership/` or delete duplicates once coverage is proven elsewhere. |
| `audits/lexical_and_syntax/*.sifr` | Promote into `verification/areas/core_language/fixtures/syntax/` or delete duplicates once covered by syntax/parser suites. |
| `audits/type_inference/*.sifr` | Promote into `verification/areas/core_language/fixtures/type_inference/` or delete duplicates once covered by type-system suites. |
| `audits/type_system/*.sifr` | Promote into `verification/areas/core_language/fixtures/type_system/` or delete duplicates once covered by type-system suites. |
| `audits/iteration_protocol/*.sifr` | Promote according to the asserted contract: compile-time iteration semantics to `core_language`, CPython-observable behavior to `stdlib_parity`, fixed bugs to `regression`. |
| `audits/object_model/*.sifr` | Promote according to the asserted contract: compile-time object-model semantics to `core_language`, CPython-observable behavior to `stdlib_parity`, fixed bugs to `regression`. |
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
- `golden-case`: convert to an area-owned case with `kind: "golden"` or an area-local `golden` suite
- `profile-policy`: move under `verification/profiles/`
- `global-policy`: move under `verification/policy/`
- `schema`: move under `verification/schemas/`
- `generated-report`: delete from active tree and regenerate under `target/`
- `historical-inventory`: convert to JSON if consumed by a gate, otherwise archive externally or delete
- `obsolete`: delete

Rules:

- `verification/` should not contain loose domain files at the root after cleanup.
- Verification material must live under one area, policy, schema, profile, or runner owner.
- No `golden/` directory may remain outside an owning area.
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
- `diagnostic_codes.md`
- `generated_code_quality.md`
- `hir_maintainability_guardrails.md`
- `performance_budgets.md`
- `sifr_driver_maintainability_guardrails.md`
- `lsp_server.md`
- `tooling_analysis.md`
- `tooling_reuse_strategy.md`
- `tooling_verification.md`
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
- move runner mechanics, schemas, profile data, baseline retention, and generated-artifact policy under `verification/`

Rules:

- Internal docs should explain current architecture, accepted decisions, implemented behavior, durable state, and accepted engineering conventions.
- Internal docs should not tell contributors how to implement a future change. Those instructions belong in `plans/`.
- Internal docs should not contain long validation transcripts, repeated pass/fail logs, or AI-review path ledgers.
- Phase-numbered architecture-transfer notes should either consolidate into a small design narrative or move to an archive directory.
- Absolute personal reference paths must become environment-variable based references, repo-relative references, or explicit external-reference notes.
- Docs that describe planned work but not accepted design belong under `plans/`, not `internal_docs/`.
- Verification convention docs may live in `internal_docs/verification/` when they describe human-facing conventions.
- Verification docs that are part of runner mechanics, schema contracts, profile data, baseline retention, or generated-artifact policy belong under `verification/policy/`, `verification/schemas/`, or a verification area.

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
- Optional heavy corpora are excluded from default create-pr validation unless explicitly selected by profile policy.

## Cargo Lock Policy

`Cargo.lock` is a load-bearing file for this binary-producing compiler workspace and must begin being tracked.

Cleanup must:

- remove any ignore rule that conflicts with tracking `Cargo.lock`
- add `Cargo.lock` to the root tracked set
- document why the compiler workspace tracks the lockfile
- validate that CI and local builds use the same dependency graph
- document that lockfile diffs are contributor-visible dependency changes and must be reviewed intentionally

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

## Discovered Current-State Disposition

This phase is implementation-ready only if the current surfaces have an explicit destination before the first move PR. The following inventory is the source of truth for the cleanup PRs.

Tracked top-level entries discovered:

| Current entry | Tracked state | Final disposition |
| --- | ---: | --- |
| `.cursor/` | 19 files | Keep as portable workflow assets after deleting `.cursor/.rules/` and extra Claude skill variants. |
| `.github/` | 2 files | Keep; retarget validation vocabulary from lane to profile. |
| `.gitignore` | 1 file | Keep; edit for `Cargo.lock`, bytecode/cache/editor-state hygiene, and generated-artifact exclusions. |
| `.gitmodules` | 1 file | Keep; update atomically with submodule moves. |
| `AGENTS.md`, `CLAUDE.md`, `README.md` | 3 files | Keep; update planning, verification, and profile paths atomically with moves. |
| `Cargo.toml`, `sifr.toml`, `LICENSE.md`, `logo.webp` | 4 files | Keep. |
| `Cargo.lock` | ignored/untracked now | Track in the dedicated Cargo lock PR. |
| `crates/`, `demos/`, `docs/`, `lib/`, `third_party/`, `editor_integrations/` | active trees | Keep; only update references and submodule metadata required by moves. |
| `internal_docs/` | 96 files | Keep only current architecture, design state, and conventions; move phases/roadmap/planning and runner policy out. |
| `plans/issues/` | active root issue plus archive | Move under `plans/issues/{active,completed,archive}`. |
| `reviews/` | active root review artifacts plus archive | Move retained artifacts under `plans/reviews/{active,archive}`; delete logs and transcripts without planning value. |
| `audits/` | audit fixtures, reports, scripts, and LeetCode submodule | Remove as a top-level system; promote fixtures/corpora into verification areas or delete/archive reports. |
| `scripts/` | 54 tracked files | Keep only public facade, repo guardrails, release tooling, generators, and maintenance utilities. Move verification implementation into areas. |
| `verification/` | 303 tracked files | Rebuild into `runner`, `schemas`, `profiles`, `areas`, and `policy`. |

Ignored local/generated top-level material discovered and not to be tracked:

| Current entry | Final disposition |
| --- | --- |
| `.DS_Store`, `.claude/`, `.claude.log`, `.obsidian/`, `.sifr_cache/`, `sifr_output/`, `target/`, `tmp/`, `tmp_*` | Delete locally when encountered; keep ignored. |
| `scripts/__pycache__/`, `verification/**/__pycache__/`, `*.pyc` | Delete locally when encountered; guard against tracking. |

Cursor exact disposition:

| Current path | Final disposition |
| --- | --- |
| `.cursor/.rules/architecture-overview.mdc` | Delete; no `.cursor/.rules/` tree remains. |
| `.cursor/commands/*.md` | Keep and retarget from `plans/issues/`, `reviews/`, and `plans/phases/` to `plans/...` after the planning tree exists. |
| `.cursor/references/*.md` | Keep only portable templates/references; remove local path assumptions. |
| `.cursor/skills/project-workflow/`, `.cursor/skills/phase-closure-loop/`, `.cursor/skills/sifr-demo-authoring/` | Keep after replacing personal paths and old planning paths. |
| `.cursor/skills/talk-to-claude-opus/` | Keep as the single Claude/Fable review workflow; make output default to `plans/reviews/active/`. |
| `.cursor/skills/talk-to-claude-default/`, `.cursor/skills/talk-to-claude-gui-review/` | Delete after folding any useful wording into `talk-to-claude-opus`. |
| `.cursor/plans/.obsidian/` and other local state | Delete; no embedded Obsidian or local planning state is tracked. |

Verification exact disposition by current path:

| Current path | Final owner |
| --- | --- |
| `verification/validation_lanes/manifest.json` | Convert to `verification/profiles/{create-pr,merge,nightly,release}.json`; field names become profile-oriented. |
| `verification/validation_lanes/create_pr_e2e_manifest.json`, `verification/validation_lanes/merge_e2e_manifest.json` | Convert to area-owned suite/case selections, primarily `core_language`, with profiles selecting suites instead of owning fixture lists. |
| `verification/suites/manifest.json` | Dissolve into area manifests and profiles; no top-level suite taxonomy remains. |
| `verification/validation_contracts/manifest.json` | Split suites by contract owner: `frontend_mode_parity` and `phase23_graph_isolation` to `project_workspace`; `integer_dtype_contract`, `phase24_hir_analysis`, and `phase25_cfg_flow` to `core_language`. |
| `verification/validation_contracts/integer_dtype_contract.md` | Move to `verification/areas/core_language/data/integer_dtype_contract.md`; the runner validates sentinel text there. |
| `verification/crashes/index.json` | Move to `verification/areas/regression/data/crashes.json`. |
| `verification/fixedbugs/index.json` | Move to `verification/areas/regression/data/fixedbugs.json`. |
| `verification/determinism/manifest.json` | Split into runner determinism self-tests and `verification/policy/`; do not create a `determinism` area. |
| `verification/flake/quarantine.json` | Move to `verification/policy/flake_quarantine.json`. |
| `verification/fuzz_property/*` | Move to `verification/areas/fuzz_property/`; `sustained_lane.md` becomes profile/policy language using profile vocabulary. |
| `verification/generated_code_quality/*` | Move to `verification/areas/generated_code_quality/`; shell helpers become Python area runner helpers unless shell behavior itself is under test. |
| `verification/perf/sifr_int_loop.sifr` | Move to `verification/areas/performance/fixtures/sifr_int_loop.sifr`. |
| `verification/performance/*` | Move to `verification/areas/performance/`. |
| `verification/platform/*` | Move to `verification/areas/runtime_platform/`; platform golden material becomes `kind: "golden"` cases or a `platform_contract` suite. |
| `verification/distribution/*` | Move validation cases and schemas to `verification/areas/distribution_release/`; keep only release-mutating tools under `scripts/distribution/`. |
| `verification/oss/*` | Move to `verification/areas/ecosystem_compatibility/`; curated and broader manifests become profile-selected suites. |
| `verification/package_management/*` | Move to `verification/areas/package_management/`; demo repository submodules remain area-owned corpora/fixtures. |
| `verification/tooling/*` | Move to `verification/areas/developer_tooling/`. |
| `verification/sifr-large-lsp-verification/`, `verification/sifr_large_lsp_verification.md` | Move submodule and its current README/manifest wrapper to `verification/areas/developer_tooling/corpora/sifr-large-lsp-verification/`. |
| `verification/stdlib/*` | Move to `verification/areas/stdlib_parity/`; consumed JSON inventories remain data, while traceability markdown becomes generated reports, compact runbooks, or deleted history. |
| `verification/integer_model_*.md` | Move current integer-model contract material into `core_language` data or `internal_docs/integer_model.md`; delete point-in-time implementation inventories after current state is represented. |

Crate-local verification fixture disposition:

| Current path | Final owner |
| --- | --- |
| `crates/sifr/tests/verification/crashes/` | `verification/areas/regression/fixtures/crashes/`. |
| `crates/sifr/tests/verification/diagnostics/` | `verification/areas/diagnostics/fixtures/` plus diagnostic baselines where expected output is contractual. |
| `crates/sifr/tests/verification/package/` | `verification/areas/package_management/fixtures/`. |
| `crates/sifr/tests/verification/project/` | `verification/areas/project_workspace/fixtures/`. |
| Current crate-private exemptions | None. Every current `crates/sifr/tests/verification/` fixture moves to an owning verification area. |

Audits exact disposition by current directory:

| Current audit material | Count/shape | Final owner |
| --- | ---: | --- |
| `audits/borrowing/` | 50 `.sifr` plus report | `core_language/fixtures/ownership/`; delete report. |
| `audits/lexical_and_syntax/` | 7 `.sifr` plus reports | `core_language/fixtures/syntax/`; delete reports. |
| `audits/type_inference/` | 30 `.sifr` plus reports | `core_language/fixtures/type_inference/`; delete reports. |
| `audits/type_system/` | 41 `.sifr` plus reports | `core_language/fixtures/type_system/`; delete reports. |
| `audits/modules_and_imports/` | 5 `.sifr` plus reports | `project_workspace/fixtures/imports/`; delete reports. |
| `audits/iteration_protocol/` | 5 `.sifr` plus reports | Split by contract into `core_language`, `stdlib_parity`, or `regression`; delete reports after manifest coverage. |
| `audits/object_model/` | 6 `.sifr` plus reports | Split by contract into `core_language`, `stdlib_parity`, or `regression`; delete reports after manifest coverage. |
| `audits/python_basics/` | 45 `.sifr` plus reports | Split by contract into `core_language`, `stdlib_parity`, or `regression`; delete duplicates after manifest coverage. |
| `audits/stdlib/` | 10 `.sifr`, CPython convention doc, reports | Fixtures to `stdlib_parity`; CPython convention to `verification/areas/stdlib_parity/README.md`; reports delete/archive. |
| `audits/leetcode/` | submodule corpus | `verification/areas/algorithmic_compatibility/corpora/leetcode/` with `.gitmodules` updated. |
| `audits/STDLIB_PARITY_MASTER_REPORT.md`, `audits/*/REPORT.md`, `audits/*/POST_HARDENING_REPORT.md` | historical markdown | Delete from active tree after current state is represented by manifests/data/docs. |

Internal docs exact disposition:

| Current path/group | Final disposition |
| --- | --- |
| `plans/roadmap.md` | Move to `plans/roadmap.md` and update links. |
| `plans/phases/*.md` | Move flat into `plans/phases/`; create `plans/phases/index.md`. Preserve existing filenames; the duplicate phase-27 numbering is shown as two indexed phase-27 entries rather than renamed during this cleanup. |
| `internal_docs/verification/artifact_schema_and_retention.md` | Move to `verification/policy/artifact_schema_and_retention.md`. |
| `internal_docs/verification/baseline_governance.md` | Move to `verification/policy/baseline_governance.md`. |
| `internal_docs/verification/deterministic_sharding_and_flake_policy.md` | Move to `verification/policy/deterministic_sharding_and_flake_policy.md`. |
| `internal_docs/verification/fuzz_property_policy.md` | Move to `verification/policy/fuzz_property.md` and update lane wording to profile wording. |
| `internal_docs/verification/oss_gate_policy.md` | Move to `verification/policy/ecosystem_compatibility.md`. |
| `internal_docs/verification/regression_corpus_policy.md` | Move to `verification/policy/regression_corpus.md`. |
| `internal_docs/verification/suite_taxonomy.md` | Delete after the new area/profile README and schemas replace it. |
| `internal_docs/validation_lane_policy.md` | Move to `verification/policy/profile_policy.md` with lane vocabulary removed. |
| `internal_docs/compiler_pipeline.html` | Stop linking as the canonical compiler overview; either regenerate from source into `target/` or move public visualizer material to `docs/`. `README.md` currently links to this file and must be updated in the same PR. |
| `internal_docs/typescript_go_architecture_transfer_m*.md` | Consolidate current accepted state into the existing current-state docs (`frontend_query_architecture.md`, `lsp_server.md`, `tooling_analysis.md`, `tooling_verification.md`, `editor_integrations.md`, `vscode_extension.md`) and delete milestone transfer notes from active `internal_docs/`. |
| `internal_docs/diagnostic_emission_inventory.md` | Keep in `internal_docs/` as current diagnostic emission state after removing validation ledgers and stale milestone prose. |
| `internal_docs/tooling_verification.md` | Keep as current human-facing tooling verification convention if rewritten without validation ledgers; runner-consumed checks move to `verification/areas/developer_tooling/`. |

Stale reference cleanup discovered:

| Reference family | Required update |
| --- | --- |
| `validation_lane`, `validation_lanes`, and user-facing `lane` validation wording in `.github/`, `scripts/`, `README.md`, `AGENTS.md`, `internal_docs/`, and `verification/` | Replace with `profile` vocabulary except where `lane` refers to compiler/LSP worker lanes. |
| `plans/phases/` references in `AGENTS.md`, `README.md`, Cursor commands, scripts, and phase gate checks | Retarget to `plans/phases/`. |
| Root `plans/issues/` references in roadmap, Cursor commands, archive scripts, and active docs | Retarget to `plans/issues/{active,completed,archive}`. |
| Root `reviews/` references and `.claude.log` paths | Retarget retained summaries to `plans/reviews/`; delete log/transcript references that are not retained artifacts. |
| `/Users/yaseralnajjar/...` references in `.cursor/skills/*` and `verification/distribution/common.sh` | Replace with environment variables or repo-relative paths; fail guardrail on future personal paths. |
| `verification/validation_lanes/*` references in scripts and inventories | Replace with `verification/profiles/*` and area-owned suite references. |

## PR 1 Repository Surface And Relevance Audit

This audit is the no-move planning artifact for PR 1. Later PRs must update the rows they complete rather than inventing a second disposition source. "Gate consumed" means a current validation, CI, release, or workflow command directly reads or executes the path today. "Doc referenced" means active docs, workflow files, or issue plans point at the path today. Classification values are `keep`, `move`, `delete`, and `generate`; `generate` is reserved for currently untracked material that must become tracked only through a later explicit generation or lockfile policy step.

The audit scope is tracked repository material plus currently discovered generated or local-state artifacts that sit under active ownership surfaces and need guardrails. Ignored root-level working-tree artifacts such as `.DS_Store`, `.obsidian/`, `.sifr_cache/`, `sifr_output/`, `target/`, and `tmp*` remain covered by the top-level ignored-material disposition table above rather than repeated in every surface table.

### Top-Level Entry Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `.github/` | keep | CI workflows remain top-level repo automation; retarget lane vocabulary to profiles. | Yes | Yes | local facade plus workflow syntax review | CPython keeps CI policy outside source and test data. |
| `.cursor/` | keep | Portable workflow assets only after local state, `.rules`, and extra Claude variants are removed. | Yes | Yes | Cursor hygiene guardrail and create-pr validation | Mature repos keep contributor workflow metadata portable. |
| `.gitignore` | keep | Root hygiene contract; edit for tracked lockfile, bytecode, editor state, and generated reports. | Yes | Yes | git status and hygiene guardrails | Rust treats generated build output as untracked. |
| `.gitmodules` | keep | Submodule ownership registry; update atomically with corpus and integration moves. | Yes | Yes | `git submodule status --recursive` | TypeScript and Rust keep external inputs explicit. |
| `AGENTS.md` | keep | Contributor automation contract; retarget to `plans/` and profiles as moves land. | Yes | Yes | link/reference checks and create-pr validation | CPython separates maintainer guidance from implementation. |
| `CLAUDE.md` | keep | Agent guidance mirrors current repo workflow after path cleanup. | Yes | Yes | stale reference guardrail | Portable workflow docs should not encode local machines. |
| `Cargo.lock` | generate | Currently ignored and untracked, but PR 2 tracks it as the workspace dependency contract. | Yes | Yes | `cargo check --workspace` and create-pr validation | Rust binary workspaces review lockfile diffs. |
| `Cargo.toml` | keep | Workspace manifest remains top-level build contract. | Yes | Yes | cargo check/test | Rust keeps workspace ownership at the root. |
| `LICENSE.md` | keep | Legal root surface. | No | Yes | link check only | Standard project root convention. |
| `README.md` | keep | Public repo entrypoint; update validation and structure references. | Yes | Yes | link/reference checks and create-pr validation | CPython and Rust keep human validation entrypoints current. |
| `logo.webp` | keep | Public branding asset referenced from docs. | No | Yes | asset link check | Public assets stay explicit, not hidden in tooling. |
| `sifr.toml` | keep | Root Sifr project configuration. | Yes | Yes | package/workspace validation | Product repo root should show buildable project intent. |
| `crates/` | keep | Compiler implementation remains the primary source tree. | Yes | Yes | cargo test/build and facade validation | Rust separates compiler crates from tests and tools. |
| `demos/` | keep | User-facing executable examples and milestone demos. | Yes | Yes | demo compilation and create-pr validation | Rust keeps examples separate from compiler tests and benchmarks. |
| `docs/` | keep | Public documentation surface. | No | Yes | link/reference checks | CPython separates public docs from internal docs. |
| `editor_integrations/` | keep | Editor integration ownership surface and submodule parent. | Yes | Yes | tooling checks and submodule status | TypeScript keeps editor tooling contracts explicit. |
| `internal_docs/` | keep | Current architecture and durable conventions only after moving planning/policy out. | No | Yes | link/reference checks | CPython keeps internal docs current, not process history. |
| `plans/issues/` | move | Active and archived issue plans move to `plans/issues/{active,completed,archive}`. | No | Yes | roadmap/Cursor/AGENTS reference checks | Mature repos separate plans from architecture. |
| `reviews/` | move | Retained summaries move to `plans/reviews/`; logs and low-value transcripts delete. | No | Yes | no top-level reviews guardrail | Review process history should not obscure product surface. |
| `audits/` | move | Top-level audit system is dissolved into verification areas or deleted history. | Yes | Yes | area manifest ownership and runner execution | TypeScript keeps fixtures area-owned, not audit-owned. |
| `scripts/` | keep | Public facade, guardrails, generators, release and maintenance tools only. | Yes | Yes | create-pr/merge validation and stale script guardrails | Rust separates test runner/tooling from compiler implementation. |
| `verification/` | keep | Rebuilt as runner, schemas, profiles, areas, and policy. | Yes | Yes | schema validation, area execution, facade validation | TypeScript and Rust make verification inputs first-class. |
| `lib/` | keep | Runtime/library payload remains active top-level product surface. | Yes | Yes | cargo and e2e validation | Product runtime is distinct from repo tooling. |
| `third_party/` | keep | External parser/compiler dependencies remain explicitly vendored/submoduled. | Yes | Yes | submodule status and parser build tests | Rust and CPython isolate external dependencies. |

### Cursor Workflow Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `.cursor/commands/*.md` | keep | Keep commands, but retarget paths only after destination trees exist. | Yes | Yes | Cursor hygiene and stale path guardrails | Contributor workflows should follow the current repo map. |
| `.cursor/references/*.md` | keep | Keep portable templates and field references; remove local assumptions. | Yes | Yes | Cursor hygiene guardrail | Templates are active process docs, not local state. |
| `.cursor/skills/project-workflow/` | keep | Retain workflow skill with `plans/` paths after planning tree exists. | Yes | Yes | workflow reference checks | CPython keeps contributor process docs concise. |
| `.cursor/skills/phase-closure-loop/` | keep | Retain closure loop after replacing absolute Telegram and review-skill assumptions. | Yes | Yes | personal-path guardrail | Workflow automation must be portable. |
| `.cursor/skills/sifr-demo-authoring/` | keep | Retain demo authoring guidance and ensure paths stay repo-relative. | Yes | Yes | demo validation | Demos are product examples, not validation logs. |
| `.cursor/skills/talk-to-claude-opus/` | keep | Single retained Claude/Fable review workflow; retarget output to `plans/reviews/active/`. | Yes | Yes | reviewer handoff smoke check | One review workflow contract avoids process forks. |
| `.cursor/skills/talk-to-claude-default/` | delete | Delete after useful content is folded into the Opus workflow. | Yes | Yes | no duplicate Claude skill guardrail | Avoid model-branded workflow variants. |
| `.cursor/skills/talk-to-claude-gui-review/` | delete | Delete after useful GUI wording is folded into the Opus workflow if needed. | Yes | Yes | no duplicate Claude skill guardrail | Workflow variants should not encode obsolete review paths. |
| `.cursor/.rules/` | delete | Remove obsolete Cursor rules tree. | Yes | No | Cursor hygiene guardrail | Portable repo metadata should be minimal. |
| `.cursor/plans/.obsidian/` | delete | Local Obsidian state is not repo material. | No | No | local-state guardrail | Local editor state belongs outside source control. |
| `.cursor/.DS_Store` | delete | Local macOS artifact. | No | No | local-state guardrail | Generated/editor artifacts are not active tree material. |

### Scripts Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `scripts/run_all_tests.sh` | keep | Only public validation facade; final state delegates to `sifr_verify`. | Yes | Yes | create-pr and merge profile validation | CPython keeps a stable public test facade over runner internals. |
| `scripts/check_file_size_guardrails.py` | keep | Source guardrail for first-party file-size policy. | Yes | Yes | self-test and create-pr validation | Repo hygiene should be enforced by tooling. |
| `scripts/check_hir_maintainability_guardrails.py` | keep | HIR source ownership guardrail. | Yes | Yes | self-test and create-pr validation | Compiler internals need structural guardrails. |
| `scripts/check_sifr_driver_maintainability_guardrails.py` | keep | Driver source ownership guardrail. | Yes | Yes | self-test and create-pr validation | Source layout policy belongs in tools, not convention. |
| `scripts/check_source_crate_dependency_direction.py` | keep | Crate dependency direction guardrail. | Yes | Yes | self-test and create-pr validation | Rust workspaces enforce architecture boundaries. |
| `scripts/check_codegen_rawcode_gate.sh` | keep | Source guardrail unless it grows generated-output inspection. | No | Yes | manual guardrail invocation or future create-pr wiring | Generated-code quality checks belong to area runners when output-facing. |
| `scripts/check_diagnostic_cancel_usage.py` | keep | Diagnostics source hygiene guardrail. | Yes | Yes | create-pr validation | Diagnostics contracts are compiler-source policy. |
| `scripts/check_diagnostic_transport_cleanup.py` | keep | Diagnostics transport source guardrail. | Yes | Yes | create-pr validation | Keep source hygiene separate from output baselines. |
| `scripts/clone_subrepos.sh` | keep | Repository maintenance utility. | No | Yes | submodule status and clone smoke check | External corpora should have explicit restoration. |
| `scripts/generate_unicode_tables.py` | keep | Code generator. | No | Yes | generated output diff check | CPython separates generators from tests. |
| `scripts/distribution/build_preview_artifacts.sh` | keep | Release artifact builder. | No | Yes | release dry-run smoke check | Release tooling is distinct from validation cases. |
| `scripts/distribution/create_new_version.sh` | keep | Release mutation tool. | No | Yes | release dry-run smoke check | Release mutation scripts stay under tooling. |
| `scripts/distribution/generate_dispatchers.sh` | keep | Release code generator. | No | Yes | dispatcher generation diff check | Generated release assets need explicit generators. |
| `scripts/distribution/generate_version_installer.sh` | keep | Release installer generator. | No | Yes | installer generation diff check | Release generation differs from validation. |
| `scripts/distribution/validate_self_update_metadata.sh` | move | Validation gate moves to `distribution_release` if retained as a gate. | Yes | Yes | distribution area runner equivalence | Test cases belong to verification areas. |
| `scripts/check_integer_dtype_contract.py` | move | Move to `core_language` because it validates compiler integer contract data. | Yes | Yes | core_language area runner equivalence | Compiler behavior contracts are area-owned. |
| `scripts/check_package_manager_guardrails.py` | move | Move to `package_management` to validate package boundaries with package fixtures. | Yes | Yes | package_management area execution | Area ownership follows asserted contract. |
| `scripts/run_e2e_pass.sh` | move | Move into `core_language` or delete after profile owns the cases. | Yes | Yes | e2e equivalence and snapshot stability | TypeScript cases belong to test areas. |
| `scripts/run_validation_contract_matrix.sh` | move | Split by contract owner across `core_language`, `project_workspace`, and `diagnostics`. | Yes | Yes | side-by-side matrix equivalence | Suites should follow contract ownership. |
| `scripts/run_phase23_graph_isolation_matrix.sh` | move | Move to `project_workspace`; active filename encodes stale phase number. | Yes | Yes | project_workspace area equivalence | Stable test names should describe behavior. |
| `scripts/run_phase24_hir_analysis_consolidation_matrix.sh` | move | Move to `core_language`; remove phase-numbered entrypoint. | Yes | Yes | core_language area equivalence | Behavior area names outlive phase numbers. |
| `scripts/run_phase25_cfg_flow_activation_matrix.sh` | move | Move to `core_language`; remove phase-numbered entrypoint. | Yes | Yes | core_language area equivalence | Mode-specific suites belong under owning areas. |
| `scripts/run_frontend_mode_parity_matrix.sh` | move | Move to `project_workspace` as frontend/project mode parity. | Yes | Yes | project_workspace area equivalence | Workspace behavior should be area-owned. |
| `scripts/run_platform_golden.sh` | move | Move to `runtime_platform`; delete old wrapper after cutover. | Yes | Yes | runtime_platform golden case execution | Golden cases still need one owning area. |
| `scripts/run_smoke_fuzz_property.sh` | move | Move to `fuzz_property`. | Yes | Yes | fuzz_property smoke equivalence | Fuzz/property checks need resource policy. |
| `scripts/run_integer_model_closure_perf.py` | move | Move to `performance` with explicit budget policy. | Yes | Yes | performance runner budget check | Benchmarks are distinct from correctness gates. |
| `scripts/ci_e2e_throughput.sh` | move | Move to `performance`; CI wrapper name is not a public gate. | No | Yes | performance runner equivalence if retained | Performance validation needs profile policy. |
| `scripts/check_codegen_binary_size.sh` | move | Move to `performance` as binary-size budget validation. | No | Yes | performance budget execution if retained | Binary size is a budget, not source hygiene. |
| `scripts/check_diagnostic_baseline_hygiene.py` | move | Move to `diagnostics`. | Yes | Yes | diagnostics area execution | Baselines belong to output contract owners. |
| `scripts/check_diagnostic_code_coverage.py` | move | Move to `diagnostics`. | Yes | Yes | diagnostics area execution | Diagnostic coverage is an area contract. |
| `scripts/check_diagnostic_docs_sync.py` | move | Move to `diagnostics`. | Yes | Yes | diagnostics area execution | Docs/schema sync belongs with diagnostics policy. |
| `scripts/check_diagnostic_schema_sync.py` | move | Move to `diagnostics`. | Yes | Yes | diagnostics area execution | Schema sync should be area-owned. |
| `scripts/run_distribution_validation.sh` | move | Move executable validation cases to `distribution_release`. | Yes | Yes | distribution_release runner equivalence | Release validation cases are fixtures. |
| `scripts/run_stdlib_namespace_corpus_validation.py` | move | Move to `stdlib_parity`. | Yes | Yes | stdlib_parity runner equivalence | CPython-observable behavior belongs to stdlib parity. |
| `scripts/check_phase30_complexity_resource_inventory.py` | move | Move to `stdlib_parity`; stale phase number removed. | Yes | Yes | stdlib_parity inventory validation | Resource parity data belongs to the area. |
| `scripts/build_full_corpus_failure_taxonomy.py` | move | Move to `algorithmic_compatibility` as taxonomy builder. | No | Yes | taxonomy generation check | External corpus results are area artifacts. |
| `scripts/generate_concurrency_runtime_inventory.py` | move | Move to `stdlib_parity` if the output remains verification inventory. | No | Yes | inventory generation diff check | Verification inventories belong with area data. |
| `scripts/run_verification_hardening.py` | move | Split across regression, fuzz_property, ecosystem_compatibility, and runner self-tests. | Yes | Yes | per-area equivalence and runner self-tests | Generic hardening runners hide ownership. |
| `scripts/run_verification_hardening/` | move | Split module implementation with the same owners as the facade above. | Yes | Yes | per-area equivalence | Harness code should live under runner or areas. |
| `scripts/check_e2e_report_determinism.sh` | move | Move to runner self-tests and policy data. | Yes | Yes | runner determinism self-test | Determinism is runner behavior, not a domain area. |
| `scripts/check_e2e_sequential_parallel_equivalence.sh` | move | Move to runner self-tests and policy data. | Yes | Yes | sequential/parallel equivalence self-test | Scheduling policy belongs to the runner. |
| `scripts/validation_lane.py` | move | Replace with `sifr_verify.profiles`; delete old name in PR 7. | Yes | Yes | profile shell export equivalence | Profiles are runner policy, not scripts. |
| `scripts/validation_lane_report.py` | move | Replace with `sifr_verify` report handling; delete old name in PR 7. | Yes | Yes | report shape equivalence | Generated reports belong under runner/target. |
| `scripts/archive_issues.sh` | move | Retarget to `plans/issues/` or delete if no longer useful. | No | Yes | dry-run path check | Planning lifecycle tools should follow plan tree. |
| `scripts/archive_reviews.sh` | move | Retarget to `plans/reviews/` or delete if no longer useful. | No | Yes | dry-run path check | Review archives belong under plans. |
| `scripts/archive_reviews_and_issues.sh` | move | Retarget to `plans/` lifecycle or delete if redundant. | No | Yes | dry-run path check | Avoid duplicate lifecycle entrypoints. |
| `scripts/phase_contract_gate_check.py` | delete | Replace only with current `plans/phases/index.md` consistency guardrail if useful. | Yes | Yes | new phase-index guardrail self-test | Stable phase indexes beat phase-numbered assumptions. |
| `scripts/validate_phase15_backlog.py` | delete | Historical phase-15 backlog guardrail; no active destination. | Yes | Yes | no stale references | Historical phase tooling should not survive. |
| `scripts/__pycache__/` | delete | Python bytecode is generated local output and must remain untracked. | No | No | bytecode/cache guardrail | Generated caches are not source. |
| `scripts/run_verification_hardening/__pycache__/` | delete | Python bytecode is generated local output and must remain untracked. | No | No | bytecode/cache guardrail | Generated caches are not source. |

### Verification Surface Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `verification/validation_lanes/` | move | Convert to `verification/profiles/` and profile schema data. | Yes | Yes | profile schema validation and facade equivalence | Profile policy selects areas, not fixtures. |
| `verification/suites/manifest.json` | move | Dissolve into area manifests and profiles. | Yes | Yes | area/profile schema validation | Suites are area-local like Rust mode families. |
| `verification/validation_contracts/` | move | Split contracts to `core_language`, `project_workspace`, and `diagnostics`. | Yes | Yes | per-area contract execution | Contracts belong to behavior owners. |
| `verification/crashes/index.json` | move | Move to `regression/data/crashes.json`. | Yes | Yes | regression area manifest validation | Fixed crashes are regression data. |
| `verification/fixedbugs/index.json` | move | Move to `regression/data/fixedbugs.json`. | Yes | Yes | regression area manifest validation | Fixed bugs are minimized regression cases. |
| `verification/determinism/manifest.json` | move | Split into runner self-tests and policy data; no determinism area. | Yes | Yes | runner determinism self-test | Scheduling determinism is runner policy. |
| `verification/flake/quarantine.json` | move | Move to `verification/policy/flake_quarantine.json`. | Yes | Yes | policy schema validation | Flake policy is global runner policy. |
| `verification/fuzz_property/` | move | Move under `areas/fuzz_property/`; replace lane wording with profiles. | Yes | Yes | fuzz_property runner execution | Fuzz corpora need resource class policy. |
| `verification/generated_code_quality/` | move | Move under `areas/generated_code_quality/`; rewrite helpers unless shell is tested. | Yes | Yes | generated-code-quality runner equivalence | Generated output quality is its own contract. |
| `verification/perf/` | move | Move fixture to `areas/performance/fixtures/`. | Yes | Yes | performance runner execution | Benchmarks need explicit budget ownership. |
| `verification/performance/` | move | Move under `areas/performance/`. | Yes | Yes | performance schema and budget checks | Bun separates benchmarks from correctness. |
| `verification/platform/` | move | Move under `areas/runtime_platform/`; golden fixtures become owned cases. | Yes | Yes | runtime_platform golden execution | Golden baselines remain area-owned. |
| `verification/distribution/` | move | Move validation cases and schema under `areas/distribution_release/`. | Yes | Yes | distribution_release runner equivalence | Release validation cases are fixtures. |
| `verification/oss/` | move | Move manifests to `areas/ecosystem_compatibility/`. | Yes | Yes | ecosystem area execution | External compatibility should be profile-selected. |
| `verification/package_management/` | move | Move under `areas/package_management/`. | Yes | Yes | package_management runner execution | Package fixtures belong to package area. |
| `verification/tooling/` | move | Move under `areas/developer_tooling/`. | Yes | Yes | developer_tooling runner execution | Tooling checks have their own ownership. |
| `verification/sifr-large-lsp-verification/` | move | Move submodule to `developer_tooling/corpora/`. | Yes | Yes | submodule status and tooling runner | Large editor corpora should be area-owned. |
| `verification/sifr_large_lsp_verification.md` | move | Move wrapper/runbook with the developer tooling corpus or delete stale content. | No | Yes | link/reference check | Corpus docs travel with the corpus owner. |
| `verification/stdlib/` | move | Move to `areas/stdlib_parity/`; delete generated traceability reports when represented. | Yes | Yes | stdlib_parity schema/data validation | CPython parity data needs area ownership. |
| `verification/integer_model_*.md` | move | Move current contract data to `core_language` or consolidate into `internal_docs/integer_model.md`. | Yes | Yes | core_language data validation and doc link check | Compiler semantics data belongs to core language. |
| `verification/.DS_Store` | delete | Local macOS artifact. | No | No | local-state guardrail | Editor artifacts are not source. |
| `verification/**/__pycache__/`, `verification/**/*.pyc` | delete | Generated Python bytecode. | No | No | bytecode/cache guardrail | Generated caches are not verification assets. |
| `crates/sifr/tests/verification/crashes/` | move | Move fixtures to `areas/regression/fixtures/crashes/`. | Yes | Yes | Rust harness path update and regression runner | Repo-level fixtures should not hide under a crate. |
| `crates/sifr/tests/verification/diagnostics/` | move | Move fixtures and baselines to `areas/diagnostics/`. | Yes | Yes | diagnostics harness and baseline stability | Baselines are first-class area contracts. |
| `crates/sifr/tests/verification/package/` | move | Move fixtures to `areas/package_management/fixtures/`. | Yes | Yes | package tests and area runner | Ownership follows package behavior. |
| `crates/sifr/tests/verification/project/` | move | Move fixtures to `areas/project_workspace/fixtures/`. | Yes | Yes | project tests and area runner | Workspace behavior belongs to workspace area. |

### Audits Surface Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `audits/borrowing/` | move | Promote `.sifr` fixtures to `core_language/fixtures/ownership/`; delete report. | Yes | Yes | core_language runner ownership check | Compiler semantics fixtures are area-owned. |
| `audits/lexical_and_syntax/` | move | Promote fixtures to `core_language/fixtures/syntax/`; delete reports. | Yes | Yes | parser/syntax suite execution | Syntax cases should be first-class test inputs. |
| `audits/type_inference/` | move | Promote fixtures to `core_language/fixtures/type_inference/`; delete reports. | Yes | Yes | type inference suite execution | Cases should outlive audit reports. |
| `audits/type_system/` | move | Promote fixtures to `core_language/fixtures/type_system/`; delete reports. | Yes | Yes | type system suite execution | Behavior fixtures belong with the compiler contract. |
| `audits/modules_and_imports/` | move | Promote fixtures to `project_workspace/fixtures/imports/`; delete reports. | Yes | Yes | project_workspace imports suite | Module graph behavior belongs to workspace area. |
| `audits/iteration_protocol/` | move | Split fixtures by asserted contract across core_language, stdlib_parity, or regression. | Yes | Yes | manifest ownership and runner execution | Ownership follows the behavior asserted. |
| `audits/object_model/` | move | Split fixtures by asserted contract across core_language, stdlib_parity, or regression. | Yes | Yes | manifest ownership and runner execution | Avoid generic audit buckets. |
| `audits/python_basics/` | move | Split syntax/type/control-flow to core_language, stdlib behavior to stdlib_parity, fixed bugs to regression. | Yes | Yes | coverage map and duplicate pruning | CPython-observable behavior belongs to parity suites. |
| `audits/stdlib/` | move | Promote fixtures to stdlib_parity and move fixture format guidance into its runbook. | Yes | Yes | stdlib_parity runner execution | Area runbooks should live with fixtures. |
| `audits/leetcode` | move | Move submodule to `algorithmic_compatibility/corpora/leetcode/`. | Yes | Yes | `.gitmodules` and corpus runner | External corpora need one ownership model. |
| `audits/lint_panic_patterns.sh` | move | Replace under `generated_code_quality`. | Yes | Yes | generated-code-quality panic scan equivalence | Generated runtime panic policy belongs to codegen quality. |
| `audits/run_audit.sh` | delete | Delete after area runners own equivalent fixtures. | Yes | Yes | no stale references and area equivalence | Public wrappers should not duplicate runner ownership. |
| `audits/run_audit_fast.sh` | delete | Delete after area runners own equivalent fixtures. | Yes | Yes | no stale references and area equivalence | Compatibility wrappers are forbidden. |
| `audits/run_borrowing_audit.sh` | delete | Delete after core_language ownership suite executes borrowing fixtures. | Yes | Yes | core_language borrowing suite | Old audit-specific wrappers are obsolete. |
| `audits/STDLIB_PARITY_MASTER_REPORT.md` | delete | Historical report; current state must live in manifests/data/docs. | No | Yes | no active report reference | Generated reports belong under `target/`. |
| `audits/*/REPORT.md`, `audits/*/POST_HARDENING_REPORT.md` | delete | Historical reports; delete from active tree after current state is represented. | No | Yes | no active report reference | History stays in git unless a current runbook is needed. |
| `audits/.DS_Store` | delete | Local macOS artifact. | No | No | local-state guardrail | Local editor artifacts are not source. |

### Internal Docs And Planning Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `internal_docs/architecture.md` | keep | Current architecture entrypoint; update references as surfaces move. | No | Yes | link/reference checks | Architecture docs describe current state. |
| `internal_docs/*_architecture.md`, model/design docs | keep | Keep when they describe implemented architecture and accepted conventions; enumerate this grouped row during PR 5 relevance cleanup. | No | Yes | stale status cleanup and link checks | CPython separates current design from plans. |
| `internal_docs/diagnostic_codes.md` | keep | Durable diagnostic code contract. | Yes | Yes | diagnostics docs/schema checks | Diagnostics need source-of-truth docs. |
| `internal_docs/diagnostic_emission_inventory.md` | keep | Keep after removing validation ledgers and stale milestone prose. | Yes | Yes | diagnostics reference checks | Inventories are retained only when current. |
| `internal_docs/tooling_verification.md` | keep | Keep as human-facing tooling verification convention after ledger cleanup. | Yes | Yes | developer_tooling reference checks | Human conventions can stay outside runner data. |
| `plans/roadmap.md` | move | Move to `plans/roadmap.md`. | No | Yes | roadmap link checks | Execution plans should not live in architecture docs. |
| `plans/phases/*.md` | move | Move flat to `plans/phases/` and add `plans/phases/index.md`. | No | Yes | phase index/link checks | Stable phase files belong to planning. |
| `plans/issues/active/ad-hoc-repository-architecture-and-verification-surface-cleanup.md` | move | Active phase plan moves to `plans/issues/active/`. | No | Yes | AGENTS/Cursor/roadmap references | Active plans live under plans. |
| `plans/issues/active/ad-hoc-serious-build-output-and-phase-timings.md` | move | Completed or active status determines `plans/issues/completed/` or `active/`. | No | Yes | issue index checks | Issue lifecycle should be explicit. |
| `issues/archive/` | move | Move to `plans/issues/archive/`. | No | Yes | archive link checks | Historical plans should not sit at root. |
| `reviews/*.md` | move | Retain active summaries under `plans/reviews/active/`; archive historical value under `archive/`. | No | Yes | review path reference checks | Review artifacts belong with plans. |
| `reviews/*.claude.log`, `reviews/**/*.stderr.log` | delete | Point-in-time process logs are not active repo material. | No | Yes | no log references | Logs are generated artifacts. |
| `reviews/archive/` | move | Move retained archival reviews to `plans/reviews/archive/`. | No | Yes | archive link checks | Archives should not be top-level surfaces. |
| `internal_docs/verification/artifact_schema_and_retention.md` | move | Move to `verification/policy/artifact_schema_and_retention.md`. | Yes | Yes | policy link checks | Runner artifact policy belongs under verification. |
| `internal_docs/verification/baseline_governance.md` | move | Move to `verification/policy/baseline_governance.md`. | Yes | Yes | policy link checks | Baseline governance is verification policy. |
| `internal_docs/verification/deterministic_sharding_and_flake_policy.md` | move | Move to `verification/policy/deterministic_sharding_and_flake_policy.md`. | Yes | Yes | runner policy validation | Sharding/flake policy is runner-owned. |
| `internal_docs/verification/fuzz_property_policy.md` | move | Move to `verification/policy/fuzz_property.md` with profile vocabulary. | Yes | Yes | policy wording and link checks | Resource-heavy fuzzing needs profile policy. |
| `internal_docs/verification/oss_gate_policy.md` | move | Move to `verification/policy/ecosystem_compatibility.md`. | Yes | Yes | ecosystem policy link checks | External corpora need explicit policy. |
| `internal_docs/verification/regression_corpus_policy.md` | move | Move to `verification/policy/regression_corpus.md`. | Yes | Yes | regression policy link checks | Regression corpus rules belong with verification. |
| `internal_docs/verification/suite_taxonomy.md` | delete | Delete after area/profile README and schemas replace it. | No | Yes | no stale references | Top-level suite taxonomy conflicts with area ownership. |
| `internal_docs/validation_lane_policy.md` | move | Move to `verification/policy/profile_policy.md` and remove lane vocabulary. | Yes | Yes | profile policy link checks | Profiles are runner policy. |
| `internal_docs/compiler_pipeline.html` | delete | Regenerate into `target/` or replace README links with source docs. | No | Yes | README link check | Generated visualizations should not be canonical docs. |
| `internal_docs/typescript_go_architecture_transfer_m*.md` | delete | Consolidate accepted state into current docs, then delete milestone transfer notes. | No | Yes | current-doc link checks | Current docs should not be transfer ledgers. |
| `internal_docs/.DS_Store` | delete | Local macOS artifact. | No | No | local-state guardrail | Editor artifacts are not source. |

### Submodule Audit

| Current path | Classification | Destination or deletion rationale | Gate consumed | Doc referenced | Validation required after changing | Reference-compiler lesson |
| --- | --- | --- | --- | --- | --- | --- |
| `third_party/ruff` | keep | Parser fork remains under external compiler dependency owner. | Yes | Yes | submodule status and parser tests | External compiler dependencies stay separate. |
| `editor_integrations` | keep | Editor integration repository remains under editor surface. | Yes | Yes | submodule status and tooling checks | Editor integrations are separate product surfaces. |
| `editor_integrations/vscode` | keep | VS Code extension submodule remains under editor integrations. | Yes | Yes | submodule status and extension checks | Editor-specific assets stay under editor owner. |
| `audits/leetcode` | move | Move to algorithmic compatibility corpus owner. | Yes | Yes | `.gitmodules`, clone script, area runner | External corpora need one owning area. |
| `verification/sifr-large-lsp-verification` | move | Move to developer tooling corpus owner. | Yes | Yes | `.gitmodules`, submodule status, tooling runner | Large LSP corpora belong to tooling validation. |
| `verification/package_management/demo_repositories/*` | move | Remain submodules but under package_management area-owned corpora/fixtures. | Yes | Yes | `.gitmodules`, package runner | Demo repos are package verification inputs. |

## Proposed PR Sequence

### PR 1: Repository Surface And Relevance Inventory

Add the full tracked-entry disposition tables and relevance audit. No moves, rewrites, or deletions in this PR.

- top-level entries
- `scripts/` entries
- `verification/` entries
- `audits/` entries
- `internal_docs/` entries
- submodules
- Cursor workflow files
- plan, issue, phase, and review artifact locations

Each row must include:

- current path
- classification
- destination or deletion rationale
- whether a gate consumes it
- whether a current doc references it
- validation required after changing it
- reference-compiler lesson when one applies

Validation:

- `git diff --check`
- `python3 scripts/check_file_size_guardrails.py`
- no file moves
- no deletions

### PR 2: Cargo Lock Policy

Begin tracking `Cargo.lock` and fix ignore rules.

Validation:

- `cargo check --workspace`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 3: Cursor Portability Cleanup

Remove local-machine assumptions from `.cursor/` without retargeting commands to paths that do not exist yet.

Scope:

- remove personal absolute paths
- replace the external `talk-to-claude` absolute path with `TALK_TO_CLAUDE_PROJECT` and a fail-fast message when unset
- remove `.cursor/.rules/`
- remove `.DS_Store`, Obsidian state, and local editor artifacts
- consolidate Claude/Fable review workflow into `.cursor/skills/talk-to-claude-opus/`
- remove the other Claude review skill variants after folding in useful instructions
- disposition `.cursor/plans/` content as move-later inventory or delete-obsolete local state

Validation:

- Cursor hygiene guardrail
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 4: Review Tree Normalization

Move intentionally retained review artifacts under `plans/reviews/active/` or `plans/reviews/archive/`. Delete review artifacts that have no ongoing planning value and replace direct transcript ledgers in active docs with concise summaries.

This PR creates `plans/reviews/{active,archive}/` and retargets `.cursor/skills/talk-to-claude-opus/` to write new review artifacts to `plans/reviews/active/` before deleting the root `reviews/` tree.

Validation:

- no top-level tracked `reviews/`
- retained review artifacts live under `plans/reviews/`
- no active doc depends on review transcript paths
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 5: Planning Tree Normalization

Move active, completed, and archived phase and issue docs into the new lifecycle directories. Retarget AGENTS and Cursor commands atomically with the new paths.

Validation:

- all roadmap links resolve
- all AGENTS/Cursor references resolve
- `git diff --check`
- `scripts/run_all_tests.sh --profile create-pr`

### PR 6: Verification Runner Foundation

Add the `uv`-managed verification runner, schemas, area discovery, and result format.

No existing runner migration yet.

Also add the migration-status table used while old and new verification surfaces temporarily coexist. The table lives in this issue plan after it moves to `plans/issues/active/`. It must track area, legacy path, new area path, current authoritative gate, equivalence evidence, and cutover status.

This PR also pins `requires-python`, documents the minimum supported `uv` version, adds the `uv` availability/version fail-fast to the facade, updates CI to install the pinned `uv` version before running the same facade, and defines the profile schema entries for selected areas, toolchain step sets, and guardrail step sets.

Validation:

- schema self-tests
- runner discovery self-test
- `uv` lockfile/check workflow
- resource class selection self-test
- resume/failure-reproduction self-test
- `scripts/run_all_tests.sh --profile create-pr`

### PR 7: Verification Profile Normalization

Split profile configuration into `verification/profiles/*.json` and validate with schemas.

Scope:

- convert `verification/validation_lanes/manifest.json` into `verification/profiles/*.json`
- rewrite facade profile resolution so `scripts/run_all_tests.sh` obtains shell exports from `uv run --project verification python -m sifr_verify profiles shell --profile <profile>`
- move report summarization from `scripts/validation_lane_report.py` into `sifr_verify` profile report handling
- delete `scripts/validation_lane.py`, `scripts/validation_lane_report.py`, and `verification/validation_lanes/manifest.json`
- keep the rest of the facade orchestration body as legacy bash until the facade cutover PR

Validation:

- all profiles schema-valid
- old validation manifest removed
- `scripts/run_all_tests.sh --profile create-pr`

### PR 8-N: Verification Area Corpus Migration

Migrate one verification area per PR into `verification/areas/<area>/`.

The migration order intentionally starts with smaller manifest-backed areas before `core_language`, because `core_language` has the largest fixture and snapshot blast radius.

Migration order:

1. `diagnostics`
2. `project_workspace`
3. `core_language`
4. `regression`
5. `fuzz_property`
6. `generated_code_quality`
7. `performance`
8. `developer_tooling`
9. `runtime_platform`
10. `distribution_release`
11. `package_management`
12. `stdlib_parity`
13. `algorithmic_compatibility`
14. `ecosystem_compatibility`

Validation for each area migration:

- area manifest schema-valid
- moved fixtures are owned by exactly one area
- crate-local `crates/sifr/tests/verification/` fixtures are moved or explicitly exempted according to the crate-fixture disposition rule
- legacy facade dispatch is updated atomically with moved paths
- shell verification helpers rewritten to Python under the verification `uv` project unless shell is the artifact under test
- old verification script names deleted, not wrapped
- runner can execute the area
- compiler-output determinism check for affected area
- sequential/parallel equivalence where applicable
- snapshot stability verified after fixture moves, including declaration-order expectations
- migration-status table updated
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+1: Verification Facade Cutover

Rewrite `scripts/run_all_tests.sh` into a thin public facade over `sifr_verify`.

Validation:

- side-by-side equivalence evidence per profile: checks executed, exit-code behavior, timeout behavior, output normalization, and report shape
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh --profile merge`

### PR N+2: Submodule Normalization

Resolve submodule ownership and move any submodule paths that belong under verification areas.

Validation:

- `.gitmodules` is correct
- clone/restoration scripts are correct
- CI checkout still initializes required submodules
- affected area runner executes
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+3: Scripts Verification Sweep

Prove no verification implementation remains in `scripts/` after the area migrations and add the guardrail that keeps it that way. Delete any missed old entrypoints. Do not add compatibility wrappers.

Validation:

- no verification implementation remains in `scripts/`
- no stale script references remain
- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh --profile merge`

### PR N+4: Audits Normalization

Promote retained audit fixtures into verification manifests and remove historical report markdown from the active tree.

Validation:

- no top-level `audits/`
- every retained audit fixture is manifest-owned
- audit area executes through the runner
- `scripts/run_all_tests.sh --profile create-pr`

### PR N+5: Internal Docs Relevance Cleanup

Move, consolidate, or delete outdated internal docs according to the relevance audit.

Validation:

- roadmap links resolve
- phase roadmap links resolve
- verification policy docs live under `verification/policy/`
- no active internal doc contains long validation transcript ledgers
- `git diff --check`

### PR N+6: Docs And Guardrails Closeout

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
- Profile files reference areas instead of duplicating fixture ownership.
- No committed validation result logs exist outside explicit baselines.
- Markdown under `verification/` is limited to README, policy, and runbooks.
- CI and local validation use the same runner and profile files.
- Migration PRs prove determinism and sequential/parallel equivalence where applicable.
- Final closeout passes local merge validation.

## Risks

- Moving fixtures can reorder e2e discovery or invalidate snapshots.
- Removing review transcripts can break historical issue references unless summaries replace them.
- Moving submodules requires atomic `.gitmodules`, script, CI, docs, and manifest updates.
- A Python runner can drift if dependencies grow casually; require every dependency to be declared, locked, reviewed, and justified by verification value.
- Parallel scheduling can hide resource contention unless every area declares resource policy.
- Historical docs may contain the only explanation for a design decision; archive by value, not by age.

## Review Notes

This plan was reviewed with the Fable high model. The main findings incorporated here were:

- the top-level contract must include load-bearing entries such as `lib/`, `.github/`, `sifr.toml`, and `logo.webp`
- corpus migration and submodule migration must be explicit PRs, not risks hidden inside script movement
- the personal-path guardrail must distinguish workflow/code leaks from intentional test fixture paths
- `Cargo.lock` tracking deserves its own policy step
- Python verification tooling should be managed by `uv`, with dependencies declared and locked when needed
- `internal_docs/verification/` should move to `verification/policy/` instead of remaining as a temporary half-state
- `run_all_tests.sh` needs an explicit facade-cutover PR with profile-by-profile equivalence evidence
- `crates/sifr/tests/verification/` must be dispositioned as area-owned repo verification or crate-local unit fixture data
- schema validation must be an explicit supported subset, not an implied full JSON Schema implementation
- runner determinism and sequential/parallel equivalence belong to runner self-tests and policy data, not a permanent verification area
- the phase should borrow reference compiler invariants from TypeScript, typescript-go, Rust, Bun, and CPython: first-class baselines, mode-specific suites, resource-aware runners, distinct benchmark policy, and current-state internal docs
- verification execution selectors should be called `profiles`, not `lanes`, because the public facade already uses `--profile` and `lane` has a separate compiler/SIMD meaning
- the top-level contract must include `.gitignore` and `.gitmodules`
- profile normalization must rewrite facade profile resolution before deleting the legacy validation-lane manifest
- the verification runner needs schema-level `toolchain` and `guardrail` step kinds so cargo tests, fmt/clippy, and repo guardrails stay in validation without raw shell in profiles
- `uv` needs explicit bootstrap, minimum-version, Python-version, and CI installation policy
- review-tree normalization must retarget `talk-to-claude-opus` before deleting the root `reviews/` tree
- final Fable review pass 4 returned `PASS` after these blockers were incorporated
