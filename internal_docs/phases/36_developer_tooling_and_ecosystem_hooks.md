# Phase 36: Production Developer Tooling and Editor Ecosystem

status: completed

## Execution Status

- `milestone_36_1` is merged in PR #2129 at `82eaf50fea0ebbf7dba7a46749ee549fa11f4d73`.
- `milestone_36_2` is merged in PR #2130 at `cb08508f8db60109740fed15df5f3ccbd19c3482`.
- `milestone_36_3` is merged in PR #2131 at `5b2315e69aaead9269dd41a092e35b37c0968504`.
- `milestone_36_4` is merged in PR #2132 at `348a3ff7c67a8740c87c7e387428b721812134bb`.
- `milestone_36_5` is merged in PR #2133 at `a4a1297b1432598c98827ad98ba68293f33211c1`.
- `milestone_36_6` is merged in PR #2134 at `ac42f73464903b75b6ab3639d5ff766f31c44341`.
- `milestone_36_7` is merged in PR #2135 at `b519c597516bb8585d48211a7d7cadc264c7b90b`.
- `milestone_36_8` is merged in PR #2136 at `bb92e3f7577251f737bcb3a706ce45874daf6050`.
- Phase 36 is completed as of 2026-05-17.
- Final crate/module names are locked as `sifr_analysis`, `sifr_format`, `sifr_lint`, and `sifr_lsp`.
- VS Code extension boundary is locked to a separate `sifr-lang/sifr-vscode` repository, validated from the `editor_integrations/vscode` submodule, `SIFR_VSCODE_REPO`, or sibling `../sifr-vscode`.
- m36.1 contract artifacts live in `internal_docs/tooling_analysis.md`, `internal_docs/lsp_server.md`, `internal_docs/vscode_extension.md`, `internal_docs/editor_integrations.md`, `internal_docs/tooling_verification.md`, `verification/tooling/lsp_protocol_matrix.json`, and `verification/tooling/vscode_extension_contract.json`.
- Developer tooling contract checks are wired into `scripts/run_all_tests.sh`.
- m36.2 added `sifr_format`, `sifr_lint`, `sifr fmt [--check]`, `sifr lint`, generated `FMT`/`LINT` diagnostic docs, and formatter/rule-suppression contract checks. The follow-on production-grade formatter phase replaces the m36.2 whitespace foundation with the Ruff-backed `sifr fmt [OPTIONS] [FILES]...` surface while preserving the same CLI/analysis/LSP ownership boundary.
- m36.3 adds `sifr_analysis`, `AnalysisHost`, analysis snapshots, workspace symbol indexing, editor query API plumbing, completion ranking/evaluation infrastructure, and analysis split-brain/snapshot guardrails.
- m36.4 adds token-backed editor query behavior, generated-Rust preview handoff, code actions, parity snapshots, completion-quality fixtures, and the tooling parity runner.
- m36.5 adds `sifr_lsp`, `sifr lsp --stdio`, LSP 3.17 stdio protocol handling, document sync, push/pull diagnostics, editor query request adapters, workspace commands, protocol smoke/stress tests, and an enforced Phase 35 `lsp-query` budget case.
- m36.6 adds checked-in Neovim, Zed, Helix, and Emacs integration assets, a TextMate grammar, parser-token scope mapping, and `check_editor_assets.py` guardrails.
- m36.7 adds the `sifr-lang/sifr-vscode` extension repository scaffold, package metadata, language contribution, native LSP launcher, commands, generated-Rust/explain surfaces, test explorer, CI, `.vsix` packaging, and active cross-repo validation.

## Objective
Deliver the complete production-grade Sifr developer tooling stack for the current workspace/project model: editor-oriented analysis queries, diagnostics policy infrastructure, formatter and policy-rule surfaces, a native Rust LSP server launched through `sifr lsp --stdio`, a packageable VS Code extension, multi-editor syntax/integration assets, and parity gates proving tooling and CLI share one compiler brain.

Phase 36 is complete only when editors can provide Sifr diagnostics, navigation, completion, formatting, code actions, rename, semantic highlighting, generated-Rust preview, and test/developer commands without reimplementing parser, lowering, type-check, ownership, diagnostic, or codegen semantics.

Phase 36 is not an MVP phase. Every capability listed in this file is required for phase exit unless a later reviewed planning PR explicitly changes the contract before implementation reaches that milestone.

## Source Of Truth

This file is the authoritative implementation contract for Phase 36 until implementation creates supporting docs. `internal_docs/tooling_reuse_strategy.md` is the reviewed reuse audit and decision matrix for ty/Ruff/LSP infrastructure. Implementation PRs must add or update:

- `internal_docs/tooling_analysis.md`
- `internal_docs/lsp_server.md`
- `internal_docs/vscode_extension.md`
- `internal_docs/editor_integrations.md`
- `internal_docs/tooling_verification.md`

Those documents support this phase file; they must not introduce behavior that conflicts with this phase file or the reuse strategy unless a reviewed PR updates the relevant planning file first.

## Depends On

- Phase 35 is completed.
- `crates/sifr_syntax/` wraps the Sifr Ruff fork parser/AST/trivia/span substrate.
- `crates/sifr_frontend/` owns canonical parse/lower/type-check/diagnostics/project graph/query cache contracts.
- Phase 35 performance/query/cache/split-brain guardrails are enforced in local validation.
- Phase 35 exposes the symbol, source-map, type-display, formatting-trivia, diagnostic, and codegen-handoff data called out in its "Editor Analysis Boundary For Phase 36" section.
- `internal_docs/tooling_reuse_strategy.md` has been reviewed and is the source of truth for what is reused, adapted, referenced, or rejected from ty/Ruff infrastructure.
- Phase 27 runtime-safety and diagnostics invariants remain green.

## Feeds Into

- Package-aware external dependency intelligence after Phase 37 package management.
- Public documentation and editor installation guides in Phase 38.
- Stable release governance and marketplace publication mechanics in Phase 39.
- Future ecosystem integrations that consume the same LSP/editor assets without adding new semantics.

## Strict Non-Goals

- Using Ruff Server, ty, Pyright, or Python language-server semantics as Sifr's semantic authority.
- A custom public tsserver-style protocol. The external editor protocol is LSP 3.17.
- Package registry resolution, lockfile dependency intelligence, or third-party package auto-import across package registries. Phase 36 must be production-grade for the current workspace/project model; Phase 37 extends that model to package-managed dependencies.
- Notebook support unless a later reviewed language/product phase makes notebooks part of Sifr's production editor target.
- Marketplace account operations that require credentials or release approvals outside the repository. Phase 36 must produce packageable and publication-ready extension artifacts, manifests, docs, and checks.
- Fallback editor implementations that duplicate semantics when `sifr lsp` is missing. Extensions must fail with actionable setup errors.

## Architecture Ownership

`sifr_syntax` owns syntax substrate access to the Sifr Ruff fork. It is the only approved syntax/parser/trivia/token source for Sifr tooling. Editor packages may use generated syntax assets, but those assets must be derived from or validated against `sifr_syntax`/the Sifr Ruff fork and must not become a second parser.

`sifr_frontend` owns project loading, source maps, module graph state, parse/lower/type-check/diagnostics queries, invalidation, cache consistency, and compiler-owned analysis views.

`sifr_diagnostics` owns canonical diagnostic shape, diagnostic codes, renderer parity, structured suggestions, policy-rule metadata, suppression diagnostics, and hard-correctness vs policy-rule classification.

`sifr_format` or the final reviewed formatter module owns formatting over `sifr_syntax` token/trivia/source maps. Formatting must not lower, type-check, or derive semantic diagnostics.

`sifr_lint` or the final reviewed policy-rule module owns configurable policy rules over `sifr_frontend` and approved read-only HIR/syntax views. Hard correctness diagnostics stay in the compiler/frontend and are never suppressible.

`sifr_analysis` or `sifr_ide` owns editor-oriented semantic queries derived from `sifr_frontend`, `sifr_diagnostics`, `sifr_format`, and approved HIR/codegen views:

