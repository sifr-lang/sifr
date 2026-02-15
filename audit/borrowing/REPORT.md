# Borrowing & Ownership Audit Report

**Date:** February 15, 2026  
**Scope:** 50 test files in `audit/borrowing/`  
**Goal:** Determine how much of Sifr's borrowing/ownership behavior matches Rust semantics versus Python semantics.

---

## Summary

| Status | Count | Percentage |
|--------|-------|------------|
| **PASS** | 37 | 74.0% |
| **Fail (Sifr compile)** | 5 | 10.0% |
| **Fail (Rust compile)** | 8 | 16.0% |
| **Fail (runtime)** | 0 | 0.0% |
| **Total** | 50 | 100% |

### Failure Breakdown

| Category | Count | Description |
|----------|-------|-------------|
| **Correct Sifr rejection** | 4 | Sifr correctly detects use-after-move (Rust behavior) |
| **Sifr type error** | 1 | `list.pop()` returns `int \| None` but test expected `int` |
| **Missing Sifr move detection (assignment)** | 6 | Sifr doesn't detect move via `x = y` assignment; Rust catches it |
| **Codegen: Display not implemented** | 2 | `HashSet<i64>` doesn't implement `Display` for `println!` |

---

## Rust vs Python Behavior Analysis

### Behaviors That Match Rust (Correct)

These behaviors demonstrate Sifr successfully implementing Rust-like ownership semantics:

| # | Test | Behavior | Rust Match |
|---|------|----------|------------|
| 01 | Copy types reuse | `int`, `float`, `bool` are Copy — reusable after assignment | Yes |
| 02 | String move on assignment | `s2 = s1` moves s1 (codegen emits move) | Yes |
| 04 | List move on assignment | `l2 = l1` moves l1 (codegen emits move) | Yes |
| 06 | Dict move on assignment | `d2 = d1` moves d1 (codegen emits move) | Yes |
| 08 | Move into function | Passing list to function consumes it | Yes |
| 09 | Use-after-move (function) | `consume(nums); print(nums)` — correctly rejected by Sifr | Yes |
| 10 | Copy type into function | `int` passed to function, still usable | Yes |
| 11 | print() borrows | `print(s); print(s)` works — print doesn't move | Yes |
| 12 | len() borrows | `len(s); print(s)` works — len doesn't move | Yes |
| 13 | String methods borrow | `.upper()`, `.lower()` don't consume original | Yes |
| 14 | Reassignment resets move | After move, `s = "new"` makes s usable again | Yes |
| 15 | Borrow in loop | `len(s)` in loop body works (borrows each iteration) | Yes |
| 17 | For loop borrows | `for n in nums` borrows; nums usable after loop | Yes |
| 18 | Multiple for loops | Same collection iterated multiple times | Yes |
| 19 | Class instance move | `p2 = p1` moves class instance | Yes |
| 21 | Method borrows &self | Read-only method doesn't consume object | Yes |
| 22 | Method &mut self | Mutating method works (auto &mut self) | Yes |
| 23 | Class passed to function | Class instance moved into function | Yes |
| 24 | Use class after function move | Correctly rejected by Sifr | Yes |
| 25 | Tuple move on assignment | `t2 = t1` — tuple treated as move type | Partial* |
| 27 | List append after use | Mutation after borrow works | Yes |
| 28 | Return local string | Local string moved out of function | Yes |
| 29 | Return local list | Local list moved out of function | Yes |
| 30 | Multiple calls with Copy | `int` passed to multiple functions | Yes |
| 31 | String to two functions | Second call correctly rejected (use-after-move) | Yes |
| 34 | sorted() borrows | Collection reusable after sorted() | Yes |
| 35 | enumerate() borrows | Collection reusable after enumerate() | Yes |
| 36 | zip() borrows | Both collections reusable after zip() | Yes |
| 37 | Comprehension borrows | Source list reusable after comprehension | Yes |
| 38 | Dict methods borrow | `.keys()` borrows; dict reusable | Yes |
| 39 | Nested collection move | Nested list moved on assignment | Yes |
| 41 | F-string borrows | Variables reusable after f-string | Yes |
| 42 | Chained string methods | Method chains on temporaries work | Yes |
| 43 | List sort in-place | `.sort()` mutates via &mut self | Yes |
| 45 | Multiple borrows | Multiple len/print/for on same collection | Yes |
| 48 | Built-in functions borrow | min/max/sum/any/all don't consume | Yes |
| 49 | String concatenation | `a + b` creates new string | Yes |
| 50 | Augmented string assign | `s = s + " world"` works | Yes |

