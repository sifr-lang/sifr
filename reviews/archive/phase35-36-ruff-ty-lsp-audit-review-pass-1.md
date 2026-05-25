

Based on my thorough review of the phase docs and the local Ruff/ty code paths you specified, here is my detailed audit:

---

## VERDICT: **NOT READY**

The phase docs are well-structured and the architectural direction is sound, but there are concrete gaps that would cause implementation problems. Below are the specific gaps with recommended text.

---

## 1. Concrete Phase 35 Gaps for Production LSP

### Gap A: Position Encoding Metadata Missing

**Location**: Phase 35 `SourceMapView` / `SourceFileView`

The local `ty_server/src/document/range.rs` converts LSP positions using `PositionEncoding` (UTF-8/UTF-16/UTF-32) via `lsp_position_to_text_size()`. LSP clients may send UTF-8 or UTF-16 positions, and Sifr must handle both.

**Current Phase 35** `SourceFileView` has:
```rust
pub document_version: Option<DocumentVersion>,
```

**Missing**: `position_encoding: PositionEncoding` per document.

**Recommended text to add to Phase 35 `SourceFileView`:**
```rust
pub position_encoding: PositionEncoding,

// where:
pub enum PositionEncoding {
    UTF8,
    UTF16,
    UTF32,
}
```

This must be set from the LSP client's `general.clientPositionEncoding` capability and stored per-document in the source map.

---

### Gap B: Document Version in InvalidationReport

**Location**: Phase 35 `InvalidationReport`

The local `ty_server/src/document/text_document.rs` tracks `DocumentVersion` per document and rejects stale updates via `debug_assert!(self.version >= old_version)`. The local `request_queue.rs` uses document version for cancellation.

**Current Phase 35** `InvalidationReport` has:
```rust
pub invalidated_modules: Vec<ModuleId>,
pub invalidated_queries: Vec<QueryKind>,
```

**Missing**: Document version information in the invalidation signal. When a document changes, the LSP session needs to know which document versions are now stale vs. which are still valid for in-flight queries.

**Recommended text to add to Phase 35 `InvalidationReport`:**
```rust
pub updated_documents: Vec<UpdatedDocumentInfo>,

// where:
pub struct UpdatedDocumentInfo {
    pub file: FileId,
    pub old_version: DocumentVersion,
    pub new_version: DocumentVersion,
    pub text_changed: bool,  // true if source text changed, false if only version bumped
}
```

Without this, the LSP layer cannot implement the "ignore stale query results for superseded document versions" requirement from Phase 36's contract.

---

### Gap C: Snapshot Model Is Implicit, Not Explicit

**Location**: Phase 35 `AnalysisHost` API

The local `ty_server/src/server/main_loop.rs` creates explicit `DocumentSnapshot` and `SessionSnapshot` objects that carry revision metadata. Every background handler receives a snapshot rather than the live session.

**Current Phase 35** `AnalysisHost` methods take `&mut self` and return `QueryResult`. There is no explicit snapshot concept.

**Problem**: Without explicit snapshots, the LSP layer cannot safely implement concurrent query processing. Every document change would need to lock the entire `AnalysisHost`.

**Recommended text to add to Phase 35:**

