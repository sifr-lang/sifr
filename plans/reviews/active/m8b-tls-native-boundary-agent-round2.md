Round 2 review complete. Two follow-up changes verified against the current tree; both guards run green.

# 1. Blocking findings

None.

## Verification of the two follow-up changes

**Architecture doc (`internal_docs/sifr_sysroot_and_stdlib_architecture.md:129-130`).** The updated Network and TLS rows correctly replace the deleted `preamble/net_runtime.rs` / `preamble/tls_runtime.rs` references with `_sifr.net` / `_sifr.tls` private Rust interop through `sifr_stdlib::{net,tls}` and note the retained substrate in `sifr_runtime::{net,tls}`. This matches the actual end-state of M8a and M8b and resolves the non-blocking staleness flagged in round 1.

**Allowlist script (`scripts/check_stdlib_native_intrinsic_allowlist.py:30-34`).** Both edits are necessary and safe:
- `EXPECTED_PREFIX_DISPATCHERS` now `{"http_", "py_"}`. `crates/sifr_codegen/src/intrinsics/registry.rs` only has `starts_with("http_")` (line 117) and `starts_with("py_")` (line 134) — leaving `tls_` in the expected set would fail the guard as a stale prefix.
- `PREFIX_DISPATCH_LOWERERS` no longer references `registry/tls.rs` — that file is deleted, so leaving it would crash the reader.

Independent verification:
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` — PASS (exact_intrinsics=115, registry_files=15, preamble_files=10, fallback_signature_modules=24, retained_direct_dependency_packages=44, direct_runtime_roots=3).
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py --self-test` — PASS.
- `python3 scripts/check_stdlib_migration_closure.py` — PASS (retired_intrinsics=282, registry_file_deleted=True).
- `_sifr.tls` closing-row invariants hold: `sifr_retained_intrinsics/src/lib.rs` no longer mounts a `tls` module, so the closing-state fallback-module check passes.
- No lingering references to `preamble/tls_runtime.rs` or `intrinsics/registry/tls.rs` in `crates/`, `internal_docs/`, or `scripts/`.

# 2. Non-blocking suggestions

- **Stale traceability report row.** `verification/areas/stdlib_parity/reports/network_http_tls_traceability.md:9` still cites `crates/sifr_retained_intrinsics/src/tls.rs`, `crates/sifr_codegen/src/intrinsics/registry/tls.rs`, and `crates/sifr_codegen/src/preamble/tls_runtime.rs` — all deleted by M8b. That file is anchored to PR #2496 at commit `742ea9f33...` so it reads as historical evidence rather than live docs, but the paths it points to no longer exist. Same shape as the round-1 architecture-doc flag — worth noting in milestone closeout even if you decide to leave the merged-report snapshot untouched.
- **Round-1 non-blocking suggestions unchanged.** The wasted-handles allocation in `crates/sifr_runtime/src/tls.rs:422-423` on the split error path, and the absence of an e2e fixture exercising `TlsStream.split()` failure from Sifr, both still apply. Neither is introduced by round 2.

# 3. Verdict

**Satisfied.** The two round-2 follow-ups are correct, consistent with the M8a/M8b end-state, and unblock the two guards they touch without introducing any new issue. Combined with the round-1 verification that survives unchanged, M8b is ready.