*\*Test 25/26: Sifr marks tuples as Move, but `(i64, i64)` is actually Copy in Rust. The test passes at runtime because Rust's Copy semantics kick in, even though Sifr's type system is more conservative.*

### Behaviors That Differ from Python (By Design)

These are cases where Sifr intentionally diverges from Python to enforce Rust-like safety:

| Python Behavior | Sifr Behavior | Rationale |
|----------------|---------------|-----------|
| `s2 = s1` — both usable (shared ref) | `s2 = s1` — s1 is moved | Move semantics prevent aliasing |
| `l2 = l1` — both usable (shared ref) | `l2 = l1` — l1 is moved | Move semantics prevent aliasing |
| `consume(s); print(s)` — works | Rejected: use-after-move | Ownership tracking |
| `d2 = d1` — both usable (shared ref) | `d2 = d1` — d1 is moved | Move semantics prevent aliasing |
| `p2 = p1` — both usable (shared ref) | `p2 = p1` — p1 is moved | Move semantics prevent aliasing |

---

## Issues Found

### Issue 1: Move Detection Gap — Assignment-Based Moves Not Caught by Sifr (6 tests)

**Severity:** Medium  
**Impact:** Sifr's type checker (`sifr check`) does NOT detect use-after-move when the move happens via variable assignment (`s2 = s1`). It only detects moves through function calls. The Rust backend catches these errors, so programs don't compile incorrectly — but the error messages come from `rustc` instead of Sifr's own diagnostics.

| Test | Pattern | Sifr Check | Rust Compile |
|------|---------|------------|--------------|
| 03 | `s2 = s1; print(s1)` (str) | No error | E0382: borrow of moved value |
| 05 | `l2 = l1; print(l1)` (list) | No error | E0382: borrow of moved value |
| 07 | `d2 = d1; print(d1)` (dict) | No error | E0382: borrow of moved value |
| 16 | `consume(s)` in loop body | No error | E0382: use of moved value |
| 20 | `p2 = p1; print(p1.x)` (class) | No error | E0382: borrow of moved value |
| 40 | `inner = outer; print(outer)` (nested list) | No error | E0382: borrow of moved value |

**Root Cause:** The HIR's `mark_moved()` is only called when a variable is passed as an argument to a user-defined function (in `lower.rs` around line 3610). Assignment (`s2 = s1`) does not call `mark_moved()` on `s1`. The codegen correctly emits `let s2 = s1` (which is a Rust move), but Sifr's own checker doesn't track it.

**Fix:** In the assignment lowering code, when the RHS is a `Name` expression referencing a Move-type variable, call `ctx.scope.mark_moved(name)` on the source variable.

### Issue 2: Move-in-Loop Not Detected by Sifr (1 test)

**Severity:** Medium  
**Impact:** When a Move-type variable is consumed inside a loop body, Sifr doesn't detect that the second iteration would use a moved value. Rust catches this.

| Test | Pattern | Sifr Check | Rust Compile |
|------|---------|------------|--------------|
| 16 | `for i in range(3): consume(s)` | No error | E0382: use of moved value |

**Root Cause:** Sifr's move tracking is linear (single-pass). It doesn't consider that loop bodies execute multiple times, so a move in the loop body means the variable is moved on the first iteration and unavailable on subsequent iterations.

**Fix:** Before lowering a loop body, snapshot the scope's moved state. After lowering, check if any variables were newly moved inside the body — if so, they would be moved on the first iteration and unavailable on subsequent ones. Emit a "value moved inside loop body" error.

### Issue 3: `list.pop()` Return Type (1 test)

**Severity:** Low  
**Impact:** `list.pop()` returns `int | None` (Option type) in Sifr, which is correct for safe indexing. The test expected `int` directly.

