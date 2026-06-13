PASS

## Findings

Verified the three post-review changes against the actual files:

1. **BoolLiteral erasure comment** — `crates/sifr_lowering/src/lower/expressions/regular_calls.rs:213-215` reads: "This marker is checked entirely during lowering. Emit a concrete expression so statement-position calls do not generate an ambiguous Rust `None` literal before the generated schema extractor exists." Directly addresses the first review's tidy-up follow-up about the `HirExpr::BoolLiteral(true)` vs declared `None` return drift.

2. **Enum clarification** — `crates/sifr_lowering/src/lower/ipc_payload_calls.rs:113` carries the comment "Sifr enums currently carry integer-backed variants, not typed payload fields." paired with `Type::Enum { .. } => None`. The new test case at `ipc_payload_calls.rs:203-206` adds a `Type::Enum { name: "Color", variants: [("RED", Some(1)), ("BLUE", Some(2))] }` payload to the `accepts_initial_payload_families` set — confirming variant payloads in Sifr are integer-backed only, which justifies skipping variant-field recursion. Addresses the first review's "non-IPC-serializable variant field could slip through" follow-up.

3. **Ledger evidence** — `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md` now records:
   - "Reviewer pass 1: `reviews/ad-hoc-production-concurrency-runtime-m6-ipc-payload-diagnostics-review-pass-1.md` -> PASS; non-blocking enum/erasure clarity follow-ups addressed with an enum unit case and compiler-erased marker comment."
   - "Post-review focused validation: `cargo fmt --check` and `cargo test -p sifr_lowering ipc_payload_calls -- --nocapture` -> PASS."
   - Prior create-pr lane PASS with `125 passed`, `report_signature=50edc954137c87b4`.

No regressions to the first review's acceptance items:
- `SIFR-OWN-0013` registry, docs page, message template, and ACTIVE_DIAGNOSTIC_CODES wiring intact.
- Accept/reject taxonomy in `ipc_payload_calls.rs` unchanged in behavior; class cycle guard via `visiting` HashSet preserved.
- `require_serializable` interception still occurs in `regular_calls.rs:202-217` before type-checking; statement-position lowering remains safe with `BoolLiteral(true)` placeholder.
- Three e2e fixtures (`ipc_payload_require_serializable_basic.sifr` accepted; `ipc_payload_process_resource_rejected.sifr` and `ipc_payload_sync_endpoint_rejected.sifr` both expect `SIFR-OWN-0013`) still aligned with `expect-error` markers.
- Design doc (`verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md`) and supported-host matrix (`verification/platform/supported_host_matrix.md`) explicitly keep generated schema extraction, public connection/worker APIs, generated worker integration, and Windows process-pipe fixtures as follow-up — no scope overclaim.

The changes are comment-only/test-additive on top of the first PASS review; no correctness, diagnostic-contract, test-coverage, or documentation-overclaim issues introduced.
