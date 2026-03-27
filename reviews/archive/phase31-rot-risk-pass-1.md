# Phase 31 Carry-Forward: Language Rot Risk Review — Pass 1

Date: 2026-03-26
Reviewer scope: narrow focus on language rot risk
Input files reviewed:

- `issues/phase31-strategy-synthesis-review.md`
- `issues/phase31-ad-hoc-followup-milestones.md`
- `verification/leetcode/phase31_current_full_results_20260321.json`
- `issues/ad-hoc-full-recursive-type-feature.md`
- `issues/ad-hoc-own-mut-parameter-convention.md`
- `issues/ad-hoc-full-nested-function-pipeline.md`

---

## Question 1: Which remaining items are safe as canonical Sifr fixture adaptation versus first-class compiler/language work?

### Safe as canonical fixture adaptation (no rot risk)

These items require only source-level rewrites to use already-landed Sifr language features. They do not require any new compiler capability and carry zero rot risk:

| ID | Required adaptation | Why safe |
| --- | --- | --- |
| `0007` | Add `mut` to parameter | Uses already-landed `mut` parameter support |
| `0009` | Add `mut` to parameter | Same |
| `0151` | Add `mut` to parameter | Same |
| `1299` | Rewrite to `own mut` | Uses already-landed `own mut` feature |
| `0043` | Canonical Sifr rewrite for parse-safety divergence | Preserving `int(str) -> Result` is an intentional language guarantee |
| `0215` | Reduce to single canonical solution | Multi-solution file normalization, not a language gap |
| `1046` | Reduce to single canonical solution | Same |

These are purely editorial work on fixtures. The milestones `m31_j`, `m31_k`, and `m31_i` that own them are correctly scoped.

### Requires first-class compiler/language work (rot risk if treated as fixture adaptation)

| ID(s) | Milestone | Nature of work |
| --- | --- | --- |
| `0001`, `0242`, `0424`, `0523`, `0560` | `m31_g` container-literal specialization | Real type-inference feature: empty-literal first-write specialization |
| `0053`, `0238`, `0322`, parts of `0015`, `0746` | `m31_a` optional-flow completion | Real type-narrowing feature: proving non-None after guards |
| `0226`, `0295`, `0703`, `0997`, `1209` | `m31_b` destructuring | Real HIR/codegen feature: composite lvalue support |
| `0052` | `m31_d` nested-function completion | Explicit language-boundary decision: recursive `nonlocal` mutation |

### Mixed cases requiring both adaptation AND compiler work

| ID(s) | Fixture adaptation needed | Compiler work also needed |
| --- | --- | --- |
| `0015`, `0090`, `0127`, `0226`, `0746`, `0912` | Add `mut`/`own` annotations | Also blocked on optional-flow, list typing, or destructuring |
| `0043` | Parse-safety canonical rewrite | Post-canonicalization residual likely falls into `m31_a` optional-flow |

These are correctly classified in the synthesis review. No wording change needed.

---

## Question 2: Which proposed milestones risk rotting the language if implemented as compatibility hacks?

### HIGH ROT RISK: `m31_g_container_literal_specialization_and_state_tracking`

**Current wording risk**: The milestone description says "specialize empty container literals from later typed writes and reads" and "use first-write specialization for empty literals." This is a real type-inference feature that must be implemented as a general rule in the type system, not as a pattern-match on LeetCode dict-creation idioms.

**Rot scenario**: A corpus-driven implementation could add special-case recognition of `d = {}; d[key] = value` as a pattern rather than implementing general forward-propagation of container element types. This would create a brittle type-inference path that breaks on any variation (e.g., `d = {}; helper(d)` where `helper` writes to `d`).

**Verdict**: The milestone scope text is acceptable, but the "implementation notes" section needs a stronger architectural guardrail (see corrections below).

### MODERATE ROT RISK: `m31_a_optional_flow_completion`

**Current wording risk**: The remaining scope says "fixed-index reads after length guards," "non-empty queue/heap/list pop results under truthiness guards," and "subtractive/value-dependent recurrence indexing." Each of these is a narrowing rule that should derive from a general flow-sensitive proof system, not from individual pattern recognition.

