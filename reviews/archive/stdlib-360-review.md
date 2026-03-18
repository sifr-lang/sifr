# STDlib 360 Review - Module-by-Module Gap Analysis

**Review Date:** 2026-03-17
**Focus:** Functionality and CPython test gaps for ALL 45 implemented modules

---

## Summary

| Metric | Count |
|--------|-------|
| Sifr stdlib modules | 45 |
| CPython test modules | 392 |
| CPython-derived tests in Sifr | 67 (~17%) |
| Modules with CPython tests | 43/45 |
| Modules with stdlib-only tests | 2 (functools, html, operator, test) |

---

## Module-by-Module Gap Analysis

### 1. math

| Metric | Count |
|--------|-------|
| CPython items | 62 |
| Sifr intrinsics | 59 |
| Sifr pure Sifr wrappers | 13 |
| **Test coverage** | ✅ cpython_math*.sifr + stdlib_math |

**Functionality Gaps:**
- `pow(x, y, mod)` - 3-argument form not supported
- `factorial(-n)` returns 0 instead of ValueError
- `isclose()` returns False for negative tolerance instead of ValueError
- `comb(n, k)` / `perm(n, k)` return 0 for invalid k instead of ValueError
- `fmod()` returns positive remainder (CPython uses trunc-toward-zero)

---

### 2. json

| Metric | Count |
|--------|-------|
| CPython items | 8 |
| Sifr items | 22 (classes + functions) |
| **Test coverage** | ✅ cpython_json*.sifr + stdlib_json |

**Functionality Gaps:**
- ❌ No `JSONDecoder` class
- ❌ No `JSONEncoder` class
- ❌ No `load()` (file-based JSON reading)
- ❌ No `dump()` (file-based JSON writing)
- ⚠️ JsonValue uses list/dict wrapper not native JSON types

---

### 3. re (regex)

| Metric | Count |
|--------|-------|
| CPython items | 35 |
| Sifr items | 27 |
| **Test coverage** | ✅ cpython_re*.sifr + stdlib_re |

**Functionality Gaps:**
- ❌ No `Pattern` class
- ❌ No `Match` class
- ❌ No compiled regex caching (`re.compile()` returns nothing)
- ❌ No `finditer()` (lazy iterator)
- ❌ No `fullmatch()`
- ⚠️ Limited flag support

---

### 4. hashlib

| Metric | Count |
|--------|-------|
| CPython items | 20 |
| Sifr items | 31 |
| **Test coverage** | ✅ cpython_hashlib*.sifr |

**Functionality Gaps:**
- ❌ No SHA3 family (`sha3_224`, `sha3_256`, `sha3_384`, `sha3_512`)
- ❌ No SHAKE (`shake_128`, `shake_256`)
- ❌ No `pbkdf2_hmac()`
- ❌ No `scrypt()`
- ⚠️ `digest()` returns hex string not bytes

---

### 5. base64

| Metric | Count |
|--------|-------|
| CPython items | 18 |
| Sifr items | 20 |
| **Test coverage** | ✅ cpython_base64*.sifr |

**Functionality Gaps:**
- ❌ Missing `a85decode` / `a85encode` (Ascii85)
- ❌ Missing `b16decode` / `b16encode` (Base16)
- ❌ Missing `b85decode` / `b85encode` (Base85)

---

### 6. datetime

| Metric | Count |
|--------|-------|
| CPython items | 19 |
| Sifr items | 16 |
| **Test coverage** | ✅ cpython_datetime*.sifr + stdlib_datetime |

**Functionality Gaps:**
- ❌ No timezone support
- ❌ No `timedelta` class methods
- ❌ No `fold` support
- ⚠️ Returns strings not datetime objects

---

### 7. time

| Metric | Count |
|--------|-------|
| CPython items | 28 |
| Sifr items | 13 |
| **Test coverage** | ✅ cpython_time*.sifr + stdlib_time |

**Functionality Gaps:**
- ❌ No `CLOCK_*` constants
- ❌ No timezone handling
- ❌ No `tzset()`
- ❌ No `struct_time` class
- ⚠️ `strptime()` returns string not struct_time

---

### 8. os

| Metric | Count |
|--------|-------|
| CPython items | 206 |
| Sifr items | 20 |
| **Test coverage** | ✅ cpython_os*.sifr + stdlib_os |

**Functionality Gaps:**
- ❌ No process management (`fork`, `exec`, `wait`)
- ❌ No signal handling
- ❌ No file locking (`lockf`, `flock`)
- ❌ Missing many constants
- ❌ No `mmap`
- ❌ No symlink handling
- ❌ No `stat_result` class

---

### 9. sys

| Metric | Count |
|--------|-------|
| CPython items | 64 |
| Sifr items | 10 |
| **Test coverage** | ✅ cpython_sys*.sifr + stdlib_sys |

**Functionality Gaps:**
- ❌ No `settrace()` / `setprofile()`
- ❌ No recursion limit control
- ❌ No threading support
- ❌ No module introspection
- ❌ No audit hooks
- Only: argv, version, platform, exit, maxsize

