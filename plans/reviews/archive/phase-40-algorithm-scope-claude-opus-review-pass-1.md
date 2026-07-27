## Review — Phase 40 release-governance scope split

**Verdict: NOT APPROVED** (9 actionable findings; the core scope decision is sound, the guard rails around it are not).

### What holds up (verified, not assumed)

- **Failure inventory is faithful.** All 20 slugs preserved verbatim; the 411-vs-412 reconciliation is correct — `leetcode_profile_manifest.json` declares `expected_fixture_count: 411` and `corpora/leetcode/src/*.sifr` is exactly 411, with `full-corpus-taxonomy-smoke` accounting for the 412th lane variant.
- **Release lane is still guarded against silent corpus erosion.** `run_representative_subset` calls `load_profile_manifest()` (`runner.py:327`), which runs the full `validate_profile_manifest` — enforcing `corpus_root` identity, `expected_fixture_count == len(glob("*.sifr"))` (`runner.py:269-278`), and that the 12 representative rows cover **every** declared category (`runner.py:263-265`). Deleting a fixture or dropping a category fails release, not just nightly. This is the strongest part of the change.
- **No coverage removed.** nightly and merge selections untouched; no baseline, exclusion, or status reclassification; `algorithmic_compatibility_profile` stays `blocking`.
- **`profile_assignment_matrix.json` was updated in lockstep**, so `validate_row_membership` remains truthful.
- **`matrix_referenced_areas` addition is inert-safe** — `algorithmic_compatibility` is already pulled in via `merge_suite`, so no new area is forced into strict manifest policy.
- **No release-evidence incompatibility.** Nothing in `distribution_release` governance, `release_evidence.py`, or the qualification schemas references `leetcode-full`; `profile_runner.run_algorithmic_compatibility_suites` drives purely off `selected_suites_for_area`.

---

### Findings

**1 — HIGH · Stale durable doc contradicts the change, and it was missed by the consistency sweep.**
`verification/README.md:81-85` still reads: *"`nightly` and `release` run the same readiness coverage suite plus broader full/generated/profile-owned suites such as **full algorithmic compatibility**…"*. This is now false for `release` and is a durable, checked-in claim outside the changed file set. Fix: split the sentence so full algorithmic compatibility is attributed to nightly only, with a pointer to the new Algorithmic Corpus Policy section.

**2 — HIGH · `release_suite` is an unenforced governance record; the divergence can silently rot.**
`coverage_matrix.py:225-226` only asserts *"if present, must be a non-empty string."* Nothing checks that `release_suite` matches `profile_assignment_matrix.json`'s `release` list, that its `area:suite` tokens even exist, or that a row **must** declare `release_suite` when its release assignment diverges from `nightly_release_suite`. Concretely: a later edit to `release.json` + `profile_assignment_matrix.json` dropping `representative-subset` from release would pass every gate while `compiler_surface_matrix.json:333` continues to advertise it. The field is documentation dressed as enforcement.
Fix: in `profile_assignment_matrix.py`, resolve each surface row's effective release suite (`release_suite` if present, else `nightly_release_suite`) and require set-equality with `rows[].profiles.release`; run its tokens through `validate_expected_tokens` for existence.

**3 — HIGH · No negative self-test for the new enforcement claim.**
`coverage_matrix_readiness_self_test.py` carries 16 cases — one per readiness enforcement claim — which is the contract this area established in the wave-10 closeout. The `release_suite` branch adds none. Per finding 2's fix, add at least a `release_suite` drift case that lets production code emit the error (following the `profile_assignment_mismatch` pattern at L273, which deliberately calls `validate_row_membership` rather than re-implementing it).

**4 — HIGH · The relaxation has no expiry and no machine-readable owner.**
The matrix already has exactly the right mechanism for time-boxed divergence: `TEMPORARY_STATUSES` requires `issue` + `expiry` and errors on `expiry has passed` (`coverage_matrix.py:238-258`). Keeping the row `blocking` (defensible — both release suites *are* blocking) routes around it, so the release carve-out carries no deadline and the executable registry contains no reference to `ad-hoc-algorithmic-full-corpus-preexisting-failures.md`. `profile_policy.md`'s *"tracked from the phase index"* is prose an automated gate cannot read. Fix: require `release_divergence_issue` (path must exist) and `release_divergence_expiry` (must not be past) whenever `release_suite` is present. This is what converts "temporary Phase 40 carve-out" from an intention into a fail-closed contract.

