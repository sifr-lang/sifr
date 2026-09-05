## Review — PR #3037, exact head `47967b84c`

**Head/branch identity.** `gh pr view 3037` reports `headRefOid = 47967b84c2fc4e4388ecbf007149cb334c0c58af` on `codex/phase-40-release-algorithm-scope`, base `main`, `MERGEABLE`. Local `HEAD` matches exactly; `git merge-base HEAD origin/main` == `c17f3c7d1ea1ed97ca125eb7a43344b30cf9413b`, so the reviewed range is the whole PR. Working tree is clean except the zero-byte untracked placeholder `plans/reviews/active/phase-40-algorithm-scope-agent-review-pass-6-final-pr-head.md`; I wrote to no file (all mutation probing ran in a `/tmp/p6probe` copy, now deleted).

**Tail commit `99c847705..47967b84c`** is exactly two files, docs-only: the pass-5 archive plus a 6-line ledger entry under `milestone_40_4`. No implementation, data, profile, or scope change enters in the tail — `verification/**`, `crates/**`, `demos/**` are byte-identical between the two commits.

### Independently re-verified (not taken from pass 5)

- **Constraints hold.** `nightly.json` untouched — `algorithmic_compatibility: ["leetcode-full","taxonomy-smoke"]`; surface row still `"status": "blocking"`. The 20 slugs are byte-identical (no `± - \`slug\`` lines in the diff). No fixture/baseline/exclusion/reclassification, no `crates/**` or `demos/**` change, no Rust-interop implementation. Restoration of `leetcode-full` is recorded as closeout scope + acceptance criterion, not a Phase 40 prerequisite.
- **Release still pins the corpus.** `run_representative_subset` → `load_profile_manifest` → `validate_profile_manifest` enforces `expected_fixture_count` (411) against the actual `corpus_root` glob (411 ✓). All 12 representative rows are `expected_classification: PASS`, cover 12/12 `required_categories`, and their intersection with the 20 failing slugs is **empty**.
- **Expiry is enforced in the release lane itself** — `coverage_matrix:readiness` is selected by all four profiles including `release`.
- **Checks green at head:** `coverage matrix ok: guarantees=13 surfaces=34 temporary_rows=0 strict=yes`, `profile assignment matrix ok: rows=17`, self-tests `cases=24`, file-size guardrail PASS (largest touched file `coverage_matrix.py` at 732 lines). `algorithmic_compatibility_profile` is the only row of 34 carrying `release_suite`.
- **Fail-closed under real-file mutation** (my own probes): drop the 3 divergence fields → `coverage_matrix` rc=1 *and* `profile_assignment_matrix` rc=1; expired expiry → rc=1; `ALG-CORPUS` removed from `plans/phases/index.md` → rc=1 "record is not indexed".

### Finding

**1. `plans/reviews/archive/phase-40-algorithm-scope-agent-review-pass-5-pr-head.md` — the mutation table's last row falsely credits `coverage_matrix.py` with catching the nightly under-declaration, and the closing sentence generalizes from it.**

The row `| Under-declare PAM nightly to match release | **rc=1** | **rc=1** nightly omits required suite |` claims both checkers fire. `coverage_matrix.py` never reads `profile_assignment_matrix.json` — `PROFILE_ASSIGNMENT_MATRIX_PATH` (`coverage_matrix.py:26`) is defined and never used. I reproduced both readings of that mutation:

- PAM `nightly` row edited to match release (the reading whose error text pass 5 quotes): `coverage_matrix` **rc=0** (`coverage matrix ok`); only `profile_assignment_matrix` fires, with *both* `nightly omits required suite …` and `release_suite is declared without a profile assignment divergence`.
- `nightly.json` under-declared instead: `coverage_matrix` **rc=0** again; only PAM fires.

Only a *combined* under-declaration that also rewrites the surface row's `nightly_release_suite` trips `coverage_matrix` (`release_suite must differ from nightly_release_suite`) — and in that case PAM's message is not the one quoted. So the artifact's concluding claim, "Every deletion and under-declaration path the earlier passes raised is closed, **and closure does not depend on the hand-maintained assignment matrix**," is wrong for the under-declaration path: `profile_assignment_matrix.py` is its sole detector. That is precisely the dependency earlier passes raised, and the added durable record asserts it was eliminated.

Scope and severity: the gate itself is still fail-closed — both checkers run inside the same blocking `readiness` suite in all four profiles, so under-declaration is caught. The defect is confined to the accuracy of the record this PR adds; the fix is correcting one table cell and qualifying one sentence (e.g. "closure of the deletion paths does not depend on the assignment matrix; the under-declaration path is closed by `profile_assignment_matrix.py`"). The `milestone_40_4` ledger entry added in the tail commit is itself accurate and needs no change.

Everything else in the artifact I spot-checked is truthful: the 411/411 and 12/12 figures, the empty-overlap claim, `strict=yes`, `cases=24`, `rows=17`, the 732-line figure, `release.json`'s `resource_classes: ["default-local"]` matching `merge`'s classification of the same suite, the README/`profile_policy.md` attributions, and the "412 variants" figure in the ledger (411 full-corpus + 1 taxonomy-smoke variant).

NOT APPROVED
