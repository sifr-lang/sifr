# PRDS: milestone_stdlib_remediation — Phase 11 Gap Closure

## Product Requirements

### Problem
Phase 11 (Stdlib Deepening) is complete but a gap analysis revealed several unfinished items:
- `open()` built-in was specified but not implemented — blocks file-object-based csv/logging
- `datetime.time` and `datetime.timezone` classes are missing
- `datetime.now()` returns a string instead of a `datetime` object
- `subprocess.run` returns a plain string instead of a structured `CompletedProcess` object
- `Path.glob` and `Path.rglob` are missing
- `re` module lacks flag support (IGNORECASE, MULTILINE, DOTALL, VERBOSE)
- Minor gaps: `os.sep/linesep/name`, `time` wrapper functions, `random.choice` re-export

### Goals
Close all gaps identified in the Phase 11 gap analysis before the type system completion phase (Phase 13) rewrites the stdlib to use generics.

### Scope
- `open()` built-in with FileHandle class (text + binary modes, context manager)
- `datetime.time`, `datetime.timezone`, fix `datetime.now()` to return object
- `subprocess.CompletedProcess` structured return type
- `Path.glob` and `Path.rglob`
- `re` flags (IGNORECASE, MULTILINE, DOTALL, VERBOSE)
- Minor gaps: os constants, time wrappers, random.choice, itertools documentation

### Acceptance Criteria
- `open()` built-in works with "r", "w", "a", "rb", "wb", "ab" modes
- `with open(...) as f:` works (context manager)
- `FileHandle.read()`, `.write()`, `.readline()`, `.readlines()`, `.close()` all work
- `csv.reader` / `csv.writer` accept `FileHandle`
- `logging.FileHandler` uses `FileHandle` internally
- `logging.basicConfig` sets the global log level
- `datetime.time` and `datetime.timezone` classes implemented
- `datetime.now()` returns a `datetime` object
- `subprocess.run` returns `CompletedProcess` with `returncode`, `stdout`, `stderr`
- `Path.glob` and `Path.rglob` work
- `re` flags work in `compile()` and standalone functions
- `os.sep`, `os.linesep`, `os.name` exposed
- `time.strptime`, `time.gmtime`, `time.localtime` have wrapper functions
- `random.choice` re-exported
- All existing E2E tests still pass

## Solution Design

### Architecture

#### 1. FileHandle — New class in Sifr stdlib
A new `FileHandle` class backed by Rust file I/O intrinsics. The class is defined in a new `io.sifr` or embedded in the built-in scope. The `open()` function is registered as a built-in function (like `print`, `len`, etc.).

**Intrinsics needed in `_sifr.fs`:**
- `open_file(path: str, mode: str) -> Result[int, IOError]` — returns a file handle ID (opaque int)
- `file_read(handle: int) -> Result[str, IOError]`
- `file_write(handle: int, data: str) -> Result[None, IOError]`
- `file_readline(handle: int) -> Result[str | None, IOError]`
- `file_readlines(handle: int) -> Result[list[str], IOError]`
- `file_close(handle: int) -> None`
- `file_read_bytes(handle: int) -> Result[list[int], IOError]`
- `file_write_bytes(handle: int, data: list[int]) -> Result[None, IOError]`

**FileHandle class** in `lib/sifr/io.sifr`:
```python
class FileHandle:
    _handle: int
    _mode: str
    
    def __init__(self, handle: int, mode: str):
        self._handle = handle
        self._mode = mode
    
    def read(self) -> Result[str, IOError]: ...
    def write(self, data: str) -> Result[None, IOError]: ...
    def readline(self) -> Result[str | None, IOError]: ...
    def readlines(self) -> Result[list[str], IOError]: ...
    def close(self) -> None: ...
    def __enter__(self) -> FileHandle: ...
    def __exit__(self) -> None: ...
```

**`open()` built-in**: Registered in the compiler's built-in scope. Calls `_sifr.fs.open_file` and wraps result in `FileHandle`.

#### 2. datetime completion
- Add `datetime_now_struct` intrinsic returning year/month/day/hour/minute/second as a tuple
- Update `datetime.now()` in `datetime.sifr` to use the struct intrinsic and return a `datetime` object
- Add `datetime.time` class with `hour`, `minute`, `second`, `isoformat()`, `__eq__`, `__str__`
- Add `datetime.timezone` class with `utc` constant and `offset` field
- Add `datetime.today()` alias

#### 3. subprocess.CompletedProcess
- Add `subprocess_run_structured` intrinsic returning stdout, stderr, returncode as separate values
- Add `CompletedProcess` class in `subprocess.sifr`
- Update `subprocess.run()` to return `Result[CompletedProcess, IOError]`

#### 4. Path.glob / Path.rglob
- Add `glob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]` intrinsic
- Add `rglob_pattern(dir: str, pattern: str) -> Result[list[str], IOError]` intrinsic
- Add `glob()` and `rglob()` methods to `Path` class

#### 5. re flags
- Add flag constants to `re.sifr`: `IGNORECASE = 2`, `MULTILINE = 8`, `DOTALL = 16`, `VERBOSE = 64`
- Update `re_match`, `re_find`, etc. intrinsics to accept optional flags parameter
- Flags map to regex inline flags: `(?i)` for IGNORECASE, `(?m)` for MULTILINE, `(?s)` for DOTALL, `(?x)` for VERBOSE
- Update `compile(pattern, flags=0)` to store flags and use them in all methods

#### 6. Minor gaps
- `os.sep`, `os.linesep`, `os.name` — add intrinsics or compile-time constants
- `time.strptime`, `time.gmtime`, `time.localtime` — add wrapper functions
- `random.choice` — add re-export
- `itertools.take`, `itertools.flatten` — document as Sifr extensions in architecture.md

### Testing Strategy
New E2E pass tests: `open_read`, `open_write`, `open_context_manager`, `open_readline`, `open_binary_read`, `open_binary_write`, `csv_reader_file`, `datetime_time_class`, `datetime_now_object`, `subprocess_completed_process`, `path_glob`, `re_flags_ignorecase`

### Demo
`./demos/milestone_stdlib_remediation_demo.sifr` — showcases all new features