- diagnostics
- completion
- hover/type display
- signature help
- go-to-definition targets
- declaration targets where distinct from definition
- type-definition targets where meaningful
- find-references data
- prepare-rename and rename edits
- document symbols
- workspace symbols for the current workspace/project
- semantic tokens
- inlay hints
- document highlights
- folding ranges
- selection ranges
- type hierarchy where Sifr has meaningful type relationships
- code actions from Sifr diagnostic suggestions and safe refactors
- formatting and range-formatting requests
- generated Rust preview query/command data
- explain-diagnostic query/command data
- test discovery, test command metadata, and editor test explorer metadata where Sifr test fixtures are supported by the CLI

The final crate/module names must be chosen by `milestone_36_1` and used consistently. This phase document uses `sifr_analysis`, `sifr_format`, and `sifr_lint` as placeholder names.

`sifr_lsp` owns only the LSP protocol adapter:

- JSON-RPC/LSP handshake and message dispatch
- client capability negotiation
- document synchronization transport
- workspace configuration transport
- conversion between LSP positions/ranges/URIs and Sifr source maps
- conversion from `sifr_analysis` results to LSP response payloads
- cancellation, request scheduling, stale-version rejection, and protocol error behavior

`sifr_lsp` must use `lsp-server` and `lsp-types` as the direct LSP protocol foundation. These crates are generic protocol/data-type dependencies, carry no Python semantics, and are already used by the Sifr Ruff fork's `ty_server`. Audited `ty_server` session, document, request-queue, cancellation, diagnostics-publication, logging, and test-shell patterns may be adapted only when the final implementation is Sifr-owned and cleaned of Python semantic/project assumptions.

`sifr_lsp` must not own parser logic, type checking, HIR construction, symbol-table construction, diagnostic derivation, policy-rule derivation, formatting decisions, generated Rust decisions, or Sifr semantic rules.

Forbidden production dependencies for `sifr_lsp` and `sifr_analysis` include `ty_python_semantic`, Python module-resolution semantics from `ty_project`, Python environment discovery, Python diagnostic rules, and any `ruff_server`/`ty_server` path that answers semantic questions using Python rather than Sifr.

The reuse decisions in `internal_docs/tooling_reuse_strategy.md` are binding for Phase 36 implementation:

- `lsp-server` and `lsp-types` are `reuse-direct`.
- `ty_server` initialization, capability negotiation, request dispatch, document sync, range conversion, diagnostics lifecycle, and scheduler patterns are `adapt` or `adapt-with-review`.
- `ty_ide` query surface and completion-quality evaluation are `reference-only` or pattern adaptation; semantic query code is not reused directly.
- `ty_project` as a project database and `ty_python_semantic` as a semantic/rule engine are rejected production dependencies.
- Ruff parser/AST/trivia/text crates remain `reuse-direct` only through `sifr_syntax`.

Local Ruff/ty implementation audit inputs for this phase contract:

- `third_party/ruff/crates/ty_server/src/capabilities.rs` for resolved client capability flags, server capability construction, semantic-token legends, pull diagnostics, dynamic diagnostic registration, workspace folders, selection range, and type hierarchy capability wiring.
- `third_party/ruff/crates/ty_server/src/server/main_loop.rs`, `server/schedule.rs`, `server/api/traits.rs`, and `session/request_queue.rs` for the Sifr-owned LSP session shape: one-response request tracking, cancellation tokens, deferred/retried requests, sync handlers, background document handlers, latency-sensitive work, worker work, and a separate formatting lane.
- `third_party/ruff/crates/ty_server/src/document/text_document.rs` and `document/range.rs`, plus `third_party/ruff/crates/ruff_server/src/edit/text_document.rs`, for document versions, full and incremental sync application, multi-byte edit tests, line index updates, URI-carrying ranges, and client position-encoding conversion.
- `third_party/ruff/crates/ty_server/src/server/api/diagnostics.rs` for push/pull diagnostics lifecycle patterns, document-version tagging, related information capability checks, result ids, stale-result clearing, and settings diagnostics.
- `third_party/ruff/crates/ty_server/src/server/api/requests.rs` and `server/api/notifications.rs` for required handler coverage and one-file-per-protocol-method organization.
- `third_party/ruff/crates/ty_ide/src/lib.rs`, `selection_range.rs`, and `type_hierarchy.rs` for the editor-query inventory and the distinction between generic syntax queries worth adapting and Python semantic hierarchy logic that must not be reused.
- `third_party/ruff/crates/ruff_server/src/server/api/requests/code_action.rs`, `ruff_server/src/fix.rs`, and `ruff_server/src/format.rs` for deferred code-action resolution, workspace edit construction, fix-all/organize-import structure, and format/range-format edit behavior.
- `third_party/ruff/crates/ty_server/tests/e2e/` and `third_party/ruff/crates/ty_completion_eval/` for protocol snapshot coverage and completion quality evaluation patterns.

`crates/sifr` owns the CLI command surface. Phase 36 adds:

```bash
sifr lsp --stdio
sifr fmt [OPTIONS] [FILES]...
sifr lint <path-or-project>
```

The exact CLI spelling may change during `milestone_36_1` only if the final reviewed command set preserves the same capabilities. `--stdio` is the required LSP transport for phase exit.

VS Code extension ownership:

- language id: `sifr`
- file extensions: `.sifr`
- grammar/filetype registration and language configuration
- LSP launcher and settings UI
- commands and test explorer integration that call `sifr lsp`, `sifr fmt`, `sifr lint`, `sifr check`, `sifr test`, or generated-Rust preview commands
- no type checker, parser, HIR traversal, diagnostic derivation, formatter, linter, or codegen logic inside extension TypeScript/JavaScript

## Editor Query Contract

Phase 36 must define and implement the editor query API below. Names may change during implementation only if the final reviewed API preserves the same capability and ownership boundary.

```rust
// Target crate: sifr_analysis, final name decided in milestone_36_1.

pub struct AnalysisHost {
    pub fn open_project(root: ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>>;
    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>>;
    pub fn update_document(
        &mut self,
        file: FileId,
        version: DocumentVersion,
        text: SourceText,
    ) -> Result<InvalidationReport, Vec<RenderedDiagnostic>>;

    pub fn diagnostics(&mut self, file: FileId) -> QueryResult<Vec<RenderedDiagnostic>>;
    pub fn workspace_diagnostics(&mut self) -> QueryResult<Vec<FileDiagnostics>>;
    pub fn completion(&mut self, file: FileId, position: TextPosition) -> QueryResult<CompletionItems>;
    pub fn hover(&mut self, file: FileId, position: TextPosition) -> QueryResult<Option<HoverInfo>>;
    pub fn signature_help(&mut self, file: FileId, position: TextPosition) -> QueryResult<Option<SignatureHelp>>;
    pub fn definition(&mut self, file: FileId, position: TextPosition) -> QueryResult<Vec<Location>>;
    pub fn declaration(&mut self, file: FileId, position: TextPosition) -> QueryResult<Vec<Location>>;
    pub fn type_definition(&mut self, file: FileId, position: TextPosition) -> QueryResult<Vec<Location>>;
    pub fn references(&mut self, file: FileId, position: TextPosition) -> QueryResult<Vec<Location>>;
    pub fn prepare_rename(&mut self, file: FileId, position: TextPosition) -> QueryResult<Option<RenameTarget>>;
    pub fn rename(&mut self, file: FileId, position: TextPosition, new_name: SymbolName) -> QueryResult<WorkspaceEdit>;
    pub fn document_symbols(&mut self, file: FileId) -> QueryResult<Vec<DocumentSymbol>>;
    pub fn workspace_symbols(&mut self, query: SymbolQuery) -> QueryResult<Vec<WorkspaceSymbol>>;
    pub fn semantic_tokens(&mut self, file: FileId, range: Option<TextRange>) -> QueryResult<Vec<SemanticToken>>;
    pub fn inlay_hints(&mut self, file: FileId, range: Option<TextRange>) -> QueryResult<Vec<InlayHint>>;
    pub fn document_highlights(&mut self, file: FileId, position: TextPosition) -> QueryResult<Vec<DocumentHighlight>>;
    pub fn folding_ranges(&mut self, file: FileId) -> QueryResult<Vec<FoldingRange>>;
    pub fn selection_ranges(&mut self, file: FileId, positions: Vec<TextPosition>) -> QueryResult<Vec<SelectionRange>>;
    pub fn prepare_type_hierarchy(&mut self, file: FileId, position: TextPosition) -> QueryResult<Option<TypeHierarchyItem>>;
    pub fn type_hierarchy_supertypes(&mut self, item: TypeHierarchyItemId) -> QueryResult<Vec<TypeHierarchyItem>>;
    pub fn type_hierarchy_subtypes(&mut self, item: TypeHierarchyItemId) -> QueryResult<Vec<TypeHierarchyItem>>;
    pub fn code_actions(&mut self, file: FileId, range: TextRange, context: CodeActionContext) -> QueryResult<Vec<CodeAction>>;
    pub fn format_document(&mut self, file: FileId, options: FormatOptions) -> QueryResult<TextEdits>;
    pub fn format_range(&mut self, file: FileId, range: TextRange, options: FormatOptions) -> QueryResult<TextEdits>;
    pub fn generated_rust_preview(&mut self, file: FileId, range: Option<TextRange>) -> QueryResult<GeneratedRustPreview>;
    pub fn explain_diagnostic(&mut self, diagnostic: DiagnosticId) -> QueryResult<DiagnosticExplanation>;
    pub fn discover_tests(&mut self) -> QueryResult<Vec<TestItem>>;
    pub fn test_command(&mut self, test: TestItemId) -> QueryResult<TestCommand>;
}
```

