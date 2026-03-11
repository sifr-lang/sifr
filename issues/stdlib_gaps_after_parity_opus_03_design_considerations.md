# Design Considerations

How Sifr's language design affects stdlib porting decisions.

---

## 1. Result/Option vs Exceptions

**CPython pattern:** Functions raise exceptions on failure (`ValueError`, `KeyError`, `IOError`, etc.)
**Sifr pattern:** Functions return `Result[T, E]` or `Option[T]`

### Impact on Stdlib Porting

Every CPython function that can raise an exception needs its return type changed:

| CPython Pattern | Sifr Pattern | Example |
|----------------|-------------|---------|
| Raises `ValueError` | Returns `Result[T, ValueError]` | `int("abc")` |
| Raises `KeyError` | Returns `Option[V]` | `dict["missing"]` |
| Raises `IndexError` | Returns `Option[T]` | `list[99]` |
| Raises `FileNotFoundError` | Returns `Result[T, IOError]` | `open("missing.txt")` |
| Raises `JSONDecodeError` | Returns `Result[T, JSONDecodeError]` | `json.loads("{bad")` |
| Raises `TimeoutError` | Returns `Result[T, TimeoutError]` | `socket.connect(...)` |

**Implication:** Every stdlib module needs its error types defined. This is currently deferred (see Phase 7 "What's Explicitly Deferred"). Before expanding the stdlib significantly, the error type export pipeline from stdlib `.sifr` files needs to be validated.

---

## 2. Ownership & Borrowing

**CPython pattern:** Everything is reference-counted; passing objects around is free.
**Sifr pattern:** Move semantics for heap types; borrow-by-default for function parameters.

### Impact on Stdlib Porting

| CPython Pattern | Sifr Consideration |
|----------------|-------------------|
| `list.sort()` mutates in-place | Needs `&mut self` receiver |
| `dict.update(other)` mutates | Needs `&mut self` receiver |
| `file.read()` consumes from stream | Needs `&mut self` receiver |
| Returning internal references | Must clone or use lifetimes |
| Callback functions stored in objects | Needs `Box<dyn Fn(...)>` (fixed in compiler_hardening) |
| Iterator holding reference to collection | Needs lifetime tracking |

**Key concern:** CPython's `csv.reader(file)` holds a reference to the file object. In Sifr, this requires either:
- The reader owns the file (takes `own file: File`)
- The reader borrows the file (requires lifetime tracking, which Sifr infers)
- The reader takes a path and opens internally (simpler but less flexible)

This pattern repeats across many stdlib modules: `json.load(fp)`, `xml.parse(fp)`, `configparser.read_file(fp)`, etc.

---

## 3. No Dynamic Typing / No Reflection

**CPython pattern:** `type()`, `getattr()`, `hasattr()`, `isinstance()` at runtime; duck typing.
**Sifr pattern:** All types known at compile time; protocols for structural typing.

### Modules That Can't Be Directly Ported

| CPython Module | Why It Can't Port Directly |
|----------------|---------------------------|
| `pickle` | Serializes arbitrary Python objects by inspecting their type at runtime |
| `shelve` | Built on `pickle` |
| `copy.deepcopy` | Recursively copies objects of unknown type |
| `json` (full) | `JSONEncoder.default()` dispatches on runtime type |
| `pprint` | Formats objects by inspecting their type at runtime |
| `inspect` | Runtime introspection of live objects |
| `functools.singledispatch` | Runtime type-based dispatch |

### Sifr Alternatives

| CPython Feature | Sifr Alternative |
|----------------|-----------------|
| `pickle` | `serde`-based serialization with compile-time derive |
| `copy.deepcopy` | `.clone()` (auto-derived) |
| `json` encoding | Derive `Serialize`/`Deserialize` traits |
| `pprint` | Auto-derived `Debug` trait |
| `singledispatch` | Protocol-based dispatch or match expressions |

---

## 4. No GC / Deterministic Destruction

**CPython pattern:** Garbage collector handles cleanup; `__del__` for custom destructors.
**Sifr pattern:** RAII / `Drop` semantics; `with` statement for explicit resource management.

### Impact on Stdlib Porting

| CPython Pattern | Sifr Pattern |
|----------------|-------------|
| `file = open("x.txt")` (GC closes eventually) | Must use `with open("x.txt") as file:` or explicit close |
| `socket.socket()` (GC closes) | Must use `with` or explicit close |
| `sqlite3.connect(db)` (GC closes) | Must use `with` or explicit close |
| `tempfile.NamedTemporaryFile()` | Context manager with `Drop` |

**Implication:** Every resource-holding class needs to implement `ContextManager` protocol. The `with` statement is not optional in Sifr — it's the primary resource management mechanism.

---

