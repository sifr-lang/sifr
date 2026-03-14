# Sifr Stdlib vs CPython: Module-by-Module Audit

Date: 2026-03-14

## Scope

This audit re-checks parity against the local CPython source tree at:

- `/Users/yaseralnajjar/work/sifr/cpython`

It is intentionally current-repo-state oriented. The goal is not to repeat the older gap audits, but to verify which parity gaps still exist after the recent Phase 30 and Phase 31 work.

## Method

- Sifr surface checked from `lib/sifr/*.sifr`
- CPython pure-Python module references checked from `Lib/<module>.py` or `Lib/<package>/__init__.py`
- C-backed CPython surfaces checked against these source roots, with public-name spot checks from the local Python runtime:
  - `Modules/mathmodule.c`
  - `Modules/itertoolsmodule.c`
  - `Modules/timemodule.c`
  - `Python/sysmodule.c`
  - `Python/bltinmodule.c`

## High-Level Conclusion

The repo is no longer missing stdlib modules in the broad sense. The main gap is now Python-shaped surface parity:

- missing builtin constructor parity
- missing class/object-model parity
- missing optional-argument parity
- missing structured return-model parity
- workaround-first APIs still standing in for natural CPython-shaped entry surfaces

Several modules are materially stronger than the older audits suggested:

- `math`
- `random`
- `re`
- `statistics`
- `datetime`
- `hashlib`
- `collections`

The weakest parity areas are now:

- builtin constructors and builtins
- class-heavy modules
- configuration/structured-data modules
- iterator/object-model modules
- process/runtime wrapper modules

## Builtins Cross-Check

CPython builtin reference:

- `/Users/yaseralnajjar/work/sifr/cpython/Python/bltinmodule.c`

Present today in lowering:

- `len`
- `abs`
- `min`
- `max`
- `sum`
- `sorted`
- `reversed`
- `enumerate`
- `zip`
- `map`
- `range`
- `any`
- `all`
- `str`
- `int`
- `float`
- `bool`
- `set`

Still missing or still not parity-complete:

- `list(...)` constructor parity is missing
- `tuple(...)` constructor parity is missing
- `dict(...)` constructor parity is missing
- `ord(...)` is missing
- `chr(...)` is missing
- `sorted(...)` lacks CPython option parity such as `key=` and `reverse=`
- `enumerate(...)` lacks `start`
- `zip(...)` is still fixed-shape rather than variadic
- `map(...)` is still eager and effectively single-iterable

## Module-by-Module Audit

Legend:

- `strong subset`: substantial useful parity, but still incomplete
- `medium subset`: meaningful surface exists, but large CPython gaps remain
- `minimal`: only a small slice of CPython is present
- `custom`: no direct 1:1 CPython module counterpart

