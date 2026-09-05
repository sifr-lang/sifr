# M8a Net Native Boundary Review Round 2

Created: 2026-07-09T05:37:38Z

## Request

Review the updated M8a TCP/network stdlib native-boundary migration on branch
`m8a-net-native-boundary`.

This is round 2 after the findings in
`plans/reviews/active/m8a-net-native-boundary-agent-round1-response.md`.

Do not modify files. Return blocking findings only, ordered by severity with
concrete file/line references. If the round-one blocking findings are satisfied,
say so explicitly.

## Round 1 Findings and Response

1. Certification handoff missed.
   - Fixed: `_sifr.net` now uses `opaque_resource_core` and
     `async_runtime_core` in
     `internal_docs/stdlib_retained_compiler_intrinsics.toml`.
   - Fixed: M8 status text in
     `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`
     now records the core certification row handoff.

2. Split-half two-call side channel.
   - Fixed: removed the `PENDING_SPLIT_WRITES` global side channel from
     `crates/sifr_stdlib/src/net.rs`.
   - Fixed: `_sifr.net` now exposes one native `net_tcp_stream_split` call
     returning `Result[list[int], NetError]`.
   - Fixed: public `TcpStream.split` now returns
     `Result[tuple[TcpReadHalf, TcpWriteHalf], NetError]` and validates the
     returned handle list before constructing halves.

3. `tcp_stream_split` silently returns bogus handles.
   - Fixed: `sifr_runtime::net::tcp_stream_split` now returns
     `Result<(i64, i64), String>` and rejects unknown/closed stream handles at
     the split call.
   - Added runtime regression test
     `tcp_stream_split_rejects_unknown_handle`.

4. Preemptive TLS dependency inference.
   - Fixed: removed `sifr_stdlib::tls::` inference and `sifr.tls`/`_sifr.tls`
     stdlib dependency branches from the M8a e2e harness changes.

5. `_wrap_accept_tcp` re-queries peer address.
   - Not changed in this slice. This is a non-blocking extra bridge call; the
     accepted stream handle remains correct, and public accept still returns the
     peer address via `net_tcp_stream_peer_addr`.

6. `bridge_error_expr` alias branch trusts the name alone.
   - Fixed: the alias branch now checks that the alias body resolves to a class
     with a string `message` field before emitting message-shaped conversion.

## Current Scope

Changed files include:

- `crates/sifr_stdlib/src/net.rs`
- `crates/sifr_runtime/src/net.rs`
- `stdlib/_sifr/net.sifr`
- `stdlib/sifr/net.sifr`
- `demos/network_tcp_echo/main.sifr`
- `crates/sifr/tests/e2e/pass/network_http_tcp_loopback_split.sifr`
- `crates/sifr_codegen/src/rust_interop_error_mapping.rs`
- `crates/sifr/tests/e2e_support/fixture_cargo_toml.rs`
- `crates/sifr/tests/e2e_support/fixture_dependency_paths.rs`
- `crates/sifr/tests/e2e_support/harness_model.rs`
- `crates/sifr/tests/e2e_support/network_http_dependency_rules_tests.rs`
- net retained compiler glue deletions under `crates/sifr_codegen` and
  `crates/sifr_retained_intrinsics`
- `internal_docs/stdlib_retained_compiler_intrinsics.toml`
- `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`

## Fresh Validation

Passed after round-one fixes:

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
- `python3 scripts/check_stdlib_migration_closure.py`
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py`
- `python3 scripts/check_sysroot_stdlib_resource_certification_gate.py`
