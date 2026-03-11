# Missing Modules Assessment

CPython ships ~289 stdlib modules. Sifr has 37. This document categorizes the ~252 missing modules by priority and Sifr-applicability.

---

## HIGH Priority — Commonly Used in Real Python Projects

These modules are used in the majority of non-trivial Python programs. Their absence significantly limits what Sifr programs can do.

### Networking & Web

| CPython Module | Description | Sifr Design Notes | Rust Crate |
|----------------|-------------|-------------------|------------|
| `subprocess` | Process creation, pipes, shell commands | Return `Result` for process failures; `Popen` maps to struct with owned handles | `std::process` |
| `socket` | Low-level TCP/UDP networking | All operations return `Result`; socket maps to owned resource with `Drop` | `std::net`, `tokio::net` |
| `http` / `http.client` / `http.server` | HTTP protocol | Wrap `reqwest` (client) and `axum` (server); all I/O returns `Result` | `reqwest`, `axum` |
| `urllib` / `urllib.parse` / `urllib.request` | URL handling | `urllib.parse` is pure logic (no I/O); `urllib.request` wraps `reqwest` | `url`, `reqwest` |
| `ssl` | TLS/SSL wrapper | Wrap `rustls` or `native-tls`; all operations return `Result` | `rustls`, `native-tls` |
| `email` | Email/MIME handling | Mostly string parsing — can be pure Sifr | `lettre` |
| `html` | HTML entities, parsing | `html.escape`/`html.unescape` are pure string ops | `html-escape` |

### Concurrency & Async

| CPython Module | Description | Sifr Design Notes | Rust Crate |
|----------------|-------------|-------------------|------------|
| `asyncio` | Async I/O, event loop | Maps to `tokio` runtime; `async`/`await` syntax | `tokio` |
| `threading` | Thread-based parallelism | `Thread` maps to `std::thread::spawn`; `Lock` maps to `Mutex` | `std::thread`, `std::sync` |
| `concurrent.futures` | High-level async execution | `ThreadPoolExecutor` maps to `rayon` or `tokio` task pool | `rayon`, `tokio` |
| `multiprocessing` | Process-based parallelism | Complex — may defer; `Pool` maps to process spawning | `std::process` |
| `queue` | Thread-safe FIFO queue | Maps to `std::sync::mpsc` or `crossbeam::channel` | `crossbeam` |
| `selectors` | I/O multiplexing | Maps to `mio` or `tokio` | `mio`, `tokio` |

### Data & Serialization

| CPython Module | Description | Sifr Design Notes | Rust Crate |
|----------------|-------------|-------------------|------------|
| `sqlite3` | SQLite database | Wrap `rusqlite`; all queries return `Result` | `rusqlite` |
| `pickle` | Object serialization | **Not directly applicable** — Sifr has no dynamic types. Consider `serde` instead | `serde` |
| `configparser` | INI file parser | Pure string parsing; can be pure Sifr or wrap `configparser` crate | `configparser` |
| `xml` / `xml.etree.ElementTree` | XML processing | Wrap `quick-xml` or `roxmltree` | `quick-xml`, `roxmltree` |
| `struct` | Binary data packing | Maps to `bytemuck` or manual byte manipulation | `bytemuck`, `zerocopy` |

### Type System & OOP

| CPython Module | Description | Sifr Design Notes |
|----------------|-------------|-------------------|
| `dataclasses` | Data class decorator | **Built into language** — Sifr classes auto-derive Debug, Clone, PartialEq |
| `enum` | Enumeration types | **Built into language** — Sifr union types + literal types cover this |
| `abc` | Abstract base classes | **Built into language** — Sifr protocols cover this |
| `typing` | Type hints | **Built into language** — Sifr has static types |
| `copy` | Shallow/deep copy | **Built into language** — `.clone()` is auto-derived |

### Utilities

| CPython Module | Description | Sifr Design Notes | Rust Crate |
|----------------|-------------|-------------------|------------|
| `sys` | System parameters | `sys.argv` → `get_args()`, `sys.exit()` → `std::process::exit()`, `sys.path` → N/A | `std::env`, `std::process` |
| `pprint` | Pretty-printing | **Built into language** — auto-derived `Debug` trait; `print()` uses `Display` | — |
| `contextlib` | Context manager utilities | Partially built-in via `with` statement; `contextmanager` decorator needs generators | — |
| `warnings` | Warning system | Could be compile-time diagnostics instead of runtime warnings | — |
| `traceback` | Stack trace formatting | Different model — Sifr uses `Result` not exceptions; stack traces only from `assert` panics | — |
| `array` | Efficient numeric arrays | Maps to `Vec<T>` with fixed element type; or wrap `ndarray` | `ndarray` |

---

## MEDIUM Priority — Used Occasionally

### Compression & Archives

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `zipfile` | ZIP archive read/write | `zip` |
| `tarfile` | TAR archive read/write | `tar` |
| `gzip` | Gzip compression | `flate2` |
| `bz2` | Bzip2 compression | `bzip2` |
| `lzma` | LZMA compression | `lzma-rs` |
| `zlib` | Zlib compression | `flate2` |

### Math & Science

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `decimal` | Arbitrary-precision decimal | `rust_decimal` |
| `fractions` | Rational numbers | `num-rational` |
| `cmath` | Complex number math | `num-complex` |

