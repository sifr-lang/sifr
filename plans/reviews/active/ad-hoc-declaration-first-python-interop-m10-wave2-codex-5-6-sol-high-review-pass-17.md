# M10 Wave 2 review pass 17

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete `main...HEAD` implementation and validation evidence
- Verdict: **CHANGES REQUIRED**

## Findings

1. **High — aliased generic imports bypass specialization requirements.**
   Requirements are stored under the local alias while concrete class identity is
   renamed, so binding only equal class names misses the specialization. An
   aliased `deque[Local].count(...)` is accepted and emits an invalid `&D` type,
   while the unaliased spelling correctly rejects missing `Clone + PartialEq`.
2. **High — generic operator consumers are not specialization-checked.**
   `Box[Local] == Box[Local]` is accepted whenever the class declares `__eq__`,
   even though the emitted conditional `PartialEq` impl requires traits that
   `Local` does not provide.
3. **High — multi-parameter generic ordering impls use the wrong semantics and
   bounds.** The `PartialOrd` emitter ignores the source `__lt__` body and always
   compares the first field, while its bounds are inferred from the actual body.
4. **High — mixed conditional sources bypass keyed-`sorted()` Clone admission.**
   Lowering treats a named/temporary conditional as consumed, while codegen
   materializes the named branch with `.iter().cloned()`.
5. **High — top-level return inference remains source-order dependent.** A
   forward call to a later unannotated union-returning function can be accepted
   and emit an `i64` caller returning an `IntOrStr`; reversing declaration order
   correctly rejects it.
6. **Medium — generic negation and its evidence are overstated.** TypeVar and
   class negation are rejected before the newly derived `Neg` bounds can be used,
   while the runtime fixture neither performs generic negation nor invokes the
   generic `__neg__` operator.

## Reproduced evidence

- Aliased `from sifr.collections import deque as D` bypassed the specialization
  diagnostic and emitted `fn f(values: &D, ...)`.
- `Box[Local]` equality passed checking but emitted a conditional
  `impl<T: Clone + PartialEq> PartialEq for Box<T>` and an unsupported call.
- `Ordered[A, B].__lt__` over `second: B` emitted a `B: PartialOrd` impl that
  compares the unbounded `first: A` field.
- `sorted(values if flag else make(), key=key)` over non-Clone elements passed
  checking but emitted `(values).iter().cloned()`.
- A forward caller annotated as `int` passed checking while calling a later
  inferred `int | str` function and emitted invalid Rust; reverse order failed.
- `return -self.value` for a generic method and unary minus on a generic class
  were both rejected by the type checker.

Read-only verification confirmed codegen `824/824`, HIR maintainability, diff
hygiene, and the 900-line guardrail over 2,659 files. Lowering passed 744 tests
with one sandbox-dependent Unix worker test failure. The only dirty path remained
the excluded `third_party/ruff` submodule. No additional buffer-runtime lifecycle
blocker was found.

## Required remediation

- Preserve canonical generic class identity through aliased imports and test
  cross-module alias specialization.
- Validate exact conditional operator impl requirements at concrete consumers.
- Emit ordering behavior and bounds from one representation of the source method.
- Share branch-wise keyed-sort materialization requirements between lowering and
  codegen.
- Finalize mutually visible top-level inferred signatures before body checking.
- Implement and execute generic negation end-to-end or narrow the claim.
- Add permanent positive and negative coverage for every reproduction and rerun
  authoritative validation followed by another complete review.

## Remediation record

All six findings were implemented before the next review:

- Imported generic classes now keep canonical declaration identity in specialized
  annotations and constructor HIR while the local alias remains the lookup name;
  requirements and declared parameters remain available through both identities.
- Binary, comparison, and unary operator consumers invoke the same concrete
  specialization validator as ordinary methods.
- `PartialOrd` delegates to an emitted helper containing the actual `__lt__`
  body. Its impl adds the representation/custom-`__eq__` `PartialEq` supertrait
  requirements without leaking unrelated body bounds.
- Keyed sorting performs branch-wise Clone admission independently from whole-
  expression ownership consumption.
- Module inference is union-aware, reachability-aware for terminal statement
  tails, and iterates to a fixed point based on the declaration-group size.
- TypeVar negation can defer to generated `Neg` bounds inside generic methods,
  and generic class negation validates the concrete specialization.

Permanent coverage includes six new compile-fail fixtures, an aliased generic
runtime fixture, multi-parameter ordering that deliberately reverses the first-
field order, invoked generic negation, two declaration-order union checks, and a
forward-call chain longer than the former eight-pass inference cap. Focused
validation passes: codegen `824/824`, lowering `747` with one ignored,
compile-fail `526/526`, and native execution for the aliased import and generic
operator fixtures. The post-remediation `cargo clean` authoritative create-PR
gate passed every blocking lane, including Python interop `11/11`, runtime
platform `28/28` with one gated skip, and cold-cache E2E `131/131` with report
signature `7c39b8c1dd4fec7c`. All 42 E2E groups rebuilt successfully; the
`855.62s` wall time produced only the expected non-blocking warm-target advisory.
File-size guardrails pass over `2667` files, HIR maintainability passes, and the
next whole-diff review is tracked in the phase document.