| Test | Pattern | Error |
|------|---------|-------|
| 44 | `last: int = nums.pop()` | type mismatch: expected 'int', got 'int \| None' |

**This is correct behavior** — Sifr's safety philosophy requires handling the None case. The test annotation should use `int | None` or handle the Option.

### Issue 4: `HashSet` Display Not Implemented (2 tests)

**Severity:** Low  
**Impact:** `print(set)` fails because `HashSet<i64>` doesn't implement `Display` in Rust. The codegen should emit `Debug` formatting (`{:?}`) for set types, similar to how it handles other collection types.

| Test | Pattern | Error |
|------|---------|-------|
| 46 | `print(s2)` where s2 is `set[int]` | HashSet doesn't implement Display |
| 47 | `print(s1)` where s1 is `set[int]` | HashSet doesn't implement Display |

**Note:** Test 47 was designed to test use-after-move for sets, but it can't even reach that point because `print(set)` fails first.

---

## Ownership Model Summary

### What Sifr Gets Right (Rust-like)

1. **Copy vs Move distinction** — `int`, `float`, `bool` are Copy; `str`, `list`, `dict`, `set`, `tuple`, classes are Move
2. **Function argument moves** — passing a Move type to a user-defined function marks it as moved
3. **Built-in function borrows** — `print`, `len`, `sorted`, `enumerate`, `zip`, `min`, `max`, `sum`, `any`, `all` all borrow their arguments
4. **String method borrows** — `.upper()`, `.lower()`, `.strip()` etc. borrow &self
5. **For loop borrows** — `for x in collection` borrows; collection is reusable after
6. **Reassignment resets move** — `s = "new"` after a move makes `s` usable again
7. **Method receiver inference** — read-only methods get `&self`, mutating methods get `&mut self`
8. **Returning local values** — local Move types can be returned (moved out)
9. **Comprehension borrows** — list comprehensions borrow the source collection
10. **F-string borrows** — format strings borrow interpolated variables

### What Sifr Misses (Gaps)

1. **Assignment-based moves** — `s2 = s1` doesn't mark `s1` as moved in Sifr's checker (Rust catches it)
2. **Move-in-loop detection** — consuming a variable in a loop body isn't detected by Sifr
3. **Conditional move analysis** — Sifr doesn't track whether a variable is moved in one branch vs both branches of an if/else

### What Python Would Do Differently

In Python, **none of the "should fail" tests would fail**. Python uses reference counting and garbage collection — all variables remain usable after assignment or function calls. The concept of "move" doesn't exist. Sifr's move semantics are a fundamental departure from Python, aligning with Rust's ownership model for memory safety without a garbage collector.

---

## Test File Index

