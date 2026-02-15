---
name: Stdlib Parity Audit
overview: Create a comprehensive audit of Sifr's standard library against CPython's stdlib, module by module, with .sifr test files organized in subdirectories under audit/. Each test exercises specific functions/behaviors from the corresponding CPython module, identifying gaps, mismatches, and missing features.
todos:
  - id: math-audit
    content: Create audit/stdlib_math/ with 15 test files covering all 57 CPython math functions + 5 constants vs Sifr's 12 functions + 2 constants
    status: pending
  - id: json-audit
    content: Create audit/stdlib_json/ with 6 test files covering json.dumps, json.loads, roundtrip, and type handling
    status: pending
  - id: re-audit
    content: Create audit/stdlib_re/ with 8 test files covering match, find, replace, findall, split, compile, flags, groups
    status: pending
  - id: string-audit
    content: Create audit/stdlib_string/ with 12 test files covering all str methods + string module constants
    status: pending
  - id: collections-audit
    content: Create audit/stdlib_collections/ with 10 test files covering Set, Counter, defaultdict, deque, OrderedDict, namedtuple, ChainMap
    status: pending
  - id: os-audit
    content: Create audit/stdlib_os/ with 10 test files covering run_command, get_args, getcwd, listdir, path ops, mkdir, rename, walk
    status: pending
  - id: io-audit
    content: Create audit/stdlib_io/ with 8 test files covering read_text, write_text, exists, read_lines, append, binary, file objects
    status: pending
  - id: time-audit
    content: Create audit/stdlib_time/ with 6 test files covering time_now, sleep, time_format, monotonic, perf_counter, strptime
    status: pending
  - id: random-audit
    content: Create audit/stdlib_random/ with 8 test files covering random_int, random_float, random_choice, shuffle, sample, seed, uniform
    status: pending
  - id: hashlib-audit
    content: Create audit/stdlib_hashlib/ with 6 test files covering sha256, md5, sha1, sha512, blake2, hmac
    status: pending
  - id: base64-audit
    content: Create audit/stdlib_base64/ with 5 test files covering b64 encode/decode, urlsafe, b32, b16
    status: pending
  - id: functools-audit
    content: Create audit/stdlib_functools/ with 4 test files for reduce, partial, lru_cache, wraps
    status: pending
  - id: itertools-audit
    content: Create audit/stdlib_itertools/ with 4 test files for chain, combinations, permutations, product, groupby
    status: pending
  - id: bisect-audit
    content: Create audit/stdlib_bisect/ with 3 test files for bisect_left, bisect_right, insort
    status: pending
  - id: heapq-audit
    content: Create audit/stdlib_heapq/ with 3 test files for heappush, heappop, heapify, nlargest, nsmallest
    status: pending
  - id: statistics-audit
    content: Create audit/stdlib_statistics/ with 3 test files for mean, median, stdev, variance
    status: pending
  - id: textwrap-audit
    content: Create audit/stdlib_textwrap/ with 3 test files for wrap, fill, dedent, indent
    status: pending
  - id: copy-audit
    content: Create audit/stdlib_copy/ with 3 test files for copy, deepcopy
    status: pending
  - id: csv-audit
    content: Create audit/stdlib_csv/ with 3 test files for reader, writer, DictReader
    status: pending
  - id: uuid-audit
    content: Create audit/stdlib_uuid/ with 2 test files for uuid4, uuid1
    status: pending
  - id: datetime-audit
    content: Create audit/stdlib_datetime/ with 4 test files for date, datetime, timedelta, timezone
    status: pending
  - id: pathlib-audit
    content: Create audit/stdlib_pathlib/ with 3 test files for Path operations
    status: pending
  - id: struct-audit
    content: Create audit/stdlib_struct/ with 2 test files for pack, unpack, calcsize
    status: pending
  - id: builtins-audit
    content: Create audit/stdlib_builtins/ with 10 test files covering all Python builtins vs Sifr builtins (print, len, range, int, str, float, bool, abs, round, hash, min, max, sum, sorted, reversed, enumerate, zip, any, all, map, filter, isinstance, type, input, open, chr, ord, hex, oct, bin, id, dir, vars, getattr, setattr, hasattr, delattr, callable, iter, next, super, property, classmethod, staticmethod)
    status: pending
  - id: typing-audit
    content: Create audit/stdlib_typing/ with 4 test files for Union, Optional, List, Dict, Tuple, Callable, TypeVar, Generic
    status: pending
  - id: remaining-modules
    content: Create audit directories for dataclasses, enum, abc, contextlib, subprocess (2-3 test files each)
    status: pending
  - id: parity-reports
    content: Create PARITY_REPORT.md in each audit directory with CPython member count, Sifr coverage, gap table, and priority
    status: pending
  - id: master-report
    content: Create audit/STDLIB_PARITY_MASTER_REPORT.md summarizing all modules, total coverage percentage, and prioritized gap list
    status: pending
