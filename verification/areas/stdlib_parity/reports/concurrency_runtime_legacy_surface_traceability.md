# Concurrency Runtime Legacy Surface Traceability

The removed CPython-shaped runtime, concurrency, and process modules are not part of the public embedded stdlib. They now fail as ordinary unknown source modules with `SIFR-IMPORT-0002`. The compiler has no table that recognizes these names or proposes replacements.

| Removed import | Terminal state | Regression fixture |
| --- | --- | --- |
| `sifr.asyncio` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_asyncio_removed.sifr` |
| `sifr.subprocess` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_subprocess_removed.sifr` |
| `sifr.concurrent` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_concurrent_removed.sifr` |
| `sifr.concurrent.futures` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_concurrent_futures_removed.sifr` |
| `sifr.queue` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_queue_removed.sifr` |
| `sifr.multiprocessing` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_multiprocessing_removed.sifr` |
| `sifr.threading` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_threading_removed.sifr` |
| `sifr.contextlib` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_contextlib_removed.sifr` |
| `sifr.warnings` | unknown source module | `crates/sifr/tests/e2e/fail/legacy_sifr_warnings_removed.sifr` |

Implementation evidence:

- `stdlib/sifr/asyncio.sifr`, `stdlib/sifr/concurrent.sifr`, `stdlib/sifr/subprocess.sifr`, and `stdlib/sifr/threading.sifr` were removed.
- `crates/sifr_stdlib_manifest/src/sources.rs` does not embed the removed modules.
- `sifr_stdlib_imports` contains no removed-module recognition or replacement metadata.
- `SIFR-IMPORT-0002` reports these names through the normal unknown-module path.
- Legacy `sifr.asyncio` compatibility lowering, `asyncio.run` entrypoint inference, and `LowerCtx.asyncio_compat_imports` were removed so native task lowering no longer depends on CPython-shaped imports.
- Historical fail fixture names now assert the generic unknown-module diagnostic.
- `verification/areas/runtime_platform/golden/legacy_sifr_runtime_surfaces_removed.sifr` is active and checks the cross-surface removal gate.