**Rot scenario**: Each narrowing case gets its own ad hoc recognizer in the type checker. The type checker accumulates a growing list of "if you see this pattern, narrow that type" branches rather than a compositional proof-propagation system. Later language features (e.g., user-defined guards, match expressions) cannot reuse the narrowing logic.

**Verdict**: The current implementation note — "prefer a general forward-propagation rule for definite in-bounds access rather than adding more narrow special cases" — is the right intent but is advisory rather than enforceable. It should be promoted to a hard constraint (see corrections below).

### MODERATE ROT RISK: `m31_d_nested_function_pipeline_completion`

**Current wording risk**: The milestone says "keep this milestone corpus-driven rather than expanding into a broader nested-function feature redesign." This is correct in spirit — the broad phase already landed. But the wording "corpus-driven" could be misread as "implement whatever the corpus needs" rather than "close residual bugs in the already-landed architecture."

**Rot scenario**: A new nested-function subshape appears in the seed corpus (e.g., a backtracking helper that captures a dict and mutates it through a nested recursive call). The "corpus-driven" framing incentivizes adding a special-case lowering path for that exact shape rather than sending it back to the nested-function phase as a gap report.

**Verdict**: Wording should clarify the boundary between "residual closure bug in the landed architecture" and "new unsupported shape requiring a feature extension" (see corrections below).

### LOW ROT RISK: `m31_b_destructuring_and_composite_lvalues`

The scope is clearly a compiler feature (destructuring support) and the affected cases are concrete enough that there is little temptation to hack around them. The risk is low as long as destructuring is implemented as a general HIR/codegen capability rather than a case-by-case pattern match on the five LeetCode fixtures.

**Verdict**: No wording change needed. The current scope is appropriately general.

### NO ROT RISK: `m31_e`, `m31_l`, `m31_h`, `m31_j`, `m31_k`, `m31_i`

These milestones are either pure fixture adaptation, corpus closure on already-landed features, or narrow single-case work. None of them risk introducing compatibility hacks into the compiler.

---

## Question 3: What guardrails or wording changes should be added?

### Guardrail 1: Explicit "no pattern-match hacks" rule for type-inference milestones

Add to the **Planning Policy** section of `phase31-ad-hoc-followup-milestones.md`:

> **Type-inference and narrowing milestones must implement general rules, not corpus-driven pattern matches.** If a milestone requires a new type-inference capability (container specialization, optional narrowing, etc.), the implementation must be a general compositional rule in the type system that happens to close the corpus cases, not a recognizer for the specific code shapes found in the seed fixtures. If a general rule cannot be designed within the milestone scope, the work must be escalated to a standalone ad hoc feature phase.

### Guardrail 2: Explicit "residual closure vs. feature extension" boundary for post-phase milestones

Add to the **Planning Policy** section:

> **Post-phase closure milestones may only fix bugs in already-landed architecture, not extend the feature surface.** When a closure milestone (`m31_d`, `m31_e`, `m31_l`) encounters a seed-corpus case that fails because the landed feature does not support the required shape, that case must be reclassified as a feature gap and sent back to the owning ad hoc phase with a concrete gap report — not patched locally inside the closure milestone.

### Guardrail 3: Regression test requirement for general vs. special-case validation

Add to the **Planning Policy** section:

> **Every type-inference or narrowing implementation must include at least one regression test that exercises the rule on a shape NOT present in the seed corpus.** This validates that the implementation is general rather than overfit to the specific LeetCode fixtures.

---

## Question 4: Concrete corrections to current phase wording

### Correction 1: `m31_g` implementation notes — strengthen from advisory to mandatory

**File**: `issues/phase31-ad-hoc-followup-milestones.md`
**Section**: `m31_g_container_literal_specialization_and_state_tracking` → Implementation notes

**Current wording**:
> - use first-write specialization for empty literals
> - propagate the specialized key/value shape through subsequent reads, `.get(...)`, membership checks, and equality
> - reject conflicting later writes with deterministic "empty literal type conflict" diagnostics

