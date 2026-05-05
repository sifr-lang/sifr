# Review: Source Map and Diagnostic Architecture Cross-Check

This review evaluates [reviews/source-map-diagnostics-ts-rust-assessment.md](reviews/source-map-diagnostics-ts-rust-assessment.md) against the parent proposal in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) and against the local TypeScript and rustc trees at `/Users/yaseralnajjar/work/sifr/TypeScript` and `/Users/yaseralnajjar/work/sifr/rust`.

## Verdict

The assessment is directionally correct. Its core claims are accurate against the reference implementations:

- TypeScript stores diagnostics as `file + start + length` and derives line/column from `getLineStarts` / `computeLineAndCharacterOfPosition` at the boundary (`TypeScript/src/compiler/scanner.ts`, `utilities.ts`). Generated `DiagnosticMessage` entries with numeric `code` plus `category` are the identity (`diagnosticMessages.json`). `DiagnosticMessageChain` and `relatedInformation` are first-class. `SourceMapperHost`/`DocumentPositionMapper` does generated-to-source mapping for declarations and emit (`services/sourcemaps.ts`). All confirmed.
- rustc owns spans in a separate low-level crate (`rustc_span`) with `BytePos`, `SourceFile`, monotonic `SourceMap`, line starts, multibyte tracking, normalization (`NormalizedPos`), and remapped paths (`RealFileName`). JSON spans serialize `byte_start`, `byte_end`, 1-based `line_start`/`line_end`/`column_start`/`column_end`, source `text`, `is_primary`, `label`, `suggested_replacement`, and `suggestion_applicability` (`rustc_errors/src/json.rs:182-220`). Construction goes through builders with a destructor bomb that panics on constructed-but-not-emitted (`rustc_errors/src/diagnostic.rs:1326-1340`). All confirmed.

The seven proposed additions in §"Proposed Additions to the Sifr Proposal" are individually right and not over-engineered, with one minor caveat (see Findings F8). However, the assessment **under-specifies** a few items that the reference implementations treat as load-bearing, and **omits** a few patterns whose absence will force re-work after `milestone_diag_1` locks the model.

The findings below are gaps to fold into the proposal — not directional course corrections — and should be patched before `milestone_diag_1` ships, since `milestone_diag_1`'s DoD says source diagnostics cannot be constructed without a `SourceSpan` and JSON serialization must be lossless.

## Findings

Severity scale: **High** = will require model/schema change after `milestone_diag_1` if not addressed now; **Medium** = ambiguity that will surface as fixture or renderer churn; **Low** = forward-looking nice-to-have.

### F1 — Diagnostic emission discipline is implied but not proposed (High)

The assessment notes in line 70 that "a diagnostic sink/context should be non-lossy: emitting a diagnostic should be deliberate, and guardrails should detect constructed-but-not-emitted diagnostics where practical." This implication never makes it into the §"Proposed Additions" list. rustc enforces this with a `Drop` impl on `Diag` that panics if a builder is dropped without `.emit()`/`.cancel()` ([rustc_errors/src/diagnostic.rs:1326-1340](../../rust/compiler/rustc_errors/src/diagnostic.rs)). The Sifr proposal already uses a `DiagnosticSink` model (proposal lines 384, 474), which is the natural home for this discipline.

**Patch direction**: add an explicit addition #9: a `DiagnosticBuilder`/`Diag` value is `#[must_use]`, non-`Clone`, and either consumed by `sink.emit(...)` or explicitly `.cancel()`-ed. Drop without consume is a programmer bug: panic in debug, route to `SIFR-INTERNAL-*` in release. This belongs in `milestone_diag_1` because retrofitting move-only builders later requires touching every emission site again.

### F2 — Per-span labels and `is_primary` are not pinned in the model (High)

The proposal has `primary_span: SourceSpan` and `related_spans: Vec<RelatedSpan>` but never states that each span (primary and related) carries an optional label string and that the JSON envelope flattens both into a single span list with an `is_primary: bool` flag — exactly rustc's serialized shape ([rustc_errors/src/json.rs:197-220](../../rust/compiler/rustc_errors/src/json.rs)). The assessment notes "primary span, labels, notes/help, related spans, and suggestions should all reference `SourceSpan` until rendering" (line 67) but doesn't translate that into an explicit field on `RelatedSpan`/`SourceDiagnostic`.

**Patch direction**: in §"Source Mapping Architecture" or the proposed additions, state:

- `RelatedSpan { span: SourceSpan, label: Option<String>, kind: RelatedKind }` where `RelatedKind` distinguishes label / note / origin / replacement-target.
- The serialized JSON form is one flat `spans: [...]` array per diagnostic where each entry has `is_primary: bool` and an optional `label`. This matches rustc's wire format and makes consumer code identical for primary and related spans.

### F3 — JSON output should embed source-text snippet per span (High)

