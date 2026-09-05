Both pass4 blockers are resolved and the described delta matches the working tree:

- `base32_fallback.rs` / `base64_fallback.rs` are absent from `crates/sifr_codegen/src/intrinsics/registry/` and no source references them (`grep` is empty).
- `crates/sifr_codegen/src/intrinsics/rust_interop_probe.rs` has no diff against HEAD.
- `registry.rs` (lines 737–762) dispatches only the fallible names (`base64_decode`, `base64_decode_bytes`, `base64_encode_opts`, `base64_decode_opts`, `urlsafe_b64decode`, `urlsafe_b64decode_bytes`, `b32decode`, `b32hexdecode`) via the original `base32::`/`base64::` modules. Infallible encoders are gone from active lowering.
- `stdlib/_sifr/crypto.sifr` declares the six encoders as `@rust(sifr_stdlib.base64.*)`; `stdlib/sifr/base64.sifr` imports them and re-exports public encoders plus CPython-compat aliases.
- `sifr_stdlib::base64` implements the encoder/decoder leaf; `_sifr.crypto` manifest adds the `base64` feature; tests assert: (a) the six encoder names lower to `None` while the eight fallible names still lower to `Some` (`registry_extended_tests.rs`); (b) generated private code calls `sifr_stdlib::base64::*encode*` but not `sifr_stdlib::base64::*decode*` (`stateless_private_codegen_tests.rs`); (c) RFC vectors and error paths covered in `api_behavior.rs`.

No new blocker introduced.

VERDICT: PASS