## 5. Static Typing & Generics

**CPython pattern:** Functions accept "any iterable", "any sequence", "any mapping" via duck typing.
**Sifr pattern:** Generic functions with protocol bounds (`T: Comparable`, `T: Iterator`).

### Impact on Stdlib Porting

| CPython Pattern | Sifr Requirement |
|----------------|-----------------|
| `sorted(iterable)` | `sorted[T: Comparable](data: list[T]) -> list[T]` |
| `min(iterable)` | `min[T: Comparable](data: list[T]) -> T` |
| `functools.reduce(func, iterable)` | `reduce[T](func: Callable[[T, T], T], data: list[T]) -> T` |
| `itertools.chain(*iterables)` | `chain[T](a: Iterator[T], b: Iterator[T]) -> Iterator[T]` |
| `collections.Counter(iterable)` | `Counter[T: Hashable](data: list[T])` |

**Current state:** Generic stdlib functions were proven in Phase 7 (`bisect`, `heapq`, `itertools` use `TypeVar`). But many stdlib functions still use concrete types (`list[int]`, `list[str]`) instead of generics.

---

## 6. Module Organization Divergences

Sifr has made some organizational choices that differ from CPython:

| CPython | Sifr | Assessment |
|---------|------|-----------|
| `os.environ` / `os.getenv` | `sifr.env.env_get` | **Should consolidate** — env vars should be in `sifr.os` |
| `os.path.join` | `sifr.pathlib.join_path` | **Acceptable** — CPython is moving toward `pathlib` too |
| `bytes` built-in type | `sifr.bytes` module | **Should be built-in** — `bytes` is a fundamental type |
| `open()` built-in | `sifr.io.read_text` | **Should add `open()`** — it's Python's most-used function |
| `sys.argv` | `sifr.os.get_args()` | **Acceptable** — but should also have `sifr.sys` |
| `functools.identity/clamp` | Not in CPython | **Should remove or rename** — confusing for Python users |

---

## 7. What Should NOT Be Ported

Some CPython stdlib modules fundamentally don't make sense in a compiled language:

### Definitely Skip

| Module | Reason |
|--------|--------|
| `gc` | No garbage collector |
| `weakref` | Ownership model handles this |
| `ast` / `compile` / `exec` / `eval` | No runtime code execution |
| `importlib` / `pkgutil` / `zipimport` | Sifr has its own import system |
| `tokenize` / `token` / `keyword` | Python-specific |
| `symtable` / `codeop` / `py_compile` | Python compiler internals |
| `copyreg` / `pickletools` | Pickle-specific |
| `site` / `ensurepip` / `venv` | Python environment management |
| `lib2to3` | Python migration tool |
| `__future__` | Python version compatibility |
| All deprecated modules | Don't port dead code |

### Replace with Language Features

| CPython Module | Sifr Language Feature |
|----------------|----------------------|
| `dataclasses` | All classes auto-derive Debug, Clone, PartialEq |
| `enum` | Union types + literal types |
| `abc` | Protocols |
| `typing` | Built-in static type system |
| `copy` | `.clone()` auto-derived |
| `pprint` | Auto-derived `Debug` / `Display` |
| `contextlib.contextmanager` | Generator-based context managers (if supported) |

### Replace with Rust Ecosystem

| CPython Module | Rust Crate | Notes |
|----------------|------------|-------|
| `pickle` | `serde` + `serde_json` / `bincode` / `messagepack` | Compile-time serialization |
| `struct` | `bytemuck` / `zerocopy` | Zero-copy binary data |
| `ctypes` | Sifr FFI (milestone_ffi) | Direct Rust crate access |

---

## 8. Recommended Porting Strategy

### Phase 1: Fix Existing Modules (Deepen Coverage)
Focus on bringing the 37 existing modules from ~35% to ~70% average coverage. This means:
- Adding missing functions to `math`, `statistics`, `random`, `collections`
- Making `itertools` match CPython's iterator types
- Replacing `functools.identity/clamp` with `reduce`, `partial`
- Adding `choice`, `shuffle`, `sample` to `random`
- Adding `deque` to `collections`

### Phase 2: Add Critical Missing Modules
- `subprocess` — process management
- `sys` — system parameters
- `socket` — networking
- `http` — HTTP client/server
- `asyncio` — async runtime (already planned for Phase 8)
- `sqlite3` — database
- `configparser` — INI files
- `xml.etree.ElementTree` — XML processing

### Phase 3: Add Convenience Modules
- Compression (`zipfile`, `gzip`, `tarfile`)
- `decimal` / `fractions` — precise arithmetic
- `calendar` / `zoneinfo` — date utilities
- `smtplib` / `imaplib` — email protocols
- `unicodedata` — Unicode support
