# PRD: milestone_stdlib_generic_rewrite

## Goal

Rewrite the monomorphic stdlib to use generics. This is the integration test for the entire phase — every compiler feature from milestones 1-5 is exercised. After this milestone, the stdlib is type-safe, generic, and free of duplicated type-specific functions.

## Scope

### itertools.sifr
- Replace `list[int]`-specific functions with generic `T = TypeVar("T")` versions
- Delete `chain_str` (covered by generic `chain`)
- Delete `accumulate_float` (covered by generic `accumulate`)
- Change `dropwhile`/`takewhile`/`filterfalse` from threshold-based to predicate-based APIs

### functools.sifr
- Make `reduce` fully generic with two type parameters

### collections.sifr
- Make `Counter` generic: `Counter[T]` with `counts: dict[T, int]` (requires T: Hashable)
- Add `from_list` class method

### heapq.sifr
- Make all functions generic with `T = TypeVar("T")`

### random.sifr
- Make `shuffle` and `sample` generic

### test.sifr
- Make `assert_eq` generic

## Definition of Done

- All listed stdlib modules rewritten with generic type parameters
- `chain_str` and `accumulate_float` deleted
- `Counter[T]` works for any hashable type
- `heapq` functions work for any comparable type
- All existing E2E tests still pass
- New E2E pass tests demonstrating generic usage
- Demo: `demos/milestone_stdlib_generic_rewrite_demo.sifr`
