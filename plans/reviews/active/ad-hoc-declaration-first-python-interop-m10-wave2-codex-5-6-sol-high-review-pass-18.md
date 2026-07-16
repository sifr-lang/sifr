# M10 Wave 2 review pass 18

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete `main...HEAD` implementation and validation evidence
- Verdict: **CHANGES REQUIRED**

## Findings

1. **High — transitive generic operator bounds are missing from emitted trait
   impls.** Lowering closes direct `self.method()` dependencies and ordinary
   method impls receive those requirements, but operator emitters pass no
   transitive bounds. A generic `__eq__` delegating to a `Clone + PartialEq`
   helper emits an unbounded `PartialEq` impl that Rust rejects. The same gap
   applies to ordering, arithmetic, and unary operators.
2. **High — canonical generic identity breaks user-module aliases and collides
   across modules.** Specialized annotations and constructor HIR use the
   unqualified declaration name while project codegen imports only the local
   alias. `from models import Box as B` therefore emits `Box<i64>` and
   `Box::new` despite only `B` being in scope. Unqualified requirement keys can
   also conflate same-named generic classes from distinct modules. Existing
   evidence aliases only merged stdlib `deque`, so it does not cover this path.
3. **High — module return inference ignores statement families and incomplete
   reachability.** The prepass does not analyze `match`, `try`, or context
   statements and only recognizes terminal return, raise, and exhaustive `if`.
   Reachable returns inside an exhaustive `match` are ignored while an
   unreachable tail seeds the authoritative signature and causes valid source
   to be rejected.
4. **Medium — specialization rejects concrete classes with custom operators.**
   Requirement admission delegates to primitive type-checking helpers that do
   not recognize class dunder implementations. A custom class with `__neg__`
   is rejected as the type argument of a generic negating wrapper even though
   codegen emits a valid Rust `Neg` impl; arithmetic and ordering share the gap.
5. **Medium — capability evidence overstates closure.** The ledger claims
   canonical alias identity, exact recursive operator bounds, and fixed-point
   reachability, but permanent fixtures do not cover user-module aliases,
   same-name collisions, operator-helper dependencies, nested custom operator
   arguments, or returns inside `match`/`try`.

## Reproduced evidence

- `Box[T].__eq__` calling `self.same(...)` emits
  `impl<T> PartialEq for Box<T>` while `same` exists only under
  `T: Clone + PartialEq`.
- A user-module `Box as B` import emits only `use ...::Box as B` but refers to
  the unavailable unaliased `Box` in generated types and constructors.
- An unannotated function with all reachable returns inside an exhaustive
  `match` and an unreachable string tail is inferred as `str` and rejected.
- `Wrap[Inner]` generic negation rejects `Inner` as lacking `Neg` even when
  `Inner.__neg__` is declared and emitted.

Read-only diff hygiene, HIR maintainability, and selected existing tests passed.
The only dirty path remained the excluded `third_party/ruff` submodule.

## Remediation

- Operator protocol emission now combines the lowering-owned closed semantic
  requirement graph with the exact closed bounds of every directly called
  helper impl. Rust operator requirements retain their associated output type,
  and ordering bounds are deduplicated.
- User-project imports localize class identity to the emitted alias while
  merged stdlib imports retain canonical identity. Generic requirement keys,
  imported method signatures, and per-module codegen templates use that local
  identity, so two modules may export same-named generic classes without
  collision.
- Module signature inference now analyzes `match`, `try`, and sync/async
  `with`, including bindings, returns, and conservative terminal reachability.
- Specialization admission recognizes concrete class `__neg__`, `__eq__`, and
  `__lt__` implementations, recursively validating requirements of nested
  generic specializations. Binary dunders whose current Rust ownership shape
  does not satisfy owned generic operator traits remain conservatively
  rejected.
- Permanent evidence now includes
  `generic_method_recursive_bounds_runtime`,
  `generic_nested_custom_operator_runtime`,
  `generic_nested_custom_operator_specialization_rejected`,
  `top_level_compound_return_inference`, and
  `test_build_project_keeps_aliased_same_name_generic_classes_distinct`.

Focused native and project-build reproductions pass. Full code generation
passes `825/825`, lowering passes `750` with one ignored, and the complete
compile-fail corpus passes `527/527`. Workspace Clippy, formatting, JSON and
diff checks, HIR maintainability, and the `900`-line file-size guardrail pass
over `2672` files. The authoritative create-PR gate passes every blocking lane:
Python interop `11/11`, runtime platform `28/28` with one gated skip, and E2E
`131/131` with signature `7c39b8c1dd4fec7c` and `42/42` cache hits. Its
`476.95s` wall time produced only the non-blocking warm-wall-time advisory.
A follow-up whole-diff review remains pending.

## Required remediation

- Carry collision-safe declaration identity separately from the local emitted
  spelling and cover cross-module aliases plus same-name imports.
- Feed the closed method-requirement graph into every operator trait impl.
- Make module inference and terminal analysis cover every supported compound
  statement family used by ordinary body lowering.
- Derive specialization capabilities from concrete class operator declarations
  as well as primitive types.
- Add permanent coverage for every reproduction, correct the ledger, rerun the
  authoritative gate, and request another complete review.
