Findings (low effort, hunk-only, no verify):

- `crates/sifr_stdlib/src/fs.rs:47` — `mkdir` is implemented as `fs::create_dir_all`, identical to `makedirs` at line 79. If the deleted `io.rs` lowerer used `fs::create_dir` (the usual Python `os.mkdir` semantics), this silently loses errors: `mkdir("/nonexistent/child")` now succeeds where it previously raised `IOError`, and `mkdir` on an existing directory no longer raises `AlreadyExists`. Behavior parity with the removed lowerer is not preserved and `mkdir`/`makedirs` are indistinguishable.
- `crates/sifr_stdlib/src/fs.rs` (`touch`, ~line 85) — opens with `create(true).truncate(false).write(true)` and drops the handle. This creates missing files but does **not** update mtime for pre-existing files, unlike `pathlib.Path.touch`. Failure: `touch(existing_file)` no longer bumps modification time, so callers relying on touch-to-refresh (make-style staleness checks, marker files) silently see stale timestamps.

BLOCKED — verify `mkdir` parity against the deleted `registry/io.rs::lower_mkdir` and confirm `touch` mtime semantics against the prior lowerer before landing; if either matches the new behavior, downgrade to READY.