In the "Concurrency model for Phase 35" section, add:
```rust
/// Snapshot model for the LSP layer.
///
/// Phase 35's `FrontendContext` does not implement explicit snapshots.
/// The LSP layer in Phase 36 must implement its own snapshot discipline
/// over `FrontendContext` - creating a consistent view of the source map
/// and module graph for the duration of a request, so that concurrent
/// queries see coherent state even when document updates arrive during
/// processing.
///
/// The minimum snapshot interface needed by `sifr_lsp` is:
/// - `AnalysisSnapshot::source_text(file: FileId) -> SourceText`
/// - `AnalysisSnapshot::module_graph() -> ModuleGraphView<'_>`
/// - `AnalysisSnapshot::document_version(file: FileId) -> Option<DocumentVersion>`
/// - `AnalysisSnapshot::query(AnalysisQuery) -> QueryResult<AnalysisResult>`
///
/// Snapshot validity is tracked by `SourceRevision` and `GraphRevision`.
/// Stale snapshots are rejected at the LSP protocol layer before
/// reaching `sifr_frontend` query methods.
```

This documents the requirement that Phase 36's LSP must add a snapshot layer, even though Phase 35 doesn't implement one.

---

### Gap D: Signature Help Trigger/Retrigger Characters Not Defined

**Location**: Phase 35 - needed for Phase 36 LSP capability advertisement

The local `ty_server/src/capabilities.rs` advertises:
```rust
signature_help_provider: Some(SignatureHelpOptions {
    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
    retrigger_characters: Some(vec![")".to_string()]),
    work_done_progress_options: WorkDoneProgressOptions::default(),
}),
```

**Recommended text to add to Phase 35 in the "Additional production tooling views" section:**
```rust
pub struct SignatureHelpConfig {
    pub trigger_characters: Vec<char>,  // e.g., ['(', ',']
    pub retrigger_characters: Vec<char>, // e.g., [')']
}
```

Sifr's trigger characters should be `(` for function calls and `,` for parameter position. The phase docs should specify this so Phase 36's LSP capability registration is deterministic.

---

### Gap E: Semantic Token Legend Must Be Defined as Sifr-Specific

**Location**: Phase 35 - needed for Phase 36 semantic token capability

The local `ty_server/src/capabilities.rs` uses:
```rust
legend: SemanticTokensLegend {
    token_types: ty_ide::SemanticTokenType::all()
        .iter()
        .map(|token_type| token_type.as_lsp_concept().into())
        .collect(),
    token_modifiers: ty_ide::SemanticTokenModifier::all_names()
        .iter()
        .map(|&s| s.into())
        .collect(),
},
```

Sifr's token categories differ from Python's. The phase docs must define Sifr's legend rather than copying Python semantics.

**Recommended text to add to Phase 35:**

In the "Additional production tooling views required for Phase 36" section, add:
```rust
pub struct SemanticTokenLegend {
    pub token_types: Vec<SifrSemanticTokenType>,
    pub token_modifiers: Vec<SifrSemanticTokenModifier>,
}

pub enum SifrSemanticTokenType {
    Keyword,       // sifr keywords (fn, let, mut, own, struct, enum, etc.)
    Type,         // user-defined types and stdlib types
    Function,     // function definitions
    Method,        // method definitions
    Variable,     // variable bindings
    Parameter,    // function parameters
    Property,     // struct/enum fields
    Module,       // module names
    Comment,      // comments
    String,       // string literals
    Number,       // numeric literals
    Operator,     // operators
    Attribute,    // decorator/attribute annotations
    Mutable,      // mutable binding indicator (sifr-specific)
    Ownership,    // ownership-sensitive location (sifr-specific)
    Deprecated,   // deprecated symbol
    Unsafe,      // unsafe operation marker (sifr-specific)
}

pub enum SifrSemanticTokenModifier {
    Declaration,
    Definition,
    Reference,
    Mutability,
    Ownership,
    Static,
    Abstract,
    Async,
    ReadOnly,
    Deprecated,
    Modification,
    Documentation,
}
```

This prevents Phase 36 from simply copying Python token categories.

---

## 2. Concrete Phase 36 Gaps for LSP Feature/Capability/Session/Testing

### Gap 1: Code Action Resolve Handler Is Conditionally Listed But Should Be Explicit

**Location**: Phase 36 "Required `sifr lsp --stdio` capabilities"

The current text says:
> `codeAction/resolve` when action edits are lazily computed

**Problem**: The conditional means it's not always required. But the local `ruff_server/src/server/api/requests/code_action.rs` shows that `fix_all` and `organize_imports` code actions use deferred edit resolution:
```rust
let (edit, data) = if snapshot
    .resolved_client_capabilities()
    .code_action_deferred_edit_resolution
{
    // The editor will request the edit in a `CodeActionsResolve` request
    (None, Some(...))
} else {
    (Some(resolve_edit_for_fix_all(...)?), None)
};
```

Sifr's code actions will likely have the same pattern - `SOURCE_FIX_ALL` and `SOURCE_ORGANIZE_IMPORTS` need deferred edit resolution.

**Recommended text change:**
```diff
- `codeAction/resolve` when action edits are lazily computed
+ `codeAction/resolve` - required because Sifr's code actions (diagnostic quick fixes,
+   source fix-all, source organize imports) support deferred edit resolution when the
+   client advertises `codeAction.deferredEditResolution`.
```

---

### Gap 2: Server-Initiated Requests Not Defined

**Location**: Phase 36 "LSP Server Contract"

The local `main_loop.rs` shows the server sending `SendRequest` actions to the client:
- `workspace/configuration` during initialization
- `window/showMessage` on errors
- `window/logMessage` for tracing

Phase 36's contract only defines client-to-server requests.

**Recommended text to add to Phase 36:**

In the "LSP Server Contract" section, add:
```rust
/// Server-initiated requests.
///
/// `sifr_lsp` may send the following requests to the client:
/// - `workspace/configuration`: query workspace-level and global settings.
///   The `ConfigurationController` in `sifr_lsp` handles responses and caches
///   settings for the current session.
/// - `window/showMessage`: display user-facing error messages (e.g., project
///   load failure, crash). This is reserved for non-recoverable conditions,
///   not routine diagnostics.
/// - `window/logMessage`: protocol trace logging at the info/debug level.
///
/// All server-initiated requests are tracked in the `RequestQueue::outgoing`
/// registry and must receive exactly one response. Response handlers are
/// registered at send time.
```

---

### Gap 3: File Watcher Notification Handler Not Listed as Required

**Location**: Phase 36 "Required notification handlers"

The current text lists:
- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didSave`
- `textDocument/didClose`

