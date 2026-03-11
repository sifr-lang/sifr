# CPython vs Sifr Stdlib Gap Audit (Design-Aware)

Generated on 2026-02-17.

## Scope and Method

- Canonical CPython module list source: `cpython/Python/stdlib_module_names.h` (same source as `sys.stdlib_module_names`).
- Sifr module surface source: `codebase/lib/sifr/*.sifr`.
- Existing parity estimates reused from `codebase/audits/STDLIB_PARITY_MASTER_REPORT.md` for currently implemented modules.
- Classification is design-aware: modules were tagged according to Sifr architecture constraints (safety model, no exception-driven runtime, wrapper+FFI strategy, async deferred to ecosystem phase).

## Headline Metrics

- Canonical CPython stdlib modules audited: **296**
- CPython-named modules implemented in Sifr: **34** (11.5%)
- Average parity across implemented CPython-named modules: **~37.8%** (based on existing parity report estimates)
- Sifr-only support modules (non-CPython names): **bytes, env, test**

## Category Breakdown (All CPython Modules)

| Category | Count | Meaning |
| --- | ---: | --- |
| implemented | 34 | Present in `lib/sifr` with partial parity |
| algorithmic_general | 43 | General-purpose modules that are good parity candidates |
| async_concurrency | 7 | Depends on async/task runtime and concurrency primitives |
| ecosystem_networking_io | 35 | Large IO/network/framework wrappers better tackled in ecosystem phase |
| os_platform_ext | 27 | Platform/C-extension heavy; best via wrappers/FFI |
| runtime_introspection | 46 | Runtime/introspection/tooling modules that diverge from Sifr design |
| cpython_internal | 104 | CPython internals/private modules (`_*`) |

## Current Implemented CPython-Named Modules

| Module | Estimated Parity | Notes |
| --- | ---: | --- |
| `argparse` | ~15% | 3 functions |
| `base64` | ~50% | 2 functions |
| `bisect` | ~75% | 3 functions |
| `collections` | ~40% | 14 functions + Counter class (8 methods) |
| `csv` | ~30% | 4 functions |
| `datetime` | ~15% | 3 functions |
| `difflib` | ~20% | 3 functions |
| `fnmatch` | ~50% | 2 functions |
| `functools` | ~15% | 2 functions |
| `glob` | ~20% | 1 function (renamed to `glob`) |
| `graphlib` | ~30% | 1 function |
| `hashlib` | ~40% | 2 functions |
| `heapq` | ~60% | 6 functions |
| `io` | ~35% | 5 functions |
| `ipaddress` | ~25% | 4 functions |
| `itertools` | ~20% | 6 functions |
| `json` | ~60% | 2 functions |
| `logging` | ~15% | 4 functions |
| `math` | ~85% | 29 functions + 5 constants |
| `os` | ~40% | 13 functions |
| `pathlib` | ~15% | 4 functions |
| `platform` | ~20% | 2 functions |
| `random` | ~35% | 4 functions |
| `re` | ~45% | 5 functions |
| `secrets` | ~40% | 2 functions |
| `shutil` | ~20% | 3 functions (copy, move_file, rmtree) |
| `statistics` | ~50% | 5 functions |
| `string` | ~60% | 8 constants |
| `tempfile` | ~25% | 3 functions |
| `textwrap` | ~60% | 4 functions |
| `time` | ~45% | 5 functions (added perf_counter, monotonic) |
| `timeit` | ~60% | 3 functions (default_timer, timeit, repeat) |
| `tomllib` | ~50% | 2 functions (loads, load) |
| `uuid` | ~20% | 1 function |

## Priority Gaps After Parity (Recommended)

1) **Phase 8 stdlib expansion (high ROI, low design friction)**

   - `calendar`, `configparser`, `copy`, `enum`, `fractions`, `getopt`, `gettext`, `hmac`, `numbers`, `operator`, `pprint`, `sched`, `stat`, `struct`

2) **Phase 8 ecosystem wrappers (networking/IO stacks)**

   - `email`, `ensurepip`, `fileinput`, `ftplib`, `gzip`, `html`, `http`, `imaplib`, `mailbox`, `mimetypes`, `netrc`, `plistlib`, `poplib`, `quopri`, `shlex`, `smtplib`, `socket`, `socketserver`, `ssl`, `subprocess`

3) **Phase 8 async runtime dependencies**

   - `asyncio`, `concurrent`, `contextvars`, `multiprocessing`, `queue`, `selectors`, `threading`

4) **Intentional or low-priority divergence from CPython internals/runtime model**

   - Modules in `cpython_internal` and much of `runtime_introspection` should be treated as explicitly non-goals unless needed for compatibility guarantees.

## Artifacts

- Full module-by-module matrix: `cpython_to_sifr_module_matrix.csv`
- This summary: `AUDIT_SUMMARY.md`

## Notes on Design Alignment

- Sifr architecture uses `Result/Option` semantics instead of exception-driven behavior; modules with exception-heavy APIs need adaptation rather than direct porting.
- Sifr’s stdlib strategy is thin wrappers over Rust crates + FFI fallback, so parity should prioritize API ergonomics and safety contracts over implementation mirroring.
- CPython internals and interpreter tooling are not strong parity targets for a compiled language pipeline unless explicitly required for migration tooling.
