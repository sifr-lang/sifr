# milestone_safe_indexing: Safe Indexing and Option Returns

## Product Requirements

### Objective

Make all indexing and fallible collection operations safe by returning `Option[T]` instead of panicking. This eliminates the last remaining panic sources from collection access. Users handle `Option` with `try`/`except` (auto-unwrap), `_ = expr` (discard), or direct assignment.

### Scope

#### Features In

1. **Safe list indexing:** `list[i]` -> `Option[T]` (returns `None` if out-of-bounds)
2. **Safe dict indexing:** `dict[key]` -> `Option[V]` (returns `None` if key missing)
3. **Safe string indexing:** `str[i]` -> `Option[str]` (returns `None` if out-of-bounds)
4. **Negative indexing:** `list[-1]` -> `Option[T]` (resolved relative to length)
5. **List methods:** `.pop()` -> `Option[T]`, `.index(item)` -> `Option[int]`
6. **Dict methods:** `.get(key)` -> `Option[V]`, `.pop(key)` -> `Option[V]`
7. **String methods:** `.find(sub)` -> `Option[int]`
8. **`del` statement:** `del d[key]`, `del a[i]` as syntax sugar for `.pop()` with discard

#### Features Out

| Feature | Reason |
|---------|--------|
| `.remove(item)` -> `Result` | Deferred -- needs more complex error types |
| `.setdefault(key, default)` | Deferred -- needs default value semantics |
| `.rfind(sub)` | Deferred -- low priority |
| Tuple `.index(item)` | Deferred -- tuples are immutable, less common |
| `int ** negative_int` -> `Result` | Deferred -- complex static analysis |
| `.unwrap_or(default)` / `.expect("msg")` | Deferred to milestone_protocols |

## Solution Design

### Architecture

The key change is that `Option[T]` is sugar for `T | None` (union type), which already exists in the type system. Safe indexing means changing the return type of indexing operations from `T` to `T | None`.

```
sifr_type_system  (Option[T] = T | None already works via unions)
       ↓
sifr_hir          (update index/subscript lowering to return Option)
       ↓
sifr_codegen      (update index codegen to use .get()/.nth() instead of direct index)
       ↓
sifr (tests)      (E2E pass/fail tests)
```

### Key Design Decisions

1. **Option[T] = T | None**: No new type needed. The existing union type system handles this.
2. **Indexing returns Option**: `list[i]` returns `T | None`, requiring the user to handle the `None` case.
3. **Auto-unwrap in try blocks**: Inside `try` blocks, `Option` values can be auto-unwrapped (like `Result`).
4. **`del` as syntax sugar**: `del d[key]` maps to discarding the result of `.pop(key)`.

### Task Breakdown

**Task 1: Safe List Indexing**
- Change `list[i]` return type from `T` to `T | None`
- Update codegen to use `.get(i).cloned()` instead of `[i]`
- Handle negative indexing with safe bounds checking
- Update existing tests that use list indexing

**Task 2: Safe Dict & String Indexing**
- Change `dict[key]` return type from `V` to `V | None`
- Change `str[i]` return type from `str` to `str | None`
- Update codegen for dict (`.get(&key).cloned()`) and string (`.chars().nth(i)`)

**Task 3: Collection Methods with Option Returns**
- List `.pop()` -> `Option[T]`, `.index(item)` -> `Option[int]`
- Dict `.get(key)` -> `Option[V]`, `.pop(key)` -> `Option[V]`
- String `.find(sub)` -> `Option[int]`

**Task 4: Del Statement & E2E Tests**
- `del d[key]` -> `let _ = d.remove(&key);`
- `del a[i]` -> `let _ = a.remove(i);`
- E2E pass tests: safe_list_index, safe_dict_key, safe_string_index, safe_negative_index
- E2E fail tests: unused_option_error
- Milestone demo

### Testing Strategy

| Test | Layer | Check |
|------|-------|-------|
| safe_list_index | E2E pass | list[i] returns Option, None for OOB |
| safe_dict_key | E2E pass | dict[key] returns Option, None for missing |
| safe_string_index | E2E pass | str[i] returns Option, None for OOB |
| safe_negative_index | E2E pass | negative indexing with Option |
| list_pop_option | E2E pass | .pop() returns Option |
| dict_get_option | E2E pass | .get() returns Option |
| string_find_option | E2E pass | .find() returns Option |
| del_statement | E2E pass | del d[key], del a[i] |
| unused_option_error | E2E fail | unused Option value |
