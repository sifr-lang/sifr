# Concurrency Runtime Legacy Surface Traceability

legacy-subprocess rejection capability removes CPython-shaped runtime/concurrency/process modules from the public embedded stdlib surface. These names are evidence sources only and now emit `SIFR-IMPORT-0009` with a Sifr-native replacement namespace.

| Legacy import | Terminal state | Native direction | Regression fixture |
| --- | --- | --- | --- |
| `sifr.asyncio` | `unsupported-with-diagnostic` | `sifr.task`, `sifr.sync` | `crates/sifr/tests/e2e/fail/legacy_sifr_asyncio_removed.sifr` |
| `sifr.subprocess` | `unsupported-with-diagnostic` | `sifr.process` | `crates/sifr/tests/e2e/fail/legacy_sifr_subprocess_removed.sifr` |
| `sifr.concurrent` | `unsupported-with-diagnostic` | `sifr.runtime`, `sifr.parallel` | `crates/sifr/tests/e2e/fail/legacy_sifr_concurrent_removed.sifr` |
| `sifr.concurrent.futures` | `unsupported-with-diagnostic` | `sifr.runtime`, `sifr.parallel` | `crates/sifr/tests/e2e/fail/legacy_sifr_concurrent_futures_removed.sifr` |
| `sifr.queue` | `unsupported-with-diagnostic` | `sifr.sync` | `crates/sifr/tests/e2e/fail/legacy_sifr_queue_removed.sifr` |
| `sifr.multiprocessing` | `rejected` | future `sifr.ipc` design gates | `crates/sifr/tests/e2e/fail/legacy_sifr_multiprocessing_removed.sifr` |
| `sifr.threading` | `unsupported-with-diagnostic` | `sifr.sync`, `sifr.runtime`, scoped offload | `crates/sifr/tests/e2e/fail/legacy_sifr_threading_removed.sifr` |
| `sifr.contextlib` | `unsupported-with-diagnostic` | `sifr.resource` | `crates/sifr/tests/e2e/fail/legacy_sifr_contextlib_removed.sifr` |
| `sifr.warnings` | `rejected` | typed diagnostics and runtime observability | `crates/sifr/tests/e2e/fail/legacy_sifr_warnings_removed.sifr` |

Implementation evidence:

- `lib/sifr/asyncio.sifr`, `lib/sifr/concurrent.sifr`, `lib/sifr/subprocess.sifr`, and `lib/sifr/threading.sifr` were removed.
- `crates/sifr_stdlib/src/sources.rs` no longer embeds those legacy modules.
- `sifr_stdlib::unsupported_legacy_stdlib_module` records native replacement namespaces.
- `SIFR-IMPORT-0009` distinguishes removed legacy Sifr modules from unknown modules.
- Legacy `sifr.asyncio` compatibility lowering, `asyncio.run` entrypoint inference, and `LowerCtx.asyncio_compat_imports` were removed so native task lowering no longer depends on CPython-shaped imports.
- Historical fail fixture names that referenced member-level legacy behavior now assert the module-level removal diagnostic and provide imported-member payload coverage.
- `verification/areas/runtime_platform/golden/legacy_sifr_runtime_surfaces_removed.sifr` is active and checks the cross-surface removal gate.
