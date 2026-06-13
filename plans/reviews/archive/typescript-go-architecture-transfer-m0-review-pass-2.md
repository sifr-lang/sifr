Based on my comprehensive review, here are my findings:

# M0 Code Review

## Correctness (verified by tracing through code and running tests)

**sifr_source semantics** (`crates/sifr_source/src/lib.rs:1-294`)
- UTF-8/UTF-16/UTF-32 conversions, CRLF handling, EOF, multibyte scalar rejection, and surrogate-pair interior rejection all behave correctly.
- `LineMap::line_byte_range` correctly strips `\r\n` and `\n` from the byte range; `\r` alone is preserved as content (matches the existing diagnostic renderer convention).
- `position_at` at EOF (offset == text.len()) correctly returns the start of the next (possibly virtual) line.
- `encoded_character_byte_offset` is the right shape: counts UTF-16 code units / Unicode code points, then returns the byte offset for that position.
- All three `sifr_source` unit tests pass.

**Migrations**
- `sifr_syntax/src/lib.rs:1-87` correctly removes the duplicate `SourceText`/`TextPosition`/`TextRangeUtf` definitions and re-exports from `sifr_source`. The local `text_position`/`byte_offset` callsites in tests now route through the shared implementation.
- `sifr_diagnostics/src/source_map/mod.rs:53-65` replaces the `text: String, line_starts: Vec<u32>` pair with `text: SourceText`. Stable hash is now provided by `SourceText::source_hash()`.
- `sifr_diagnostics/src/render/mod.rs:242-336` uses `LineMap` for both column computation and snippet rendering. Behavior preserved; the `unwrap_or_else` fallback in `render_line` is safer than the old direct indexing.
- `sifr_frontend/src/source_maps.rs:82-112` replaces the `SourceMapView` stubs with real implementations backed by `sifr_source::SourceText`. Signatures take `&TextPosition` (consistent with `sifr_source::byte_offset_with_encoding`).
- `sifr_frontend/src/graph_cache_and_queries.rs:1-16, 491-508` removes the duplicate `SourceText`/`SourceFileView`/`SourceMapView`/`PositionEncoding` definitions and re-imports them from `source_maps` (which re-exports from `sifr_source`).
- `sifr_lsp/src/conversion.rs:12, 45-100, 477-509` swaps `SyntaxSourceText` for `SourceTextMap` and adds private `lsp_range_with_encoding`/`text_range_with_encoding` helpers plus two new UTF-16 tests.

## Findings, ordered by severity

### Medium
1. **`sifr_source::byte_offset_with_encoding` is stricter than the legacy behavior for positions past the end of a line's content.** Previously, `byte_offset` accepted `line_start + character` up to and including the start of the next line (i.e., the line terminator). The new code uses `line_byte_range` (with terminator stripped) as the upper bound. For UTF-8 position `{line: 0, character: 4}` on `"abc\n"`, the old code returned `Some(3)` (the `\n` byte); the new code returns `None`. This is a correctness improvement, but downstream callers that relied on the lenient behavior would now see rejections. No such callers surfaced in the validation suite; just worth flagging in the PR description.
2. **Tracker is not yet marked complete.** `issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md:11` still reads `[ ] M0 source and position foundation completed`, and the `PR Log` (line 133) has no M0 link. These are administrative and should be filled in when the PR is opened.

### Low
3. **`sifr_source::SourceText::text_range` is now redundant with `range_at(..., UTF8)`.** Consider removing the wrapper in a follow-up; not a blocker.
4. **`sifr_lsp` advertises only `PositionEncodingKind::UTF8`** (`crates/sifr_lsp/src/capabilities.rs:28`), so the new private `*_with_encoding` helpers are only exercised by the new tests. This is fine for M0 (it builds the foundation for later UTF-16 wiring) but worth noting.
5. **Empty placeholder review files.** `reviews/typescript-go-m0-source-foundation-review-pass-1.md` and `reviews/typescript-go-architecture-transfer-m0-review-pass-2.md` are 0 bytes. Likely created by an earlier review tool run; harmless but untracked noise.
6. **`sifr_lint` reaches `sifr_frontend::SourceText`** rather than `sifr_source::SourceText` (`crates/sifr_lint/src/engine.rs:248`, `crates/sifr_lint/src/rules/large_parameter_list.rs:93`). Functionally fine (frontend re-exports it), but inconsistent with the "one source-position authority" intent. Not blocking for M0; trivial to tighten later.
7. **`sifr_format` builds a fresh `SourceText` per call** in `span_for_range` (`crates/sifr_format/src/lib.rs:386, 410`). Could share a `LineMap` if format is called many times for the same buffer. Not M0 scope.

## What was checked

- New `sifr_source` crate: API surface matches the locked contract (`SourceText`, `LineMap`, `PositionEncoding`, `TextPosition`, `TextRangeUtf`, `SourceFile`).
- Dependency direction guardrail passes (only `ruff_text_size` allowed; no references to any higher crate in source).
- No `SourceProvider`, `WorkspaceSession`, `WorkspaceSnapshot`, `DirtyScope`, scheduler, or cache-reuse work introduced.
- Validation log commands rerun: `cargo test -p sifr_source`, `cargo test -p sifr_syntax`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr_frontend`, `cargo test -p sifr_lsp`, `cargo test -p sifr_analysis`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo fmt --check`, `git diff --check`, `python3 scripts/check_file_size_guardrails.py`, `python3 scripts/check_source_crate_dependency_direction.py`, and `cargo clippy --workspace -- -D warnings` all pass.
- `internal_docs/architecture.md` and `internal_docs/frontend_query_architecture.md` updated to mention the new crate and the merged source-position authority.

M0 approved for PR
