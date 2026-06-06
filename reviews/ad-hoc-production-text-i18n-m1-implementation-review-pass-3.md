# M1 Implementation Review Pass 3 — Encoding And Explicit Text I/O

Phase: [ad-hoc-production-text-i18n-platform-substrate.md](../issues/ad-hoc-production-text-i18n-platform-substrate.md), `milestone_text_i18n_1: Encoding And Explicit Text I/O`
Prior reviews:
- [reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-1.md](./ad-hoc-production-text-i18n-m1-implementation-review-pass-1.md) — Result: `FAIL` with blockers B1–B7.
- [reviews/ad-hoc-production-text-i18n-m1-implementation-review-pass-2.md](./ad-hoc-production-text-i18n-m1-implementation-review-pass-2.md) — Result: `PASS` with closure preconditions C1 (missing `open(...)` dynamic-`errors` fixture), C2 (full `scripts/run_all_tests.sh` not yet run), and a single non-blocking remediation candidate N2 (write-mode encode-handler classification).

## 1. Verdict: **PASS**

Both pass-2 closure preconditions are satisfied and the touched code paths have been further hardened. The post-pass-2 deltas (open-site dynamic-`errors` fixture, write-mode encode-handler classification, DCE/rooting for compiler-special text open, and stdlib module filtering when a module is both directly imported and a transitive dep) are correctly scoped, covered by tests, and traceable to the ledger. M1 is mergeable.

## 2. Blocking findings

None.

### Verification of post-pass-2 deltas

**D1 — Open-site dynamic-`errors` fixture (closes pass-2 C1).**
`crates/sifr/tests/e2e/fail/text_i18n_open_dynamic_errors_handler.sifr:1-10` exercises `open(..., "r", encoding="utf-8", errors=errors)` with a dynamic identifier and expects `SIFR-ENCODING-0803` at `col=91`, the start of the value expression. The diagnostic is emitted at `crates/sifr_lowering/src/lower/expressions/call_shadowable_builtins.rs:163-166` against `keyword.value.range()`, which matches the annotated column. Traceability is updated at `verification/stdlib/text_i18n_m1_traceability.md:12` (now lists three `SIFR-ENCODING-0803` fixtures: the bytes-method, the encode-only-on-decode, and the new open-site case).

**D2 — Write-mode encode-handler classification (closes pass-2 N2).**
`call_shadowable_builtins.rs:78-80` introduces `is_open_write_mode`, and `:167-171` selects `is_encode_handler_label` vs `is_decode_handler_label` based on the literal mode. `xmlcharrefreplace` / `namereplace` / `backslashreplace` now compile for `w`/`wt`/`a`/`at`, while decode handler labels still apply to `r`/`rt`. Positive pass-fixture coverage: `crates/sifr/tests/e2e/pass/text_i18n_encoding_io.sifr:203-209` opens an ASCII text writer with `errors="xmlcharrefreplace"`, writes `"snowman ☃"`, then re-reads in binary and asserts `b"snowman &#9731;"`. Negative coverage remains via `text_i18n_decode_encode_only_handler.sifr` (encode-only handler on a decode site) and the three `SIFR-ENCODING-0803` fixtures above.

**D3 — DCE/rooting for compiler-special text open.**
`crates/sifr_codegen/src/intrinsic_method_emitters/builtin_core_methods.rs:33-43` registers `sifr.io` as a used stdlib module and roots `BinaryFileHandle` and `TextFileHandle` whenever `builtin_open_text` is emitted, mirroring the existing `builtin_open` rooting at `:30-32`. The handle types are now safe from stdlib DCE in programs whose only `sifr.io` reference is the compiler-special `open(..., encoding=..., errors=...)` form. Unit-tested at `builtin_open_text_roots_text_handle_support` (`:382-401`) and recorded in the ledger at `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:282`.

**D4 — Stdlib module filtering for direct-imported transitive deps.**
`crates/sifr_codegen/src/lib_modules_and_codegen.rs:316-348` collects `transitive_dependency_modules` while expanding `transitive_deps`, then chooses between the imports-filtered slice and the full module body using `if transitive_dependency_modules.contains(module_name) { rust_code.clone() } else { ... filter_stdlib_ir_to_needed(...) }`. This correctly preserves dependency-required classes/functions when a module is both directly imported (so it has an `imported_stdlib_names` entry) and a transitive dependency of another used stdlib module. Behavioral validation comes from the consolidated pass fixtures listed in the ledger at `:283-287` (`open_read`, `open_write`, `stdlib_io_consolidated`, `stdlib_logging_consolidated`) plus the full `scripts/run_e2e_pass.sh --profile merge` 73/73.

