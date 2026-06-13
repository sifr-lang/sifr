I have enough evidence to render a verdict. The implementation is solid, but I found one clippy blocker that the user did not include in their validation pass.

---

# M0 Implementation Review

## Verdict

**M0 is not yet approved for PR.** One blocker must be fixed first: a pedantic clippy warning that escalates to an error under `cargo clippy --workspace -- -D warnings`, which AGENTS.md lists as a required local validation step. The user did not run that command; running it fails on a single source line.

## Blocker

**`cargo clippy --workspace -- -D warnings` fails on `crates/sifr_frontend/src/source_maps.rs:87`.**

```
error: this argument is passed by value, but not consumed in the function body
  --> crates/sifr_frontend/src/source_maps.rs:87:19
   |
87 |         position: TextPosition,
   |                   ^^^^^^^^^^^^
```

`SourceMapView::text_position_to_span` takes `position: TextPosition` by value but only passes a reference to `sifr_source::SourceText::byte_offset_with_encoding`. This is the only warning in the workspace introduced by the M0 diff, and it conflicts with the AGENTS.md requirement:

> `cargo clippy --workspace -- -D warnings`

Two acceptable fixes:

- Change the signature to `position: &TextPosition` and update the bench (`crates/sifr_frontend/src/bin/frontend_query_bench.rs:269`) plus the two test call sites in `crates/sifr_frontend/src/source_maps.rs:140,181` to pass `&position`. Note this is a public API change on `SourceMapView`.
- Or add `#[allow(clippy::needless_pass_by_value)]` on the method. The bench and tests stay unchanged but the public API retains the by-value signature.

Either is fine. Pick the one the project prefers for SourceMapView ergonomics and apply it.

## Correctness — what I checked, what holds

I walked the new `sifr_source` API and the consumer migrations with a fine-tooth comb. No correctness defects found in any of the M0 closeout categories the user listed.

### `crates/sifr_source/src/lib.rs`

- **UTF-8 byte offsets** (line 149-181, 184-215): `is_char_boundary` is checked for the UTF-8 path; UTF-16/UTF-32 offsets come from `encoded_character_byte_offset` which iterates `text.char_indices()`, so the returned offset is always a char boundary of `line_text`. Char-boundary safety holds.
- **UTF-16 surrogate interiors** (line 265-285): `encoded_character_byte_offset` rejects a request that lands inside a surrogate pair via `if consumed.checked_add(char_width)? > character { return None; }`. Verified for `a🦀b` — `character: 2, Utf16` returns `None` as expected.
- **UTF-32 char counts** (line 265-285): `EncodedWidth::Utf32` uses `ch.len_utf16() == 1`, treating each scalar as one code point, so it counts characters correctly. `position_at` uses `prefix.chars().count()`. Both consistent.
- **CRLF line endings** (line 89-103): `line_byte_range` strips both `\r\n` and `\n` from the trailing end of the slice, so a CRLF line's content range does not include the `\r` or `\n`. `parser_diagnostic_preserves_crlf_source_text_in_json_span` (`crates/sifr_syntax/src/lib.rs:553`) verifies the renderer still shows the `\r` in the displayed snippet, which is the existing convention.
- **EOF** (line 105-113, 189-215): For `"alpha\nβeta\n"` (12 bytes, line_starts `[0, 6, 12]`), `position_at(12, Utf8)` returns `Some({line: 2, character: 0})` and round-trips back to offset 12. `line_index_at_offset` uses `Err(index) => index.checked_sub(1)?`, which correctly handles `offset == text_len` because the returned `Err(line_starts.len())` decrements to the last real line.
- **Spans in diagnostic rendering** (`crates/sifr_diagnostics/src/render/mod.rs:244-307`): The migration replaces the per-source `Vec<u32>` line_starts with `LineMap::line_starts()`. The `binary_search` argument flips from `u32` to `TextSize` (which is `Ord`); logic is unchanged. `render_line` uses `line_full_byte_range` (newline-inclusive) and `trim_end_matches('\n')` to preserve the same `lines[].text` shape as before, including the trailing `\r` for CRLF.
- **Multi-file `SourceMapView`** (`crates/sifr_frontend/src/source_maps.rs:82-111`): `source_for_file` linear-scans the `files` vec by `id`; `text_position_to_span` returns `None` for unregistered files (test at line 168-178 confirms).

### Dependency direction