Required production semantics:

- Completion covers locals, functions, types, modules, imports, stdlib items, fields/methods when type information is available, enum/union variants, contextual keywords, snippets only where semantically valid, and current-workspace auto-import candidates. Package-registry auto-import lands after Phase 37.
- Hover shows inferred Sifr type, symbol kind, mutability/ownership-relevant facts where applicable, and docs when doc comments are available.
- Signature help covers Sifr functions, methods, constructors, generic parameters, and active parameter selection.
- Definition/declaration/type-definition cover local variables, functions, methods, classes/types, modules, imported symbols, current-workspace symbols, and stdlib symbols when source spans are available.
- References and rename are workspace-wide for the current workspace/project graph and must reject unsafe or ambiguous rename targets.
- Document symbols and workspace symbols are deterministic and stable across equivalent inputs.
- Semantic tokens classify the following categories:
  - required for every Sifr workspace: keyword, type, function, method, variable, and parameter
  - required where the source contains the language concept: property/field, module, comment, string, number, operator, decorator/attribute
  - Sifr-specific required categories: mutable binding and ownership-sensitive location
  - optional until the compiler exposes the facts: deprecated symbol and unsafe/error-prone operation
  The semantic token legend is Sifr-owned and must be locked in `milestone_36_1` before `sifr_lsp` advertises semantic-token capabilities.
- Inlay hints include inferred variable types, parameter names, generic type parameters where useful, and ownership/borrow hints only where the type checker can state them precisely.
- Selection ranges are syntax-aware, deterministic, ordered outer-to-inner to match Phase 35 `SelectionRangeView::ranges_outer_to_inner`, and backed by `sifr_syntax`/Phase 35 syntax-ancestry views rather than ad hoc text scanning.
- Type hierarchy supports prepare, supertypes, and subtypes where Sifr has meaningful class/trait/interface/type-extension relationships; it must return precise empty results for symbols without hierarchy rather than borrowing Python `object`/class assumptions.
- Code actions expose Sifr diagnostic suggestions, safe import insertion, suppression insertion only for suppressible policy rules, organize imports, and safe rename/format-related fixes.
- Formatting preserves comments/trivia, is idempotent, and matches `sifr fmt --check`.
- Generated Rust preview uses compiler/codegen APIs and source maps; it must not reimplement lowering or codegen in the LSP or extension.
- Generated Rust preview is a potentially expensive background command. It must run on a cancellable background path, report progress when it exceeds the interactive threshold, obey the `lsp-generated-rust-preview` budget, and return a deterministic cancellation/content-modified response instead of partial generated code if a document change invalidates the request.
- Explain diagnostic returns structured data for a diagnostic id: primary and concise messages, severity, code, docs URL, related annotations/spans, subdiagnostics, structured fix suggestions, and applicability. The Sifr-owned command name is locked in `milestone_36_1`; the payload must preserve the `sifr_diagnostics` schema rather than inventing an extension-only explanation format.
- Test discovery and editor test explorer metadata must use CLI/test-runner metadata when available and must fail closed when a project has no test surface rather than guessing from Python semantics.

Anti-split-brain rules:

- `sifr_analysis` may call `sifr_frontend`, `sifr_diagnostics`, `sifr_format`, `sifr_lint`, and approved HIR/codegen read-only views.
- `sifr_lsp` handlers call `sifr_analysis`; they do not traverse HIR directly for semantic answers.
- VS Code/Neovim/Zed/Helix/Emacs packages call LSP, CLI commands, or syntax-highlighting assets; they do not implement Sifr semantics.
- Any new tooling path that parses, lowers, type-checks, formats through a separate parser, codegens through an editor-specific path, or derives semantic diagnostics outside approved crates fails the split-brain guardrail.

## LSP Server Contract

The external protocol target is LSP 3.17 over stdio.

The LSP implementation must be a Sifr-owned protocol shell that adapts the generic architecture of the audited `ty_server`/`ruff_server` code, not a new semantic implementation. The required internal layers are:

- `CapabilityRegistry`: resolves client capabilities once at initialization and records position encoding, workspace configuration support, dynamic diagnostic registration, pull diagnostics, related information, semantic-token multiline support, signature label-offset support, hierarchical document-symbol support, completion label-details/documentation support, work-done progress, file watchers, and code-action resolve support.
- `DocumentStore`: tracks open `.sifr` documents by URI, document version, language id, source text, line index, canonical `FileId`, full/incremental edit application, and UTF-8/UTF-16/UTF-32 conversion through Sifr source maps.
- `SifrLspSession`: owns `AnalysisHost`, open-document overrides, workspace folders, settings, diagnostics mode, request queues, suspended workspace diagnostics, stale-result rejection state, and client command/settings metadata.
- `RequestQueue`: tracks incoming and outgoing request ids, method names, start times, cancellation tokens, response handlers, and exactly-one-response completion. Responses for canceled or superseded requests must be ignored deterministically.
- `Scheduler`: separates sync state mutation from background document queries, workspace queries, latency-sensitive requests, and formatting/code-action work. Completion, hover, signature help, definition, and diagnostics refresh must not be starved by workspace diagnostics.
- `SnapshotLayer`: creates coherent analysis snapshots per request so document changes can cancel/retry or reject stale work without corrupting frontend caches.
- `ConversionLayer`: owns all LSP-to-Sifr URI/range/position/location/diagnostic/edit conversions and rejects invalid or out-of-date positions with protocol errors rather than panics.
- `DiagnosticsController`: handles push diagnostics, pull document diagnostics, pull workspace diagnostics, result ids, diagnostic clearing, document-version tagging, related information/tags, and dynamic registration for `off`, `open-files`, and `workspace` modes.
- `CommandRegistry`: owns restart, logs, explain diagnostic, generated Rust preview, check, test, and any future commands. Command names are Sifr-owned and must be versioned/documented.
- `ProtocolTestHarness`: launches `sifr lsp --stdio`, drives JSON-RPC messages, records deterministic snapshots, and covers every handler family before VS Code or other editor adapters depend on it.

Each LSP request handler must live in a module named after the LSP method or a clearly mapped Sifr command. Handlers may share helpers, but the user-facing request coverage must be discoverable from file names and test names.

Required `sifr lsp --stdio` capabilities for phase exit:

