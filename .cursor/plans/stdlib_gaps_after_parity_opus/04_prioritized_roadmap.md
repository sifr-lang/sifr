# Prioritized Roadmap

Recommended order for closing stdlib gaps, organized into work packages.

---

## Work Package 1: Deepen Existing Modules (Highest ROI)

**Goal:** Bring the 37 existing modules from ~35% to ~65% average coverage.
**Effort:** Medium — mostly adding functions to existing modules.
**Blocked by:** Nothing — can start immediately.

### 1A. Quick Wins — Pure Sifr Functions (No New Intrinsics)

These can be implemented entirely in `.sifr` files using existing intrinsics:

| Module | Functions to Add | Effort |
|--------|-----------------|--------|
| `math` | `acosh`, `asinh`, `atanh` (via `log` + `sqrt`), `cbrt` (via `pow`), `isqrt`, `dist`, `fsum` | Small |
| `statistics` | `quantiles`, `multimode`, `covariance`, `correlation`, `linear_regression`, `median_grouped` | Medium |
| `random` | `choice` (expose existing intrinsic), `shuffle`, `sample`, `randrange`, `gauss`, `triangular` | Medium |
| `bisect` | `bisect` alias, `insort` alias | Trivial |
| `functools` | `reduce` (pure Sifr loop), remove `identity`/`clamp` or move to utils | Small |
| `collections` | `Counter.update`, `Counter.subtract`, `Counter.elements`, `Counter.__add__`/`__sub__` | Medium |
| `itertools` | `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `count`, `cycle` | Medium |
| `secrets` | `token_urlsafe`, `token_bytes` | Small |
| `heapq` | `merge` | Small |
| `string` | `Template` class | Medium |

### 1B. New Intrinsics Needed

| Module | Functions to Add | Intrinsic Needed | Rust Implementation |
|--------|-----------------|-----------------|---------------------|
| `math` | `erf`, `erfc`, `gamma`, `lgamma` | `_sifr.math.erf`, etc. | `f64::erf()` (libm) |
| `math` | `frexp`, `ldexp`, `modf` | `_sifr.math.frexp`, etc. | `f64::frexp()` (libm) |
| `math` | `nextafter`, `ulp` | `_sifr.math.nextafter`, etc. | `f64::next_up()` / `f64::next_down()` |
| `os` | `chdir`, `getpid`, `cpu_count`, `stat` | `_sifr.sys.chdir`, etc. | `std::env::set_current_dir`, `std::process::id` |
| `os` | `sep`, `linesep`, `name` | `_sifr.platform.os_sep`, etc. | `std::path::MAIN_SEPARATOR` |
| `hashlib` | `sha224`, `sha384`, `blake2b`, `blake2s` | `_sifr.crypto.sha224`, etc. | `sha2`, `blake2` crates |
| `platform` | `node`, `release`, `version`, `processor` | `_sifr.platform.*` | `hostname`, `uname` |
| `time` | `strptime`, `gmtime`, `localtime` | `_sifr.time.strptime`, etc. | `chrono::NaiveDateTime::parse_from_str` |
| `base64` | `b32encode`, `b32decode`, `b16encode`, `b16decode` | `_sifr.crypto.b32*`, etc. | `base32`, `hex` |
| `shutil` | `which`, `copytree`, `disk_usage` | `_sifr.fs.which`, etc. | `which`, `fs_extra` crates |

### 1C. Class Enhancements

| Module | Class Enhancement | Effort |
|--------|------------------|--------|
| `collections` | `deque` class (double-ended queue) | Medium |
| `datetime` | Full `datetime` class (not just functions returning strings) | Large |
| `datetime` | `date`, `time`, `timezone` classes | Large |
| `pathlib` | Add `resolve`, `glob`, `rglob`, `iterdir`, `unlink`, `rmdir`, `touch`, `with_name`, `with_suffix` | Medium |
| `re` | Add `compile` → `Pattern` class, `match`, `fullmatch`, flags | Medium |
| `logging` | Add `basicConfig`, `FileHandler`, `Formatter`, level constants | Medium |
| `uuid` | Add `uuid1`, `uuid3`, `uuid5` | Small |
| `graphlib` | Add `prepare`, `is_active`, `done`, `get_ready` to `TopologicalSorter` | Small |

---

## Work Package 2: Critical Missing Modules

**Goal:** Add the most commonly needed modules that don't exist yet.
**Effort:** Large — new modules with new intrinsics.
**Blocked by:** Some need async (Phase 8), some need FFI.

### 2A. Can Implement Now

| New Module | CPython Equivalent | Key API | Rust Crate | Effort |
|-----------|-------------------|---------|------------|--------|
| `sifr.subprocess` | `subprocess` | `run(cmd) -> Result[CompletedProcess, Error]`, `Popen` class | `std::process` | Medium |
| `sifr.sys` | `sys` | `argv`, `exit(code)`, `platform`, `version`, `maxsize`, `stdin`/`stdout`/`stderr` | `std::env`, `std::process` | Small |
| `sifr.html` | `html` | `escape(s)`, `unescape(s)` | Pure Sifr or `html-escape` | Small |
| `sifr.configparser` | `configparser` | `ConfigParser` class, `read`, `get`, `set`, `sections` | `configparser` crate or pure Sifr | Medium |
| `sifr.xml` | `xml.etree.ElementTree` | `parse(file)`, `Element` class, `find`, `findall`, `iter` | `quick-xml` | Large |
| `sifr.decimal` | `decimal` | `Decimal` class, arithmetic, `quantize`, `sqrt` | `rust_decimal` | Medium |
| `sifr.fractions` | `fractions` | `Fraction` class, arithmetic, `limit_denominator` | `num-rational` | Medium |
| `sifr.operator` | `operator` | `add`, `sub`, `mul`, `truediv`, `eq`, `lt`, `gt`, `itemgetter`, `attrgetter` | Pure Sifr | Small |
| `sifr.calendar` | `calendar` | `isleap`, `weekday`, `monthrange`, `month`, `calendar` | `chrono` | Medium |
| `sifr.unicodedata` | `unicodedata` | `name`, `category`, `normalize` | `unicode-properties` | Medium |

### 2B. Compression (Can Implement Now)

| New Module | CPython Equivalent | Key API | Rust Crate | Effort |
|-----------|-------------------|---------|------------|--------|
| `sifr.zipfile` | `zipfile` | `ZipFile` class, `read`, `write`, `extractall` | `zip` | Medium |
| `sifr.gzip` | `gzip` | `compress(data)`, `decompress(data)`, `open(file)` | `flate2` | Small |
| `sifr.tarfile` | `tarfile` | `TarFile` class, `open`, `extractall`, `add` | `tar` | Medium |

### 2C. Needs Async Runtime (Phase 8)

| New Module | CPython Equivalent | Key API | Rust Crate |
|-----------|-------------------|---------|------------|
| `sifr.asyncio` | `asyncio` | `run`, `gather`, `sleep`, `create_task`, `Queue` | `tokio` |
| `sifr.socket` | `socket` | `socket` class, `connect`, `bind`, `listen`, `send`, `recv` | `tokio::net` |
| `sifr.http` | `http.client` + `http.server` | `get`, `post`, `Response`, `Server` | `reqwest`, `axum` |
| `sifr.urllib` | `urllib.parse` + `urllib.request` | `urlparse`, `urlencode`, `urlopen` | `url`, `reqwest` |
| `sifr.ssl` | `ssl` | `SSLContext`, `wrap_socket` | `rustls` |
| `sifr.sqlite3` | `sqlite3` | `connect`, `Cursor`, `execute`, `fetchall` | `rusqlite` |

### 2D. Needs Threading Support

| New Module | CPython Equivalent | Key API | Rust Crate |
|-----------|-------------------|---------|------------|
| `sifr.threading` | `threading` | `Thread`, `Lock`, `Event`, `Semaphore`, `Barrier` | `std::thread`, `std::sync` |
| `sifr.queue` | `queue` | `Queue`, `PriorityQueue`, `LifoQueue` | `crossbeam` |
| `sifr.concurrent` | `concurrent.futures` | `ThreadPoolExecutor`, `ProcessPoolExecutor`, `Future` | `rayon`, `tokio` |

---

## Work Package 3: API Alignment Fixes

**Goal:** Fix existing APIs that diverge from CPython conventions without good reason.
**Effort:** Small — mostly renames and re-exports.

| Current Sifr API | Should Be | Reason |
|------------------|-----------|--------|
| `sifr.env.env_get` / `env_set` | `sifr.os.getenv` / `sifr.os.putenv` | CPython uses `os.environ` / `os.getenv` |
| `sifr.functools.identity` / `clamp` | Remove or move to `sifr.utils` | Not CPython functions; confusing |
| `sifr.os.run_command` | `sifr.subprocess.run` | CPython uses `subprocess` for process execution |
| `sifr.os.get_args` | Also expose as `sifr.sys.argv` | CPython uses `sys.argv` |
| `sifr.re.search_match` | `sifr.re.search` (return Match) | CPython's `search` returns Match |
| `sifr.json.loads` returns `str` | Should return structured type | CPython returns dict/list |
| `sifr.datetime.now()` returns `str` | Should return `datetime` object | CPython returns datetime |
| `sifr.tomllib.loads` returns `str` | Should return structured type | CPython returns dict |
| `sifr.csv.parse_csv` | `sifr.csv.reader` | CPython uses `csv.reader` |
| `sifr.shutil.move_file` | `sifr.shutil.move` | CPython uses `shutil.move` |
| `sifr.heapq.heappop_rest` | Remove or document as Sifr extension | Not in CPython |
| `sifr.itertools.chain_str` | Remove (use generic `chain`) | Not in CPython; generics should handle this |
| `sifr.itertools.enumerate_list` | Remove (use built-in `enumerate`) | Not in CPython |
| `sifr.itertools.take` | Rename to match `itertools.islice` | `take` is not a CPython itertools function |

---

## Work Package 4: Error Type Infrastructure

**Goal:** Define and export error types from stdlib modules.
**Effort:** Medium — requires validating the error type export pipeline.
**Blocked by:** Error type export from stdlib `.sifr` files (currently deferred).

| Module | Error Type | CPython Equivalent |
|--------|-----------|-------------------|
| `sifr.json` | `JSONDecodeError` | `json.JSONDecodeError` |
| `sifr.tomllib` | `TOMLDecodeError` | `tomllib.TOMLDecodeError` |
| `sifr.re` | `PatternError` | `re.error` |
| `sifr.csv` | `CSVError` | `csv.Error` |
| `sifr.graphlib` | `CycleError` | `graphlib.CycleError` |
| `sifr.statistics` | `StatisticsError` | `statistics.StatisticsError` |
| `sifr.ipaddress` | `AddressValueError` | `ipaddress.AddressValueError` |
| `sifr.argparse` | `ArgumentError` | `argparse.ArgumentError` |
| `sifr.xml` | `ParseError` | `xml.etree.ElementTree.ParseError` |
| `sifr.sqlite3` | `DatabaseError` | `sqlite3.DatabaseError` |

---

## Recommended Execution Order

```
WP1A (Pure Sifr quick wins)          ← Start here, highest ROI
  ↓
WP3 (API alignment fixes)            ← Fix naming before adding more
  ↓
WP1B (New intrinsics for existing)   ← Deepen existing modules
  ↓
WP1C (Class enhancements)            ← datetime, pathlib, re, collections
  ↓
WP4 (Error type infrastructure)      ← Unblocks proper Result returns
  ↓
WP2A (New modules, no async)         ← subprocess, sys, html, configparser
  ↓
WP2B (Compression modules)           ← zipfile, gzip, tarfile
  ↓
WP2C (Async modules)                 ← After Phase 8 async runtime
  ↓
WP2D (Threading modules)             ← After concurrency support
```
