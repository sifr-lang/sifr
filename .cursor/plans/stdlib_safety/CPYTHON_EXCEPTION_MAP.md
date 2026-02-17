# CPython Exception → Sifr Safety Mapping

**Date:** 2026-02-17

This document maps every CPython exception that occurs in the stdlib modules sifr has implemented, showing the expected sifr safety adaptation and current status.

---

## Mapping Rules (from `architecture.md`)

| CPython Pattern | Sifr Adaptation | Rule # |
| --- | --- | --- |
| Raises exception → | Returns `Result[T, E]` | Rule 1 |
| Raises `IndexError` → | Returns `Option[T]` | Rule 2 |
| Raises `KeyError` → | Returns `Option[V]` | Rule 3 |
| Silent overflow → | Rust default (panic debug / wrap release) | Rule 4 |
| Runtime mutation of immutable → | Compile-time error | Rule 5 |
| Undefined/platform-dependent → | Explicit defined behavior | Rule 6 |

---

## Module: `os` / `io` / `shutil` / `pathlib`

### File System Operations

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `FileNotFoundError` | `open(missing)`, `read(missing)`, `remove(missing)`, `listdir(missing)` | `Result[T, IOError]` | **PANICS** (`.unwrap()`) |
| `PermissionError` | `write(readonly)`, `mkdir(no_perms)`, `remove(no_perms)` | `Result[T, IOError]` | **PANICS** |
| `FileExistsError` | `mkdir(existing)`, `rename(to_existing)` | `Result[T, IOError]` | **PANICS** |
| `IsADirectoryError` | `remove(directory)`, `open(directory, 'w')` | `Result[T, IOError]` | **PANICS** |
| `NotADirectoryError` | `listdir(file)`, `rmdir(file)` | `Result[T, IOError]` | **PANICS** |
| `OSError` | `getcwd()` (deleted cwd), `rmdir(nonempty)` | `Result[T, IOError]` | **PANICS** |

### Recommended Error Type

```
class IOError:
    message: str
    path: str
    kind: str  # "not_found" | "permission" | "exists" | "is_directory" | "not_directory" | "other"
```

---

## Module: `json`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `json.JSONDecodeError` | `loads(invalid_json)` | `Result[str, ParseError]` | **PANICS** |
| `TypeError` | `dumps(non_serializable)` | `Result[str, TypeError]` | Not applicable (sifr uses string) |
| `ValueError` | `loads("")` | `Result[str, ParseError]` | **PANICS** |

---

## Module: `tomllib`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `tomllib.TOMLDecodeError` | `loads(invalid_toml)` | `Result[str, ParseError]` | **PANICS** |
| `FileNotFoundError` | `load(missing_file)` | `Result[str, IOError]` | **PANICS** |

---

## Module: `re`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `re.error` | `search(invalid_regex, text)` | `Result[str \| None, RegexError]` | **PANICS** |
| `re.error` | `sub(invalid_regex, repl, text)` | `Result[str, RegexError]` | **PANICS** |
| `re.error` | `findall(invalid_regex, text)` | `Result[list[str], RegexError]` | **PANICS** |
| `re.error` | `split(invalid_regex, text)` | `Result[list[str], RegexError]` | **PANICS** |
| Returns `None` | `search(pattern, text)` no match | `str \| None` | **CORRECT** |

---

## Module: `base64`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `binascii.Error` | `b64decode(invalid_base64)` | `Result[str, ParseError]` | **PANICS** |
| `binascii.Error` | `urlsafe_b64decode(invalid)` | `Result[str, ParseError]` | **PANICS** |

---

## Module: `bytes`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `UnicodeDecodeError` | `decode_utf8(invalid_bytes)` | `Result[str, ParseError]` | **PANICS** |
| `ValueError` | `bytes.fromhex(invalid_hex)` | `Result[str, ParseError]` | **PANICS** |

---

