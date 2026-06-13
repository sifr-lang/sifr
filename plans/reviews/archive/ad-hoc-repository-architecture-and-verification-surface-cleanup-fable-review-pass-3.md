# Fable Review — Pass 3: Ad Hoc Repository Architecture And Verification Surface Cleanup

Reviewed file: `issues/ad-hoc-repository-architecture-and-verification-surface-cleanup.md`
Reviewer: Fable (Claude Fable 5)
Verdict: **REQUEST CHANGES**

The plan is close. The disposition tables were verified against the live repository and are accurate and complete where they exist: every one of the 55 tracked `scripts/` files has a disposition row, all `verification/` subtrees and loose root files are covered, the four `crates/sifr/tests/verification/` subdirectories match the crate-fixture table, the duplicate phase-27 numbering is real and handled, `internal_docs/verification/` contains exactly the seven files dispositioned, and the personal-path findings in `.cursor/skills/*` and `verification/distribution/common.sh` are confirmed. The area model, the profile vocabulary, the schema-subset rule, and the golden-as-case-kind normalization are all sound.

Five blocking gaps remain. Four are sequencing/ownership holes that an implementation PR would hit mid-flight — exactly the "hidden unknowns" this phase is supposed to eliminate — and one is a contract inconsistency that makes an acceptance criterion unsatisfiable as written.

## 1. Blocking issues

### B1. Target Top-Level Contract omits `.gitignore` and `.gitmodules`

Both are tracked, load-bearing top-level files today (`git ls-files` confirms), and the plan itself depends on both: the Cargo Lock PR edits `.gitignore`, and the Submodule Policy section requires `.gitmodules` correctness. But the contract block in "Target Top-Level Contract" lists neither, while the acceptance criterion says "Fresh clone top-level tree matches the top-level contract." As written, the criterion is unsatisfiable (or perversely demands deleting `.gitmodules` with nine live submodules).

Fix: add `.gitignore` and `.gitmodules` to the contract block, and add a row for each to the "Tracked top-level entries discovered" table (`keep`; `.gitignore` is edited by the Cargo lock PR and the bytecode/editor-state guardrails).

### B2. PR 7 removes the old validation manifest while the legacy facade still reads it

`scripts/run_all_tests.sh` resolves all profile policy by `eval`-ing the output of `python3 scripts/validation_lane.py shell --profile <p>`, which reads `verification/validation_lanes/manifest.json` (verified live: it exports `RESOLVED_PROFILE`, budgets, `CONTRACT_SUITES`, `TOOLING_SUITES`, etc.). PR 7's validation says "old validation manifest removed", but the facade cutover is PR N+1 — many PRs later. Between PR 7 and PR N+1 nothing in the plan states what the still-legacy bash facade reads for profile policy. The script disposition row ("`validation_lane.py` … delete old names after PR 7") hints at the answer but never states it.

Fix: make PR 7's scope explicit. Proposed wording for PR 7:

> Scope: convert `verification/validation_lanes/manifest.json` into `verification/profiles/*.json`; rewrite the facade's profile resolution so `run_all_tests.sh` obtains its shell exports from the new profile files (via `uv run --project verification python -m sifr_verify profiles shell --profile <p>`); delete `scripts/validation_lane.py` and `scripts/validation_lane_report.py` in this PR, with report summarization moving to `sifr_verify` profile report handling. The facade's orchestration body remains legacy bash until PR N+1; only its policy source changes.

(If report handling is too heavy for PR 7, keep `validation_lane_report.py` until PR N+1 — but then say so; the current "after PR 7" is exactly the ambiguity an implementer should not have to resolve.)

### B3. Toolchain gates and repo guardrails have no owner in the end-state execution model