**D5 — `open_readline.sifr` optional list-index narrowing.**
`crates/sifr/tests/e2e/pass/open_readline.sifr:8-15` now binds `line1`/`line2` as `str | None` from a `list[str]` subscription and gates use behind `if line1 is not None:` / `if line2 is not None:` before string concatenation, consistent with M1's tightened typing of list subscription.

**D6 — Ledger evidence (closes pass-2 C2).**
`issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:266-289` records the post-remediation focused suite plus both lane reports. The merge-gate row at `:289` is the contract-required line: `scripts/run_all_tests.sh passed; report target/validation_lane_reports/merge.latest.json, wall time 589.16s, 73/73 e2e pass fixtures, hardening variants 34/34 with 0 failures, non-blocking warm-cache/group-skew advisories.` I independently re-read `target/validation_lane_reports/merge.latest.json` and confirmed: `profile: merge`, `time.real_seconds: 589.16`, every `lane_steps[].status == "pass"` (core guardrails, crate tests, validation contract matrix, platform golden, e2e pass suite, verification hardening suites, extra e2e checks), `hardening_summary.{failures, blocking_failures, non_blocking_failures}: 0`, `hardening_summary.variants: 34`. `target/validation_lane_reports/create-pr.latest.json` likewise shows `profile: create-pr`, `time.real_seconds: 183.44`, and every lane step passing.

## 3. Non-blocking observations

The residual non-blocking items from pass-2 §3 are unchanged and remain non-blockers for M1 closure:

- **N1.** `lib/sifr/io.sifr:308-312, 330-360`: `TextFileHandle.readline`/`readlines`, `TextReader`, and `TextWriter` still raise `IOError` rather than a typed `UnsupportedSurfaceError`. Traceability at `text_i18n_m1_traceability.md:11` now reads "`TextReader`/`TextWriter` names exist as unsupported direct-construction wrappers; line-buffer cursor semantics are deferred", which matches the implementation, so the gap is documented rather than masked.
- **N3.** Method surface (`str.encode`/`bytes.decode`) still maps runtime errors to `ParseError` while function surface (`sifr.encoding.encode`/`decode`) raises `EncodeError`/`DecodeError`. Same substrate, different typed errors at the public seam.
- **N4.** `Decoder.decode` (`lib/sifr/encoding.sifr:249-261`) still calls `encoding_decode_incremental_outcome` followed by `encoding_decode_incremental_pending`. The second call only computes the trailing-byte length and does not re-decode, so the pass‑1 footgun is gone; a future consolidated `(text, recoveries, pending)` intrinsic would remove the second FFI hop.
- **N5.** `text_i18n_codecs_register_unsupported.sifr` continues to satisfy the M0 codec-registry-mutation row via the generic `SIFR-IMPORT-0008` namespace-contract diagnostic rather than a dedicated registry-mutation code.
- **N6.** `crates/sifr_runtime/src/encoding.rs` `recover_utf8` still has the multi-byte `valid_up_to` re-scan loop (worst case O(n²) on adversarial inputs); worth a code comment if intentional but not a correctness gap.
- **N7.** `decode_outcome`/`encode_outcome` Sifr wrappers (`lib/sifr/encoding.sifr:205-218, 221-234`) still `try / except ... raise DecodeError(e.message)` with no shape change; cosmetic.

## 4. Validation gaps / closure preconditions

None. Both pass-2 closure preconditions C1 and C2 are now satisfied. The ledger entries at `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:266-289` reflect every required line, the merge-gate wall time and counts match the report files, and the M1 checkbox at `:25` can be flipped to `[x]` as part of the merge commit.

## 5. Re-review required

No. M1 PR creation and merge are unblocked. The non-blocking items in §3 are appropriate to record as follow-ups (e.g. N1 typed `UnsupportedSurfaceError`, N3 unified typed-error surface, N4 single-call incremental intrinsic) rather than gating M1 closure.
