# Stdlib Deepening

**Why before ecosystem:** The stdlib is at ~38% average parity. Deepening it now means ecosystem features (async, web, db) build on a mature stdlib. This phase also adds critical missing modules that the ecosystem needs.

---

## milestone_stdlib_pure_expansion: Pure Sifr Function Additions

status: completed

**Goal:** Add high-ROI stdlib functions that can be implemented purely in Sifr (no new intrinsics needed). Also clean up non-CPython functions and document API naming divergences.

**Depends on:** milestone_borrow_stdlib (new functions should be written with the final ownership model)

### Fix Existing Return Types (Phase 09 Carryover)

The following existing `statistics` functions were implemented in Phase 09 (`milestone_collection_safety`) with safe defaults (`return 0.0` on empty input) instead of the specified `Result[float, StatisticsError]` return type. Fix them here alongside the new `statistics` additions:

- `statistics.mean`, `statistics.median`, `statistics.variance`, `statistics.stdev`, `statistics.mode`, `statistics.harmonic_mean` -> change return type from `float` to `Result[float, StatisticsError]`, raising `StatisticsError` on empty/invalid input (matching CPython's behavior). `StatisticsError` is already defined and registered in HIR — only the `.sifr` wrapper signatures and return paths need updating.

### New Functions

- `math`: `acosh`, `asinh`, `atanh`, `isqrt`, `dist`, `fsum`
- `statistics`: `quantiles`, `multimode`, `covariance`, `correlation`, `linear_regression`
- `random`: `shuffle`, `sample`, `randrange`, `gauss`
- `functools`: `reduce`
- `collections`: `Counter.update`, `Counter.subtract`, `Counter.elements`, `Counter.__add__`/`__sub__`
- `itertools`: `accumulate`, `compress`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest`, `count`, `cycle`

### Cleanup

Remove any non-CPython functions that were added during Phase 07 for convenience (e.g., extra helpers in `functools` or `itertools` that don't exist in CPython). The stdlib must expose only CPython-equivalent functionality — no invented API surface. (Naming divergences forced by Rust keyword conflicts are acceptable and documented separately below.)

### Document API Naming Divergences

Several stdlib functions intentionally diverge from CPython names due to Rust keyword conflicts (e.g., `sifr.shutil.move_file` instead of `sifr.shutil.move` because `move` is a Rust keyword). Audit all such cases across the stdlib, collect them into a table in the Python Divergences section of `architecture.md` with the format: `| sifr name | CPython name | reason |`. This makes the divergences discoverable and prevents future contributors from "fixing" them or introducing inconsistent workarounds.

### Definition of Done (milestone_stdlib_pure_expansion)

- All listed functions implemented and tested
- Existing `statistics` functions return `Result[float, StatisticsError]` (Phase 09 carryover fixed)
- Non-CPython functions removed
- API naming divergence table added to `architecture.md`
- E2E tests for all new functions

---

## milestone_new_modules: Critical Missing Modules

status: completed

**Goal:** Add critical missing modules that are needed by subsequent milestones and ecosystem phases. Placed second because several of these modules (`subprocess`, `sys`, `gzip`, `zipfile`) unblock downstream work.

**Depends on:** milestone_stdlib_pure_expansion (cleanup and naming audit should be done first)

### Modules

- `sifr.subprocess` (wraps `std::process`) — sync `run(cmd) -> Result[CompletedProcess, Error]` (async Popen added in Phase 14's `milestone_networking_stdlib`)
- `sifr.sys` — `argv`, `exit(code)`, `platform`, `version`, `maxsize`
- `sifr.html` — `escape(s)`, `unescape(s)`
- `sifr.configparser` — `ConfigParser` class
- `sifr.gzip` — `compress(data)`, `decompress(data)` (wraps `flate2`)
- `sifr.zipfile` — `ZipFile` class (wraps `zip`)
- `sifr.calendar` — `isleap`, `weekday`, `monthrange`
- `sifr.operator` — `add`, `sub`, `mul`, `itemgetter`

### Definition of Done (milestone_new_modules)

- All listed modules implemented with core APIs
- All fallible operations return `Result` or `Option`
- E2E tests for each module
- `sifr.sys.argv` and `sifr.sys.exit` work in CLI programs
- `sifr.subprocess.run` executes commands and returns results

---

## milestone_stdlib_intrinsic_expansion: New Intrinsics for Existing Modules

status: completed

**Goal:** Add new Rust intrinsics to deepen existing stdlib modules.

**Depends on:** milestone_new_modules (new modules should exist before deepening existing ones)

### New Intrinsics

- `math`: `erf`, `erfc`, `gamma`, `lgamma`, `frexp`, `ldexp`, `modf`, `nextafter`, `ulp`
- `os`: `chdir`, `getpid`, `cpu_count`, `stat`, `sep`, `linesep`, `name`
- `hashlib`: `sha224`, `sha384`, `blake2b`, `blake2s`
- `platform`: `node`, `release`, `version`, `processor`
- `time`: `strptime`, `gmtime`, `localtime`
- `base64`: `b32encode`, `b32decode`
- `shutil`: `which`, `disk_usage`

### Definition of Done (milestone_stdlib_intrinsic_expansion)

- All listed intrinsics implemented in `stdlib.rs` and codegen
- Corresponding `.sifr` wrappers updated
- E2E tests for each new intrinsic

---

## milestone_stdlib_class_deepening: Class API Enhancements

status: completed

**Goal:** Add class-based APIs to existing modules and introduce the `open()` built-in with file object protocol.

**Depends on:** milestone_stdlib_intrinsic_expansion (intrinsics should be in place before class APIs that may use them)

### Work Items

- `open()` built-in with file object protocol (`read`, `write`, `readline`, context manager support) — prerequisite for `csv.reader`/`csv.writer` and `logging.FileHandler`
- `collections.deque` class
- `datetime`: full `datetime` class (not string-based), `date`, `time`, `timezone` classes
- `pathlib.Path`: add `resolve`, `glob`, `rglob`, `iterdir`, `unlink`, `rmdir`, `touch`, `with_name`, `with_suffix`
- `re`: `compile` -> `Pattern` class, `match`, `fullmatch`, flags support
- `logging`: `basicConfig`, `FileHandler`, `Formatter`, level constants
- `csv`: `reader`/`writer` objects, `DictReader`/`DictWriter`

### Definition of Done (milestone_stdlib_class_deepening)

- `open()` built-in works with context manager (`with open(...) as f`)
- All listed class APIs implemented and tested
- `csv.reader`/`csv.writer` work with file objects from `open()`
- `logging.FileHandler` works with file objects from `open()`
- E2E tests for all new class APIs

---

## Milestone Ordering

- **milestone_stdlib_pure_expansion first:** Quick wins with no new intrinsics. Also performs cleanup and naming audit.
- **milestone_new_modules second:** High-impact missing modules (`sys`, `subprocess`, `html`, `operator`, `calendar`) that unblock downstream work.
- **milestone_stdlib_intrinsic_expansion third:** New intrinsics deepen existing modules.
- **milestone_stdlib_class_deepening fourth:** Large class rewrites (`datetime`, `deque`, `Pattern`, `open()`) that build on all prior work.
