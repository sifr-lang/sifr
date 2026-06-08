PASS

Rationale:

- **Substrate routing is real**: `decode_output_field` in `crates/sifr_codegen/src/intrinsics/registry/process.rs:362` lowers stdout/stderr through `sifr_runtime::encoding::decode_text(&bytes, &__encoding, &"strict")`, the same helper used by `decode_utf8_with_encoding`. The old UTF-8/utf8 allowlist guard is removed cleanly, so non-UTF-8 encodings (e.g. Latin-1, `iso8859-1`) genuinely flow through the text/i18n substrate instead of being rejected.
- **Strict error mode is enforced**: the third arg is hardcoded `"strict"`, matching the milestone scope; invalid bytes return a typed `ProcessError` via `process_map_err`, not a panic or locale fallback. The follow-up doc note explicitly defers optional non-strict handlers.
- **Feature wiring**: both `process_output_text` and `process_shell_output_text` are added to the `StdlibFeature::EncodingRs` required-features list in `registry.rs:66-71`, so generated crates pull in the substrate dependency.
- **Coverage**: a new registry unit test asserts `decode_text` appears in the rendered expression and that `EncodingRs` is required. Two new fixtures (`crates/sifr/tests/e2e/pass/process_text_explicit_encoding.sifr`, `verification/platform/golden/subprocess_text_explicit_encoding.sifr`) are added to both create-pr and merge manifests, exercising sync argv and shell text output with explicit encoding aliases.
- **Docs/manifests consistent**: traceability, host matrix, manifest `blocked_until`, and issue ledger all updated coherently. The umbrella subprocess row correctly downgrades from "blocked-on-m4" to "in-progress" with the strict-text row carrying the supported claim, and Windows is left host-limited pending a deterministic fixture - accurate scoping.
- **Validation evidence**: `scripts/run_all_tests.sh --profile create-pr` PASS, platform golden 6 pass / 1 skip, guardrails (file size + HIR maintainability) green, fmt/check/clippy implied via the script. No backwards-compat hacks or dead code introduced; net is a 18-line reduction.

No blocking correctness, coverage, or routing gaps. Acceptable to merge.