---

### 10. random

| Metric | Count |
|--------|-------|
| CPython items | 32 |
| Sifr items | 19 |
| **Test coverage** | ✅ cpython_random*.sifr + stdlib_random |

**Functionality Gaps:**
- ❌ No seeding (`seed()` function)
- ❌ No `getstate()` / `setstate()`
- ❌ No `getrandbits()`
- ❌ Missing variate functions: `betavariate`, `binomialvariate`, `expovariate`, `gammavariate`, `lognormvariate`, `normalvariate`, `paretovariate`, `triangular`, `vonmisesvariate`, `weibullvariate`
- ❌ No `Random` class
- ❌ No `SystemRandom` class

---

### 11. collections

| Metric | Count |
|--------|-------|
| CPython items | 10 |
| Sifr items | 14 |
| **Test coverage** | ✅ cpython_collections*.sifr + stdlib_collections |

**Functionality Gaps:**
- ❌ No `namedtuple`
- ❌ No `deque`
- ❌ No `ChainMap`
- ⚠️ `Counter` uses JSON serialization workaround

---

### 12. itertools

| Metric | Count |
|--------|-------|
| CPython items | 20 |
| Sifr items | 22 |
| **Test coverage** | ✅ cpython_itertools*.sifr + stdlib_itertools |

**Functionality Gaps:**
- ❌ All functions return lists (not lazy iterators)
- ❌ No `tee()`
- ❌ No lazy `count()`
- ❌ No lazy `cycle()`
- ❌ No `groupby()`
- ⚠️ Performance implication for large datasets

---

### 13. statistics

| Metric | Count |
|--------|-------|
| CPython items | 20 |
| Sifr items | 19 |
| **Test coverage** | ✅ cpython_statistics*.sifr + stdlib_statistics |

**Functionality Gaps:**
- ❌ No `geometric_mean()`
- ❌ No `harmonic_mean()`
- ❌ No `multimode()`
- ❌ No `quantiles()`
- ❌ No `Fraction` / `Decimal` support

---

### 14. uuid

| Metric | Count |
|--------|-------|
| CPython items | 14 |
| Sifr items | 10 |
| **Test coverage** | ✅ cpython_uuid*.sifr + stdlib_uuid |

**Functionality Gaps:**
- ❌ Only `uuid4()` implemented
- ❌ No `uuid1()` (time-based)
- ❌ No `uuid3()`, `uuid5()` (name-based)
- ❌ No `uuid6()`, `uuid7()`, `uuid8()`
- ❌ No `getnode()` (MAC address)

---

### 15. logging

| Metric | Count |
|--------|-------|
| CPython items | 46 |
| Sifr items | 11 |
| **Test coverage** | ✅ cpython_logging*.sifr + stdlib_logging |

**Functionality Gaps:**
- ❌ No `Handler` classes (FileHandler, etc.)
- ❌ No `Formatter` class
- ❌ No Logger hierarchy
- Only: `set_global_level()`, `get_global_level()`

---

### 16. argparse

| Metric | Count |
|--------|-------|
| CPython items | 14 |
| Sifr items | 9 |
| **Test coverage** | ✅ cpython_argparse*.sifr + stdlib_argparse |

**Functionality Gaps:**
- ❌ No subparsers
- ❌ No nargs support
- ❌ No type/coerce
- ❌ No help formatting
- ❌ No `FileType`

---

### 17. ipaddress

| Metric | Count |
|--------|-------|
| CPython items | 10 |
| Sifr items | 17 |
| **Test coverage** | ✅ cpython_ipaddress*.sifr + stdlib_ipaddress |

**Functionality Gaps:**
- ⚠️ Only IPv4 supported (documented as intentional)
- ❌ No IPv6
- ❌ No `IPv4Interface` / `IPv4Network`
- ❌ No `IPv6Address` / `IPv6Network`

---

### 18. pathlib

| Metric | Count |
|--------|-------|
| CPython items | 6 |
| Sifr items | 20 |
| **Test coverage** | ✅ cpython_pathlib*.sifr + stdlib_pathlib |

**Functionality Gaps:**
- ❌ Limited Path methods
- ❌ No `PurePath` variants
- ❌ No Windows path support

---

### 19. subprocess

| Metric | Count |
|--------|-------|
| CPython items | 17 |
| Sifr items | 7 |
| **Test coverage** | ✅ cpython_subprocess*.sifr + stdlib_subprocess |

**Functionality Gaps:**
- ❌ No `Popen`
- ❌ No async execution
- ❌ No `shell=True`
- ❌ No pipe handling (stdin/stdout/stderr)

---

### 20. csv

| Metric | Count |
|--------|-------|
| CPython items | 25 |
| Sifr items | 31 |
| **Test coverage** | ✅ cpython_csv*.sifr + stdlib_csv |

**Functionality Gaps:**
- ❌ No `Sniffer`
- ❌ No `Dialect` classes
- ⚠️ Limited reader/writer customization

---

### 21. tempfile

