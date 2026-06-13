# M1 Implementation Review Pass 2 — Encoding And Explicit Text I/O

Phase: [ad-hoc-production-text-i18n-platform-substrate.md](../issues/ad-hoc-production-text-i18n-platform-substrate.md), `milestone_text_i18n_1: Encoding And Explicit Text I/O`
Prior review: [reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-1.md](./ad-hoc-production-text-i18n-m1-implementation-review-pass-1.md) — Result: `FAIL` with blockers B1–B7

## 1. Verdict: **PASS**

All pass-1 blockers B1–B7 are remediated. The substrate now has true incremental decode with pending-byte carry-over, the three M0-reserved compile-time diagnostics (`SIFR-IO-0801`, `SIFR-IO-0802`, `SIFR-ENCODING-0803`) fire from the lowering, `str.encode(encoding, errors)` / `bytes.decode(encoding, errors)` propagate the typed handler through the same substrate as `sifr.encoding`, traceability no longer downgrades the M0 acceptance criteria, and `decode_with_recoveries` / `encode_with_recoveries` return both produced output and recoveries in a single runtime call.

Non-blocking observations remain (TextReader/TextWriter placeholders, asymmetric error typing on method paths, missing `open()`-site dynamic-`errors` fixture, missing merge-gate run before milestone closure). Another review pass is **not** required to clear the implementation; the closure preconditions in §4 must still be satisfied before the milestone is marked complete.

## 2. Blocking findings

None.

### B1 — true incremental Decoder/Encoder

Fixed. `crates/sifr_runtime/src/encoding.rs:56-91` introduces `incremental_decode_with_recoveries` (combines pending + data, computes a `pending_tail_len`, decodes only the ready prefix) and `incremental_decode_pending` (returns the trailing bytes to carry across calls). `pending_tail_len` and `utf8_pending_tail_len` (`encoding.rs:93-123`) implement UTF‑8 leading-byte/continuation tracking and UTF‑16 odd-byte tracking. The decoder Sifr surface (`lib/sifr/encoding.sifr:237-261`) maintains `_pending: bytes` across calls and clears it on final.

Fixture `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr:124-141` splits `b"\xc3"` then `b"\xa9"` across two non-final/dynamic-final calls and verifies the first returns `""`, the second returns `"é"`, and a third call raises an exhausted error. The dynamic-`final` case is exercised by `dynamic_final = len(actual) > 0` and reused symmetrically in the Encoder block (`text_i18n_encoding_io.sifr:157-170`).

### B2 — `SIFR-IO-0801` text-mode open without encoding

Fixed. `crates/sifr_lowering/src/lower/expressions/call_shadowable_builtins.rs:224-227` reports `IO_TEXT_OPEN_REQUIRES_ENCODING` when the resolved mode literal is non-binary and no `encoding=` keyword is present. The code is registered in `crates/sifr_diagnostics/src/codes/registry.rs:84` and `crates/sifr_diagnostics/src/codes/registry/registry_entries/project_and_backend.rs:7-17`. Fixture: `crates/sifr/tests/e2e/fail/text_i18n_open_without_encoding.sifr`. Demo and pass-suite call sites that previously used implicit-UTF‑8 text `open(...)` have been updated (e.g. `demos/io/main.sifr`, `demos/file_streams/main.sifr`, `crates/sifr/tests/e2e/pass/open_read.sifr`, `lib/sifr/logging.sifr` switching to `open_text(..., encoding=utf8())`).

### B3 — `SIFR-IO-0802` dynamic open mode

Fixed. `call_shadowable_builtins.rs:87-95` requires the mode expression to be a `Expr::StringLiteral` and reports `IO_OPEN_MODE_REQUIRES_LITERAL` otherwise. Registered in `registry.rs:85` and `project_and_backend.rs:18-28`. Fixture: `crates/sifr/tests/e2e/fail/text_i18n_open_dynamic_mode.sifr`.

### B4 — `SIFR-ENCODING-0803` dynamic / unsupported error handler

Fixed. The compile-time gate has three call sites:

