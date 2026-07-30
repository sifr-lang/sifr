# Independent exact-published-head review — Item 2 / PR #3082

**Reviewed head:** `581b363aa888747e83a6c498d5fb70d65f8d00db`
**Base:** `f1c34cf9aaabadda546e670fca190decc580c935`
**Working tree:** clean (`git status --porcelain` → empty), no files modified, no PR state changed.

## 1. Verification of the "docs-only tail commit" claim — CONFIRMED

```
$ git diff --stat e174a9ec8 581b363aa
 plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md   |  19 +-
 plans/reviews/active/...-item2-claude-opus-pr-review-pass-8.md                | 194 +++++
 2 files changed, 210 insertions(+), 3 deletions(-)
```

`581b363aa` touches only planning/review markdown. The implementation head `e174a9ec8` and the published head are code-identical, so the pass-8 SATISFIED evidence carries over to `581b363aa` for compiler behavior. **However, that equivalence is exactly why the finding below survives into the published head.**

## 2. What I verified as sound

Built both compilers and probed differentially (`target/debug/sifr` at head; `/private/tmp/sifr_base_wt` built at base `f1c34cf9a` after copying the pinned `third_party/ruff` submodule tree).

- **Canonical place modeling / emission.** Nested chains, inherited and *grand*parent-declared fields, generics, `mut`/`own`/local roots all emit against real storage and mutations persist: `class A/B(A)/C(B)` with `self.a_items.append(1)` through `self.c_items.append(3)` runs and prints `[1] [2] [3]`; a compound probe (`Root.mid.leaf.add`, `Leaf.absorb(mut other)`, `Gen[T].put`) runs and prints `[3, 2] / [1] / ["x"]` — sibling-field disjointness is honored, no raw-rustc leakage, no clone-swallowed mutation.
- **Constructor materialization / storage-root gap (`SIFR-OWN-0014`).** `class_semantics.rs:101-166` plus `class_body_lowering.rs:609-627`. Repeated-field-before-init correctly rejects (`explicit_initializers` guard at `class_semantics.rs:116,139,145`), matching the new fail fixture `crates/sifr/tests/e2e/fail/constructor_repeated_field_before_initialization.sifr`; repeated assignment after complete storage still compiles. Spans stayed source-accurate under docstrings, `pass`, annotated assigns, tuple unpack, `for`, guarded index, `assert`, and a valid `with` block.
- **Structured diagnostics.** `SIFR-OWN-0014`, `SIFR-PROTO-0005`, `SIFR-PROTO-0006` are registered with args and representative fixtures (`registry/registry_entries/calls_flow_and_protocols.rs:214-224, 412-434`), backed by three new `verification/areas/diagnostics` compact baselines. `SIFR-OWN-0002` docs (`docs/errors/SIFR-OWN-0002.mdx`) document the new same-call rule *and* the snapshot remediation.
- **Module-constant rejection, walrus, slice, specialized indexed storage, owned temporaries** behave as the plan specifies; `place_emitter.rs:60-96` never falls back to value semantics for a proven mutable place (`emit_checked_place` returning `None` aborts the path rather than cloning).
- **File-size guardrail:** largest touched hand-maintained files are `method_receiver_places.rs` 876, `ir_optimize/mutability_and_clone_rewrites.rs` 874, `class_method_emitter.rs` 814 — all under 900.

## 3. Actionable finding

### F1 — HIGH / blocking: the pinned LeetCode corpus at this PR's submodule pointer still contains a program the PR rejects; the nightly `leetcode-full` lane will fail

The PR bumps the corpus submodule:

```
$ git diff f1c34cf9a 581b363aa --raw | grep leetcode
:160000 160000 a20d9d5020 7772857c6f M  verification/areas/algorithmic_compatibility/corpora/leetcode
```

That bump exists *because* of this PR — corpus commit `7772857c6f` is titled **"Snapshot LRU head before mutable receiver calls (#40)"** and changes exactly one file (`src/0146_lru_cache.sifr`, `self.insertAfter(node, self.head)` → snapshot into a local). The same migration shape was applied in-repo to `stdlib/sifr/heapq.sifr:193,239` (`_sift_down(heap, 0, len(heap))` → `heap_len` snapshot).

**A second corpus fixture with the identical `f(mut x, …, len(x))` shape was missed.** `src/0189_rotate_array.sifr` is byte-identical at `a20d9d5020` and at the new pointer `7772857c6f` (`git diff a20d9d5 7772857 -- src/0189_rotate_array.sifr` → empty), and:

