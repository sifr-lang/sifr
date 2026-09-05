## Verdict: **PASS**

The migration cleanly executes the wave-6 policy:

- Public API shape preserved: `Encoding`, `DecodeOutcome`, `EncodeOutcome`, `DecodeError`, `EncodeError`, `Decoder`, `Encoder` all still live in `stdlib/sifr/encoding.sifr` (stdlib/sifr/encoding.sifr:41-365).
- Private declarations bind via direct `@rust(sifr_stdlib.encoding.*)` with `panic=trusted_no_panic` and use `ParseError` as message-only transport (stdlib/_sifr/encoding.sifr:4-50).
- Public wrappers translate `ParseError` → `DecodeError`/`EncodeError` at the wrapper boundary (stdlib/sifr/encoding.sifr:110-188).
- No callee injection, `@rust.via`, `bridge.*`, or converter pipelines are introduced.
- `crates/sifr_stdlib/src/encoding.rs` owns the adapter boundary behind `feature = "encoding"` and delegates to `sifr_runtime::encoding` (crates/sifr_stdlib/src/encoding.rs:9-82).
- Compiler intrinsics for the 10 public helpers were retired from `registry.rs`/`registry/encoding.rs`; only `lower_str_encode_result` / `lower_bytes_decode_result` remain for compiler-owned `str.encode`/`bytes.decode` language glue with `ParseError` typing.
- Generated Cargo no longer emits direct `encoding_rs`; `sifr_stdlib` with feature `encoding` is emitted instead. This is enforced by `text_i18n_feature_dependency_snapshots_cover_feature_combinations`, `encoding_intrinsics_are_owned_by_compiled_stdlib_declarations`, and `encoding_private_declarations_codegen_through_sifr_stdlib`.
- `stdlib_native_surface_ownership.toml` marks `_sifr.encoding` with `deletion_stage = "migrated-to-sysroot-stdlib"` and the architecture doc reflects the same handoff.

### Non-blocking follow-ups

1. **Dead intrinsic-module signatures** — `text_encoding.rs:66-89` and `:150-160` still expose `encoding_decode_outcome`, `encoding_decode_incremental_outcome`, and `encoding_encode_outcome` in `intrinsic_encoding()`. Nothing imports them and no lowerer remains, so they're unreachable but should be pruned to keep the intrinsic registry a faithful mirror of live declarations.
2. **Redundant `StdlibFeature::EncodingRs` marker** — after this wave, `ENCODING_RS_DEPS = &[]` and the stdlib-feature mapping is `&[]`, so `additional_required_features(...)` returning `[StdlibFeature::EncodingRs]` for `str_encode_utf8_result` and friends (`registry/requirements.rs:31-36`) is now a no-op. Remove the marker (or drop the variant once no other surface needs it) so the required-feature list matches actual emissions.
3. **Outcome recomputation cost** — `encoding_decode_outcome`, `encoding_encode_outcome`, and `Decoder.decode` each call the private text/recoveries impls separately, and the incremental path also calls `_pending` — running `decode_with_recoveries`/`encode_with_recoveries` 2× or 3× per outcome vs the prior single-call intrinsic. This is a consequence of the no-converter-pipelines policy and is acceptable, but future work could add a fused `sifr_stdlib` helper (e.g., returning recoveries as a `\n`-joined string) to halve the cost without violating the bridge contract.
