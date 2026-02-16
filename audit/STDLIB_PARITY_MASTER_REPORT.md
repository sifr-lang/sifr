# Sifr Stdlib Parity Report

Generated: 2026-02-16 (updated after milestone_stdlib_classes)

## Summary

| Metric | Value |
| --- | --- |
| Total Sifr stdlib modules | 37 |
| CPython top-20 modules covered | 18/20 (90%) |
| Average function coverage | ~52% |
| Stdlib modules with class-based APIs | 1 (`collections.Counter`) |
| E2E pass tests (stdlib) | 46 |
| E2E fail tests (stdlib) | 7 |

## Changes in milestone_stdlib_classes

| Change | Details |
| --- | --- |
| First stdlib class | `Counter` class in `lib/sifr/collections.sifr` — proves full class-in-stdlib pipeline |
| New intrinsics | `_sifr.collections.counter_total`, `counter_values`, `counter_keys`, `counter_items`, `counter_increment` |
| New stdlib class methods | `Counter.__init__`, `get`, `most_common`, `total`, `values`, `keys`, `items`, `increment` |
| New factory function | `from_list(items: list[str]) -> Counter` |
| Pipeline fixes | Class method signatures exported via `StdlibCode.func_signatures` for correct borrow convention at call sites |
| Codegen fix | `counter_get` uses `.as_str()` pattern to handle both owned and borrowed string arguments |
| Format fix | `counter_most_common` and `counter_items` output corrected to `["key",count]` format |
| New E2E pass tests | `stdlib_collections_counter`, `stdlib_collections_counter_mutate` |
| New E2E fail tests | `stdlib_counter_wrong_type` |

## Changes in milestone_stdlib_polish

| Change | Details |
| --- | --- |
| API renames | `glob.glob_match` → `glob.glob`, `shutil.copy_file` → `shutil.copy` |
| New intrinsics | `_sifr.time.perf_counter`, `_sifr.time.monotonic`, `_sifr.fs.copy_file`, `_sifr.fs.walk_dir`, `_sifr.fs.rmdir_all` |
| New stdlib functions | `sifr.time.perf_counter`, `sifr.time.monotonic`, `sifr.shutil.rmtree`, `sifr.tomllib.load` |
| Rewritten module | `sifr.timeit` — full CPython API: `default_timer`, `timeit(stmt, number)`, `repeat(stmt, count, number)` using `Callable` type |
| New E2E pass tests | `stdlib_glob`, `stdlib_shutil`, `stdlib_tempfile` |
| New E2E fail tests | `stdlib_invalid_module`, `stdlib_wrong_type`, `stdlib_missing_function`, `stdlib_intrinsic_direct_v2`, `stdlib_wrong_arg_count` |
| Fixes | Stale `lower.rs` comment updated, `has_pure_sifr_code` includes classes |

## Module-by-Module Parity

### Tier 1: Migrated Modules (from milestone_stdlib_migration)

| Sifr Module | CPython Equivalent | Functions | Coverage |
| --- | --- | --- | --- |
| sifr.math | math | 29 functions + 5 constants | ~85% |
| sifr.os | os / os.path | 13 functions | ~40% |
| sifr.io | io / builtins (open) | 5 functions | ~35% |
| sifr.re | re | 5 functions | ~45% |
| sifr.json | json | 2 functions | ~60% |
| sifr.time | time | 5 functions (added perf_counter, monotonic) | ~45% |
| sifr.hashlib | hashlib | 2 functions | ~40% |
| sifr.base64 | base64 | 2 functions | ~50% |
| sifr.random | random | 4 functions | ~35% |
| sifr.bytes | bytes/bytearray | 4 functions | ~30% |
| sifr.collections | collections | 14 functions + Counter class (8 methods) | ~40% |
| sifr.env | os.environ | 2 functions | ~50% |
| sifr.test | unittest/assert | 4 functions | N/A (custom) |

