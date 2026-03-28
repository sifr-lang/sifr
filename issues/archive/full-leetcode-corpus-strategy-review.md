# Full LeetCode Corpus Strategy Review

Status: completed on 2026-03-14

Inputs reviewed:
- `verification/leetcode/full_corpus_current_results_20260314.json`
- `verification/leetcode/phase31_corpus_inventory.json`
- `issues/phase31-ad-hoc-followup-milestones.md`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`
- `reviews/full-corpus-strategy-review-pass-1.md`

## Question

For the full LeetCode corpus currently checked into the repo, decide which remaining failures need:

1. a broader ad hoc language/compiler phase,
2. canonical Sifr fixture normalization,
3. both,
4. or ordinary compiler/runtime closure work.

## Measured Current State

Fresh corpus sweep using `target/release/sifr` over all `411` checked-in fixtures:

- `PASS=53`
- `NO_ORACLE=4`
- `CHECK_ERROR=328`
- `RUN_ERROR=26`

Primary artifact:
- `verification/leetcode/full_corpus_current_results_20260314.json`

Interpretation:

- `53` fixtures are already compiling and running successfully.
- `4` fixtures run successfully but still have no embedded oracle, so they are verification gaps rather than compiler blockers.
- `354` fixtures are still blocked by compiler/codegen/corpus issues.

## Method

- Ran a full `check`/`run` sweep across all `audits/leetcode/*.sifr` fixtures.
- Bucketed failures by concrete stderr rather than milestone names.
- Read representative raw fixtures in each family directly.
- Ran targeted validation on ambiguous cases:
  - `0043`, `0394`, `1985` for parse-safety mismatches,
  - `0006`, `0026` for ownership/mutability surface,
  - `0215`, `1046` for multi-solution fixture normalization,
  - `0278`, `0374`, `2405` for harness/fixture correctness,
  - and the four `NO_ORACLE` fixtures to separate verification gaps from compiler failures.
- Spawned an external reviewer and validated its notes against the local evidence.

## Final Conclusion

For the full corpus, the right strategy is **both**, but not uniformly:

- keep the two existing broad feature phases:
  - `issues/ad-hoc-full-recursive-type-feature.md`
  - `issues/ad-hoc-own-mut-parameter-convention.md`
- add **one new broad feature phase** for nested local functions / closure inference / `nonlocal` / captured-state typing:
  - `issues/ad-hoc-full-nested-function-pipeline.md`
- treat several families as **canonical Sifr or corpus normalization work**, not as language-feature phases
- keep the remaining large buckets as **ordinary compiler/runtime closure**

This is the important difference from the earlier 50-case seed-corpus review: the full 411-fixture corpus shows that nested local functions are no longer a narrow milestone-sized issue. They are a broad language/compiler surface.

## Reviewer Validation

External review file:
- `reviews/full-corpus-strategy-review-pass-1.md`

What the external review got right:

- recursive types remain a real broad prerequisite
- `own mut` remains a real broad prerequisite for `1299`
- stdlib gaps, destructuring, container specialization, and optional flow are still real ordinary-closure buckets
- fixture cleanup is needed for raw-source and parse issues

What the external review got wrong or understated:

- it treated `0006` as an `own mut` case; that is false. `0006` needs canonical `own` or `clone`, not owned mutability.
- it concluded no new broad phase is needed. The full corpus evidence does not support that for nested local functions:
  - `114` still-failing fixtures contain nested `def` bodies
  - `95` of those remain after excluding recursive-type and duplicate-top-level-def families
  - the failures span missing nested parameter inference, captured-state typing, `Any` leakage, recursive helper return typing, and `nonlocal`-style control-flow gaps
- it treated too much of the nested-helper fallout as generic `Any` cascading, but the repeated source pattern is broader than one seed milestone

Best judgment after validation:

- keep recursive types and `own mut`
- add a new broad nested-function phase
- do not add new broad phases for stdlib parity, class field state, or codegen hardening

## Broad Prerequisite Phases

### 1. `prereq_recursive_types`

Keep:
- `issues/ad-hoc-full-recursive-type-feature.md`

Why:
- `61` failing fixtures still hit recursive-type blockers:
  - `unknown type: 'ListNode'`
  - `unknown type: 'TreeNode'`
  - `unknown type: 'Node'`
  - attribute-expression failures on `.next`, `.val`, `.left`, `.right`, `.random`, `.neighbors`

This is clearly broader than LeetCode closure. It is a language/type-system/codegen feature.

Representative cases:
- `0002`, `0019`, `0021`, `0023`, `0024`, `0025`
- `0100`, `0101`, `0102`, `0103`, `0104`, `0105`, `0106`, `0108`, `0110`
- `0133`, `0138`
- `0226`, `0235`, `0236`
- `0450`, `0701`, `1669`, `1721`, `2130`

### 2. `prereq_own_mut`

Keep:
- `issues/ad-hoc-own-mut-parameter-convention.md`

Why:
- `1299` still needs a real owned-mutable parameter surface, not a narrow fixture hack.

Important correction:
- `0006` is **not** an `own mut` case.
- The broader ownership/mutability raw-source family is `19` fixtures, but only `1299` needs the language feature.
- The other `18` should be handled by canonical Sifr rewrites using already-intended ownership or mutability forms.

### 3. `prereq_nested_function_pipeline`

Keep/add:
- `issues/ad-hoc-full-nested-function-pipeline.md`

Why this now qualifies as a broad feature phase:

- `114` still-failing fixtures contain nested local functions.
- `95` remain even after excluding recursive-type fixtures and duplicate-top-level-definition fixtures.
- The failures are not one narrow pattern. They cover:
  - missing nested parameter annotations/inference,
  - recursive local helper return typing,
  - captured-state typing,
  - `nonlocal`-shaped update patterns,
  - fallback to `Any`,
  - downstream failures like:
    - `'<` not supported between instances of 'Any' and 'int'`
    - `bad operand type for unary not: 'Any'`
    - `parameter 'i' in function 'dfs' is missing a type annotation`
    - `function expects return type 'Any', but returns nothing`

This is the same shape as recursive types: no longer a narrow compatibility patch, but a broader source-language capability that needs one coherent architecture.

Representative cases:
- `0017`, `0039`, `0050`, `0052`, `0078`, `0090`, `0207`, `0684`, `0912`
- plus many additional full-corpus cases such as `0010`, `0079`, `0091`, `0208`, `0211`, `0212`, `0269`, `0309`, `0410`, `0540`, `0673`, `0745`, `0981`, `1049`, `1397`, `2101`, `2616`

## Families That Should Stay as Canonical Sifr or Corpus Normalization Work

These are not reasons to weaken the language or create new feature phases.

### 1. Raw ownership/mutability surface normalization (`19` fixtures)

Primary rule:
- adapt raw Python-shaped fixtures to canonical Sifr ownership and mutability signatures

Subfamilies:

- borrowed-return canonicalization:
  - `0006`, `0021`, `0025`, `0061`, `0075`, `0083`, `0148`, `0226`, `0236`, `0450`, `0701`, `1669`, `1721`
- borrowed mutable-parameter canonicalization:
  - `0026`, `0073`, `0080`, `0274`, `0605`
- `own mut` prerequisite + canonical rewrite:
  - `1299`

Important note:
- many recursive-node cases need **both** the recursive-type prerequisite and an ownership-surface rewrite after that prerequisite lands

### 2. Raw parse-safety policy mismatches (`3` fixtures)

Cases:
- `0043`
- `0394`
- `1985`

Why:
- these fixtures rely on unchecked Python-style parse conversions such as `int(ch)`
- Sifr intentionally keeps parse safety with `Result[int, ParseError]`

Required handling:
- canonical Sifr rewrite first
- then re-run and reclassify any remaining compiler gaps

Validated nuance:
- `0043` is definitely **canonical rewrite + closure**
- `0394` and `1985` also still expose non-policy follow-on issues after the parse mismatch is isolated, so they should be treated the same way unless later evidence proves otherwise

### 3. Multi-solution scraped fixture normalization (`21` fixtures)

These files contain multiple top-level solution bodies for the same problem and should be reduced to one canonical Sifr-target solution each.

Examples:
- `0010`, `0021`, `0027`, `0049`, `0104`, `0112`
- `0200`, `0201`, `0215`, `0231`
- `0338`, `0513`, `0516`, `0621`, `0658`, `0678`
- `1046`, `1481`, `2215`, `2864`, `2971`

Important nuance:
- canonicalization is often necessary but not sufficient
- `0215` and `1046` were already validated as:
  - canonicalize first,
  - then reclassify remaining failures into normal compiler closure

### 4. Harness / scraped-fixture correctness repairs (`3` fixtures)

Cases:
- `0278` uses platform API `isBadVersion(...)`
- `0374` uses platform API `guess(...)`
- `2405` calls `minPartitions(...)` in `main()` instead of its own `partitionString(...)`

These are corpus-quality issues, not language-feature gaps.

### 5. Oracle completion (`4` fixtures)

Cases:
- `0036`
- `0190`
- `0191`
- `0474`

Observed behavior:
- all four execute successfully today
- they remain unscored because they only print `no test cases`

This is a verification backlog, not compiler failure.

## Families That Should Stay as Ordinary Compiler / Runtime Closure

These are broad workstreams, but they do not justify new prerequisite phases.

### 1. Optional-flow and in-bounds proof completion (`74` fixtures)

Representative errors:
- `type mismatch: expected 'int', got 'int | None'`
- `return type mismatch: expected 'int', got 'int | None'`
- `unsupported operand type(s) for -: 'int' and 'int | None'`
- `cannot unpack non-tuple type 'None | tuple[...]'`

This should continue as the generalized follow-on to `m31_a`, not a new language phase.

### 2. `Any` leakage and empty-container specialization (`89` fixtures)

Representative errors:
- `cannot index type 'dict[Any, Any]' with 'int'`
- `len() argument must be ... got 'Any'`
- `cannot iterate over type 'Any'`
- `'<` not supported between instances of 'Any' and 'int'`

This includes:
- empty dict/list specialization
- state tracking after writes
- residual `Any` leakage not caused by nested local functions

This should remain ordinary closure.

### 3. Destructuring and composite lvalues (`49` fixtures)

Representative errors:
- `tuple unpacking target must be a simple name`
- `for loop tuple target expects iterable elements of tuple type, got 'list[int]'`
- `augmented subscript assignment target must be a simple name`
- `attribute assignment target must be a simple name`

This should stay ordinary closure.

### 4. Stdlib / builtin callable parity (`22` fixtures)

Representative missing surfaces:
- `ord`, `chr`
- `Counter`
- `list`, `tuple`, `dict` as callable surfaces
- `choice`, `sqrt`, `ceil`

This is real work, but it is heterogeneous library parity, not one deep language feature. It should stay as ordinary closure plus canonical rewrite where the raw Python callable form is intentionally non-canonical in Sifr.

### 5. Class field-state / object initialization closure (`32` fixtures)

Representative errors:
- `type 'MinStack' has no field 'minStack'`
- `type 'Trie' has no field 'root'`
- `type 'MyStack' has no field 'q'`
- `type 'NumArray' has no field 'prefix'`

This should stay ordinary closure. The current evidence does not justify a separate broad feature phase.

### 6. Run-stage codegen hardening (`26` fixtures)

Representative run/build failures:
- mutability lowering emits `&T` where the generated Rust needs `&mut T`
- optional lowering still emits `Option<T>` where the target Rust slot expects `T`
- production codegen misses structured statement emission for some supported HIR shapes
- collection lowering falls back to `Vec<Box<dyn Any>>` in places that should stay concretely typed
- iteration lowering still moves values in generated Rust when it should iterate by reference

This is serious production hardening work, but not a new source-language phase.

## Where We Need Both

The full corpus makes the mixed cases more obvious:

### 1. prerequisite phase + canonical rewrite

- recursive types + ownership rewrite:
  - `0021`, `0025`, `0061`, `0083`, `0148`, `0226`, `0236`, `0450`, `0701`, `1669`, `1721`
- `own mut` + canonical rewrite:
  - `1299`

### 2. canonical rewrite + ordinary closure

- parse-safety mismatch + residual closure:
  - `0043`, `0394`, `1985`
- duplicate-top-level-solution normalization + residual closure:
  - `0215`, `1046` and several others after canonicalization

### 3. prerequisite phase + ordinary closure

- recursive types + residual class/ownership/algorithm closure:
  - tree/list/node fixtures that will likely expose follow-on issues once recursive typing lands
- nested-function phase + residual optional/container/destructuring closure:
  - many of the `114` nested-helper fixtures

## Recommended Planning Delta

1. Keep `issues/ad-hoc-full-recursive-type-feature.md` as a prerequisite phase.
2. Keep `issues/ad-hoc-own-mut-parameter-convention.md` as a prerequisite phase.
3. Add `issues/ad-hoc-full-nested-function-pipeline.md` as a new prerequisite phase.
4. Add or maintain an explicit corpus-normalization workstream for:
   - duplicate top-level solution cleanup,
   - ownership/mutability canonicalization,
   - parse-safety canonical rewrites,
   - harness/API cleanup,
   - oracle completion.
5. Keep stdlib parity, class-state closure, optional flow, container specialization, destructuring, and run-stage hardening as ordinary closure tracks rather than broad prerequisite phases.

## Bottom Line

For the full corpus, the best judgment is:

- **add ad hoc phases for real broad language features**
  - recursive types
  - nested local functions / closure pipeline
  - `own mut`
- **fix raw fixtures when they conflict with intentional Sifr contracts**
  - ownership surface
  - parse-safety surface
  - multi-solution scraped files
  - harness-only platform APIs
- **do both when needed**
  - many real cases need prerequisite language work and then canonical Sifr closure

That is the cleanest root-cause strategy for getting from `53/411` to a production-grade full-corpus result.
