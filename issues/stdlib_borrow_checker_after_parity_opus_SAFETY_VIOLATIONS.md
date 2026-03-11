# Safety Violations: `.unwrap()` and Panic Paths in Codegen

Generated: 2026-02-17

## Overview

The architecture's Safety Philosophy states:

> **No panics in user code.** Sifr programs never panic during normal execution. Every operation that can fail returns `Result[T, E]` or `Option[T]`, forcing the caller to handle the failure case at compile time.

> **If it compiles, it works.** A successfully compiled program will not crash at runtime under normal conditions.

This report documents every codegen path that violates this guarantee by emitting `.unwrap()` or direct panicking Rust code.

---

## Category 1: File System Operations (Critical)

**Source:** `crates/sifr_codegen/src/lib.rs` — intrinsic emission

Every file system intrinsic generates `.unwrap()`, meaning any I/O failure (missing file, permission denied, disk full) causes a runtime panic.

| Intrinsic | Codegen Output | Line | Panic Trigger |
| --- | --- | --- | --- |
| `read_text(path)` | `std::fs::read_to_string(...).unwrap()` | ~5093 | File not found, permission denied |
| `write_text(path, content)` | `std::fs::write(...).unwrap()` | ~5100 | Permission denied, disk full |
| `read_lines(path)` | `std::fs::read_to_string(...).unwrap().lines()...` | ~5110 | File not found |
| `append_text(path, content)` | `OpenOptions::new().append(true)...open(...).unwrap(); write!(...).unwrap()` | ~5113-5117 | File not found, permission denied |
| `mkdir(path)` | `std::fs::create_dir_all(...).unwrap()` | ~5130 | Permission denied |
| `rmdir(path)` | `std::fs::remove_dir(...).unwrap()` | ~5135 | Dir not found, not empty |
| `remove_file(path)` | `std::fs::remove_file(...).unwrap()` | ~5140 | File not found |
| `rename(old, new)` | `std::fs::rename(...).unwrap()` | ~5146 | File not found, cross-device |
| `getcwd()` | `std::env::current_dir().unwrap().to_string_lossy()` | ~5120 | CWD deleted |
| `listdir(path)` | `std::fs::read_dir(...).unwrap()...` | ~5125 | Dir not found |

**Affected stdlib modules:** `sifr.io`, `sifr.pathlib`, `sifr.shutil`, `sifr.os`, `sifr.tempfile`, `sifr.tomllib`

**Architecture requirement:** These should all return `Result[T, IOError]`. The intrinsic type signatures in `stdlib.rs` declare plain return types (e.g., `read_text -> str` not `read_text -> Result[str, IOError]`), so the violation starts at the type signature level.

---

## Category 2: Collection Method Panics (High)

| Method | Codegen Output | Line | Panic Trigger |
| --- | --- | --- | --- |
| `list.remove(val)` | `.iter().position(\|x\| *x == val).unwrap(); list.remove(pos)` | ~3544-3553 | Value not in list |
| `list.index(val)` | `.iter().position(\|x\| *x == val).unwrap() as i64` | ~3556-3562 | Value not in list |
| `set.pop()` | `.iter().next().cloned().unwrap()` | ~3735 | Empty set |

**Architecture requirement:**
- `list.remove(val)` should return `Result[None, ValueError]` (CPython raises `ValueError`)
- `list.index(val)` should return `Option[int]` or `Result[int, ValueError]` (CPython raises `ValueError`)
- `set.pop()` should return `Option[T]` (CPython raises `KeyError`)

---

## Category 3: Built-in Function Panics (High)

| Function | Codegen Output | Line | Panic Trigger |
| --- | --- | --- | --- |
| `min(list)` (int) | `*list.iter().min().unwrap()` | ~4319 | Empty list |
| `min(list)` (float) | `.iter().cloned().reduce(f64::min).unwrap()` | ~4314 | Empty list |
| `max(list)` (int) | `*list.iter().max().unwrap()` | ~4343 | Empty list |
| `max(list)` (float) | `.iter().cloned().reduce(f64::max).unwrap()` | ~4338 | Empty list |
| `sorted(list)` (float) | `.sort_by(\|a, b\| a.partial_cmp(b).unwrap())` | ~4362 | NaN comparison |

**Architecture requirement:**
- `min([])`/`max([])` should return `Result[T, ValueError]` (CPython raises `ValueError`)
- `sorted()` with NaN: architecture acknowledges `float` doesn't implement `Comparable` (Contract #12), but codegen uses `.unwrap()` on `partial_cmp` which panics on NaN

---

## Category 4: Index Assignment Bypass (High)

| Operation | Codegen Output | Line | Panic Trigger |
| --- | --- | --- | --- |
| `list[i] = val` | `list[i as usize] = val` | ~2792-2797 | Index out of bounds |

**Architecture requirement (Contract #7):**
> All indexable types use safe indexing. `x[i]` returns `Option[T]`, never panics. This is enforced uniformly across the language.

The contract says "uniformly" but only the read path (`x[i]`) is safe. The write path (`x[i] = val`) uses direct Rust indexing which panics on out-of-bounds. This is an asymmetric violation.

**Stdlib usage:** `graphlib.sifr` line 32: `visited[node] = 1` — would panic if `node >= len(visited)`.

---

## Category 5: Option Unwrap in Narrowing (Medium)

| Pattern | Codegen Output | Line | Panic Trigger |
| --- | --- | --- | --- |
| Return narrowed Option value | `.unwrap()` | ~2272 | Logic error in narrowing |
| Option arithmetic | `.unwrap()` on both sides | ~3979, 3986 | None value in arithmetic |
| Option `.len()` | `.as_ref().unwrap().len()` | ~3761 | None value |
| Option indexing | `.as_ref().unwrap()...get(idx)` | ~4609 | None value |

These are less severe because they occur after narrowing checks (`if val is not None:`), but the codegen emits raw `.unwrap()` rather than propagating the `Option`. If the narrowing analysis has a bug, these become runtime panics.

---

## Category 6: Sentinel Value Returns (Medium)

These don't panic but violate the architecture's error-handling contract by returning valid values instead of `Result`/`Option`.

| Function | Return on Error | Architecture Requirement | CPython Behavior |
| --- | --- | --- | --- |
| `statistics.mean([])` | Division by zero (panic via Rust) | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.median([])` | `0.0` | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.variance([])` | Division by zero (panic) | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.harmonic_mean([])` | `0.0` | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.geometric_mean([])` | `0.0` | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.median_low([])` | `0.0` | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.median_high([])` | `0.0` | `Result[float, StatisticsError]` | Raises `StatisticsError` |
| `statistics.mode([])` | `0` | `Result[int, StatisticsError]` | Raises `StatisticsError` |
| `heapq.heappop([])` | `0` | `Option[int]` | Raises `IndexError` |
| `heapq.heapreplace([], x)` | `x` | `Option[int]` or `Result` | Raises `IndexError` |

---

## Total Panic Surface

| Category | Panic Paths | Severity |
| --- | --- | --- |
| File system I/O | 10 | Critical |
| Collection methods | 3 | High |
| Built-in functions | 5 | High |
| Index assignment | 1 | High |
| Option unwrap (post-narrowing) | 4+ | Medium |
| Sentinel returns (silent wrong answer) | 10 | Medium |
| **Total** | **33+** | — |

Every one of these paths represents a compiled sifr program that can crash at runtime, contradicting the "if it compiles, it works" guarantee.
