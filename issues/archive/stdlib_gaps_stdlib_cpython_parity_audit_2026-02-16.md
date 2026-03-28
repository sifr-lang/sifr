# Sifr Stdlib vs CPython — Comprehensive Gap Audit

Date: 2026-02-16
Source: Compared every `lib/sifr/*.sifr` module against CPython source at `/Users/yaseralnajjar/work/sifr/cpython/Lib/`

## Executive Summary

- All 37 Sifr stdlib modules exist and compile.
- **API naming mismatches** are widespread — most Sifr functions use different names than their CPython counterparts.
- **Class-based APIs** are the single biggest blocker — 12+ modules need classes to reach meaningful parity.
- **Pure-Sifr function additions** (no compiler changes needed) could close ~30 individual function gaps immediately.
- **Intrinsic additions** (Rust codegen) could close ~15 more.
- **One semantic bug**: `sifr.statistics.variance` computes population variance (÷N) but CPython's `variance` computes sample variance (÷N-1).

---

## Cross-Cutting Gap Categories

| Category | Modules Affected | Blocker | Estimated Effort |
|---|---|---|---|
| API naming mismatches | math, os, re, json, time, hashlib, base64, random, platform, shutil, fnmatch | Rename + update tests | Low |
| Missing pure-Sifr functions | math, statistics, bisect, secrets, fnmatch, string, textwrap, pathlib, heapq | None — pure Sifr | Low-Medium |
| Missing intrinsics (Rust codegen) | math, hashlib, base64, random, platform, os | Codegen additions | Medium |
| Class-based APIs | argparse, csv, logging, pathlib, graphlib, uuid, collections, datetime, re, tempfile | Class support in stdlib + Callable-as-struct-field fix | High |
| Generic type support | bisect, heapq, itertools | Generics milestone | High |
| Lazy iterators | itertools, csv, glob | Iterator protocol | High |
| Return type mismatches | tomllib (returns str not dict), json (returns str not native) | Structured return types | Medium |

---

## Module-by-Module Detail

### 1. `sifr.math` — ~85% coverage

**Has (29 functions + 5 constants):** `sqrt`, `floor`, `ceil`, `abs_val`, `log`, `sin`, `cos`, `tan`, `pow_val`, `min_val`, `max_val`, `round_val`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `log10`, `log2`, `degrees`, `radians`, `isnan`, `isinf`, `trunc`, `copysign`, `fmod`, `hypot` + `pi`, `e`, `tau`, `inf`, `nan`

**Missing (achievable with intrinsics):**
- `exp(x)` — Rust: `f64::exp()`
- `expm1(x)` — Rust: `f64::exp_m1()`
- `log1p(x)` — Rust: `f64::ln_1p()`
- `fabs(x)` — Rust: `f64::abs()`
- `isfinite(x)` — Rust: `f64::is_finite()`
- `ldexp(x, i)` — Rust: `f64::from_bits` / manual
- `frexp(x)` — needs tuple return
- `modf(x)` — needs tuple return

**Missing (achievable as pure Sifr):**
- `factorial(n)` — simple loop
- `gcd(a, b)` — Euclidean algorithm
- `lcm(a, b)` — `a * b // gcd(a, b)`
- `comb(n, k)` — combinations
- `perm(n, k)` — permutations
- `isclose(a, b, rel_tol, abs_tol)` — approximate equality
- `fsum(iterable)` — accurate sum (simplified version)
- `prod(iterable)` — product of elements
- `dist(p, q)` — Euclidean distance (2 lists)

**API naming issues:** `abs_val` → `fabs`, `pow_val` → `pow`, `min_val`/`max_val` → builtins not math, `round_val` → builtin `round`

---

### 2. `sifr.os` — ~40% coverage

**Has:** `run_command`, `get_args`, `getcwd`, `listdir`, `mkdir`, `rmdir`, `remove_file`, `rename`, `is_file`, `is_dir`

