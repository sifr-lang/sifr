# Review Round 3 — `hardening_1`: Execute the Rust-Interop Area in Authoritative Profiles

> **Authoritative post-rebase disposition:** This review describes the
> historical pre-rebase working tree. The milestone commit was later rebased
> onto `origin/main` at `44d8f7160` (PR #3017). The implementation diff reviewed
> in round 4 had 15 files, before later process-only review artifacts, and
> contains none of the diagnostics MDX, Ruff revalidation, or
> token-fixture paths discussed in §§8–10; those repairs are upstream and must
> not be claimed in this PR's description. The fixture `Cargo.lock` is tracked,
> so §12.1 is complete, and §12.2 is superseded. Round 4 verifies the rebased
> scope and lane; the later approval round is the authoritative final verdict.

**Scope reviewed:** the complete uncommitted diff in this worktree (19 modified files
+ 1 untracked fixture lockfile) against
`plans/issues/active/rust-interop-verification-matrix-hardening.md` → `hardening_1`,
`AGENTS.md`, and the phase-closure expectations.

**Verdict: APPROVED** — no actionable findings. Every round-1/round-2 finding remains
fixed, and the changes new since round 2 (the Ruff fork revalidation pin and its five
token fixtures) are substantiated by evidence I derived independently rather than
accepting the recorded rationale.

Nothing in this review was taken on trust from the round-1/round-2 artifacts; each
claim below is backed by a command I ran in this worktree.

---

## 1. Requirement-by-requirement conformance

| `hardening_1` requirement | Status | Evidence |
|---|---|---|
| Add a `rust_interop_checks` step to the fixed legacy-facade sequence | Met | `profile_runner.py:67` inside `LEGACY_FACADE_STEPS_BEFORE_GENERATED`, immediately after `python_interop` |
| Implement it by reading the suites selected for `rust_interop` and calling the existing area runner, exactly as other explicit area steps do | Met | `run_rust_interop_checks` (`profile_runner.py:513-533`) uses the same `selected_suites_for_area` + `uv_area_command` + `run_command` shape as `run_python_interop_suites` |
| Add `rust_interop` selections for `matrix`, `tiers`, `compatibility-matrix`, `stale-drafts` to all four authoritative profiles | Met | identical 4-suite `default-local` blocks in `create-pr.json`, `merge.json`, `nightly.json`, `release.json`; set-equality asserted against the manifest by the new self-test |
| Add a blocking positive create-PR step budget from a measured warm run | Met | `create-pr.json:15` → `{"budget_ms": 5000, "enforcement": "blocking"}`; measured warm area wall-clock 0.41 s (see §3) |
| Leave merge/nightly/release on existing lane budgets | Met | all three profiles have **no** `step_budgets` object at all, so no per-step budget was introduced |
| Self-tests fail if a normal legacy-facade profile selects `rust_interop` but omits the step / omits a required suite / reports no result JSON | Met | `_rust_interop_profile_self_test` (`selftest.py:280-425`) covers all three, plus 12 result-document mutations and a malformed-JSON case |
| Update the area README with direct and profile commands | Met | `verification/areas/rust_interop/README.md:63-81` — direct `areas run`, `--profile create-pr`, and `--emit-plan` |

### Exit-gate items

| Exit gate | Result |
|---|---|
| Direct area execution passes | `areas run --area rust_interop` (4 suites) → `variants=4, failures=0, blocking_failures=0` |
| Emitted plan contains the area selection | `run_all_tests.sh --profile create-pr --emit-plan` → `{"area": "rust_interop", "resource_classes": ["default-local"], "suites": ["matrix","tiers","compatibility-matrix","stale-drafts"]}` and `step_budgets.rust_interop_checks = {budget_ms: 5000, enforcement: blocking}` |
| All four profile dry/plan tests prove the step is scheduled | `sifr_verify --self-test` → 8/8 pass, including `Rust interop profile execution self-test`, which asserts `"rust_interop_checks" in legacy_facade_step_names(profile)` for all four |
| `scripts/run_all_tests.sh --profile create-pr` prints `name=rust_interop_checks ... status=pass` | See §7 |

---

## 2. Exact manifest-derived suite coverage

`required_rust_interop_suites()` (`profiles.py:181-193`) derives the required set from
`verification/areas/rust_interop/manifest.json` rather than from a hardcoded literal, and
is fail-closed on a degenerate manifest (empty/non-list suites → `ProfileError`;
`len(names) != len(suites)` catches duplicate or nameless entries).

Verified the derivation is exact, not merely a superset:

- manifest suites = `['matrix', 'tiers', 'compatibility-matrix', 'stale-drafts']`
- emitted result-document suites = the same four
- the self-test asserts **set equality** (`selected != required_suites` → `AssertionError`),
  so a profile that silently drops or gains a suite fails, and a suite added to the
  manifest later forces all four profiles to adopt it.

Unknown suite names are separately rejected by the pre-existing loop at
`profiles.py:153-158`, so coverage is bounded on both sides.

The new "the area must be present at all" rule (`profiles.py:180`) is the piece that closes
the plan's "profile JSON accepting `rust_interop` selections that the legacy facade does not
execute" hole in the opposite direction — a profile can no longer drop the area entirely.
It is enforced at load time, not only in tests: `load_profile` calls
`validate_selected_area_suites(payload)` at `profiles.py:72`. I confirmed the only
`selected-areas-only` profile is `python-interop-live.json` (which selects `python_interop`
only), so the new mandatory-area rule has no collateral effect.

---

## 3. Step budget realism

The plan requires a *measured warm* positive budget. Warm timings in this worktree:

```
uv run ... areas run --area rust_interop --suite matrix --suite tiers \
  --suite compatibility-matrix --suite stale-drafts   → 0.413 s wall
per-case: matrix 76 ms, tiers 27 ms, compatibility-matrix 28 ms, stale-drafts 197 ms
```

5000 ms gives ~12× headroom over a 0.41 s warm run. That is loose in relative terms but
consistent with the surrounding budgets in the same file (`cpython_differential` 30 s,
`frontend_syntax_guardrails` 60 s) and it must absorb `uv` interpreter startup and cold
filesystem cost on other machines. Not a finding.

---

## 4. Result JSON fail-closed validation

`validate_rust_interop_result` (`profile_runner.py:105-158`) is genuinely fail-closed and
is checked against the *real* emitted document, not an over-fitted stub. I re-emitted the
document this round and confirmed field-by-field agreement with the self-test's
`valid_payload`: `schema_version=1`, `area="rust_interop"`, `bless=false`, four suites with
`blocking=true, total_variants=1, total_failures=0`, and
`summary={blocking_failures: 0, non_blocking_failures: 0, total_failures: 0, total_variants: 4}`.

Specific hardening confirmed:

- **bool-vs-int.** `_is_positive_int` / `_is_zero_int` (`profile_runner.py:97-102`) both
  exclude `bool` explicitly. This matters because `True == 1` and `False == 0` in Python,
  so a naïve `value > 0` would accept `total_variants: true` and a naïve `== 0` would
  accept `total_failures: false`. Mutations 10 and 12 in `invalid_payloads` cover
  `total_variants: True` on both the suite and the summary and are proven to be rejected.
- **bless rejection.** `payload.get("bless") is not False` uses identity, so `true`, `0`,
  `null`, `"false"`, and a missing key all reject; only a literal JSON `false` passes.
  Mutation 3 covers `bless: true`. This is the correct polarity — a blessed run must never
  satisfy a blocking gate.
- **Missing / malformed / mismatched.** Missing file, non-JSON text, wrong
  `schema_version`, wrong `area`, non-list `suites`, a dropped suite, non-blocking suite,
  failing suite, zero-variant suite, `summary.blocking_failures > 0`, and zero
  `summary.total_variants` are each mutation-tested and rejected.
- **Suite-set equality** against the profile's own selection, so an area runner that
  silently executed a subset cannot pass.
- **Ordering.** `run_command` raises on a non-zero area exit *before* validation is
  reached, and `result_path.unlink(missing_ok=True)` runs first, so a stale artifact from a
  previous lane cannot be mistaken for fresh evidence. This is the load-bearing detail for
  "reports no Rust-interop result JSON" and it is correct.
- **No silent skip.** An empty suite selection raises `ProfileRunnerError`
  (`profile_runner.py:515-518`) rather than returning; `timed_step` converts that to
  `status=fail`.

`blocking is not True` is currently a constant-true assertion — `area_adapter.py:154`
hardcodes `"blocking": True` for every area suite — so it is defense-in-depth rather than
live coverage. Harmless and correct to keep.

---

## 5. Step registry refactoring and ordering

The refactor from an inline list of bound methods to module-level `(name, method_name)`
tuples is behavior-preserving:

- I diffed the old inline order against
  `LEGACY_FACADE_STEPS_BEFORE_GENERATED` + `GENERATED_CODE_QUALITY_STEP` +
  `LEGACY_FACADE_STEPS_AFTER_GENERATED`. The only delta is the insertion of
  `rust_interop_checks` after `python_interop`; every other step keeps its exact position,
  including the conditional placement of `generated_code_quality_checks` between
  `sysroot_release_certification` and `crate_tests`.
- The gating expression `legacy_facade(profile)["generated_code_quality"] != "none"` is the
  same value the retained `generated_code_quality_mode` property (`profile_runner.py:315`)
  computes, so the conditional step appears under exactly the same conditions as before.
- I resolved every `method_name` in the registry against `ProfileRunner` — all 21 exist, so
  the `getattr` indirection cannot fail at runtime with a typo'd name.
- Placement is also operationally sensible: the step lands early and cheap, before
  `crate_tests` and `e2e_pass_suite`, so a Rust-interop regression surfaces in seconds
  rather than after the long tail.
- `PROFILE_STEP_NAMES` gained `rust_interop_checks` (`profiles.py:30`), which is what
  permits the new `create-pr` `step_budgets` key to load (unknown keys are rejected at
  `profiles.py:282-284`).

---

## 6. Tracked nested `Cargo.lock` and the offline fixture

- `.gitignore:30` adds `!/verification/areas/rust_interop/fixtures/**/Cargo.lock`, placed
  after the `**/Cargo.lock` ignore so the negation takes effect. `git status` reports the
  lockfile as `??` (untracked, stageable) and `git check-ignore -v` attributes it to the
  negation pattern — confirmed un-ignored.
- The lockfile itself is minimal and hermetic: `version = 4`, exactly two path packages
  (`locked-offline-cache`, `locked_bridge`), no registry entries, no checksums, no absolute
  paths.
- `cargo metadata --locked --offline` succeeds inside
  `fixtures/cargo_locked_offline/examples/locked_offline_cache/` (exit 0), which proves the
  lockfile is in sync with the manifests and that the fixture needs no network. This
  satisfies the acceptance criterion "no external-network dependency is introduced".
- The lockfile is load-bearing, not cosmetic: the pre-existing `matrix` suite hard-requires
  its presence, and this PR makes `rust_interop_checks` blocking in all four authoritative
  profiles.

---

## 7. Authoritative lane execution

`scripts/run_all_tests.sh --profile create-pr` was run to completion on the exact reviewed
tree and **succeeded** — all 22 legacy-facade lane steps report `status=pass`, zero
failures. Evidence: `target/validation_lane_reports/create-pr.latest.json` and
`create-pr.latest.log`.

**The exit-gate line is present and passing:**

```
[sifr-lane-step] name=rust_interop_checks elapsed_ms=379 status=pass
[sifr-lane-step-budget] name=rust_interop_checks elapsed_ms=379 budget_ms=5000 \
  enforcement=blocking status=pass
```

379 ms against the new blocking 5000 ms budget — ~13× headroom, consistent with the 0.41 s
warm measurement in §3.

**The area ran its full manifest-derived selection**, in-lane, not as a stub:

```
suite=matrix                 → fixtures=34 diagnostics=10 crates=44 …   67 ms
suite=tiers                  → tiers=5 fixtures=34                      26 ms
suite=compatibility-matrix   → rows=34 fixture_rows=34 categories=4      24 ms
suite=stale-drafts           → stale draft scan ok                     188 ms
result_json=target/verification/areas/rust-interop-create-pr-results.json
rust interop verification ok: variants=4, failures=0, blocking_failures=0, non_blocking_failures=0
```

Four suites, 4 variants, zero failures — matching the direct-execution result in §1 and the
document shape the new `validate_rust_interop_result` gate asserts.

**Every blocking per-step budget passed** (21 budgeted steps, all `status=pass`), including
the long tail: `python_interop` 369 899/600 000 ms, `e2e_pass_suite` 367 022/600 000 ms,
`crate_tests` 128 711/600 000 ms, `generated_code_quality_checks` 44 549/120 000 ms,
`performance_budget_checks` 6156/120 000 ms (this is the step that consumes the repaired
Ruff fork revalidation record — `ruff fork update rules: PASS`),
`verification_hardening_self_tests` 269/60 000 ms.

**E2E: `131 passed, 0 failed`** (`131 pass tests completed`), `test result: ok. 1 passed; 0
failed`, `hardening_summary = {variants: 6, failures: 0, blocking_failures: 0, skipped: 0}`.

**The one advisory is non-blocking and cache-attributable.** The report records
`advisories: ["warm wall-time budget exceeded"]` — `time.real_seconds = 1074.03` against a
5-minute *warm* target. This run was not warm: `e2e.cache_hits = 0/42` with
`rebuild_groups = 42` and `cache_hit_rate = 0.0`, so all 42 e2e groups were built from
scratch (`build_ms = 341 212`, `build_sum_ms = 664 300`). The e2e cache directory was
materialized by this run (5.78 GB / 34 161 files at
`target/sifr_e2e_cache/create-pr`). The overall wall-time target is an advisory field
(`budget.within_warm_budget = false`) with no enforcement attached — it does not set a step
status and did not affect the lane outcome, and the reviewed change contributes 379 ms of
the 1074 s. Not a finding.

---

## 8. Diagnostics `.mdx` baseline repairs

Two validators still resolved `docs/errors/{code}.md` after the Mintlify MDX migration
(`0204b4e5d`, `8cf4efdd0`, `c056dfca0`). `docs/errors/` now holds **205 `.mdx`** files and
exactly one `.md` (`diagnostic-codes.md`, the index, which both checks correctly still read
as `.md`). Both checks were therefore red on `main`, and both are blocking create-PR steps.

- `verification/areas/diagnostics/checks/code_coverage.py:174` → `.mdx`. Real run: exit 0.
- `verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py`
  → three production-path repairs (`:147`, `:155`) plus the matching `seed_minimal_repo`
  and negative-self-test seeds so the self-test still exercises a failing case. Real run:
  `PASS`; `--self-test`: `PASS`.

**This is a root-cause repair, not masking.** The checks still assert the same property —
"every active diagnostic code has a docs page, and legacy codes name their replacement" —
against the extension the files actually have. Nothing was weakened, deleted, or made
conditional; no code was excluded from the check. I verified there is no remaining
`docs/errors/*.md` page reference anywhere under `verification/` or `scripts/`.

The pre-existing `.md` read in `crates/sifr_driver`'s `source_tree_diagnostic_explanation`
was correctly left untouched — it is a debug-only path that falls through to the registry,
and deleting it deserves its own change.

---

## 9. Ruff fork SHA and token fixture evidence (new since round 2)

This is the one materially new part of the diff, so I validated it from first principles
rather than reading the rationale.

**The baseline was red.** Commit `59251ac79f` ("Bump Ruff fork to include rust.async parser
formatting fix") moved the `third_party/ruff` gitlink to
`e024f2a4870568734a3b215226570aab87e396a2` but left `ruff_fork_revalidation.json` and all
five token fixtures recording `8111415495271a09f9ee89cb168fde669db240d8`.
`check_ruff_fork_update_rules.py` compares the recorded revision against
`git -C third_party/ruff rev-parse HEAD`, so it necessarily failed on `main` — and it is a
blocking create-PR `performance_budget_checks` input. `git ls-tree HEAD third_party/ruff`
confirms the committed gitlink is already `e024f2a487`, i.e. **this diff contains no
submodule bump**; it only repairs the stale evidence record.

**The revalidation claim is substantiated.** The recorded rationale asserts that "the
complete `sifr_syntax` tests and representative syntax token fixture expectations remain
valid for the checked-in fork pin." I verified both halves:

1. `cargo test -p sifr_syntax` → exit 0, `13 passed; 0 failed; 0 ignored`. This is a
   blocking suite in all four authoritative profiles' `crate_test_membership`
   (`{"id": "sifr_syntax", ..., "status": "blocking", "modes": ["smoke","full"]}`), so the
   claim is bound to executed evidence, not prose.
2. The fork delta `8111415..e024f2a` is a **single commit touching a single file** —
   `crates/ruff_python_parser/src/parser/expression.rs`, 1 insertion / 2 deletions. I read
   the diff: it is a pure `rustfmt` reflow of `rust_async_attribute_is_allowed`'s existing
   `matches!(...) && self.at(TokenKind::Async)` expression onto one line. There is no
   semantic, lexer, or token-kind change, so the five fixtures'
   `expected_token_kinds` are provably unaffected.

**Therefore this is a legitimate repair, and no unexecuted claim is introduced.** Worth
recording for the follow-up backlog (not this PR): `check_ruff_fork_update_rules.py` only
string-compares SHAs and asserts `expected_token_kinds` is non-empty. Nothing in the repo
re-derives those token kinds from the pinned fork — `crates/sifr_syntax`'s
`lexer_token_matrix_preserves_kinds_and_byte_spans` tokenizes its own syntax matrix, and
`check_editor_assets.py:147-153` only harvests the kinds as a grammar-scope coverage set.
So the "token fixture expectations remain valid" half of the rationale is, in general, a
human assertion the gate cannot verify. It happens to be independently provable here
because the delta is formatting-only; a future semantic fork bump would deserve a real
tokenize-and-compare check. That is a pre-existing gap in a file this item does not own.

---

## 10. Unrelated changes and masking assessment

| Change | Related to `hardening_1`? | Assessment |
|---|---|---|
| `profile_runner.py`, `profiles.py`, `selftest.py`, four profile JSONs, area README | Yes | Core of the item |
| `.gitignore` negation + tracked fixture `Cargo.lock` | Yes | Required for the newly blocking `matrix` suite to pass on a fresh clone |
| Two plan docs: `--area rust_interop` → `areas run --area rust_interop` | Yes | Real repair, not cosmetic: the old form errors with `unrecognized arguments: --area` (verified); the new form runs. Both plan docs' Required Validation blocks are now executable |
| `code_coverage.py` + `check_diagnostic_source_canonicalization_rules.py` `.md`→`.mdx` | No — baseline repair | Blocking create-PR steps red on `main`; §8. Root-cause fix, no weakening |
| `ruff_fork_revalidation.json` + five token fixtures | No — baseline repair | Blocking create-PR input red on `main` since `59251ac79f`; §9. Evidence independently confirmed |

**No change masks a baseline failure.** All three baseline repairs restore a check to a
truthful assertion about the current tree; none narrows scope, adds an exclusion, relaxes a
threshold, introduces a fallback, or skips a test. The one place where weakening was
plausible — pinning a Ruff revision without revalidating — is discharged by the
formatting-only fork delta plus the passing blocking `sifr_syntax` suite.

The three baseline repairs are outside Wave 0's literal scope but are strictly necessary to
satisfy `hardening_1`'s own exit gate, which requires the create-PR lane to run to
`rust_interop_checks` and pass. **They must be called out explicitly in the PR
description** so they do not read as unexplained drift. That is a description requirement,
not a code finding.

---

## 11. Acceptance criteria and AGENTS.md

- "The Rust-interop area runs in create-PR, merge, nightly, and release profiles." — met and
  mechanically enforced in both directions (area mandatory; suite set exactly manifest-derived).
- "profile selections not executed by the legacy facade are rejected" — met via the new
  mandatory-area rule plus the executable step.
- "No user-triggerable panic path, fallback, test skip, or external-network dependency is
  introduced." — met. Failures raise `ProfileRunnerError`/`ProfileError`, which the lane
  converts to `status=fail`; there is no fallback branch; `cargo metadata --locked
  --offline` proves the fixture is hermetic.
- No panic paths, `unwrap`, or generated-runtime code involved; no `crates/` changes at all
  (`git diff --stat` shows zero Rust files), so `cargo clippy`/`cargo fmt` risk is nil.
- File-size guardrail: `PASS (2821 files, limit 900 lines)`. `profile_runner.py` is at
  **840** lines — 60 under the cap. Worth noting for the next item that touches it:
  `hardening_3` should not add bulk here without extracting the step registry into its own
  module.
- `git diff --check`: clean. `py_compile` on all three touched runner modules: clean.
- Items are ordered one-PR-at-a-time per the required workflow; this PR touches only
  `hardening_1` scope plus the unblocking repairs.

---

## 12. Mechanical preconditions before publishing

Not code defects — steps that must not be lost:

1. **`git add` the untracked lockfile**
   `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`.
   It is still `??`. `git commit -am` would silently omit it, and the newly blocking
   `matrix` suite hard-requires it, so every fresh clone would fail the new gate.
2. Include the §10 scope note (three baseline repairs) in the PR description.

## 13. Non-blocking follow-ups (do not gate this PR)

- **Step-registry drift guard.** Now that the legacy-facade steps are data, a two-line
  self-test asserting `PROFILE_STEP_NAMES == {names in the registry}` is cheap. It would
  immediately surface a pre-existing inconsistency I found: `sysroot_release_certification`
  is in the step registry but **absent** from `PROFILE_STEP_NAMES`, so a `step_budgets`
  entry for it would be rejected as an "unknown step". Pre-existing on `main` and out of
  scope here.
- `validate_rust_interop_result` does not cross-check `summary.total_variants` against the
  sum of per-suite `total_variants`, nor validate `summary.total_failures` /
  `non_blocking_failures`. An internally inconsistent machine-generated document would pass.
  (Carried from round 2.)
- The broadened `.gitignore` negation un-ignores `Cargo.lock` under every cargo project in
  `rust_interop/fixtures/`. When `certification_11` performs in-tree `--locked/--offline`
  builds, expect generated lockfiles to surface as untracked. (Carried from round 2.)
- The mandatory-`rust_interop`-area rule and the "python_interop suites only if selected"
  rule are asymmetric hand-written pairs; collapse into one data-driven rule when a third
  area becomes mandatory. (Carried from rounds 1–2.)
- A real tokenize-and-compare check behind `check_ruff_fork_update_rules.py` would turn the
  token-fixture half of the revalidation rationale into executed evidence (§9).

---

## Commands run for this review

```
git diff / git diff --stat / git status --short         # full uncommitted diff
git ls-tree HEAD third_party/ruff                       # confirms no submodule bump here
git -C third_party/ruff diff 8111415..e024f2a           # 1 file, formatting-only
git submodule status                                    # pin == recorded revision
uv run --project verification --locked python -m sifr_verify --self-test          # 8/8 pass
uv run ... sifr_verify areas run --area rust_interop --suite {4} --result-json …  # 4/0/0
scripts/run_all_tests.sh --profile create-pr --emit-plan                          # selection + budget
scripts/run_all_tests.sh --profile create-pr                                      # §7
cargo test -p sifr_syntax                               # 13 passed, 0 failed
python3 verification/areas/performance/check_ruff_fork_update_rules.py            # PASS
python3 verification/areas/diagnostics/checks/code_coverage.py                    # exit 0
python3 .../check_diagnostic_source_canonicalization_rules.py [--self-test]        # PASS / PASS
cargo metadata --locked --offline    (in the fixture example dir)                 # exit 0
git check-ignore -v <fixture Cargo.lock>                # negation pattern matches
python3 scripts/check_file_size_guardrails.py           # PASS (2821 files)
git diff --check                                        # clean
python3 -m py_compile <three touched runner modules>     # clean
```

No implementation file was modified during this review.