**5 — MEDIUM · The guarantee layer still asserts release coverage by the full corpus.**
`shipped_guarantees.json:120` — the **stable** guarantee `ecosystem-stdlib-algorithm-rule`, whose `public_doc_path` is `verification/policy/ecosystem_compatibility.md` — declares `"nightly_release_surface": "algorithmic_compatibility_profile"`, and the guarantee schema has no release-specific field. The divergence is recorded one layer down (surface row) but not at the layer that maps to the public guarantee. Either add the qualification at the guarantee row or state explicitly in the new policy section that guarantee-level `nightly_release_surface` is nightly-authoritative for this guarantee.

**6 — MEDIUM · `profile_policy.md` overstates what taxonomy-smoke does.**
The new section calls it *"the full-corpus taxonomy self-test"* and says *"This preserves coverage across every declared algorithm category."* `run_taxonomy_smoke` (`runner.py:485-515`) invokes `build_full_corpus_failure_taxonomy.py` against the **static** `data/taxonomy_smoke_results.json` and asserts exactly one synthetic failing case in one category. It compiles zero corpus fixtures. The category-coverage property comes entirely from `representative-subset` (see "What holds up"). As written, a reader — or a future auditor of the GA decision — will over-read the release lane's strength. Reword: representative-subset preserves per-category coverage and pins corpus size; taxonomy-smoke keeps taxonomy generation executable. `taxonomy self-test` in the ad hoc issue and roadmap is fine; it's the `full-corpus` qualifier here that misleads.

**7 — MEDIUM · Self-contradiction inside `profile_policy.md`.**
Line 12 still defines `release` as *"highest-confidence release qualification profile"*, while the section appended at line 120 makes release strictly weaker than nightly for this area. Add a qualifier or cross-reference at line 12. Separately, the enumeration at lines 44-50 of what the readiness suite rejects should gain the `release_suite` rule once finding 2 makes it real.

**8 — LOW · Inaccurate resource-class declaration in `release.json`.**
The selection is now `"resource_classes": ["long-running"]`. `merge.json` classifies the same `representative-subset` as `["default-local"]`, and neither remaining suite is long-running now that the corpus run is gone. Dropping `external-corpus` is defensible (merge sets the precedent), but the entry now claims a class it doesn't need and omits the one merge assigns it. Functionally inert — `selected_resource_classes` unions with `resource_policy.classes`, which still lists all three — but this is declarative metadata that governance reads. Should be `["default-local"]`.

**9 — LOW · Phase 40 carve-out is filed under the wrong milestone for discoverability.**
The note lands in `milestone_40_4` (Documentation and VS Code), mirroring the packaged-Rust precedent. But release-qualification *scope* is defined by `milestone_40_1: Canonical Release Plan and Qualification`, already `[x]`, and the exit criterion it feeds is *"Release profile and all Phase 40 suites pass on the final source commit"* under Final Phase Closure. An auditor checking 40_1's completed qualification claim or the closure criterion will not see the carve-out. Add a one-line cross-reference from `milestone_40_1` or from Frozen GA Decisions — this is a frozen GA scope decision, which is where the other such decisions live.

---

### On the scope decision itself

Truthful and fail-closed as a *decision*: nothing is hidden, nightly stays blocking, both release suites stay blocking, corpus size and category coverage remain enforced in the release lane, and the ad hoc issue correctly makes restoring `leetcode-full` to release a closeout acceptance criterion rather than a Phase 40 prerequisite — which matches the user's direction. No weakening beyond algorithmic full-corpus scope was found.

The gap is durability, not honesty. Findings 2, 3, and 4 are the substantive ones: as landed, the "temporary" release divergence is recorded in three hand-maintained places with no gate tying them together and no clock, which is precisely the drift class this area's closeout machinery was built to prevent. Findings 1 and 6 are correctness-of-record issues in durable docs. Address 1–4 and 6 and this is approvable.