**Missing (intrinsic-feasible):**
- `makedirs(path)` — recursive mkdir
- `chdir(path)` — change directory
- `getpid()` — Rust: `std::process::id()`
- `path.abspath(path)` — Rust: `std::fs::canonicalize`
- `path.getsize(path)` — Rust: `std::fs::metadata().len()`
- `sep`, `linesep` — OS constants

**Missing (needs classes/advanced features):**
- `walk(top)` — recursive tree walker (returns tuples of (dirpath, dirnames, filenames))
- `environ` dict-like interface
- `path` submodule as a proper module

**API naming:** `run_command` (CPython: `subprocess.run`/`os.system`), `get_args` (CPython: `sys.argv`), `remove_file` (CPython: `os.remove`)

---

### 3. `sifr.io` — ~35% coverage

**Has:** `read_text`, `write_text`, `exists`, `read_lines`, `append_text`

**Missing (intrinsic-feasible):**
- `read_bytes(path)` / `write_bytes(path, data)` — binary I/O

**Missing (needs classes):**
- `open(path, mode)` — requires context manager / file objects
- `StringIO` / `BytesIO` — in-memory streams
- File objects with `.read()`, `.write()`, `.close()`, `.readline()`

---

### 4. `sifr.re` — ~45% coverage

**Has:** `re_match`, `re_find`, `re_replace`, `re_findall`, `re_split`

**Missing (intrinsic-feasible):**
- `re.fullmatch(pattern, string)` — match entire string
- `re.escape(pattern)` — escape special chars
- `re.sub(pattern, repl, string, count)` — with count limit
- `re.subn(pattern, repl, string)` — sub + replacement count

**Missing (needs classes):**
- `re.compile(pattern)` — compiled pattern object
- Match object: `.group()`, `.groups()`, `.start()`, `.end()`, `.span()`
- Flags: `re.IGNORECASE`, `re.MULTILINE`, `re.DOTALL`

**API naming:** `re_match` → `match`, `re_find` → `search`, `re_replace` → `sub`, `re_findall` → `findall`, `re_split` → `split`

---

### 5. `sifr.json` — ~60% coverage

**Has:** `json_loads`, `json_dumps`

**Missing (achievable):**
- `json.load(fp)` — load from file path (like `tomllib.load`)
- `json.dump(obj, fp)` — dump to file path

**Missing (needs advanced features):**
- `indent` parameter for pretty-printing
- `sort_keys` parameter
- Custom encoder/decoder
- `JSONDecodeError` exception

**API naming:** `json_loads` → `loads`, `json_dumps` → `dumps`

---

### 6. `sifr.time` — ~45% coverage

**Has:** `time_now`, `sleep`, `time_format`, `perf_counter`, `monotonic`

**Missing (intrinsic-feasible):**
- `time_ns()` — nanosecond epoch (Rust: `SystemTime` → `.as_nanos()`)
- `perf_counter_ns()` — nanosecond perf counter
- `strptime(string, format)` — parse time string (Rust: `chrono::NaiveDateTime::parse_from_str`)

**Missing (needs libc):**
- `process_time()` — CPU time
- `thread_time()` — thread CPU time

**Missing (needs struct/tuple returns):**
- `gmtime()`, `localtime()` — struct_time
- `mktime(t)` — inverse of localtime

**API naming:** `time_now` → `time`, `time_format` → `strftime`

---

### 7. `sifr.hashlib` — ~40% coverage

**Has:** `sha256`, `md5`

**Missing (intrinsic-feasible, Rust crates exist):**
- `sha1(data)` — `sha1` crate
- `sha384(data)`, `sha512(data)` — `sha2` crate (already a dependency)
- `blake2b(data)`, `blake2s(data)` — `blake2` crate

**Missing (needs classes):**
- Hash object with `.update()`, `.digest()`, `.hexdigest()`
- `new(name, data)` — generic constructor
- `pbkdf2_hmac()` — key derivation

---

