## Review round 2 — findings

**MEDIUM — `mkdir` missing from migration-closure guard**
`scripts/check_stdlib_migration_closure.py:116-117` adds 17 of the 18 retired intrinsics to `RETIRED_INTRINSICS` but omits `"mkdir"` (only `makedirs` is present). Every other deleted registry arm is listed. Non-blocking today (the arm is deleted so the guard passes), but if `mkdir` is later re-added to `registry.rs` it won't trip the regression guard the way its siblings would. Add `"mkdir",` to the set for parity.

**Behavior parity — OK**
- `mkdir`/`makedirs`: new `sifr_stdlib::fs::mkdir` uses `fs::create_dir_all`, matching deleted `io.rs::lower_mkdir`; `makedirs` remains an alias — matches round-1 clarification.
- `touch`: new impl uses `OpenOptions::new().create(true).truncate(false).write(true).open(path).map(|_|())` — byte-identical to deleted `pathlib.rs::lower_touch`; no new mtime semantics — matches round-1 clarification.
- `glob_pattern`/`rglob_pattern`: hand-rolled `*`/`?` matcher is semantically equivalent to the prior regex-escape approach for POSIX globs; hidden-file policy (`starts_with('.')`) preserved.

**Cargo features — OK**
`features.rs:609` flips `sifr.pathlib` from `Regex` → `Fs`; `generated_stdlib_features.rs` moves it into the `fs` bucket accordingly.

**Retained manifest / signatures — OK**
`sys_fs.rs` intrinsic deletions align with the 18 new host-fn signatures in `stdlib/_sifr/fs.sifr:26-114` (arg names, `Result[…, IOError]` shapes, infallible `bool` returns for `is_file`/`is_dir`/`gettempdir`). `stdlib_retained_compiler_intrinsics.toml` cleanly drops the 18 entries plus `io.rs`/`pathlib.rs`; `file_handles`/`open_text_handles` untouched. `lib.rs` smoke-test updated `getcwd` → `open_file`.

**Stale references — none**
`lower_mkdir`, `lower_touch`, `registry::io`, `registry::pathlib`, `io_mkdir`, `pathlib_touch` return zero hits across `crates/` and `scripts/`.

READY
