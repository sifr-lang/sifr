## Fix Callable[[T], bool] with TypeVar — Generic Predicate Functions

#### **Current Situation**

- When a generic function accepts a `Callable[[T], bool]` parameter (e.g., `dropwhile(pred: Callable[[T], bool], data: list[T])`), the Sifr compiler generates `impl Fn(&T) -> bool` in Rust because TypeVar is conservatively treated as a Move type (Borrow convention).
- However, when a concrete function like `is_small(x: int) -> bool` is passed as the predicate, its Rust signature is `fn(i64) -> bool` (Copy types are passed by value). Rust cannot unify `fn(i64) -> bool` with `impl Fn(&i64) -> bool`, causing a type mismatch compilation error.
- This forces `dropwhile`, `takewhile`, `filterfalse` (itertools) and `reduce` (functools) to use concrete `int` types instead of being properly generic, which is a significant limitation.

#### **Desired Situation**

- `Callable[[T], bool]` with TypeVar should work end-to-end for both Copy types (int, float, bool) and Move types (str).
- `dropwhile`, `takewhile`, `filterfalse` should accept `Callable[[T], bool]` and `list[T]` — fully generic.
- `reduce` should accept `Callable[[T, T], T]` and `list[T]` — fully generic.
- Users can write generic higher-order functions that accept predicates and binary functions working on any type.

#### **Suggested Solution**

1. **Codegen: Callable adapter closures** — When emitting a call where a concrete function is passed to a `Callable` parameter with TypeVar params, detect if any TypeVar param resolves to a Copy type in the concrete function. If so, wrap the function in an adapter closure that dereferences Copy-type params. E.g., `dropwhile(is_small, nums)` emits `dropwhile(|__a0: &_| is_small(*__a0), &nums)`. Move-type functions (like `is_long(s: &String)`) are passed directly since they already match.

2. **Codegen: Let/Assign clone for borrowed TypeVar params** — When a borrowed TypeVar parameter (`&T`) is assigned to a local TypeVar variable (`result: T = initial`), emit `.clone()` to convert from borrowed to owned.

3. **Stdlib updates** — Change `dropwhile`, `takewhile`, `filterfalse` from `Callable[[int], bool]` to `Callable[[T], bool]`, and `reduce` from `Callable[[int, int], int]` to `Callable[[T, T], T]`.

4. **E2E tests** — Add comprehensive test covering generic Callable with int (Copy), str (Move), and float (Copy) types for all affected functions.