### 8. `sifr.base64` — ~50% coverage

**Has:** `base64_encode`, `base64_decode`

**Missing (intrinsic-feasible):**
- `urlsafe_b64encode(s)` / `urlsafe_b64decode(s)` — Rust: `base64::engine::general_purpose::URL_SAFE`
- `b32encode(s)` / `b32decode(s)`
- `b16encode(s)` / `b16decode(s)` — hex encoding

**API naming:** `base64_encode` → `b64encode`, `base64_decode` → `b64decode`

---

### 9. `sifr.random` — ~35% coverage

**Has:** `random_int`, `random_float`, `random_choice`, `random_uniform`

**Missing (intrinsic-feasible):**
- `seed(n)` — Rust: `StdRng::seed_from_u64`
- `shuffle(list)` — Rust: `SliceRandom::shuffle` (ownership challenge)
- `sample(population, k)` — random sample without replacement
- `randrange(start, stop, step)` — with step
- `randbytes(n)` — random bytes
- `gauss(mu, sigma)` — Rust: `rand_distr::Normal`

**API naming:** `random_int` → `randint`, `random_float` → `random`, `random_choice` → `choice`, `random_uniform` → `uniform`

---

### 10. `sifr.bytes` — Custom module (no direct CPython equivalent)

**Has:** `encode_utf8`, `decode_utf8`, `bytes_to_hex`, `bytes_from_hex`

No direct CPython module to compare against. This is a pragmatic Sifr-specific utility module.

---

### 11. `sifr.collections` — ~25% coverage

**Has:** `new_set`, `set_from_list`, `set_add`, `set_contains`, `set_remove`, `set_len`, `set_union`, `set_intersection`, `counter_from_list`, `counter_get`, `counter_most_common`, `defaultdict_new`, `defaultdict_get`, `defaultdict_set`

**Missing (needs classes):**
- `Counter` class — with arithmetic operators, `.elements()`, `.most_common()`, `.subtract()`, `.total()`
- `defaultdict` class — with `__missing__`
- `deque` class — double-ended queue
- `OrderedDict` class
- `ChainMap` class
- `namedtuple(typename, field_names)` — factory function
- `UserDict`, `UserList`, `UserString`

---

### 12. `sifr.env` — Custom module (~50% of `os.environ`)

**Has:** `env_get`, `env_set`

**Missing:**
- `unsetenv(key)` — remove env var
- Dict-like `.keys()`, `.values()`, `.items()` interface

---

### 13. `sifr.string` — ~60% coverage

**Has:** `ascii_lowercase`, `ascii_uppercase`, `ascii_letters`, `digits`, `hexdigits`, `octdigits`, `punctuation`, `whitespace`

**Missing (pure Sifr):**
- `printable` constant — all printable characters
- `capwords(s)` — capitalize words

**Missing (needs classes):**
- `Template` class — string substitution
- `Formatter` class — custom formatting

---

### 14. `sifr.statistics` — ~50% coverage

**Has:** `mean`, `median`, `variance`, `stdev`, `mode`

**BUG: `variance` computes population variance (÷N). CPython's `variance` computes sample variance (÷N-1).**

**Missing (pure Sifr, easy):**
- `pvariance(data)` — population variance (what current `variance` actually does)
- `pstdev(data)` — population stdev
- Fix `variance` to use N-1 (sample variance)
- Fix `stdev` to use corrected `variance`
- `fmean(data)` — fast float mean (same as current `mean`)
- `harmonic_mean(data)` — `n / sum(1/x for x in data)`
- `geometric_mean(data)` — `exp(mean(log(x) for x in data))`
- `median_low(data)`, `median_high(data)` — low/high median
- `multimode(data)` — all modes, not just first
- `quantiles(data, n)` — quantile boundaries

**Missing (needs classes):**
- `NormalDist` class
- `covariance(x, y)`, `correlation(x, y)`, `linear_regression(x, y)` — need paired iteration

---

