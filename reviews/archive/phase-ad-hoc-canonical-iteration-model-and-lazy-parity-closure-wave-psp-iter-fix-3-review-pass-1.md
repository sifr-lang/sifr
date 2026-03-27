# Review: wave_psp_iter_fix_3 (Concrete Iterator Codegen Pipelines)

**Phase:** ad-hoc-canonical-iteration-model-and-lazy-parity-closure
**Wave:** wave_psp_iter_fix_3
**Date:** 2026-03-20
**Review:** Pass 1 (self-review)

## Scope

From `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure-execution.md`:

> **wave_psp_iter_fix_3: Concrete Iterator Codegen Pipelines**
>
> Scope:
> - emit concrete Rust iterator chains
> - centralize collection-to-iterator lowering
> - remove clone-based fake re-iteration of true iterators
>
> Definition of done:
> - `iter(...)` emits concrete `.into_iter()` path for both collections and iterators
> - iterator consumers (`any`, `all`, `sum`, `min`, `max`, `sorted`) use centralized iterable-to-iterator conversion
> - `filter(pred, iter(xs))` emits concrete Rust `.filter()` closure instead of unresolved builtin fallback

## Implementation Summary

### 1. Registry Lowering for `iter(...)`

**Status:** ✅ Complete

**Changes in `crates/sifr_codegen/src/intrinsic_method_emitters.rs`:**

```rust
"iter" if args.len() == 1 => {
    if matches!(
        crate::resolve_alias_type_for_plain_call(args[0].ty()),
        Type::Iterator(_)
    ) {
        self.try_lower_registry_expr_strict(&args[0])
    } else {
        Some(registry_box_iterator_expr(registry_iterable_to_owned_iter_expr(
            self, &args[0],
        )?))
    }
}
```

Key behavior:
- For `Iterator[T]` inputs: returns the iterator as-is (no `.into_iter()`)
- For collection inputs: wraps in `registry_iterable_to_owned_iter_expr()` which emits `.clone().into_iter()`

### 2. Centralized `registry_iterable_to_owned_iter_expr`

**Status:** ✅ Complete

This function (`intrinsic_method_emitters.rs:173-260`) handles conversion from any iterable type to an owned iterator expression:

| Input Type | Emitted Rust |
|------------|--------------|
| `List[T]` | `.clone().into_iter()` |
| `Set[T]` | `.clone().into_iter()` |
| `Iterable[T]` | `.clone().into_iter()` |
| `Bytes` | `.iter().map(...).into_iter()` |
| `Iterator[T]` | `.into_iter()` |
| `Range` | `.into_iter()` |
| `Str` | `.chars().map(...)` |
| `Dict[K,V]` | `.keys().cloned()` |
| Homogeneous tuple | Custom tuple iteration expression |

This replaces ad-hoc `.iter().cloned()` patterns throughout the codebase.

### 3. Iterator Consumer Rewiring

**Status:** ✅ Complete

Updated builtins to use `registry_iterable_to_owned_iter_expr()` instead of manual `.iter().cloned()`:

| Builtin | Before | After |
|---------|--------|-------|
| `any(x)` | `.iter().cloned().any(\|x\| *x)` | `registry_iterable_to_owned_iter_expr(x)?.any(\|x\| x)` |
| `all(x)` | `.iter().cloned().all(\|x\| *x)` | `registry_iterable_to_owned_iter_expr(x)?.all(\|x\| x)` |
| `sum(x)` | `.iter().cloned().sum()` | `registry_iterable_to_owned_iter_expr(x)?.sum()` |
| `min(x)` / `max(x)` | `.iter().cloned().min()/max()` | `registry_iterable_to_owned_iter_expr(x)?.min()/max()` |
| `sorted(x)` | Manual type matching | Uses `.iterable_element_type()` helper |

### 4. `filter(...)` Codegen

**Status:** ✅ Complete

**New codegen in `intrinsic_method_emitters.rs:2030-2082`:**

```rust
"filter" if args.len() == 2 => {
    let item_ty = args[1].ty().iterable_element_type()?;
    let filtered_iter = RustExpr::MethodCall {
        receiver: Box::new(registry_iterable_to_owned_iter_expr(self, &args[1])?),
        method: "filter".to_string(),
        args: vec![RustExpr::ClosureBlock {
            params: vec![...],
            body: vec![
                crate::RustStmt::Let { ... clone ... },
                crate::RustStmt::Return(Some(registry_call_callable_with_owned_args(...)))
            ],
            is_move: false,
        }],
    };
    // Returns Box<dyn Iterator> for Iterator inputs
    // Returns Vec::from_iter(filtered_iter) for collection inputs
}
```