- `crates/sifr_lowering/src/lower/bytes_methods.rs:47-105` validates the second argument of `str.encode` / `bytes.decode` is a string literal in the typed handler set, with `validate_decode_error_handler` excluding encode-only handlers (`xmlcharrefreplace`, `namereplace`).
- `call_shadowable_builtins.rs:143-151` validates the `errors=` keyword on `open(..., encoding=..., errors=...)` with the same decode-handler set.
- Registered in `registry.rs:87-88` and `project_and_backend.rs:29-39`.

Fixtures: `crates/sifr/tests/e2e/fail/text_i18n_dynamic_errors_handler.sifr` (dynamic `errors` variable on `bytes.decode`) and `crates/sifr/tests/e2e/fail/text_i18n_decode_encode_only_handler.sifr` (encode-only `xmlcharrefreplace` on a decode site).

### B5 — `str.encode(encoding, errors)` / `bytes.decode(encoding, errors)` propagation

Fixed. `bytes_methods.rs:113-143, 253-288` accepts 0–2 arguments and routes the validated literal handler through to the intrinsic. The method-to-intrinsic dispatch in `crates/sifr_lowering/src/lower/expressions/methods_lambdas_and_comprehensions.rs:194-221` selects `str_encode_utf8_result_with_encoding` / `decode_utf8_with_encoding` when arguments are present, and both branches now route through `encoding::lower_str_encode_result` / `encoding::lower_bytes_decode_result` (`crates/sifr_codegen/src/intrinsics/registry.rs:308-323`). Those lowerers accept 1–3 args and forward `errors` to `sifr_runtime::encoding::encode_bytes` / `decode_text` (`crates/sifr_codegen/src/intrinsics/registry/encoding.rs:287-317`).

The fixture exercises both supported handlers in the method-call surface (`text_i18n_encoding_io.sifr:100-108`: `"...".encode("ascii", "xmlcharrefreplace")` and `b"\xffA".decode("ascii", "replace")`). Negative coverage lives in `text_i18n_dynamic_errors_handler.sifr` and `text_i18n_decode_encode_only_handler.sifr`.

### B6 — traceability restored to M1 acceptance criteria

Fixed. `verification/stdlib/text_i18n_m1_traceability.md:12` now lists the three required compile-time diagnostics by code (`SIFR-IO-0801` / `SIFR-IO-0802` / `SIFR-ENCODING-0803`) and the unsupported-surface fixtures, with no "deferred to a separate breaking migration" language. The "No implicit text encoding" row now matches the M0 inventory at `verification/stdlib/text_i18n_substrate_inventory.md:84-86`.

### B7 — single-call outcome intrinsics

Fixed for the primary `encode_outcome` / `decode_outcome` surface. `sifr_runtime::encoding::decode_with_recoveries` and `encode_with_recoveries` (`crates/sifr_runtime/src/encoding.rs:125-202`) return `(String, Vec<String>)` and `(Vec<u8>, Vec<String>)` from a single decode/encode pass; `lower_encoding_decode_outcome` / `lower_encoding_encode_outcome` (`registry/encoding.rs:49-119, 182-197, 253-268`) build the `DecodeOutcome` / `EncodeOutcome` struct from that tuple without a second runtime call.

The `Decoder` path still uses two runtime calls (`encoding_decode_incremental_outcome` for the produced text + recoveries, then `encoding_decode_incremental_pending` for the tail bytes to carry across calls — `lib/sifr/encoding.sifr:249-261`). However the second call (`incremental_decode_pending`, `encoding.rs:76-91`) only does byte-boundary computation — it does **not** re-decode — so the original "double-decode" footgun is gone. See non-blocking N4 below for the residual API-shape observation.

## 3. Non-blocking findings

- **N1.** `lib/sifr/io.sifr:308-312, 330-360`: `TextFileHandle.readline` / `readlines` and `TextReader` / `TextWriter` still raise at use. Pass-1 noted this; M1 closes with line-iterator surfaces explicitly deferred per the traceability row. The deferral is recorded but the surfaces remain typed as `Result[..., IOError]`, which produces a runtime `IOError` rather than a typed `unsupported`. Either tighten the traceability to read "unsupported direct construction" or follow up with a typed `UnsupportedSurfaceError`.

