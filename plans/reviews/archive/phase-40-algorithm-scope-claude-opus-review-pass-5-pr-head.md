## Review — Phase 40 algorithmic release-scope, pass 5 (exact PR head `99c847705`)

Reviewed the full `c17f3c7d1..99c847705` diff (16 files) and the exact commit tree. Working tree is clean at `99c847705`; the only untracked path is the zero-byte `plans/reviews/active/phase-40-algorithm-scope-claude-opus-review-pass-5-pr-head.md` placeholder, which I did not write to. No file was modified; all mutation probing ran in a `/tmp/p5` copy, now deleted.

### Independently verified (not taken from prior passes)

**Scope decision is honest and nothing is hidden.**
- The 20 slugs are byte-identical — the diff touches no line in the inventory list (`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md`); the only edits are surrounding prose and criteria.
- `nightly.json` untouched: `algorithmic_compatibility: ["leetcode-full","taxonomy-smoke"]`, and the surface row keeps `"status": "blocking"` with owner `algorithmic/compatibility` (validated against `owners.json`).
- No fixture, baseline, exclusion, or reclassification anywhere in the diff; no demo added; no Rust-interop or other out-of-scope change. Largest touched file is `coverage_matrix.py` at 732 lines (under the 900 cap).
- Release lane still pins the corpus: `run_representative_subset` → `load_profile_manifest` → `validate_profile_manifest` enforces `expected_fixture_count == len(glob("*.sifr"))` (verified: 411 == 411) and every declared category present (12/12). All 12 representative rows are `expected_classification: PASS`, and **overlap with the 20 failing slugs is empty** — so the release subset by construction excludes the deferred failures, which is exactly what the docs say it does.
- `coverage_matrix:readiness` is selected by all four profiles including `release`, so the expiry clock is enforced *in the release lane itself*, not only in nightly/merge.

**Checks are green at HEAD.** `coverage matrix ok: guarantees=13 surfaces=34 temporary_rows=0 strict=yes`; `profile assignment matrix ok: rows=17`; self-tests `cases=24`. Zero false positives: I re-ran the profile-derived divergence predicate over all 34 surface rows — `algorithmic_compatibility_profile` is the only diverging row, and it declares `release_suite`.

**Governance is fail-closed under mutation.** Ten probes:

| Mutation | `coverage_matrix.py` | `profile_assignment_matrix.py` |
|---|---|---|
| Drop the 3 CSM divergence fields | **rc=1** profile-derived divergence | **rc=1** diverges without `release_suite` |
| Drop CSM fields **+ delete PAM row** | **rc=1** | rc=0 (covered by CSM) |
| Past `release_divergence_expiry` | **rc=1** expiry has passed | — |
| Remove `ALG-CORPUS` from `plans/phases/index.md` | **rc=1** record is not indexed | — |
| Repoint index link to a missing file | **rc=1** record target does not exist | — |
| Remove `release_divergence_record` | **rc=1** missing or invalid | — |
| Under-declare PAM `nightly` to match release | **rc=1** | **rc=1** nightly omits required suite |

Every deletion and under-declaration path the earlier passes raised is closed, and closure does not depend on the hand-maintained assignment matrix.

**Prior-pass findings recheck.** All nine pass-1, six pass-2, and three pass-3 findings are closed at this commit, confirmed against the current text/code rather than the prior write-ups: README attributes the full corpus to nightly only and its remaining release suite list matches `release.json`; `profile_policy.md:11-13` qualifies the release bullet and `:50-52` names only implemented rules (no "ownerless"); the guarantee-layer authority note is present; taxonomy-smoke attribution matches `run_taxonomy_smoke`'s actual behavior; `release.json` description qualified and `resource_classes` is `["default-local"]` (matching merge's classification of the same suite); `performance_budget_checks` in `full` mode named in both evidence paragraphs; `milestone_40_1` cross-reference present; id-convention coupling documented; empty-`surface_id` `continue` in place.

### Non-blocking observations (no action required for this diff)

1. **`nightly_release_suite` fidelity remains ungated** (pass-4 observation 1; I reconfirmed it — falsifying that field to `taxonomy-smoke` alongside the PAM edits passes both checks). The obvious tightening (`declared ⊆ nightly selection`) is genuinely not free: I ran it over all 34 rows and it false-positives on two pre-existing rows, `lowering_layer_snapshots` → `core_language:lowering_layer_inventory` and `runtime_platform_golden` → `runtime_platform:platform-rules`. Correctly a separate issue, not a change here. The bypass also requires an affirmative misstatement in a reviewer-visible diff.
2. **No maximum expiry horizon.** `release_divergence_expiry: "2999-01-01"` passes. This matches the existing `TEMPORARY_STATUSES` convention (`validate_temporary_row` has no horizon either), so it's consistent rather than a new gap.
3. **The record target is only checked for existence**, not for living under `plans/issues/active/`. Repointing the index link at an archived stub passes. `rust_interop/checks/check_compatibility_matrix.py:28` already has the `FUTURE_OWNER_PREFIXES` precedent if this is ever worth tightening.
4. `selected_area_suite_tokens` / `is_area_suite_token` are duplicated across the two checkers with slightly different signatures; `coverage_matrix.py`'s `{cargo, e2e, sifr, sifr_codegen}` denylist is inert there since `selected_areas` only holds real area names.

### Assessment

The carve-out does what the constraints require: the full pinned corpus stays blocking in nightly, both release suites stay blocking, release still pins corpus size and per-category coverage, the divergence is machine-enforced, indexed to `ALG-CORPUS`, expiry-bound to 2026-10-31, and fails readiness closed on every deletion path — with restoration of `leetcode-full` recorded as both a closeout gate and an acceptance criterion rather than a Phase 40 prerequisite. The 20 failures are moved, not hidden.

APPROVED
