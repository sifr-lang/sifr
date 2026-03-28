# milestone_ergonomics: Language Ergonomics

## Product Requirements

### Objective

Add essential language features that make Sifr pleasant to use for everyday programming. These features work with concrete types only (no `Option`/`Result` dependency). This milestone bridges the gap between a "working compiler" and a "usable language."

### Scope

#### Features In

1. Augmented assignment operators (`+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`)
2. Conditional expressions (ternary: `a if cond else b`)
3. Keyword arguments and default parameter values
4. For-loop borrow semantics (collection usable after loop)
5. List slice copy semantics
6. Negative indexing (`a[-1]`)
7. Step slicing (`a[::2]`, `a[::-1]`)
8. Tuple slicing (compile-time constant indices)
9. String UTF-8 fixes (character-based indexing)
10. List methods (append, extend, insert, clear, copy, reverse, count, contains, sort)
11. Dict methods (keys, values, items, update, clear, copy, contains)
12. Extended string methods (replace, startswith, endswith, join, count, is* methods, etc.)
13. Tuple methods (count, immutability enforcement)
14. Built-in functions (len, abs, round, repr)
15. Chained comparisons (`1 < x < 10`)
16. String multiplication (`"-" * 40`)
17. `pass` statement
18. Star unpacking (`first, *rest = items`)
19. Walrus operator (`:=`)
20. Power operator codegen (`**`)
21. Multiple return values (tuple packing)
22. `for`/`while` ... `else` clauses

#### Features Out

| Feature | Reason |
|---------|--------|
| Safe indexing (`Option` returns) | Deferred to milestone_safe_indexing |
| `.pop()`, `.get()`, `.find()` | Require `Option`/`Result` types |
| `*args`/`**kwargs` | Deferred to milestone_decorators |
| Generic sorting (key functions) | Deferred to milestone_generics |

## Solution Design

### Architecture

All changes span four crates in the pipeline:

```
sifr_type_system  (new type checking rules, method signatures)
       ↓
sifr_hir          (new HIR nodes, lowering logic)
       ↓
sifr_codegen      (Rust code emission for new features)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### Task Breakdown

**Task 1: Augmented Assignment & Core Syntax**
- Add `AugAssign` HIR node
- Add `IfExpr` (ternary) support
- Add `pass` statement
- Add `for`/`while` ... `else` clauses
- Add string multiplication
- Add power operator codegen

**Task 2: Keyword Arguments & Function Ergonomics**
- Extend `HirParam` with default values
- Extend `HirCall` with keyword argument resolution
- Add keyword-only parameter support (after `*`)
- Add multiple return value testing

**Task 3: Indexing & Slicing**
- Negative indexing for lists, strings, tuples
- Step slicing (`[start:stop:step]`)
- Tuple slicing (compile-time)
- UTF-8 string fixes (character-based indexing)
- For-loop borrow semantics
- List slice copy semantics

**Task 4: Collection & String Methods**
- List methods (append, extend, insert, clear, copy, reverse, count, contains, sort)
- Dict methods (keys, values, items, update, clear, copy, contains)
- Extended string methods
- Tuple methods (count)
- Built-in functions (len enhancement, abs, round, repr)

**Task 5: Advanced Syntax Sugar**
- Chained comparisons
- Star unpacking (`first, *rest = items`)
- Walrus operator (`:=`)

**Task 6: Comprehensive E2E Tests**
- 27 pass tests, 3 fail tests
- Regression tests for M1/M2/M3

### Testing Strategy

| Test | Layer | Check |
|------|-------|-------|
| augmented_assign | E2E pass | `+=`, `-=` for int, str, list |
| ternary_expr | E2E pass | Conditional expression |
| keyword_args_basic | E2E pass | Named arguments |
| keyword_args_default | E2E pass | Default values |
| keyword_only_params | E2E pass | `*` separator |
| negative_index_list | E2E pass | `a[-1]` on list |
| negative_index_string | E2E pass | `s[-1]` on string |
| step_slice_basic | E2E pass | `a[::2]` |
| step_slice_reverse | E2E pass | `a[::-1]` |
| step_slice_string | E2E pass | String step slicing |
| tuple_slice | E2E pass | Compile-time tuple slicing |
| string_char_index | E2E pass | UTF-8 character indexing |
| string_char_len | E2E pass | Character count vs byte count |
| string_slice | E2E pass | Character-based slicing |
| list_methods_concrete | E2E pass | All concrete list methods |
| dict_methods_concrete | E2E pass | All concrete dict methods |
| string_replace | E2E pass | String method suite |
| chained_comparison | E2E pass | `1 < x < 10` |
| string_multiply | E2E pass | `"-" * 40` |
| pass_statement | E2E pass | Empty bodies |
| star_unpacking | E2E pass | `first, *rest = items` |
| walrus_operator | E2E pass | `:=` in conditions |
| power_operator | E2E pass | `x ** y` codegen |
| multiple_return | E2E pass | Tuple return |
| loop_else | E2E pass | `for`/`while` else |
| for_loop_borrow | E2E pass | Collection usable after loop |
| list_slice_copy | E2E pass | Slice produces copy |
| ternary_type_mismatch | E2E fail | Mismatched branch types |
| keyword_after_positional_error | E2E fail | Wrong argument order |
| missing_keyword_only_arg | E2E fail | Missing required kwarg |
