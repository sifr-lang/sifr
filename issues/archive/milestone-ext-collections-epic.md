# milestone_ext_collections — Extended Collections and Binary Data

## 1. Product Requirements

### Objective

Provide Python's extended collection types and binary data handling via stdlib modules. These types are commonly needed in real programs but were not part of the core foundation.

### Scope — Scoped Down for Initial Implementation

**In Scope:**

1. **`sifr.collections.Set`** — Set operations: `new_set()`, `set_add()`, `set_contains()`, `set_remove()`, `set_len()`, `set_union()`, `set_intersection()`
2. **`sifr.collections.Counter`** — `counter_new(items)`, `counter_get(key)`, `counter_most_common(n)`
3. **`sifr.collections.DefaultDict`** — `defaultdict_new(default)`, `defaultdict_get(key)`, `defaultdict_set(key, value)`
4. **`sifr.bytes`** — `encode(s)` (str→bytes as list[int]), `decode(bytes)` (list[int]→str), `bytes_hex(bytes)`, `bytes_from_hex(s)`

Since the type system doesn't support generic custom types yet, we'll implement these as stdlib functions that operate on existing types:
- Set → `HashSet<i64>` (specialized for int)
- Counter → `HashMap<String, i64>`
- DefaultDict → `HashMap<String, i64>`
- bytes → `Vec<u8>` represented as `list[int]`

**Out of Scope (deferred):**

| Feature | Reason |
| --- | --- |
| frozenset (immutable set) | Requires compile-time mutation rejection |
| bytearray (mutable bytes) | Redundant with list[int] approach |
| Set operators (`\|`, `&`, `-`, `^`) | Requires operator overloading for new types |
| Generic Set/Counter/DefaultDict | Requires generics support |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | Set operations work: create, add, contains, remove, union, intersection |
| AC-2 | Counter counts elements and supports most_common |
| AC-3 | DefaultDict auto-creates default values on missing key access |
| AC-4 | bytes encode/decode between str and list[int] |
| AC-5 | All existing E2E tests pass (no regressions) |

---

## 2. Solution Design

### 2.1 Implementation as Stdlib Functions

Since we can't add new types to the type system without generics, we implement collections as stdlib functions that return existing types:

| Sifr Function | Return Type | Rust Code |
| --- | --- | --- |
| `new_set()` | `list[int]` | `Vec::new()` (use sorted vec as set) |
| `set_add(s, item)` | `list[int]` | push + dedup |
| `set_contains(s, item)` | `bool` | `.contains()` |
| `set_union(a, b)` | `list[int]` | merge + dedup |
| `counter_from_list(items)` | `str` | JSON-encoded HashMap |
| `encode_utf8(s)` | `list[int]` | `.as_bytes().to_vec()` |
| `decode_utf8(bytes)` | `str` | `String::from_utf8()` |
| `bytes_to_hex(bytes)` | `str` | hex encoding |

Actually, a simpler approach: use the existing `dict[str, int]` type for Counter/DefaultDict, and `list[int]` for Set/bytes. The stdlib functions just provide convenient constructors and operations.

### 2.2 Testing Strategy

- E2E tests for each collection type
- Demo: `demos/milestone_ext_collections_demo.sifr`