isProject: false
---

# Sifr Stdlib Parity Audit Against CPython

## Current State

Sifr has **13 stdlib modules** in `[crates/sifr_hir/src/stdlib.rs](crates/sifr_hir/src/stdlib.rs)`:

- `sifr.io` (4 functions), `sifr.json` (2), `sifr.env` (2), `sifr.os` (2), `sifr.math` (12 functions + 2 constants), `sifr.test` (4), `sifr.collections` (14), `sifr.bytes` (4), `sifr.time` (3), `sifr.random` (3), `sifr.re` (3), `sifr.hash` (2), `sifr.encoding` (2)

Sifr has **~30 builtin functions**: `print`, `len`, `range`, `isinstance`, `str`, `int`, `float`, `bool`, `pow`, `abs`, `hash`, `round`, `repr`, `min`, `max`, `sum`, `sorted`, `reversed`, `enumerate`, `zip`, `any`, `all`, `map`, `filter`, `reveal_type`

The existing `audit/stdlib/` has 10 test files with 30% pass rate. The existing audit runner scripts are at `[audit/run_audit.sh](audit/run_audit.sh)` (full compile+run) and `[audit/run_audit_fast.sh](audit/run_audit_fast.sh)` (check only).

CPython (from `/Users/yaseralnajjar/work/sifr/cpython`) has **289 stdlib modules**. We will audit the most important ones relevant to Sifr's scope.

## Audit Structure

Create separate directories under `audit/` for each CPython stdlib module category. Each directory contains numbered `.sifr` test files that exercise specific behaviors, plus a `PARITY_REPORT.md` summarizing coverage gaps.

```
audit/
  stdlib_math/           # vs CPython Lib math (C module)
  stdlib_json/           # vs CPython Lib/json/
  stdlib_re/             # vs CPython Lib/re/
  stdlib_string/         # vs CPython Lib/string.py + str builtins
  stdlib_collections/    # vs CPython Lib/collections/
  stdlib_os/             # vs CPython Lib/os.py
  stdlib_io/             # vs CPython Lib/io.py
  stdlib_time/           # vs CPython Lib/time (C module)
  stdlib_random/         # vs CPython Lib/random.py
  stdlib_hashlib/        # vs CPython Lib/hashlib.py
  stdlib_base64/         # vs CPython Lib/base64.py
  stdlib_functools/      # vs CPython Lib/functools.py
  stdlib_itertools/      # vs CPython itertools (C module)
  stdlib_bisect/         # vs CPython Lib/bisect.py
  stdlib_heapq/          # vs CPython Lib/heapq.py
  stdlib_statistics/     # vs CPython Lib/statistics.py
  stdlib_textwrap/       # vs CPython Lib/textwrap.py
  stdlib_copy/           # vs CPython Lib/copy.py
  stdlib_csv/            # vs CPython Lib/csv.py
  stdlib_uuid/           # vs CPython Lib/uuid.py
  stdlib_datetime/       # vs CPython Lib/datetime.py
  stdlib_pathlib/        # vs CPython Lib/pathlib/
  stdlib_struct/         # vs CPython struct (C module)
  stdlib_builtins/       # vs CPython builtins (built-in functions)
  stdlib_typing/         # vs CPython Lib/typing.py
  stdlib_dataclasses/    # vs CPython Lib/dataclasses.py
  stdlib_enum/           # vs CPython Lib/enum.py
  stdlib_abc/            # vs CPython Lib/abc.py
  stdlib_contextlib/     # vs CPython Lib/contextlib.py
  stdlib_subprocess/     # vs CPython Lib/subprocess.py
```

