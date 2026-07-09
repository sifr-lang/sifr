# M8b TLS native-boundary migration review round 1

Please review the current working tree diff for milestone M8b of `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`.

Do not modify files. Report findings only.

## Scope

This milestone migrates `_sifr.tls` from retained compiler-native intrinsics to private Rust interop declarations backed by `sifr_stdlib::tls`.

Key intended changes:

- Add Rust wrappers in `crates/sifr_stdlib/src/tls.rs` that call `sifr_runtime::tls` and bridge `SifrIntBridge` handles.
- Replace marker-only `stdlib/_sifr/tls.sifr` with Rust-decorated declarations returning raw handles or simple values plus `TlsError`.
- Update `stdlib/sifr/tls.sifr` so public TLS config/stream wrapper classes construct from raw handles.
- Make `sifr_runtime::tls::tls_stream_split` return `Result<(i64, i64), String>` instead of phantom handles for unknown streams.
- Remove compiler-retained TLS registry lowering, TLS preamble helpers, and retained fallback signatures.
- Add dependency inference for generated `sifr_stdlib::tls::` references and stdlib `tls` feature selection.
- Move `_sifr.tls` from retained to closing in `internal_docs/stdlib_retained_compiler_intrinsics.toml`.
- Add TLS intrinsic names to `scripts/check_stdlib_migration_closure.py`.

## Validation already run

- `cargo fmt`
- `cargo test -p sifr_runtime --features tls tls_stream_split_rejects_unknown_handle -- --nocapture`
- `cargo test -p sifr_stdlib --features tls -- --nocapture`
- `cargo test -p sifr network_http_dependency_rules -- --nocapture`
- `cargo test -p sifr_retained_intrinsics -- --nocapture`
- `cargo test -p sifr test_generate_cargo_toml -- --nocapture`
- `python3 scripts/check_stdlib_migration_closure.py`
- `cargo run -q -p sifr -- run demos/network_tls_loopback/main.sifr`
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/network_http_tls_loopback_split.sifr`
- Emit check: `cargo run -q -p sifr -- emit demos/network_tls_loopback/main.sifr > /tmp/sifr_tls_emit.rs && if rg "__sifr_tls_" /tmp/sifr_tls_emit.rs; then exit 1; fi && rg "sifr_stdlib::tls::" /tmp/sifr_tls_emit.rs`
- `git diff --check`
- File-size spot check: touched source files remain below 900 lines (`crates/sifr_runtime/src/tls.rs` 793, `crates/sifr_codegen/src/lib_modules_and_codegen.rs` 796).

## Review focus

Please focus on blocking correctness issues:

- Any remaining compiler-owned TLS path or fallback that should have been removed.
- Any generated Cargo dependency gap for `_sifr.tls`, `sifr.tls`, or generated `sifr_stdlib::tls::*` calls.
- Any public API regression in `sifr.tls` from returning wrapper objects via compiler preamble to constructing them in Sifr source.
- Any Rust interop signature mismatch, especially `bytes`, `list[bytes]`, async futures, and `Result[list[int], TlsError]`.
- Any TLS runtime lifecycle issue caused by making split fallible.
- Any missing test coverage that should block this milestone before PR.

Return:

1. Blocking findings with file/line references.
2. Non-blocking suggestions, if any.
3. A final verdict: either "satisfied" or "needs changes".
