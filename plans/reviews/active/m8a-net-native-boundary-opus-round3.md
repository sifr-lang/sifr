# M8a Net Native Boundary Review Round 3

Created: 2026-07-09T05:48:08Z

## Request

Final review of the current M8a TCP/network stdlib native-boundary migration on
branch `m8a-net-native-boundary`.

Round 2 said: "Round-one blocking findings are all satisfied" and "No new
blocking findings from the round-2 delta."

After round 2, I applied one non-blocking hygiene fix from your notes:

- Added the deleted `net_*` compiler dispatch names to
  `RETIRED_INTRINSICS` in `scripts/check_stdlib_migration_closure.py`.
- Validation passed:
  - `python3 scripts/check_stdlib_migration_closure.py --self-test`
  - `python3 scripts/check_stdlib_migration_closure.py`

Do not modify files. Review the current full diff and return blocking findings
only, ordered by severity with concrete file/line references. If the current
diff is satisfactory, say explicitly that there are no blocking findings.

## Current M8a Summary

- `_sifr.net` declarations route through `sifr_stdlib::net`.
- Public `sifr.net` wrappers construct public resource wrappers in Sifr source.
- Compiler net intrinsic registry/preamble/fallback signature glue is deleted.
- `sifr_runtime::net` remains the low-level socket table/TCP substrate.
- `TcpStream.split` is now a single native boundary call returning
  `Result[list[int], NetError]` privately and
  `Result[tuple[TcpReadHalf, TcpWriteHalf], NetError]` publicly.
- `_sifr.net` manifest row is `closing` and points to
  `opaque_resource_core`/`async_runtime_core`.
- E2E generated Cargo dependency inference now recognizes
  `sifr_stdlib::net::` and enables `sifr_stdlib` with the `net` feature.

## Validation Since Round-One Fixes

Passed:

- `cargo fmt --check`
- `cargo test -p sifr_runtime --features net tcp_stream_split_rejects_unknown_handle -- --nocapture`
- `cargo test -p sifr_stdlib --features net net -- --nocapture`
- `cargo test -p sifr_codegen rust_interop_direct -- --nocapture`
- `cargo test -p sifr network_http_dependency_rules -- --nocapture`
- `cargo run -q -p sifr -- run demos/network_tcp_echo/main.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_loopback_split.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_errors.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tcp_cancel_accept.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tls_loopback_split.sifr`
- `python3 scripts/check_file_size_guardrails.py`
- `python3 scripts/check_stdlib_migration_closure.py --self-test`
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