The merge profile today runs, inside `run_all_tests.sh`: `cargo test` across ~12 crates (`run_crate_tests`), plus `run_core_guardrails` (file-size, HIR/driver maintainability, dependency direction, package-manager guardrails — all `scripts/check_*`). The end state requires: (a) `run_all_tests.sh` is "a thin profile dispatcher over `sifr_verify`"; (b) profiles "may not define … one-off shell commands"; (c) the boundary rule says guardrails are *not* verification and stay in `scripts/`; (d) the area table assigns no owner to Rust workspace unit tests, clippy, or fmt. These four constraints are jointly unsatisfiable: after cutover, either crate tests and guardrails silently fall out of the gates (violating the "No test coverage reduction" non-goal) or profiles smuggle them in as the shell commands they are forbidden to define.

Fix: define the missing concept before PR 6, because the profile/area schemas depend on it. Recommended shape, consistent with the existing principles:

- Add two runner-owned step kinds to the profile schema: `toolchain` steps (cargo test/clippy/fmt with declared crate scope, executed by `sifr_verify` itself) and `guardrail` steps (entries resolved from a committed registry such as `verification/policy/guardrails.json` that maps stable guardrail names to `scripts/` entrypoints). Profiles then select guardrail/toolchain step sets by name, the same way they select areas — they still never define raw shell.
- Update the "Profile files may define" list to include "toolchain step selection" and "guardrail step selection", and add one sentence to the Scripts Cleanup boundary rule: "Guardrails remain implemented in `scripts/`, but are invoked by `sifr_verify` through the guardrail registry so the facade stays a thin dispatcher."

A `workspace_toolchain` pseudo-area is the alternative, but it muddies "areas own fixtures"; the registry/step-kind shape is cleaner.

### B4. uv has no bootstrap, version, or CI availability story

`.github/workflows/local-first-validation.yml` runs only `bash scripts/run_all_tests.sh --profile …`; nothing installs uv. The plan mandates uv everywhere but never states: minimum uv version, `requires-python` pin in `verification/pyproject.toml`, how CI gets uv, or what a developer without uv sees. Given AGENTS.md's "CI mirrors these exact scripts — no CI-only behavior", the availability check must live in the repo, not the workflow.

Fix: add to the Verification Architecture rules and PR 6 scope:

