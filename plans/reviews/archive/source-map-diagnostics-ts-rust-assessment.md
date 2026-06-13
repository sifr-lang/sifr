# Source Map and Diagnostic Architecture Cross-Check: TypeScript and rustc

This assessment checks the current Sifr proposal in `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md` against local TypeScript and rustc compiler implementations.

## TypeScript Patterns

Relevant files:

- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/types.ts`
- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/utilities.ts`
- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/scanner.ts`
- `/Users/yaseralnajjar/work/sifr/TypeScript/src/compiler/diagnosticMessages.json`
- `/Users/yaseralnajjar/work/sifr/TypeScript/src/services/sourcemaps.ts`

Observed design:

- `SourceFile` owns the source text, file names/paths, parse/bind diagnostics, and a cached `lineMap`.
- Nodes are `TextRange` values with `pos` and `end`; diagnostics store `file`, `start`, and `length`.
- Line/column is derived at the boundary using `getLineStarts`, `computeLineAndCharacterOfPosition`, and binary search over line starts.
- Diagnostic identities come from generated `DiagnosticMessage` entries with `category`, numeric `code`, and message text.
- Builders such as `createFileDiagnostic`, `createDiagnosticForNodeInSourceFile`, and `createCompilerDiagnostic` validate diagnostic ranges and keep compiler diagnostics without source separate from file diagnostics with source.
- `relatedInformation` and `DiagnosticMessageChain` are first-class structures, not text appended to one rendered string.
- `canonicalHead` exists for deduplication when related messages differ but the underlying problem is the same.
- `SourceMapperHost` can map generated/declaration locations back to source locations through document position mappers, with identity fallback for unmapped files.

Implications for Sifr:

- The proposal is right to preserve byte ranges internally and derive line/column only at rendering/serialization.
- `SourceId` should point to a source-map-owned source file record, not just a path string. That record should own source text, canonical path, display path, module name, line starts, and possibly a content hash.
- Diagnostic builders should validate spans against the owning source file text at construction time or at emission time.
- Sifr should keep a clear distinction between source diagnostics and compiler/internal diagnostics, mirroring TypeScript's file diagnostics versus compiler diagnostics.
- `message_template` plus scalar args is better than TypeScript's already-rendered message-only API for Sifr's compact grouping and JSON re-rendering goals.
- The proposal should explicitly add a canonical deduplication key or say `code + message_template + primary source/range + args subset` is the dedupe identity. TypeScript's `canonicalHead` shows this becomes necessary when a diagnostic can have optional extra context.
- If Sifr later emits generated Rust source maps, source-map records should support identity mappings first and generated-to-source mapping second, without embedding this in diagnostics themselves.

## rustc Patterns

Relevant files:

- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_span/src/source_map.rs`
- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_span/src/lib.rs`
- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_errors/src/diagnostic.rs`
- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_errors/src/json.rs`
- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_macros/src/diagnostics/mod.rs`
- `/Users/yaseralnajjar/work/sifr/rust/compiler/rustc_expand/src/errors.rs`

Observed design:

- `rustc_span` is a separate low-level crate. It owns `Span`, `BytePos`, `SourceFile`, `SourceMap`, file loading, source hashing, line starts, multibyte character tracking, path remapping, and source lookup.
- `Span` stores compact absolute byte positions in a `SourceMap`, not path/line/column.
- `SourceMap` stores source files in a monotonic collection so positions and indices stay stable.
- Files have stable IDs based on filename and crate identity; imported files can exist without full source text but still preserve enough line metadata.
- Source maps account for source normalization, multibyte characters, and remapped diagnostic paths.
- `span_to_lines`, `span_to_snippet`, `lookup_char_pos`, and JSON rendering derive display data at the output boundary.
- Diagnostics are typed structs implementing/deriving `Diagnostic`; spans and subdiagnostics are fields, often marked as primary spans, labels, notes, help, or suggestions.
- JSON diagnostics include `byte_start`, `byte_end`, 1-based line/column, source text lines, primary-span flags, labels, suggestions, and macro expansion backtraces.
- Diagnostics are constructed first, emitted through a diagnostic context, and have non-clone / must-use mechanics to avoid dropped or duplicated diagnostics.

