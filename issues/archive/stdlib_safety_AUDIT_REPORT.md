# Sifr Stdlib Safety Principles Audit

**Date:** 2026-02-17
**Scope:** All 37 sifr stdlib modules compared against CPython originals, evaluated against sifr's safety philosophy.
**Reference:** `architecture.md` Safety Philosophy + Safety Adaptation Rules

---

## Executive Summary

Sifr's architecture promises: **"if it compiles, it works"** — no panics in user code, mandatory error handling via `Result[T, E]` / `Option[T]`, and compile-time rejection of unsafe patterns. This audit evaluates every stdlib module against those principles.

### Headline Findings

| Metric | Value |
| --- | --- |
| Total stdlib modules audited | 37 |
| Modules with **no safety violations** | 6 |
| Modules with **critical safety violations** (panics on normal input) | 19 |
| Modules with **moderate safety gaps** (missing Result/Option) | 12 |
| Total `.unwrap()` panic paths in intrinsics | ~45+ |
| Intrinsics returning `Result`/`Option` correctly | 2 (`env_get`, `re_find`) |
| Intrinsics that should return `Result`/`Option` but panic | ~40+ |

### Violation Severity Scale

| Severity | Meaning |
| --- | --- |
| **CRITICAL** | Operation panics on normal/expected input (e.g., file not found, empty list, invalid JSON) |
| **HIGH** | Operation can panic on edge-case input (e.g., NaN comparison, overflow) |
| **MODERATE** | Missing `Result`/`Option` return type but unlikely to panic in practice |
| **LOW** | API gap vs CPython but no safety issue |
| **SAFE** | Correctly implements sifr safety principles |

---

## Safety Principles Checklist

From `architecture.md`, every stdlib operation must satisfy:

1. **No panics in user code** — operations that can fail return `Result[T, E]` or `Option[T]`
2. **Mandatory error handling** — `Result` values are `#[must_use]`; ignoring is a compile-time error
3. **Where CPython raises an exception, sifr returns `Result[T, E]`**
4. **Where CPython raises `IndexError`, sifr returns `Option[T]`**
5. **Where CPython raises `KeyError`, sifr returns `Option[V]`**
6. **No panics on any input** — fuzz-safe

---

## Module-by-Module Safety Audit

### 1. `sifr.math` — Mathematical Functions

**CPython module:** `math` (C: `Modules/mathmodule.c`)
**Sifr implementation:** Intrinsics (`_sifr.math`) + pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `sqrt(-1)` | Raises `ValueError` | Returns `NaN` (Rust `f64::sqrt`) | **MODERATE** — should return `Result[float, ValueError]` |
| `log(0)` | Raises `ValueError` | Returns `-inf` (Rust `f64::ln`) | **MODERATE** — should return `Result[float, ValueError]` |
| `log(-1)` | Raises `ValueError` | Returns `NaN` | **MODERATE** |
| `factorial(-1)` | Raises `ValueError` | **Panics** (integer underflow or logic error) | **CRITICAL** |
| `factorial(very_large)` | Returns bigint | **Panics** (i64 overflow) | **CRITICAL** |
| `comb(n, k)` where k > n | Returns 0 | Depends on implementation | **MODERATE** |
| `floor/ceil/trunc` | Returns `int` | Returns `float` (type mismatch) | **LOW** — API divergence |
| `isnan/isinf/isfinite` | Returns `bool` | Returns `bool` | **SAFE** |
| `gcd/lcm` | Works on any int | Works on `i64` | **SAFE** (within i64 range) |
| `prod([])` | Returns 1 | Depends on implementation | **MODERATE** |

**Missing CPython functions:** `dist`, `erf`, `erfc`, `gamma`, `lgamma`, `frexp`, `ldexp`, `modf`, `remainder`, `nextafter`, `ulp`, `sumprod`, `cbrt`

**Safety Score: 6/10** — Two critical panic paths (factorial overflow, negative factorial), several moderate gaps where CPython raises but sifr returns NaN/inf silently.

---

### 2. `sifr.os` — Operating System Interface

