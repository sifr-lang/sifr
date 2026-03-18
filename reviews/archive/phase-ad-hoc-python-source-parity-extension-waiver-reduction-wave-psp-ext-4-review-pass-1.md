# Review: wave_psp_ext_4 (Waiver Ledger Reduction and Exit-Closure Governance Updates)

**Phase:** `ad-hoc-python-source-parity-extension-waiver-reduction.md`
**Wave:** `wave_psp_ext_4` — Waiver Ledger Reduction and Exit-Closure Governance Updates
**Review type:** Completion-gap review (pass 1)
**Date:** 2026-03-18

---

## Scope Review

Per `issues/ad-hoc-python-source-parity-extension-waiver-reduction.md:299-312`, wave_psp_ext_4 owns:

1. Publish the post-iterator successor governance inventory
2. Update all affected wave ledgers
3. Shrink or replace the old lazy waivers
4. Align architecture/public wording with actual post-phase behavior

**Definition of done:**
- No broad lazy-iterator waiver remains where the predecessor architecture already removed the root cause
- No affected surface remains in `open` state
- The repo has one clear post-phase account of what is now parity-closed, what remains intentionally different, and why

---

## Completion Gap Analysis

### ✅ Governance Inventory Updated

The `milestone_psp_7_parity_governance_inventory.md` has been comprehensively updated:

| Section | Updated | Evidence |
|---------|---------|----------|
| Canonical Builtin Parity Inventory | ✅ | Lines 30-34 document `reversed`, `enumerate`, `zip`, `map` as iterator-returning |
| Per-Module Closure Inventory | ✅ | Lines 64, 72, 78, 81 reference `wave_psp_ext_3` additions |
| Canonical CPython Adopt/Adapt/Waive Ledger | ✅ | Lines 111-114 document all extension waves |
| Waiver Index | ✅ | Lines 116-148 contain precise waiver entries with rationale and revisit rules |

### ✅ Wave Traceability Updated

| Ledger | Status | Evidence |
|--------|--------|----------|
| `wave_psp_a1_cpython_traceability.md` | ✅ Updated | Lines 28-32 document iterator-returning behavior for `reversed`, `enumerate`, `zip`, `map` |
| `wave_psp_b2_cpython_traceability.md` | ✅ Updated | Lines 7, 19-20 document iterator-returning itertools combinators |
| `wave_psp_d1_cpython_traceability.md` | ✅ Updated | Lines 8-9 document `iterdir/glob/rglob/iglob` as iterator-returning |
| `wave_psp_e1_cpython_traceability.md` | ✅ Updated | Line 8 documents `finditer` as iterator-returning |

### ✅ Architecture Documentation Updated

| Document | Update | Evidence |
|----------|--------|----------|
| `internal_docs/architecture.md` | ✅ Added wave_psp_ext_4 to milestones | Line 738: Documents waiver-reduction phase |
| `internal_docs/phases/07_stdlib_parity.md` | ✅ Clarified iterator semantics | Lines 66-67: "Iterator[T] is a first-class type and is **not** implicitly assignable to `list[T]`" |
| `internal_docs/phases/12_stdlib_remediation.md` | ✅ Updated Path.glob/rglob contracts | Lines 63-64: Document iterator-returning contracts |
| `internal_docs/roadmap.md` | ✅ Updated phase status | Line 55: References continuation phase and wave closure |

### ✅ Execution Ledger Updated

| Document | Status |
|----------|--------|
| `issues/ad-hoc-python-source-parity-extension-waiver-reduction-execution.md` | ✅ Updated with wave_psp_ext_4 validation |

---

## Waiver Ledger Analysis

### ✅ Broad Lazy-Waiver Retirement

The phase planning document explicitly required retiring "the broad 'lazy iterator object families' waiver." The governance inventory now correctly classifies:

- ✅ Core iterator surfaces: `parity-closed` (via predecessor phase)
- ✅ Approved itertools combinators: `parity-closed` (line 20 of wave_psp_b2)
- ✅ Regex/filesystem iterator surfaces: `parity-closed` (via wave_psp_ext_3)

### ✅ Materialize-Then-Iterate Documentation

Per wave_psp_ext_3 reviews, the implementation uses a "materialize-then-iterate" pattern. This is correctly documented in the governance inventory (line 128):

```
| Materialize-then-iterate behavior behind iterator-returning regex/filesystem surfaces | intentional-diff | Public contracts are iterator-returning and type-safe, but current intrinsic layer computes full match/path lists before yielding iterator values. |
```

**Assessment:** This correctly documents the implementation reality with a clear revisit rule.