**Missing**: `workspace/didChangeWatchedFiles`

The local `ty_server/src/server/api/notifications.rs` has a `did_change_watched_files` module. The capability registry checks `FILE_WATCHER_SUPPORT` and `RELATIVE_FILE_WATCHER_SUPPORT`.

**Recommended text to add to Phase 36:**

In the "Required notification handlers" section, after `workspace/didChangeConfiguration`:
```rust
- `workspace/didChangeWatchedFiles` - handle external file changes (e.g.,
  git operations, linter auto-fix, build artifacts). The handler classifies
  each event as: project file changed (invalidate and re-diagnose), config
  file changed (reload settings and re-publish), or irrelevant (ignore).
  The handler must not block the main loop; file system queries are
  serialized through the snapshot model. File watcher registration is
  negotiated at initialization based on `workspace.didChangeWatchedFiles.dynamicRegistration`.
```

---

### Gap 4: Notebook Capability Must Be Explicitly Rejected

**Location**: Phase 36 "Explicitly unsupported protocol surfaces"

The current text says:
> Notebook synchronization is not part of Phase 36. The audited ty server supports notebooks for Python, but Sifr has no production notebook target; the Sifr LSP must not advertise notebook capabilities unless a later reviewed phase adds that product surface.

**Missing**: The specific capability bits that must be absent from `ServerCapabilities`.

**Recommended text to add:**
```rust
/// Notebook capability rejection.
///
/// Phase 36 must explicitly NOT set:
/// - `notebook_document_sync` in `ServerCapabilities` (must be `None`)
/// - Any `NotebookCellSelector` language registration
/// - `textDocument/didOpenNotebook` notification handler
/// - `textDocument/didChangeNotebook` notification handler
/// - `textDocument/didCloseNotebook` notification handler
///
/// The `DocumentStore` implementation must reject notebook URIs with a
/// protocol error (`InvalidRequest` with message "notebooks are not
/// supported") rather than silently ignoring them.
```

---

### Gap 5: LSP Protocol Test Harness Pattern Not Referenced

**Location**: Phase 36 "Verification Infrastructure" and "Local Ruff/ty LSP implementation audit inputs"

