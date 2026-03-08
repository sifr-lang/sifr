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
| bytes | encode/decode/hex conversion and byte-search helper subset | done | parity | CPython-derived behavior subset is validated with canonical vector fixtures and safety-adapted assertions | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit when broader CPython bytes object-model surface is promoted into phase scope | `lib/sifr/bytes.sifr`, `crates/sifr/tests/e2e/pass/cpython_bytes_subset.sifr`, `demos/m30_1a_bytes_parity_demo/main.sifr` |
| bytes | object-model surface uses `list[int]` adapters instead of CPython `bytes` objects | done | intentional-diff | Current Sifr stdlib surface is list-backed and safety-adapted rather than full CPython bytes object parity | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit when bytes object-model parity expansion is scheduled | `lib/sifr/bytes.sifr`, `crates/sifr_hir/src/stdlib/collections_bytes_time.rs`, `crates/sifr/tests/e2e/pass/cpython_bytes_subset.sifr` |
| base64 | base64/base32/base16 encode-decode behavioral subset | done | parity | CPython-derived base64 subset behaviors are validated with canonical vector fixtures | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit when additional CPython option flags/surfaces are brought into approved phase scope | `lib/sifr/base64.sifr`, `crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`, `demos/m30_1a_base64_parity_demo/main.sifr` |
| base64 | decode error signaling uses `Result[..., ParseError]` rather than CPython exceptions | done | intentional-diff | Sifr safety contract uses typed Result/Option adaptation instead of exception control flow | phase_30 execution loop | `issues/phase30-reliability-parity-and-performance-budgets-execution.md` | Revisit if typed stdlib error-class expansion changes base64 error model | `lib/sifr/base64.sifr`, `crates/sifr/tests/e2e/pass/cpython_base64_subset.sifr`, `crates/sifr/tests/e2e/pass/parse_safety_error_paths.sifr` |
