# TypeScript-Go Architecture Transfer M13: LSP Cancellation, Progress, And Watchdog

status: in progress

M13 makes LSP work operationally bounded while keeping execution serialized.
The server now creates an explicit `CancellationToken` for the active request and
tracks cancellation separately for queued and in-flight requests: queued
requests can be removed before dispatch, while in-flight requests retain a
cancellation marker that phase-boundary checks observe before and after
compiler-service work.

The scheduler remains a lane classifier. Cancellation state lives in
`RequestQueue` and `Session`, which keeps request priority separate from
publication validity. `Session::with_document_analysis` now checks the active
request before and after analysis work, and the server suppresses a completed
result if the request was canceled before response publication.
Because M13 keeps the LSP loop serialized, in-flight cancellation cannot be
observed from stdio while a synchronous compiler-service call is running; those
phase-boundary checks are the deterministic propagation point for later worker
lanes.

Delayed work progress is gated by workload size so single-document fast editor
paths stay quiet. Clients that advertise `window.workDoneProgress` can receive
`$/progress` begin/end notifications for multi-document full diagnostics. The
progress state also reserves stable kinds for references, index warming, and
workspace loading as later milestones make those operations genuinely
long-lived.

`sifr lsp --stdio --parent-pid <pid>` wires an explicit Unix parent watchdog
into the server options. The watchdog checks parent liveness at message
boundaries and cancels the server loop if the parent process is gone. On
non-Unix targets the option is accepted but currently treated as a no-op until a
platform-specific process liveness API is wired.

Validation so far:

- `cargo fmt --check` -> PASS
- `cargo build -p sifr` -> PASS
- `cargo test -p sifr_lsp` -> PASS, 23 tests
- `cargo clippy -p sifr_lsp -p sifr -- -D warnings` -> PASS
- `python3 -m py_compile verification/tooling/lsp_protocol.py verification/tooling/lsp_protocol_smoke.py verification/tooling/lsp_protocol_stress.py` -> PASS
- `python3 verification/tooling/lsp_protocol_smoke.py` -> PASS
- `python3 verification/tooling/lsp_protocol_stress.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py` -> PASS
- `python3 verification/tooling/check_phase36_closeout.py --self-test` -> PASS
- `python3 scripts/check_diagnostic_cancel_usage.py` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> SATISFIED with residual cleanup (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED with residual cleanup (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time 280.41s, advisory: group skew is high