The local `ty_server/tests/e2e/main.rs` shows the `TestServerBuilder` pattern:
- Spawns server in a separate thread
- Uses `Connection::memory` for in-process JSON-RPC
- Has `await_request`, `await_response`, `await_notification` helpers
- Validates no unconsumed messages on drop

The phase docs reference the e2e tests but don't specify that `lsp_protocol_smoke.py` and `lsp_protocol_stress.py` must implement this pattern.

**Recommended text to add to Phase 36's `milestone_36_5` definition:**
```rust
/// Protocol test harness pattern.
///
/// `verification/tooling/lsp_protocol_smoke.py` and
/// `lsp_protocol_stress.py` must implement an in-process LSP test harness
/// modeled on `third_party/ruff/crates/ty_server/tests/e2e/main.rs`:
/// - Spawn `sifr lsp --stdio` as a subprocess with stdio transport
/// - Send JSON-RPC initialize/initialized handshake
/// - Track request IDs, expected responses, and pending notifications
/// - Validate that all server messages are consumed before test completion
/// - Implement request cancellation, timeout handling, and malformed JSON-RPC injection
///
/// The harness must support both stdio subprocess mode (production target) and
/// in-memory mode for local test iteration speed.
```

---

### Gap 6: Completion MRR Evaluation Pattern Not Defined

**Location**: Phase 36 "Verification Infrastructure"

The phase docs reference `completion_quality/` and `ty_completion_eval` but don't define the evaluation contract.

**Recommended text to add to Phase 36:**

In the `verification/tooling/` section, add:
```rust
- `verification/tooling/completion_quality/` - completion ranking evaluation
  inspired by `third_party/ruff/crates/ty_completion_eval/`:
  - `truth/` - ground truth completion tasks with `<CURSOR:module.symbol>`
    directives in `.sifr` files
  - `completion.toml` per task with `CompletionSettings` (e.g., auto_import)
  - Mean Reciprocal Rank (MRR) evaluation against ground truth
  - Threshold: default MRR minimum `0.001` (configurable per task)
  - Output: MRR summary, per-task rank, CSV results
  - Sifr-specific tasks covering: locals, functions, types, modules,
    imports, member access, auto-import candidates
```

---

### Gap 7: Unknown Options Warning Mechanism Not Defined

**Location**: Phase 36 "LSP Server Contract"

The local `ty_server/tests/e2e/initialize.rs` shows that the ty server sends `ShowMessage` notifications for unknown options:
```rust
insta::assert_json_snapshot!(show_message_params.message, @"Received unknown options during initialization...")
```

**Recommended text to add to Phase 36:**

In the `CommandRegistry` or a new `ConfigurationController` section:
```rust
/// Unknown options handling.
///
/// `sifr_lsp` sends `window/showMessage` (type: Warning) when the client
/// provides unknown initialization options or workspace configuration keys.
/// The message format is:
///   "Received unknown options during initialization: {json}"
///   "Received unknown options for workspace `{uri}`: {json}"
///
/// This follows ty's behavior from `ty_server/tests/e2e/initialize.rs`.
/// Unknown options do not prevent server initialization - they are logged
/// and the server continues with default values.
```

---

### Gap 8: Language Services Per Workspace Not Defined

**Location**: Phase 36 "LSP Server Contract"

The local `ty_server/tests/e2e/initialize.rs` shows `disableLanguageServices` per workspace:
```rust
.with_initialization_options(ClientOptions::default().with_disable_language_services(true))
```

For multi-workspace support in the future, this capability matters.

**Recommended text to add to Phase 36:**
```rust
/// Language services enable/disable per workspace.
///
/// Phase 36 v1 targets single-workspace use cases. Multi-workspace support
/// (where different workspace roots can have different diagnostic modes,
/// language services enabled/disabled, or settings) is deferred to Phase 36.x
/// or a later phase. The `SifrLspSession` must be designed so that this
/// extension is possible without breaking the Phase 36 session model.
```

---

## 3. Selection Range and Type Hierarchy: Correctly Sifr-Owned