- `verification/pyproject.toml` pins `requires-python` and the repo documents a minimum uv version in `verification/README.md`.
- The facade fail-fasts with an actionable message when `uv` is missing or below the minimum ("install via `curl -LsSf https://astral.sh/uv/install.sh | sh` or `brew install uv`").
- PR 6 updates `.github/workflows/local-first-validation.yml` to install the pinned uv version (the one permitted CI-only step is tool installation, which should be stated explicitly so it doesn't read as a violation of the no-CI-only-behavior rule).

### B5. PR 4 deletes top-level `reviews/` while the review skill still writes there

`.cursor/skills/talk-to-claude-opus/SKILL.md` writes review conversations to `${PWD_NOW}/reviews/<file>.md`. PR 3 deliberately avoids retargeting to not-yet-existing paths; PR 4 removes the top-level `reviews/` tree but its scope says nothing about the skill's output path; the Cursor retarget is PR 5. So between PR 4 and PR 5, every review run recreates the banned top-level `reviews/` tree — directly violating a PR 4 acceptance check through the normal workflow.

Fix: PR 4 scope must include "create `plans/reviews/{active,archive}/` and retarget the `talk-to-claude-opus` output path to `plans/reviews/active/` in the same PR" (PR 4 thereby creates the first slice of `plans/`; note that in PR 5's description so the ordering reads intentionally). Relatedly, PR 3 should name the portability contract for this skill explicitly: the skill depends on the external `~/work/talk-to-claude` project, so "remove personal absolute paths" concretely means an environment variable such as `TALK_TO_CLAUDE_PROJECT` with a fail-fast message when unset — say so, or PR 3's author has to invent the contract.

## 2. Non-blocking elegance improvements

- **Duplicate disposition row.** `scripts/check_codegen_binary_size.sh` appears twice in the "Move into verification areas" list (once bundled with the perf scripts, once alone). Merge into the single standalone row.
- **Migration-status table has no declared location.** PR 6 adds it but never says where it lives. Since it is process state, the cleanest home is a section inside this issue doc (which by then lives in `plans/issues/active/`). Name the location explicitly so area PRs know what file to update.
- **The internal_docs durable-topic list contradicts the plan's own dispositions.** The "small set of durable topics" list omits `diagnostic_codes.md`, `hir_maintainability_guardrails.md`, `sifr_driver_maintainability_guardrails.md`, `tooling_analysis.md`, `tooling_reuse_strategy.md`, and `tooling_verification.md` — yet the typescript-go consolidation row names `tooling_analysis.md` and `tooling_verification.md` as kept consolidation targets, and the guardrail docs describe active conventions. Either complete the list or mark it "representative, not exhaustive; the PR 1 relevance audit is authoritative per file." Otherwise PR N+5 can read the list as a deletion mandate.
- **`.claude/` is missing from the ignored-local table.** The table covers `.claude.log` but not the local `.claude/` directory, and `.gitignore` only covers `.claude/worktrees/*`. Add a `.claude/` row (delete-locally/keep-ignored) or extend the ignore rule.
- **Pin the runner invocation and package layout.** `pyproject.toml` at `verification/` with the package at `verification/runner/sifr_verify/` requires explicit package-dir configuration. State the canonical command once (e.g. `uv run --project verification python -m sifr_verify --profile create-pr`) and the layout choice, so PR 6 doesn't bikeshed it.
- **PR N+3 is nearly empty by construction.** Each area PR already requires "old verification script names deleted, not wrapped", so by PR N+3 little should remain. Re-scope it honestly as "sweep: prove no verification implementation remains in `scripts/` and add the guardrail enforcing it" or fold it into PR N+6.
- **Say that area PRs edit the legacy facade.** `run_all_tests.sh` calls `verification/tooling/*.py` and `verification/performance/*.py` checks directly today, so each PR 8-N area migration necessarily edits the facade's dispatch in the same PR. Add one line to the PR 8-N validation list ("legacy facade dispatch updated atomically with moved paths") so it isn't discovered mid-PR.
- **Plan for the Review Notes section's own afterlife.** Per the plan's review-artifact policy, when this issue moves to `plans/issues/archive/`, "Review Notes" should already be the concise-summary form (it nearly is; just keep it that way rather than appending per-pass logs).

## 3. Vocabulary and folder-structure recommendations

The vocabulary survives challenge; do not churn it. `areas` / `profiles` / `suites` / `cases` / `fixtures` / `baselines` / `corpora` / `policy` maps cleanly onto the reference compilers the plan cites: areas ≈ Rust's behavior-mode test families, baselines ≈ TypeScript's `tests/baselines/reference/`, suites ≈ lit-style area-local groupings, profiles ≈ CI tier selection — and `profiles` over `lanes` is unambiguously right given the public `--profile` flag and the compiler/SIMD meaning of "lane". The golden-as-case-kind normalization (no `golden/` ownership outside areas) is the correct shape and matches how TypeScript treats baselines as ordinary owned assets.

Two specifics:

- `data/` is the weakest term (it means "machine-readable inventories consumed by gates"). `inventories/` would be more self-describing, but the rename buys little and the schema description can carry the definition — keep `data/` and define it in `area.schema.json`.
- The one structural addition I strongly recommend is the B3 concept: a named, schema-level home for toolchain and guardrail steps (runner-executed step kinds selected by profiles via a committed registry). That is a genuinely missing concept in the model, not vocabulary churn — without it the "thin facade" end state cannot honor the "no coverage reduction" non-goal.

## 4. Verdict

**REQUEST CHANGES.** All five blocking items are document-level fixes — none invalidates the architecture, the PR sequence skeleton, or the disposition tables, which are otherwise complete and verified against the live tree. Resolve B1–B5 (B3 is the only one requiring a real design decision, and it must land before PR 6 because the profile schema depends on it) and the phase is implementation-ready.
