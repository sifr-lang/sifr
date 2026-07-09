Reviewed the round-3 delta and re-scanned the full M8a diff.

**No blocking findings.**

Round-3 change verification:

- `scripts/check_stdlib_migration_closure.py:140-159` — the 20 `net_*` entries added to `RETIRED_INTRINSICS` exactly match the 20 pre-deletion dispatch arms that were removed from `crates/sifr_codegen/src/intrinsics/registry.rs`. Confirmed by diffing against `HEAD` and counting names in both sets.
- `python3 scripts/check_stdlib_migration_closure.py --self-test` and `python3 scripts/check_stdlib_migration_closure.py` both pass locally (guard reports `retired_intrinsics=257`, `registry_file_deleted=True`).
- The addition is purely a defensive tightening of the closure guard — no runtime or codegen surface changes since round 2. The prior round-2 conclusion ("Round-one blocking findings are all satisfied. No new blocking findings from the round-2 delta.") still holds against the current tree.

Prior non-blocking hygiene notes from round 2 that remain (still non-blocking):

- `tcp_stream_split` in `crates/sifr_runtime/src/net.rs:377-392` still calls `next_handle_infallible()` twice before verifying the stream is present, so a reject burns two handles. Safe because handle allocation is monotonic and non-recycling.
- `bridge_error_expr` alias branch in `crates/sifr_codegen/src/rust_interop_error_mapping.rs:6-17` still hard-codes a single `"message"` field, relying on `is_message_error_alias` restricting to the three known one-field errors (`ProcessError`/`NetError`/`TlsError`).
- `_wrap_accept_tcp` in `stdlib/sifr/net.sifr:283-289` still re-queries `net_tcp_stream_peer_addr` after accept.

The current diff is satisfactory to merge.