**CPython module:** `os` (C: `Modules/posixmodule.c`, Py: `Lib/os.py`)
**Sifr implementation:** Intrinsics (`_sifr.fs`, `_sifr.sys`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `getcwd()` | Raises `OSError` if deleted | **Panics** (`.unwrap()`) | **CRITICAL** |
| `listdir(bad_path)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `mkdir(existing)` | Raises `FileExistsError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `rmdir(nonempty)` | Raises `OSError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `remove_file(missing)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `rename(missing, dst)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `is_file(path)` | Returns `bool` | Returns `bool` | **SAFE** |
| `is_dir(path)` | Returns `bool` | Returns `bool` | **SAFE** |
| `run_command(cmd)` | Returns exit code | Returns string | **MODERATE** — no error handling |
| `get_args()` | Returns list | Returns list | **SAFE** |

**Missing CPython functions:** `walk`, `makedirs`, `removedirs`, `symlink`, `readlink`, `chmod`, `chown`, `stat`, `getenv`, `putenv`, `environ`, `path.join`, `path.exists`, `path.abspath`, `path.realpath`, `path.expanduser`, `path.splitext`, `path.getsize`, `path.islink`, `sep`, `linesep`, `name`, `devnull`

**Safety Score: 2/10** — Nearly every filesystem operation panics instead of returning `Result`. This is the most critical safety violation in the stdlib.

---

### 3. `sifr.io` — File I/O

**CPython module:** `io` (C: `Modules/_io/`, Py: `Lib/io.py`)
**Sifr implementation:** Intrinsics (`_sifr.io`, `_sifr.fs`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `read_text(missing_file)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `write_text(readonly_path, data)` | Raises `PermissionError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `read_lines(missing)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `append_text(path, data)` | Raises `IOError` on failure | **Panics** (`.unwrap()`) | **CRITICAL** |
| `exists(path)` | Returns `bool` | Returns `bool` | **SAFE** |

**Missing CPython classes/functions:** `open()` (context manager), `StringIO`, `BytesIO`, `BufferedReader`, `BufferedWriter`, `TextIOWrapper`, `SEEK_SET/CUR/END`, file object protocol (`read`, `write`, `seek`, `tell`, `close`, `flush`, `readline`, `readlines`, `writelines`, `__enter__`, `__exit__`)

**Safety Score: 2/10** — All file read/write operations panic. This directly violates the architecture's rule: "Where CPython raises an exception, sifr returns `Result[T, E]`."

---

### 4. `sifr.re` — Regular Expressions

**CPython module:** `re` (C: `Modules/_sre/`, Py: `Lib/re/`)
**Sifr implementation:** Intrinsics (`_sifr.regex`) + pure sifr `Match` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `search(invalid_regex, text)` | Raises `re.error` | **Panics** (`Regex::new().unwrap()`) | **CRITICAL** |
| `search(pattern, text)` no match | Returns `None` | Returns `None` (via `re_find`) | **SAFE** |
| `sub(invalid_regex, repl, text)` | Raises `re.error` | **Panics** | **CRITICAL** |
| `findall(pattern, text)` | Returns `list` | Returns `list[str]` | **SAFE** |
| `split(pattern, text)` | Returns `list` | Returns `list[str]` | **SAFE** |

**Missing CPython functions:** `match`, `fullmatch`, `compile`, `subn`, `escape`, `purge`, flags (`IGNORECASE`, `MULTILINE`, `DOTALL`, `VERBOSE`, `ASCII`, `UNICODE`), `Match.group(n)`, `Match.groups()`, `Match.groupdict()`, `Match.expand()`, `Pattern` object

**Safety Score: 5/10** — Invalid regex panics instead of returning `Result`. Search correctly returns `Option`. Missing compile/flags API.

---

### 5. `sifr.json` — JSON Serialization

**CPython module:** `json` (Py: `Lib/json/`)
**Sifr implementation:** Intrinsics (`_sifr.json`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `loads(invalid_json)` | Raises `JSONDecodeError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `loads(valid_json)` | Returns object | Returns `str` (not typed) | **MODERATE** — loses type info |
| `dumps(obj)` | Returns `str` | Returns `str` | **SAFE** (if obj is serializable) |

**Missing CPython functions:** `dump` (to file), `load` (from file), `JSONEncoder`, `JSONDecoder`, `JSONDecodeError`, indent/sort_keys/separators options

**Safety Score: 3/10** — Invalid JSON panics. Return type is untyped `str` instead of structured data.

---

### 6. `sifr.time` — Time Functions

**CPython module:** `time` (C: `Modules/timemodule.c`)
**Sifr implementation:** Intrinsics (`_sifr.time`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `time()` | Returns `float` | Returns `float` | **SAFE** |
| `sleep(seconds)` | Blocks thread | Blocks thread | **SAFE** |
| `strftime(epoch, fmt)` | Raises on bad format | Depends on implementation | **MODERATE** |
| `perf_counter()` | Returns `float` | Returns `float` | **SAFE** |
| `monotonic()` | Returns `float` | Returns `float` | **SAFE** |

**Missing CPython functions:** `sleep` (with float seconds), `gmtime`, `localtime`, `mktime`, `asctime`, `ctime`, `strptime`, `struct_time`, `timezone`, `altzone`, `daylight`, `tzname`, `clock_gettime`, `clock_settime`, `process_time`, `thread_time`, `time_ns`, `perf_counter_ns`, `monotonic_ns`

**Safety Score: 8/10** — Time functions are inherently safe. Minor gap on format string validation.

---

### 7. `sifr.hashlib` — Hashing

**CPython module:** `hashlib` (C: `Modules/_hashopenssl.c`, Py: `Lib/hashlib.py`)
**Sifr implementation:** Intrinsics (`_sifr.crypto`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `sha256(data)` | Returns hash object | Returns hex string directly | **SAFE** |
| `md5(data)` | Returns hash object | Returns hex string directly | **SAFE** |
| `sha1(data)` | Returns hash object | Returns hex string directly | **SAFE** |
| `sha512(data)` | Returns hash object | Returns hex string directly | **SAFE** |

**Missing CPython features:** Hash object protocol (`update`, `digest`, `hexdigest`, `copy`, `block_size`, `digest_size`, `name`), `new(name)`, `algorithms_guaranteed`, `algorithms_available`, `pbkdf2_hmac`, `scrypt`, `blake2b`, `blake2s`, `sha3_*`, `shake_*`, `file_digest`

**Safety Score: 9/10** — Hashing is inherently safe. API is simplified (returns string directly vs hash object).

---

### 8. `sifr.base64` — Base64 Encoding

**CPython module:** `base64` (Py: `Lib/base64.py`)
**Sifr implementation:** Intrinsics (`_sifr.crypto`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `b64encode(data)` | Returns `bytes` | Returns `str` | **SAFE** |
| `b64decode(invalid)` | Raises `binascii.Error` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `urlsafe_b64encode(data)` | Returns `bytes` | Returns `str` | **SAFE** |
| `urlsafe_b64decode(invalid)` | Raises `binascii.Error` | **Panics** (`.unwrap()`) | **CRITICAL** |

**Missing CPython functions:** `standard_b64encode/decode`, `b32encode/decode`, `b16encode/decode`, `a85encode/decode`, `b85encode/decode`, `encodebytes`, `decodebytes`

**Safety Score: 4/10** — Decoding invalid base64 panics instead of returning `Result`.

---

### 9. `sifr.random` — Random Numbers

**CPython module:** `random` (Py: `Lib/random.py`)
**Sifr implementation:** Intrinsics (`_sifr.crypto`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `randint(a, b)` | Returns int in [a, b] | Returns int | **SAFE** |
| `random()` | Returns float in [0, 1) | Returns float | **SAFE** |
| `uniform(a, b)` | Returns float in [a, b] | Returns float | **SAFE** |
| `randint(5, 3)` (inverted range) | Raises `ValueError` | Undefined behavior | **HIGH** |

**Missing CPython functions:** `seed`, `getstate`, `setstate`, `choice`, `choices`, `shuffle`, `sample`, `randrange`, `getrandbits`, `randbytes`, `triangular`, `normalvariate`, `gauss`, `lognormvariate`, `expovariate`, `vonmisesvariate`, `gammavariate`, `betavariate`, `paretovariate`, `weibullvariate`, `binomialvariate`, `Random` class, `SystemRandom` class

**Safety Score: 7/10** — Core operations are safe. Missing validation on inverted ranges.

---

### 10. `sifr.collections` — Extended Collections

**CPython module:** `collections` (C: `Modules/_collectionsmodule.c`, Py: `Lib/collections/__init__.py`)
**Sifr implementation:** Intrinsics (`_sifr.collections`) + pure sifr `Counter` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `Counter.get(missing_key)` | Returns 0 | Returns 0 | **SAFE** |
| `Counter.most_common(n)` | Returns list of tuples | Returns string (serialized) | **MODERATE** — type safety loss |
| `set_add(set, item)` | Adds item | Adds item | **SAFE** |
| `set_contains(set, item)` | Returns `bool` | Returns `bool` | **SAFE** |
| `set.pop()` on empty | Raises `KeyError` | **Panics** (`.unwrap()`) | **CRITICAL** |

**Missing CPython classes/functions:** `OrderedDict`, `defaultdict` (partial intrinsic exists), `deque`, `ChainMap`, `UserDict`, `UserList`, `UserString`, `namedtuple`, `Counter.subtract`, `Counter.update`, `Counter.elements`, `Counter.__add__`, `Counter.__sub__`, `Counter.__and__`, `Counter.__or__`

**Safety Score: 5/10** — `set.pop()` on empty panics. Counter API returns serialized strings instead of typed data.

---

### 11. `sifr.string` — String Constants

**CPython module:** `string` (Py: `Lib/string.py`)
**Sifr implementation:** Pure sifr constants

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| All constants | String constants | String constants | **SAFE** |
| `capwords(s)` | Returns capitalized words | Returns capitalized words | **SAFE** |

**Missing CPython features:** `Template` class, `Formatter` class

**Safety Score: 10/10** — Constants are inherently safe.

---

### 12. `sifr.statistics` — Statistical Functions

**CPython module:** `statistics` (Py: `Lib/statistics.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `mean([])` | Raises `StatisticsError` | **Panics** (division by zero) | **CRITICAL** |
| `median([])` | Raises `StatisticsError` | Returns `0.0` sentinel | **HIGH** — silent wrong answer |
| `variance([])` | Raises `StatisticsError` | **Panics** (division by zero) | **CRITICAL** |
| `variance([x])` | Raises `StatisticsError` (need ≥2) | **Panics** (division by zero) | **CRITICAL** |
| `stdev([])` | Raises `StatisticsError` | **Panics** | **CRITICAL** |
| `mode([])` | Raises `StatisticsError` | Undefined | **CRITICAL** |
| `harmonic_mean([0])` | Raises `StatisticsError` | **Panics** (division by zero) | **CRITICAL** |
| `geometric_mean([0])` | Raises `StatisticsError` | Returns `0.0` or `NaN` | **HIGH** |

**Missing CPython functions:** `median_grouped`, `multimode`, `quantiles`, `covariance`, `correlation`, `linear_regression`, `NormalDist` class, `StatisticsError`

**Safety Score: 1/10** — Nearly every function panics on empty input. This is the worst safety violation — these are pure sifr functions that could easily return `Result`.

---

### 13. `sifr.bisect` — Array Bisection

**CPython module:** `bisect` (C: `Modules/_bisectmodule.c`, Py: `Lib/bisect.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `bisect_left([], x)` | Returns 0 | Returns 0 | **SAFE** |
| `bisect_right([], x)` | Returns 0 | Returns 0 | **SAFE** |
| `insort_left(a, x)` | Inserts in place | Returns new list | **SAFE** (different semantics) |
| `insort_right(a, x)` | Inserts in place | Returns new list | **SAFE** (different semantics) |

**Missing CPython features:** `lo`/`hi` bounds parameters, `key` parameter

**Safety Score: 9/10** — Bisection on sorted arrays is inherently safe. Minor API divergence (returns new list vs in-place).

---

### 14. `sifr.functools` — Functional Tools

**CPython module:** `functools` (Py: `Lib/functools.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `identity(x)` | N/A (not in CPython) | Returns x | **SAFE** |
| `clamp(value, min, max)` | N/A (not in CPython) | Returns clamped value | **SAFE** |

**Missing CPython functions:** `reduce`, `partial`, `partialmethod`, `wraps`, `update_wrapper`, `total_ordering`, `cmp_to_key`, `lru_cache`, `cache`, `cached_property`, `singledispatch`, `singledispatchmethod`

**Safety Score: 10/10** — The two functions present are safe, but this module bears almost no resemblance to CPython's `functools`.

---

### 15. `sifr.secrets` — Secure Random

**CPython module:** `secrets` (Py: `Lib/secrets.py`)
**Sifr implementation:** Intrinsics (`_sifr.crypto`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `token_hex(nbytes)` | Returns hex string | Returns hex string | **SAFE** |
| `randbelow(0)` | Raises `ValueError` | Undefined | **HIGH** |
| `randbelow(n)` | Returns int in [0, n) | Returns int | **SAFE** |

**Missing CPython functions:** `token_bytes`, `token_urlsafe`, `compare_digest`, `choice`, `SystemRandom`

**Safety Score: 7/10** — Core operations safe. Missing edge case validation.

---

### 16. `sifr.heapq` — Heap Queue

**CPython module:** `heapq` (C: `Modules/_heapqmodule.c`, Py: `Lib/heapq.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `heappop([])` | Raises `IndexError` | **Panics** (empty list access) | **CRITICAL** |
| `heapreplace([], item)` | Raises `IndexError` | **Panics** | **CRITICAL** |
| `heappush(heap, item)` | Modifies in place | Returns new list | **SAFE** (different semantics) |
| `nsmallest(n, [])` | Returns `[]` | Returns `[]` | **SAFE** |
| `nlargest(n, [])` | Returns `[]` | Returns `[]` | **SAFE** |

**Missing CPython functions:** `merge`, `heappop_max`, `heapreplace_max`, `heappush_max`, `heappushpop_max`, `heapify_max`, `key` parameter

**Safety Score: 4/10** — `heappop` and `heapreplace` on empty heap panics. Should return `Option[int]`.

---

### 17. `sifr.itertools` — Iterator Building Blocks

**CPython module:** `itertools` (C: `Modules/itertoolsmodule.c`)
**Sifr implementation:** Pure sifr (eager, not lazy)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `chain(a, b)` | Returns lazy iterator | Returns `list[int]` (eager) | **SAFE** (different semantics) |
| `take(n, data)` | N/A (itertools recipe) | Returns first n elements | **SAFE** |
| `flatten(lists)` | N/A (chain.from_iterable) | Returns flat list | **SAFE** |
| `pairwise(data)` | Returns lazy iterator | Returns `list[list[int]]` | **SAFE** |
| `batched(data, 0)` | Raises `ValueError` | Undefined | **HIGH** |

**Missing CPython functions:** `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `groupby`, `starmap`, `tee`, `zip_longest`, `product`, `permutations`, `combinations`, `combinations_with_replacement`, `count`, `cycle`, `repeat` (proper lazy version)

**Safety Score: 7/10** — Functions are safe but eager (not lazy). Only works with `int` and `str` types (not generic).

---

### 18. `sifr.textwrap` — Text Wrapping

**CPython module:** `textwrap` (Py: `Lib/textwrap.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `wrap(text, 0)` | Raises `ValueError` | Undefined | **HIGH** |
| `wrap(text, width)` | Returns list of lines | Returns `list[str]` | **SAFE** |
| `fill(text, width)` | Returns single string | Returns `str` | **SAFE** |
| `dedent(text)` | Removes common indent | Returns `str` | **SAFE** |
| `indent(text, prefix)` | Adds prefix | Returns `str` | **SAFE** |
| `shorten(text, width)` | Truncates with placeholder | Returns `str` | **SAFE** |

**Missing CPython features:** `TextWrapper` class, `initial_indent`, `subsequent_indent`, `expand_tabs`, `replace_whitespace`, `fix_sentence_endings`, `break_long_words`, `drop_whitespace`, `break_on_hyphens`, `tabsize`, `max_lines`, `placeholder`

**Safety Score: 8/10** — Mostly safe. Missing edge case for width=0.

---

### 19. `sifr.csv` — CSV Parsing

**CPython module:** `csv` (C: `Modules/_csv.c`, Py: `Lib/csv.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `parse_row(line)` | Via `reader` object | Returns `list[str]` | **SAFE** |
| `parse_csv(text)` | Via `reader` object | Returns `list[list[str]]` | **SAFE** |
| `format_row(fields)` | Via `writer` object | Returns `str` | **SAFE** |
| `format_csv(rows)` | Via `writer` object | Returns `str` | **SAFE** |

**Missing CPython features:** `reader`/`writer` objects, `DictReader`/`DictWriter`, `Dialect` classes, `Sniffer`, quoting modes (`QUOTE_MINIMAL`, `QUOTE_ALL`, etc.), `register_dialect`, `field_size_limit`, delimiter/quotechar/escapechar options

**Safety Score: 8/10** — Simple API is safe. Missing advanced features.

---

### 20. `sifr.argparse` — Argument Parsing

**CPython module:** `argparse` (Py: `Lib/argparse.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `parse_flag(args, flag)` | N/A (different API) | Returns `bool` | **SAFE** |
| `parse_option(args, name, default)` | N/A | Returns `str` | **SAFE** |
| `parse_positional(args, index, default)` | N/A | Returns `str` | **SAFE** |

**Missing CPython features:** `ArgumentParser` class, `add_argument`, `parse_args`, `parse_known_args`, subparsers, mutually exclusive groups, argument types, choices, required arguments, help generation, `Namespace`, `FileType`, `BooleanOptionalAction`, error handling

**Safety Score: 9/10** — Simple API is safe. Completely different design from CPython's argparse.

---

### 21. `sifr.fnmatch` — Filename Matching

**CPython module:** `fnmatch` (Py: `Lib/fnmatch.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `fnmatch(name, pat)` | Returns `bool` | Returns `bool` | **SAFE** |
| `filter(names, pat)` | Returns filtered list | Returns `list[str]` | **SAFE** |
| `fnmatchcase(name, pat)` | Returns `bool` | Returns `bool` | **SAFE** |

**Missing CPython functions:** `translate` (convert pattern to regex), `filterfalse`

**Safety Score: 10/10** — Pattern matching is inherently safe.

---

### 22. `sifr.glob` — Filename Globbing

**CPython module:** `glob` (Py: `Lib/glob.py`)
**Sifr implementation:** Intrinsics (`_sifr.fs`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `glob(dir, pattern)` | Returns list of paths | Returns `list[str]` | **MODERATE** — no error on bad dir |

**Missing CPython functions:** `iglob` (lazy), `escape`, `translate`, `recursive` parameter, `include_hidden` parameter, `root_dir` parameter

**Safety Score: 7/10** — Works but silently returns empty list on invalid directory instead of `Result`.

---

### 23. `sifr.shutil` — High-Level File Operations

**CPython module:** `shutil` (Py: `Lib/shutil.py`)
**Sifr implementation:** Intrinsics (`_sifr.fs`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `copy(missing, dst)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `move_file(missing, dst)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `rmtree(missing)` | Raises `FileNotFoundError` | **Panics** (`.unwrap()`) | **CRITICAL** |

**Missing CPython functions:** `copy2`, `copytree`, `copyfileobj`, `copyfile`, `copymode`, `copystat`, `make_archive`, `unpack_archive`, `which`, `get_terminal_size`, `chown`, `ignore_patterns`, `disk_usage`

**Safety Score: 1/10** — Every operation panics on failure. All should return `Result`.

---

### 24. `sifr.tempfile` — Temporary Files

**CPython module:** `tempfile` (Py: `Lib/tempfile.py`)
**Sifr implementation:** Pure sifr + intrinsics

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `mktemp_path(prefix)` | Deprecated, returns path | Returns path string | **SAFE** (path generation only) |
| `mkstemp(prefix)` | Creates file, returns (fd, name) | Returns path string | **MODERATE** — no fd, no error handling |
| `mkdtemp(prefix)` | Creates dir, returns path | Returns path string | **MODERATE** — no error handling |

**Missing CPython classes/functions:** `NamedTemporaryFile`, `TemporaryFile`, `SpooledTemporaryFile`, `TemporaryDirectory` (context managers), `gettempdir`, `gettempprefix`

**Safety Score: 6/10** — Path generation is safe but actual file/dir creation may fail silently.

---

### 25. `sifr.graphlib` — Graph Algorithms

**CPython module:** `graphlib` (Py: `Lib/graphlib.py`)
**Sifr implementation:** Pure sifr + `TopologicalSorter` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `topological_sort(cyclic_graph)` | Raises `CycleError` | Undefined (likely infinite loop or wrong result) | **CRITICAL** |
| `TopologicalSorter.static_order()` | Raises `CycleError` on cycle | Undefined | **CRITICAL** |

**Missing CPython features:** `CycleError`, `prepare()`, `get_ready()`, `is_active()`, `done(*nodes)`, parallel execution support

**Safety Score: 3/10** — Cycle detection missing. Should return `Result[list[int], CycleError]`.

---

### 26. `sifr.uuid` — UUID Generation

**CPython module:** `uuid` (Py: `Lib/uuid.py`)
**Sifr implementation:** Intrinsics (`_sifr.uuid`) + `UUID` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `uuid4()` | Returns `UUID` object | Returns `str` | **SAFE** |
| `UUID(invalid_hex)` | Raises `ValueError` | Undefined | **HIGH** |

**Missing CPython functions:** `uuid1`, `uuid3`, `uuid5`, `SafeUUID`, UUID properties (`bytes`, `bytes_le`, `fields`, `time`, `clock_seq`, `node`, `variant`, `version`, `is_safe`), namespace constants (`NAMESPACE_DNS`, `NAMESPACE_URL`, `NAMESPACE_OID`, `NAMESPACE_X500`)

**Safety Score: 6/10** — `uuid4()` is safe. `UUID` constructor with invalid hex is undefined.

---

### 27. `sifr.platform` — Platform Information

**CPython module:** `platform` (Py: `Lib/platform.py`)
**Sifr implementation:** Intrinsics (`_sifr.platform`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `system()` | Returns OS name | Returns OS name | **SAFE** |
| `machine()` | Returns arch | Returns arch | **SAFE** |

**Missing CPython functions:** `node`, `release`, `version`, `processor`, `platform`, `python_version`, `python_implementation`, `python_compiler`, `python_build`, `python_branch`, `python_revision`, `uname`, `architecture`, `mac_ver`, `win32_ver`, `libc_ver`, `freedesktop_os_release`

**Safety Score: 10/10** — Platform info queries are inherently safe.

---

### 28. `sifr.pathlib` — Path Manipulation

**CPython module:** `pathlib` (Py: `Lib/pathlib/`)
**Sifr implementation:** Pure sifr `Path` class + intrinsics

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `Path.read_text()` on missing | Raises `FileNotFoundError` | **Panics** (delegates to `_sifr.fs.read_text`) | **CRITICAL** |
| `Path.write_text(data)` on readonly | Raises `PermissionError` | **Panics** | **CRITICAL** |
| `Path.mkdir()` existing | Raises `FileExistsError` | **Panics** | **CRITICAL** |
| `Path.exists()` | Returns `bool` | Returns `bool` | **SAFE** |
| `Path.is_file()` | Returns `bool` | Returns `bool` | **SAFE** |
| `Path.is_dir()` | Returns `bool` | Returns `bool` | **SAFE** |
| `Path.name/parent/suffix/stem` | Returns path component | Returns `str` | **SAFE** |

**Missing CPython features:** `PurePath`, `PurePosixPath`, `PureWindowsPath`, `PosixPath`, `WindowsPath`, `/` operator, `with_name`, `with_stem`, `with_suffix`, `relative_to`, `is_relative_to`, `as_posix`, `as_uri`, `match`, `glob`, `rglob`, `iterdir`, `resolve`, `absolute`, `stat`, `lstat`, `chmod`, `rename`, `replace`, `unlink`, `rmdir`, `touch`, `symlink_to`, `hardlink_to`, `readlink`, `owner`, `group`, `open`, `read_bytes`, `write_bytes`

**Safety Score: 3/10** — File operations panic. Path manipulation (pure) is safe.

---

### 29. `sifr.logging` — Logging

**CPython module:** `logging` (Py: `Lib/logging/__init__.py`)
**Sifr implementation:** Pure sifr `Logger` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `log_info(msg)` | Logs to handler | Prints to stdout | **SAFE** |
| `Logger.debug/info/warning/error/critical` | Logs with level | Prints to stdout | **SAFE** |
| `Logger.set_level(level)` | Sets minimum level | Sets level | **SAFE** |

**Missing CPython features:** `Handler`, `StreamHandler`, `FileHandler`, `NullHandler`, `Formatter`, `Filter`, `LogRecord`, `basicConfig`, `getLogger` hierarchy, `addHandler`, `removeHandler`, `setFormatter`, `propagate`, `exception`, `log`, `makeRecord`, `isEnabledFor`, `getEffectiveLevel`, `hasHandlers`, log level constants (`DEBUG`, `INFO`, `WARNING`, `ERROR`, `CRITICAL`)

**Safety Score: 9/10** — Logging is inherently safe (print-based). Very simplified vs CPython.

---

### 30. `sifr.difflib` — Sequence Comparison

**CPython module:** `difflib` (Py: `Lib/difflib.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `get_close_matches(word, [], n, cutoff)` | Returns `[]` | Returns `[]` | **SAFE** |
| `unified_diff(a, b)` | Returns iterator of lines | Returns `list[str]` | **SAFE** |

**Missing CPython classes/functions:** `SequenceMatcher`, `Differ`, `HtmlDiff`, `ndiff`, `context_diff`, `diff_bytes`, `restore`, `IS_CHARACTER_JUNK`, `IS_LINE_JUNK`

**Safety Score: 9/10** — Simple comparison functions are safe.

---

### 31. `sifr.ipaddress` — IP Address Manipulation

**CPython module:** `ipaddress` (Py: `Lib/ipaddress.py`)
**Sifr implementation:** Pure sifr

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `is_valid_ipv4(invalid)` | N/A (raises `ValueError` in `ip_address()`) | Returns `false` | **SAFE** |
| `ip_to_int(invalid)` | Raises `ValueError` | Undefined (may panic on parse) | **HIGH** |
| `is_private/loopback/multicast/global` | Returns `bool` | Returns `bool` | **SAFE** |
| `int_to_ip(negative)` | Raises `ValueError` | Undefined | **HIGH** |

**Missing CPython classes/functions:** `IPv4Address`, `IPv6Address`, `IPv4Network`, `IPv6Network`, `IPv4Interface`, `IPv6Interface`, `ip_address()`, `ip_network()`, `ip_interface()`, `v4_int_to_packed`, `v6_int_to_packed`, `summarize_address_range`, `collapse_addresses`, `AddressValueError`, `NetmaskValueError`

**Safety Score: 6/10** — Boolean queries are safe. Conversion functions may panic on invalid input.

---

### 32. `sifr.timeit` — Execution Timing

**CPython module:** `timeit` (Py: `Lib/timeit.py`)
**Sifr implementation:** Pure sifr using `Callable`

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `default_timer()` | Returns `float` | Returns `float` | **SAFE** |
| `timeit(stmt, number)` | Returns `float` | Returns `float` | **SAFE** |
| `repeat(stmt, count, number)` | Returns `list[float]` | Returns `list[float]` | **SAFE** |
| `timeit(stmt, 0)` | Returns 0.0 | Returns 0.0 | **SAFE** |

**Missing CPython features:** `Timer` class, `setup` parameter, `globals` parameter, `autorange`, `print_exc`, string statement support

**Safety Score: 9/10** — Timing operations are inherently safe.

---

### 33. `sifr.tomllib` — TOML Parsing

**CPython module:** `tomllib` (Py: `Lib/tomllib/`)
**Sifr implementation:** Intrinsics (`_sifr.toml`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `loads(invalid_toml)` | Raises `TOMLDecodeError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `loads(valid_toml)` | Returns `dict` | Returns `str` (serialized) | **MODERATE** — type safety loss |
| `load(missing_file)` | Raises `FileNotFoundError` | **Panics** | **CRITICAL** |

**Missing CPython features:** `TOMLDecodeError`, typed return values

**Safety Score: 2/10** — Both parsing and file loading panic on invalid input.

---

### 34. `sifr.datetime` — Date and Time

**CPython module:** `datetime` (Py: `Lib/datetime.py`)
**Sifr implementation:** Intrinsics (`_sifr.datetime`) + `timedelta` class

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `now()` | Returns `datetime` object | Returns `str` | **SAFE** |
| `from_timestamp(invalid)` | Raises `ValueError`/`OSError` | Undefined | **HIGH** |
| `format_datetime(dt, bad_fmt)` | Raises `ValueError` | Undefined | **HIGH** |
| `timedelta` arithmetic | Returns `timedelta` | Returns `timedelta` | **SAFE** |
| `timedelta` overflow | Raises `OverflowError` | **Panics** (i64 overflow) | **CRITICAL** |

**Missing CPython classes:** `datetime`, `date`, `time`, `timezone`, `tzinfo`, `MINYEAR`, `MAXYEAR`, `datetime.strptime`, `datetime.strftime`, `datetime.combine`, `datetime.fromordinal`, `datetime.fromisoformat`, `datetime.isoformat`, `datetime.timetuple`, `datetime.weekday`, `datetime.isoweekday`, `datetime.isocalendar`, `datetime.replace`, `date.today`, `time.min`, `time.max`

**Safety Score: 4/10** — String-based datetime representation loses type safety. Overflow panics.

---

### 35. `sifr.env` — Environment Variables

**CPython module:** `os.environ` (C: `Modules/posixmodule.c`)
**Sifr implementation:** Intrinsics (`_sifr.sys`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `env_get(missing_key)` | Raises `KeyError` (on `os.environ[key]`) / Returns `None` (on `os.getenv`) | Returns `str \| None` | **SAFE** |
| `env_set(key, value)` | Sets env var | Sets env var | **SAFE** |

**Safety Score: 9/10** — One of the few modules that correctly returns `Option` for missing keys.

---

### 36. `sifr.bytes` — Binary Data

**CPython module:** `bytes`/`bytearray` (C: `Objects/bytesobject.c`)
**Sifr implementation:** Intrinsics (`_sifr.bytes`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `encode_utf8(s)` | Returns `bytes` | Returns encoded data | **SAFE** |
| `decode_utf8(invalid)` | Raises `UnicodeDecodeError` | **Panics** (`.unwrap()`) | **CRITICAL** |
| `bytes_to_hex(data)` | Returns hex string | Returns hex string | **SAFE** |
| `bytes_from_hex(invalid)` | Raises `ValueError` | **Panics** (`.unwrap()`) | **CRITICAL** |

**Missing CPython features:** `bytes` type, `bytearray` type, `bytes.decode`, `str.encode`, `bytes.hex`, `bytes.fromhex`, `bytes.__getitem__`, `bytes.__contains__`, `bytes.split`, `bytes.join`, `bytes.replace`, `bytes.strip`, `bytes.startswith`, `bytes.endswith`, `bytes.find`, `bytes.count`, `bytes.upper`, `bytes.lower`, `bytearray.append`, `bytearray.extend`, `bytearray.pop`, `bytearray.insert`, `bytearray.remove`, `bytearray.reverse`, `bytearray.clear`, `memoryview`

**Safety Score: 4/10** — Decoding and hex parsing panic on invalid input.

---

### 37. `sifr.test` — Test Assertions

**CPython module:** `unittest` (Py: `Lib/unittest/`)
**Sifr implementation:** Intrinsics (`_sifr.test`)

| Operation | CPython Behavior | Sifr Behavior | Safety Verdict |
| --- | --- | --- | --- |
| `assert_eq(a, b)` | Raises `AssertionError` | Panics (intentional) | **SAFE** (by design — `assert` is the only panic) |
| `assert_ne/true/false/gt/lt` | Raises `AssertionError` | Panics (intentional) | **SAFE** (by design) |

**Safety Score: 10/10** — Test assertions are intentionally panicking (matches sifr's "`assert` is the only panic" rule).

---

## Summary: Safety Scores by Module

| Module | Safety Score | Critical Violations | Key Issue |
| --- | ---: | ---: | --- |
| `sifr.string` | 10/10 | 0 | — |
| `sifr.fnmatch` | 10/10 | 0 | — |
| `sifr.functools` | 10/10 | 0 | — (but barely resembles CPython) |
| `sifr.platform` | 10/10 | 0 | — |
| `sifr.test` | 10/10 | 0 | — (panics by design) |
| `sifr.bisect` | 9/10 | 0 | — |
| `sifr.env` | 9/10 | 0 | — |
| `sifr.hashlib` | 9/10 | 0 | — |
| `sifr.logging` | 9/10 | 0 | — |
| `sifr.difflib` | 9/10 | 0 | — |
| `sifr.timeit` | 9/10 | 0 | — |
| `sifr.argparse` | 9/10 | 0 | — |
| `sifr.time` | 8/10 | 0 | Format string validation |
| `sifr.textwrap` | 8/10 | 0 | Width=0 edge case |
| `sifr.csv` | 8/10 | 0 | — |
| `sifr.random` | 7/10 | 0 | Inverted range edge case |
| `sifr.secrets` | 7/10 | 0 | `randbelow(0)` edge case |
| `sifr.itertools` | 7/10 | 0 | Eager, not lazy; `batched(data, 0)` |
| `sifr.glob` | 7/10 | 0 | Silent failure on bad dir |
| `sifr.math` | 6/10 | 2 | `factorial` overflow/negative panics |
| `sifr.uuid` | 6/10 | 0 | Invalid hex constructor |
| `sifr.ipaddress` | 6/10 | 0 | Parse failures undefined |
| `sifr.tempfile` | 6/10 | 0 | File creation may fail silently |
| `sifr.re` | 5/10 | 2 | Invalid regex panics |
| `sifr.collections` | 5/10 | 1 | `set.pop()` empty panics |
| `sifr.heapq` | 4/10 | 2 | `heappop`/`heapreplace` empty panics |
| `sifr.base64` | 4/10 | 2 | Decode invalid panics |
| `sifr.bytes` | 4/10 | 2 | Decode/fromhex panics |
| `sifr.datetime` | 4/10 | 1 | Overflow panics, string-based API |
| `sifr.json` | 3/10 | 1 | Invalid JSON panics |
| `sifr.graphlib` | 3/10 | 2 | Cycle detection missing |
| `sifr.os` | 2/10 | 6 | Nearly all ops panic |
| `sifr.io` | 2/10 | 4 | All file ops panic |
| `sifr.tomllib` | 2/10 | 2 | Parse + file load panic |
| `sifr.pathlib` | 3/10 | 3 | File ops panic |
| `sifr.shutil` | 1/10 | 3 | Every op panics |
| `sifr.statistics` | 1/10 | 6 | Nearly every function panics on empty |

---

## Aggregate Analysis

### By Violation Category

| Category | Count | Modules |
| --- | ---: | --- |
| **File I/O panics** | 5 modules | os, io, shutil, pathlib, tomllib |
| **Empty collection panics** | 4 modules | statistics, heapq, collections, math |
| **Parse/decode panics** | 5 modules | json, tomllib, base64, bytes, re |
| **Overflow panics** | 2 modules | math, datetime |
| **Missing cycle/error detection** | 1 module | graphlib |

### Root Cause

The root cause is a **single architectural gap**: stdlib intrinsics in `crates/sifr_hir/src/stdlib.rs` declare infallible return types (e.g., `read_text -> str`), and the codegen in `crates/sifr_codegen/src/lib.rs` emits `.unwrap()` for all Rust `Result`/`Option` values. The `Result`/`Option` type system support exists and works (for user code), but has not been applied to the stdlib layer.

### What Works Well

1. **Type system infrastructure**: `Result[T, E]` and `Option[T]` types exist, `try`/`except` generates `match`, `?` operator works, `raise` generates `Err(...)`.
2. **Pure sifr modules**: Modules like `string`, `fnmatch`, `bisect`, `csv` that don't touch OS/parsing are inherently safe.
3. **Boolean queries**: `exists()`, `is_file()`, `is_dir()`, `is_valid_ipv4()` correctly return `bool`.
4. **`env_get`**: Correctly returns `str | None` — the model for how all fallible intrinsics should work.
5. **`re_find`**: Correctly returns `str | None` for search operations.

---

## Recommendations

### Priority 1: Critical — File I/O Safety (5 modules, ~15 intrinsics)

All file system intrinsics must return `Result[T, IOError]`:
- `read_text(path) -> Result[str, IOError]`
- `write_text(path, data) -> Result[None, IOError]`
- `mkdir(path) -> Result[None, IOError]`
- `rmdir(path) -> Result[None, IOError]`
- `remove_file(path) -> Result[None, IOError]`
- `rename(src, dst) -> Result[None, IOError]`
- `copy_file(src, dst) -> Result[None, IOError]`
- `rmdir_all(path) -> Result[None, IOError]`
- `listdir(path) -> Result[list[str], IOError]`
- `getcwd() -> Result[str, IOError]`
- `read_lines(path) -> Result[list[str], IOError]`
- `append_text(path, data) -> Result[None, IOError]`

**Impact:** Fixes `sifr.os`, `sifr.io`, `sifr.shutil`, `sifr.pathlib`, `sifr.tomllib` (file loading)

### Priority 2: Critical — Parse/Decode Safety (5 modules, ~8 intrinsics)

All parsing intrinsics must return `Result[T, ParseError]`:
- `json_loads(s) -> Result[str, ParseError]`
- `toml_parse(s) -> Result[str, ParseError]`
- `base64_decode(s) -> Result[str, ParseError]`
- `urlsafe_b64decode(s) -> Result[str, ParseError]`
- `decode_utf8(data) -> Result[str, ParseError]`
- `bytes_from_hex(s) -> Result[str, ParseError]`
- `Regex::new(pattern)` → return `Result` on invalid regex

**Impact:** Fixes `sifr.json`, `sifr.tomllib`, `sifr.base64`, `sifr.bytes`, `sifr.re`

### Priority 3: High — Empty Collection Safety (4 modules, ~10 functions)

Pure sifr functions must return `Result`/`Option` on empty input:
- `statistics.mean([]) -> Result[float, StatisticsError]`
- `statistics.median([]) -> Result[float, StatisticsError]`
- `statistics.variance([]) -> Result[float, StatisticsError]`
- `statistics.stdev([]) -> Result[float, StatisticsError]`
- `statistics.mode([]) -> Result[int, StatisticsError]`
- `heapq.heappop([]) -> Option[int]`
- `heapq.heapreplace([], item) -> Result[int, ValueError]`
- `collections.set_pop(empty) -> Option[str]`
- `math.factorial(-1) -> Result[int, ValueError]`

**Impact:** Fixes `sifr.statistics`, `sifr.heapq`, `sifr.collections`, `sifr.math`

### Priority 4: Moderate — Edge Case Validation

- `math.factorial(large_n)` — check for i64 overflow, return `Result`
- `random.randint(5, 3)` — validate a ≤ b, return `Result`
- `secrets.randbelow(0)` — validate n > 0, return `Result`
- `textwrap.wrap(text, 0)` — validate width > 0, return `Result`
- `itertools.batched(data, 0)` — validate n > 0, return `Result`
- `graphlib.topological_sort(cyclic)` — detect cycles, return `Result`
- `uuid.UUID(invalid_hex)` — validate hex, return `Result`
- `ipaddress.ip_to_int(invalid)` — validate address, return `Result`
- `datetime.from_timestamp(invalid)` — validate timestamp, return `Result`

### Implementation Strategy

The fix requires changes at three layers:

1. **`stdlib.rs` (type signatures):** Change intrinsic return types from `T` to `Result[T, E]` or `Option[T]`
2. **`codegen/lib.rs` (Rust codegen):** Replace `.unwrap()` with `?` or proper error propagation
3. **`lib/sifr/*.sifr` (wrappers):** Update sifr wrappers to propagate errors with `?` or handle them

The infrastructure already exists — `env_get` and `re_find` prove the pattern works end-to-end.
