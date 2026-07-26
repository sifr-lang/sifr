# Review Round 2 — Wave 0 `hardening_1`

**No actionable code findings remain.** All seven round-1 findings are resolved; I verified each independently rather than taking the fix list at face value.

## Round-1 findings — recheck

| # | Round-1 finding | Status | Evidence |
|---|---|---|---|
| 1 | `explain_cli.rs:64` MDX scraper regression | **Fixed** | `git diff origin/main -- crates/` is empty; `cargo run -q -p sifr -- --explain SIFR-LINT-0001` prints `SIFR-LINT-0001 / Suppression references an unknown policy rule id. / Docs: …` — the correct registry text |
| 2 | Missing test for that path | **Moot** | Edit reverted; no new code path to cover |
| 3 | Silent skip when no suites selected | **Fixed** | `profile_runner.py:515` raises `ProfileRunnerError`; `timed_step` catches it → `status=2` → `status=fail`, so it cannot escape uncaught |
| 4 | `bless: true` accepted | **Fixed** | `payload.get("bless") is not False` (identity, so `0`/`None` also reject); mutation-tested |
| 5 | Untested `total_variants` branches | **Fixed** | 12 mutations + malformed-JSON + missing-file, covering suite/summary `0` and `True` on both fields |
| 6 | Hardcoded `.gitignore` path | **Fixed** | `!/verification/areas/rust_interop/fixtures/**/Cargo.lock`; `git status` shows the lockfile as `??`, confirming it is stageable |
| 7 | 0-byte round-1 artifact | **Fixed** | 3.3 KB, substantive |

## Independent verification this round

- **Validator matches reality, not just the fixture payload.** I ran the real area with `--result-json target/verification/areas/rust-interop-create-pr-results.json`: `schema_version=1`, `area=rust_interop`, `bless=False`, all four suites `blocking=True, total_variants=1, total_failures=0`, `summary.blocking_failures=0, total_variants=4`. The self-test's `valid_payload` is not an over-fitted stub — it mirrors the emitted document.
- **Result-path plumbing.** `run_command` uses `cwd=REPO_ROOT`, and `area_adapter.resolve_repo_path` rejects out-of-tree paths (I confirmed by deliberately passing `/tmp/ri.json` → `area path must stay under repo root`, exit 1). `result_path.parent.mkdir(parents=True)` in the adapter means the `target/verification/areas/` dir need not pre-exist.
- **No double execution.** `rust_interop` appears only in `run_rust_interop_checks` (`profile_runner.py:514`); the `selected_areas` entry is not independently iterated in legacy-facade mode.
- **Refactor is behavior-preserving.** `legacy_facade_step_methods` gates on `legacy_facade(profile)["generated_code_quality"] != "none"`, which is the same expression backing the old `self.generated_code_quality_mode` property (`profile_runner.py:315`) — same value, same ordering, same step names.
- **Fail-closed guard has no collateral.** `python-interop-live.json` is the only other profile and is `selected-areas-only`, so the new "must select the rust_interop area" rule doesn't break it.
- **Exit-gate machinery.** `--emit-plan` exists (`run_all_tests.sh:24`) and the emitted create-pr plan carries `rust_interop_checks: {budget_ms: 5000, enforcement: blocking}` plus the four-suite selection. `timed_step` prints `[sifr-lane-step] name=… status=pass`, matching the plan's literal exit-gate wording.
- **Fixture lockfile.** `cargo metadata --locked --offline` succeeds in `examples/locked_offline_cache/`; the lockfile is 13 lines, two path packages, no absolute paths or registry churn.
- **`.mdx` repairs are the root cause.** `docs/errors/` holds 205 `.mdx` pages and exactly one `.md` (`diagnostic-codes.md`, the index, which both checks still read as `.md`). No `errors/*.md` references remain in `verification/` or `scripts/`.
- Self-tests 8/8 pass including the new one; `code_coverage.py` exit 0; canonicalization self-test PASS; `py_compile` clean; file-size guardrails PASS (2821 files); `git diff --check` clean; no stray artifacts in the tree.

## Mechanical preconditions before publishing

Neither is a code defect — both are steps you already stated are pending, listed so nothing is lost:

1. **Stage `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`.** This is load-bearing, not cosmetic: the pre-existing `matrix` suite hard-requires it (`_scenario_checks.py:261-263`, `Cargo.lock is required`), and this PR makes `rust_interop_checks` **blocking** in all four profiles. If it ships untracked, every fresh clone fails the new gate.
2. **Write `plans/reviews/active/rust-interop-hardening-1-review-round2.md`** — currently 0 bytes, the same defect round 1 flagged for its own artifact.
3. Run the full `scripts/run_all_tests.sh --profile create-pr` — the plan's exit gate requires the actual `name=rust_interop_checks … status=pass` line, which targeted validation does not produce.

## Non-blocking observations (follow-ups, not this PR)

- **Broadened `.gitignore` negation has a forward-looking cost.** It un-ignores `Cargo.lock` under all 29 cargo projects in `rust_interop/fixtures/`. The certification issue plans in-tree `--locked/--offline/--frozen` builds in exactly those fixtures (`rust-interop-runtime-ecosystem-certification.md:126`), which will surface generated lockfiles as untracked. No clean-tree guard exists today so nothing breaks — but when `certification_11` lands, expect to either commit or re-ignore those.
- `validate_rust_interop_result` doesn't cross-check `summary.total_variants` against the sum of per-suite variants, nor validate `summary.total_failures`/`non_blocking_failures`. An internally inconsistent machine-generated doc would pass. Cheap to add later.
- Untested-but-correct branches: bool `total_failures`, non-dict payload, non-dict suite entry, duplicate suite names (`actual_suites` is a set). The no-suites raise at `profile_runner.py:515` is also untested but unreachable given `validate_selected_area_suites`.
- Carried from round 1: the `rust_interop` "area must be present" rule and the `python_interop` "suites only if present" rule are asymmetric hand-written pairs; worth collapsing into one data-driven rule when a third area becomes mandatory.
- Pre-existing on `main`, correctly left alone: `source_tree_diagnostic_explanation` (`explain_cli.rs:59-77`) reads `docs/errors/{code}.md`, which no longer exists post-MDX-migration, so it is dead in debug builds and always falls through to the registry. Reverting was the right call for this PR; deleting it deserves its own change.
- Scope note: the two `.md`→`.mdx` validator repairs are outside Wave 0's stated scope but were required to unblock two blocking create-PR steps that are red on `main`. Call that out in the PR description so it doesn't read as unexplained drift.

## Verdict

**APPROVED** — no actionable findings remain. The implementation is ready for full create-PR validation and PR publication, contingent only on the three mechanical steps above (stage the fixture lockfile, fill the round-2 artifact, run the full create-pr lane).
