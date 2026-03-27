# Phase 31 Ad Hoc Follow-up Milestones — Review Pass 1

**Date:** 2026-03-26
**Scope:** PRs #1431 (m31_k) and #1432 (m31_i), plus holistic review of all milestone closure claims in `issues/phase31-ad-hoc-followup-milestones.md` and `issues/phase31-strategy-synthesis-review.md`.

---

## Verdict Summary

| Area | Assessment |
|------|-----------|
| Correctness of closed milestones | Partially justified — see NO_ORACLE concern below |
| Regressions introduced | None detected — last 5 commits are fixture-only |
| Missing tests | Yes — NO_ORACLE gap and regression cases unaddressed |
| Closure claims justified | PASS closures: yes. NO_ORACLE closures: weaker than presented |

---

## 1. PR #1431 — m31_k Canonical Fixture Normalization (0043)

**What it did:** Rewrote `0043_multiply_strings` to use explicit `parseDigit`/`parseNumber` helpers instead of unchecked `int(str)`, respecting Sifr's parse-safety guarantee.

**Assessment: Sound.**

- Root cause correctly identified as a language-policy mismatch, not a compiler bug.
- The canonical rewrite preserves the algorithm shape (schoolbook multiply, then stringify).
- Status moved to `PASS` with `embedded_asserts` oracle — assertions actually execute and pass.
- Consistent with the raw-source divergence policy defined in the carry-forward plan.
- No compiler changes required — fixture-only PR.

**No issues found.**

---

## 2. PR #1432 — m31_i Multi-Solution Fixture Canonicalization (0215, 1046)

**What it did:** Reduced multi-solution scraped fixtures to single canonical implementations:
- `0215`: sorting-based kth-largest with explicit `mut` parameter.
- `1046`: pop-based stone reduction with explicit `mut` parameter, replacing private `heapq._heapify_max` usage.

**Assessment: Correct canonicalization, but closure status is weaker than it appears.**

- Both cases moved from `CHECK_ERROR` to `NO_ORACLE`.
- The fixtures contain embedded `assert` statements that exercise the algorithms.
- However, the seed corpus manifest declares `oracle.mode = "no_oracle"` for both cases.
- The verification runner only counts a case as `PASS` when `oracle.mode == "embedded_asserts"`.
- **Result:** The assertions are dead code in the verification pipeline — they execute during `sifr run` but the runner does not check their outcome for status determination.

**Finding [F1]:** The closure claim "m31_i owner scope is now closed" is technically true but materially weaker than a PASS closure. It means "compiles and runs without crashing," not "produces correct output."

**Recommendation:** Update the seed corpus manifest for 0215 and 1046 to `oracle.mode = "embedded_asserts"` so the existing assertions are actually validated.

---

## 3. NO_ORACLE Gap — Systemic Concern

This is the most significant finding in the review.

**14 cases are closed as NO_ORACLE across the following milestones:**

| Milestone | NO_ORACLE cases |
|-----------|----------------|
| m31_a | 0127, 0502, 0743 |
| m31_b | 0226, 0295, 0703 |
| m31_d | 0207, 0684 |
| m31_e | 0100, 0102, 0235 |
| m31_i | 0215, 1046 |
| m31_l | 0110 |

**The issue:** All of these fixture files contain embedded `assert` statements, but the seed corpus manifest declares them as `no_oracle`. The verification runner therefore never validates their assertion outcomes. A fixture that compiles, runs, and silently produces wrong answers would still report `NO_ORACLE` (green).

**Finding [F2]:** The NO_ORACLE closure category creates a false sense of validation. The strategy review and milestone documents do not distinguish between "assertions verified" (PASS) and "just compiled and ran" (NO_ORACLE) when declaring milestone scopes closed.

**Finding [F3]:** The verification policy documentation (`phase31_leetcode_corpus_policy.md`) describes `no_oracle` as "the fixture currently has no embedded sample assertions." This is factually incorrect for at least 14 of the 14 NO_ORACLE cases checked — all have assertions.

**Recommendation:**
1. Audit all NO_ORACLE seed corpus entries and update those with embedded assertions to `oracle.mode = "embedded_asserts"`.
2. Re-run the verification harness after the manifest update to confirm the assertions actually pass.
3. If any assertions fail, those cases are not actually closed and should be re-opened.

---

## 4. Snapshot Regressions — Unresolved

The strategy review documents a regression between 2026-03-13 (`PASS=15`) and 2026-03-21 (`PASS=13`):

| Case | Regression | Current status |
|------|-----------|---------------|
| 0007 | `PASS -> CHECK_ERROR` | Unresolved — classified as "canonical fixture adaptation, explicit `mut`" |
| 0009 | `PASS -> CHECK_ERROR` | Unresolved — classified as "canonical fixture adaptation, explicit `mut`" |
| 0151 | `PASS -> CHECK_ERROR` | Unresolved — classified as "canonical fixture adaptation, explicit `mut`" |
| 0078 | `CHECK_ERROR -> RUN_ERROR` | **Resolved** in m31_d slice 1 (now `PASS`) |

