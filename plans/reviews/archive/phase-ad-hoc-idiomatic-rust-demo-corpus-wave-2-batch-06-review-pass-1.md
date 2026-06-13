I'll review each file for behavioral equivalence, ownership/API/design quality, and readability.

## demos/collections/idiomatic.rs

**Behavioral Equivalence:**
- `collect_set_and_counter_actual` - correct
- `collect_deque_actual` - correct

**Actionable Issues:**

1. **`append` method has a boundary bug at `maxlen=0`** (`Deque::append` line 116):
   ```rust
   if self.values.len() == limit {
       self.values.pop_front();
   }
   ```
   When `limit` equals the current length, it pops before checking if it should stop. For `maxlen=0`, this allows one element to be added before the early return. The early return is correct, but this code path is dead code that could confuse readers.

## demos/itertools/idiomatic.rs

**Behavioral Equivalence:**
- All functions match the Sifr reference.

**Actionable Issues:**

None.

## demos/heapq/idiomatic.rs

**Behavioral Equivalence:**
- `collect_actual` - all assertions correct.

**Actionable Issues:**

1. **`nsmallest` and `nlargest` do unnecessary allocation** (`nsmallest` line 44, `nlargest` line 50):
   ```rust
   fn nsmallest<T: Ord + Clone>(n: usize, values: &[T]) -> Vec<T> {
       let mut result = values.to_vec();  // unnecessary copy
       result.sort();
       result.truncate(n);
       result
   }
   ```
   This creates a full copy then truncates. An idiomatic Rust approach would sort in place:
   ```rust
   let mut result = values.to_vec();
   result.sort();
   result.truncate(n);
   ```
   While `sort()` takes `&mut self`, it can't be called directly on a slice (`&[T]`). The `to_vec()` is necessary for `sort()`, but for `nlargest` you could use `select_nth_unstable` for better algorithmic complexity on large inputs. However, for the demo's educational purpose and small inputs, this is acceptable.

## Summary

**No actionable issues** in `collections/idiomatic.rs` and `itertools/idiomatic.rs`. 

For `heapq/idiomatic.rs`: The `to_vec()` in `nsmallest`/`nlargest` is a minor inefficiency but not incorrect for the demo's purpose and scale. The boundary bug in `Deque::append` at `maxlen=0` is a latent issue not exercised by the tests.
