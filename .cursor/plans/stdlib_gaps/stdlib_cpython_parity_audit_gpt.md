# Sifr Stdlib vs CPython Gap Audit

Date: 2026-02-16

## Scope

- Audited all modules in `lib/sifr` (37 modules).
- Compared exported API shape and implementation level against CPython stdlib modules in `/Users/yaseralnajjar/work/sifr/cpython/Lib`.
- Included milestone polish expectations from `issues/milestone_stdlib_polish.md`.

## Executive Findings

- The stdlib architecture is in place and all 37 modules exist.
- `milestone_stdlib_polish` appears mostly implemented (new intrinsics, `glob`, `timeit`, `tomllib.load`, additional tests).
- Significant CPython parity gaps still remain, mostly due to intentionally simplified/function-first APIs and deferred class-based surfaces.
- One notable naming mismatch still exists: `sifr.shutil` exports `move_file` instead of CPython-style `move`.
- Safety contract work is still incomplete: intrinsic codegen still contains many `.unwrap()` panic paths.

## High-Priority Gaps (Cross-Cutting)

1. API naming parity
   - `sifr.shutil`: `move_file` should be `move` for CPython alignment.
2. Class-based stdlib APIs are still not present
   - `argparse.ArgumentParser`, `logging.Logger`, `pathlib.Path`, `datetime` object model, `graphlib.TopologicalSorter`.
3. Safety contract not yet fully enforced
   - Intrinsic emission in `crates/sifr_codegen/src/lib.rs` still uses many `.unwrap()` and similar panic-prone paths.
4. Function-signature parity gaps
   - Several functions exist but with reduced signatures/defaults compared to CPython (`glob`, `timeit`, `bisect`, etc.).

## Milestone Polish Delta (Current Status)

Largely done:
- `_sifr.time`: `perf_counter`, `monotonic`
- `_sifr.fs`: `copy_file`, `walk_dir`, `rmdir_all`
- `sifr.time`: re-exports monotonic/perf APIs
- `sifr.timeit`: `default_timer`, `timeit`, `repeat`
- `sifr.glob`: exports `glob`
- `sifr.tomllib`: has both `loads` and `load`
- Added pass/fail tests for the new polish surface

Still incomplete vs strict CPython naming:
- `sifr.shutil.move_file` is still not renamed to `move`.

## Module-by-Module Findings

### Tier: Core and Wrappers

- `argparse`
  - Current: `parse_flag`, `parse_option`, `parse_positional`.
  - Missing: `ArgumentParser` class model, `add_argument`, `parse_args`, help/usage formatting, subparsers.

- `base64`
  - Current: intrinsic re-exports `base64_encode`, `base64_decode`.
  - Missing: CPython naming/surface (`b64encode`, `b64decode`, `urlsafe_*`, `b32*`, `b16*`, etc.).

- `bisect`
  - Current: `bisect_left`, `bisect_right`, `insort_left`.
  - Missing: `insort_right`, `insort` alias, key/lo/hi behavior parity.

- `bytes` (custom Sifr module)
  - Current: UTF-8 and hex helpers.
  - Note: no direct one-to-one CPython module equivalent.

- `collections`
  - Current: low-level intrinsic operations for set/counter/defaultdict-like behavior.
  - Missing: CPython public class API (`Counter`, `defaultdict`, `deque`, `OrderedDict`, `ChainMap`, `namedtuple`, etc.).

- `csv`
  - Current: simple comma split/join parser/writer.
  - Missing: quoting/escaping, dialects, `DictReader`, `DictWriter`, sniffer behavior.

- `datetime`
  - Current: string-based wrappers (`now`, `format_datetime`, `from_timestamp`).
  - Missing: class model (`date`, `datetime`, `time`, `timedelta`, `timezone`, `tzinfo`) and typed operations.

- `difflib`
  - Current: `get_close_matches`, simple `_similarity`, basic `unified_diff`.
  - Missing: `SequenceMatcher`, `Differ`, `context_diff`, `ndiff`, `HtmlDiff`, richer algorithmic parity.

- `env` (custom Sifr module)
  - Current: `env_get`, `env_set`.
  - Note: pragmatic wrapper; not a direct CPython module target.

- `fnmatch`
  - Current: basic `fnmatch`, list filter helper.
  - Missing: `fnmatchcase`, `translate`, full shell-pattern semantics.

- `functools`
  - Current: `identity`, `clamp`.
  - Missing: `reduce`, `partial`, `wraps`, `lru_cache`, and most of CPython surface.

- `glob`
  - Current: `glob(directory, pattern)` using `listdir` + `fnmatch`.
  - Missing: `iglob`, recursive `**`, `root_dir`, hidden file controls, `escape`.

- `graphlib`
  - Current: function-style `topological_sort`.
  - Missing: `TopologicalSorter` class and `CycleError`.