- **N2.** `call_shadowable_builtins.rs:148` reuses the **decode** handler-label set for `open(..., errors=...)` regardless of write mode. `open(path, "w", encoding="ascii", errors="xmlcharrefreplace")` is therefore rejected at compile time, even though `xmlcharrefreplace` is a valid encode-side handler and the resulting `TextFileHandle` writes via the encode path. Either accept the encode-only handlers when the mode literal is `w`/`wt`/`a`/`at`, or document this restriction in the inventory.

- **N3.** Public method surface vs. function surface error types diverge: `str.encode(...)` / `bytes.decode(...)` map runtime errors to `ParseError` (`registry/encoding.rs:287-317`) while `sifr.encoding.encode(...)` / `decode(...)` raise `EncodeError` / `DecodeError`. Same substrate, different typed errors at the public seam. Pick one; today's split is documentation noise consumers will hit on their first try/except.

- **N4.** Single-call intrinsic surface for incremental decode would still be a small win: `Decoder.decode` calls two runtime functions where one could return `(text, recoveries, pending)`. No double-decoding today, but two intrinsics is two type-checked seams and two trip rounds across the FFI for every chunk.

- **N5.** `text_i18n_codecs_register_unsupported.sifr` records `SIFR-IMPORT-0008` on `from codecs import register`. This satisfies the M0 inventory row 36 fixture requirement only via the namespace-contract diagnostic, not a dedicated codec-registry-mutation diagnostic. Acceptable but the weakest possible form, per pass‑1's same observation about `text_i18n_textiowrapper_unsupported.sifr`.

- **N6.** `crates/sifr_runtime/src/encoding.rs:212-240` (`recover_utf8`) still has the multi-byte `valid_up_to` re-scan loop carried over from pass‑1. O(n²) on adversarial inputs. Worth a comment if intentional.

- **N7.** `lib/sifr/encoding.sifr:205-218, 221-234`: `decode_outcome` / `encode_outcome` still `try / except ... raise DecodeError(e.message)` for no benefit. Either drop the wrapper or add a comment explaining the re-typing.

## 4. Validation gaps / closure preconditions

The fixture coverage now satisfies the pass‑1 list with the following caveats:

1. **Missing fixture: `open(...)` site dynamic-`errors`.** The lowering at `call_shadowable_builtins.rs:143-151` validates the `errors=` keyword on `open(...)`, but no fail-fixture exercises it. `text_i18n_dynamic_errors_handler.sifr` covers `bytes.decode`; `text_i18n_decode_encode_only_handler.sifr` covers the encode-only-on-decode case. Add a fixture like `open(path, "r", encoding="utf-8", errors=dynamic_handler)` so the open-site branch has positive evidence.

2. **Merge-gate command before milestone closure.** Pass-1 §4 raised that the full `scripts/run_all_tests.sh` (no `--profile`) is required by the contract at `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:213-217` before milestone closure. The M1 post-remediation evidence at `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:266-280` only records `scripts/run_all_tests.sh --profile create-pr` (222.17s). Run the full suite and append its report/wall-time before flipping the M1 checklist box.

3. **`TextFileHandle` read path with a recovery handler.** No pass-suite case exercises `open(path, "r", encoding="ascii", errors="replace")` against intentionally invalid bytes, even though the substrate supports it. Optional — the same code path is exercised via `decode_outcome(b"\xffA", ascii(), replace_decode_handler())` in `text_i18n_encoding_io.sifr:47-53`, so coverage exists at the substrate level.

Local validation reported by the implementer (`cargo test -p sifr_diagnostics codes`, `cargo check`, the M1 e2e pass fixture, `cargo test -p sifr --test e2e test_e2e_fail`, `scripts/run_e2e_pass.sh --profile create-pr`, `cargo fmt --check`, file/HIR guardrails, `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_stdlib`, `cargo test -p sifr -- stdlib`, `scripts/run_all_tests.sh --profile create-pr` at 222.17s) all passed. Item (2) above is the only gap.

## 5. Re-review required

No. All pass‑1 blockers are remediated; the remaining items in §3 are non-blocking and can be addressed in this PR or follow-ups, and §4 item (2) is a closure precondition that does not require a further review pass once the full `scripts/run_all_tests.sh` evidence is appended.