**Finding [F4]:** Three cases that previously passed now fail, and no milestone owns their resolution. The strategy review correctly identifies them as needing explicit `mut` adaptation, but they are not assigned to any specific milestone in the carry-forward plan. They appear in the "Canonical Sifr mutability / ownership adaptation" bucket alongside 7 other cases, but that bucket has no corresponding `m31_*` milestone with a definition of done.

**Recommendation:** Create a milestone (or expand an existing one) to own the `mut` adaptation bucket. The 0007/0009/0151 regressions should have explicit closure targets since they represent real backward movement.

---

## 5. PASS Closures — Verified Sound

The following milestone closures achieved `PASS` status with `embedded_asserts` oracle mode and are well-justified:

| Milestone | Cases | Status |
|-----------|-------|--------|
| m31_d | 0017, 0050, 0052, 0078, 0090, 0912 | PASS (assertions verified) |
| m31_h | 0015, 0424 | PASS (assertions verified) |
| m31_j | 1299 | PASS (assertions verified) |
| m31_k | 0043 | PASS (assertions verified) |
| m31_b (partial) | 0997, 1209 | PASS (assertions verified) |

These 11 cases have strong closure evidence: they compile, run, and their embedded assertions pass.

---

## 6. Compiler Stability

**No compiler regressions detected.** The last 5 merged commits (m31_h through m31_i) are fixture-only changes — no modifications to `crates/sifr_hir`, `crates/sifr_codegen`, or any other compiler crate. The last compiler-touching commit was `cf75ff1e` (m31_b recursive field boxing), which modified `lower_stmt.rs`, `stmt_support_emitter.rs`, `hir_nodes.rs`, and `statements.rs`.

The 12 existing Phase 31 e2e pass tests continue to provide regression coverage for the compiler changes landed earlier in the phase.

---

## 7. Test Coverage Assessment

**Adequate:**
- 12 e2e pass tests covering Phase 31 compiler features (narrowing, tuple unpack, stdlib compat, etc.).
- 1 e2e fail test for arity checking.
- 8 demo files covering milestones m31_b through m31_l.
- 50+ verification result JSONs documenting wave-by-wave status transitions.

**Missing:**
- **[F5]** No new e2e pass/fail tests were added in PRs #1431 or #1432. These are fixture-only canonicalizations so no compiler feature was added, but the pattern of adding zero regression tests for canonicalization work means there is no guard against future fixture breakage from compiler changes.
- **[F6]** The m31_h and m31_j closures (PRs #1429, #1430) also added no new e2e tests. These involved fixture rewrites that exercise specific language features (`mut` parameters, local shadowing) — a targeted e2e test for each would strengthen confidence.

---

## 8. Documentation Consistency

**Finding [F7]:** The strategy synthesis review and the milestones document are internally consistent. Case classifications match between the two documents. Execution log entries reference the correct result artifacts and demos.

**Finding [F8]:** The milestones document's "Definition of done" for m31_i states: "current expectation: 0215 and 1046 both fall into m31_a_optional_flow_completion after canonicalization." However, m31_a is already marked as closed. If these cases have residual issues beyond compilation, there is no active milestone to own them.

---

## Findings Summary

| ID | Severity | Description |
|----|----------|-------------|
| F1 | Medium | m31_i closure is NO_ORACLE, not assertion-verified |
| F2 | High | 14 NO_ORACLE closures across 6 milestones lack assertion validation |
| F3 | Medium | Verification policy docs incorrectly describe NO_ORACLE cases as "no assertions" |
| F4 | High | Three snapshot regressions (0007, 0009, 0151) have no owning milestone |
| F5 | Low | No e2e tests added for canonicalization PRs #1431, #1432 |
| F6 | Low | No e2e tests added for fixture-rewrite PRs #1429, #1430 |
| F7 | None | Documentation is internally consistent |
| F8 | Medium | m31_i post-canonicalization residuals have no active owner (m31_a is closed) |

---

## Recommended Actions

1. **Upgrade NO_ORACLE manifest entries** — Audit all 14 NO_ORACLE cases, update seed corpus to `embedded_asserts` where fixtures have assertions, and re-run verification. This is the highest-leverage action: it either confirms 14 closures or surfaces real correctness bugs.

2. **Create a `mut` adaptation milestone** — The 0007/0009/0151 regressions plus the broader `mut` bucket (0127, 0226, 0746, 0912) need an explicit owner with a definition of done.

3. **Fix verification policy docs** — Update `phase31_leetcode_corpus_policy.md` to accurately describe the NO_ORACLE/embedded_asserts distinction.

4. **Clarify post-canonicalization ownership** — If m31_a is closed but 0215/1046 have residual issues, assign them to a follow-on or re-open m31_a for those specific cases.