```
$ /private/tmp/sifr-class-field-receiver-item1/target/debug/sifr check src/0189_rotate_array.sifr
error[SIFR-OWN-0002]: borrow conflict for nums in the same call
  --> src/0189_rotate_array.sifr:22:29
   22 |     _reverse_range(nums, 0, len(nums) - 1)
error[SIFR-OWN-0002]: borrow conflict for nums in the same call
  --> src/0189_rotate_array.sifr:24:31
   24 |     _reverse_range(nums, rot, len(nums) - 1)

$ /private/tmp/sifr_base_wt/target/debug/sifr check src/0189_rotate_array.sifr
no errors found
```

I swept all 411 corpus fixtures with the head compiler for `SIFR-OWN-*`/`SIFR-PROTO-*`; the only new rejections versus base are `0146_lru_cache.sifr` (migrated) and `0189_rotate_array.sifr` (**not** migrated). `0143_reorder_list.sifr` (OWN-0001) and `0778_swim_in_rising_water.sifr` (OWN-0004) reproduce identically on base, so they are pre-existing single-file-check artifacts, not regressions.

**Failure scenario / why no lane caught it.** `verification/areas/algorithmic_compatibility/runner.py:384-389` (`run_leetcode_full`) runs `sifr check` on every `src/*.sifr` and marks any non-zero exit as `fail`; `run_leetcode_check` (`runner.py:521-546`) does the same and returns exit 1 on any failure. Suite selection:

| profile | algorithmic_compatibility suites |
|---|---|
| create-pr | `profile-manifest` only |
| merge | `representative-subset` (manifest `representative_subset` does **not** list `0189`) |
| release | `representative-subset`, `taxonomy-smoke` |
| nightly | **`leetcode-full`**, `taxonomy-smoke` |

So the create-pr and merge runs cited in the validation summary structurally cannot observe this fixture, while the blocking nightly `leetcode-full` case (`expected_fixture_count: 411`, every fixture must exit 0) will fail on `0189_rotate_array.sifr`. This is both a real regression against the pinned corpus and an instance of "tests that cannot catch the claimed regression."

**Remediation:** land the analogous snapshot in the corpus (`rot_len = len(nums)` hoisted before the three `_reverse_range` calls) and re-pin the submodule, or re-scope the corpus pointer bump to cover every affected fixture; then run the `leetcode-full` suite once on the new pointer before merge.

## 4. Non-blocking observations (no action required for approval)

- **Conservative same-call overlap rejects some safe code that base compiled.** `self.items.append(self.compute())` and `self.items.append(self.pair[0])` now emit `SIFR-OWN-0002` (head) where base checked, built and ran them (`ov6` printed `[14]`, `ov5` printed `[3]` on base). Cause: `collect_footprint` maps a `self`-rooted method call to `Place(self)` (`method_receiver_places.rs:714-717`) and any `Index`/`Slice` to `Footprint::Dynamic(root)` discarding projections (`:658-677`), both of which prefix-overlap every `self.*` place (`places_overlap`, `:336-343`). This is *explicitly the approved design* — plan §4 lines 356 and 368 ("`self` overlaps every `self.*` place"; "an unsupported/dynamic projection under the same root is conservatively treated as overlapping") — and `docs/errors/SIFR-OWN-0002.mdx` documents the snapshot remediation with the `values.append(len(values))` example. Recording it as an intentional, documented narrowing, not a defect. F1 is the concrete unmigrated consequence of it.
- **OWN-0014 span indexing.** `class_body_lowering.rs:619-621` indexes the *source* AST statement list with a *HIR* statement index (`gap.statement_index`). I could not make this drift on any well-formed program (docstring, `pass`, annotated assign, tuple unpack, `for`, guarded index, `assert`, valid `with` all resolved correctly); the one drift I reproduced (`with open("f.txt")` without `encoding`, pointing at line 8 instead of line 10) was on a body that already fails with `SIFR-IO-0801`. Cosmetic, secondary-diagnostic-only.

## 5. Checks that found nothing

Ownership unsoundness (no lost mutation, no aliasing acceptance; `b = a; b.bump(); print(a…)` still `SIFR-OWN-0001` identically to base), raw-rustc leakage (all accepted probes built and ran clean release binaries), panics in user-triggerable paths (no new `unwrap`/`expect` on data-dependent paths in `method_receiver_places.rs`, `place_emitter.rs`; `emit_checked_place` fails closed), diagnostic instability (codes, args, messages, spans stable across repeated runs), file-responsibility/guardrail limits.

---

**Severity-ranked actionable findings:** 1 (F1 — HIGH, blocking).

VERDICT: NOT SATISFIED
