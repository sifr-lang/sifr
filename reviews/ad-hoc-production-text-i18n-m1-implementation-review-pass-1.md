Now I have enough context to produce the review.

# M1 Implementation Review — Encoding And Explicit Text I/O

## 1. Verdict: **FAIL**

The change adds a real shared encoding substrate (`sifr_runtime::encoding`) and routes `sifr.encoding`, `str.encode`, `bytes.decode`, and `open(..., encoding=..., errors=...)` through it. But several M1 DOD bullets and M0-reserved diagnostics are not actually shipped, the `Decoder`/`Encoder` claim of incrementality is mechanically untrue, and the traceability silently downgrades the M0-reserved `open(path)` diagnostic to "deferred to a separate breaking migration." Another remediation pass is required.

## 2. Blocking findings (highest severity first)

### B1. `Decoder`/`Encoder` are not incremental — DOD violation
`lib/sifr/encoding.sifr:247-278` — each call to `Decoder.decode(data, final=...)` calls `decode_outcome(data, ...)`, which decodes `data` as a one‑shot complete input. There is no buffered byte tail, no partial-codepoint carry‑over, and `final=False` is functionally identical to `final=True` except for the `_exhausted` flag. A split UTF‑8 sequence such as `decoder.decode(b"\xc3", final=False)` then `decoder.decode(b"\xa9", final=False)` cannot succeed; under strict mode the first call raises and under non‑strict it emits two recovery diagnostics instead of one valid `é`.

The phase contract (`Incremental Codec Ownership`) requires "stateful linear values" and the M1 DOD requires "Incremental encoder/decoder finalization and post-finalization exhaustion have fixtures for both statically known and dynamic `final` values." Neither true incrementality nor dynamic-`final` fixtures exist (`crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr:59-86` only uses literal `final=True`). The current pass fixture would still pass if `final` were ignored entirely.

### B2. M0-reserved diagnostic `SIFR-IO-0801` is missing
`crates/sifr_lowering/src/lower/expressions/call_shadowable_builtins.rs:60-129` — when `encoding=` is absent, the lowering falls straight through to legacy `builtin_open` / `FileHandle`. `FileHandle.read()` is then lowered by `crates/sifr_codegen/src/intrinsics/registry/file_handles.rs:493-505` to `std::io::Read::read_to_string`, i.e. a silent UTF‑8 default.

The M0 inventory reserves this exact code with message `text-mode open requires an explicit encoding; Sifr does not use locale-derived default encodings` (`verification/stdlib/text_i18n_substrate_inventory.md:84`); the phase contract states "Statically visible text-mode opens without `encoding=` produce a compile-time diagnostic requiring `encoding=...`" and "Sifr must not silently substitute UTF-8 for CPython's locale-derived default." This is unambiguously M1 scope and is unshipped. Re-classifying it as "pending a separate breaking diagnostic migration" in `verification/stdlib/text_i18n_m1_traceability.md:12` is an unauthorized scope downgrade — M1 cannot close while UTF‑8 is the de‑facto implicit default.

### B3. M0-reserved diagnostic `SIFR-IO-0802` is missing
Same file, line 48-59. `mode_arg` is taken from any `lower_expr` result with no literal‑string check, so `open(path, mode_var, encoding="latin-1")` compiles, and the lowered match in `open_text_handles.rs:213-237` falls into the `_` arm at runtime, returning `IOError("invalid mode: ...")` instead of a compile-time `SIFR-IO-0802` (`open mode must be a string literal so Sifr can choose a binary or text handle type`).

### B4. M0-reserved diagnostic `SIFR-ENCODING-0803` is missing
`call_shadowable_builtins.rs:73-89` and the `bytes.decode` / `str.encode` method paths accept any `Type::Str` for `errors`/`encoding` without validating that a literal value is a member of the typed handler set. Dynamic handler strings such as `errors=some_runtime_string` compile and fail only at runtime inside `encoding::decode_handler`/`encode_handler`. M0 (`verification/stdlib/text_i18n_substrate_inventory.md:86`) reserved `SIFR-ENCODING-0803` for exactly this case. The phase contract is explicit: "Dynamic `errors=` strings and dynamic handler registration are unsupported in this phase," with compile-time diagnostics for invalid static literals.

