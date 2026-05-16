

I'll now verify each gap from the pass-1 review against the current phase docs, and check whether the generated Rust preview correction is applied.

**Phase 35 verification:**

| Gap | Status | Evidence |
|-----|--------|----------|
| **A: PositionEncoding** | [OK] | `PositionEncoding` enum at lines 182-186. `text_position_to_span` takes `encoding: PositionEncoding` as parameter. Caller-responsible pattern allows both per-document and session-level storage. |
| **B: DocumentVersion in InvalidationReport** | [OK] | `UpdatedDocumentInfo` struct at lines 201-206 with `old_version`, `new_version`, and `text_changed`. `InvalidationReport::updated_documents` at line 198. |
| **C: Snapshot discipline** | [OK] | Concurrency model (lines 400-405) explicitly requires deterministic cancellation or content-modified response for stale snapshots. No partial result publication. |
| **D: SignatureHelpConfig** | [OK] | `SignatureHelpConfig` at lines 276-279 with `trigger_characters` and `retrigger_characters`. |
| **E: SemanticTokenLegend** | [OK] | `SemanticTokenLegend`, `SifrSemanticTokenType` (lines 286-303), and `SifrSemanticTokenModifier` (lines 306-319) all present with Sifr-specific categories (mutable, ownership, unsafe). |

**Phase 36 verification:**

| Gap | Status | Evidence |
|-----|--------|----------|
| **1: codeAction/resolve** | [OK] | Line 280: explicit, not conditional. Links deferred edit resolution to `codeAction.deferredEditResolution` capability. |
| **2: server-initiated requests** | [OK] | Lines 293-299: `workspace/configuration`, `window/showMessage`, `window/logMessage`, and optional `client/registerCapability`. |
| **3: file watcher** | [OK] | Line 248: `workspace/didChangeWatchedFiles` listed as required. Line 298: dynamic registration per `workspace.didChangeWatchedFiles.dynamicRegistration`. |
| **4: notebook rejection** | [OK] | Lines 321-322: explicit. `notebook_document_sync` unset, no notebook cell selectors, no notebook handlers. Notebook URIs rejected with deterministic protocol error. |
| **5: protocol test harness** | [OK] | Lines 478-486: models on `ty_server/tests/e2e`, subprocess stdio mode, request tracking, helper awaits, all messages consumed, injection tests. |
| **6: completion MRR** | [OK] | Lines 465-466: `truth/` fixtures, `completion.toml` per task, MRR evaluation, threshold, Sifr-specific tasks. |
| **7: unknown options** | [OK] | Lines 300-301: deterministic warning messages and logs, continues with defaults, does not prevent initialization unless workspace impossible. |
| **8: multi-workspace extensibility** | [OK] | Line 334: session model designed for per-workspace isolation without rewriting document storage, request dispatch, or `AnalysisHost`. |

**Generated Rust preview correction:**

Line 211: *"return a deterministic cancellation/content-modified response instead of partial generated code if a document change invalidates the request."* - [OK] The deterministic cancellation behavior is correctly specified. No partial results on cancellation.

**Ambiguous language fixes verified:**

- Selection range ordering (line 206): *"ordered outer-to-inner to match Phase 35 `SelectionRangeView::ranges_outer_to_inner`"* - [OK] explicit reference to Phase 35 contract
- Semantic token categories (lines 199-204): Required / required-where-concept-exists / Sifr-specific / optional classification - [OK] unambiguous
- Generated Rust preview cancellation (line 211): [OK] covered above

---

## VERDICT: **READY**

All pass-1 gaps are addressed. The generated Rust preview correctly specifies deterministic cancellation rather than partial results. No implementation ambiguity remains.

---

## Minor Follow-Up Notes (non-blocking)

These do not prevent implementation from proceeding:

1. **Position encoding storage** (Phase 35, Gap A): `SourceFileView` does not store `position_encoding`. The current design passes encoding per-call to `text_position_to_span`, which is a valid choice. Phase 36's `DocumentStore` can store the negotiated encoding once and pass it per-call. No contract conflict, but an explicit note (e.g., "Phase 36 stores negotiated encoding in `DocumentStore` and passes it to `text_position_to_span`") would clarify the session-level pattern.

2. **Snapshot interface documentation** (Phase 35, Gap C): The concurrency section describes stale snapshot behavior but does not enumerate the minimum `AnalysisSnapshot` interface. Phase 36 has enough context to define it from `SourceMapView`, `ModuleGraphView`, and `DocumentVersion` access. Consider adding a concise interface list in `milestone_36_3`'s session model section rather than here, to avoid Phase 35 owning an interface it doesn't implement.
