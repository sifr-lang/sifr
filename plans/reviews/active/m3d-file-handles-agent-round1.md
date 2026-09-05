READY.

Verified the migration end-to-end against the diff — no blocking issues.

**Correctness checks that pass**
- `builtin_open` lowerer no longer emits `__SIFR_FILE_HANDLES` / `__sifr_next_file_handle_id` / `SifrFileHandle::*` — it routes entirely through `sifr_stdlib::fs::open_file` + `NativeFileHandle::new` (`crates/sifr_codegen/src/intrinsics/registry/file_handles.rs:76-118`). The emitter side-effect no longer sets `RuntimeNeed::FileHandles`, and the test updated to assert `!file_handles()` matches (`builtin_core_methods.rs:15-25,382`).
- `builtin_open_text` translates text mode → binary mode ("r"/"rt"→"rb", "w"/"wt"→"wb", "a"/"at"→"ab") and always opens through `sifr_stdlib::fs::open_file` with the binary mode, then wraps as `BinaryFileHandle` → `TextFileHandle` (`open_text_handles.rs:26-183`). Encoding/decoding sits above raw bytes — correct for the TextFileHandle contract.
- Registry dispatch table dropped all `file_*`/`open_file` entries (`registry.rs:59-63`); `registry_extended_tests.rs` now asserts `lower_intrinsic` returns `None` for each and that `builtin_open` routes through `sifr_stdlib::fs::open_file`.
- `sifr_retained_intrinsics::sys_fs` no longer registers the eight file-handle functions; test asserts `open_file` is not in the `_sifr.fs` module while `chdir` still is (`lib.rs:137`, `sys_fs.rs:150`).
- Migration closure script adds all eight to `RETIRED_INTRINSICS` (`check_stdlib_migration_closure.py:70-133`). Retained TOML narrows `exact_intrinsics` to just `builtin_open`/`builtin_open_text` with an updated rationale.
- `stateless_fs_codegen_tests` requires each of the eight new `@rust` declarations to codegen and be absent from the public surface (`stateless_fs_codegen_tests.rs:25-92`) — this is the guard that catches accidental re-exposure.
- `sifr_stdlib::fs` handle table (`fs.rs:1-360`) uses `LazyLock<Mutex<HashMap>>` with `unwrap_or_else(PoisonError::into_inner)` and `AtomicU64` id generation — no user-triggerable panics. Variant-mismatched access (e.g. write on a read handle, use-after-close, invalid mode) returns `io::Error::other(...)`, which the api_behavior test exercises for both text and binary and for double-close idempotency.
- Public `sifr.io.FileHandle`/`BinaryFileHandle` now hold `NativeFileHandle` (`io.sifr:111,201`), so raw `i64`s are no longer forgeable across the public boundary. Forging is only possible from inside `_sifr.fs`, which matches the underscore-namespace convention.

**Non-blocking observations**
- `stdlib/_sifr/fs.sifr:71-77`: `open_file`'s `try/except IOError as e: raise IOError(e.message)` is a no-op re-raise — can be simplified to `return NativeFileHandle(_open_file(path, mode))` (same pattern already used in `file_read`, `file_write`, etc.).
- `crates/sifr_stdlib/src/fs.rs` handle table serializes all file I/O behind a single global `Mutex`; a long `file_read` on one handle blocks unrelated handles. Fine for M3d, but worth revisiting if concurrent I/O becomes a workload — a `DashMap` or per-entry mutex would remove that contention.
- `NativeFileHandle.__init__(id: str)` is still constructible by any code that reaches into `_sifr.fs`. The naming convention is the only barrier; if a stronger guarantee is desired later, gate construction behind a module-private factory or an opaque tag.
