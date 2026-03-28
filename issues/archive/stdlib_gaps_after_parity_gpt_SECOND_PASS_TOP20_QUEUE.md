# Second Pass: Top-20 Stdlib Module Queue (Impact/Effort)

Generated on 2026-02-17.

## Method

- Candidate set: missing non-internal CPython stdlib modules from `cpython_to_sifr_module_matrix.csv`.
- Impact signals:
  - import reference count across CPython `Lib/*.py` (`import X` / `from X import ...`).
  - category alignment with Sifr roadmap (algorithmic/core, wrappers, async).
  - strategic boost for modules that unblock many APIs or language ergonomics.
- Effort signals:
  - CPython source LOC proxy for each module/package.
  - CPython `Lib/test/test_<module>.py` LOC proxy.
  - category complexity bias (platform internals and runtime-introspection cost more).
- Score formula: `score = impact*2.1 - effort*1.2` (higher is better).

## Milestone-Ordered Top 20

| Order | Module | Global Rank | Category | Impact | Effort | Score | CPython Import Refs | Milestone | Why now |
| ---: | --- | ---: | --- | ---: | ---: | ---: | ---: | --- | --- |
| 1 | `operator` | 11 | algorithmic_general | 5.0 | 3.3 | 6.54 | 70 | `milestone_phase8a_stdlib_core_parity_i` | Foundational operators/helpers used pervasively by higher-level stdlib APIs. |
| 2 | `struct` | 13 | algorithmic_general | 5.0 | 3.3 | 6.54 | 75 | `milestone_phase8a_stdlib_core_parity_i` | Binary packing/unpacking unlocks protocol and file-format wrappers. |
| 3 | `copy` | 9 | algorithmic_general | 5.0 | 3.3 | 6.54 | 86 | `milestone_phase8a_stdlib_core_parity_i` | Common utility for safe value duplication semantics in userland code. |
| 4 | `enum` | 53 | algorithmic_general | 5.0 | 4.8 | 4.74 | 54 | `milestone_phase8a_stdlib_core_parity_i` | High-level API ergonomics and parity for many CPython-facing libraries. |
| 5 | `numbers` | 6 | algorithmic_general | 4.8 | 2.5 | 7.08 | 6 | `milestone_phase8a_stdlib_core_parity_i` | Numeric protocol layer useful for generic math/statistics parity. |
| 6 | `stat` | 4 | algorithmic_general | 5.0 | 2.5 | 7.5 | 48 | `milestone_phase8a_stdlib_core_parity_i` | Natural extension of existing `os`/`pathlib` wrappers. |
| 7 | `getopt` | 2 | algorithmic_general | 5.0 | 1.8 | 8.34 | 12 | `milestone_phase8a_stdlib_core_parity_i` | Low-effort CLI compatibility win and precursor to richer argparse parity. |
| 8 | `calendar` | 7 | algorithmic_general | 5.0 | 3.3 | 6.54 | 10 | `milestone_phase8b_stdlib_core_parity_ii` | Frequent utility module with manageable algorithmic surface. |
| 9 | `configparser` | 40 | algorithmic_general | 4.8 | 4.1 | 5.16 | 4 | `milestone_phase8b_stdlib_core_parity_ii` | High ROI for app configuration workflows and migration. |
| 10 | `hmac` | 41 | algorithmic_general | 4.8 | 4.1 | 5.16 | 7 | `milestone_phase8b_stdlib_core_parity_ii` | Security primitive that complements existing `hashlib` and `base64`. |
| 11 | `pprint` | 12 | algorithmic_general | 5.0 | 3.3 | 6.54 | 16 | `milestone_phase8b_stdlib_core_parity_ii` | Developer ergonomics module; relatively self-contained. |
| 12 | `fractions` | 27 | algorithmic_general | 5.0 | 4.1 | 5.58 | 36 | `milestone_phase8b_stdlib_core_parity_ii` | Completes numeric story for deterministic arithmetic use-cases. |
| 13 | `socket` | 78 | ecosystem_networking_io | 5.0 | 5.0 | 4.5 | 128 | `milestone_phase8c_system_wrappers` | Core network substrate required by many ecosystem modules. |
| 14 | `ssl` | 79 | ecosystem_networking_io | 5.0 | 5.0 | 4.5 | 40 | `milestone_phase8c_system_wrappers` | Security layer needed for production network stack parity. |
| 15 | `subprocess` | 80 | ecosystem_networking_io | 5.0 | 5.0 | 4.5 | 154 | `milestone_phase8c_system_wrappers` | Critical process-control API for tooling and automation. |
| 16 | `urllib` | 89 | ecosystem_networking_io | 5.0 | 5.0 | 4.5 | 73 | `milestone_phase8c_system_wrappers` | High-demand HTTP/URL client surface built on socket/ssl wrappers. |
| 17 | `queue` | 39 | async_concurrency | 5.0 | 4.3 | 5.34 | 30 | `milestone_phase8d_async_concurrency_substrate` | Concurrency primitive needed before higher-level task runtime parity. |
| 18 | `selectors` | 18 | async_concurrency | 5.0 | 3.5 | 6.3 | 18 | `milestone_phase8d_async_concurrency_substrate` | IO multiplexing primitive that unblocks async runtime design. |
| 19 | `contextvars` | 5 | async_concurrency | 5.0 | 2.8 | 7.14 | 23 | `milestone_phase8d_async_concurrency_substrate` | Needed for context propagation semantics in async task execution. |
| 20 | `threading` | 82 | async_concurrency | 5.0 | 5.0 | 4.5 | 300 | `milestone_phase8d_async_concurrency_substrate` | Required baseline for practical concurrency and compatibility. |

## Recommended Execution Order

1. `milestone_phase8a_stdlib_core_parity_i`
2. `milestone_phase8b_stdlib_core_parity_ii`
3. `milestone_phase8c_system_wrappers`
4. `milestone_phase8d_async_concurrency_substrate`

### Dependency rationale

- Run Core Parity I first to establish foundational APIs (`operator`, `struct`, `copy`, `enum`) used by other modules.
- Core Parity II builds on those primitives and expands application-level coverage (`calendar`, `configparser`, `hmac`, `fractions`).
- System wrappers should precede most network-facing parity work (`socket` -> `ssl` -> `urllib`, plus `subprocess`).
- Async/concurrency substrate should follow wrapper and runtime groundwork (`selectors`, `queue`, `contextvars`, `threading`) and align with Phase 8 async milestones.

## Additional Artifacts

- Full second-pass scoring table: `second_pass_module_scoring.csv`
