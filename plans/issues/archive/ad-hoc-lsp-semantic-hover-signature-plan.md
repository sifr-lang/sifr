# LSP Semantic Hover And Signature Verification Plan

## Problem

Sifr LSP hover currently leaks lexer-token metadata such as `generate_random (Name)` and `x (Name)`. That is not a semantic editor feature. It passed existing checks because the LSP smoke and stress tests only required hover to be non-null or to contain the token text.

The same root cause affects `textDocument/signatureHelp`: it returns heuristic labels such as `generate_random(...)` instead of the callable signature produced by Sifr analysis.

## Principle

No fallback and no backward-compatibility behavior for semantic editor answers. If a hover or signature answer is available, it must come from Sifr semantic analysis. If semantic analysis cannot produce an answer, the LSP should return `null` rather than token-kind placeholder text.

`sifr_lsp` remains a protocol adapter. `sifr_analysis` remains an editor-query
facade. Semantic authority belongs in `sifr_frontend` and the compiler data it
owns. Analysis may map an LSP position to a frontend semantic view, but it must
not traverse HIR, re-render compiler types, or infer signatures itself.

## Reference Expectations

TypeScript language server:
- Hover delegates to tsserver `quickinfo` and renders `displayString` as a typed code block.
- Signature help delegates to tsserver `signatureHelp`, returns full signatures, parameter labels, active signature, and active parameter.
- Completion resolve uses semantic details, not lexer token names.

Python/Pyright-style editor expectation:
- Hover over functions shows callable signature and return type.
- Hover over variables and parameters shows inferred or declared type.
- Signature help shows parameter labels and the active argument.
- Unknown or unsupported semantic positions should return no hover, not token-class text.

## Verification Area Additions

### 1. Static Guard: Semantic Placeholder Rejection

Add or extend a developer tooling guard that fails if production hover/signature help formats raw lexer kinds for user-facing semantic content.

Required checks:
- Forbid production hover content containing `token.kind` or equivalent raw token-kind formatting.
- Forbid signature help labels that are only `name(...)`.
- Forbid `sifr_analysis` from importing compiler semantic construction types such as `FunctionType` or `Type`.
- Allow token inspection only as a locator for the identifier/call site, not as the final answer.
- Keep self-tests that seed the forbidden patterns and prove the guard catches them.

### 2. LSP Protocol Semantic Corpus

Create a protocol-level corpus under `verification/areas/developer_tooling` that opens a real Sifr project and asserts exact semantic responses.

Cases:
- Function definition hover includes `generate_random`, `->`, and the canonical `int | None` return type.
- Function call hover returns the same callable semantic signature as the definition.
- Local variable hover from annotation includes `x: int`.
- Local variable hover from call result includes `x: int | None`.
- Parameter hover includes the parameter name and declared type.
- Imported project function hover/signature works across files.
- Imported stdlib function hover/signature for `randint` uses the real public signature `randint(minimum: int, maximum: int) -> Result[int, ValueError]`.
- Shadowed local names resolve to the nearest semantic binding, not the first token with the same text.
- Signature help for zero-arg call exposes no parameters and the callable return type.
- Signature help for multi-arg calls emits parameter labels and updates active parameter before/after comma, including a multi-line call.
- Negative hover over keywords, whitespace, strings, and comments returns `null`.
- UTF-16 positions still resolve when non-ASCII text appears before the query target.
- No placeholder leakage: response text must not contain raw lexer labels such as `(Name)`, `(Identifier)`, or `NonLogicalNewline`.

### 3. Analysis Unit Tests

Add focused `sifr_analysis` tests for semantic editor answers without LSP JSON conversion:
- Hover over local function definition and call returns the same function signature.
- Hover over annotated local binding returns the declared type.
- Hover over inferred local binding returns the inferred type.
- Hover over function parameter returns the parameter type.
- Signature help returns labels and parameter labels from the same semantic source as hover.
- Unsupported positions return `None`.

### 4. Conversion Tests

Keep `sifr_lsp` conversion thin but covered:
- Hover renders semantic contents as a `sifr` markdown code block.
- Signature help includes signature label and parameter labels.
- No conversion path introduces placeholder labels.

### 5. Regression Stress

Extend the existing LSP stress test to run semantic hover and signature help after open/change/save sequences, including shortened documents, so semantic ranges and answers stay current after project owner refreshes.

### 6. Related LSP Defects To Track

This work fixes hover and signature help. It should also document the same
position-resolution weakness in definition/declaration/type-definition,
references, rename, type hierarchy, and source-derived inlay hints. Do not add
fallbacks to those paths in this PR; either reuse the new semantic surface where
low-risk, or leave explicit follow-up notes.

## Implementation Shape

1. Extend `sifr_frontend::ModuleAnalysisView` with frontend-owned editor semantic entries, including callable signatures, binding types, source ranges, and callable call ranges.
2. Build those entries while frontend has both parsed AST ranges and lowered HIR semantic types available.
3. Expose fully rendered source-level semantic text from frontend views so `sifr_analysis` does not import HIR or type-system construction APIs.
4. Resolve identifier/call positions in `sifr_analysis` by querying the frontend semantic entries for the file/module.
5. Resolve signature help active parameter from frontend-owned call argument ranges, including multi-line and UTF-16-positioned calls.
6. Remove token-kind hover rendering entirely.
7. Keep token scanning only for non-semantic editor features such as semantic token coloring, selection ranges, and as a last-mile locator when the semantic view already owns the final answer.

## Acceptance

- The `scribbles` example returns semantic hover for `generate_random` and `x`.
- The direct LSP protocol tests assert semantic content, not only non-null responses.
- Static guards reject reintroducing token-kind placeholder hover or `name(...)` signature help.
- Shadowed variables, project imports, stdlib imports, active parameter, unsupported hover positions, and post-change refresh are covered in verification.
- No project-rooted fallback and no semantic fallback paths are introduced.
- Local validation passes for focused tests and `scripts/run_all_tests.sh --profile create-pr`.

## Current Status

- Implemented frontend-owned `EditorSemanticView` for semantic hover entries and callable signature entries.
- Removed token-kind hover/signature responses from `sifr_analysis`; unsupported semantic positions now return `None`.
- Kept `sifr_lsp` as a JSON conversion layer for hover/signature data, including parameter labels.
- Added `lsp-semantic-editor` verification with function, variable, parameter, project import, stdlib import, try/except stdlib call, active-parameter, UTF-16/non-ASCII, null-hover, and post-change cases.
- Extended LSP smoke/stress checks and static split-brain guards.
- Direct protocol probe of `/Users/yaseralnajjar/work/sifr/scribbles/main.sifr` copied into a temp package with `sifr.toml` returns semantic hover/signature for `generate_random`, local `x`, and `randint`.

## Review Disposition

- Claude implementation review artifact: `plans/reviews/active/lsp_semantic_hover_implementation_review_4.md`.
- Position encoding risk addressed with a UTF-16 corpus case containing non-ASCII text before the hover target.
- The supplementary AST walk is constrained to frontend semantic view construction and uses the frontend callable signature table; analysis and LSP do not infer or render semantics. It fills source call ranges for lowered constructs whose HIR statement pairing is transformed, without adding a fallback answer path.
- Incremental rebuild risk is covered by post-change semantic corpus checks; requests use the current `WorkspaceSession` analysis view and no shared mutable semantic cache is added outside frontend query caching.
- Remaining Claude suggestions for keyword/named-argument active-parameter behavior, recursive callable signatures, and memory sizing are useful follow-up coverage, but not blocking for this root-cause fix.
