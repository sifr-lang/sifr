Here's the review. The plan captures the right principle and most of the verification axes, but it has a hard architectural mismatch, a few missing concrete deletions, and three classes of weak-assertion coverage that will let the same regression slip back in.

## Architectural risk (must fix before implementation)

**`sifr_analysis` does not own HIR or `FunctionType`.** Step 2 ("Build callable signatures from Sifr `FunctionType`/HIR data already owned by analysis") is misframed:
- `check_analysis_split_brain.py:15–27` forbids `HirModule`, `lower_module(`, and `compile_module_hir` inside `crates/sifr_analysis/**`.
- What analysis actually receives from the frontend is `ProjectAnalysisView { modules: Vec<ModuleAnalysisView { symbols: Vec<SymbolView { name, kind }> }> }` (`crates/sifr_frontend/src/graph_cache_and_queries.rs:215–238`). That is a name + kind tuple. There are no types, no parameter labels, no parameter types, no return type, no defining range.

The plan must explicitly require a new query in `sifr_frontend` (alongside `analysis_for_module` in `graph_cache_and_queries/reuse.rs:91`) that returns position-keyed semantic info: signature display string, parameter ranges within the signature label, parameter types, declared/inferred binding types, and span back to the source. HIR traversal stays in `sifr_frontend::query_diagnostics` (where `symbols_from_hir` already lives) so analysis stays split-brain-clean.

Without this, either the implementation violates the existing analysis guard, or it falls back to the very token rendering the plan tries to ban.

## Position-to-symbol resolution is unspecified

Step 3 says "Resolve identifier at cursor to a semantic symbol" but does not say where the bridge lives. The current path (`implementation.rs:702–713`) uses token-text identifier matching, which is name-only and scope-blind — two locals named `result` resolve identically.

Pin the contract: a frontend query keyed by `(FileId, TextSize)` returns a typed `SemanticInfoAtPosition` enum (function | parameter | local-binding | module | type | none) with display strings already rendered by the type printer. Without this, the LSP gets identifier-text equality and the corpus will only catch the obvious cases.

## Concrete deletions are missing from the plan

The plan says "guard against" patterns that should be *deleted*:
- `crates/sifr_analysis/src/host/implementation.rs:212–214` (`format!("{} ({})", token.text, token.kind)`) — delete.
- `crates/sifr_analysis/src/host/implementation.rs:882–899` (`call_identifier_before_position` returning `name(...)`) — delete, not "guard."
- `crates/sifr_analysis/src/host/implementation.rs:226` hard-coded `active_parameter: Some(0)` — must come from semantic analysis or be `None`.
- `crates/sifr_lsp/src/conversion.rs:209–215` `signature_help` always emits `"parameters": []` and `"activeSignature": 0` — that is a placeholder fallback identical in spirit to the bug. Must propagate real parameter labels and require the analysis schema (`queries.rs::SignatureHelp`) to carry them.

State these as deletions in section 5 (Implementation Shape) so reviewers know what disappears.

## Schema gap in `sifr_analysis::SignatureHelp`

`queries.rs:26–30` has only `label` + `active_parameter`. Plan section 4 says "include … parameter labels" but the schema cannot carry them today. Require a field like `parameters: Vec<ParameterLabel { range_in_label: (u32, u32) }>` so:
- The LSP `parameters` array is no longer a permanent `[]`.
- Active parameter index has something to index into.
- A static guard can reject `parameters: []` for callables with arity > 0.

## Static guard (section 1) is too abstract

Make the pattern set concrete, and add the negative direction:
- Reject any source line where `.kind` of an `EditorToken` is interpolated into a `HoverInfo::contents` or `SignatureHelp::label` initializer.
- Reject `format!("{}(...)", *)` constructing a `SignatureHelp { label: … }`.
- Reject `active_parameter: Some(0)` literal in any signature-help builder.
- Reject `SignatureHelp { … parameters: vec![] }` and the equivalent `"parameters": []` literal in `sifr_lsp/src/conversion.rs`.
- Self-test must include both directions: seeded-bad fails, seeded-good passes. The plan only requires the failing direction.

## Existing weak assertions the plan does not tighten