### Tier 2: Expansion Modules (from milestone_stdlib_expansion)

| Sifr Module | CPython Equivalent | Functions | Coverage |
| --- | --- | --- | --- |
| sifr.string | string | 8 constants | ~60% |
| sifr.statistics | statistics | 5 functions | ~50% |
| sifr.bisect | bisect | 3 functions | ~75% |
| sifr.functools | functools | 2 functions | ~15% |
| sifr.secrets | secrets | 2 functions | ~40% |
| sifr.heapq | heapq | 6 functions | ~60% |
| sifr.itertools | itertools | 6 functions | ~20% |
| sifr.textwrap | textwrap | 4 functions | ~60% |
| sifr.csv | csv | 4 functions | ~30% |
| sifr.argparse | argparse | 3 functions | ~15% |
| sifr.fnmatch | fnmatch | 2 functions | ~50% |
| sifr.glob | glob | 1 function (renamed to `glob`) | ~20% |
| sifr.shutil | shutil | 3 functions (copy, move_file, rmtree) | ~20% |
| sifr.tempfile | tempfile | 3 functions | ~25% |

### Tier 3: Parity Modules (from milestone_stdlib_parity)

| Sifr Module | CPython Equivalent | Functions | Coverage |
| --- | --- | --- | --- |
| sifr.graphlib | graphlib | 1 function | ~30% |
| sifr.uuid | uuid | 1 function | ~20% |
| sifr.platform | platform | 2 functions | ~20% |
| sifr.pathlib | pathlib | 4 functions | ~15% |
| sifr.logging | logging | 4 functions | ~15% |
| sifr.difflib | difflib | 3 functions | ~20% |
| sifr.ipaddress | ipaddress | 4 functions | ~25% |
| sifr.timeit | timeit | 3 functions (default_timer, timeit, repeat) | ~60% |
| sifr.tomllib | tomllib | 2 functions (loads, load) | ~50% |
| sifr.datetime | datetime | 3 functions | ~15% |

## CPython Top-20 Modules Not Yet Covered

| CPython Module | Reason | Priority |
| --- | --- | --- |
| sys | Partially covered via sifr.os/sifr.env | Medium |
| typing | Built into Sifr's type system | N/A |

## Key Gaps and Recommendations

### Blocked by Language Features
- **Class-based APIs (partially unblocked)**: `collections.Counter` proves the stdlib class pipeline works end-to-end. Classes that don't need `Callable` fields (Path, Logger, Match, TopologicalSorter) are now unblocked. Classes needing `Callable`-as-struct-field (`argparse.ArgumentParser`, `collections.defaultdict`, `timeit.Timer`) still require the `impl Fn` → `Box<dyn Fn>` codegen fix.
- **Complex generics**: Generic container types beyond `list[T]` and `dict[K,V]`
- **Context managers**: `open()/File` requires `with` statement support
- **CPU clocks**: `time.process_time()` / `time.thread_time()` require `libc` crate

### Achievable with Current Features
- **More math functions**: `factorial`, `gcd`, `lcm` can be pure Sifr
- **More string functions**: `capwords`, `Template` patterns
- **Expand csv**: Quoted field support
- **Expand datetime**: More formatting options
- **Safety contract**: Result/Option for fallible operations (separate milestone)

### Architecture Strengths
- Three-tier hybrid model works well: intrinsics for OS/crypto, pure Sifr for algorithms
- Two-phase compilation pipeline is stable and handles inter-module dependencies
- Transitive dependency tracking ensures correct Cargo.toml generation
- Pure Sifr modules compile to efficient Rust code
- `Callable` type support enables higher-order stdlib functions (timeit, functools)
- 37 modules covering 90% of CPython's top-20

## Test Coverage

| Category | Tests |
| --- | --- |
| E2E pass tests (stdlib) | 46 |
| E2E fail tests (stdlib) | 7 |
| Unit tests | 300+ |
| Total | 350+ |

All tests pass as of this report.