Key behavior:
- Uses Rust's native `.filter()` method
- Invokes the callable inside the closure via `registry_call_callable_with_owned_args`
- Handles both iterator inputs (returns boxed iterator) and collection inputs (materializes to Vec)

### 5. `sorted(...)` Generalized Element Type

**Status:** ✅ Complete

Changed from manual type matching to using `.iterable_element_type()`:

```rust
// Before (manual matching)
let elem_ty = match resolve_alias_type_for_plain_call(args[0].ty()) {
    Type::List(inner) => inner.as_ref().clone(),
    Type::Bytes => Type::Int,
    Type::Set(inner) => inner.as_ref().clone(),
    Type::Range => Type::Int,
    Type::Str => Type::Str,
    Type::Dict(key, _) => key.as_ref().clone(),
    _ => return None,
};

// After (generalized)
let elem_ty = resolve_alias_type_for_plain_call(args[0].ty()).iterable_element_type()?;
```

This allows `sorted(iter(xs))` to work with iterator-typed inputs.

## Verification Evidence

### Positive Fixture
```
crates/sifr/tests/e2e/pass/phase_psp_iter_fix_3_concrete_iterator_codegen.sifr
```
```sifr
def main():
    xs: list[int] = [4, 1, 3, 2]
    flags: list[bool] = [False, True, False]
    has_true: bool = any(iter(flags))
    evens: Iterator[int] = filter(is_even, iter(xs))
    ordered: list[int] = sorted(iter(xs))
    assert str(has_true) == "true"
    assert str(list(evens)) == "[4, 2]"
    assert str(ordered) == "[1, 2, 3, 4]"
```

### Demo Output
```
$ cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave3_codegen_demo.sifr
true
[5, 3, 4]
[1, 3, 4, 5]
```

### Generated Rust (Emitted)
```rust
fn main() {
    let xs: Vec<i64> = vec![4, 1, 3, 2];
    let flags: Vec<bool> = vec![false, true, false];
    let has_true: bool = (Box::new((flags).clone().into_iter())).into_iter().any(|x| x);
    let mut evens: Box<dyn Iterator<Item = i64>> = Box::new(
        (Box::new((xs).clone().into_iter())).into_iter()
            .filter(|__filter_item| {
                let __filter_value = __filter_item.clone();
                return is_even(__filter_value);
            })
    );
    let ordered: Vec<i64> = {
        let mut __sifr_sorted_v = (Box::new((xs).clone().into_iter())).into_iter().collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    };
    // ...
}
```

Key observations:
1. `iter(flags)` → `.clone().into_iter()` (not `.iter().cloned()`)
2. `filter(is_even, iter(xs))` → `.filter(|__filter_item| { ... is_even(...) })` (concrete Rust closure)
3. `sorted(iter(xs))` → uses iterator chain (`.into_iter().collect()`)

### Capability Guard (Negative Test)
```
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversed_iterator_not_reversible.sifr
type error: reversed() argument must be reversible, got 'Iterator[int]'
```
✅ Capability guard still enforced.

### Unit Tests
```
cargo test -p sifr -- --skip test_e2e_pass
test result: ok. 25 passed; 0 failed; 0 ignored
```

## Completeness Assessment

| Criterion | Status |
|-----------|--------|
| `iter(...)` emits concrete `.into_iter()` for collections | ✅ |
| `iter(...)` passes through iterator inputs unchanged | ✅ |
| Iterator consumers use centralized conversion | ✅ |
| `filter(pred, iter(xs))` emits Rust `.filter()` closure | ✅ |
| `sorted(iter(xs))` works with iterator inputs | ✅ |
| Capability guard (`reversed(iter(xs))`) still enforced | ✅ |
| No unresolved-symbol fallback in emitted code | ✅ |
| Legacy `.iter().cloned()` patterns removed | ✅ |

## Code Quality Observations

1. **No clippy warnings** introduced by this wave
2. **HIR maintainability guardrails** pass (no monolithic file growth)
3. **Centralized conversion function** (`registry_iterable_to_owned_iter_expr`) provides single source of truth for iterable-to-iterator conversion
4. **Iterator input vs collection input** handling is explicit in each builtin

