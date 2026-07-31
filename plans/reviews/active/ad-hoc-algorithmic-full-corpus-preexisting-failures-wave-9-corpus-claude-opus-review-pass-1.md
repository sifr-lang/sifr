Repo unchanged (only the pre-existing untracked items you told me to ignore). Reporting.

## Verdict: do NOT approve `1f732fae15967e7a9c308c5a798e6a22a5f7b738` as a fixture-surface closeout

The annotation itself is correct, minimal, and parity-preserving. But the premise behind it is wrong: **`0022_generate_parentheses` is not a pre-existing fixture defect — it is a compiler regression that this issue's own Wave 4 introduced.** I bisected it.

### Finding 1 — BLOCKING (correctness / root cause)
**Location:** `crates/sifr_lowering/src/lower/statements/statement_dispatch.rs:639-640` and `:749` (regression origin); masked at `verification/.../corpora/leetcode/src/0022_generate_parentheses.sifr:5`

Emitted type of the unannotated `res = []`, built from clean clones at each point (`emit` on the *unannotated* base fixture):

| commit | `res` lowering | build |
|---|---|---|
| `78d21d8d98` — pre-Wave-1 diagnosis base (PR #3064) | `Vec<String>` | passes |
| `789b359737` — Wave 3 merge (PR #3077) | `Vec<String>` via `__sifr_empty_list_literal` specialization | passes |
| `f1c34cf9aa` — **Wave 4 merge (PR #3079)** | `Vec<Box<dyn Any>>` | **fails** |
| `9e80f3a23` — current base | `Vec<Box<dyn Any>>` | fails (E0308 ×2, reproduced) |

This matches the parent issue exactly: `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md:156-189` enumerates all 23 latent `BUILD_FAIL`s and `0022` is not among them — it was one of the 368 `BUILD_PASS`.

Mechanism: Wave 4 added an unconditional save/restore of the deferred container-specialization map around nested-function lowering. The comment's intent (a *same-named nested binding* must not consume the enclosing patch) is right, but `:749` overwrites the whole map, so patches the nested body records for **captured enclosing names** are discarded too. `backtrack`'s `res.append("".join(stack))` therefore no longer specializes the enclosing `res = []`. Wave 4 also de-recursed `patch_stmt_container_specialization` (`crates/sifr_lowering/src/lower/container_literal_specialization.rs:281`); I probed that separately and it is currently covered by another path, so `:749` is the trigger.

This is a general, user-visible defect, reproducible with no corpus involvement — `check` says "no errors found", `build` emits non-compiling Rust:

```python
def f() -> list[str]:
    lit = []
    called = []
    def add(s: str):
        lit.append("z")                 # -> Vec<String>              (literal: OK)
        called.append("".join([s]))     # -> Vec<Box<dyn Any>>        (call result: broken)
    add("x")
    return lit + called
```

Per AGENTS.md ("solve root causes, not superficial symptoms") and this issue's own partition between *fixture surface* and *compiler defect*, the fix belongs in lowering. The issue's carve-out for "adjacent nested-function inference limitations" does not cover a defect the wave series itself created after the inventory was taken.

### Finding 2 — HIGH (evidence integrity)
`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` has no Wave 9 row and no mention of `0022`, and its inventory (23 `BUILD_FAIL` / 368 `BUILD_PASS`, `0022` in the pass set) now contradicts the stated base measurement of 410 PASS + 1 BUILD_FAIL. Whichever ledger row lands must record this as a Wave-4 regression with the bisection above, not as pre-existing fixture residue. Related: `plans/reviews/active/...-wave-9-corpus-claude-opus-review-pass-1.md` is **0 bytes** and is not approval evidence.

### Finding 3 — MEDIUM (regression risk / gate coverage)
`verification/areas/algorithmic_compatibility/runner.py:374-390` runs only `sifr check` per fixture, and `data/leetcode_profile_manifest.json` `full_corpus.command` is check-only. Nothing in the automated gate builds corpus fixtures, which is precisely why Wave 4's regression went undetected across Waves 5–8 (Wave 4's own evidence built only its 4 affected fixtures). A focused lowering/codegen regression test for the nested-capture patch shape is the minimum; without one, this class recurs silently regardless of the fixture annotation.

### Finding 4 — LOW (scope)
`src/0022_generate_parentheses.sifr` also drops one trailing blank line — outside the stated "annotates only `res`" scope. Not a normalization (125/411 fixtures still end with a blank line). Harmless.

### What I verified as sound
- **Diff:** exactly one tracked file; one annotation; algorithm and Python sibling byte-identical otherwise.
- **Minimal & idiomatic:** `list[str]` on `res` is the smallest sufficient change; 139/411 fixtures already annotate locals.
- **Generated Rust (head):** clean `Vec<String>` throughout, no `dyn Any`, nested `fn backtrack(..., res: &mut Vec<String>, ...)` consistent with the `list[str]` return.
- **Fixture behavior:** `check --isolated` → clean; `build --quiet --isolated` → release binary; binary runs, exit 0.
- **Python parity:** sibling runs clean; differential probe over `n = 1..6` — Sifr and CPython outputs match exactly (not just the fixture's `n=3` assert).
- **Corpus-wide:** independent sweep of all 411 top-level fixtures → `checked=411 failures=0`. All 6 other fixtures with the same "unannotated `= []` + nested `def`" shape (`0039`, `0040`, `0077`, `0210`, `0304`, `1462`) build clean, so `0022` really is the only corpus member of this class.
- **Consumers:** no consumer references `0022`; `expected_fixture_count: 411` unaffected; `data/leetcode_full_baseline_results.json` (411 PASS, zero failure records) needs no change since `0022` passed the check lane before and after. Gitlink advance `9d71595 → 1f732fa` is consistent.

### Recommended path
Keep the annotation if you want it as defensive typing, but land the Wave-9 closeout with the lowering fix at `statement_dispatch.rs:749` (restore the enclosing map while preserving patches recorded for captured enclosing names), a focused regression test for the nested-capture shape, and a ledger row that names PR #3079 as the regression source. Otherwise the all-411 native gate is green over a live compiler defect, which is exactly what the issue forbids.
