# Stdlib Remediation

**Why now:** Phase 11 (Stdlib Deepening) is complete but a gap analysis revealed several unfinished items — most critically the `open()` built-in, incomplete `datetime` classes, and a `subprocess.run` that returns a plain string instead of a structured `CompletedProcess` object. These gaps must be closed before the type system completion phase (Phase 13) rewrites the stdlib to use generics, and before the async phase (Phase 14) builds async I/O on top of these primitives. Fixing them now means the generic rewrite operates on a complete stdlib, and async file/subprocess operations have the right foundation.

---

## milestone_stdlib_remediation: Phase 11 Gap Closure

status: done

**Goal:** Close all gaps identified in the Phase 11 gap analysis. This is a single focused milestone that completes the stdlib surface area before the type system completion phase rewrites it with generics.

**Depends on:** milestone_stdlib_class_deepening (Phase 11 must be complete)

### High-Priority: `open()` Built-in and File Object Protocol

The `open()` built-in was specified in Phase 11's `milestone_stdlib_class_deepening` but was not implemented. It is a prerequisite for file-object-based `csv.reader`/`csv.writer` and `logging.FileHandler`.

#### Work Items

- Implement `open(path: str, mode: str = "r") -> Result[FileHandle, IOError]` as a built-in function
- `FileHandle` class with methods:
  - `read() -> Result[str, IOError]` — read entire file contents
  - `write(data: str) -> Result[None, IOError]` — write string to file
  - `readline() -> Result[str | None, IOError]` — read one line, `None` at EOF
  - `readlines() -> Result[list[str], IOError]` — read all lines
  - `close() -> None` — explicit close (also called by `__exit__`)
- Context manager support: `FileHandle` implements `__enter__` / `__exit__` so `with open(...) as f:` works
- Modes: `"r"` (read text), `"w"` (write text, truncate), `"a"` (append text), `"rb"` (read binary), `"wb"` (write binary), `"ab"` (append binary)
- Binary modes return `FileHandle` backed by `BufReader<File>` / `BufWriter<File>` with `read_bytes() -> Result[bytes, IOError]` and `write_bytes(data: bytes) -> Result[None, IOError]` methods
- Binary file support is needed by Phase 19 (`milestone_data_processing` for Parquet I/O), Phase 15 (`milestone_crypto_auth` for AES encryption), and existing stdlib modules (`gzip`, `zipfile`)
- Rust intrinsics: add `_sifr.fs.open_file`, `_sifr.fs.file_read`, `_sifr.fs.file_write`, `_sifr.fs.file_readline`, `_sifr.fs.file_readlines`, `_sifr.fs.file_close` to `stdlib.rs`
- The `FileHandle` wraps a Rust `BufReader<File>` or `BufWriter<File>` depending on mode
- All file operations return `Result[T, IOError]` — no panics

#### Integration with existing modules

- Update `csv.reader` / `csv.writer` to accept `FileHandle` (in addition to the existing `str` API for backward compatibility)
- Update `logging.FileHandler` to use `FileHandle` internally via `open()` instead of the raw `append_text` intrinsic
- Update `logging.basicConfig` to be functional (currently a no-op) — at minimum, set the global log level

### Medium-Priority: `datetime` Completion

#### Work Items

- `datetime.time` class with `hour`, `minute`, `second` fields, `isoformat()`, `__eq__`, `__str__`
- `datetime.timezone` class with the canonical `UTC()` zero-offset constructor and `offset` field
- `datetime.now()` must return a `datetime` object (not a string). This requires a new intrinsic `_sifr.datetime.datetime_now_struct` that returns year/month/day/hour/minute/second as integers, which the Sifr `datetime` constructor uses
- `datetime.today()` alias for `datetime.now()` (date portion only, returns `date` object)

### Medium-Priority: `subprocess.CompletedProcess`

#### Work Items

- `CompletedProcess` class with fields: `returncode: int`, `stdout: str`, `stderr: str`
- Update `subprocess.run(cmd: str) -> Result[CompletedProcess, IOError]` to return the structured class instead of a plain string
- The existing `_sifr.sys.run_command` intrinsic needs to return stdout, stderr, and exit code separately. Update the intrinsic to return a tuple or struct.