**Recommended replacement**:
> - implement first-write specialization as a general forward-propagation rule in the type system, not as a pattern recognizer for specific dict-usage idioms
> - the rule must propagate element types through any subsequent operation that consumes or queries the container, not only the specific operations observed in the current seed corpus
> - reject conflicting later writes with deterministic "empty literal type conflict" diagnostics
> - the implementation must not add LeetCode-specific or dict-specific branches to the type checker; it must compose with existing container type machinery
> - at least one regression test must exercise container specialization on a shape not present in the Phase 31 seed corpus

### Correction 2: `m31_a` implementation notes — promote advisory to hard constraint

**File**: `issues/phase31-ad-hoc-followup-milestones.md`
**Section**: `m31_a_optional_flow_completion` → Implementation notes

**Current wording**:
> - prefer a general forward-propagation rule for definite in-bounds access rather than adding more narrow special cases

**Recommended replacement**:
> - implement a general forward-propagation rule for definite in-bounds access; do not add narrow special-case recognizers for individual access patterns
> - each narrowing rule must derive from a compositional proof (e.g., "variable `x` was proven non-None by a prior guard in the current flow") rather than a syntactic pattern match (e.g., "if the source looks like `if x is not None: ... x.foo`")
> - narrowing logic added in this milestone must be reusable by future narrowing consumers (match expressions, user-defined type guards) without modification
> - at least one regression test must exercise optional narrowing on a guard shape not present in the Phase 31 seed corpus

### Correction 3: `m31_d` scope — clarify "corpus-driven" boundary

**File**: `issues/phase31-ad-hoc-followup-milestones.md`
**Section**: `m31_d_nested_function_pipeline_completion` → Implementation notes

**Current wording**:
> - keep this milestone corpus-driven rather than expanding into a broader nested-function feature redesign

**Recommended replacement**:
> - this milestone may only fix residual bugs in the already-landed nested-function architecture; it must not add new lowering paths, new inference rules, or new capture-mutation support that was not part of the landed phase contract
> - if a seed-corpus case fails because the landed nested-function feature does not support the required shape, that case must be reclassified as a feature gap and sent back to `ad-hoc-full-nested-function-pipeline` with a concrete gap report rather than patched locally
> - `0052` is already correctly flagged as a language-boundary decision; the same principle applies to any other case that requires extending the nested-function feature surface

### Correction 4: Definition-of-done for `m31_d` — too permissive on "generic frontend"

**File**: `issues/phase31-ad-hoc-followup-milestones.md`
**Section**: `m31_d_nested_function_pipeline_completion` → Definition of done

**Current wording**:
> - the generic frontend bucket reaches zero for the Phase 31 corpus

**Problem**: "Generic frontend bucket reaches zero" is a metric target that could be achieved by any means — including patching individual cases with special-case lowering. A metric target without a quality constraint incentivizes hacks.

**Recommended replacement**:
> - no Phase 31 seed-corpus case fails with a generic nested-function frontend error (undefined helper, missing annotation, `Any`-fallback from nested body)
> - every fix contributing to this result is a bug fix in the landed nested-function architecture, not a new special-case lowering path

---

## Summary of Rot Risk Assessment

| Milestone | Rot risk | Primary concern | Action needed |
| --- | --- | --- | --- |
| `m31_g` container specialization | **HIGH** | Could become pattern-match hack instead of general type rule | Strengthen implementation notes |
| `m31_a` optional-flow completion | **MODERATE** | Could accumulate ad hoc narrowing recognizers | Promote advisory to hard constraint |
| `m31_d` nested-function completion | **MODERATE** | "Corpus-driven" framing could incentivize feature extensions disguised as closure | Clarify residual-bug vs. feature-extension boundary |
| `m31_b` destructuring | LOW | Clear compiler feature scope | No change needed |
| `m31_e`, `m31_l`, `m31_h` | NONE | Pure closure on landed features | No change needed |
| `m31_j`, `m31_k`, `m31_i` | NONE | Pure fixture adaptation | No change needed |

---

## Bottom Line

The Phase 31 carry-forward strategy is well-structured and the classification of items into fixture adaptation vs. compiler work is accurate. The primary rot risk is concentrated in three milestones (`m31_g`, `m31_a`, `m31_d`) where the LeetCode-corpus-driven framing could incentivize narrow pattern-match implementations rather than general language rules. The four concrete corrections above — plus three new planning-policy guardrails — would close that risk without changing the milestone scope or execution order.