| File | Status | Category | Expected |
|------|--------|----------|----------|
| 01_copy_types_reuse.sifr | PASS | Copy semantics | Pass |
| 02_move_str_assignment.sifr | PASS | Move semantics | Pass |
| 03_move_str_use_after_assign.sifr | FAIL (Rust) | Move detection gap | Should fail (Sifr should catch) |
| 04_move_list_assignment.sifr | PASS | Move semantics | Pass |
| 05_move_list_use_after_assign.sifr | FAIL (Rust) | Move detection gap | Should fail (Sifr should catch) |
| 06_move_dict_assignment.sifr | PASS | Move semantics | Pass |
| 07_move_dict_use_after_assign.sifr | FAIL (Rust) | Move detection gap | Should fail (Sifr should catch) |
| 08_move_into_function.sifr | PASS | Function move | Pass |
| 09_use_after_move_into_function.sifr | FAIL (Sifr) | Correct rejection | Expected fail |
| 10_copy_type_into_function.sifr | PASS | Copy semantics | Pass |
| 11_print_borrows_not_moves.sifr | PASS | Borrow semantics | Pass |
| 12_len_borrows_not_moves.sifr | PASS | Borrow semantics | Pass |
| 13_string_method_borrow.sifr | PASS | Borrow semantics | Pass |
| 14_reassignment_resets_move.sifr | PASS | Move reset | Pass |
| 15_move_in_loop.sifr | PASS | Borrow in loop | Pass |
| 16_move_in_loop_consume.sifr | FAIL (Rust) | Move-in-loop gap | Should fail (Sifr should catch) |
| 17_for_loop_borrows_collection.sifr | PASS | Iterator borrow | Pass |
| 18_multiple_for_loops_same_collection.sifr | PASS | Iterator borrow | Pass |
| 19_class_instance_move.sifr | PASS | Class move | Pass |
| 20_class_instance_use_after_move.sifr | FAIL (Rust) | Move detection gap | Should fail (Sifr should catch) |
| 21_class_method_self_borrow.sifr | PASS | Method &self | Pass |
| 22_class_method_mut_self.sifr | PASS | Method &mut self | Pass |
| 23_pass_class_to_function.sifr | PASS | Class function move | Pass |
| 24_use_class_after_function_move.sifr | FAIL (Sifr) | Correct rejection | Expected fail |
| 25_tuple_copy_vs_move.sifr | PASS | Tuple semantics | Pass |
| 26_tuple_use_after_move.sifr | PASS | Tuple Copy in Rust | Pass* |
| 27_list_append_after_use.sifr | PASS | Mutation after borrow | Pass |
| 28_return_local_string.sifr | PASS | Return move type | Pass |
| 29_return_local_list.sifr | PASS | Return move type | Pass |
| 30_multiple_function_calls_same_var.sifr | PASS | Copy reuse | Pass |
| 31_str_move_into_two_functions.sifr | FAIL (Sifr) | Correct rejection | Expected fail |
| 32_conditional_move.sifr | PASS | Conditional move | Pass |
| 33_move_in_both_branches.sifr | FAIL (Sifr) | Correct rejection | Expected fail |
| 34_sorted_borrows.sifr | PASS | Built-in borrow | Pass |
| 35_enumerate_borrows.sifr | PASS | Built-in borrow | Pass |
| 36_zip_borrows.sifr | PASS | Built-in borrow | Pass |
| 37_list_comprehension_borrows.sifr | PASS | Comprehension borrow | Pass |
| 38_dict_method_borrow.sifr | PASS | Dict method borrow | Pass |
| 39_nested_collection_move.sifr | PASS | Nested move | Pass |
| 40_nested_collection_use_after_move.sifr | FAIL (Rust) | Move detection gap | Should fail (Sifr should catch) |
| 41_fstring_borrows.sifr | PASS | F-string borrow | Pass |
| 42_chained_string_methods.sifr | PASS | Temporary lifetime | Pass |
| 43_list_sort_in_place.sifr | PASS | Mutation &mut self | Pass |
| 44_list_pop_mutates.sifr | FAIL (Sifr) | Option return type | Correct (safety) |
| 45_multiple_borrows_same_scope.sifr | PASS | Multiple borrows | Pass |
| 46_set_move_semantics.sifr | FAIL (Rust) | Display not impl | Codegen bug |
| 47_set_use_after_move.sifr | FAIL (Rust) | Display not impl | Codegen bug |
| 48_builtin_functions_borrow.sifr | PASS | Built-in borrow | Pass |
| 49_str_concat_creates_new.sifr | PASS | String ops | Pass |
| 50_augmented_assign_str.sifr | PASS | Augmented assign | Pass |

*\*Test 26: Sifr marks tuples as Move, but `(i64, i64)` is Copy in Rust — the test passes because Rust's Copy trait kicks in.*

---

## Verdict

**Sifr's borrowing model is fundamentally Rust-like, not Python-like.** The language successfully implements:

- Move-by-default for heap types (str, list, dict, set, classes)
- Copy for primitive types (int, float, bool)
- Borrow semantics for built-in functions, methods, for loops, and comprehensions
- Method receiver inference (&self vs &mut self)

The main gap is that **Sifr's own type checker only tracks moves through function calls**, not through variable assignments. Assignment-based moves (`s2 = s1`) are correctly enforced by the Rust backend, but Sifr doesn't report them with its own diagnostics. This means users see `rustc` error messages (E0382) instead of Sifr's friendlier error format for these cases.

**Effective pass rate (accounting for expected failures):** 43/50 tests behave as designed (37 pass + 4 correct Sifr rejections + 1 correct safety type error + 1 Rust-level Copy behavior). The remaining 8 are: 6 move-detection gaps (caught by Rust) and 2 codegen Display bugs for sets.