- `initialize`
- `initialized`
- `shutdown`
- `exit`
- `workspace/didChangeConfiguration`
- `workspace/didChangeWatchedFiles`
- `workspace/symbol`
- `workspace/executeCommand`
- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didSave`
- `textDocument/didClose`
- `textDocument/publishDiagnostics`
- `textDocument/diagnostic` where the client supports pull diagnostics
- `workspace/diagnostic` where the client supports pull diagnostics
- `textDocument/completion`
- `completionItem/resolve`
- `textDocument/hover`
- `textDocument/signatureHelp`
- `textDocument/definition`
- `textDocument/declaration`
- `textDocument/typeDefinition`
- `textDocument/references`
- `textDocument/prepareRename`
- `textDocument/rename`
- `textDocument/documentSymbol`
- `workspace/symbol`
- `textDocument/semanticTokens/full`
- `textDocument/semanticTokens/range`
- `textDocument/inlayHint`
- `textDocument/documentHighlight`
- `textDocument/foldingRange`
- `textDocument/selectionRange`
- `textDocument/prepareTypeHierarchy`
- `typeHierarchy/supertypes`
- `typeHierarchy/subtypes`
- `textDocument/codeAction`
- `codeAction/resolve`
- `textDocument/formatting`
- `textDocument/rangeFormatting`

Required workspace commands:

- restart language server
- show server logs
- explain diagnostic
- show generated Rust for current file or selection
- run Sifr check for current workspace
- run Sifr tests where the CLI exposes test metadata

Required server-initiated client requests and notifications:

- `workspace/configuration` to query workspace/global Sifr settings when the client supports workspace configuration.
- `window/showMessage` for user-facing warnings or non-recoverable setup errors, including unknown initialization options and missing/invalid project configuration. Routine compiler diagnostics must remain diagnostics, not modal messages.
- `window/logMessage` for protocol trace and server lifecycle logging.
- Optional dynamic `client/registerCapability` requests for diagnostics and file watching when the client advertises support.

Unknown initialization options and workspace configuration keys must produce deterministic warning messages and logs while continuing with default values. Unknown options must not prevent initialization unless they make the requested workspace impossible to load safely.

Document sync model:

- Phase 36 supports both full-document and incremental sync.
- Incremental sync must be proven against partial insert/delete/replace edits, multi-byte characters, UTF-8/UTF-16/UTF-32 client position encodings, stale versions, and parse/type-check recovery.
- Each LSP diagnostic publication includes the latest known document version when the client supplies one.
- LSP handlers must ignore stale query results for superseded document versions.
- Open editor buffers override on-disk content for analysis until closed or saved according to LSP sync semantics.

Cancellation and concurrency:

- Document updates are serialized so no query observes a partially applied edit.
- Latency-sensitive requests such as completion, hover, signature help, and definition must not be blocked behind full workspace diagnostics when a valid snapshot exists.
- Cancellation may abort pending editor queries, but it must not publish partial diagnostics or corrupt frontend/analysis cache state.
- The LSP session must use explicit snapshots or equivalent revision discipline so each request sees a coherent source graph.
- Internal errors are reported through LSP error responses and compiler diagnostics according to the Phase 27 exit-code/diagnostic contract where applicable.
- Request scheduling, cancellation, and stale-version handling must have negative protocol tests.

### LSP Protocol Versioning Policy

- The target protocol version is LSP 3.17 at phase exit and must be recorded in `internal_docs/lsp_server.md` with the exact `lsp-types` crate version pinned in `Cargo.lock`.
- Upstream `lsp-types` version bumps require a reviewed PR that documents which new capabilities are adopted, which are deferred, how compatibility with older LSP clients is preserved, and which protocol matrix entries changed.
- New LSP capabilities are adopted only when Sifr's semantic model has a meaningful answer, the implementation does not require Python/ty semantics, and the capability has positive and negative protocol test coverage.
- A capability must not be advertised in server capabilities unless it passes local protocol smoke tests.
- Deferring a new LSP capability requires an explicit rationale in `internal_docs/lsp_server.md` or the protocol matrix, not silent omission.

Explicitly unsupported protocol surfaces:

- Notebook synchronization is not part of Phase 36. The audited ty server supports notebooks for Python, but Sifr has no production notebook target; the Sifr LSP must not advertise notebook capabilities unless a later reviewed phase adds that product surface.
- Server capabilities must leave `notebook_document_sync` unset, must not register any notebook cell selectors, and must not expose notebook open/change/close handlers. Notebook URIs are rejected with a deterministic protocol error instead of being silently analyzed as regular files.
- Python-specific import, environment, stub, interpreter, or settings protocol behavior is forbidden.
- A custom public protocol alongside LSP is forbidden.

Protocol features intentionally outside Phase 36 unless `milestone_36_1` proves they are necessary for current Sifr semantics:

- `textDocument/implementation`; add only if the Sifr type model exposes interface/trait-to-implementation relationships during Phase 36.
- `textDocument/linkedEditingRange`; Sifr has no required paired-edit surface yet.
- `textDocument/prepareCallHierarchy`; references and workspace symbols cover the required navigation contract for this phase.
- `textDocument/documentLink`; import path links may be added after package/module UX stabilizes.
- `textDocument/willSave` and `textDocument/willSaveWaitUntil`; document formatting uses explicit formatting/range-formatting and editor save hooks configured by clients.

Phase 36 v1 targets one active Sifr workspace/project per server session. The session model must still keep workspace folders and settings isolated enough that later multi-root support can add per-workspace diagnostics modes and language-services enablement without rewriting document storage, request dispatch, or `AnalysisHost` ownership.

## Diagnostics, Rules, Suppressions, And Exclusions

Sifr keeps `crates/sifr_diagnostics/` as the canonical diagnostic model. Phase 36 may adapt ty/Ruff UX concepts from `internal_docs/tooling_reuse_strategy.md`, but it must not replace Sifr diagnostic codes, rendered JSON schema, renderer parity, child note/help behavior, structured suggestions, or Phase 27 exit-code contracts with Ruff/ty diagnostic types.

Phase 36 must implement and document this diagnostic split:

- hard correctness diagnostics are not suppressible and cannot be downgraded: parse errors, soundness-critical type errors, ownership/move/borrow errors, `Result`/`Option` safety errors, runtime-panic-prevention errors, and workspace/import errors that would make compilation ambiguous or unsound
- policy rules are configurable when they do not affect Sifr's core guarantee: unused code/imports, unreachable-code warnings, migration advisories, style-adjacent static analysis, optional strictness checks, and tooling-quality advisories

### Diagnostic Rule Lifecycle Policy

- Every Sifr diagnostic rule id is stable once it ships in a release. Deprecated rules must retain their id for backward-compatible suppression comments, emit deprecation status in rule metadata, and document the replacement rule id when one exists.
- New policy rules added in patch releases must be `off` by default or explicitly marked experimental. New policy rules added in minor releases may be `warn` by default if they do not conflict with existing accepted code behavior.
- Experimental rules are allowed only with an explicit `experimental` status label and documentation URL. Experimental rules may be removed without a deprecation window if they prove unmaintainable.
- `# sifr: ignore[deprecated-rule-id]` must continue to work for at least two minor releases after deprecation and must produce an actionable diagnostic or metadata hint that names the replacement rule when one exists.
- Rule metadata (`id`, summary, docs URL, default level, status, and source location) belongs in `sifr_diagnostics` or a Sifr-owned rule registry. Rule metadata must not be sourced from `ty_python_semantic` or any Python semantic dependency.

Rules and suppression requirements:

- Adopt ty's `ignore`/`warn`/`error` rule-level product concept only for Sifr-owned policy rules.
- Rule metadata belongs in Sifr-owned diagnostics or a Sifr-owned policy-rule registry, not in `ty_python_semantic`.
- Suppression syntax is Sifr-specific: `# sifr: ignore[rule-id]`.
- Bare blanket suppressions are not allowed.
- Unknown suppression rule ids and unused suppression comments produce deterministic diagnostics.
- Python `type: ignore` comments must not suppress Sifr diagnostics by default. Compatibility may be added only for an explicit Sifr-prefixed form such as `type: ignore[sifr:rule-id]`.
- LSP code actions may insert suppressions only for suppressible policy rules and must never offer suppression for hard correctness diagnostics.

Exclusion requirements:

- Include/exclude behavior adapts ty's product model: project include roots, exclude globs, default ignored directories, respect for `.gitignore`/`.ignore`, and explicit CLI targets overriding excludes.
- Generic glob matching may be extracted from `ty_project` only if moved behind a Sifr-owned API and cleaned of Python project assumptions.
- Exclusions affect project discovery scope; they must not silently change semantics for files explicitly passed to `sifr check`, `sifr build`, `sifr fmt`, `sifr lint`, or `sifr lsp`.