rustc's `DiagnosticSpan` carries a `text: Vec<DiagnosticSpanLine>` field with the actual line text and `highlight_start`/`highlight_end` offsets ([rustc_errors/src/json.rs:210-229](../../rust/compiler/rustc_errors/src/json.rs)). This is the difference between a JSON consumer that needs the file on disk to render a code frame and one that doesn't. The assessment's serialized `DiagnosticSpan` (line 432-440 of the proposal) has no `text` field. For LSP-style consumers and the compact renderer this gap is paper-cut-level today and structural after `milestone_diag_1` lands.

**Patch direction**: extend the proposal's `DiagnosticSpan` with `lines: Vec<DiagnosticSpanLine>` where each line is `{ text, highlight_start, highlight_end }` (1-based char columns). Renderers derive lines from the source map at serialization time; the source map already owns line starts, so cost is one slice per line. Add a fixture exercising a multiline span.

### F4 — Normalized vs original byte offsets is unspecified (Medium)

The assessment mentions "source normalization, multibyte characters, and remapped diagnostic paths" (line 51) but does not declare whether the JSON `byte_start`/`byte_end` Sifr emits correspond to **on-disk byte offsets** or **post-normalization byte offsets**. rustc maintains a `NormalizedPos` table specifically so it can emit `original_relative_byte_pos(...)` — i.e. on-disk offsets — even though internal positions are post-normalization ([rustc_span/src/lib.rs:1614, 2385-2405](../../rust/compiler/rustc_span/src/lib.rs); [rustc_errors/src/json.rs:494-499](../../rust/compiler/rustc_errors/src/json.rs)). If Sifr ever normalizes BOM or CRLF and skips this mapping, JSON byte offsets will silently disagree with what an editor sees in the file.

**Patch direction**: pin a policy in §"Source Mapping Architecture":

- Either: source map stores text verbatim; no normalization; JSON byte offsets are on-disk offsets. (Simplest. Recommended for current Sifr scope.)
- Or: source map normalizes; emits a `NormalizedPos`-equivalent table; JSON byte offsets are mapped back to on-disk offsets at the boundary.

Pick one and write a test that opens a CRLF or BOM file and asserts that `byte_start` matches `wc -c` byte position.

### F5 — JSON `column` units are not declared (Medium)

The proposal's `DiagnosticSpan` has `column: Option<u32>` and `end_column: Option<u32>` with no statement of units. rustc emits 1-based **character** columns ([rustc_errors/src/json.rs:201-206, 498-499](../../rust/compiler/rustc_errors/src/json.rs)). Display column (which accounts for tab width and East Asian wide characters) is a renderer-only concept. The assessment touches this with "character column and display column are distinct where needed" (line 88) but stops short of declaring what the JSON wire format is.

**Patch direction**: state explicitly that JSON `column` is 1-based UTF-8 character offset within the line (matching rustc) and that display column is internal to the human renderer. Same goes for `line` (1-based, matching rustc and TypeScript). This is a one-line addition that saves a future fixture rewrite.

### F6 — Suggestion applicability levels are not enumerated (Medium)

