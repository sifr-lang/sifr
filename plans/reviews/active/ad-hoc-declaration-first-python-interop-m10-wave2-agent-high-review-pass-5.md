# M10 Wave 2 Full-Diff Review — Pass 5

Reviewer: agent, high reasoning, fast service tier
Frozen scope: `main...0b9649a29` (PR #2988)

## Blockers

1. **High — borrowed affine buffers still escape through owned calls,
   constructors, and aggregates.** Regular calls and constructors can ask
   codegen to clone a borrowed buffer for an owned parameter. Return escape
   checking covers a direct name but does not recursively reject borrowed
   affine names inside aggregate returns.
2. **High — `min()` and `max()` remain unguarded affine iterator
   projections.** The one-iterable forms preserve the input and codegen selects
   `.iter().cloned()` plus Rust ordering, but lowering does not apply the affine
   iterator capability guard.
3. **High — dynamic collection capability handling still admits Rust trait
   failures.** `list[Any]` concatenation and dynamic `sorted()`/`sum()` inputs
   can reach clone or ordering/arithmetic codegen without a statically known
   reusable element capability.
4. **High — several buffer-owning collection paths still fail to record
   moves.** `tuple(iterable)`, keyword `dict(...)` values, the combined
   `dict(iterable, **keywords)` branch, and list `+=` do not consistently reject
   copying or consume moved affine inputs even though generated Rust consumes
   them.
5. **Medium — permanent evidence and phase tracking overstate closure.** The
   focused tests omit the paths above, while the phase tracker describes the
   previous findings as fully remediated.

## Cleared Areas

Distinct-view runtime admission is closed through computed backing-memory
footprints and shared/disjoint-view tests. Conditional-expression moves and
`PYZC` documentation are corrected. No new blocker was found in runtime
access/release linearization, exact-once resource identity, or the other
previously cleared declaration/code-generation paths. The frozen lowering
binary passed all `23/23` focused buffer contract tests.

## Verdict

VERDICT: CHANGES REQUIRED