### 15. `sifr.bisect` — ~75% coverage

**Has:** `bisect_left`, `bisect_right`, `insort_left`

**Missing (pure Sifr):**
- `insort_right(a, x)` — insert at right position
- `insort(a, x)` — alias for `insort_right`

**Missing (needs generics):**
- `key` parameter on all functions
- `lo`/`hi` range parameters
- Generic type support (currently `list[int]` only)

---

### 16. `sifr.functools` — ~15% coverage (NOT CPython parity)

**Has:** `identity`, `clamp`

**Note:** Neither `identity` nor `clamp` exist in CPython's `functools`. This module has **zero** CPython parity.

**Missing (needs Callable support):**
- `reduce(function, iterable, initial)` — the core function (needs `Callable`)
- `partial(func, *args)` — partial application (needs closures)

**Missing (needs decorators/classes):**
- `lru_cache(maxsize)` — memoization
- `cache` — unbounded cache
- `cached_property` — descriptor
- `wraps(wrapped)` — decorator helper
- `total_ordering` — class decorator
- `singledispatch` — generic dispatch
- `cmp_to_key(func)` — comparison adapter

---

### 17. `sifr.secrets` — ~40% coverage

**Has:** `token_hex`, `randbelow`

**Missing (pure Sifr / intrinsic):**
- `token_bytes(nbytes)` — random bytes
- `token_urlsafe(nbytes)` — URL-safe base64 token

**Missing (needs advanced features):**
- `compare_digest(a, b)` — constant-time comparison

---

### 18. `sifr.heapq` — ~60% coverage

**Has:** `heapify`, `heappush`, `heappop`, `heappop_rest`, `nsmallest`, `nlargest`

**Missing (pure Sifr):**
- `heapreplace(heap, item)` — pop + push
- `heappushpop(heap, item)` — push + pop

**Missing (needs generics):**
- Generic type support (currently `list[int]` only)
- `merge(*iterables)` — merge sorted iterables (needs iterators)

**API note:** `heappop_rest` is Sifr-specific (not in CPython). CPython's `heappop` mutates in-place; Sifr returns new lists.

---

### 19. `sifr.itertools` — ~20% coverage (mostly non-CPython functions)

**Has:** `chain`, `chain_str`, `repeat_val`, `take`, `flatten`, `enumerate_list`

**Note:** `chain_str`, `repeat_val`, `take`, `flatten`, `enumerate_list` are **not CPython itertools functions**. Only `chain` has a CPython equivalent.

**Missing (needs Callable + generics):**
- `accumulate(iterable, func)` — running totals
- `dropwhile(pred, iterable)` — drop while true
- `takewhile(pred, iterable)` — take while true
- `filterfalse(pred, iterable)` — inverse filter
- `starmap(func, iterable)` — map with unpacked args
- `groupby(iterable, key)` — group consecutive

**Missing (pure Sifr, int-only feasible):**
- `product(a, b)` — Cartesian product of two lists
- `permutations(data, r)` — permutations
- `combinations(data, r)` — combinations
- `zip_longest(a, b, fillvalue)` — zip with fill
- `pairwise(data)` — consecutive pairs
- `batched(data, n)` — batch into groups
- `islice(data, start, stop)` — slice

---

### 20. `sifr.textwrap` — ~60% coverage

**Has:** `wrap`, `fill`, `dedent`, `indent`

**Missing (pure Sifr):**
- `shorten(text, width)` — truncate with placeholder ("...")

**Missing (needs classes):**
- `TextWrapper` class — configurable wrapper

---

### 21. `sifr.csv` — ~30% coverage

**Has:** `parse_row`, `parse_csv`, `format_row`, `format_csv`

**Missing (pure Sifr, significant):**
- Quoted field handling — fields with commas, quotes, newlines
- Custom delimiter support
- Header row parsing