## Additional Findings (External Review)

### Regression Bug Confirmed

**Status:** ❌ **REGRESSION CONFIRMED** - Needs fix before production readiness

**Issue:** `filter(pred, iterator_variable)` fails to compile while `filter(pred, iter(collection))` works.

**Failing test case:**
```sifr
def main():
    nums: list[int] = [1, 2, 3, 4]
    it: Iterator[int] = iter(nums)
    result: Iterator[int] = filter(is_even, it)  # FAILS
```

**Generated broken code:**
```rust
let mut result: Box<dyn Iterator<Item = i64>> = Vec::from_iter(it.clone().into_iter().filter(is_even));
//                                                             ^^^^^^
// Error: dyn Iterator is not Clone
```

**Working case:**
```sifr
result: Iterator[int] = filter(is_even, iter(nums))  # WORKS
```
Generated correct code:
```rust
Box::new((nums).clone().into_iter().filter(...))
```

**Root cause:** The filter codegen at `intrinsic_method_emitters.rs:2070-2080` checks `matches!(resolve_alias_type_for_plain_call(args[1].ty()), Type::Iterator(_))` but this check appears to return `false` for iterator variable references, causing it to take the wrong branch (Vec materialization + invalid `.clone()`).

**Impact:** Users cannot pass existing iterator variables to `filter()`. This breaks the expected pattern of creating an iterator once and passing it to multiple operations.

**Verification performed:**
```bash
# Working - filter with iter() call
$ cargo run -q -p sifr -- run /tmp/test_filter_iter_call.sifr
[2, 4]  # PASS

# Failing - filter with iterator variable
$ cargo run -q -p sifr -- run /tmp/test_filter_iterator_var.sifr
error[E0599]: the method `clone` exists for struct `Box<dyn Iterator<Item = i64>>`, but its trait bounds were not satisfied
```

**Other iterator operations with iterator variables:**

| Operation | Iterator Variable | Status |
|-----------|------------------|--------|
| `any(it)` (bool) | `Iterator[bool]` | ✅ Works |
| `sorted(it)` | `Iterator[int]` | ✅ Works |
| `filter(pred, it)` | `Iterator[int]` | ❌ **REGRESSION** - tries to clone |

### Pre-existing Failures (Unrelated to This Wave)

The e2e pass suite shows failures in:
- `logging_basic_config` / `logging_file_handler` - missing `_closed` field (pre-existing)
- `cpython_uuid_subset` / `stdlib_uuid_consolidated` - missing `uuid` crate (pre-existing)
- `phase_psp_d1_filesystem_paths_archives` - pre-existing
- `phase_psp_struct_3_uuid_datetime_expansion` - pre-existing

These are unrelated to wave_psp_iter_fix_3.

## Risk Assessment

| Risk | Level | Mitigation |
|------|-------|------------|
| Backward compatibility with existing iterator semantics | Low | Iterator inputs pass through unchanged (except filter regression) |
| Performance regression from `.clone()` on collections | Low | Collections must be cloned to produce owned iterators anyway |
| Missing type coverage for new iterables | Medium | `.iterable_element_type()` should be extended as new types are added |
| **filter() regression with iterator variables** | **High** | **Requires fix before production** |

## Conclusion

The wave_psp_iter_fix_3 implementation addresses the core definition-of-done criteria:

1. ✅ `iter(...)` emits concrete `.into_iter()` path for both collections and iterators
2. ✅ Iterator consumers use centralized `registry_iterable_to_owned_iter_expr()`
3. ✅ `filter(pred, iter(collection))` emits concrete Rust `.filter()` closure
4. ✅ `sorted(iter(xs))` works with iterator-typed inputs
5. ✅ Capability guard remains enforced
6. ✅ No unresolved-symbol fallback in generated Rust

**However**, there is a **confirmed regression bug** that blocks production readiness:

- ❌ `filter(pred, iterator_variable)` fails to compile (tries to clone a `dyn Iterator`)

This regression was identified in the initial review and has been independently verified in this review:
- Verified: `filter(is_even, iter(nums))` works ✅
- Verified: `filter(is_even, it)` where `it: Iterator[int] = iter(nums)` fails ❌

**Recommendation:** Return to implementation for regression fix. The filter codegen needs to correctly identify iterator variable types at codegen time and emit the correct branch (boxed iterator) instead of trying to clone and materialize to Vec.
