# M8b TLS Native-Boundary Review — Round 3

## Verification of Round 2 doc follow-up

I verified every path reference the user updated in `verification/areas/stdlib_parity/reports/network_http_tls_traceability.md`:

- Row 1 — `stdlib/sifr/tls.sifr` (present), `stdlib/_sifr/tls.sifr` (present), `crates/sifr_stdlib/src/tls.rs` (present; contains 25 pub wrapper fns), `crates/sifr_runtime/src/lib.rs` (has `#[cfg(feature = "tls")] pub mod tls;` gate at line 18), `crates/sifr_runtime/src/tls.rs` (present).
- Row 7 (TLS full-duplex split) — `TlsStream.split()` at `stdlib/sifr/tls.sifr:130`; runtime split at `crates/sifr_runtime/src/tls.rs:421`; e2e fixture at `crates/sifr/tests/e2e/pass/network_http_tls_loopback_split.sifr` (present).
- Row 9 (Typed TLS/certificate errors) — `TlsError` at `stdlib/_sifr/tls.sifr:4`; `CertificateError` at `stdlib/sifr/tls.sifr:33`.

Stale-path grep across `internal_docs`, `verification/areas/stdlib_parity/reports`, `scripts`, and `crates` for `lib/sifr/tls`, `intrinsics/registry/tls`, `preamble/tls_runtime`, and `sifr_retained_intrinsics/src/tls`: only the report row itself and the valid `include_str!("../../../stdlib/sifr/tls.sifr")` in `sifr_stdlib_manifest/src/sources.rs:189` match — no stale references.

The rest of the M8b diff (registry/preamble deletions, `_sifr.tls` closure marker in `internal_docs/stdlib_retained_compiler_intrinsics.toml`, migration-closure/allowlist script updates, e2e harness dependency inference for `_sifr.tls`, `tls_stream_split` now returning `Result`) is coherent with what round 2 already accepted; no new blocker introduced.

## 1. Blocking findings

None.

## 2. Non-blocking suggestions

- `verification/areas/stdlib_parity/reports/network_http_tls_traceability.md:27` — the validation-evidence row still says `... and TLS lowerer/preamble compilation`, but the TLS lowerer (`intrinsics/registry/tls.rs`) and preamble (`preamble/tls_runtime.rs`) are deleted. This row appears to record the historical PR #2496 merge validation (line 3 marks the report as merged at `742ea9f3…`), so it may be intentionally frozen. If treated as historical it can stay; if the report is intended to reflect current architecture, replace with something like `... and TLS interop declaration / stdlib wrapper compilation`. Non-blocking either way.

## 3. Final verdict

**satisfied**
