# M8a Net Native Boundary Review Round 1

Created: 2026-07-09T05:13:44Z

## Request

Review the M8a TCP/network stdlib native-boundary migration on branch
`m8a-net-native-boundary`.

Focus on correctness risks, missed retained compiler glue, generated Cargo/e2e
dependency planning, Rust interop bridge compatibility, handle lifecycle, split
half behavior, cancellation/error semantics, and validation gaps.

Do not modify files. Return findings only, ordered by severity with concrete
file/line references.

## Scope

Changed files:

- `crates/sifr_stdlib/src/net.rs`
- `stdlib/_sifr/net.sifr`
- `stdlib/sifr/net.sifr`
- `crates/sifr_codegen/src/intrinsics/registry.rs`
- deleted `crates/sifr_codegen/src/intrinsics/registry/net.rs`
- `crates/sifr_codegen/src/intrinsics/registry/requirements.rs`
- `crates/sifr_codegen/src/lib.rs`
- `crates/sifr_codegen/src/lib_modules_and_codegen.rs`
- `crates/sifr_codegen/src/preamble.rs`
- deleted `crates/sifr_codegen/src/preamble/net_runtime.rs`
- `crates/sifr_codegen/src/rust_interop_direct.rs`
- `crates/sifr_codegen/src/rust_interop_error_mapping.rs`
- `crates/sifr_retained_intrinsics/src/lib.rs`
- deleted `crates/sifr_retained_intrinsics/src/net.rs`
- `crates/sifr_runtime/src/net.rs`
- `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs`
- `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs`
- `crates/sifr/tests/e2e_support/harness_model.rs`
- `crates/sifr/tests/e2e_support/network_http_dependency_rules_tests.rs`
- `internal_docs/stdlib_retained_compiler_intrinsics.toml`
- `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`

## Summary

- Added `_sifr.net` private native declarations backed by
  `sifr_stdlib::net`.
- Moved public `sifr.net` wrappers to await/call `_sifr.net` declarations and
  construct public `TcpStream`, `TcpListener`, `SocketAddr`, and split-half
  wrappers in Sifr source.
- Implemented `sifr_stdlib::net` as the behavior boundary that delegates only
  low-level socket/table substrate to `sifr_runtime::net`.
- Removed net compiler intrinsic registry, generated net preamble, and retained
  fallback signature module.
- Updated Rust interop error mapping for message-shaped imported `NetError` and
  `TlsError` aliases/classes.
- Updated e2e dependency inference so generated `sifr_stdlib::net::` calls pull
  `sifr_stdlib` with the `net` feature in batched e2e Cargo manifests.
- Moved `_sifr.net` manifest row from `retained` to `closing`.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test -p sifr_stdlib --features net net -- --nocapture`
- `cargo test -p sifr_codegen rust_interop_direct -- --nocapture`
- `cargo test -p sifr_retained_intrinsics -- --nocapture`
- `cargo test -p sifr network_http_dependency_rules -- --nocapture`
- `cargo test -p sifr test_generate_cargo_toml -- --nocapture`
- `cargo run -q -p sifr -- run demos/network_tcp_echo/main.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_loopback_split.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_errors.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_cancel_accept.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tls_loopback_split.sifr`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
- `scripts/run_all_tests.sh --profile create-pr`
  - Result: pass, 129 e2e pass fixtures, 0 failures
  - Report: `target/validation_lane_reports/create-pr.latest.json`
  - Advisory only: warm wall-time budget exceeded

## Known Prior Reviewer Tool State

In earlier M7 rounds, `claude --model claude-opus-4-7 --effort xhigh --print`
timed out with no output, and `--bare` reported not logged in. Try the required
reviewer command again for this round; if it times out, record the timeout and
perform local fallback review.
