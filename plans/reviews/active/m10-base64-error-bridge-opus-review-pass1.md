Based on my comprehensive review, here is the verdict:

## VERDICT: PASS

The M10 wave 1 migration correctly moves fallible base64/base32 decode and option helpers from compiler intrinsic fallback to private `_sifr.crypto` Rust interop declarations backed by `sifr_stdlib.base64`, and establishes the typed direct Result/error bridge.

### Key correctness verifications

1. **Public API shape preserved** — `stdlib/sifr/base64.sifr` wraps every migrated private import behind an `_impl` alias (`_base64_decode_impl`, `_b32hexdecode_impl`, etc.), so borrowed Rust interop parameter conventions cannot leak into public call sites, matching the established hash/math pattern.

2. **Result/error bridge is bounded** — `rust_interop_direct.rs:91-106` only constructs the message-shaped error class when the Sifr err_type is a class with `parent_class = Some("Error")` *and* a `Str message` field; otherwise it falls through. Combined with the probe at `rust_interop_probe.rs:392-401` (only allows `__SifrBridgeError: Display` generic when the err type contains `__sifr_bridge` AND ends with `Bridge`), the bridge cannot silently accept unbounded Rust error shapes.

3. **All migrated names removed from active intrinsic dispatch** — `registry.rs` no longer imports `base32`/`base64` modules; `registry_extended_tests.rs:440-469` asserts that all 14 base64/base32/urlsafe/b32hex names return `None` from `lower_intrinsic`. Deleted `registry/base64.rs` (800 LOC) and `registry/base32.rs` (625 LOC) confirm removal.

4. **Direct generated `base64` dependency eliminated** — `BASE64_DEPS = &[]`, `features_for_stdlib_module("sifr.base64") => &[]`, and `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` asserts `sifr.base64` emits exactly one dep (`sifr_stdlib = { features = ["base64"] }`).

5. **Codegen evidence covered** — `crypto_hash_private_declarations_codegen_through_sifr_stdlib` (stateless_private_codegen_tests.rs:197-214) verifies every fallible name lowers to `sifr_stdlib::base64::*` and that the rendered private code contains the `ParseError { message: __sifr_bridge_error.to_string() }` shape.

6. **`SifrIntBridge` arg lowering** — `base64_encode_opts` Rust signature now accepts `SifrIntBridge` for `wrapcol` (base64.rs:34), matching the `direct_rust_arg_expr` `Type::Int → SifrIntBridge::from(...)` lowering.

7. **Tracker accuracy** — M9 row is updated to record the M10 wave 1 work and the wave 6 explicit deferral; M10 wave 1 evidence is scoped to base64/base32 fallible helpers only, with regex/TOML/JSON/encoding/Unicode/i18n/URL/gzip/zipfile correctly left as pending. Architecture doc now reads "full base encoding helper subset" (no longer "encoder subset") and adds the Result/E:Display → message-shaped error class statement.

### Non-blocking suggestions (do not block PR)

- **Unnecessary `.map(|x| x)` identity on Ok branch** (`rust_interop_direct.rs:59-73`) — when `direct_rust_return_expr` produces an identity transform (e.g., `Result[bytes, ParseError]` Ok type doesn't need conversion), `bridge_result_expr` still wraps with `.map(|__sifr_bridge_ok| __sifr_bridge_ok)`. Generated `base64_decode_bytes` lowers to `…map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(…)`. Skipping `.map` when the body is structurally `Ident == ok_name` would be cleaner; functionally inert today.

- **`_sifr.crypto` → `[Rand, RandDistr]` + `["base64", "hash", "random"]` mappings still active** (`features.rs:605`, `generated_stdlib_features.rs:50`) — because random helpers reach `_sifr.crypto` through intrinsic fallback (bootstrap re-exports), `sifr.base64`-only programs that transitively pull `_sifr.crypto` still enable the `random` sifr_stdlib feature and compile `rand`/`rand_distr` crates that go unused. Pre-existing; deferred until random's stateful surface migrates per the recorded plan.

- **Probe parser uses `rsplit_once(", ")` on the Result inner** (`rust_interop_probe.rs:396`) — handles the simple shapes today (`String`, `Vec<u8>` Ok types) since the err side is single-segment. If future Ok types embed `, ` (e.g., `HashMap<K, V>`), the split still works because of `rsplit_once`, but a more structural parse would be more robust. Not exercised by current wave.
