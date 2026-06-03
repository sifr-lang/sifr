`parser_category` args and `while parsing` child notes are both preserved. Everything is in order.

---

## M2 Review: Parser Diagnostic Source Spans

**Verdict: SATISFIED.**

### What I checked

**1. Implementation correctness (`crates/sifr_syntax/src/lib.rs`, 692 lines)**

- `parse_error_diagnostic` threads `error.location` (a `TextRange`) through `parse_diagnostic`, which builds a local `SourceMap`, registers the source with the display path, creates a validated `SourceSpan`, and renders through `DiagnosticBuilder::source` → `DiagnosticSink` → `render_sink`. This is the canonical source-map path, not a hand-built partial span.
- `unsupported_syntax_diagnostic` threads `error.range()` through the same `parse_diagnostic` path.
- `parse_diagnostic` handles invalid/out-of-bounds ranges by returning an internal compiler panic diagnostic, not a misleading user-facing span.
- All helper detail functions (`lexical_or_string_details`, `expected_details`, etc.) correctly carry `parser_category`.
- The `while parsing <context>` child note is preserved via `builder.child(...)`.

**2. Parser fixtures (7/7 pass)**

| Fixture | Code | Span file | Byte range | Line/col | Snippet |
|---|---|---|---|---|---|
| `parser_bad_indent` | SIFR-PARSE-0002 | main | 12–17 | 2:1 | `print("bad indent")` |
| `parser_unterminated_string` | SIFR-PARSE-0003 | main | 22–36 | 2:11 | `print("unterminated)` |
| `parser_invalid_call_order` | SIFR-PARSE-0006 | main | 77–78 | 5:18 | `f(a=1, 2)` |
| `parser_empty_declaration` | SIFR-PARSE-0007 | main | 22–23 | 2:11 | `global` |
| `parser_invalid_declaration` | SIFR-PARSE-0002 | main | ✓ | ✓ | ✓ |
| `parser_invalid_match_pattern` | SIFR-PARSE-0008 | main | 38–44 | 3:14 | `case *value:` |
| `parser_unsupported_syntax` | SIFR-PARSE-0009 | main | 0–4 | 1:1 | `lazy` |

All 7 produce correct codes, primary spans with complete fields, no `<unknown>`, and clean human/compact output.

**3. Edge-case unit tests (8/8 pass)**

- **Zero-length EOF range** — `test_parser_diagnostic_renders_zero_length_eof_span`: zero-width caret at line 1, column 1, empty snippet with `highlight_start == highlight_end`.
- **Invalid out-of-bounds range** — `test_parser_diagnostic_invalid_span_becomes_internal_error`: out-of-bounds range produces `SIFR-INTERNAL-0001` with no user-facing span.
- **CRLF source text** — `test_parser_diagnostic_preserves_crlf_source_text_in_json_span`: `\r\n` preserved in snippet text, line/column still correct.
- **Non-ASCII before error** — `test_parser_diagnostic_columns_are_utf8_character_based`: emoji (`🦀`) before error, byte offset ≠ character offset, column still 1-based.
- **Unsupported syntax with range** — `test_unsupported_syntax_diagnostic_uses_ruff_range`: `lazy` at byte 0–4, line 1, column 1.
- Additionally: basic suite/token exposure, UTF-8 position round-trip, span attachment on real bad-indent error.

**4. Contract checker**

- Self-test passes.
- Parser runtime contract correctly exercises all 7 fixtures through JSON, human, and compact modes.
- M3 boundary correctly fails with `workspace_missing_import_canonical: expected SIFR-IMPORT-0002, got ['SIFR-WORKSPACE-0101']` — exactly as expected.

**5. Architecture compliance**

- `lib.rs` is 692 lines, under the 900-line cap.
- No monolithic files; the file is well-structured with clear helper functions per error category.
- `cargo fmt --check` passes.
- Parser category args (`parser_category`) and context child notes (`while parsing <module>`) are preserved in JSON output.

### No blockers identified

M2 is complete. The only failure is at the M3 boundary (workspace missing import still using `SIFR-WORKSPACE-0101` instead of `SIFR-IMPORT-0002`), which is expected since M3 (canonical import resolution diagnostics) is not yet implemented.

### Ready for M3

Parser diagnostics are fully canonical: every category has primary spans through the source-map path, all edge cases are tested, and `parser_category`/`while parsing` context survives to JSON. Proceed to M3 (canonical import resolution diagnostics) when ready.