### Text & Encoding

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `unicodedata` | Unicode character database | `unicode-properties` |
| `codecs` | Codec registry | `encoding_rs` |
| `shlex` | Shell lexical analysis | `shlex` |

### Date & Calendar

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `calendar` | Calendar functions | `chrono` |
| `zoneinfo` | IANA timezone support | `chrono-tz` |

### Introspection & Debugging

| CPython Module | Description | Sifr Design Notes |
|----------------|-------------|-------------------|
| `operator` | Standard operators as functions | Could be useful for `functools.reduce` key functions |
| `inspect` | Object introspection | **Not applicable** — no runtime introspection in compiled language |
| `dis` | Bytecode disassembler | **Not applicable** — no bytecode |
| `types` | Dynamic type creation | **Not applicable** — all types known at compile time |

### Testing

| CPython Module | Description | Sifr Design Notes |
|----------------|-------------|-------------------|
| `unittest` | Unit testing framework | Sifr has `sifr.test`; may want richer framework later |
| `doctest` | Test from docstrings | Could be a Sifr-specific tool |

### File Handling

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `filecmp` | File/directory comparison | `std::fs` |
| `fileinput` | Iterate over input lines | `std::io::BufRead` |
| `linecache` | Random access to text lines | `std::io::BufRead` |
| `mimetypes` | MIME type mapping | `mime_guess` |

### Networking (Advanced)

| CPython Module | Description | Rust Crate |
|----------------|-------------|------------|
| `smtplib` | SMTP client | `lettre` |
| `imaplib` | IMAP client | `imap` |
| `ftplib` | FTP client | `suppaftp` |

---

## LOW Priority — Rarely Used or Python-Specific

### Python Internals (Not Applicable to Sifr)

| CPython Module | Why Not Applicable |
|----------------|-------------------|
| `ast` | Python-specific AST manipulation |
| `compile` / `exec` / `eval` | Dynamic code execution — impossible in compiled language |
| `gc` | Garbage collection — Sifr uses ownership/RAII |
| `weakref` | Weak references — ownership model eliminates most use cases |
| `symtable` | Compiler symbol tables — Python-specific |
| `tokenize` / `token` | Python tokenizer — Python-specific |
| `keyword` | Python keywords — Python-specific |
| `codeop` | Compile incomplete Python code — Python-specific |
| `py_compile` / `compileall` | Bytecode compilation — Python-specific |
| `copyreg` / `pickletools` | Pickle support — Python-specific serialization |
| `importlib` / `pkgutil` / `modulefinder` | Import system — Sifr has its own |
| `zipimport` | Import from ZIP — Python-specific |
| `runpy` | Run Python modules — Python-specific |
| `site` | Site configuration — Python-specific |
| `ensurepip` / `venv` | Python packaging — Sifr will have its own |
| `__future__` | Future statements — Python-specific |
| `lib2to3` | Python 2→3 migration — Python-specific |

### GUI & Terminal

| CPython Module | Why Low Priority |
|----------------|-----------------|
| `tkinter` | GUI toolkit — Sifr would use a different approach (web, native) |
| `turtle` | Educational graphics — niche |
| `curses` | Terminal UI — niche; wrap `crossterm` if needed |
| `idlelib` | IDLE IDE — Python-specific |

### Legacy / Deprecated

| CPython Module | Status |
|----------------|--------|
| `cgi` / `cgitb` | Deprecated in Python 3.11+ |
| `telnetlib` | Deprecated |
| `nntplib` | Deprecated |
| `optparse` | Superseded by `argparse` |
| `getopt` | Superseded by `argparse` |
| `aifc` | Deprecated |
| `sunau` | Deprecated |
| `chunk` | Deprecated |

### Niche / Specialized

| CPython Module | Description |
|----------------|-------------|
| `wave` | WAV audio files |
| `colorsys` | Color system conversions |
| `mailbox` | Mailbox formats |
| `poplib` | POP3 client |
| `xmlrpc` | XML-RPC protocol |
| `netrc` | .netrc file parser |
| `quopri` | MIME quoted-printable |
| `stringprep` | String preparation (RFC 3454) |
| `reprlib` | Alternate repr() |
| `sched` | Event scheduler |
| `tabnanny` | Indentation detection |
| `antigravity` / `this` | Easter eggs |
| `gettext` / `locale` | Internationalization |
| `signal` | Signal handling |
| `contextvars` | Context variables |
| `sysconfig` | Python configuration |
| `stat` | Interpret stat() results |
| `bdb` / `profile` / `cProfile` / `pstats` | Debugging/profiling |
| `trace` / `tracemalloc` | Execution/memory tracing |
| `dbm` | Unix database interfaces |
| `shelve` | Persistent dict storage |
| `wsgiref` | WSGI reference implementation |
| `cmd` | Command interpreter framework |
| `readline` / `rlcompleter` | GNU readline interface |

---

## Summary Counts

| Category | Count | Notes |
|----------|-------|-------|
| HIGH — Should implement | ~27 modules | Core functionality for real programs |
| HIGH — Built into language | ~5 modules | `dataclasses`, `enum`, `abc`, `typing`, `copy` |
| MEDIUM — Implement selectively | ~30 modules | Based on user demand |
| LOW — Rarely needed | ~60+ modules | Implement on request |
| NOT APPLICABLE | ~20+ modules | Python-specific internals |
| DEPRECATED | ~10 modules | Don't implement |

**Net new modules to implement (HIGH priority):** ~22 modules (excluding those built into the language)
