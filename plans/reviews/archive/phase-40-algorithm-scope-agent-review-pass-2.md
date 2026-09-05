## Review — Phase 40 algorithmic release-scope, pass 2

**Verdict: NOT APPROVED** (1 high, 1 medium, 4 low). The nine pass‑1 findings are all addressed at the level the prompt asked for; the high finding is the half of pass‑1 finding 2 that was not implemented, and I confirmed it is exploitable, not theoretical.

### Prior findings — verified closed

| # | Claim | Verified |
|---|---|---|
| 1 | README no longer claims release runs full algorithmic compatibility | `verification/README.md:81-86` now attributes the pinned corpus to nightly only; release text points at `policy/profile_policy.md`. Remaining suite names (`generated_code_quality:full`, `performance:full`, `distribution_release:full`, `stdlib_parity:module-full-check`, `ecosystem-broader`, `sanitizer-full`) are still genuinely selected by release — checked against `release.json`. |
| 2 (partial) | `release_suite` machine-enforced, ids align | Enforced. I ran `profile_assignment_matrix.py` against mutated copies: drift → `release_suite does not match release profile assignment: advertised=[…] assigned=[…]`; bogus token → `references unknown suite`. Set equality, not subset. `profile_assignment_matrix.json:60` renamed to `algorithmic_compatibility_profile`, which is the real `compiler_surface_matrix.json:326` `surface_id` and the `shipped_guarantees.json:120` `nightly_release_surface` target. |
| 3 | Negative self-tests for drift + expiry | `coverage_matrix_readiness_self_test.py:285-309`; both delegate to production code. Ran it: `cases=18`. |
| 4 | Fail-closed divergence with indexed record, target existence, expiry | `coverage_matrix.py:265-297`. `ALG-CORPUS` is indexed at `plans/phases/index.md:51`; its `../issues/active/…` link resolves to a real file; `2026-10-31` expiry errors once past. Metadata without `release_suite` is rejected (`:229-230`). |
| 5 | Guarantee-layer authority explicit | `profile_policy.md:134-138` states the registry's `nightly_release_surface` is nightly-authoritative and the surface row's `release_suite`/record/expiry is release-authoritative. |
| 6 | Correct attribution of coverage properties | `profile_policy.md:126-131` now credits corpus-size pinning and per-category coverage to representative-subset and only taxonomy executability to taxonomy-smoke. Matches `runner.py`. |
| 7 | Policy release bullet + rejection inventory | Bullet qualified (`:11-13`); inventory extended (`:50-51`) — but see L2 below. |
| 8 | Resource class | `release.json:186-188` is `["default-local"]`, matching merge's classification of the same suite; `resource_policy.classes` unchanged. |
| 9 | `milestone_40_1` cross-reference | Added at `phase-40-stable-channel-ga-execution.md:79-81`. |

Also confirmed: all 20 slugs byte-identical; nightly still `leetcode-full` + `taxonomy-smoke` (`nightly.json`, untouched); release selects `representative-subset` + `taxonomy-smoke` with no advisory/non-blocking flag and row `status: blocking`; no fixture, baseline, exclusion, or manifest-count change anywhere in the diff; restoring `leetcode-full` is acceptance criterion + closeout gate in the ad hoc issue; nothing in `plans/releases/`, `distribution_release`, or `docs/` references algorithmic scope, so no release-evidence incompatibility; file-size guardrail PASS (largest touched file 666 lines); no demos added.

---

### Findings

**1 — HIGH · The divergence declaration is itself deletable with zero gate failure.**
`profile_assignment_matrix.py:65-75` only validates the *present* branch: if `release_suite` exists it must equal `profiles.release`. Pass‑1 finding 2 asked for the effective-suite form — `release_suite` if present, **else `nightly_release_suite`** — and the else branch was not implemented. `nightly_release_suite` is now compared against a profile in exactly one place (`coverage_matrix.py:272`, only to require it differ from `release_suite`).