| Metric | Count |
|--------|-------|
| CPython items | 12 |
| Sifr items | 11 |
| **Test coverage** | ✅ cpython_tempfile*.sifr + stdlib_tempfile |

**Functionality Gaps:**
- ❌ No `TemporaryFile`
- ❌ No `SpooledTemporaryFile`
- ❌ No `TemporaryDirectory`

---

### 22. zipfile

| Metric | Count |
|--------|-------|
| CPython items | 14 |
| Sifr items | 5 |
| **Test coverage** | ✅ cpython_zipfile*.sifr + stdlib_zipfile |

**Functionality Gaps:**
- ❌ Limited write support
- ❌ No `ZipInfo` customization
- ❌ No compression options

---

### 23. configparser

| Metric | Count |
|--------|-------|
| CPython items | 17 |
| Sifr items | 18 |
| **Test coverage** | ✅ cpython_configparser*.sifr |

**Functionality Gaps:**
- ❌ No interpolation
- ❌ No raw access
- ⚠️ Limited write support

---

### 24. functools

| Metric | Count |
|--------|-------|
| CPython items | 12 |
| Sifr items | 1 |
| **Test coverage** | ⚠️ stdlib_functools only (NO cpython test) |

**Functionality Gaps:**
- ❌ Only `reduce()` implemented
- ❌ No `lru_cache()`
- ❌ No `partial()`
- ❌ No `singledispatch()`
- ❌ No `wraps()`
- ❌ No `cmp_to_key()`
- ❌ No `total_ordering()`

---

### 25. html

| Metric | Count |
|--------|-------|
| CPython items | 6 |
| Sifr items | 4 |
| **Test coverage** | ⚠️ stdlib_html only (NO cpython test) |

**Functionality Gaps:**
- ❌ No `HTMLParser` class
- Only: `escape()`, `unescape()`

---

### 26. operator

| Metric | Count |
|--------|-------|
| CPython items | 51 |
| Sifr items | 19 |
| **Test coverage** | ⚠️ stdlib_operator only (NO cpython test) |

**Functionality Gaps:**
- ❌ Limited operator functions
- ❌ No `attrgetter`
- ❌ No `itemgetter`
- ❌ No `methodcaller`

---

### 27. test (sifr.test)

| Metric | Count |
|--------|-------|
| **Test coverage** | ⚠️ stdlib_test only (NO cpython test - by design) |

**Notes:**
- This is Sifr infrastructure, not CPython stdlib parity
- Provides: assert_eq, assert_ne, assert_true, assert_false, assert_almost_eq, assert_gt, assert_lt

---

### 28-45. Other Modules (Full Coverage)

These modules have good test coverage and minimal gaps:

| Module | Status | Notes |
|--------|--------|-------|
| **bisect** | ✅ Good | Full implementation |
| **bytes** | ✅ Good | encode_utf8, decode_utf8, hex conversion |
| **calendar** | ✅ Good | isleap, weekday, monthrange |
| **difflib** | ✅ Good | UnifiedDiff, HtmlDiff |
| **env** | ✅ Good | Environment variables |
| **fnmatch** | ✅ Good | fnmatch, filter, translate |
| **glob** | ✅ Good | glob, rglob |
| **graphlib** | ✅ Good | TopologicalSorter |
| **gzip** | ✅ Good | compress, decompress |
| **heapq** | ✅ Good | heappush, heappop, heapify, nlargest, nsmallest |
| **io** | ✅ Good | FileHandle class |
| **secrets** | ✅ Good | token_bytes, token_hex, etc. |
| **shutil** | ✅ Good | copy, move, rmtree |
| **string** | ✅ Good | constants, Formatter, Template |
| **textwrap** | ✅ Good | wrap, fill, indent, dedent |
| **timeit** | ✅ Good | timeit, repeat, Timer |
| **tomllib** | ✅ Good | load, loads |

---

## Test Coverage Summary

| Status | Count |
|--------|-------|
| Has both CPython + stdlib tests | 40 |
| Has stdlib tests only | 4 (functools, html, operator, test) |
| Total with tests | 45/45 |

---

## Priority Gap List

### HIGH PRIORITY

1. **functools** - Only 1/12 items (8% complete)
2. **hashlib** - Missing SHA3, pbkdf2, scrypt
3. **re** - No Pattern/Match objects
4. **random** - No seeding, state management
5. **os** - Missing 186+ functions

### MEDIUM PRIORITY

6. **itertools** - Not lazy (performance)
7. **json** - No file I/O
8. **logging** - Only 11/46 items
9. **subprocess** - No Popen
10. **uuid** - Only uuid4

### LOW PRIORITY

11. **base64** - Missing Ascii85, Base16, Base85
12. **datetime** - No timezone
13. **time** - No struct_time
14. **statistics** - Missing 5 functions

---

## What IS Working

- ✅ All 45 modules compile and type-check
- ✅ Error handling follows safety contract (Result types)
- ✅ 67 CPython-derived test files
- ✅ Core functionality for most modules
- ✅ Good coverage for: math, json, collections, csv, datetime