**Selection Range**: The local `ty_ide/src/selection_range.rs` implementation is purely syntax-ancestry driven (`covering_node` + `ancestors().rev()`). The filter `should_include_in_selection` is a simple exclusion list. This is correctly implementable through `sifr_syntax` without any semantic dependency. The Phase 35 `SelectionRangeView` contract (`ranges_outer_to_inner`) is correct. [OK]

**Type Hierarchy**: The local `ty_ide/src/type_hierarchy.rs` uses `ty_python_semantic::SemanticModel` and `ty_python_semantic::type_hierarchy_*` functions with deep Python class hierarchy semantics (implicit `object` supertype, typeshed reachability, Python version conditional subtypes, dynamic classes via `type()`, named tuples, re-export filtering). This is the correct rejection - Sifr must not use Python hierarchy semantics.

The Phase 35 `TypeHierarchyQuery` trait is correctly abstract. Phase 36's implementation must answer from Sifr's type relationships (which may not map to Python-style inheritance). The contract correctly states: "if the language model has no meaningful hierarchy for a symbol, Phase 36 must return an empty/unsupported query result." [OK]

---

## 4. Missing Protocol Features

| Feature | Recommendation |
|---|---|
| `textDocument/implementation` | Add to Phase 36. If Sifr has interface/trait definitions with implementations, this is needed. If Sifr has no interface concept, skip. |
| `textDocument/linkedEditingRange` | Not needed. Sifr has no matching pair semantics. |
| `textDocument/prepareCallHierarchy` | Consider for Phase 36.x. Useful but subsumed by references for initial release. |
| `textDocument/documentLink` | Consider for Phase 36.x. Useful for import path links. |
| `textDocument/willSave` / `willSaveWaitUntil` | Not required. Optional client feature. |
| `workspace/didChangeWatchedFiles` | **Must add** to Phase 36 as a required notification handler. Already tracked in capability registry but not listed as required. |
| `completionItem/resolve` | **Must be explicit** if Sifr's completion uses lazy detail computation. Current conditional language is ambiguous. |
| `codeAction/resolve` | **Must be explicit** since Sifr's code actions will use deferred edit resolution for expensive edits. |
| `textDocument/hover` | Listed. [OK] |
| `textDocument/signatureHelp` | Listed. [OK] |

**Recommended text to add to Phase 36 for missing features:**

```rust
### Protocol Features Intentionally Deferred

The following LSP 3.17 features are not in the Phase 36 contract:
- `textDocument/implementation` - add if Sifr develops interface/trait semantics
- `textDocument/linkedEditingRange` - Sifr has no matching pair semantics
- `textDocument/prepareCallHierarchy` - consider for 36.x (references covers caller navigation for v1)
- `textDocument/documentLink` - consider for 36.x (import path links)
- `textDocument/willSave` / `textDocument/willSaveWaitUntil` - optional client feature
- Notebook synchronization - explicitly rejected per contract
```

---

## 5. Ambiguous/Overpromising Language

### Ambiguity 1: Selection Range Ordering

**Phase 35** says:
```rust
pub struct SelectionRangeView {
    pub ranges_outer_to_inner: Vec<SourceSpan>,  // EXPLICIT
}
```

**Phase 36** says:
> "selection ranges are syntax-aware, deterministic, outer-to-inner or inner-to-outer according to the final API contract"

**Fix**: Phase 36 should reference Phase 35's explicit ordering:
```diff
- "selection ranges are syntax-aware, deterministic, outer-to-inner or inner-to-outer according to the final API contract"
+ "selection ranges are syntax-aware, deterministic, and ordered outer-to-inner
+   (root-first, innermost-last). The order matches `SelectionRangeView::ranges_outer_to_inner`
+   from Phase 35's editor analysis boundary."
```

---

### Ambiguity 2: Semantic Token Categories

**Phase 36** says:
> "semantic tokens classify at least keyword, type, function, method, variable, parameter, property/field, module, comment, string, number, operator, decorator/attribute where applicable"

