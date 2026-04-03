# Phase 26 Follow-Up: Multiple Bounds (`T: A & B`) Feature Gap

Status: open (documented 2026-03-07)  
Phase context: Phase 26 type-system soundness closeout

## Summary

Sifr currently supports single TypeVar bounds/constraints (for example `T: Comparable` or
`TypeVar("T", int, str)`), but does not support intersection-style multiple bounds such as
`T: Comparable & Hashable`.

This is a feature gap, not a regression or soundness bug.

## What This Means

Not currently supported:

```python
def foo[T: Comparable & Hashable](x: T):
    ...
```

Currently supported:

```python
def foo[T: Comparable](x: T):
    ...
```

```python
from typing import TypeVar
T = TypeVar("T", int, str)
```

## Why This Is Not a Phase 26 Defect

- It was not supported before Phase 26 either.
- Phase 26 objectives were to close known soundness holes, which are now addressed.
- Single-bound and constrained TypeVar behavior is strict and regression-covered.

## User Impact

- No existing code breaks because of this.
- Users needing intersection semantics (`A` and `B` simultaneously) cannot express that directly yet.

## Current Workarounds

- Use a single bound where acceptable (`T: Comparable`).
- Use explicit constraints for known finite concrete types (`TypeVar("T", int, str)`).
- Refactor APIs to avoid requiring intersection constraints at the signature level.

## Recommended Future Scope

Potential future milestone:

- Add parser/lowering support for multiple bounds syntax.
- Represent intersection bounds canonically in type metadata.
- Enforce all listed bounds during generic call validation.
- Add positive and negative e2e coverage for multi-bound TypeVars.

Complexity: medium  
Priority: low (enhancement, not blocker)