## Test File Design

Each `.sifr` file follows the existing convention (see `[audit/borrowing/01_copy_types_reuse.sifr](audit/borrowing/01_copy_types_reuse.sifr)`):

- Header comment with test number and description
- Comment noting the CPython equivalent being tested
- `def main():` entry point
- Tests that exercise specific API surface

For modules Sifr **has** (math, json, re, etc.): test every function in the sifr module and note which CPython equivalents are missing.

For modules Sifr **does not have** (functools, itertools, datetime, etc.): write tests using the `from sifr.X import Y` pattern that will fail, documenting what would need to be added.

## Detailed Test Plans Per Module

### 1. `audit/stdlib_math/` -- CPython `math` (57 functions, 5 constants)

Sifr has: sqrt, floor, ceil, abs_val, log, sin, cos, tan, pow_val, min_val, max_val, round_val, pi, e

Tests (15+ files):

- `01_basic_arithmetic.sifr` -- sqrt, floor, ceil (HAVE)
- `02_trig_functions.sifr` -- sin, cos, tan (HAVE)
- `03_inverse_trig.sifr` -- asin, acos, atan, atan2 (MISSING)
- `04_hyperbolic.sifr` -- sinh, cosh, tanh (MISSING)
- `05_logarithms.sifr` -- log, log2, log10, log1p (PARTIAL: only log)
- `06_exponential.sifr` -- exp, exp2, expm1 (MISSING)
- `07_power_abs.sifr` -- pow_val, abs_val (HAVE)
- `08_rounding.sifr` -- round_val, trunc (PARTIAL)
- `09_constants.sifr` -- pi, e, tau, inf, nan (PARTIAL: pi, e only)
- `10_combinatorics.sifr` -- factorial, comb, perm (MISSING)
- `11_gcd_lcm.sifr` -- gcd, lcm (MISSING)
- `12_special_values.sifr` -- isnan, isinf, isfinite (MISSING)
- `13_fmod_remainder.sifr` -- fmod, remainder, fabs (MISSING)
- `14_hypot_dist.sifr` -- hypot, dist (MISSING)
- `15_sum_prod.sifr` -- fsum, prod, sumprod (MISSING)

### 2. `audit/stdlib_json/` -- CPython `json` (4 core functions)

Sifr has: json_loads, json_dumps

Tests (6 files):

- `01_dumps_string.sifr` -- json_dumps with str
- `02_dumps_int.sifr` -- json_dumps with int (tests Any param)
- `03_dumps_float_bool.sifr` -- json_dumps with float, bool
- `04_loads_basic.sifr` -- json_loads basic parsing
- `05_dumps_list_dict.sifr` -- json_dumps with collections (MISSING: needs dict/list serialization)
- `06_roundtrip.sifr` -- dumps then loads roundtrip

### 3. `audit/stdlib_re/` -- CPython `re` (10 core functions)

Sifr has: re_match, re_find, re_replace

Tests (8 files):

- `01_match_basic.sifr` -- re_match bool result
- `02_find_basic.sifr` -- re_find returns str|None
- `03_replace_basic.sifr` -- re_replace substitution
- `04_findall.sifr` -- findall returning list[str] (MISSING)
- `05_split.sifr` -- split by pattern (MISSING)
- `06_compile.sifr` -- compiled pattern object (MISSING)
- `07_flags.sifr` -- IGNORECASE, MULTILINE etc (MISSING)
- `08_groups.sifr` -- capture groups (MISSING)

### 4. `audit/stdlib_string/` -- CPython `string` module + str builtins

