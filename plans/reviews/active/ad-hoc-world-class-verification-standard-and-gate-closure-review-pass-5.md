I've verified the patch deltas against the plan and the pass-4 artifact. All seven pass-4 required edits have landed (lines 109, 111, 121, 125, 536–543, 901, 68/174 for `sifr_ir` seed-tests).

# Findings

## P0 — Plan-blocking

**None.** All pass-4 P1 items are closed. Verified:

- `legacy_facade` migration surface decision is explicit and bounded out of scope (line 109).
- Suite schema v1 retention + area-owned metadata is explicit (line 111).
- Secondary bins/features default classification (`internal` vs `performance`) with Wave 1 enforcement gate (line 121).
- Local `python3` prerequisite declared as setup failure, not host skip (line 125).
- Wave 5 expanded to 5.1–5.8 with the parsed-source layer at 5.1 (lines 536–543); the eight-layer task list at lines 547–554 maps 1:1.
- Wave 9.3 added to gate-expanding wall-time tracking (line 901).
- `sifr_ir` either/or removed: Decisions §68 names it the initial seed-test target, Wave 1 §321 requires seed tests, and §174 confirms 0 unit tests today. `tests:none` is reserved for future data-only crates with issue + expiry and illegal at phase close. No remaining either/or.

Local-first authority is preserved cleanly: §Decisions, §Resolved Decisions (Hermetic local-first rule, Local/CI parity, Reference host, Local Python prerequisite), and §Acceptance Criteria all keep merge offline and reject CI as source of truth.

## P1 — Should fix before treating as final

**None.** No surviving contradictions or fallback/silent-quarantine loopholes:

- Red-blocker policy (lines 69, 76, 114, 132) explicitly bans silent quarantine.
- Quarantine confined to flake/crash policy with issue, owner, expiry, reproduction (line 83).
- No-fallback rule (line 84) and Non-Acceptable Closeout States (lines 948–966) close the back door.
- The Wave 0 advisory→Wave 10 blocking promotion is explicit and tied to the `closes_in_wave` field with a 1–9 range, preventing perpetual `expected-missing` drift.

## P2 — Tightening, not blocking

**1. Test-only bin/feature classification is not in the §121 default table.**
The Secondary-bins/features decision scopes only "non-test bins and features." Discovery (line 242) flags `sifr_stdlib`'s fixture bin and `__test_fixture` feature as test-only internal surfaces and says Wave 1 assigns them to "merge/nightly fixture validation or explicit internal-no-run classification." The Wave 1 guardrail catches the gap at implementation time, but the §Resolved Decision could explicitly name a third category (e.g., `test-fixture: validated by owning test suite`) so the matrix has no ambiguous row.

**2. Local/CI plan-equivalence is added as a skeleton but never explicitly promoted to blocking.**
Wave 0 adds the skeleton (line 285). Wave 1 adds profile-plan emission (line 324). Acceptance criterion at line 920 only requires that the checking "exists." Wave 10 promotes the *coverage-matrix* and *profile-assignment-matrix* checks to blocking but does not name plan-equivalence. The skeleton can satisfy "exists," but if intent is that local/CI plan drift fails merge at phase close, Wave 10 should name the plan-equivalence check in its promotion list.

**3. Wave 0 remains the largest unsplit wave.**
Pass-4 P2#7 noted this and was not addressed. Implementable but heavy: schemas v2 + coverage-matrix area + shipped-guarantees registry + advisory check + promotion switch + hermetic/offline fields + plan-equivalence skeleton + policy doc updates. Optional sub-PR split (0.1 schemas, 0.2 coverage matrix + registry, 0.3 hermetic/offline + plan-equivalence + docs) matches the Wave 2/5/9 pattern.

**4. Wave 5 task body and sub-PR list now align, but task line 547 still bullet-lists the layers without numbering.**
Pure cosmetic — implementer can read sub-PR list at 536–543 for the canonical order, but renumbering the task layers to match 5.1…5.8 would remove any ambiguity that sub-PR 5.8 maps to the crash/ICE contract paragraph rather than to something else.

**5. Duplicate "Verified during this planning pass" sub-headers from pass 3 were never merged.**
Single block recommendation from pass 4 P2#6 is still open. Cosmetic.

---

# Required Edits

None blocking. If the author wants a final polish pass before kickoff, the minimal patch is:

- §Resolved Implementation Decisions / Secondary bins/features (line 121): add a sentence — "Test-only bins and features (e.g. `sifr_stdlib`'s `__test_fixture` feature and fixture bin) are classified as `test-fixture` and validated by their owning test suite; they do not need a separate runtime profile membership."
- §Wave 10 (around line 871): extend the promotion bullet to include local/CI plan-equivalence — "Promote the coverage-matrix check **and the local/CI plan-equivalence check** from advisory to blocking."
- §Wave 0 (after line 254): optional sentence noting that Wave 0 may ship as `0.1 schemas v2`, `0.2 coverage matrix area + shipped-guarantees`, `0.3 hermetic/offline + plan-equivalence skeleton + policy docs` if size becomes prohibitive.

None of these gate Wave 0 from starting.

---

# Verdict

**Elegant / implementation-ready.**

All Wave 0 decisions are made, no contradictions remain, no silent-quarantine or fallback loopholes survive, and local-first authority is preserved emphatically across §Decisions, §Resolved Decisions, §Acceptance Criteria, and §Non-Acceptable Closeout States. The remaining P2 items are polish — none requires another review pass. Wave 0 can begin.
