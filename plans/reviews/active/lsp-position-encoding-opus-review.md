Confirmed: diagnostic columns are `prefix.chars().count()` — UTF-32 codepoint count — emitted unconditionally by `conversion::diagnostic`, bypassing encoding negotiation.

## Review

The encoding negotiation, the conversion-layer threading, the UTF-16-by-default policy, and the new diagnostic-jobs split are well-executed. Validation passed end-to-end. One blocking gap remains and a few minor cleanups.

### Blocking

**`conversion::diagnostic` is the missed encoding boundary** — `crates/sifr_lsp/src/conversion.rs:397-421` and the helpers at `:483-494`.

`DiagnosticSpan.column`/`end_column` are populated by `sifr_diagnostics::render::line_column` (`crates/sifr_diagnostics/src/render/mod.rs:284`) as `prefix.chars().count() + 1` — i.e. a **UTF-32 codepoint count**, used for terminal rendering. `conversion::diagnostic` emits that value directly as the LSP `character` field with no encoding translation and no source-text input. After this PR the server may negotiate UTF-16 (default) or UTF-8, and in both cases the diagnostic range is wrong:

- UTF-8 clients: off by `(utf8_bytes − codepoints)` for any non-ASCII byte on the line (every multi-byte character on the prefix).
- UTF-16 clients: correct for BMP, but off by `−1` per non-BMP character (emoji, mathematical symbols, etc.) — exactly the category of input that motivated negotiating an encoding in the first place.

Effect: misplaced squigglies, quick-fix targeting the wrong span, and (because `code_action` reads the client-supplied `range` through the encoded boundary) potential downstream encoding mismatches when the client echoes a diagnostic range back. The PR description lists "ranges" and "code actions" as in-scope boundaries, and `crates/sifr_lsp/src/diagnostics.rs:131,141` is the only converter that still ignores `session.position_encoding()`.

Fix sketch: thread `(source, encoding)` into `conversion::diagnostic`, build a `TextRange` from `span.byte_start`/`byte_end`, and route through the existing `text_range_with_encoding`. The byte offsets are already on the struct — no rendering changes needed.

### Non-blocking

1. **`lsp_position_to_utf8` skips validation when the negotiated encoding is UTF-8** (`crates/sifr_lsp/src/conversion.rs:50-53`). UTF-16/UTF-32 paths validate the position via `byte_offset_with_encoding`; UTF-8 returns the raw client position. Pre-existing asymmetry, but now reachable by any UTF-8-capable client.

2. **`prepare_rename` end-position math** (`crates/sifr_lsp/src/requests/navigation.rs:62-67`) still assumes the symbol begins at the cursor and uses `target.symbol.name.len()` (UTF-8 byte length). The encoding round-trip now goes through `text_position`, which is internally consistent (UTF-8 in, UTF-8 column + UTF-8 byte length → re-encode). But clicking mid-identifier produces a wrong range — pre-existing, just inherited unchanged by this PR.

3. **`lsp_range` and `text_range` are now trivial pass-throughs** to their `_with_encoding` siblings (`crates/sifr_lsp/src/conversion.rs:81-87,117-123`). Two of the four can be folded together.

4. **The UTF-16 smoke check is too lax to detect the regression it's meant to catch** (`verification/areas/developer_tooling/lsp_protocol_smoke.py:259-282`). Line `    δ: int = helper(41)` with position `(line=4, character=14)` lands inside `helper` for both UTF-16 (col 14 → `e`) and UTF-8 (byte 14 → `h`). The assertion only checks `"helper" in contents` — it passes regardless of which encoding the server used to decode the position. Add a non-BMP character (e.g. emoji) and pick a column where UTF-16 vs UTF-8 disagree by more than the symbol's length, then assert exact placement.

5. **`Session::position_encoding` defaults to `Utf16` before `initialize`** (`crates/sifr_lsp/src/session.rs:51`). Safe in practice (the server cannot serve requests before initialize), but folding the negotiated encoding into a constructor argument once initialize is observed would remove the implicit-default window.

### Other notes

- `diagnostic_jobs.rs` extraction is a clean split — keeps `session.rs` at 882 lines (under the 900 cap) and the new module has no encoding involvement.
- All other touched request handlers (`semantic_tokens`, `inlay_hint`, `folding_range`, `selection_range`, `formatting`, `code_action`, `navigation`, `symbols`, `type_hierarchy`) correctly read `session.position_encoding()` and propagate it to the conversion layer. The new `document_position` helper centralizes inbound position decode cleanly.
- `negotiated_position_encoding` priority list (UTF-8 > UTF-32 > UTF-16 default) ignores client preference order. Spec says "any of the encodings is supported by the client" with no ordering contract on the server side, so this is a defensible choice; just worth being aware of if a client offers `["utf-32", "utf-8"]` expecting UTF-32.

Address finding 1 before merging.