The assessment proposes "Make suggestions multipart with applicability, modeled as edits over `SourceSpan`" (addition #6). It doesn't name the applicability values. rustc uses a 4-variant enum ([rustc_lint_defs/src/lib.rs:66-85](../../rust/compiler/rustc_lint_defs/src/lib.rs)):

- `MachineApplicable` — safe to apply automatically.
- `MaybeIncorrect` — the suggestion may not match user intent.
- `HasPlaceholders` — contains literal placeholders that need user editing.
- `Unspecified` — applicability unknown.

This is the right level of granularity for an LSP/code-action consumer. Naming it now lets the JSON schema lock it.

**Patch direction**: define `SuggestionApplicability { MachineApplicable | MaybeIncorrect | HasPlaceholders | Unspecified }` on `DiagnosticSuggestion`. Per rustc, only `MachineApplicable` is auto-applied by tooling. Multipart edits (rustc's `Substitution { parts: Vec<SubstitutionPart> }`) — model as `Vec<SuggestionEdit { span, replacement }>`; emitting multi-part suggestions is not required in `milestone_diag_1`, only the type shape is.

### F7 — Recovery dedup identity vs compact grouping key are conflated (Medium)

The assessment proposes a single dedup identity: `code + message_template + primary source/range + args subset` (addition #5). The proposal already specifies a separate compact grouping key: `(severity, code, message_template, primary file)` (proposal lines 561, 739, 888). These are two different keys serving two purposes:

- **Recovery dedup**: prevents the same diagnostic from being emitted twice during HIR error tainting. Should include enough span/arg granularity to distinguish "same rule, different occurrence."
- **Compact grouping**: collapses a category of related diagnostics into one human-readable summary line. Coarser by design.

Conflating them invites either over-grouping (real duplicates of the same span survive) or under-grouping (one type-mismatch per call site shown separately in compact mode).

**Patch direction**: rewrite addition #5 to define both keys side-by-side and tie each to a milestone — recovery dedup belongs in `milestone_diag_10`, compact grouping in `milestone_diag_4a`. Both must use `message_template`, never rendered `message`.

### F8 — Origin-chain hook is sized correctly but mistitled (Low)

Addition #7 asks for a hook for "lowered/desugared/generated code" but frames it as a future-generated-Rust source-map concern. In practice the more immediate need is HIR desugaring — e.g. `for x in xs:` lowering into iterator-protocol calls where the diagnostic should still point at the `for` syntax. rustc handles this with macro-expansion backtraces; for Sifr the equivalent is HIR's lowering chain. The assessment correctly says "leave a hook" rather than build it; that's the right scope. Just rename the hook so it's clearly about lowering provenance, not just emitted Rust.

**Patch direction**: in addition #7, replace "origin-chain/future-generated-source hook" with "lowering-origin chain: every `SourceSpan` may carry an optional `lowered_from: SourceSpan` parent for desugared/synthesized HIR nodes." Generated Rust source-mapping is a later concern that rides on the same field.

### F9 — Stable source-file identity for incremental/LSP is not flagged (Low)

The assessment says `SourceId` should be "stable within a compilation session" (line 64). rustc additionally has `StableSourceFileId` derived from filename + crate identity so the source file can be looked up across sessions for incremental compilation ([rustc_span/src/lib.rs](../../rust/compiler/rustc_span/src/lib.rs)). Sifr does not need this today, but the source-map record should keep `canonical_path` and `content_hash` fields that a future stable id can be derived from without a model change.

**Patch direction**: in addition #2, confirm that the source-map record stores `canonical_path: PathBuf` and `content_hash: [u8; 32]` (or equivalent) even though current users only consume `SourceId`. No need to expose stable ids now. This is forward-looking only.

### F10 — Test list is good; one item under-specified (Low)

Addition #8's test list is solid. The "JSON byte/line/column consistency" item should expand into:

- byte offsets agree with on-disk byte offsets (covers F4),
- column is 1-based UTF-8 character offset, not byte (covers F5),
- end position is exclusive byte / 1-based-inclusive char column (the rustc convention),
- multibyte file at U+1F600 (4-byte char) produces consistent byte vs char column.

## Items the assessment got right and shouldn't change

- Byte ranges first, line/column at the boundary.
- `SourceId` opaque, owned by a `SourceManager` in the driver, monotonic store, no path strings inside HIR.
- Strict source-vs-internal diagnostic split (mirroring TS file vs compiler diagnostics).
- `message_template` + named scalar args is the right call for compact grouping and JSON re-rendering. TypeScript ships rendered messages only and would block this if Sifr copied it.
- Path remapping/display policy as a first-class concern.
- Crate split inside `crates/sifr_diagnostics` (`source_map`, `codes`, `model`, `render`, `schema`).
- Synthesized HIR nodes inheriting nearest parser-origin span — already in the proposal at line 454; the assessment's reaffirmation is correct.

## Items that would be over-engineered if added now

- Hygiene / `SyntaxContext`-style span encoding (rustc has it; Sifr has no macro system, no need).
- Multi-part suggestion **emission** (the type shape is fine; the generator infrastructure is premature).
- Caching source-map view (rustc's `caching_source_map_view.rs`); irrelevant at Sifr's current file count.
- Fluent-style i18n message store; rustc invested heavily here, TypeScript did not, and Sifr has only English diagnostics today.
- A second JSON schema version field on individual diagnostics; the proposal's envelope-only `version: 1` is correct.

## Patch summary for the proposal

The assessment should be folded into the proposal with these concrete edits before `milestone_diag_1` opens:

1. Add to §"Source Mapping Architecture": JSON `byte_start`/`byte_end` are on-disk byte offsets; `line`, `column`, `end_line`, `end_column` are 1-based UTF-8 character offsets within line. (F4, F5)
2. Extend `DiagnosticSpan` with `is_primary: bool`, `label: Option<String>`, and `lines: Vec<DiagnosticSpanLine { text, highlight_start, highlight_end }>`. Flatten primary + related into one JSON span list. (F2, F3)
3. Define `SuggestionApplicability { MachineApplicable | MaybeIncorrect | HasPlaceholders | Unspecified }` on `DiagnosticSuggestion`. Multipart edits modeled as `Vec<SuggestionEdit>`; emission of multipart edits not required in `milestone_diag_1`. (F6)
4. Add §"Diagnostic Emission Discipline": `Diag`/builder is `#[must_use]`, non-`Clone`; drop without consume panics in debug and routes to `SIFR-INTERNAL-*` in release. (F1)
5. Split addition #5 into "recovery dedup identity" and "compact grouping key" with explicit tuples and milestone owners. (F7)
6. Rename addition #7 to "lowering-origin chain" and define `lowered_from: Option<SourceSpan>`. (F8)
7. In addition #2, list source-map record fields explicitly: `source_id`, `canonical_path`, `display_path`, `module_name`, `text`, `line_starts`, `content_hash`, optional `normalization_table`. (F9, F4)
8. Expand the JSON-consistency test bullet in addition #8. (F10)

## Bottom line

Approve the assessment's directional conclusions. Block `milestone_diag_1` from starting on the strength of the assessment alone — fold findings F1–F7 into the proposal first so the canonical model and JSON schema lock the right shape on the first try. F8–F10 are forward-looking and can be addressed during `milestone_diag_1` implementation.
