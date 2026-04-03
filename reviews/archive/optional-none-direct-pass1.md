Looking at this failure data, I'll provide a rigorous analysis.

## 1. Subproblem Split

**A. Control-flow narrowing** — `if x is not None:` or `if x:` fails to exclude `None` from `x`'s type in subsequent branches.
- Symptom: `int | None` vs `float` in if branches, `Any | None` comparison failures

**B. Index expression narrowing** — index values used in subscriptions aren't narrowed before use.
- Symptom: "cannot index type 'list[int]' with 'int | None'"

**C. Iteration source narrowing** — iterating over `list[T] | None` fails to narrow.
- Symptom: "cannot iterate over type 'list[int] | None'"

**D. Container element refinement** — `list[A | None]` doesn't refine to `list[A]` after filtering.
- Symptom: `list[None | ListNode]` appearing where `list[ListNode]` expected

**E. Recursive type base-case handling** — functions returning `T` receive `None` as if it were valid.
- Symptom: `expected 'ListNode', got 'None'` or `expected 'TreeNode', got 'None | TreeNode'`

**F. Inference accumulation** — `Unknown | None` or `T | None` builds up and never simplifies.
- Symptom: `Unknown | None` return types, `bool | None` returns

**G. Fixture canonicalization** — some fixtures may declare types that don't match Sifr's model.

---

## 2. What Each Fix Belongs To

| Subproblem | Category |
|------------|----------|
| A. Control-flow narrowing after `is not None`/`is None` checks | control-flow narrowing |
| B. Index expression narrowing | control-flow narrowing |
| C. Iteration source narrowing | control-flow narrowing |
| D. Container element refinement after `is not None` guards | container element refinement |
| E. Recursive base-case typing (None as T) | recursion/graph/tree shape typing |
| F. `Unknown | None` → `Unknown` simplification, `T \| None` return unwrapping | inference/Unknown cleanup |
| G. Fixtures using Python patterns Sifr doesn't model | fixture canonicalization |

---

## 3. What Not To Do

- **Do not** add implicit `None`-to-`T` coercion — this violates Sifr's explicit Option philosophy and enables hidden runtime failures.
- **Do not** infer `T` when `T | None` is actual intent — the 62 failures aren't Python compatibility bugs; the fixtures encode Python semantics Sifr intentionally rejects.
- **Do not** weaken type checking for `!=`/`==` with `None` — the `Any | None` comparison failure is correct; fix the `Any` inference, not the comparison.
- **Do not** auto-unwrap `None` in return positions — `return None` when `-> T` is wrong code, not a narrowing opportunity.
- **Do not** paper over `Unknown | None` with implicit `Unknown` — the inference system is producing these; fix inference, don't hide it.
- **Do not** add Python-style `except None` or `if x: ...` truthiness narrowing unless Sifr formally adopts that semantics (it shouldn't).

---

## 4. Fix Order

1. **A (control-flow narrowing)** — highest leverage. Fix `if x is not None:` to actually narrow. This unblocks B, C, D downstream. This is one mechanism, not three — the index/iteration failures are all narrowing failures at their core.

2. **F (inference/Unknown cleanup)** — `Unknown | None` and return type `T | None` failures are partly downstream of A, but some are independent inference accumulation bugs. Attack after A catches the easy wins.

3. **D (container element refinement)** — after narrowing works, `list[None | ListNode]` should refine to `list[ListNode]` via element-level narrowing.

4. **E (recursive base-case typing)** — fix the `expected 'ListNode', got 'None'` class. This is a distinct problem: the recursive structure's base case isn't being typed as "definitely absent" vs "present but None".

5. **G (fixture canonicalization)** — handle remaining failures that aren't compiler bugs.

---

## 5. Judgment

**One root cause with cascading symptoms.** The dominant mechanism is **control-flow narrowing failure** (A/B/C). When `if x is not None:` doesn't narrow, you get:
- `int | None` persisting where `int` should flow
- `list[T] | None` persisting where `list[T]` should flow
- `T | None` return types instead of `T`
- `Unknown | None` from deferred inference never resolving

The secondary mechanism is **inference accumulation** (F) — types like `Unknown | None` and `bool | None` build up through operations without simplification, also remediated by proper narrowing.

The tertiary mechanism is **recursive type base cases** (E) — a distinct issue where `None` as a value isn't being classified as "absent" vs "present of type None" in recursive data structures.

Fix narrowing (A), and expect 30-40 of the 62 to move or clear. Fix inference (F), and another 10-15 clear. The recursive base-case fixes (E) are maybe 5-8 fixtures. Fixture canonicalization (G) will handle the remainder — likely 5-10 fixtures that use Python patterns Sifr doesn't model.

The bucket is **mostly one root cause** (narrowing) with two follow-on clusters (inference accumulation, recursive typing).