The same class of "non-null / contains-name" weakness that hid the original bug is in three more places — the plan must explicitly tighten them, not just add a parallel corpus:
- `verification/areas/developer_tooling/editor_query_snapshots/single_file_editor_queries.json:5–7` asserts `"hover": { "token": "value" }` — `value (Name)` would pass. Update the schema to require a typed semantic string and to forbid `(Name)` / `(Identifier)` / `NonLogicalNewline` suffixes.
- `verification/areas/developer_tooling/lsp_protocol_smoke.py:309–310` checks only `"helper" in contents.get("value", "")` — `helper (Name)` passes. Replace with an exact semantic-form check (`def helper(value: int) -> int` inside the `sifr` code block).
- `verification/areas/developer_tooling/lsp_marker_corpus/manifest.json` declares `hover` and `signature-help` *coverage* (lines 9, 27, 33) but the marker schema has no payload assertion. Either extend the manifest to carry per-marker expected payloads and update `check_lsp_marker_corpus.py` accordingly, or pivot those markers to the new corpus. The plan doesn't acknowledge the marker corpus exists.

## Stress / large-session coverage (section 5) is too vague

"Run hover and signature help after open/change/save" can pass even if responses are still token placeholders. Require:
- Typed-after-edit assertions: change `value: int = …` to `value: str = …` and assert hover flips from `value: int` to `value: str` on the same offset.
- Version coherence: hover and signature responses for version N must reflect version N, not N-1, after `didChange`.
- The shortened-document case must assert that hover at a still-valid offset returns the typed value and at a now-removed range returns `null` (not a stale token answer).

This is the only verification that catches a stale-snapshot regression, which is the natural failure mode once semantic data joins the snapshot queries.

## Stdlib (`randint`) acceptance is unsafe as written

`ModuleAnalysisView::symbols` is built by `symbols_from_hir` (`crates/sifr_frontend/src/query_diagnostics.rs:79–94`) over the active module only. Stdlib external defs flow in via `sifr_driver::stdlib_external_defs()` (`host/implementation.rs:41–43, 49–51`) but they are not surfaced as part of the per-module symbols view. The plan promises `randint(start: int, stop: int) -> int` hover but does not specify how external-def `FunctionType`s reach the new semantic-info query. Either:
- Extend the new frontend query to resolve cross-module / external-def symbols and pin that as an explicit deliverable, OR
- Downgrade the acceptance case to require `null` for stdlib symbols until a follow-up — consistent with "no fallback."

Right now this looks like an implicit fallback waiting to fail.

## Coverage gaps

- **`host/snapshot_queries.rs`.** Section 5 mentions snapshot stress but not that `snapshot_queries.rs` exists and must route hover/signature through the same semantic source. Add to implementation shape: "snapshot-rooted variants share the semantic path; no token rendering on either path."
- **UTF-16 parameter ranges.** The corpus already exercises UTF-16 positions for hover, but signature-label parameter ranges are a new wire-level concern. Add a Unicode parameter-name case (`def 🦀(x: int)` style or a string literal with `🦀` adjacent to params) to confirm parameter ranges report UTF-16 code units when negotiated.
- **`check_lsp_transcript_replay.py`.** Not mentioned. If transcripts contain golden hover/signature payloads, they must be regenerated; the guard should refuse a transcript whose hover response matches `\([A-Z][a-zA-Z]+\)` or whose signature label is exactly `<name>\(\.\.\.\)`.
- **In-tree fixture for the scribbles repro.** Acceptance cites the user's `scribbles/main.sifr`, which is outside the repo. Add an in-tree fixture under `verification/areas/developer_tooling/editor_query_corpus/semantic_hover/` that reproduces `def generate_random() -> int | None` + `x = generate_random()` so the regression has a permanent home.

## Summary

The principle and the verification *axes* are right. To prevent both regressions (token-kind hover returning, semantic ownership split-brain returning), the plan needs three concrete fixes:

1. Re-home the semantic data: a new frontend query (not "HIR already owned by analysis") returning typed `SemanticInfoAtPosition`, position-keyed, with rendered display strings — analysis stays thin.
2. Schema-update `sifr_analysis::SignatureHelp` to carry parameter labels and ranges; require the LSP conversion test to assert non-empty parameters; forbid the existing hard-coded `parameters: []` and `active_parameter: Some(0)`.
3. Tighten the three existing weak assertions (`editor_query_snapshots`, `lsp_protocol_smoke` UTF-16 check, `lsp_marker_corpus` coverage list) instead of only adding a new corpus alongside them — and add typed-after-edit assertions to the stress test.

Treat the `randint` acceptance bullet as load-bearing: either extend the new query to resolve external defs, or remove it and accept `null` for stdlib hover until a follow-up.
