# M6 typed IPC dependency metadata — review pass 1

Verdict: **PASS**

Branch: `codex/concurrency-runtime-m6-ipc-deps`
Diff base: `origin/main`
Touched files: 4 source + 1 ledger (`crates/sifr_stdlib/src/features.rs`, `crates/sifr/tests/e2e_support/fixture_compilation.rs`, `crates/sifr/tests/e2e_support/harness_model.rs`, `crates/sifr/tests/e2e_support/harness_behavior_tests.rs`, `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`).

## Scope discipline

- The diff is limited to dependency-metadata plumbing and the grouped e2e harness's generated-Cargo.toml inference for typed IPC. No public process-worker APIs are added; no runtime IPC frame/encoder/decoder implementation lands here.
- `StdlibFeature::Ipc` is introduced as an internal feature variant alongside its `cargo_name`, `cargo_dependencies`, codegen-requirement aliases, and module mapping — no exported runtime surface.
- Runtime diagnostics wiring is preserved: `sifr.runtime` / `_sifr.runtime` still resolve to `[StdlibFeature::Metrics, StdlibFeature::Tracing]`, and the `sifr.runtime` / `tracing` arms in `generate_cargo_toml` are only refactored to share the existing `TRACING_DEP` constant (no version, feature-flag, or default-features change).

## Locked dependency specs

- `IPC_DEPS` in `crates/sifr_stdlib/src/features.rs` renders exactly:
  - `postcard = { version = "1.1.3", default-features = false, features = ["use-std"] }`
  - `serde = { version = "1.0.228", features = ["derive"] }`
- These match the approved design and phase doc; no `serde_json`, `bincode`, or alternative codec slips in.
- `feature_for_codegen_requirement` aliases both `"ipc"` and `"postcard"` onto `StdlibFeature::Ipc`, and `features_for_stdlib_module` maps both `"sifr.ipc"` and `"_sifr.ipc"` onto `[StdlibFeature::Ipc]`.

## Grouped e2e harness inference

- `crates/sifr/tests/e2e_support/harness_model.rs::infer_dependencies` now detects `postcard::` and `use postcard` in generated Rust and adds `"postcard"` to the required-crate set — mirroring the existing `serde_json` / `metrics` detection style.
- `crates/sifr/tests/e2e_support/fixture_compilation.rs::generate_cargo_toml` adds:
  - a `sifr.ipc` / `_sifr.ipc` module arm that inserts `POSTCARD_DEP` + `SERDE_DEP` (no `serde_json`), and
  - a `"postcard" | "ipc"` required-crate arm that inserts the same locked pair.
- The four spec strings (`POSTCARD_DEP`, `SERDE_DEP`, `SERDE_JSON_DEP`, `TRACING_DEP`) are de-duplicated into file-level constants without changing the rendered strings.
- The new contract test `test_generate_cargo_toml_ipc_uses_locked_postcard_specs` exercises both the module-driven and required-crate-driven paths and asserts `serde_json` is absent in both renders.

## File-size guardrail

- `wc -l` confirms touched-file line counts match the brief: `fixture_compilation.rs` 900, `features.rs` 894, `harness_model.rs` 790, `harness_behavior_tests.rs` 872 — all within the 900-line cap (fixture_compilation sits exactly at the cap; the constant extraction makes that headroom honest rather than monkey-patched).
- `python3 scripts/check_file_size_guardrails.py` PASS was reported on 2246 files at the 900-line limit.

## Ledger

- The issue file appends two clearly scoped sections: "M6 typed IPC dependency metadata implementation" and "M6 typed IPC dependency metadata targeted local validation". Wording is explicit that this wave covers dependency metadata only — no claim of full M6 typed IPC completion, no claim about runtime frame APIs, and no claim that downstream wiring is in place. Validation commands quoted match the runs reported in the task brief.

## Validation re-checked from artifacts

- `cargo test -p sifr_stdlib ipc_feature_renders_locked_postcard_specs_without_json -- --nocapture`: PASS (reported).
- `cargo test -p sifr --test e2e test_generate_cargo_toml_ipc_uses_locked_postcard_specs -- --nocapture`: PASS (reported).
- `cargo fmt --check`, `git diff --check`, `python3 scripts/check_file_size_guardrails.py`: PASS (reported).

## Findings

None. The change is tightly scoped to internal dependency-metadata plumbing for Ring 4 typed IPC, the locked specs match the approved design, the grouped e2e harness now infers and renders the IPC pair from both module metadata and explicit required crates, no `serde_json` is pulled in, runtime diagnostics wiring is unaltered, the 900-line file-size cap is respected, and the ledger records implementation and validation without overclaiming M6 completion.

Verdict: **PASS**.
