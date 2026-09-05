Reviewed the M8a round-2 changes against each round-1 finding and scanned the current diff for new blocking issues.

**Round-one blocking findings are all satisfied.**

Verification against round 1:

1. **Certification handoff (fixed)** — `internal_docs/stdlib_retained_compiler_intrinsics.toml:188` now lists `certification_rows = ["opaque_resource_core", "async_runtime_core"]` for `_sifr.net`, and the issue plan (`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:845-848`) records the row handoff.

2. **Split-half two-call side channel (fixed)** — `PENDING_SPLIT_WRITES` is gone from `crates/sifr_stdlib/src/net.rs`. A single call `net_tcp_stream_split` (`crates/sifr_stdlib/src/net.rs:127-132`) returns `Result<Vec<SifrIntBridge>, String>` using the checked `bridge_i64` conversion, and the Sifr wrapper validates the returned list (`stdlib/sifr/net.sifr:73-87`).

3. **`tcp_stream_split` silently returning bogus handles (fixed)** — `crates/sifr_runtime/src/net.rs:377-392` now returns `Result<(i64, i64), String>` and errors when the stream handle is unknown; regression test `tcp_stream_split_rejects_unknown_handle` covers it (`crates/sifr_runtime/src/net.rs:504-513`).

4. **Preemptive TLS dependency inference (fixed)** — `sifr_stdlib::tls::` inference removed from `crates/sifr/tests/e2e_support/harness_model.rs:398-507` and `_sifr.tls`/`sifr.tls` no longer appear under `sifr_stdlib_dependency_spec_for_modules` or `needs_sifr_stdlib_module_dependency`. The remaining `_sifr.tls` runtime-feature references (`fixture_dependency_paths.rs:12-19`, `fixture_cargo_toml.rs:265-272`) correctly gate the still-retained `sifr_runtime::tls` surface, matching the `retained` state of `_sifr.tls` in the manifest.

5. **`_wrap_accept_tcp` peer address re-query** — Unchanged, as declared. Non-blocking; the accepted stream handle and peer address remain correct.

6. **`bridge_error_expr` alias branch (fixed)** — `crates/sifr_codegen/src/rust_interop_error_mapping.rs:6-17` now guards the alias branch with `body.resolve_alias()` matching a `Type::Class` where `is_message_error_alias(name) && message_error_fields(fields).is_some()`, mirroring the class branch's shape check.

No new blocking findings from the round-2 delta. Minor non-blocking hygiene notes (not blocking): `next_handle_infallible` consumes two handles before the stream-existence check in `tcp_stream_split`; the alias branch of `bridge_error_expr` still hard-codes a single `"message"` field (safe today because `is_message_error_alias` restricts to `ProcessError`/`NetError`/`TlsError`, all one-field); and the deleted `net_*` intrinsic names aren't added to `RETIRED_INTRINSICS` in `scripts/check_stdlib_migration_closure.py`, so a future regression re-registering them wouldn't be caught by that guard.
