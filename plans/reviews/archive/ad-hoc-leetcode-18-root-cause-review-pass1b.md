# Review — Ad-hoc Phase: LeetCode 18-Failure Root-Cause Closure (pass1b)

Reviewer: Claude
Source docs reviewed:
- `issues/ad-hoc-phase-leetcode-18-failure-root-cause-closure-2026-04-08.md`
- `issues/ad-hoc-phase-execution-leetcode-18-failure-root-cause-closure-2026-04-08.md`
Scope of review: fixture classification, RCA quality, language-policy alignment, workstream order & gates.

## Verdict

**NOT_READY**

The phase is close but has three blocking issues that must be resolved before implementation. All three are tractable with edits to the two docs (no rework to the underlying taxonomy). The rest of the material is sound and cleanly actionable.

## Critical Corrections

### CC1. WS1 is missing fixture `0705_design_hashset` (run-stage coverage gap)

**Symptom.** The execution doc states `run-stage failures: 7`. Enumerating the fixture table yields exactly 7 run-stage fixtures: `0049, 0144, 0145, 0286, 0705, 0973, 1137`. But `WS1_codegen_soundness_run_stage` lists only 6: `0049, 0144, 0145, 0286, 0973, 1137`. `0705` is dropped.

**Impact.** WS1's exit criterion is *"all six fixtures pass `sifr run`"* and its goal is *"zero run-stage build/assert failures from lowering/codegen defects."* Since `0705`'s build failure (`list[Any]` → `Vec<Box<dyn Any>>` with missing `Clone`/`Eq`) is a lowering defect, WS1's stated goal cannot be met while excluding it. `0705` is tagged `both`, and WS3's "adaptation portions of mixed fixtures" correctly absorbs its fixture-side work — but nothing picks up its compiler-side work.

**Required fix.** Either (a) add `0705` to the WS1 fixture list (preferred, making it 7 fixtures) and update the exit criterion to "all seven fixtures pass `sifr run`"; or (b) explicitly assert in the doc that `0705`'s compiler safety-hole is out of scope for this phase and move it under a named follow-up. Silent omission is not acceptable because the execution doc's own baseline count (7 run-stage failures) makes the gap mechanically detectable.

### CC2. C7 section text discusses the wrong fixture

**Symptom.** Category counts sum correctly (6+4+2+2+2+1+1=18). By elimination across the other six category texts:

- C1 lists `0049, 0144, 0145, 0286, 0973, 1137` (6)
- C2 (`ownership_and_mutability_boundary`, 4) = `0018, 0056, 0402, 0442`
- C3 (`nonlocal_…`, 2) = `0543, 0673`
- C4 (`recursive_node_…`, 2) = `0230, 0707`
- C5 (`optional_none_flow_…`, 1) = `0721`
- C6 (`signature_invalid_fixture_surface`, 2) = `1849, 1930`

That leaves exactly one fixture unaccounted for: `0705`. So C7 (`other_type_surface_and_api_mismatch`, 1) = `{0705}`. Yet the C7 section text discusses `0973` ("`0973` is primarily a compiler/codegen Optional-index issue; overflow warnings are advisory …"). `0973` is already claimed by C1 and is not the C7 member.

**Impact.** The taxonomy/category text and the WS1 fixture list together leave `0705` with no written home. A reader following the category text will not see `0705` reasoned about anywhere at the RCA-group level, and the reviewer loop loses its ability to audit what language-policy decision applies to `0705`'s `Vec<Box<dyn Any>>` emission.

**Required fix.** Rewrite the C7 section to discuss `0705`. The substantive content should read roughly: *"`0705_design_hashset` exposes an `Any`-type propagation defect: an empty class-field initializer (`self.hashset = []`) stays as `list[Any]` and lowers to `Vec<Box<dyn Any>>`, which then trips `Clone`/`Eq` trait requirements downstream. This is both a compile-safety hole (compiler should reject or force annotation of unbounded `Any` collection fields) and a fixture typing miss. Language-policy decision: no new language feature; tighten compiler diagnostic to require explicit element type for empty-collection field initializers, and adapt the fixture to add the annotation."* The dangling `0973` note, if kept at all, should move into the C1 discussion as an advisory about overflow warnings.

### CC3. `1137` lane classification relies on an unstated architectural claim