- `hashlib`
  - Current: `sha256`, `md5`.
  - Missing: broader algorithms and constructors (`sha1`, `sha512`, `new`, `file_digest`, etc.).

- `heapq`
  - Current: core heap functions plus `nsmallest`/`nlargest`.
  - Missing: `heapreplace`, `heappushpop`, `merge`, full behavioral parity.

- `io`
  - Current: `read_text`, `write_text`, `exists`, `read_lines`, `append_text`.
  - Missing: `open()`/`File` context-manager object model and stream classes.

- `ipaddress`
  - Current: IPv4 validation/int conversion/private/loopback checks.
  - Missing: `ip_address`, `ip_network`, interfaces, IPv6, rich address/network classes.

- `itertools`
  - Current: custom list helpers (`chain`, `repeat_val`, `take`, etc.).
  - Missing: canonical iterator toolbox (`accumulate`, `combinations`, `product`, `groupby`, `zip_longest`, etc.).

- `json`
  - Current: `json_loads`, `json_dumps`.
  - Missing: CPython-style `load`/`dump` variants and advanced encoder/decoder options.

- `logging`
  - Current: print-based functions (`log_info`, `log_warn`, `log_error`, `log_debug`).
  - Missing: logger hierarchy, handlers, formatters, levels, `getLogger`.

- `math`
  - Current: broad wrapper set including trig/inverse/hyperbolic and constants.
  - Missing: many CPython functions still absent (`exp`, `expm1`, `factorial`, `gcd`, `lcm`, `isfinite`, combinatorics, etc.).

- `os`
  - Current: basic command and fs wrappers.
  - Missing: large CPython breadth (path/process/stat/env and many platform APIs).

- `pathlib`
  - Current: function helpers (`join_path`, `basename`, `dirname`, `extension`).
  - Missing: class-based `Path` model and method surface.

- `platform`
  - Current: intrinsic exports (`platform_system`, `platform_arch`).
  - Missing: CPython-style `system`, `machine`, `architecture`, `uname`, and broader surface.

- `random`
  - Current: `random_int`, `random_float`, `random_choice`, `random_uniform`.
  - Missing: `seed`, `randrange`, `shuffle`, `sample`, distributions, class APIs.

- `re`
  - Current: `re_match`, `re_find`, `re_replace`, `re_findall`, `re_split`.
  - Missing: compiled regex object flow, flags/types parity, and complete function set.

- `secrets`
  - Current: `token_hex`, `randbelow`.
  - Missing: `token_bytes`, `token_urlsafe`, `choice`, `randbits`, `compare_digest`.

- `shutil`
  - Current: `copy`, `move_file`, `rmtree`.
  - Missing: `move` naming parity and much of CPython surface (`copy2`, `copytree`, archive helpers, etc.).

- `statistics`
  - Current: `mean`, `median`, `variance`, `stdev`, `mode`.
  - Missing: `fmean`, `multimode`, `median_low/high`, population stats, quantiles, and more.

- `string`
  - Current: key constants (`ascii_*`, `digits`, `hexdigits`, `octdigits`, `punctuation`, `whitespace`).
  - Missing: `printable`, `capwords`, `Template`, `Formatter`.

- `tempfile`
  - Current: `mkstemp`, `mkdtemp`, helper path builder.
  - Missing: `NamedTemporaryFile`, `TemporaryDirectory`, `TemporaryFile`, tempdir helpers and fuller semantics.

- `test` (custom Sifr module)
  - Current: assertion helpers.
  - Note: not intended to match CPython `test` package.

- `textwrap`
  - Current: `wrap`, `fill`, `dedent`, `indent`.
  - Missing: `shorten`, `TextWrapper` class.

- `time`
  - Current: `time_now`, `sleep`, `time_format`, `perf_counter`, `monotonic`.
  - Missing: most of CPython `time` API surface (`gmtime`, `localtime`, `strftime`, process/thread clocks, ns variants).

- `timeit`
  - Current: `default_timer`, `timeit`, `repeat`.
  - Missing: `Timer` class and full CPython signature/default parity.

- `tomllib`
  - Current: `loads`, `load`.
  - Missing: exposed `TOMLDecodeError` parity and typed mapping semantics parity.

- `uuid`
  - Current: `uuid4`.
  - Missing: `UUID` class, namespace constants, and `uuid1/3/5/6/7/8`.

## Suggested Next Focus (If Goal Is Practical Pre-1.0 Parity)

1. Fix remaining naming mismatch (`shutil.move_file` -> `move`).
2. Prioritize class-backed modules (`argparse`, `pathlib`, `logging`, `datetime`) only after class/codegen blockers are addressed.
3. Complete safety-contract pass for intrinsic codegen to eliminate panic paths.
4. Define explicit parity target per module (minimal useful subset vs full CPython surface) to avoid scope churn.