**Problem**: "where applicable" is ambiguous. Which categories are required vs. optional?

**Fix**:
```diff
- "semantic tokens classify at least keyword, type, function, method, variable, parameter, property/field, module, comment, string, number, operator, decorator/attribute where applicable, mutable binding, ownership-sensitive parameter convention, deprecated symbol, and unsafe/error-prone operation categories"
+ "semantic tokens classify the following categories:
+   REQUIRED (must be implemented): keyword, type, function, method, variable, parameter
+   REQUIRED where language has the concept: property (struct/enum fields), module, comment, string, number, operator, decorator/attribute
+   SIFR-SPECIFIC (required because Sifr has ownership semantics): mutable binding, ownership-sensitive location
+   OPTIONAL (implement when data is available): deprecated, unsafe/error-prone operation
+
+   The semantic token legend must be defined in Phase 35's editor analysis boundary
+   before Phase 36 LSP implementation begins."
```

---

### Ambiguity 3: Generated Rust Preview Latency

**Phase 36** says:
> "generated Rust preview uses compiler/codegen APIs and source maps; it must not reimplement lowering or codegen in the LSP or extension."

**Problem**: Codegen is expensive. The preview could take seconds on large modules.

**Fix**:
```diff
+ Generated Rust preview is a potentially expensive operation (codegen runs the
+ full compiler pipeline for the selected span or module). The implementation must:
+ - Use a background thread with cancellation support
+ - Implement a separate formatting lane (per `Scheduler::fmt_pool` in ty_server)
+ - Apply the budget from Phase 35: `lsp-generated-rust-preview median <= 750ms`
+ - Return partial results if codegen is interrupted by document changes
+ - Show progress for operations exceeding the interactive threshold
```

---

### Ambiguity 4: Explain Diagnostic Scope

**Phase 36** says:
> "explain diagnostic command backed by `sifr_diagnostics`"

**Problem**: "Explain" could mean anything from a simple message to a rich interactive help system.

**Fix**:
```diff
+ Explain diagnostic command returns structured data for a diagnostic id:
+   - Primary message and concise message
+   - Full diagnostic details (severity, code, docs URL)
+   - Related annotations with spans and messages
+   - Subdiagnostics
+   - Fix suggestions with applicability
+   - Links to docs
+
+ The command is `sifr.explainDiagnostic` and returns a JSON payload matching
+ `sifr_diagnostics` schema. The LSP extension displays this in a dedicated
+ help view. This is not a rich interactive drill-down - that belongs to a later
+ phase if user research validates the need.
```

---

## 6. Summary of Recommended Text Additions

### Phase 35 - Add to "Editor Analysis Boundary For Phase 36" section:

1. **PositionEncoding** in `SourceFileView`
2. **DocumentVersion in InvalidationReport** (`updated_documents: Vec<UpdatedDocumentInfo>`)
3. **Snapshot model documentation** in concurrency section
4. **SignatureHelpConfig** definition
5. **SemanticTokenLegend** with Sifr-specific token types and modifiers

### Phase 36 - Add to "LSP Server Contract" section:

1. **Code action resolve** - make explicit (not conditional)
2. **Server-initiated requests** section (workspace/configuration, showMessage, logMessage)
3. **File watcher notification handler** - add to required notifications
4. **Notebook capability rejection** - specific capability bits
5. **Protocol test harness pattern** - reference ty e2e tests
6. **Completion MRR evaluation pattern** - define truth/task structure
7. **Unknown options warning** mechanism
8. **Language services per workspace** - deferred but designed for extension

### Phase 36 - Fix ambiguous language:

1. Selection range ordering -> reference `ranges_outer_to_inner` from Phase 35
2. Semantic token categories -> required vs. optional classification
3. Generated Rust preview -> async, cancellation, budget
4. Explain diagnostic -> scope to structured JSON payload, not rich UI

---

**Bottom line**: The docs are close to ready but need these concrete additions before implementation begins. The gaps are not about missing features - the feature set is well thought out. They're about missing protocol-level details that would cause implementation ambiguity or missed requirements.