### Medium-Priority: `pathlib.Path` Completion

#### Work Items

- `Path.glob(pattern: str) -> Result[Iterator[str], IOError]` — match files in directory using glob pattern (explicit `list(...)` materialization at call sites when eager values are needed)
- `Path.rglob(pattern: str) -> Result[Iterator[str], IOError]` — recursive glob (explicit `list(...)` materialization at call sites when eager values are needed)
- These require new Rust intrinsics wrapping the `glob` crate or `std::fs::read_dir` with pattern matching

### Low-Medium Priority: `re` Flags Support

#### Work Items

- Define flag constants in `re.sifr`: `IGNORECASE`, `MULTILINE`, `DOTALL`, `VERBOSE`
- Update `compile(pattern: str, flags: int = 0) -> Pattern` to accept flags
- Update `re_match`, `re_find`, `re_replace`, `re_findall`, `re_split` intrinsics to accept a flags parameter
- Flags map to Rust `regex` crate inline flags: `(?i)` for IGNORECASE, `(?m)` for MULTILINE, `(?s)` for DOTALL

### Low Priority: Minor Surface Area Gaps

#### `os` module constants

- Expose `os.sep` (path separator), `os.linesep` (line separator), `os.name` (OS name) as module-level constants
- These require either new intrinsics that return platform-specific strings, or compile-time constants injected during codegen

#### `time` module wrapper functions

- Add explicit wrapper functions for `strptime`, `gmtime`, `localtime` in `time.sifr` with documentation, matching the pattern used by `time()` and `strftime()`

#### `random.choice` re-export

- `random.choice` is imported from `_sifr.random` but not re-exported in `random.sifr`. Add it to the exports.

### Cleanup: Non-CPython `itertools` Functions

The Phase 11 plan specified removing non-CPython functions. The following remain and should be addressed:

- `itertools.take` — not in CPython. Keep but document as a Sifr extension in the API naming divergences table in `architecture.md`. It is genuinely useful and removing it would break existing code.
- `itertools.flatten` — not in CPython (CPython uses `chain.from_iterable`). Same treatment: keep and document.
- `itertools.accumulate_float` — type-specialized variant that will be deleted in Phase 13's stdlib generic rewrite (replaced by generic `accumulate[T]`). Mark as deprecated but do not remove yet.

### Definition of Done (milestone_stdlib_remediation)

- `open()` built-in works with `"r"`, `"w"`, `"a"`, `"rb"`, `"wb"`, `"ab"` modes
- `with open(...) as f:` works (context manager)
- `FileHandle.read()`, `.write()`, `.readline()`, `.readlines()`, `.close()` all work
- `csv.reader` / `csv.writer` accept `FileHandle`
- `logging.FileHandler` uses `FileHandle` internally
- `logging.basicConfig` sets the global log level (not a no-op)
- `datetime.time` and `datetime.timezone` classes implemented
- `datetime.now()` returns a `datetime` object
- `subprocess.run` returns `CompletedProcess` with `returncode`, `stdout`, `stderr`
- `Path.glob` and `Path.rglob` work with iterator-returning contracts and explicit materialization boundaries
- `re` flags (`IGNORECASE`, `MULTILINE`, `DOTALL`, `VERBOSE`) work in `compile()` and standalone functions
- `os.sep`, `os.linesep`, `os.name` exposed
- `time.strptime`, `time.gmtime`, `time.localtime` have wrapper functions
- `random.choice` re-exported
- `itertools.take` and `itertools.flatten` documented as Sifr extensions in `architecture.md`
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- New E2E pass tests: `open_read`, `open_write`, `open_context_manager`, `open_readline`, `open_binary_read`, `open_binary_write`, `csv_reader_file`, `datetime_time_class`, `datetime_now_object`, `subprocess_completed_process`, `path_glob`, `re_flags_ignorecase`
- Milestone demo in `./demos/stdlib_fixes/main.sifr`
