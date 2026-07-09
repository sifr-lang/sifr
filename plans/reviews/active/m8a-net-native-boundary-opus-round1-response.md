Reviewed the M8a net native-boundary migration. Six findings, ordered by severity:

1. **Certification handoff missed** — `_sifr.net` closing row still cites `opaque_resource_matrix` / `async_runtime_reqwest` (future-owned ecosystem rows) instead of the stdlib-owned `opaque_resource_core` / `async_runtime_core` rows the plan mandates and that sibling migrations (`_sifr.time`, `_sifr.fs`) already use. Compatibility matrix and cert-issue plan are not updated.

2. **Split-half two-call side channel** — `TcpStream.split` was atomic (one Rust call returning `(TcpReadHalf, TcpWriteHalf)`); it is now split across `net_tcp_stream_split_read` + `net_tcp_stream_split_write` coordinated by a global `PENDING_SPLIT_WRITES` HashMap. Leak on failure between the two calls, `-1` sentinel on missing keys aliasing the general handle namespace, and inconsistent `to_i64_saturating` handle conversion versus the checked `bridge_i64` used everywhere else in the same file.

3. **`tcp_stream_split` silently returns bogus handles** — When the input stream isn't in `STREAMS`, the runtime still allocates fresh read/write handles that are never inserted into the halves tables. `split()` appears to succeed and the failure surfaces later as "TCP read half handle is closed or unknown: N", pointing at the wrong root cause. Pre-existing but in scope for M8's split-lifecycle acceptance criteria.

4. **Preemptive TLS dependency inference** — `harness_model.rs`, `fixture_dependency_paths.rs`, `fixture_cargo_toml.rs` now handle `sifr_stdlib::tls::` and `sifr.tls`/`_sifr.tls`, but `sifr_stdlib::tls` is empty and no generated code emits that prefix. Dead branches out of M8a scope; will silently drift if M8b renames the module.

5. **`_wrap_accept_tcp` re-queries the peer address** — Runtime `accept_tcp` returns `(handle, peer_addr)`; the stdlib wrapper discards `peer_addr` and Sifr code then calls `net_tcp_stream_peer_addr` as a second bridge call per accept. Not incorrect, just wasteful.

6. **`bridge_error_expr` alias-branch trusts the name alone** — Now that `NetError`/`TlsError` join `ProcessError` in `is_message_error_alias`, the alias short-circuit emits `StructInit { name, fields: [("message", ...)] }` without resolving the alias or checking `message_error_fields`. Rust would catch a shape mismatch at compile time, but the class-branch already has the correct guard — the alias branch should mirror it.