### B5. `str.encode(encoding, errors)` / `bytes.decode(encoding, errors)` reject the second argument
`crates/sifr_lowering/src/lower/bytes_methods.rs:45-69` and `181-205` — the arity check is `args.len() > 1`. Any call like `b"...".decode("latin-1", "replace")` or `"...".encode("ascii", "ignore")` is rejected with `bytes.decode() takes 0 or 1 argument, got 2`. The M1 DOD explicitly requires: "`str.encode(encoding, errors)` and `bytes.decode(encoding, errors)` have fixtures for supported typed handlers, unsupported dynamic handler names, and invalid context combinations." None of those fixtures can exist while the second arg is rejected at HIR. The intrinsic lowerers (`registry/encoding.rs:144-180`) already hardcode `"strict"` so even if the front-end were fixed, the lowering would still ignore the user's handler.

### B6. Traceability has been silently relaxed below M1 acceptance criteria
`verification/stdlib/text_i18n_m1_traceability.md:12` rewrites the "No implicit text encoding" row from the original "Negative fixtures for `open(path)` / text mode without `encoding=` and dynamic/nonliteral mode" to "existing implicit `open(path)` remains accepted for legacy Sifr `FileHandle` pending a separate breaking diagnostic migration." This contradicts the phase contract (`milestone_text_i18n_1` definition of done in `issues/ad-hoc-production-text-i18n-platform-substrate.md:441-442` plus the cross-phase dependency contract at line 59), the M0 reserved diagnostic table, and the "no backward-compatibility shims, fallback paths" policy in `Quality Contract`. M1 cannot close by rewriting its own acceptance criteria.

### B7. `Decoder.decode` runs the decode twice
`lib/sifr/encoding.sifr:201-216` and `Decoder.decode` calling `decode_outcome` which in turn calls both `encoding_decode_text` *and* `encoding_decode_recoveries` separately — each invocation re-runs the full decode in the runtime. For a strict failure on bytes already partially processed this is benign but doubles work; more importantly the contract requires "Recoverable non-strict handlers return typed success outcomes that preserve both produced output and recovery evidence" returned **together**. The current shape can drift if a future runtime change ever makes the two calls non-deterministic. Not a correctness bug today, but it indicates the runtime's single-call API (`decode_with_recoveries`) wasn't exposed to the substrate — the lib wraps two thin intrinsics instead of one. Fix the intrinsic surface to return `(text, recoveries)` in one call, or this is a latent footgun in every consumer.

## 3. Non-blocking findings