## Module: `math`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `sqrt(-1)` | `Result[float, ValueError]` | Returns NaN (silent) |
| `ValueError` | `log(0)`, `log(-1)` | `Result[float, ValueError]` | Returns -inf/NaN (silent) |
| `ValueError` | `factorial(-1)` | `Result[int, ValueError]` | **PANICS** |
| `OverflowError` | `factorial(very_large)` | `Result[int, OverflowError]` | **PANICS** (i64 overflow) |
| `ValueError` | `asin(2)`, `acos(2)` | `Result[float, ValueError]` | Returns NaN (silent) |

---

## Module: `statistics`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `StatisticsError` | `mean([])` | `Result[float, StatisticsError]` | **PANICS** (div by zero) |
| `StatisticsError` | `median([])` | `Result[float, StatisticsError]` | Returns 0.0 (wrong) |
| `StatisticsError` | `variance([])` | `Result[float, StatisticsError]` | **PANICS** |
| `StatisticsError` | `variance([x])` (need ≥2) | `Result[float, StatisticsError]` | **PANICS** |
| `StatisticsError` | `stdev([])` | `Result[float, StatisticsError]` | **PANICS** |
| `StatisticsError` | `mode([])` | `Result[int, StatisticsError]` | **PANICS** |
| `StatisticsError` | `harmonic_mean` with zero | `Result[float, StatisticsError]` | **PANICS** |

---

## Module: `heapq`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `IndexError` | `heappop([])` | `Option[int]` | **PANICS** |
| `IndexError` | `heapreplace([], item)` | `Result[int, IndexError]` | **PANICS** |

---

## Module: `collections`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `KeyError` | `set.pop()` on empty | `Option[str]` | **PANICS** |
| `KeyError` | `dict[missing_key]` | `Option[V]` | N/A (not in sifr collections) |

---

## Module: `graphlib`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `CycleError` | `static_order()` on cyclic graph | `Result[list[int], CycleError]` | **UNDEFINED** (no cycle detection) |
| `ValueError` | `prepare()` called twice | `Result[None, ValueError]` | N/A (no prepare method) |

---

## Module: `datetime`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `from_timestamp(invalid)` | `Result[str, ValueError]` | **UNDEFINED** |
| `OverflowError` | `timedelta` arithmetic overflow | `Result[timedelta, OverflowError]` | **PANICS** (i64 overflow) |
| `ValueError` | Invalid format string | `Result[str, ValueError]` | **UNDEFINED** |

---

## Module: `random`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `randint(5, 3)` (a > b) | `Result[int, ValueError]` | **UNDEFINED** |
| `ValueError` | `randrange(0, 0)` | `Result[int, ValueError]` | N/A (no randrange) |
| `IndexError` | `choice([])` | `Option[T]` | N/A (no choice) |

---

## Module: `secrets`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `randbelow(0)` | `Result[int, ValueError]` | **UNDEFINED** |

---

## Module: `uuid`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `UUID(invalid_hex)` | `Result[UUID, ValueError]` | **UNDEFINED** |

---

## Module: `ipaddress`

| CPython Exception | Trigger | Sifr Should Return | Current Status |
| --- | --- | --- | --- |
| `ValueError` | `ip_address(invalid)` | `Result[IPv4Address, ValueError]` | N/A (no ip_address function) |
| `ValueError` | `ip_to_int(invalid)` | `Result[int, ValueError]` | **UNDEFINED** (may panic on parse) |

---

## Summary Statistics

| Status | Count |
| --- | --- |
| **PANICS** (`.unwrap()` in codegen) | ~40 exception paths |
| **UNDEFINED** (behavior not specified) | ~12 exception paths |
| **CORRECT** (returns `Result`/`Option`) | 2 exception paths (`env_get`, `re_find`) |
| **SILENT** (returns NaN/inf/0.0 instead of error) | ~5 exception paths |
| **Total CPython exception paths in implemented modules** | ~59 |
| **Percentage correctly handled** | ~3.4% |
