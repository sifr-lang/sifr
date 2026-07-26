# Review Round 4 — `hardening_1`: Execute the Rust-Interop Area in Authoritative Profiles

**Scope reviewed:** the exact committed diff `HEAD` (`d41c52ed2` "Run Rust interop checks in
authoritative profiles") versus `origin/main` (`44d8f7160`, PR #3017) after the rebase.
`git merge-base HEAD origin/main` == `44d8f7160`, so the diff is a single commit and
15 files, with no residue of the pre-rebase tree.

Measured against `plans/issues/active/rust-interop-verification-matrix-hardening.md` →
`hardening_1`, its exit gate, and `AGENTS.md`.

**Verdict: NOT APPROVED** — one actionable finding, severity **Low**, documentation-only
(the committed round-3 review artifact still describes the pre-rebase 19-file tree and
instructs the PR description to declare three baseline repairs that are now upstream).
No code, profile, or fixture finding remains: the implementation is correct, fail-closed,
and the full authoritative create-PR lane passes on this exact rebased tree.

---

## 0. Rebase resolution — explicit statement

**Yes: the rebase resolution correctly removed every now-upstream baseline repair.** Verified
directly, not inferred.

`origin/main` (#3017, "Restore authoritative local validation") contains all of them:
`crates/sifr/src/explain_cli.rs`, `crates/sifr_diagnostics/src/bin/gen-error-docs.rs`,
`verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py`,
`verification/areas/diagnostics/checks/code_coverage.py`,
`verification/areas/performance/ruff_fork_revalidation.json`, and the five
`sifr_syntax_token_fixtures/*.json` records, plus two unrelated Rust lint/format repairs.

`git diff --name-status origin/main...HEAD` contains **none** of those paths, and no
`crates/**` path at all:

```
M  .gitignore
M  plans/issues/active/rust-interop-runtime-ecosystem-certification.md
M  plans/issues/active/rust-interop-verification-matrix-hardening.md
A  plans/reviews/active/rust-interop-hardening-1-review-round1.md
A  plans/reviews/active/rust-interop-hardening-1-review-round2.md
A  plans/reviews/active/rust-interop-hardening-1-review-round3.md
M  verification/areas/rust_interop/README.md
A  verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock
M  verification/profiles/create-pr.json
M  verification/profiles/merge.json
M  verification/profiles/nightly.json
M  verification/profiles/release.json
M  verification/runner/sifr_verify/profile_runner.py
M  verification/runner/sifr_verify/profiles.py
M  verification/runner/sifr_verify/selftest.py
```

There is no submodule change (`third_party/ruff` gitlink untouched), no diagnostics-area
change, no performance-area change. The 15-file diff is exactly `hardening_1` scope plus the
two plan-doc command repairs and the three review artifacts. Working tree is clean, and
stayed clean after a full lane run.

---

## 1. Requirement-by-requirement conformance

| `hardening_1` requirement | Status | Evidence |
|---|---|---|
| Add a `rust_interop_checks` step to the fixed legacy-facade sequence | Met | `profile_runner.py:66` in `LEGACY_FACADE_STEPS_BEFORE_GENERATED`, immediately after `python_interop` |
| Implement it by reading the suites selected for `rust_interop` and calling the existing area runner, exactly as other explicit area steps do | Met | `run_rust_interop_checks` (`profile_runner.py:513-533`) reuses `selected_suites_for_area` + `uv_area_command` + `run_command`, the same shape as `run_python_interop_suites` (`profile_runner.py:504-511`) |
| Add `rust_interop` selections for the four suites to `create-pr`, `merge`, `nightly`, `release` | Met | identical 4-suite `default-local` blocks in all four profile JSONs; set equality against `manifest.json` asserted by the new self-test |
| Blocking positive create-PR step budget from a measured warm run | Met | `create-pr.json:15` → `{"budget_ms": 5000, "enforcement": "blocking"}`; measured 0.44 s warm direct, 402 ms in-lane (§6) |
| Leave merge/nightly/release on existing lane budgets | Met | none of the three has a `step_budgets` object at all — confirmed programmatically |
| Self-tests fail if a legacy-facade profile selects `rust_interop` but omits the step / omits a required suite / reports no result JSON | Met | `_rust_interop_profile_self_test` (`selftest.py:280-425`) covers all three, plus 12 result-document mutations and a malformed-JSON case |
| Update the area README with direct and profile commands | Met | `verification/areas/rust_interop/README.md:63-81`: direct `areas run`, `--profile create-pr`, `--emit-plan` |

### Exit gate

| Exit gate | Result |
|---|---|
| Direct area execution passes | `areas run --area rust_interop` (4 suites) → `variants=4, failures=0, blocking_failures=0`, 0.44 s |
| Emitted plan contains the area selection | all four `--emit-plan` outputs carry `{"area": "rust_interop", "suites": [4], "resource_classes": ["default-local"]}`; `create-pr` additionally carries `step_budgets.rust_interop_checks` |
| All four profile plan tests prove the step is scheduled | `sifr_verify --self-test` → 8/8 pass, including `Rust interop profile execution self-test` |
| `scripts/run_all_tests.sh --profile create-pr` prints `name=rust_interop_checks ... status=pass` | **Confirmed on the rebased tree** (§6) — `LANE_EXIT=0` |

---

## 2. Manifest-derived suite coverage is exact and fail-closed in both directions

`required_rust_interop_suites()` (`profiles.py:187-199`) derives the required set from
`verification/areas/rust_interop/manifest.json` — no hardcoded literal — and rejects a
degenerate manifest (non-list/empty suites → `ProfileError`; `len(names) != len(suites)`
catches nameless or duplicate entries).

- Manifest suites: `['matrix', 'tiers', 'compatibility-matrix', 'stale-drafts']`; all four
  profiles select exactly those four. A fifth manifest suite fails all four profiles closed
  until adopted.
- Upper bound: unknown suite names are rejected by the pre-existing loop
  (`profiles.py:156-160`), and a suite name the area runner does not know fails in
  `select_suites` (`area_adapter.py:129-133`, `unknown … suite filter(s)`).
- Area presence: the new rule at `profiles.py:177-184` rejects any non
  `selected-areas-only` profile that drops the area entirely — this closes the plan's
  "profile JSON accepting `rust_interop` selections the legacy facade does not execute" hole
  from the opposite side.
- Enforced at load time, not only in tests: `load_profile` calls
  `validate_selected_area_suites` (`profiles.py:72`), which is on the `profiles run` and
  `profiles plan` paths (`profile_runner.py:234`). I confirmed no collateral: the only other
  profile, `python-interop-live.json`, is `selected-areas-only` and therefore exempt.
- `PROFILE_STEP_NAMES` gained `rust_interop_checks` (`profiles.py:30`), which is what allows
  the new `create-pr` `step_budgets` key to load — unknown keys are rejected at
  `profiles.py:282-283`.

## 3. Result-JSON validation is genuinely fail-closed

`validate_rust_interop_result` (`profile_runner.py:105-158`) was checked against the **real**
emitted document, not only the self-test stub. I re-emitted it this round
(`--result-json target/verification/areas/rust-interop-review-round4.json`) and it agrees
field-for-field with the self-test's `valid_payload`: `schema_version=1`,
`area="rust_interop"`, `bless=false`, four suites with
`blocking=true, total_variants=1, total_failures=0`, `summary.blocking_failures=0`,
`summary.total_variants=4`.

- **Freshness.** `result_path.unlink(missing_ok=True)` precedes the run, the path is
  profile-scoped (`rust-interop-<profile>-results.json`), and `run_command` raises
  `CommandFailed` on a non-zero area exit *before* validation is reached. A stale artifact
  cannot be mistaken for fresh evidence.
- **bool-vs-int.** `_is_positive_int` / `_is_zero_int` exclude `bool` explicitly, so
  `total_variants: true` and `total_failures: false` are rejected; both are mutation-tested.
- **bless polarity.** `payload.get("bless") is not False` is an identity check, so `true`,
  `0`, `null`, `"false"`, and a missing key all reject. A blessed run can never satisfy a
  blocking gate.
- **Suite-set equality** against the profile's own selection, so an area runner that silently
  executed a subset cannot pass.
- **No silent skip.** An empty selection raises `ProfileRunnerError`
  (`profile_runner.py:514-517`); `timed_step` converts `ProfileRunnerError` to `status=2` →
  `[sifr-lane-step] … status=fail` (`profile_runner.py:220-226`), so it cannot escape.
- **Path plumbing verified.** `--result-json` is passed repo-relative and resolved by
  `area_adapter.resolve_repo_path` against the same `REPO_ROOT`; the package is an editable
  install (`sifr_verify.__file__` resolves inside the worktree), and the adapter does
  `mkdir(parents=True)` so `target/verification/areas/` need not pre-exist.

## 4. Legacy step order is preserved

The refactor from an inline list of bound methods to module-level `(name, method_name)`
tuples is behaviour-preserving:

- The only delta versus the removed inline list is the insertion of `rust_interop_checks`
  after `python_interop`. Every other step keeps its exact index, including the conditional
  placement of `generated_code_quality_checks` between `sysroot_release_certification` and
  `crate_tests`.
- The gate `legacy_facade(profile)["generated_code_quality"] != "none"` is the same
  expression backing the retained `generated_code_quality_mode` property, so the conditional
  step appears under identical conditions.
- Empirically confirmed by the lane log: the 22 executed steps appear in exactly the
  `main` order with `rust_interop_checks` sixth (§6).
- Placement is operationally sound — cheap and early, well before `crate_tests` (58.9 s) and
  `e2e_pass_suite` (398 s).

## 5. Tracked offline `Cargo.lock`

- The lockfile is **load-bearing, not cosmetic**: `_scenario_checks.py:261-263` already
  hard-requires `examples/locked_offline_cache/Cargo.lock` for the `cargo_locked_offline`
  fixture on `origin/main`, and `git ls-tree origin/main` shows the file was **not tracked**
  there. Since this PR makes the `matrix` suite blocking in all four authoritative profiles,
  shipping without the lockfile would fail every fresh clone. Committing it is required, not
  incidental.
- `.gitignore:30` adds `!/verification/areas/rust_interop/fixtures/**/Cargo.lock` after the
  `**/Cargo.lock` ignore, so the negation takes effect;
  `git check-ignore -v <path>` now reports the file as un-ignored and it is tracked in the
  commit.
- Content is minimal and hermetic: `version = 4`, two path packages
  (`locked-offline-cache`, `locked_bridge`), no registry entries, no checksums, no absolute
  paths.
- `cargo metadata --locked --offline` inside the fixture example directory → exit 0, and the
  worktree stayed clean afterwards, proving the lockfile is in sync with both manifests and
  that no network is required.
- No other fixture lockfile is surfaced as untracked today (`git status --porcelain` empty).

## 6. Authoritative lane execution on the rebased tree

`scripts/run_all_tests.sh --profile create-pr` was run to completion on this exact commit →
`LANE_EXIT=0`, 22 steps, `grep -c "status=fail"` = **0**. This is the post-rebase
confirmation the earlier rounds could not provide (round 3's lane run predates #3017).

Exit-gate lines, verbatim:

```
[sifr-lane-step] name=rust_interop_checks elapsed_ms=402 status=pass
[sifr-lane-step-budget] name=rust_interop_checks elapsed_ms=402 budget_ms=5000 \
  enforcement=blocking status=pass
```

The area ran its full manifest-derived selection in-lane, not a stub:

```
suite=matrix               → fixtures=34 diagnostics=10 crates=44 package_examples=51   78 ms
suite=tiers                → tiers=5 fixtures=34                                        27 ms
suite=compatibility-matrix → rows=34 fixture_rows=34 categories=4
suite=stale-drafts         → stale draft scan ok
rust interop verification ok: variants=4, failures=0, blocking_failures=0
```

Step order as executed: `coverage_matrix_checks, core_guardrails, diagnostic_rules,
cpython_differential, python_interop, rust_interop_checks, frontend_syntax_guardrails,
developer_tooling_checks, performance_budget_checks, verification_hardening_self_tests,
verification_runner_foundation, fuzz_property_checks, algorithmic_compatibility_checks,
distribution_validation, sysroot_release_certification, generated_code_quality_checks,
crate_tests, validation_suite_matrix, runtime_platform_suites, e2e_pass_suite,
verification_hardening_suites, extra_e2e_checks` — all `status=pass`.

The lane report records the pre-existing, unenforced advisory
`["warm wall-time budget exceeded"]` (`real_seconds=1059.36` against a 5-minute *warm*
target, dominated by `python_interop` 399 s and `e2e_pass_suite` 398 s). It sets no step
status and did not affect the exit code; this change contributes 402 ms of it. Not a
finding, and not attributable to this diff.

Other focused checks this round, all green: `sifr_verify --self-test` 8/8 (including the new
one); `coverage_matrix:readiness` (the check that validates
`profile_assignment_matrix.json` tokens) → `variants=4, failures=0`;
`python3 scripts/check_file_size_guardrails.py` → `PASS (2821 files, limit 900 lines)`;
`git diff --check origin/main...HEAD` clean.

## 7. Unrelated-change assessment

| Change | In `hardening_1` scope? | Assessment |
|---|---|---|
| `profile_runner.py`, `profiles.py`, `selftest.py`, four profile JSONs, area README | Yes | Core of the item |
| `.gitignore` negation + tracked fixture `Cargo.lock` | Yes | Required for the newly blocking `matrix` suite to pass on a fresh clone (§5) |
| Two plan docs: `-m sifr_verify --area rust_interop` → `-m sifr_verify areas run --area rust_interop` | Yes | Real repair, not cosmetic: the old form exits with `__main__.py: error: unrecognized arguments: --area` (I reproduced it); the new form runs. I grepped the whole repo — these were the only two stale occurrences, so the sweep is complete |
| Three review artifacts under `plans/reviews/active/` | Yes (process) | Matches the existing convention in that directory; content issue in Finding 1 |

No change masks a baseline failure: nothing narrows a scope, adds an exclusion, relaxes a
threshold, introduces a fallback, or skips a test. No `crates/**` file is touched, so
`cargo clippy` / `cargo fmt` risk is nil. No panic path, `unwrap`, generated-runtime code, or
external-network dependency is introduced; failures raise `ProfileRunnerError` /
`ProfileError`, which the lane converts to `status=fail`.

---

## Actionable findings

### Finding 1 — Low (documentation accuracy) — committed round-3 artifact still describes the pre-rebase tree and directs a now-false PR description

**Location:** `plans/reviews/active/rust-interop-hardening-1-review-round3.md` — header
("19 modified files + 1 untracked fixture lockfile"), §8 ("Diagnostics `.mdx` baseline
repairs"), §9 ("Ruff fork SHA and token fixture evidence"), §10 rows 4–5 and its closing
paragraph, §12 items 1–2, §13 last bullet. Secondarily
`rust-interop-hardening-1-review-round2.md` ("Scope note: the two `.md`→`.mdx` validator
repairs …", "Mechanical preconditions" items 1 and 3) and
`rust-interop-hardening-1-review-round1.md` ("The `.mdx` validator repairs are the right
root cause", "the still-untracked `Cargo.lock`").

**Rationale:** these three files are **added by this diff**, so their claims ship as this
PR's permanent record. They assert that the diagnostics `.mdx` repairs, the Ruff fork
revalidation record, and five token fixtures are part of the change under review. They are
not — all of them are upstream in `origin/main` via #3017, and `git diff` confirms none of
those paths appears here. Two statements are actively harmful rather than merely stale:

1. §10 / §12.2: "The three baseline repairs … **must be called out explicitly in the PR
   description** so they do not read as unexplained drift." Following this instruction would
   produce a PR description claiming changes this PR does not contain.
2. §12.1: "`git add` the untracked lockfile … **It is still `??`**." The lockfile is now
   tracked in `d41c52ed2`, so the stated precondition reads as outstanding when it is done.

This is not a code defect and does not affect any gate; it is a truthfulness defect in an
artifact the repo keeps as the review record, and it is the one thing in the diff that
misrepresents the PR's scope.

**Fix (docs only, no implementation file touched):** add a short "Post-rebase addendum" at
the top of `rust-interop-hardening-1-review-round3.md` stating that (a) the tree was rebased
onto `origin/main` `44d8f7160` (#3017), (b) the diagnostics `.mdx` repairs, the Ruff fork
revalidation record, and the five token fixtures are upstream and are **not** part of this
PR — §8, §9, §10 rows 4–5 and §12.2 no longer apply, (c) §12.1 is resolved (the lockfile is
tracked), and (d) the reviewed diff is the 15 files listed above. A one-line pointer to the
same addendum in rounds 1 and 2 is sufficient for those two. The PR description should
describe only the 15-file scope.

---

## Non-blocking observations (do not gate; no change requested here)

- **`selected_areas` ordering.** `rust_interop` is inserted as the *first* entry in all four
  profiles, ahead of `coverage_matrix`, while the executed step is sixth. Inert in
  `legacy-facade` mode (the array is a declarative plan there), but `run_selected_areas_only`
  iterates `selected_areas` in order, so the declared order would become the execution order
  if a profile ever switched modes. Appending instead of prepending would keep declaration
  and execution order aligned.
- **Per-selection vs unioned suite validation.** `validate_selected_area_suites` applies the
  required-suite check to each `rust_interop` selection individually, whereas
  `selected_suites_for_area` unions duplicate selections (as `developer_tooling` legitimately
  does today). Splitting the four suites across two `rust_interop` entries would therefore
  fail validation even though execution would be complete. The failure direction is safe, so
  this is a robustness nit only.
- **Summary self-consistency.** `validate_rust_interop_result` does not cross-check
  `summary.total_variants` against the sum of per-suite `total_variants`, nor validate
  `summary.total_failures` / `non_blocking_failures`. An internally inconsistent
  machine-generated document would pass. (Carried from rounds 2–3.)
- **Evidence depth vs suite shape.** Per-suite `total_variants > 0` proves a suite ran, not
  that it kept its manifest cases. A suite silently reduced to one stub case would still
  satisfy the gate. Deepening this belongs to `hardening_3` (binding claims to executed
  tests), not here.
- **`blocking is not True`** is currently a constant-true assertion (`area_adapter.py:154`
  hardcodes `"blocking": True`) — defense-in-depth, correct to keep.
- **`profile_runner.py` is at 840 / 900 lines.** Under the cap, but the next item that adds
  to it should first extract the step registry into its own module rather than grow this file.
- **Step-registry drift guard.** Now that the steps are data, a self-test asserting
  `PROFILE_STEP_NAMES == {names in the registry}` is cheap and would surface the pre-existing
  inconsistency that `sysroot_release_certification` is in the registry but absent from
  `PROFILE_STEP_NAMES` (so a `step_budgets` entry for it would be rejected as unknown).
  Pre-existing on `main`. (Carried from round 3.)
- **Coverage-matrix representation.** `verification/areas/coverage_matrix/profile_assignment_matrix.json`
  has no `rust_interop` surface row, so this new scheduling is not yet represented in the
  derived coverage/claim tables. The readiness check only validates listed tokens, so nothing
  is red — and claim derivation is explicitly `certification_0` / `hardening_3` territory —
  but that row will be needed before Phase 40 `milestone_40_1` claims release-profile
  Rust-interop execution.
- **Asymmetric mandatory-area rules.** The `rust_interop` "area must be present" rule and the
  `python_interop` "suites only if present" rule are hand-written and inconsistent; collapse
  into one data-driven rule when a third area becomes mandatory. (Carried from rounds 1–3.)
- **Broadened `.gitignore` negation** un-ignores `Cargo.lock` under every cargo project in
  `rust_interop/fixtures/`. When `certification_11` performs in-tree `--locked/--offline`
  builds, expect generated lockfiles to surface as untracked. (Carried from rounds 2–3.)
- **Issue-doc status.** `hardening_1` is not marked complete in
  `rust-interop-verification-matrix-hardening.md`, and no PR link is recorded. That is
  consistent with the issue's own design — `hardening_5` owns "records merged PRs and final
  row/schema counts" — and the link cannot exist pre-merge, so it is not a finding; just
  don't lose it at closeout.
- The commit message is a bare subject line with no body. Fine mechanically; the PR
  description is where the 15-file scope should be stated (see Finding 1).

---

## Commands run for this review

```
git fetch origin main; git merge-base HEAD origin/main; git log --oneline origin/main..HEAD
git diff --stat / --name-status origin/main...HEAD; git diff origin/main...HEAD   # full diff
git show --stat origin/main                                  # confirms #3017 owns the repairs
git ls-tree origin/main -r --name-only | grep cargo_locked_offline   # lockfile untracked on main
git show origin/main:verification/areas/rust_interop/checks/_scenario_checks.py  # lock required
git check-ignore -v <fixture Cargo.lock>; git status --porcelain
uv run --project verification --locked python -m sifr_verify --self-test               # 8/8
uv run ... areas run --area rust_interop --suite {4} --result-json …                   # 4/0/0
uv run ... areas run --area coverage_matrix --suite readiness                          # 4/0/0
uv run ... python -m sifr_verify --area rust_interop      # reproduces the stale plan-doc error
scripts/run_all_tests.sh --profile {create-pr,merge,nightly,release} --emit-plan
scripts/run_all_tests.sh --profile create-pr                    # LANE_EXIT=0, 22/22 pass (§6)
cargo metadata --locked --offline    (in the fixture example dir)                    # exit 0
python3 scripts/check_file_size_guardrails.py                    # PASS (2821 files)
git diff --check origin/main...HEAD                                                   # clean
python3 -c "from sifr_verify.paths import REPO_ROOT"   # editable install resolves in-worktree
grep -rn "sifr_verify --area" (repo-wide)                    # no stale occurrences remain
```

No implementation file was modified during this review.