LSP diagnostics mode:

- Phase 36 supports `off`, `open-files`, and `workspace` diagnostics modes for editor publication.
- `off` only disables editor publication; it does not change CLI compiler behavior.
- `open-files` mode publishes diagnostics for open documents and their directly required context.
- `workspace` mode publishes project diagnostics for the current workspace/project graph within Phase 35 performance budgets.
- Push and pull diagnostics must agree on codes, severity, ranges, related information, tags, and structured suggestions.

## Formatting And Policy-Rule Contract

Formatting is part of Phase 36.

Formatter requirements:

- `sifr fmt --check` exits nonzero on non-idempotent formatting drift and prints deterministic diagnostics.
- `sifr fmt` produces stable, idempotent output.
- LSP document/range formatting returns text edits equivalent to `sifr fmt`.
- Formatting preserves comments, meaningful blank lines, string contents, source spans needed by diagnostics, and Sifr-specific parameter-convention syntax.
- Formatting tests include round-trip parse checks and drift checks against `sifr_syntax` fixtures.

Policy-rule requirements:

- `sifr lint` runs suppressible policy diagnostics without changing hard compiler diagnostics.
- Policy-rule configuration uses Sifr-owned rule ids and deterministic severity resolution.
- Policy diagnostics share the canonical `sifr_diagnostics` shape and renderer behavior.
- LSP diagnostics, code actions, and CLI lint output use the same rule engine and suppression/exclusion behavior.

## VS Code Extension Contract

Phase 36 must implement and package the VS Code extension. A separate repository such as `sifr-lang/sifr-vscode` is acceptable only if the phase PR records the repository boundary and the validation commands are still reproducible from this repository through the `editor_integrations/vscode` submodule or from an explicitly configured checkout.

The concrete execution checklist is `issues/phase36-vscode-extension-production-execution.md`. The default recommendation is a separate `sifr-lang/sifr-vscode` repository; `milestone_36_1` may choose an in-repo extension only with reviewed rationale and equivalent validation/release-boundary guarantees. This is a required Phase 36 milestone, not an ad hoc phase after Phase 36.

The extension owns:

- `package.json` language contribution for `.sifr`
- TextMate grammar and/or Tree-sitter-backed grammar contribution for basic syntax highlighting before the LSP starts
- language configuration: comments, brackets, indentation, auto-closing pairs
- LSP client launcher for `sifr lsp --stdio`
- user settings under `sifr.*`
- commands: restart server, show server logs, locate Sifr binary, run check, run tests, format document, show generated Rust, explain diagnostic
- VS Code Test Explorer integration backed by Sifr test discovery and CLI test commands
- `.vsix` packaging and extension test workflow

The extension must not own:

- type checking
- ownership/move analysis
- diagnostics derivation
- Sifr symbol analysis
- formatter logic
- linter/policy-rule logic
- generated Rust decisions

Binary discovery:

- default command: `sifr`
- default args: `["lsp", "--stdio"]`
- setting: `sifr.lsp.path` overrides the binary path
- setting: `sifr.lsp.trace.server` controls protocol tracing
- extension startup must fail with an actionable message if no Sifr binary is found
- the extension must not silently fall back to Python tooling

Syntax highlighting strategy:

- Basic highlighting must not depend on the LSP being ready.
- The grammar may be TextMate first for VS Code velocity, Tree-sitter first for multi-editor reuse, or both if generated from the same source.
- The grammar/token queries must be generated from or validated against `sifr_syntax`/the Sifr Ruff fork tokenization fixtures. Manually authored grammar rules are allowed only if a checked-in validation test catches drift from parser tokenization.
- LSP semantic tokens layer meaning-aware highlighting on top of basic syntax.

Publication readiness:

- Phase 36 must produce a packageable extension artifact and publication checklist.
- Actual marketplace upload may be performed by Phase 39 release governance if credentials or release approvals are required.

### Extension Versioning Covenant

- The VS Code extension version must either be explicitly coupled to the Sifr compiler version or have a documented version-independence policy before `milestone_36_7` closes.
- When the main `sifr-lang/sifr` repository releases version `X.Y.Z`, the extension must either release a corresponding compatible version or document a supported Sifr version range in extension metadata and release notes.
- Extension releases are gated on the extension contract check passing, `sifr lsp --stdio` smoke tests passing with the target Sifr version, and the `check_vscode_extension.py` build/test/package sequence passing.
- Phase 39 owns marketplace publication governance, but this phase must establish the versioning covenant and validation coupling between the compiler, LSP, and extension.
- Extension validation must not silently skip when the main Sifr version advances.

## Multi-Editor Integration Contract

Phase 36 must deliver checked-in editor integration assets or contribution-ready docs for:

- Neovim: LSP config, filetype detection, and Tree-sitter/TextMate strategy notes.
- Zed: language extension metadata or contribution-ready config using `sifr lsp --stdio`.
- Helix: language configuration using `sifr lsp --stdio` and syntax asset instructions.
- Emacs: LSP client configuration and filetype mode guidance.

These integrations must delegate semantics to `sifr lsp --stdio`. Syntax/highlighting assets must be validated against `sifr_syntax` fixtures or generated from the same source-of-truth grammar.

## Verification Infrastructure

Phase 36 creates and owns `verification/tooling/`.

Required files:

- `verification/tooling/parity_manifest.json` - source of truth for diagnostics and editor-query parity cases.
- `verification/tooling/run_tooling_parity.py` - compares CLI/frontend/analysis/LSP results for manifest entries.
- `verification/tooling/lsp_protocol_smoke.py` - launches `sifr lsp --stdio`, performs initialize/open/change/query/shutdown, and validates JSON-RPC behavior.
- `verification/tooling/lsp_protocol_stress.py` - validates cancellation, stale versions, incremental sync, request interleaving, malformed JSON-RPC, and workspace diagnostics behavior.
- `verification/tooling/lsp_protocol_matrix.json` - checked-in request/notification/capability matrix covering every required LSP method, command, setting, diagnostic mode, and unsupported protocol surface.
- `verification/tooling/check_analysis_snapshot_coherence.py` - verifies `sifr_analysis` snapshots cannot publish stale `sifr_frontend` query results, reject stale revision publications, and preserve the `InvalidationReport` boundary between `sifr_frontend` and `sifr_analysis`.
- `verification/tooling/check_lsp_split_brain.py` - verifies LSP handlers do not import or traverse forbidden semantic internals directly, including `ty_python_semantic`, `ty_project` Python semantics, `ruff_server` diagnostics as Sifr behavior, Python module-resolution paths, and direct HIR traversal for semantic answers.
- `verification/tooling/check_tooling_dependency_boundaries.py` - verifies forbidden ty/Ruff/Python semantic dependencies are not introduced.
- `verification/tooling/check_formatter_contract.py` - verifies idempotence, range-formatting, parser round trips, and formatter/LSP equivalence.
- `verification/tooling/check_rule_suppression_contract.py` - verifies hard-vs-policy diagnostics, suppression, unknown suppression, unused suppression, severity config, and exclusions.
- `verification/tooling/check_editor_assets.py` - verifies syntax assets, extension metadata, editor configs, and drift checks.
- `verification/tooling/check_vscode_extension_contract.py` - main-repo cross-repo contract validator that reads `vscode_extension_contract.json`, locates `sifr-lang/sifr-vscode` through `editor_integrations/vscode`, `SIFR_VSCODE_REPO`, or a sibling checkout, fails if the extension repo is missing, and validates language id, extension id, launch command, required settings, required commands, package/test commands, and forbidden semantics-bearing extension behavior.
- `verification/tooling/check_vscode_extension.py` - verifies extension build/test/package behavior for the located extension repo.
- `verification/tooling/completion_quality/` - completion ranking/evaluation fixtures inspired by `ty_completion_eval`, including `truth/` Sifr fixtures with cursor/expected-answer directives, per-task completion settings, mean reciprocal rank output, per-task rank CSV output, and thresholds for locals, functions, types, modules, imports, member access, and current-workspace auto-import candidates.
- `verification/tooling/editor_query_snapshots/` - checked-in deterministic expected results for editor queries.
- `verification/tooling/vscode_extension_contract.json` - extension settings, language id, command, and repository-boundary contract.