### ⚠️ Minor Gap: tee/groupby Explicit Waiver

**Finding:** `itertools.tee` and `itertools.groupby` are documented as `intentional-diff` in the milestone governance inventory (line 125), but are NOT explicitly mentioned in `wave_psp_b2_cpython_traceability.md`.

| Location | tee | groupby |
|----------|-----|---------|
| `milestone_psp_7_parity_governance_inventory.md:125` | ✅ Listed | ✅ Listed |
| `wave_psp_b2_cpython_traceability.md` | ❌ Not mentioned | ❌ Not mentioned |

**Assessment:** This is a **minor documentation clarity gap**. Someone reading only the wave traceability document might not realize tee/groupby are out of scope. However, this is NOT an actionable defect because:
1. The governance inventory correctly lists them
2. The phase planning document correctly identifies them as residual waiver candidates
3. The wave definition of done is satisfied (broad waiver is retired, narrow residual entries exist)

---

## Root Cause Quality

### ✅ No Stale Claims Remain

The governance inventory correctly reflects post-phase reality:

| Claim Type | Pre-Wave State | Post-Wave State |
|------------|----------------|-----------------|
| `reversed` return shape | "closed through eager adaptation" | "Iterator-returning contract is closed" |
| `itertools` lazy model | "broad waiver" | "Approved combinators are parity-closed" |
| `glob.iglob` | "not in scope" | "iterator-returning contract" |
| `re.finditer` | "not in scope" | "iterator-returning contract" |

### ✅ No Open Parity States

The governance inventory confirms (line 156): "No `open` parity state is carried in this milestone inventory."

---

## Safety / Consistency Verification

### ✅ Test Suite Validation

| Test Suite | Result |
|-------------|--------|
| `cargo test -p sifr -- --skip test_e2e_pass` | ✅ PASS (37 tests) |
| `cargo test -p sifr_hir` | ✅ PASS (121 tests) |
| `scripts/run_all_tests.sh --profile quick` | ✅ PASS (wall 59.52s, e2e 24/24) |
| Validation contract matrix | ✅ PASS (14 rows) |

### ✅ Documentation Consistency with Shipped Behavior

| Shipped Behavior (Waves 1-3) | Documented in Governance |
|-------------------------------|--------------------------|
| `reversed` returns `Iterator[T]` | ✅ `milestone_psp_7:30` |
| `enumerate` returns `Iterator[T]` | ✅ `milestone_psp_7:31` |
| `zip`/`map` return `Iterator[T]` | ✅ `milestone_psp_7:32` |
| itertools combinators return iterators | ✅ `wave_psp_b2:7,20` |
| `glob.iglob` returns `Iterator[str]` | ✅ `wave_psp_d1:9` |
| `Path.iterdir/glob/rglob` return iterators | ✅ `wave_psp_d1:8` |
| `re.finditer` returns `Iterator[Match]` | ✅ `wave_psp_e1:8` |

---

## Findings

### Finding 1: tee/groupby Explicit Waiver (Minor Documentation Gap)

**Severity:** Informational (not a blocker)

**Description:** `itertools.tee` and `itertools.groupby` are documented as `intentional-diff` in the milestone governance inventory but are not explicitly mentioned in `wave_psp_b2_cpython_traceability.md`.

**Evidence:**
- Governance inventory line 125: Lists tee/groupby as intentional-diff
- wave_psp_b2_cpython_traceability.md: Only lists approved combinators as parity-closed, does not mention tee/groupby

**Assessment:** This does NOT block wave closure because:
1. The milestone governance inventory correctly classifies these surfaces
2. The phase planning document identifies them as residual waiver candidates
3. The wave's definition of done is satisfied (broad lazy-waiver is retired, narrow residual entries exist)

**Recommendation:** For future clarity, consider adding an explicit "waiver not covered" entry in `wave_psp_b2_cpython_traceability.md` stating that tee/groupby are out of scope. However, this is NOT required for wave closure.

---

## Conclusion

**Wave Status:** ✅ **Ready for completion review**

- ✅ Governance inventory comprehensively updated
- ✅ All wave traceability ledgers updated
- ✅ Broad lazy-waiver retired to narrow residual entries
- ✅ Architecture/public wording aligned with shipped behavior
- ✅ No open parity states remain
- ✅ Test suite passes (all validation gates green)
- ✅ Documentation consistent with shipped iterator behavior

**Action Items:**
1. Proceed to production-grade review (pass 2)
2. No code changes required for wave closure
3. Optional (non-blocking): Consider adding explicit tee/groupby note to wave_psp_b2_cpython_traceability.md for clarity