**Symptom.** The architecture lock in the phase doc cites `internal_docs/architecture.md` *"Python Divergences table, global / nonlocal row"*. The row covers **both** `global` and `nonlocal`. The lock then states the policy verbatim for `nonlocal` ("no hidden mutable closure capture") but does not explicitly say what the policy is for `global` mutable writes.

`1137`'s RCA describes an asymmetry: writes go to an unresolved symbol `Memo`, reads go through a synthesized `__const_Memo()`. The `__const_` prefix on the read path is the interesting signal — it suggests the compiler is *intentionally* treating module-level bindings as const for read, which is consistent with a policy that module-global mutable state is not supported. If that reading is correct, "emission inconsistency" is a misdiagnosis: the writer side is correctly refusing to emit, and the fixture is using an unsupported pattern.

**Impact.** If module-global mutable dicts are out of scope under the same divergence row, then:
- `1137` should be lane = `adaptation`, not `compiler`.
- WS1 should drop `1137` and WS3 should pick it up (rewrite memo to an explicit state object or recursive-with-arg pattern).
- `compiler` count drops 7→6, `adaptation` count rises 5→6.

If module-global mutable dicts **are** supported, the current classification is correct but needs a one-line justification in the phase doc so the reviewer does not have to guess.

**Required fix.** Read the Python Divergences table in `internal_docs/architecture.md` and add one sentence to the architecture-lock section of the phase doc, either:
- "`global` mutable binding is supported; `1137` is a compiler emission bug." — keeps current classification, or
- "`global` mutable binding is also intentionally unsupported (same row); `1137` is adaptation." — reclassify and move fixture between workstreams.

This is a blocker because the resolution changes a workstream's membership, not just its prose.

## Non-Critical Improvements

### NC1. `0049` RCA shorthand is slightly misleading

The table describes the defect as *"`groups.get(...).cloned().push(...)`, mutating a clone instead of map entry"*. `Option::cloned().push(...)` does not compile directly (you cannot `push` on an `Option<Vec<_>>`), so a reader trying to find this pattern in codegen output may waste time. The actual defect is almost certainly an `Option::map(|v| v.clone().push(...))` or a `HashMap::get` returning a shared ref that is then cloned into a local before the mutation — both of which drop the write. Recommend rewriting as: *"`dict[k].append(v)` lowers through a read-then-clone path (e.g., `map.get(&k).cloned()`) that mutates a local copy instead of the map entry. Correct lowering is `entry(k).or_insert_with(Vec::new).push(v)` or a `get_mut` borrow."* This keeps the RCA precise enough to write a targeted regression test.

### NC2. "Field-expression parity" scope is under-specified

The phase doc treats C4 as *"add compiler field-expression parity for typed objects"*. "Parity" suggests a small gap, but the table descriptions ("field access expression unsupported", "field-expression surface gap") read like the feature may be largely absent for recursive/typed object nodes rather than 90% present. WS2's exit criterion *"compiler diagnostics disappear for compiler-owned parts on all six fixtures"* is then under-budgeted if field reads are a new surface rather than a patch.

Recommend adding one line to WS2 stating the scope: e.g., *"Scope is attribute-read of named fields on `class`-typed locals and recursive self-referential types (`TreeNode`, `ListNode`); mutation via field expressions and assignment to field expressions are out of scope for this phase."* This also protects WS2 from creeping into a larger object-model refactor.

### NC3. Validation contract should explain the `--skip test_e2e_pass` flag

`cargo test -p sifr -- --skip test_e2e_pass` appears in the validation contract without explanation. Future maintainers reading this doc will not know whether the skip is because `test_e2e_pass` *is* the corpus rerun (expensive, covered by WS4) or because it is known-flaky. One sentence in the validation contract clearing this up is cheap insurance. Not blocking.

### NC4. WS4 exit criteria should explicitly include a regression guard

WS4's current exit criteria list the regenerated artifacts and "any residual failures are re-categorized with fresh RCA". The implicit regression guard (no previously-passing fixture starts failing) is only stated in the execution doc's *Phase exit validation* block ("zero known regressions in previously passing fixtures"). Adding a matching bullet under WS4 directly — "delta of (prev-passing, now-failing) is empty" — makes WS4 auditable without cross-referencing two docs. Not blocking.