**Missing (needs classes):**
- `csv.reader(file)` / `csv.writer(file)` — file-based
- `DictReader` / `DictWriter` — dict-based
- `Sniffer` — dialect detection
- `Dialect` class — configurable parsing

---

### 22. `sifr.argparse` — ~15% coverage (functional alternative)

**Has:** `parse_flag`, `parse_option`, `parse_positional`

**Missing (needs classes — entire CPython API):**
- `ArgumentParser` class
- `.add_argument()` with type, choices, default, required, help, nargs, action
- `.parse_args()` → Namespace
- Subcommands, mutually exclusive groups
- Help/usage text generation
- Type conversion and validation

---

### 23. `sifr.fnmatch` — ~50% coverage

**Has:** `fnmatch`, `fnmatch_filter`

**Missing (pure Sifr):**
- `fnmatchcase(name, pat)` — case-sensitive match
- Character class support (`[abc]`, `[!abc]`, `[a-z]`) in patterns

**Missing (needs re integration):**
- `translate(pat)` — convert glob pattern to regex

**API naming:** `fnmatch_filter` → `filter`

---

### 24. `sifr.glob` — ~20% coverage

**Has:** `glob(directory, pattern)`

**Missing (significant):**
- Single-argument `glob(pathname)` — e.g., `glob("/tmp/*.txt")` with embedded wildcards
- `iglob(pathname)` — lazy iterator
- `recursive=True` / `**` pattern
- `root_dir` parameter
- `escape(pathname)` — escape special chars
- `translate(pat)` — to regex

**API difference:** Sifr takes `(directory, pattern)` separately; CPython takes single pathname.

---

### 25. `sifr.shutil` — ~20% coverage

**Has:** `copy`, `move_file`, `rmtree`

**Missing (intrinsic-feasible):**
- `copy2(src, dst)` — copy with metadata (Rust: `std::fs::copy` + metadata)
- `copytree(src, dst)` — recursive copy
- `which(cmd)` — find executable on PATH
- `disk_usage(path)` — disk space info

**Missing (needs classes/advanced):**
- Archive operations (`make_archive`, `unpack_archive`)
- `get_terminal_size()`
- `chown(path, user, group)`

**API naming:** `move_file` should be `move` (blocked by Rust keyword)

---

### 26. `sifr.tempfile` — ~25% coverage

**Has:** `mktemp_path`, `mkstemp`, `mkdtemp`

**Missing (pure Sifr):**
- `gettempdir()` — return temp directory path
- `suffix`/`dir` parameters on `mkstemp`/`mkdtemp`

**Missing (needs classes):**
- `NamedTemporaryFile` — auto-cleanup file
- `TemporaryDirectory` — auto-cleanup directory
- `SpooledTemporaryFile` — in-memory until threshold
- `TemporaryFile` — unnamed temp file

---

### 27. `sifr.graphlib` — ~30% coverage

**Has:** `topological_sort(num_nodes, from_nodes, to_nodes)`

**Missing (needs classes):**
- `TopologicalSorter` class — `.add()`, `.prepare()`, `.get_ready()`, `.done()`, `.is_active()`, `.static_order()`
- `CycleError` exception — cycle detection

**API difference:** Sifr uses flat function with parallel int arrays; CPython uses class with node objects.

---

### 28. `sifr.uuid` — ~20% coverage

**Has:** `uuid4`

**Missing (intrinsic-feasible):**
- `uuid1()` — time-based (Rust: `uuid` crate)
- `uuid3(namespace, name)` — MD5-based
- `uuid5(namespace, name)` — SHA1-based

**Missing (needs classes):**
- `UUID` class — `.hex`, `.int`, `.urn`, `.version`, `.variant`, `.bytes`
- Namespace constants: `NAMESPACE_DNS`, `NAMESPACE_URL`, etc.

---

### 29. `sifr.platform` — ~20% coverage

**Has:** `platform_system`, `platform_arch`

