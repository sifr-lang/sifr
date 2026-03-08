# Phase 30 Parity Matrix and Waiver Inventory

Canonical columns for every module behavior row:

- `module`
- `behavior`
- `status` (`done` | `open`)
- `classification` (`parity` | `intentional-diff` | `unsupported`)
- `rationale`
- `owner`
- `tracking_issue`
- `revisit_rule`
- `evidence`

| module | behavior | status | classification | rationale | owner | tracking_issue | revisit_rule | evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| env | missing-key behavior without explicit default | done | intentional-diff | CPython exposes `os.getenv(key)` directly; Sifr currently exposes explicit no-default path as `getenv_opt(key)` because imported-function default arguments are not yet applied at call sites | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit when imported-function default-argument lowering is implemented | `lib/sifr/env.sifr`, `crates/sifr/tests/e2e/pass/cpython_env_subset.sifr`, `demos/m30_1a_env_parity_demo/main.sifr` |
| env | invalid env keys (`""`, contains `"="`) for set/get paths | done | intentional-diff | CPython can raise on invalid environment names; Sifr safety contract forbids panic/exception control-flow and keeps invalid-key handling panic-free (`None`/no-op) | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit if typed stdlib error modeling for env is introduced | `crates/sifr/tests/e2e/pass/cpython_env_subset.sifr`, `crates/sifr_codegen/src/intrinsics/env.rs`, `demos/m30_1a_env_parity_demo/main.sifr` |