### NC5. WS2 / WS3 can parallelize for mixed fixtures

The workstream order is presented serially (1→2→3→4). For the 6 `both` fixtures, the compiler fix (WS2 or WS1) and the fixture adaptation (WS3) are independent and can proceed in parallel, and validation of the fixture only requires *both* waves to be merged before rerun. Consider renaming "Workstream Order" to "Workstream Dependency Graph" and noting that only WS4 has strict dependence on the others; WS1/WS2/WS3 can be pipelined. This is a speed improvement, not a correctness issue.

### NC6. Category C2 double-lists the narrowing concern

C2 says *"Optional index flow in arithmetic-heavy loops still needs better narrowing (`0018`, `0056`)"* — but the same concern also surfaces in C5 for `0721` (*"residual optional union/index narrowing instability"*). These are the same compiler improvement ("improve narrowing precision on locally-bounded control flow"). Consider cross-referencing so the WS2 implementer doesn't treat them as three separate changes. Cosmetic.

## Revised Lane Split (conditional)

Under **CC3 scenario A** (`global` is supported → current classification stands):
- compiler: 7 | adaptation: 5 | both: 6 — **no change.**

Under **CC3 scenario B** (`global` is also unsupported per the divergence row):
- compiler: 6 | adaptation: 6 | both: 6 — `1137` moves from compiler to adaptation.

Under **CC1 (mandatory)** — no lane count change; only the WS1 fixture list grows from 6 to 7 to include `0705`.

## Revised Workstream Ordering (conditional)

No change to the 1→2→3→4 ordering is required, but:

- **Mandatory under CC1**: WS1 fixture list becomes `{0049, 0144, 0145, 0286, 0705, 0973, 1137}` (7). WS1 exit criterion updated to "all seven fixtures pass `sifr run`".
- **Conditional under CC3 scenario B**: `1137` moves from WS1 to WS3. WS1 fixture list then becomes `{0049, 0144, 0145, 0286, 0705, 0973}` (6) and WS3 gains `1137` alongside `0402, 0442, 0543, 0673, 1849`.
- **Recommended (non-blocking, per NC5)**: annotate that WS1/WS2/WS3 can be pipelined, WS4 is the only strict serial step.

## What Is Already Sound

To keep this review balanced, the following are validated and should not be reworked:

- **Lane math**: fixture table → lane counts (7/5/6) is internally consistent.
- **Stage math**: run (7) / check (11) = 18 is internally consistent with the execution doc's baseline counts.
- **Architecture lock for `nonlocal`**: correctly applied to `0543` and `0673`; the "adaptation only" decision is consistent with Sifr's explicit-data-flow principle.
- **`int(str) → Result[int, ParseError]` decision for `1849`**: correct; relaxing this would contradict parse-safety, which is a core principle.
- **`str.rfind` parity for `1930`**: correct classification as a compiler parity add, not a fixture rewrite, because rewriting to avoid `rfind` would penalize idiomatic algorithmic code with no safety benefit.
- **Mutability contract for `0018, 0056, 0402, 0442`**: correctly kept as fixture adaptation. The `mut` parameter contract is load-bearing for Sifr's ownership story and should not be softened.
- **Optional narrowing framing**: "improve where proof is local and sound, adapt where proof is non-local" is the right split and matches Sifr's existing Option/Result discipline.
- **Reviewer loop and closure gates (WS4)**: the rerun-and-regenerate pattern is consistent with prior phases; the two artifact regeneration targets are the correct closure signal.

## Summary of Required Edits Before READY

1. Resolve **CC3** by reading `internal_docs/architecture.md` Python Divergences "global / nonlocal" row and adding one line of justification to the architecture-lock section. If `global` is unsupported, move `1137` from WS1 to WS3 and update lane counts.
2. Apply **CC1**: add `0705` to the WS1 fixture list (or explicitly exclude with written justification) and update WS1's exit criterion count accordingly.
3. Apply **CC2**: rewrite the C7 section text to discuss `0705` instead of `0973`; relocate the `0973` overflow-warning aside into C1 if kept.

After these three edits, a pass2 review should be able to return READY without further structural changes. The non-critical items (NC1–NC6) can be folded in during the same edit pass but do not gate readiness on their own.