**Missing (intrinsic-feasible):**
- `node()` — hostname (Rust: `hostname` crate or `gethostname`)
- `release()` — OS release (Rust: `uname` or `sysinfo`)
- `version()` — OS version
- `processor()` — processor name
- `platform()` — full platform string
- `python_version()` → `sifr_version()` — compiler version

**API naming:** `platform_system` → `system`, `platform_arch` → `machine`

---

### 30. `sifr.pathlib` — ~15% coverage

**Has:** `join_path`, `basename`, `dirname`, `extension`

**Missing (pure Sifr):**
- `stem(path)` — filename without extension
- `splitext(path)` — split into (root, ext)
- `is_absolute(path)` — check if absolute
- `normalize(path)` — normalize separators

**Missing (needs classes — entire CPython API):**
- `Path` class — `.exists()`, `.is_file()`, `.is_dir()`, `.mkdir()`, `.rmdir()`, `.unlink()`, `.read_text()`, `.write_text()`, `.glob()`, `.rglob()`, `.iterdir()`, `.resolve()`, `.parent`, `.name`, `.stem`, `.suffix`, `.parts`, `.with_name()`, `.with_suffix()`, `/` operator

---

### 31. `sifr.logging` — ~15% coverage

**Has:** `log_info`, `log_warn`, `log_error`, `log_debug`

**Missing (needs classes — entire CPython API):**
- `getLogger(name)` — named loggers
- `Logger` class — `.info()`, `.warning()`, `.error()`, `.debug()`, `.critical()`
- `Handler` classes — `StreamHandler`, `FileHandler`, `RotatingFileHandler`
- `Formatter` class — custom formatting
- `Filter` class — filtering
- Log levels: `DEBUG`, `INFO`, `WARNING`, `ERROR`, `CRITICAL`
- `basicConfig()` — quick setup
- Hierarchical logger names

---

### 32. `sifr.difflib` — ~20% coverage

**Has:** `get_close_matches`, `unified_diff`

**Note:** `_similarity` uses simple character matching, not the SequenceMatcher (Ratcliff/Obershelp) algorithm.

**Missing (pure Sifr, significant):**
- `context_diff(a, b)` — context format
- Proper SequenceMatcher algorithm for `_similarity`

**Missing (needs classes):**
- `SequenceMatcher` class — `.ratio()`, `.get_matching_blocks()`, `.get_opcodes()`
- `Differ` class — human-readable diffs
- `HtmlDiff` class — HTML diff tables

---

### 33. `sifr.ipaddress` — ~25% coverage

**Has:** `is_valid_ipv4`, `ip_to_int`, `is_private`, `is_loopback`

**Missing (pure Sifr):**
- `int_to_ip(n)` — reverse of `ip_to_int`
- `is_multicast(addr)` — multicast check
- `is_reserved(addr)` — reserved range check
- `is_global(addr)` — global unicast check

**Missing (needs classes):**
- `IPv4Address` class, `IPv6Address` class
- `IPv4Network` class — CIDR notation, `.hosts()`, `.subnets()`
- `ip_address(addr)` — factory function
- `ip_network(addr)` — network factory
- All IPv6 support

---

### 34. `sifr.timeit` — ~60% coverage

**Has:** `default_timer`, `timeit`, `repeat`

**Missing (needs classes):**
- `Timer` class — `.timeit(number)`, `.repeat(repeat, number)`, `.autorange()`

**Note:** Functional API covers core use cases well. `Timer` class deferred due to `Callable`-as-struct-field codegen issue.

---

### 35. `sifr.tomllib` — ~50% coverage

**Has:** `loads`, `load`

**Missing:**
- `TOMLDecodeError` exception
- `parse_float` parameter
- Returns `str` (JSON representation) instead of `dict[str, Any]` — significant semantic difference from CPython

---

### 36. `sifr.datetime` — ~15% coverage

**Has:** `now`, `format_datetime`, `from_timestamp`