| module | CPython reference | status | current Sifr surface | main missing parity |
| --- | --- | --- | --- | --- |
| `argparse` | `Lib/argparse.py` | `minimal` | low-level parsing helpers only | missing `ArgumentParser`, `Namespace`, action classes, formatter classes, error types, and the whole CPython object model |
| `base64` | `Lib/base64.py` | `medium subset` | strong base64/base32/base16 subset with aliases | missing ascii85/base85/z85 families, top-level file-style `encode`/`decode`, and full bytes-native parity |
| `bisect` | `Lib/bisect.py` | `medium subset` | `bisect_left`, `bisect_right`, `insort_left`, `insort_right` | missing `bisect` / `insort` aliases and CPython optional args `lo`, `hi`, `key` |
| `bytes` | no direct module; compare against CPython `bytes` object behavior | `custom` | UTF-8/hex helpers and byte-search helpers | not a CPython module; current surface is utility-style and does not provide full CPython `bytes` / `bytearray` object-model parity |
| `calendar` | `Lib/calendar.py` | `minimal` | `isleap`, `weekday`, `monthrange` | missing constants, month/day names, `leapdays`, formatting helpers, `timegm`, and the `Calendar` / `TextCalendar` / `HTMLCalendar` class family |
| `collections` | `Lib/collections/__init__.py` | `medium subset` | `Counter`, `deque`, low-level set helpers, limited `defaultdict` compat | missing real `defaultdict` class/object model, `OrderedDict`, `ChainMap`, `namedtuple`, `UserDict`, `UserList`, `UserString`, and natural `Counter(...)` constructor parity |
| `configparser` | `Lib/configparser.py` | `minimal` | one basic `ConfigParser` class | missing error hierarchy, interpolation modes, `RawConfigParser`, `SectionProxy`, constants, and broader INI semantics |
| `csv` | `Lib/csv.py` | `medium subset` | `reader`, `writer`, `DictReader`, `DictWriter`, path/file helpers | missing dialect registration, quote constants, `Sniffer`, `field_size_limit`, richer quoting/escaping semantics, and the broader dialect object model |
| `datetime` | `Lib/datetime.py` | `strong subset` | `timedelta`, `datetime`, `date`, `time`, `timezone`, `now`, `today` | missing `tzinfo`, `MINYEAR`, `MAXYEAR`, uppercase `UTC`, broader aware/naive semantics, richer constructors/classmethods, and `from_timestamp` currently returns `Result[str, ValueError]` instead of a `datetime` object |
| `difflib` | `Lib/difflib.py` | `minimal` | `get_close_matches`, `unified_diff` | missing `SequenceMatcher`, `Differ`, `HtmlDiff`, `context_diff`, `ndiff`, `restore`, and `diff_bytes` |
| `env` | no direct module; closest CPython surface is `os.environ` / `os.getenv` | `custom` | env get/set/unset/keys/values/items | not a CPython module; parity target should really be folded into `os` / `sys` environment semantics |
| `fnmatch` | `Lib/fnmatch.py` | `medium subset` | `fnmatch`, `fnmatchcase`, `filter` | missing `translate`, `filterfalse`, and fuller CPython pattern semantics and platform-normalization behavior |
| `functools` | `Lib/functools.py` | `minimal` | `reduce` only | missing `partial`, `wraps`, `lru_cache`, `cache`, `cmp_to_key`, `total_ordering`, `singledispatch`, `cached_property`, and the rest of the public surface |
| `glob` | `Lib/glob.py` | `minimal` | one deterministic `glob` helper | missing `iglob`, `escape`, `translate`, recursive/full-path semantics, and the full CPython pathname-expansion signature |
| `graphlib` | `Lib/graphlib.py` | `medium subset` | `CycleError`, `TopologicalSorter`, `topological_sort` | missing `prepare`, `get_ready`, `done`, and `is_active` parity on `TopologicalSorter` |
| `gzip` | `Lib/gzip.py` | `minimal` | `compress`, `decompress` | missing `GzipFile`, `open`, and `BadGzipFile` |
| `hashlib` | `Lib/hashlib.py` plus C-backed constructors | `strong subset` | constructors, `new`, `HashObject`, algorithm lists, `file_digest` | top-level constructor coverage is much stronger now, but bytes-native digest/object semantics, shake-specific parity, and broader CPython crypto helpers still need review |
| `heapq` | `Lib/heapq.py` | `medium subset` | `heapify`, `heappush`, `heappop`, `heapreplace`, `heappushpop`, `nsmallest`, `nlargest` | missing `merge`, public max-heap helpers, and some helper semantics still diverge from CPython mutation/error behavior |
| `html` | `Lib/html/__init__.py` | `strong subset` | `escape`, `unescape` | top-level `html` parity is relatively close; main remaining gaps are in sibling modules like `html.parser` / `html.entities`, not the top-level module itself |
| `io` | `Lib/io.py` | `minimal` | `open`, `FileHandle`, text/binary/path helpers | missing the `IOBase` hierarchy, `BytesIO`, `StringIO`, buffering types, seek/tell constants, `open_code`, and full stream object semantics |
| `ipaddress` | `Lib/ipaddress.py` | `minimal` | IPv4 utility helpers only | missing `ip_address`, `ip_network`, `ip_interface`, IPv4/IPv6 address/network/interface classes, error types, and packed-address helpers |
| `itertools` | `Modules/itertoolsmodule.c` | `medium subset` | strong eager helper subset including `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `cycle` | missing `count` proper, `groupby`, `product`, `permutations`, `combinations`, `combinations_with_replacement`, `starmap`, `tee`, and lazy iterator object-model parity |
| `json` | `Lib/json/__init__.py` | `minimal` | `loads` alias plus `json_dumps` intrinsic path | missing `load`, `dump`, `dumps` as a natural top-level parity surface, `JSONEncoder`, `JSONDecoder`, and structured object returns; `loads` currently returns `Result[str, JSONDecodeError]` instead of typed JSON values |
| `logging` | `Lib/logging/__init__.py` | `medium subset` | `Logger`, `Formatter`, `FileHandler`, `basicConfig`, `getLogger`, level constants | missing large parts of the framework: `Handler`, `Filter`, `LogRecord`, `LoggerAdapter`, `StreamHandler`, `NullHandler`, root-helper functions, configuration/state helpers, and broader logger hierarchy behavior |
| `math` | `Modules/mathmodule.c` | `strong subset` | very broad math surface including newer numeric helpers | one of the strongest modules; remaining parity gaps are mostly result-shape or naming details such as list-pair adapters for `frexp` / `modf`, plus the usual safety-divergence decisions rather than obvious missing top-level functions |
| `operator` | `Lib/operator.py` | `minimal` | narrow numeric/comparison subset plus `itemgetter` | missing `attrgetter`, `methodcaller`, `contains`, `getitem`/`setitem`, truthiness helpers, bitwise/in-place helpers, `truediv`, and broader operator parity; still has naming divergence `mod_val` |
| `os` | `Lib/os.py` plus platform-backed runtime surfaces | `medium subset` | cwd/listdir/mkdir/rmdir/remove/rename/file checks/chdir/pid/cpu/stat/which/disk_usage/constants | strong wrapper subset exists, but missing `path` parity, environment parity at `os` level, many constants, encoding helpers, fd/file helpers, process-group APIs, exec/fork families, and broader path/process surface; `run_command` and `get_args` remain non-CPython-shaped |
| `pathlib` | `Lib/pathlib/__init__.py` | `medium subset` | `Path` class with many filesystem helpers | missing `PurePath` hierarchy, `PosixPath` / `WindowsPath`, unsupported-operation types, and full platform-specific path semantics |
| `platform` | `Lib/platform.py` | `minimal` | `system`, `machine`, `node`, `release`, `version`, `processor` | missing `uname`, `platform()`, `architecture`, Python-version/build helpers, cache invalidation helpers, and vendor/platform-specific probes |
| `random` | `Lib/random.py` | `medium subset` | `randint`, `random`, `uniform`, `shuffle`, `sample`, `randrange`, `gauss`, `choice` | much better than older audits, but still missing `seed`/state APIs, `randbytes`, class-based `Random` / `SystemRandom`, and most non-core distributions |
| `re` | `Lib/re/__init__.py` | `strong subset` | `Pattern`, `Match`, flags, `search`, `sub`, `findall`, `split`, `compile`, `fullmatch` | missing `match`, `subn`, `finditer`, `escape`, `purge`, full flag alias surface, and broader regex-engine parity edges |
| `secrets` | `Lib/secrets.py` | `minimal` | `token_hex`, `randbelow` | missing `choice`, `randbits`, `token_bytes`, `token_urlsafe`, `compare_digest`, and `SystemRandom` |
| `shutil` | `Lib/shutil.py` | `minimal` | `copy`, `move_file`, `rmtree`, `which`, `disk_usage` | missing `copyfile`, `copyfileobj`, `copy2`, `copytree`, archive/unpack APIs, error classes, `chown`, and the natural `move` name |
| `statistics` | `Lib/statistics.py` | `strong subset` | `mean`, `median`, `variance`, `pvariance`, `stdev`, `pstdev`, `fmean`, `harmonic_mean`, `geometric_mean`, `mode`, `multimode`, `quantiles`, `covariance`, `correlation`, `linear_regression` | one of the stronger modules; remaining gaps are `median_grouped`, `NormalDist`, and newer CPython KDE helpers |
| `string` | `Lib/string/__init__.py` | `medium subset` | constants plus `capwords` | missing `Template` and `Formatter` classes |
| `subprocess` | `Lib/subprocess.py` | `minimal` | `CompletedProcess`, `run`, raw/input helpers | missing `Popen`, `PIPE`, `STDOUT`, `DEVNULL`, error types, `check_call`, `check_output`, `call`, `getoutput`, `getstatusoutput`, timeout/process lifecycle parity |
| `sys` | `Python/sysmodule.c` | `minimal` | `argv`, `exit`, `version`, `platform`, `maxsize` | missing streams, path/module metadata, `version_info`, `implementation`, hooks, flags, runtime configuration APIs, and the broader interpreter-state surface |
| `tempfile` | `Lib/tempfile.py` | `minimal` | `mktemp_path`, `mkstemp`, `mkdtemp` | missing `TemporaryFile`, `NamedTemporaryFile`, `TemporaryDirectory`, `SpooledTemporaryFile`, temp-prefix/tempdir helpers, and broader lifecycle/object semantics |
| `test` | `Lib/test/__init__.py` is not the relevant parity target | `custom` | assertion helpers for Sifr tests | this is Sifr-specific infrastructure, not a meaningful CPython stdlib parity target |
| `textwrap` | `Lib/textwrap.py` | `medium subset` | `wrap`, `fill`, `dedent`, `indent`, `shorten` | missing `TextWrapper` class and full option parity |
| `time` | `Modules/timemodule.c` | `medium subset` | `time`, `sleep`, `perf_counter`, `monotonic`, `strftime`, `strptime`, `gmtime`, `localtime` | missing ns variants, `mktime`, process/thread clocks, `struct_time`, timezone constants, clock-get/set APIs, and broader platform time surface |
| `timeit` | `Lib/timeit.py` | `medium subset` | `default_timer`, `timeit`, `repeat` | missing `Timer` class parity |
| `tomllib` | `Lib/tomllib/__init__.py` | `minimal` | `loads`, `load` | missing `TOMLDecodeError` export and structured TOML returns; `load()` currently converts TOML decode failures into `IOError`, which is not parity-aligned |
| `uuid` | `Lib/uuid.py` | `minimal` | `UUID`, `uuid4`, `uuid_from_hex` | missing `SafeUUID`, `getnode`, `uuid1`, `uuid3`, `uuid5`, newer `uuid6/7/8`, and richer constructor overload parity |
| `zipfile` | `Lib/zipfile/__init__.py` | `minimal` | basic `ZipFile` class with create/write/read/namelist | missing `is_zipfile`, `ZipInfo`, `Path`, compression constants, `BadZipFile` / `LargeZipFile`, and broader archive-mode semantics |

## Modules With No Direct CPython 1:1 Counterpart

These should not be audited as ordinary module-parity targets.

- `bytes`
  - parity target is CPython `bytes` / `bytearray` object model, not a `bytes` module
- `env`
  - parity target is `os.environ` / `os.getenv` / environment APIs, not a standalone CPython module
- `test`
  - this is Sifr test infrastructure, not CPython stdlib parity

## Strongest Current Modules

These are the least urgent for a parity phase because they already have substantial surface area:

- `math`
- `statistics`
- `re`
- `datetime`
- `hashlib`

Even here, the remaining gaps are still real:

- constructor/call-shape parity
- object-model return-shape parity
- explicit optional-argument parity
- remaining class/object compatibility cleanup

## Weakest Current Modules

These are the clearest candidates for explicit parity work packages:

- `argparse`
- `functools`
- `json`
- `io`
- `ipaddress`
- `operator`
- `secrets`
- `subprocess`
- `sys`
- `tempfile`
- `zipfile`

## Root-Cause Buckets Confirmed By This Audit

This re-check confirms the current parity problem is not "we forgot to add stdlib modules." The dominant missing layers are:

1. builtin constructor parity
2. class/object-model parity
3. optional-argument parity
4. structured return-type parity
5. iterator/lazy object-model parity
6. wrapper cleanup where Sifr-specific helper names still stand in for natural CPython entry surfaces

## Recommended Planning Implication

This audit supports the ad hoc phase plan in:

- `/Users/yaseralnajjar/.codex/worktrees/9e99/codebase/issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface.md`

The right next planning unit is still:

- builtin constructors/conversions
- builtin functional helper parity
- core type object-model parity
- collections constructor parity
- existing-module Python-surface cleanup

That is a better execution model than continuing to rediscover the same source-surface gaps through problem corpora.
