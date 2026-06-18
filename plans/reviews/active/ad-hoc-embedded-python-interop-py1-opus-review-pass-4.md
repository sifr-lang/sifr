No duplicates remain in `diag/package.rs` or `diag/mod.rs`. The refactor verifies cleanly:

- `diag/python.rs:6-70` owns all three Python diagnostic constructors via `impl PackageDiagnostic`. Module is wired in `diag/mod.rs:2` (`mod python;`), and `impl` blocks need no re-export.
- `cargo/python_probe.rs:6-55` owns `run_python_probe_command` plus `PROBE_SCRIPT` and a local `probe_error` helper. Module is wired in `cargo/mod.rs:6` (`pub(crate) mod python_probe;`).
- `python/environment.rs:140-144` is the sole caller, passing the request plus the two JSON arrays. `PROBE_SCRIPT` and `Command::new` no longer appear in `environment.rs`; the local `probe_error` at `environment.rs:349` is still used for the JSON-decode error path, which is correct (separate concern from subprocess execution).
- No call sites outside these modules reference the moved items, so visibility (`pub(crate)` on the probe fn, default `pub` on the `impl` methods) is sufficient.

Behavior is preserved: the probe argv, env handling, success/error branching, and diagnostic codes are byte-identical to the prior inline version.

Reviewer satisfied after refactor; no blocking issues.