The LSP protocol matrix must include positive and negative coverage for:

- initialize/capabilities, dynamic diagnostic registration, workspace folders, workspace configuration, file watcher registration, and unsupported client capability combinations
- open/change/save/close with full sync, incremental sync, multi-byte edits, stale versions, invalid ranges, and closed-document behavior
- push diagnostics, pull document diagnostics, pull workspace diagnostics, diagnostics clearing, `off`/`open-files`/`workspace` modes, result ids, related information, tags, and document-version tagging
- completion, completion resolve, hover, signature help, definition, declaration, type definition, references, prepare rename, rename, document symbols, workspace symbols, semantic tokens full/range, inlay hints, document highlights, folding ranges, selection ranges, prepare type hierarchy, type-hierarchy supertypes/subtypes, code action, code action resolve, formatting, range formatting, and execute command
- cancellation of queued and running requests, retry or content-modified behavior after source changes, response suppression for canceled requests, malformed JSON-RPC, unsupported methods, shutdown/exit ordering, server logging, and command errors
- explicit non-advertisement of notebook capabilities and rejection of Python-specific settings/import/environment behavior

The protocol smoke/stress harness must be modeled on the audited `ty_server/tests/e2e` shape while targeting the production transport:

- launch `sifr lsp --stdio` as a subprocess for authoritative protocol tests
- provide an in-process or memory-transport mode only when it uses the same handlers and exists to speed local iteration
- send initialize/initialized/shutdown/exit sequences with tracked request ids
- expose helpers to await expected responses, requests, and notifications
- validate that all expected server messages are consumed before test completion
- inject request cancellation, timeouts, malformed JSON-RPC, invalid ranges, stale document versions, and unsupported methods

Phase 36 extends Phase 35 `verification/performance/` with protocol-level `lsp-query` benchmark cases:

- `lsp-cold-start`: process spawn to initialized response
- `lsp-did-open-diagnostics`: open document to first diagnostics publication
- `lsp-did-change-diagnostics`: full and incremental document changes to refreshed diagnostics
- `lsp-workspace-diagnostics`: workspace diagnostics publication or pull response
- `lsp-completion`: request/response at representative local, member, import, and auto-import positions
- `lsp-hover`: request/response for local, function, type, ownership-sensitive parameter, and imported symbol
- `lsp-signature-help`: request/response inside calls and generic invocations
- `lsp-definition`: request/response for local and imported definitions
- `lsp-references`: workspace reference query
- `lsp-rename`: prepare and workspace edit generation
- `lsp-semantic-tokens`: full and range semantic token requests
- `lsp-inlay-hints`: representative file/range request
- `lsp-selection-range`: nested syntax selection request
- `lsp-type-hierarchy`: prepare/supertypes/subtypes request where Sifr hierarchy semantics exist
- `lsp-code-actions`: diagnostic and organize-import actions
- `lsp-formatting`: document and range formatting requests
- `lsp-generated-rust-preview`: preview command latency

Default interactive budgets unless Phase 35 `budgets.json` records stricter values with rationale:

- cold start median <= 1000ms on local baseline hardware
- diagnostics after `didOpen` median <= 500ms for representative files
- diagnostics after `didChange` median <= 250ms for representative files
- workspace diagnostics p95 <= 2000ms for representative projects
- completion median <= 200ms
- hover median <= 100ms
- signature help median <= 150ms
- definition/declaration/type-definition median <= 150ms
- references median <= 500ms for representative projects
- rename prepare median <= 150ms; rename edit generation median <= 750ms
- semantic tokens median <= 250ms
- inlay hints median <= 250ms
- selection range median <= 100ms
- type hierarchy median <= 250ms for representative current-workspace hierarchy queries
- code actions median <= 250ms
- formatting median <= 500ms for representative files
- generated Rust preview median <= 750ms for representative selections

These are phase-start defaults. Final budgets must be derived from checked-in baselines and recorded in `verification/performance/budgets.json`.

## Milestone Sequencing

Implementation must execute milestones in order unless a later reviewed PR updates this file with rationale. The ty/Ruff reuse audit has already been performed in `internal_docs/tooling_reuse_strategy.md`; implementation milestones must follow that strategy rather than reopen the audit as exploratory work.

```mermaid
flowchart TD
    m36_1["m36.1 Production Tooling Contract Lock"]
    m36_2["m36.2 Diagnostics, Rules, Suppressions, Exclusions, And Formatting Foundation"]
    m36_3["m36.3 AnalysisHost, Symbol Index, And Session Model"]
    m36_4["m36.4 Full Editor Query Layer"]
    m36_5["m36.5 Production Native LSP Server"]
    m36_6["m36.6 Multi-Editor Syntax And Integration Assets"]
    m36_7["m36.7 VS Code Extension"]
    m36_8["m36.8 Production Verification And Performance Closeout"]

    m36_1 --> m36_2
    m36_2 --> m36_3
    m36_3 --> m36_4
    m36_4 --> m36_5
    m36_5 --> m36_6
    m36_6 --> m36_7
    m36_7 --> m36_8
```

No Phase 36 milestone may depend on parallel work. Ad hoc PR slices are allowed only inside the active milestone and must not require later milestones to repair incomplete contracts from earlier ones.

## Milestones

### milestone_36_1: Production Tooling Contract Lock
- Scope:
  - Choose final crate/module names for `sifr_analysis`, formatter, policy-rule/lint, and LSP boundaries.
  - Decide whether the VS Code extension implementation lives in the recommended separate `sifr-lang/sifr-vscode` repository or in this repository, and record the validation checkout/release boundary.
  - Create or confirm the `sifr-lang/sifr-vscode` repository when the separate-repo default is kept.
  - Update `issues/phase36-vscode-extension-production-execution.md` with the final repository decision and any reviewed deviations from its default separate-repo plan.
  - Lock the LSP capability matrix, command set, diagnostics modes, settings schema, semantic token legend, code-action kinds, generated-Rust preview command shape, test explorer command shape, syntax asset source of truth, minimum VS Code engine version, and package-management boundary.
  - Convert the local Ruff/ty LSP audit inputs in this file into `lsp_protocol_matrix.json`, including selection range, type hierarchy, dynamic diagnostics, request cancellation, workspace folders/configuration, and unsupported notebook/Python behavior.
  - Create `internal_docs/tooling_analysis.md`, `internal_docs/lsp_server.md`, `internal_docs/vscode_extension.md`, `internal_docs/editor_integrations.md`, and `internal_docs/tooling_verification.md`.
  - Confirm Phase 35 exports are sufficient for references, rename, signature help, semantic tokens, selection ranges, type hierarchy, formatting, generated-Rust preview, test discovery, and editor test explorer metadata. Any missing export must be fixed in this milestone before feature implementation continues.
  - Apply `internal_docs/tooling_reuse_strategy.md` before designing public `sifr_analysis` or `sifr_lsp` handoff types.
  - Define the Sifr-owned diagnostics/rules/suppression/exclusion handoff for editor tooling.
  - Extend split-brain guardrails to reject semantics reimplementation in `sifr_lsp`, `sifr_lint`, formatter paths, editor adapters, automation adapters, and CLI-only analysis shims.
- Definition of done:
  - Supporting architecture docs exist and agree with this phase file.
  - Extension repository boundary and validation commands are settled.
  - The complete production LSP/editor feature list is represented in contracts and tests-to-add.
  - The protocol matrix maps each required LSP method and workspace command to a Sifr analysis/CLI owner, positive tests, negative tests, performance budget id, and unsupported-feature policy.
  - Missing Phase 35 exports are either implemented in this milestone or recorded as blockers that prevent moving to `milestone_36_2`.
  - Negative guardrail seeds prove forbidden parser/type-check/diagnostic/codegen paths fail validation.