Implications for Sifr:

- The proposal is right to put a new crate in front of HIR/driver, but it should separate source-map primitives from diagnostic catalog/rendering concerns inside `crates/sifr_diagnostics` so the crate does not become monolithic.
- `SourceSpan` should be a source-map span composed of `SourceId + TextRange` or equivalent interned-file-local byte range. The serialized `DiagnosticSpan` should be a boundary product, not the internal representation.
- `SourceId` should be stable within a compilation session and backed by a monotonic source-map store; do not derive it from vector positions that can be invalidated.
- The source map should store line starts and enough UTF-8/multibyte metadata to make byte offsets, character columns, and display columns intentionally different concepts.
- Path remapping/display policy should be first-class. Diagnostics should not leak absolute local paths in user-facing or JSON output unless the chosen display policy says so.
- The proposal should explicitly name `SourceMap`/`SourceManager` ownership in the driver/front-end and say HIR receives opaque `SourceId`/ranges, not source text or filesystem paths.
- Multi-span diagnostics should be first-class: primary span, labels, notes/help, related spans, and suggestions should all reference `SourceSpan` until rendering.
- Suggestions need applicability and multipart edit support. The current proposal has `DiagnosticSuggestion` but should state it supports multiple span replacement parts and applicability.
- Sifr should not copy rustc macro expansion backtraces now, but it should leave a general "origin chain" or "generated from" hook for future generated-code/source mapping. For Sifr, this may be useful for HIR desugaring and generated Rust emission.
- A diagnostic sink/context should be non-lossy: emitting a diagnostic should be deliberate, and guardrails should detect constructed-but-not-emitted diagnostics where practical.

## Proposed Additions to the Sifr Proposal

Add these clarifications:

1. Split `crates/sifr_diagnostics` internally into at least `source_map`, `codes`, `model`, `render`, and `schema` modules so the new crate remains focused and reviewable.
2. Define a `SourceMap`/`SourceManager` owned by the driver/front-end:
   - registers source text once,
   - assigns stable session-local `SourceId`,
   - stores canonical path, display path, module name, source hash, line starts, and source text,
   - validates `SourceSpan` ranges,
   - converts ranges to line/column/display spans only at render or JSON serialization.
3. Extend `SourceSpan` policy:
   - source diagnostics carry `SourceSpan`,
   - `DiagnosticSpan` is serialized/rendered output only,
   - byte offsets are canonical,
   - line/column are derived,
   - character column and display column are distinct where needed.
4. Add path display/remapping policy to avoid accidental local-path leakage.
5. Define deduplication identity explicitly: compact/recovery dedupe should use `code`, `message_template`, primary source/range, and a declared subset of args, not fully rendered messages.
6. Make suggestions multipart with applicability, modeled as edits over `SourceSpan`.
7. Add an origin-chain/future-generated-source hook for diagnostics that originate from lowered/desugared/generated code, while keeping current source diagnostics tied to real parser-origin spans.
8. Add source-map-specific tests:
   - multibyte UTF-8 columns,
   - multiline spans,
   - zero-length spans,
   - EOF spans,
   - multi-file project diagnostics,
   - path remapping/display path behavior,
   - invalid span rejection,
   - JSON byte/line/column consistency.

## Bottom Line

The current proposal is directionally aligned with TypeScript and rustc: byte ranges first, line/column at the boundary, structured diagnostics, generated docs/schema, and strong separation between source diagnostics and internal/compiler diagnostics.

The main missing details are source-map ownership, range validation, path remapping, display-column/multibyte handling, deduplication identity, and richer suggestions/origin metadata. These are not a reason to change the per-family diagnostic taxonomy. They are source-map architecture clarifications that should be added before `milestone_diag_1` starts.