- `sifr_source/Cargo.toml` lists only `ruff_text_size`. `crates/sifr_source/src/lib.rs` `use`s only `ruff_text_size`, `std::path::PathBuf`, `std::sync::Arc`. No upward reference to `sifr_diagnostics`, `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, `sifr_lsp`, `sifr_lint`, `sifr_format`, `sifr_package`, `sifr_hir`, `sifr_type_system`, `sifr_codegen`, `sifr_driver`, or `sifr`.
- `python3 scripts/check_source_crate_dependency_direction.py` -> PASS.
- The four consumers (`sifr_diagnostics`, `sifr_frontend`, `sifr_lsp`, `sifr_syntax`) all declare `sifr_source = { workspace = true }` in their `Cargo.toml`. `sifr_lint` uses `sifr_frontend::SourceText`, which is now a re-export of `sifr_source::SourceText` (`crates/sifr_frontend/src/source_maps.rs:2`), so the linter shares the same authority by re-export, not by introducing a separate type.

### One source-position authority

After this PR, every byte/position computation routes through `sifr_source`:

- `sifr_syntax::SourceText` / `TextPosition` / `TextRangeUtf` are re-exports of `sifr_source` (line 19).
- `sifr_diagnostics::SourceMap` stores `text: SourceText` and exposes `pub(crate) fn source` returning `&SourceText` (`crates/sifr_diagnostics/src/source_map/mod.rs:158`). The old `pub(crate) fn source_text` and `line_starts` accessors and the local `fn line_starts(text: &str)` and `fn stable_hash(text: &str)` are deleted — only one FNV-1a hash function remains in the workspace (in `sifr_source::stable_hash`).
- `sifr_frontend::SourceFileView` carries `source: SourceText` (`crates/sifr_frontend/src/source_maps.rs:67-74`); `SourceMapView::text_position_to_span` and `span_to_text_range` delegate to `SourceText::byte_offset_with_encoding` and `range_at`.
- `sifr_lsp::conversion` constructs `sifr_source::SourceText` and calls `byte_offset_with_encoding` / `range_at` (`crates/sifr_lsp/src/conversion.rs:62-74, 86-91`). The default `lsp_range` and `text_range` still hardcode `PositionEncoding::Utf8`; UTF-16 is exercised only in the new `*_with_encoding` helpers, which is the right M0 scope (actual capability-driven routing is M5 per the plan).

## Test coverage I verified by running

```
cargo test -p sifr_source        -> 3 passed
cargo test -p sifr_syntax        -> 8 passed
cargo test -p sifr_diagnostics   -> 32 passed
cargo test -p sifr_frontend      -> 5 passed (lib)
cargo test -p sifr_lsp           -> 2 passed
cargo test -p sifr_analysis      -> 10 passed
cargo test -p sifr -- --skip test_e2e_pass -> 33 passed
```

`cargo fmt --check` PASS, `git diff --check` PASS, `python3 scripts/check_file_size_guardrails.py` PASS, `python3 scripts/check_source_crate_dependency_direction.py` PASS.

## Scope fit

- M0 closeout items, in order: shared `SourceText`/`LineMap`/`PositionEncoding`/`TextPosition`/`TextRangeUtf` ✓; parser-side `sifr_syntax::SourceText` consumers migrated (`crates/sifr_syntax/src/lib.rs:194-203` builds a `SourceMap::register_source`, but the local `SourceText(String)` struct is gone, replaced by the re-export) ✓; `sifr_diagnostics::SourceMap` line-map storage/rendering migrated to `sifr_source::LineMap` ✓; `crates/sifr_frontend/src/graph_cache_and_queries.rs` source-map view construction/conversion migrated (new `crates/sifr_frontend/src/source_maps.rs` holds the migrated types and methods) ✓; `crates/sifr_frontend/src/bin/frontend_query_bench.rs` `interactive.source_map_lookup` rewritten to assert a real round trip and a UTF-8 round-trip equality check ✓; `crates/sifr_lsp/src/conversion.rs` helpers migrated ✓; `SourceMapView` stubs replaced ✓; multibyte/CRLF/EOF/invalid-boundary/rendered-diagnostic-parity/LSP-UTF-16 tests added (in `sifr_source::tests`, `sifr_syntax::tests`, `sifr_diagnostics` rendering path, and `sifr_lsp::conversion::tests`) ✓; dep-direction guardrail added and wired into `scripts/run_all_tests.sh:99-100` ✓; out-of-scope items (SourceProvider, WorkspaceSession, WorkspaceSnapshot, DirtyScope, cache reuse, scheduler, LSP request flow) untouched ✓.
- The locked API surface listed in the M0 description (`SourceText`, `LineMap`, `PositionEncoding`, `TextPosition`, `TextRangeUtf`, `SourceFile`) is all in `sifr_source::lib.rs` with the expected shape. `SourceFile` is not consumed anywhere yet — that is acceptable because M2/M3 own overlay lifecycle; the type is locked here so later milestones build on a stable name.

## Required change before PR

1. Fix the `clippy::needless_pass_by_value` failure on `SourceMapView::text_position_to_span` (either `&TextPosition` or `#[allow]`), and re-run `cargo clippy --workspace -- -D warnings` to confirm. Add the result to the validation log.

## Optional, not required

- The bench file duplicates the `TextPosition { line: 0, character: 0 }` literal three times in the round-trip check; a `let target = TextPosition { line: 0, character: 0 };` would tighten it. Pure readability, not a correctness issue.
- `sifr_source::SourceFile` is unused. That is fine — it is the locked M0 surface, picked up in M2.

Once the clippy fix is in and the full local validation script passes, M0 is approved to merge.