### milestone_36_2: Diagnostics, Rules, Suppressions, Exclusions, And Formatting Foundation
- Scope:
  - Implement Sifr-owned policy-rule metadata, severity resolution, and hard-correctness vs policy-rule classification.
  - Implement `# sifr: ignore[rule-id]`, unknown suppression diagnostics, unused suppression diagnostics, and no-blanket-suppression enforcement.
  - Implement include/exclude discovery behavior and editor diagnostics modes: `off`, `open-files`, and `workspace`.
  - Implement formatter foundation over `sifr_syntax`, including document formatting, range formatting, idempotence checks, parser round trips, and `sifr fmt --check`.
  - Implement `sifr lint` or the final reviewed policy-rule CLI surface.
  - Add `check_formatter_contract.py` and `check_rule_suppression_contract.py`.
- Definition of done:
  - Hard correctness diagnostics cannot be suppressed or downgraded.
  - Policy diagnostics are configurable, suppressible only with explicit rule ids, and rendered through `sifr_diagnostics`.
  - Exclusions affect discovery but never explicit CLI targets.
  - Formatter output is deterministic, idempotent, parser-round-tripped, and equivalent between CLI and analysis/LSP formatting APIs.
  - Formatter verification uses `sifr_syntax` tokenization fixtures before later milestones rely on formatter output in editor/LSP paths.
  - Positive and negative contract checks run locally.

### milestone_36_3: AnalysisHost, Symbol Index, And Session Model
- Scope:
  - Create `crates/sifr_analysis/` or the final reviewed crate name for editor-oriented queries.
  - Implement `AnalysisHost` over `sifr_frontend` with project/open-file session state, coherent source snapshots, document versions, invalidation reports, and stale-result rejection.
  - Implement current-workspace symbol index, definition/reference identity, rename target validation, source-map handoff, syntax-ancestry selection handoff, type-hierarchy handoff where Sifr semantics support it, type-display contract, doc comment extraction where available, generated-Rust preview handoff, test discovery handoff, and test command handoff.
  - Implement completion ranking/evaluation infrastructure inspired by `ty_completion_eval`.
  - Add first parity fixtures for single-file and multi-file projects.
- Definition of done:
  - `AnalysisHost` compiles and exposes every query listed in this phase file, even if feature logic lands in `milestone_36_4`.
  - Symbol identity is stable within one analysis revision and safe for references/rename.
  - Stale document versions and invalidated snapshots are rejected deterministically.
  - Snapshot coherence validation proves `AnalysisHost` snapshots reflect the latest `FrontendContext` revision, stale document versions are rejected at the snapshot boundary, invalidated `sifr_frontend` queries cannot produce results through `sifr_analysis` snapshots, and no `sifr_analysis` query method exposes a result whose source revision differs from the snapshot's captured revision.
  - The analysis crate owns editor queries and does not bypass `sifr_frontend`.
  - Positive tests cover session load/update/query plumbing; negative tests cover stale versions, direct semantic bypass, and forbidden ty/Ruff Python semantic dependencies.

### milestone_36_4: Full Editor Query Layer
- Scope:
  - Implement diagnostics, workspace diagnostics, completion, hover, signature help, definition, declaration, type definition, references, prepare rename, rename, document symbols, workspace symbols, semantic tokens, inlay hints, document highlights, folding ranges, selection ranges, type hierarchy, code actions, formatting queries, generated Rust preview, explain diagnostic, test discovery, and test command metadata through `sifr_analysis`.
  - Add `verification/tooling/parity_manifest.json`, `run_tooling_parity.py`, `editor_query_snapshots/`, and `completion_quality/`.
  - Cover diagnostics codes, URLs, spans, child note/help payloads, structured suggestion payloads, renderer outputs, type-check outcomes, symbol kinds, rename edits, code actions, generated Rust mappings, semantic-token ordering, inlay hints, and LSP severity mapping.
- Definition of done:
  - Divergence between compiler/frontend/analysis behavior is automatically detected before merge.
  - Every editor query listed in the contract has positive and negative fixture coverage.
  - Completion ranking has checked-in quality evidence and regression thresholds.
  - Generated Rust preview is source-mapped and codegen-backed.
  - Rename/reference behavior is workspace-wide for the current workspace/project graph and rejects ambiguous edits.

### milestone_36_5: Production Native LSP Server
- Scope:
  - Add `crates/sifr_lsp/` or an equivalent reviewed module boundary.
  - Add `sifr lsp --stdio` to the CLI.
  - Implement all required LSP 3.17 capabilities listed in this file through `sifr_analysis`.
  - Implement full and incremental sync, push and pull diagnostics, workspace folders, file watching where supported by the client, workspace configuration, dynamic diagnostic registration, workspace commands, cancellation, request scheduling, stale-version rejection, deterministic protocol errors, tracing/logging, and snapshot discipline.
  - Implement the required Sifr-owned LSP shell layers from this file: capability registry, document store, session, request queue, scheduler, snapshot layer, conversion layer, diagnostics controller, command registry, and protocol test harness.
  - Add `lsp_protocol_smoke.py`, `lsp_protocol_stress.py`, `check_lsp_split_brain.py`, and `check_tooling_dependency_boundaries.py`.
  - Add Phase 35 `lsp-query` performance cases and budget evidence for every implemented LSP request family.
- Definition of done:
  - `sifr lsp --stdio` responds to initialize/shutdown and handles open/change/save/close/query flows in smoke and stress tests.
  - LSP diagnostics match canonical frontend/analysis diagnostics after protocol conversion.
  - Every required LSP capability passes parity snapshots.
  - Split-brain guardrails fail on seeded direct HIR traversal, parser/type-check bypass, forbidden ty/Ruff semantic dependency, or extension-owned semantic path.
  - LSP query performance budgets are recorded and enforced through the Phase 35 performance gate.

### milestone_36_6: Multi-Editor Syntax And Integration Assets
- Scope:
  - Deliver checked-in or contribution-ready Neovim, Zed, Helix, and Emacs configs using `sifr lsp --stdio`.
  - Deliver TextMate and/or Tree-sitter assets required by the VS Code and non-VS Code editor targets.
  - Add syntax asset drift checks against `sifr_syntax` tokenization fixtures.
  - Add `check_editor_assets.py`.
- Definition of done:
  - Each target editor has documented setup, filetype detection, LSP launch configuration, and syntax/highlighting strategy.
  - Syntax assets are validated against parser/token fixtures.
  - No editor integration duplicates Sifr semantic logic.

### milestone_36_7: VS Code Extension
- Scope:
  - Implement the VS Code extension in the chosen repository boundary after the shared syntax assets are validated, following `issues/phase36-vscode-extension-production-execution.md`.
  - Add language id, file extension, grammar, language configuration, LSP launcher, settings, commands, trace/logging, binary discovery, generated Rust preview, explain diagnostic, check/test commands, VS Code Test Explorer integration, format command, restart server, and server log access.
  - Add `.vsix` packaging, extension integration tests, and `vscode_extension_contract.json`.
  - Ensure extension tests can launch the locally built `sifr lsp --stdio`.
- Definition of done:
  - The extension builds, tests, and packages locally.
  - Extension contract validation proves the launcher points to `sifr lsp --stdio` and no extension-owned type checker/parser/formatter/linter setting exists.
  - Missing Sifr binary behavior is actionable and does not fall back to Python tooling.
  - Generated Rust preview, explain diagnostic, check/test, Test Explorer, and format commands call Sifr CLI/LSP surfaces rather than extension-owned logic.
  - Publication checklist and versioning/repository-boundary notes are documented.

### milestone_36_8: Production Verification And Performance Closeout
- Scope:
  - Finalize `internal_docs/tooling_verification.md`.
  - Ensure every Phase 36 verification script is wired into local validation.
  - Finalize LSP/editor performance baselines, budgets, waivers, and negative seeds.
  - Run full parity, protocol, stress, analysis snapshot coherence, dependency-boundary, formatter, rule/suppression/exclusion, editor asset, VS Code package, completion quality, performance, and split-brain checks.
  - Audit implementation against `internal_docs/tooling_reuse_strategy.md`.
- Definition of done:
  - `scripts/run_all_tests.sh --profile quick` passes.
  - `scripts/run_all_tests.sh --profile pr` passes.
  - Every required Phase 36 feature has positive and negative evidence.
  - Production validation evidence is recorded in the phase execution checklist issue.
  - No open deferrals remain inside the Phase 36 contract except package-registry intelligence after Phase 37 and release-governance publication mechanics after Phase 39.