- `crates/sifr_runtime/src/encoding.rs:437-439`: dead `_cow_to_string` helper with `#[allow(dead_code)]` and `use std::borrow::Cow` — delete; it adds nothing and dodges the clippy lint.
- `lib/sifr/encoding.sifr:48-58`: `Encoding.__init__` accepts any string with no validation; consumers can hold `Encoding("bogus")` until first use. Consider validating at construction or routing construction through a static factory; runtime-only validation is acceptable per the contract but lazier than the substrate spec implies.
- `lib/sifr/io.sifr:330-360`: `TextReader` / `TextWriter` are placeholders whose only behaviour is to raise on use. They satisfy the import names in the M1 inventory but provide no value. Either implement them as the production text streaming type or remove and update the inventory; the current state is the "partial public modules are rejected unless explicitly unstable/internal" form the phase rejects.
- `lib/sifr/io.sifr:308-312`: `TextFileHandle.readline`/`readlines` raise `IOError("...deferred; use read().split(...)")`. This is a regression in surface vs. the public API contract (M1 should ship a usable line iterator) and gives consumers a runtime panic-equivalent rather than a typed unsupported result; mark explicitly deferred in the traceability table.
- `lower_str_encode_result` / `lower_bytes_decode_result` (`crates/sifr_codegen/src/intrinsics/registry/encoding.rs:144-180`) map runtime errors to `ParseError`, but the public `sifr.encoding.encode/decode` returns `EncodeError`/`DecodeError`. Same substrate, different typed errors at the public surface — document, or unify.
- `lib/sifr/encoding.sifr:201-216, 219-234`: `decode_outcome`/`encode_outcome` swallow and rethrow the same `DecodeError`/`EncodeError` with a `try/except ... raise X(e.message)`, losing nothing useful. Either drop the wrapper or annotate why; today it's noise.
- `crates/sifr/tests/e2e/fail/text_i18n_textiowrapper_unsupported.sifr` only exercises that the name `TextIOWrapper` is not exported from `sifr.io`. The phase requires this be recorded as `unsupported-with-diagnostic` — a name-resolution miss is the weakest possible form; a typed `STDLIB_UNSUPPORTED_SURFACE` diagnostic naming `io.TextIOWrapper` and pointing at `sifr.io.open_text` would match the M0 inventory disposition.
- `crates/sifr_runtime/src/encoding.rs:144-172`: `recover_utf8` keeps a multi-byte `valid_up_to` re-scan loop; for big inputs this is O(n²). Acceptable but worth a comment if intentional.

## 4. Validation / fixture gaps

The DOD lists evidence the implementation must produce; the e2e/pass and registry tests do not cover the following, and the ledger should not record M1 as complete without them.

1. BOM handling: no `utf-8-sig` decode-strips-BOM / encode-prepends-BOM fixture.
2. UTF-16 LE/BE round trips: none.
3. Tier 1 `windows-125x` fixtures (decode + encode): none.
4. Alias resolution: no fixture proving `cp1252`, `utf_8`, `iso8859-1` resolve.
5. Static-registry unsupported mutation: no fixture for bare `codecs.register` / `codecs.unregister` (M0 inventory rows 36-38 expect M1 fixtures).
6. Compile-time diagnostic for implicit `open(path)` (`SIFR-IO-0801`).
7. Compile-time diagnostic for dynamic `open(path, mode_var, ...)` (`SIFR-IO-0802`).
8. Compile-time diagnostic for dynamic `errors=` string (`SIFR-ENCODING-0803`).
9. `str.encode(encoding, errors)` / `bytes.decode(encoding, errors)` typed-handler fixtures.
10. Encode-only handler on decode call site (e.g. `decode(..., xmlcharref_replace_decode_handler())`) and decode-only handler on encode call site — the runtime rejects this in `encoding.rs:415-419` but no test covers the boundary.
11. Truly incremental Decoder/Encoder fixture splitting a multi-byte sequence across two non-final calls.
12. Dynamic-`final` fixture (`final=condition`) per DOD.
13. Post-finalization exhaustion on `Encoder` is tested but only with `final=True` literal; same gap as #12.
14. `TextFileHandle` read path with `replace` decode recovery against intentionally invalid bytes — currently only round-trips Latin‑1 happy paths.

Also: the ledger entry at `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:246-264` claims `scripts/run_e2e_pass.sh --profile create-pr` passed and the broader suite ran, but cannot have exercised diagnostics that aren't implemented. The merge-gate `scripts/run_all_tests.sh` (full, not `--profile create-pr`) is missing from the M1 evidence list — required by the contract's "Required merge-gate command before milestone closure" at line 213-217.

## 5. Re-review required

Yes. After remediating B1–B7 and filling the fixture gaps in §4, a second review pass is required, with explicit attention to:

- truly incremental `Decoder`/`Encoder` behavior (split inputs, dynamic `final`),
- the three M0-reserved diagnostics actually firing at compile time,
- `str.encode`/`bytes.decode` accepting and propagating typed `errors=` through the same substrate,
- the traceability table restored to the original M0 acceptance criteria (no "deferred to a future breaking migration" escape hatch),
- and a single-call runtime surface that returns `(output, recoveries)` so the lib does not double-decode.
