## Wave 9.1 marker/capability corpus review

Cross-checked inventory against `crates/sifr_lsp/src/capabilities.rs`, manifest against fixtures, validator semantics, and runner wiring.

### Q1 — Inventory anchored to advertised capabilities

Anchored honestly for this slice. Every entry's `source_token` is the actual literal key written in `server_capabilities()` (lines 26–97):
- `textDocumentSync`, `diagnosticProvider`, `completionProvider`, `hoverProvider`, `signatureHelpProvider`, `definitionProvider`, `declarationProvider`, `typeDefinitionProvider`, `referencesProvider`, `renameProvider`, `documentSymbolProvider`, `workspaceSymbolProvider`, `semanticTokensProvider`, `inlayHintProvider`, `documentHighlightProvider`, `foldingRangeProvider`, `selectionRangeProvider`, `typeHierarchyProvider`, `codeActionProvider`, `executeCommandProvider`, `workspaceFolders`, `documentFormattingProvider` — all match.
- All six execute-command names match the literal command list in `capabilities.rs:78-86`.

Soft notes (non-blocking):
- `documentRangeFormattingProvider` (advertised on line 94) is folded into the `formatting` entry's methods rather than tracked as its own capability. Acceptable as long as later slices treat them together.
- `advertised_when: "formatEnable"` is a camelCase doc string; source uses `format_enable`. Not validated against source — harmless but slightly misleading.
- The check is one-directional (inventory → source). It will not flag a *new* capability added to `capabilities.rs` that the inventory forgot. Mentioned again under Q3.

### Q2 — Marker corpus honesty vs Wave 9.1 categories

Required categories declared in the manifest (`completion, definition, diagnostics, hover, long-session-edits, project-reload, references, rename`) all map to category strings on covered capabilities, and the manifest preamble is explicit that markers are "stable anchors … concrete protocol replay and snapshot expansion are tracked in later Wave 9 slices." That framing is true to what the fixtures provide — five lines of source can't honestly carry semantic-tokens + inlay-hints + selection-ranges + 13 other behaviors, but they can carry anchor points for a later slice. Description disclaims the rest.

Soft notes:
- `core-query-markers` advertises 16 capability covers via 5 anchor markers. Honest only because the manifest description says so. A future Wave 9.x replay slice should attach each marker to a specific capability (or replace this case-level mapping with per-marker `covers`).
- `loop-folding-range` anchors a 2-line `for` body; real folding ranges typically need ≥ 3 lines. Fine for marker placement, but later replay needs a beefier loop.
- `diagnostics.sifr` has `policy-suppression-action` and `explain-diagnostic-command` markers stacked as bare comments on lines 7–8, both pointing at the same diagnostic location. Sufficient as anchors; later code-action / executeCommand replay will need distinct positions.

### Q3 — Validator: contract gaps caught vs false-confidence risks

What the validator catches well:
- Inventory schema, sorted+unique ids, source-file existence, `source_token` present in `capabilities.rs`, command tokens present in `capabilities.rs`.
- Manifest schema, sorted+unique case ids, fixture existence, every declared marker appears as `# @lsp-marker <id>` in the fixture (or `supporting_files`), unknown-capability rejection, marker_required coverage, required-category coverage via category-of-covered-capability.
- Self-test exercises three meaningful negatives (missing capability coverage, marker absent from fixture, source token missing).

Gaps that could allow drift past the gate:
- **No reverse check from `capabilities.rs` → inventory.** A new capability added to the server is invisible until a human updates the inventory. For Wave 9.1 this is acceptable, but consider adding a future check that walks `server_capabilities()` keys and asserts each is in the inventory.
- **No duplicate-marker check** inside a case's `markers` list (only `covers` duplicates are caught). A copy-paste in the manifest would pass.
- **`supporting_files` type check is permissive.** If a user writes `"supporting_files": "x.sifr"` (string, not list), Python iterates characters and the error surfaces only as "fixture missing: x" — confusing diagnostic, not a missed contract.
- **Self-test does not cover** unsorted/duplicate ids, malformed JSON, missing required-categories, unknown-capability coverage. Negative coverage is narrower than the positive validator.

None of these create false confidence on the actual claims of the slice — the asserted contract ("inventory matches advertised tokens; markers exist; required behaviors covered") is enforced.

### Q4 — `lsp-smoke` / create-pr wiring

Correct. `runner.py:96-101` adds `lsp-marker-corpus` and `lsp-marker-corpus-self-test` as two of the four variants in `lsp-smoke`, which is in `FULL_SUITES` and is already promoted to create-pr per the tracker. The variant count (4) matches the validation evidence in the prompt. Self-test is the standard pattern used by every other check in this file. Owner is `compiler/tooling` consistent with the rest of the area.

### Q5 — File-size / manifest / profile / tracker

- `check_lsp_marker_corpus.py` is 211 lines, all fixtures < 20 lines, JSON files small. No file-size guardrail issue.
- `verification/areas/developer_tooling/manifest.json` does not need a new suite — `lsp-smoke` is reused as designed.
- Profile ownership claim in the tracker matches reality: create-pr already runs `developer_tooling` `static,lsp-smoke`, and `full` includes it.
- Tracker section at `plans/issues/.../ad-hoc-world-class-verification-standard-and-gate-closure.md:1486-1491` accurately reflects status, scope, and the validation already run.

### Verdict

No blockers. The slice is honest about what it ships (marker anchors + inventory hygiene, not protocol replay), the validator enforces the contract it claims, and the wiring is correct. The notes above are follow-up suggestions for later Wave 9 slices, not gate-closing concerns.

No further review round needed.