## Quality Contract

### Entry criteria
- Phase 35 is completed and compiler performance/query contracts plus the shared syntax/frontend/query foundation are enforced.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.

### Milestone quality checks
- Local validation gates pass for each milestone before merge:
  - `scripts/run_all_tests.sh --profile quick`
  - milestone-specific `verification/tooling/*.py` checks added by the milestone
- The authoritative pre-PR gate passes before phase-closing PRs:
  - `scripts/run_all_tests.sh --profile pr`
- No tooling path uses CI-only behavior.
- No LSP or extension code reimplements parser, lowering, type-check, ownership, semantic diagnostic logic, formatter logic, linter logic, or codegen logic.
- No `sifr_lsp` or `sifr_analysis` production path depends on Python semantic/project/runtime authority from `ty_python_semantic`, Python module resolution in `ty_project`, Python environment discovery, Python diagnostic rules, or `ruff_server` semantic behavior.
- LSP JSON-RPC output is deterministic for equivalent request sequences.
- LSP and analysis conversions preserve diagnostic codes, severities, spans, URLs, help, child notes, structured suggestions, related information, tags, and fix applicability.
- No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
- No partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
- All implementations must be production-grade compiler/tooling code: deterministic behavior, explicit invariants, cancellation-safe state updates, strict protocol handling, strict dependency boundaries, and clean ownership.
- Validation evidence must be recorded in the phase execution checklist issue before merge.
- Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.

### Validation planning goals
- `milestone_36_1`:
  - Positive: supporting docs lock crate names, repo boundary, LSP capability matrix, diagnostic/rule policy, formatter/lint strategy, syntax asset strategy, minimum VS Code engine version, and package-management boundary.
  - Negative: seeded missing Phase 35 export, forbidden Python semantic dependency, or extension-owned semantic path fails the guardrail.
- `milestone_36_2`:
  - Positive: formatter, rule, suppression, exclusion, and diagnostics-mode checks pass for representative projects.
  - Negative: hard diagnostic suppression, unknown suppression, unused suppression, non-idempotent formatting, parser-round-trip drift, and explicit-target exclusion misuse fail validation.
- `milestone_36_3`:
  - Positive: `AnalysisHost` loads and updates single-file and project sessions with stable symbols, snapshots, and document versions.
  - Negative: stale versions, invalidated snapshots, direct parser/type-check/HIR semantic bypass, and forbidden ty/Ruff Python semantic dependencies fail validation.
- `milestone_36_4`:
  - Positive: analysis returns every required editor query through `sifr_frontend` for single-file and multi-file projects.
  - Negative: seeded diagnostic severity drift, span drift, completion drift, hover type drift, signature drift, definition/reference target drift, rename edit drift, generated Rust mapping drift, semantic-token ordering drift, code-action drift, and stale-version publication fail the parity gate.
- `milestone_36_5`:
  - Positive: LSP smoke and stress tests initialize, open/change/save/close `.sifr` documents, publish/pull diagnostics, answer every required request, execute required commands, handle cancellation, and shut down cleanly.
  - Negative: malformed JSON-RPC, unsupported request, stale document version, cancellation, incremental edit mismatch, workspace diagnostics drift, and direct semantic bypass fail with deterministic protocol errors or guardrail failures.
- `milestone_36_6`:
  - Positive: Neovim/Zed/Helix/Emacs configs use `sifr lsp --stdio` and syntax assets validate against parser/token fixtures.
  - Negative: editor config with Python tooling fallback, semantic implementation, or unvalidated syntax asset fails validation.
- `milestone_36_7`:
  - Positive: VS Code extension builds, tests, packages, launches local `sifr lsp --stdio`, exposes required commands/settings, and drives Test Explorer through Sifr test metadata.
  - Negative: extension-owned parser/type-checker/formatter/linter setting, missing binary discovery error, missing launch args, failed generated-Rust preview command, failed Test Explorer command, or grammar strategy without drift validation fails the contract check.
- `milestone_36_8`:
  - Positive: all tooling verification and performance gates pass locally.
  - Negative: seeded split-brain, protocol, formatter, rule, suppression, extension, editor asset, completion-quality, and performance regressions fail the appropriate checks.
- Exit-gate evidence explicitly demonstrates: tooling integration is split-brain-resistant, renderer/protocol-stable, editor-query-complete, formatter/lint-capable, extension-packageable, multi-editor-ready, performance-budgeted, and regression-covered against compiler behavior.

### CI Integration

Tooling checks must run in `scripts/run_all_tests.sh --profile pr` under a clearly named "Developer Tooling Checks" step. Local validation and CI use the same commands. CI-only tooling behavior is not allowed.

## Exit criteria

- All milestone DoDs are satisfied.
- `internal_docs/tooling_reuse_strategy.md` remains consistent with implementation, or any strategy changes have been reviewed before implementation diverges.
- `crates/sifr_analysis/` or the reviewed final crate name exists and owns editor-oriented queries.
- `sifr_format` and `sifr_lint` or reviewed equivalent module/crate boundaries exist.
- `crates/sifr_lsp/` or the reviewed final module boundary exists.
- `sifr lsp --stdio` launches a native Rust LSP 3.17 server.
- `sifr fmt --check` and `sifr lint` or reviewed equivalent CLI commands exist.
- Required LSP capabilities pass protocol smoke and stress tests.
- Diagnostics, workspace diagnostics, completion, hover, signature help, definition, declaration, type definition, references, rename, document symbols, workspace symbols, semantic tokens, inlay hints, document highlights, folding ranges, selection ranges, type hierarchy, code actions, formatting, generated Rust preview, explain diagnostic, test discovery, and test command metadata are parity-covered.
- Neovim, Zed, Helix, and Emacs integration assets/docs exist and are syntax/LSP contract-checked.
- VS Code extension builds, tests, packages, integrates with VS Code Test Explorer, and delegates semantics to `sifr lsp --stdio` plus Sifr CLI commands.
- Phase 35 `lsp-query` performance cases exist for every required LSP capability family and pass or have explicit reviewed waivers.
- `verification/tooling/run_tooling_parity.py` passes and fails on seeded divergences.
- `verification/tooling/check_analysis_snapshot_coherence.py` passes and fails on seeded stale snapshot/revision-boundary violations.
- `verification/tooling/lsp_protocol_smoke.py` and `verification/tooling/lsp_protocol_stress.py` pass and fail on seeded protocol failures.
- `verification/tooling/check_lsp_split_brain.py` and `check_tooling_dependency_boundaries.py` pass and fail on seeded split-brain violations.
- `verification/tooling/check_formatter_contract.py` passes and fails on seeded formatting drift.
- `verification/tooling/check_rule_suppression_contract.py` passes and fails on seeded rule/suppression/exclusion drift.
- `verification/tooling/check_editor_assets.py`, `check_vscode_extension_contract.py`, and `check_vscode_extension.py` pass and fail on seeded extension/editor asset drift.
- Main-repo quick/pr validation runs the VS Code extension contract check against the `editor_integrations/vscode` submodule, `SIFR_VSCODE_REPO`, or a sibling `../sifr-vscode` checkout; it fails with actionable setup instructions if Phase 36 extension validation is required and no extension checkout is available.
- Completion quality fixtures pass configured ranking thresholds and fail on seeded regressions.
- `scripts/run_all_tests.sh --profile quick` passes.
- `scripts/run_all_tests.sh --profile pr` passes.
- Phase 27 non-regression contract remains green.
- Validation evidence is recorded in the phase execution checklist issue before merge.

## Exit Gate

Sifr has one compiler/tooling brain: syntax comes from the Sifr Ruff fork through `sifr_syntax`; semantics and diagnostics come from `sifr_frontend` and `sifr_diagnostics`; formatter and policy-rule surfaces are Sifr-owned; editor intelligence comes from `sifr_analysis`; `sifr lsp --stdio` is a thin native Rust LSP adapter; VS Code and other editor integrations delegate all semantics to the LSP/CLI; and parity, protocol, performance, packaging, syntax-asset, formatter, rule/suppression, and split-brain guardrails prove the editor ecosystem cannot drift away from compiler behavior.