Sifr has: str methods (upper, lower, find, count, startswith, endswith, replace, strip, split, isdigit, isalpha, isalnum, len)

Tests (12 files):

- `01_case_methods.sifr` -- upper, lower, title, capitalize, swapcase
- `02_search_methods.sifr` -- find, rfind, index, rindex, count
- `03_test_methods.sifr` -- isdigit, isalpha, isalnum, isspace, isupper, islower
- `04_transform_methods.sifr` -- replace, strip, lstrip, rstrip
- `05_split_join.sifr` -- split, rsplit, splitlines, join
- `06_format_methods.sifr` -- format, f-strings, center, ljust, rjust, zfill
- `07_encode_decode.sifr` -- encode, decode (MISSING)
- `08_string_constants.sifr` -- string.ascii_letters, digits, etc (MISSING module)
- `09_startswith_endswith.sifr` -- startswith, endswith with tuples
- `10_maketrans_translate.sifr` -- maketrans, translate (MISSING)
- `11_partition.sifr` -- partition, rpartition (MISSING)
- `12_expandtabs_removeprefix.sifr` -- expandtabs, removeprefix, removesuffix (MISSING)

### 5. `audit/stdlib_collections/` -- CPython `collections` (9 types)

Sifr has: set ops (new_set, set_add, etc.), counter ops, defaultdict ops

Tests (10 files):

- `01_set_basic.sifr` -- builtin Set type
- `02_set_operations.sifr` -- union, intersection, difference
- `03_counter_basic.sifr` -- counter_from_list, counter_get
- `04_counter_most_common.sifr` -- counter_most_common
- `05_defaultdict_basic.sifr` -- defaultdict_new, get, set
- `06_deque.sifr` -- deque operations (MISSING)
- `07_ordered_dict.sifr` -- OrderedDict (MISSING)
- `08_namedtuple.sifr` -- namedtuple (MISSING)
- `09_chainmap.sifr` -- ChainMap (MISSING)
- `10_user_types.sifr` -- UserDict, UserList, UserString (MISSING)

### 6. `audit/stdlib_os/` -- CPython `os` (181 functions)

Sifr has: run_command, get_args

Tests (10 files):

- `01_run_command.sifr` -- run_command basic
- `02_get_args.sifr` -- get_args
- `03_getcwd.sifr` -- getcwd (MISSING)
- `04_listdir.sifr` -- listdir (MISSING)
- `05_path_exists.sifr` -- os.path.exists, isfile, isdir (MISSING)
- `06_path_join.sifr` -- os.path.join, split, basename, dirname (MISSING)
- `07_mkdir_rmdir.sifr` -- mkdir, makedirs, rmdir (MISSING)
- `08_rename_remove.sifr` -- rename, remove, unlink (MISSING)
- `09_environ.sifr` -- environ access (covered by sifr.env)
- `10_walk.sifr` -- os.walk (MISSING)

### 7. `audit/stdlib_io/` -- CPython `io`

Sifr has: read_text, write_text, exists, read_lines

Tests (8 files):

- `01_write_text.sifr` -- write_text basic
- `02_read_text.sifr` -- read_text basic
- `03_exists.sifr` -- exists check
- `04_read_lines.sifr` -- read_lines
- `05_append_text.sifr` -- append mode (MISSING)
- `06_binary_io.sifr` -- binary read/write (MISSING)
- `07_file_object.sifr` -- file object with context manager (MISSING)
- `08_stdin_stdout.sifr` -- stdin/stdout/stderr (MISSING)

### 8. `audit/stdlib_time/` -- CPython `time` (26 functions)

Sifr has: time_now, sleep, time_format

Tests (6 files):

- `01_time_now.sifr` -- time_now epoch
- `02_sleep.sifr` -- sleep duration
- `03_time_format.sifr` -- time_format strftime
- `04_monotonic.sifr` -- monotonic clock (MISSING)
- `05_perf_counter.sifr` -- perf_counter (MISSING)
- `06_strptime.sifr` -- strptime parsing (MISSING)

### 9. `audit/stdlib_random/` -- CPython `random` (27 functions)