**Missing (needs classes — almost entire CPython API):**
- `datetime` class — `.year`, `.month`, `.day`, `.hour`, `.minute`, `.second`, `.microsecond`, `.strftime()`, `.strptime()`, `.isoformat()`, `.fromisoformat()`, `.timestamp()`, `.replace()`, arithmetic
- `date` class — date-only
- `time` class — time-only
- `timedelta` class — duration arithmetic (`datetime - datetime`, `datetime + timedelta`)
- `timezone` class — timezone info

---

### 37. `sifr.test` — Custom module (not targeting CPython `unittest`)

**Has:** `assert_eq`, `assert_ne`, `assert_true`, `assert_false`

Not intended to match CPython's `unittest`. This is a Sifr-specific test utility.

---

## Quick-Win Opportunities (No Compiler Changes Needed)

These are pure-Sifr functions that can be added immediately:

| Module | Functions to Add | Effort |
|---|---|---|
| `math` | `factorial`, `gcd`, `lcm`, `comb`, `perm`, `isclose`, `prod` | Low |
| `statistics` | Fix `variance`/`stdev` (N-1), add `pvariance`, `pstdev`, `harmonic_mean`, `geometric_mean`, `median_low`, `median_high`, `multimode`, `fmean` | Low |
| `bisect` | `insort_right` | Low |
| `string` | `printable`, `capwords` | Low |
| `textwrap` | `shorten` | Low |
| `pathlib` | `stem`, `splitext`, `is_absolute` | Low |
| `secrets` | `token_urlsafe` (using base64_encode) | Low |
| `heapq` | `heapreplace`, `heappushpop` | Low |
| `itertools` | `pairwise`, `batched`, `islice`, `zip_longest` (int-only) | Medium |
| `fnmatch` | `fnmatchcase` | Low |
| `ipaddress` | `int_to_ip`, `is_multicast`, `is_global` | Low |
| `csv` | Quoted field support | Medium |
| `difflib` | `context_diff` | Medium |
| `tempfile` | `gettempdir` | Low |

## Intrinsic Additions (Need Rust Codegen)

| Module | Intrinsics to Add | Rust Implementation |
|---|---|---|
| `math` | `exp`, `expm1`, `log1p`, `fabs`, `isfinite` | `f64::exp()`, `f64::exp_m1()`, `f64::ln_1p()`, `f64::abs()`, `f64::is_finite()` |
| `hashlib` | `sha1`, `sha384`, `sha512` | `sha1`/`sha2` crates |
| `base64` | `urlsafe_b64encode`, `urlsafe_b64decode` | `base64::engine::general_purpose::URL_SAFE` |
| `random` | `seed`, `shuffle`, `sample`, `randbytes` | `StdRng`, `SliceRandom` |
| `platform` | `node`, `release`, `version` | `gethostname`, `uname` |
| `os` | `makedirs`, `chdir`, `getpid` | `std::fs::create_dir_all`, `std::env::set_current_dir`, `std::process::id` |
| `time` | `time_ns`, `strptime` | `SystemTime::as_nanos()`, `chrono::parse_from_str` |
| `json` | `json_load`, `json_dump` | Read file + `serde_json` |
| `uuid` | `uuid1`, `uuid3`, `uuid5` | `uuid` crate features |

## Blocked by Language Features

| Feature Needed | Modules Blocked | Status |
|---|---|---|
| Classes in stdlib `.sifr` files | argparse, csv, logging, pathlib, graphlib, uuid, collections, datetime, re, tempfile, difflib | Infrastructure exists but `Callable`-as-struct-field needs `Box<dyn Fn>` fix |
| Generics | bisect, heapq, itertools | Generics milestone not yet started |
| Iterator protocol | itertools, csv, glob | Not planned yet |
| Exception types | tomllib, json, graphlib, ipaddress | Exception support needed |
| Context managers (`with`) | io, tempfile | `with` statement not implemented |
| Tuple returns | math (`frexp`, `modf`), os (`walk`) | Tuple type exists but multi-value return patterns limited |