Demonstrated, not inferred: deleting `release_suite`, `release_divergence_record`, and `release_divergence_expiry` from `compiler_surface_matrix.json:333-335` while leaving `release.json` and `profile_assignment_matrix.json` reduced yields `profile assignment matrix ok: rows=17`, rc=0, and a clean coverage-matrix run — with the row then advertising `nightly_release_suite: leetcode-full,taxonomy-smoke` as its release coverage. So the expiry clock and the ALG-CORPUS owner record can be removed in a three-line JSON edit and release stays permanently reduced behind a matrix that claims otherwise. That is the exact drift class findings 2 and 4 were about, and it applies to every other surface row too (all 16 currently have `release` assignment == `nightly_release_suite`, so nothing else is affected today).
Fix: in `load_release_surface_suites`/`main`, resolve every surface row's effective release suite (`release_suite` else `nightly_release_suite`) and require set-equality with `profiles.release`; add a negative self-test for undeclared divergence, so the "release reduction requires a record and a clock" rule is what fails, not just its bookkeeping.

**2 — MEDIUM · The new evidence paragraphs name a performance step release does not run, and contradict the existing PERF-HOST ledger.**
`ad-hoc-algorithmic-full-corpus-preexisting-failures.md:72-73` says the canonical release attempt "passed … documentation, and the representative performance budget", and `phase-40-stable-channel-ga-execution.md:112` repeats "including representative performance—passed". `release.json` sets `legacy_facade.performance_budget: "full"`; `representative` is merge-only (`merge.json`), and `run_performance_budget_checks` (`profile_runner.py:562-568`) dispatches straight off that mode. Worse, the same issue file at `:508-513` records the merge lane *stopping* at the representative performance budget on two host-sensitive medians (the indexed `PERF-HOST` condition). So the new sentence either mislabels the release run's step or asserts a pass for a step the document elsewhere records as overrunning. This paragraph is the stated evidence for the GA scope carve-out, so it should be exact.
Fix: name the actual step (`performance_budget_checks` in `full` mode) in both places.

**3 — LOW · `release.json:4` still claims unqualified "full breadth".**
`profile_policy.md:11-13` was correctly qualified, but the machine-readable profile description — the peer of the `resource_classes` metadata that pass‑1 finding 8 fixed — still reads *"Highest-confidence local qualification gate with full breadth …"*. Add the same "except where an expiry-bound release policy applies" qualifier.

**4 — LOW · Rejection inventory names a rule that does not exist, and the one that does has no self-test.**
`profile_policy.md:50-51` lists "expired or **ownerless** release divergence". There is no divergence owner field; the implemented rule is a missing or unindexed `release_divergence_record` (`coverage_matrix.py:274-286`). That path — plus `release_suite == nightly_release_suite` and metadata-without-`release_suite` — carries no negative self-test, while drift and expiry do. Reword to "missing or unindexed release divergence record" and add the third case, keeping the area's one-case-per-claim contract intact.

**5 — LOW · The record lookup is brittle and can traceback.**
`coverage_matrix.py:276-286` calls `tracking_path.read_text()` unguarded (a moved/renamed `plans/phases/index.md` raises `FileNotFoundError` out of the readiness check instead of emitting an error), matches with `line.startswith(f"| {record} |")` so cosmetic column padding in that markdown table breaks the check, and takes the first `](…)` on the row so a link in the Title cell would silently resolve the wrong target. Guard the read and match the record by splitting the row on `|` and stripping cells.

**6 — LOW · Undocumented id-convention coupling.**
14 of the 17 `profile_assignment_matrix.json` rows use area-style ids that are *not* `compiler_surface_matrix.json` `surface_id`s; only this row now matches. The new `release_suite has no profile assignment row` check depends on that equality, and nothing validates the PAM→CSM id mapping. Any future `release_suite` on a row whose PAM id follows the older convention will emit a spurious error. Worth a one-line note in `profile_policy.md` (or a mapping check) so the next author knows which convention binds.

---

### On the decision

Unchanged from pass 1 and reconfirmed: the scope split is honest, nightly stays blocking, both release suites stay blocking, and the release lane still enforces corpus size and per-category coverage through `load_profile_manifest`. Finding 1 is the one that matters — without the else branch, the "temporary, expiry-bound" contract protects the divergence's *content* but not its *existence*. Close 1 and 2, and 3–6 are mechanical.

**NOT APPROVED**