Sifr has: random_int, random_float, random_choice

Tests (8 files):

- `01_random_int.sifr` -- random_int range
- `02_random_float.sifr` -- random_float 0..1
- `03_random_choice.sifr` -- random_choice from list
- `04_shuffle.sifr` -- shuffle (MISSING)
- `05_sample.sifr` -- sample (MISSING)
- `06_seed.sifr` -- seed for reproducibility (MISSING)
- `07_uniform.sifr` -- uniform distribution (MISSING)
- `08_randrange.sifr` -- randrange (MISSING)

### 10. `audit/stdlib_hashlib/` -- CPython `hashlib` (18 functions)

Sifr has: sha256, md5 (in sifr.hash)

Tests (6 files):

- `01_sha256.sifr` -- sha256 hex digest
- `02_md5.sifr` -- md5 hex digest
- `03_sha1.sifr` -- sha1 (MISSING)
- `04_sha512.sifr` -- sha512 (MISSING)
- `05_blake2.sifr` -- blake2b/blake2s (MISSING)
- `06_hmac.sifr` -- HMAC (MISSING, separate module)

### 11. `audit/stdlib_base64/` -- CPython `base64` (23 functions)

Sifr has: base64_encode, base64_decode (in sifr.encoding)

Tests (5 files):

- `01_b64_encode.sifr` -- base64_encode
- `02_b64_decode.sifr` -- base64_decode
- `03_urlsafe.sifr` -- urlsafe variants (MISSING)
- `04_b32.sifr` -- base32 (MISSING)
- `05_b16.sifr` -- base16/hex (MISSING)

### 12-30. Modules Sifr Does NOT Have Yet

For each of these, create 2-4 test files that document what the CPython module provides and what Sifr would need:

- `audit/stdlib_functools/` -- reduce, partial, lru_cache, wraps
- `audit/stdlib_itertools/` -- chain, combinations, permutations, product, groupby, accumulate
- `audit/stdlib_bisect/` -- bisect_left, bisect_right, insort
- `audit/stdlib_heapq/` -- heappush, heappop, heapify, nlargest, nsmallest
- `audit/stdlib_statistics/` -- mean, median, stdev, variance
- `audit/stdlib_textwrap/` -- wrap, fill, dedent, indent
- `audit/stdlib_copy/` -- copy, deepcopy
- `audit/stdlib_csv/` -- reader, writer, DictReader
- `audit/stdlib_uuid/` -- uuid4, uuid1
- `audit/stdlib_datetime/` -- date, datetime, timedelta, timezone
- `audit/stdlib_pathlib/` -- Path operations
- `audit/stdlib_struct/` -- pack, unpack, calcsize
- `audit/stdlib_builtins/` -- all Python builtins vs Sifr builtins
- `audit/stdlib_typing/` -- Union, Optional, List, Dict, Tuple, Callable, TypeVar
- `audit/stdlib_dataclasses/` -- @dataclass decorator
- `audit/stdlib_enum/` -- Enum, IntEnum
- `audit/stdlib_abc/` -- ABC, abstractmethod
- `audit/stdlib_contextlib/` -- contextmanager, suppress
- `audit/stdlib_subprocess/` -- run, Popen, PIPE

## Each Directory Also Gets

- A `PARITY_REPORT.md` with:
  - CPython module member count (from CPython source)
  - Sifr equivalent functions available
  - Gap analysis table (CPython function -> Sifr status: HAVE/MISSING/PARTIAL)
  - Priority ranking for missing features

## Execution

- Use `sifr check` via `[audit/run_audit_fast.sh](audit/run_audit_fast.sh)` to validate each directory
- Tests that exercise existing Sifr functions should pass `sifr check`
- Tests that exercise missing functions will fail, documenting the gap
- Each test file has a comment header indicating expected status: `# EXPECT: PASS` or `# EXPECT: FAIL (missing X)`

## Total Scope

~200+ test files across 30 directories, providing a comprehensive module-by-module parity map between CPython's stdlib and Sifr's stdlib.