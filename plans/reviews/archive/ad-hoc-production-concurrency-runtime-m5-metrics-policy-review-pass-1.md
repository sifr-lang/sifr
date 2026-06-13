## PASS

All five scope items verified against the diff vs `origin/main`:

**1. `metrics = 0.24.6` as stable StdlibFeature + Cargo dep wiring** ✓
- `crates/sifr_stdlib/src/features.rs:19` adds `StdlibFeature::Metrics` (alphabetical Md5 < Metrics < NumBigint), `as_str() = "metrics"`, `METRICS_DEPS = "metrics = \"0.24.6\""`, registered in `STDLIB_FEATURE_SPECS`, threaded through `feature_for_codegen_requirement("metrics")` and `features_for_stdlib_module("sifr.runtime"|"_sifr.runtime") = [Metrics, Tracing]`.

**2. `runtime_emit_diagnostic` requires Tracing + Metrics; counters on accept/reject** ✓
- `registry.rs:75`: `"runtime_emit_diagnostic" => &[StdlibFeature::Metrics, StdlibFeature::Tracing]`.
- `registry/runtime.rs`: each of trace/debug/info/warn/error branches calls `metrics::counter!("sifr.runtime.diagnostic.emitted", "level" => "<level>", "surface" => "runtime").increment(1)` after the `tracing::event!` and before `Ok(())`.
- Default branch (`_`) increments `"sifr.runtime.diagnostic.rejected"` with `"reason" => "unsupported_level"`, `"surface" => "runtime"` *before* returning `DiagnosticError`.

**3. Low-cardinality, redacted labels** ✓
- Accepted: only `level` (fixed set `{trace,debug,info,warn,error}`) and `surface = "runtime"` — cardinality ≤ 5.
- Rejected: only `reason = "unsupported_level"` and `surface = "runtime"` — cardinality 1.
- No `diagnostic_target`, `diagnostic_name`, `diagnostic_message`, or user-controlled `__sifr_diagnostic_level` text flows into any label. The rejected branch deliberately uses the constant `"unsupported_level"` instead of the raw level string.

**4. Fixture dep inference picks up `metrics`** ✓
- `harness_model.rs:460`: detects `metrics::` / `use metrics` in generated Rust, inserts `"metrics"` into required crates.
- `fixture_compilation.rs:308` adds `metrics = "0.24.6"` under `sifr.runtime` / `_sifr.runtime` module, and `:429` handles `"metrics"` as an explicit required-crate spec.

**5. Tests/docs/traceability honestly record the policy** ✓
- `registry_core_tests.rs`: renamed tests to `..._with_observability_metadata` / `..._includes_observability_facades`; assert both `Metrics` and `Tracing` features, both `tracing::event!` and `metrics::counter!`, both counter names, `"reason" => "unsupported_level"`, `"surface" => "runtime"`, and the expected `vec![metrics, tracing]` cargo deps order.
- `harness_contract_tests.rs`: `test_generate_cargo_toml_runtime_diagnostics_use_locked_observability_specs` asserts both `metrics = "0.24.6"` and the tracing spec via both module-driven and required-crate paths.
- `verification/stdlib/concurrency_runtime_m5_shutdown_traceability.md`: records both metric names, full label sets and value enums, surface/reason constants, exclusion rationale (user-controlled / sensitive / high-cardinality), histogram deferral on future duration-bearing schema, redaction policy, updated test references in both Create-PR and Merge lanes.
- `issues/...-substrate.md`: Ring 3 row and quick-reference row updated to "fixed-schema counters"; histogram deferral explicit.
- `verification/platform/supported_host_matrix.md`: counters declared host-independent alongside tracing events.

Validation evidence stated in the request (fmt, three targeted cargo tests, e2e fixture run, file-size guardrail, `run_all_tests.sh --profile create-pr` 123/0) is consistent with the diff.

Minor observation (not a failure): the untracked file `reviews/ad-hoc-production-concurrency-runtime-m5-metrics-policy-review-pass-1.md` is 0 bytes. It's outside the verification scope but worth either populating or removing before commit.
