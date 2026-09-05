# Phase 35 + Phase 36 Tooling Readiness Review — Pass 1

## Review Context

- Reviewer: agent (automated tooling readiness audit)
- Date: 2026-05-16
- Branch: `codex/phase-35-readiness-review`
- Target phases: `35_performance_benchmarking_and_budgets.md`, `36_developer_tooling_and_ecosystem_hooks.md`
- Crate inventory (via `Cargo.toml`): `sifr_diagnostics`, `sifr_hir`, `sifr_codegen`, `sifr_driver`, `sifr` — no `sifr_frontend`, no `sifr_lsp`, no `sifr_analysis`, no `sifr_syntax`

---

## 1. Verdict

**NOT READY.** Phase 35 and Phase 36 as written are directionally correct but structurally incomplete for the target end state. The most critical gap is that Phase 35 targets a crate (`sifr_frontend`) that does not exist and whose public API definition is incomplete for LSP consumption. Phase 36's "thin adapter" boundary is undefined at the LSP query level and omits VS Code extension architecture entirely. There are also missing layers (`sifr_analysis`/`sifr_ide`, `sifr_syntax`) that the target architecture names but neither phase documents.

---

## 2. Blocking Gaps (Ranked by Severity)

### [BLOCKER-1] Phase 35: `sifr_frontend` crate does not exist (Phase 35 references non-existent crate)

**Severity: Critical**

Phase 35 designates `crates/sifr_frontend/` as the canonical owner of the frontend query API and makes this an exit gate requirement. The current `Cargo.toml` workspace does not include `sifr_frontend`. The current frontend entrypoints live in `sifr_driver/src/frontend/` as plain module functions (`check`, `compile`, `lower_source`, `type_check_source`, etc. in `api.rs`).

Phase 35 correctly recognizes this as a migration, but the `m35.4a` → `m35.4b` split assigns only **one** milestone to crate creation and migration. This underestimates the mechanical and semantic risk of extracting a query API from a working driver.

**What is needed:**
- Phase 35 must explicitly define the migration path: extract from `sifr_driver/src/frontend/` into `crates/sifr_frontend/` with no behavioral divergence during the transition
- The split into `m35.4a` (API skeleton) and `m35.4b` (CLI adoption) is the right approach but the scope of `m35.4a` must include establishing the crate boundary before benchmarks measure anything
- A code-level split-brain guardrail must be defined in `m35.4a`: any new parse/lower/type-check/diagnostic entrypoint outside `sifr_frontend` or `sifr_hir` internals must fail a lint gate

### [BLOCKER-2] Phase 36: No LSP query interface or `sifr_analysis`/`sifr_ide` boundary definition (Phase 36 omits editor-oriented query API)

**Severity: Critical**

Phase 36's `milestone_36_1` (Shared Frontend API Contract) only mandates CLI/tooling parity through the **Phase 35 frontend query API** — parse/lower/type-check/diagnostics. But an LSP server needs more than diagnostics. It needs:
- **Completion** (token at position, full symbol table, scoped visibility)
- **Hover** (type of symbol at position)
- **Go-to-definition** (span → DefId → span)
- **Find-references** (DefId → all use sites)
- **Semantic tokens** (token kind per span from HIR)
- **Inlay hints** (type annotations at parameter sites)
- **Document symbols** (top-level scope tree)

The Phase 35 frontend query API only exposes `parse_module`, `lower_module`, `type_check_module`, `diagnostics_for_*`, `analysis_for_module`, and `analysis_for_project`. The `analysis_for_*` query result type is undefined in Phase 35's API draft — it is listed but not specified.

**What is needed:**
- Phase 36 (or a pre-Phase-36 amendment) must define the **editor query layer** as a distinct ownership boundary between `sifr_frontend` (parse/lower/type-check/diagnostics) and `sifr_analysis`/`sifr_ide` (editor-oriented queries)
- The name is not yet decided (`sifr_analysis` or `sifr_ide`), but the boundary must be defined: editor consumers can call into `sifr_frontend` for diagnostics AND call into `sifr_analysis` for cross-reference, completion, semantic token data
- The thin-adapter property in `milestone_36_3` must be defined at the LSP handler level: LSP handlers call `sifr_analysis` queries, not reimplementing symbol table construction, span-to-node mapping, or HIR traversal
- The "full LSP server is not required in this phase" language is acceptable for Phase 36 scope, but Phase 36 must still define the query contract so a future `sifr_lsp` does not have to reverse-engineer it

### [BLOCKER-3] Phase 36: VS Code extension architecture is entirely absent

**Severity: Critical**

The user's desired end state explicitly includes: *"Sifr should eventually deliver a VS Code extension, possibly in another repo, with grammar/filetype/LSP launcher/settings."* Neither Phase 35 nor Phase 36 mentions VS Code, extension packaging, TextMate grammar, Tree-sitter integration, LSP launcher configuration, or repository strategy.

**What is needed in Phase 36:**
- Scope acknowledgment: the VS Code extension (host) is separate from the LSP server (guest) — the extension owns grammar, filetype registration, and LSP binary launch; it must NOT own type checking
- Repository boundary: the extension may live in `sifr-lang/sifr-vscode` or equivalent (separate from the main `sifr-lang/sifr` repo)
- Grammar strategy: TextMate `.tmLanguage.json` vs Tree-sitter `.wasm` — the decision should be deferred but the choice must be named and the tradeoffs documented
- LSP binary path: the extension must locate and launch `sifr lsp` as a subprocess — the launcher config should be defined in the extension's package.json, not hardcoded
- Settings surface: `sifr.*` configuration keys that the extension surfaces to the user (format on save, trace level, etc.)
- Validation gate: the extension test harness should validate that the extension correctly launches `sifr lsp` and handles LSP stdio protocol — this can be an integration test that runs the extension host against a mock LSP server

### [BLOCKER-4] Phase 35: Benchmark corpus omits LSP-relevant workloads

**Severity: High**

Phase 35's benchmark corpus (`manifest.json` groups) covers `check-single-file`, `check-project`, `build-single-file`, `build-project`, `incremental-local-loop`, and `phase27-non-regression`. These cover CLI latency. They do not cover:

- **LSP cold-start time** (time from `sifr lsp` subprocess spawn to initialized response)
- **Editor query latency** (completion, hover, go-to-definition — single-file and multi-file)
- **Incremental document sync** (didChange with partial parse vs full parse)
- **Cross-reference cache warm-up** (first find-references vs subsequent)

The LSP server is a long-running process, so it has a different performance profile than CLI invocations. Budgets established in Phase 35 for CLI check/build latency are insufficient for LSP editor UX.

**What is needed:**
- Phase 35 or a Phase 35 amendment must add an `lsp-query` benchmark group with cases covering cold-start, query round-trip, and incremental sync
- The budget thresholds for LSP queries (e.g., completion < 200ms, hover < 100ms) should be recorded in `verification/performance/budgets.json` with rationale
- LSP benchmark infrastructure may live in a separate `verification/lsp/` directory and be owned by `sifr_lsp` once that crate is created, but the budget definition should be in Phase 35 so it is not retrofitted

### [BLOCKER-5] Phase 36: Parity corpus is underspecified for tooling integration

**Severity: High**

`milestone_36_2` defines the minimum parity corpus (one parse diagnostic, one type-check diagnostic, one warning, one help, one structured suggestion, one multi-file, one recovery case). This is a reasonable start, but it is not sufficient for LSP integration. An LSP consumer also needs parity on:
- Symbol table completeness (all definitions have correct span and kind)
- Diagnostic severity mapping (error/warning/note → LSP diagnostic severity)
- `textDocument/publishDiagnostics` params (version, uri, diagnostics array)
- Completion item kinds (variable, function, class, keyword — from HIR node type)
- Hover content (type string derived from HIR `reveal_type` output)

**What is needed:**
- Phase 36 `milestone_36_2` scope must be extended to cover editor query parity (not just diagnostics)
- The parity matrix must validate that the same source file produces the same HIR-derived data for both CLI and LSP consumers
- A JSON fixture format should be defined for parity corpus entries so that tooling-facing test results can be compared against compiler-facing test results deterministically

---

## 3. Concrete Edits Required Per Phase Doc

### Phase 35 Edits

#### A. Add `sifr_frontend` migration plan (addresses BLOCKER-1)

In the **"Architecture Ownership"** section, add:

> **Migration path from `sifr_driver/src/frontend/` to `crates/sifr_frontend/`:**
>
> 1. `m35.4a` creates `crates/sifr_frontend/` with a minimal public surface (`FrontendContext`, `QueryResult`, `ModuleId`, `ModuleGraphView`) extracted from `sifr_driver/src/frontend/api.rs` and `module_lowering.rs` — no new behavior, only repackaging
> 2. During migration, `sifr_driver` re-exports `sifr_frontend` so no caller changes are required mid-milestone
> 3. The facade in `sifr_driver::frontend_query` is removed only after `sifr_driver` itself is updated to call `sifr_frontend` directly
> 4. Split-brain guardrail (code-level): add a `#[deny(lint_placeholder)]` annotation or a `check_split_brain.sh` script that greps for new parse/lower/type-check/semantic-diagnostic entrypoints outside `sifr_frontend` and `sifr_hir` — the script runs in `scripts/run_all_tests.sh --profile quick`

In **"Shared Frontend API Contract"**, add to the API surface:

```rust
// Missing from current draft but required for LSP and incremental edit loops:
pub fn symbol_at_position(&self, module: ModuleId, pos: ByteOffset) -> Option<SymbolInfo<'_>>;
pub fn completions_at_position(&self, module: ModuleId, pos: ByteOffset) -> QueryResult<CompletionList<'_>>;
pub fn references_for_symbol(&self, module: ModuleId, def: DefId) -> QueryResult<Vec<Span>>;
pub fn semantic_tokens_for_module(&self, module: ModuleId) -> QueryResult<Vec<SemanticToken>>;
```

Or, if the editor query layer is deferred to `sifr_analysis` (the preferred path), explicitly document the query contract boundary:

> `sifr_frontend` exposes parse/lower/type-check/diagnostics queries. `sifr_analysis` (Phase 36 or later) exposes editor-oriented queries (completion, hover, definitions, references, semantic tokens). `sifr_lsp` is a thin adapter that converts LSP protocol messages into `sifr_analysis` calls and `sifr_analysis` results into LSP protocol responses. **No LSP handler may call `sifr_hir` directly for editor queries.**

#### B. Add LSP-relevant benchmark group (addresses BLOCKER-4)

In **"Benchmark Corpus Contract"**, add a new group:

```json
{
  "id": "lsp-query",
  "description": "LSP server query workloads — cold-start, editor round-trips, incremental sync",
  "cases": [
    {
      "id": "lsp-cold-start",
      "command": "sifr lsp --stdio < /dev/null",
      "warmup": 2,
      "measurements": 10,
      "timeout_ms": 5000,
      "budget_id": "lsp-cold-start-median"
    },
    {
      "id": "lsp-completion-single-file",
      "query": "textDocument/completion at function call site",
      "warmup": 3,
      "measurements": 20
    }
  ]
}
```

Add a new corpus threshold:

> - at least 3 `lsp-query` cases covering cold-start, completion, and hover

---

### Phase 36 Edits

#### A. Define `sifr_analysis`/`sifr_ide` query layer boundary (addresses BLOCKER-2)

In **"milestone_36_3"**, after the thin-adapter scope section, add:

> **Editor query layer boundary (Phase 36 scope):**
>
> The `sifr_analysis` (or `sifr_ide`) crate owns editor-oriented query data derived from HIR:
> - **Completion:** symbol table scoped to position, filtered by prefix
> - **Hover:** type string for symbol at position (derived from HIR type inference)
> - **Go-to-definition:** DefId → canonical definition span
> - **Find references:** DefId → all use-site spans with file/line/col
> - **Document symbols:** top-level scope tree with kind and span
> - **Semantic tokens:** token classification per span (keyword, type, function, variable, comment, string, number)
>
> `sifr_lsp` (a future crate) is a thin protocol adapter that:
> 1. Receives LSP JSON-RPC messages over stdio
> 2. Calls `sifr_frontend` for project context and `sifr_analysis` for query results
> 3. Returns LSP protocol responses
>
> `sifr_lsp` does not own a parser, type checker, or HIR data structure.
> All editor queries flow through `sifr_frontend` → `sifr_analysis`, never directly into `sifr_hir` internals from LSP handlers.
>
> **Anti-split-brain rule for LSP:** any new LSP handler that directly traverses HIR without going through `sifr_analysis` must fail the split-brain guardrail. `sifr_analysis` is the sole semantic authority for editor-oriented queries.

#### B. Add VS Code extension scope (addresses BLOCKER-3)

Add a new milestone or extend `milestone_36_3` with a clearly named subsection:

> **milestone_36_4: VS Code Extension Architecture (deferred implementation, contract-only this phase)**
>
> Scope:
> - Define the VS Code extension as a separate repository (`sifr-lang/sifr-vscode` or equivalent)
> - The extension owns: grammar (TextMate or Tree-sitter), filetype registration, `.sifr` language configuration, LSP server launcher configuration, and user-facing settings (`sifr.*`)
> - The extension must NOT own type checking, symbol analysis, or diagnostic emission — it delegates to `sifr lsp` for all semantic work
> - LSP binary discovery: the extension searches `PATH` for `sifr` and falls back to a configured `sifr.lsp.path` setting
> - Grammar strategy: Phase 36 defers the TextMate vs Tree-sitter decision but documents the tradeoffs and requires that the chosen approach generates highlighting from `sifr_python_parser` token kinds, not from a separate hand-maintained grammar
> - Validation gate: extension integration tests must verify that `sifr lsp` subprocess starts and responds to `initialize` before any document work
>
> Definition of done:
> - Extension architecture is documented in `internal_docs/vscode_extension.md`
> - Grammar source of truth is the Sifr tokenizer output, not a manually authored `.tmLanguage.json`
> - The extension repository boundary and CI strategy are defined

#### C. Extend parity corpus for editor queries (addresses BLOCKER-5)

In **"milestone_36_2"**, extend the minimum parity corpus:

Add to the corpus items:
- one completion parity case (same source → same completion items via CLI query API and via LSP handler)
- one hover parity case (same source + position → same type string)
- one go-to-definition parity case (same source + position → same target span)
- one semantic tokens parity case (same source → same token sequence)

---

## 4. Non-Blocking Improvements

### Phase 35: Document `SourceId` and `SourceMap` persistence across sessions

Phase 35 defines process-local caching. For LSP, the source map (file URIs → text) must survive across LSP protocol messages within a session but is distinct from the query cache. Clarify that `SourceMap` is managed by `sifr_frontend` and is query-cache orthogonal.

### Phase 35: Define `GraphRevision` stability under concurrent edits

`update_module_source` is defined as `&mut self`. For LSP with `didChange` notifications, concurrent edits may arrive faster than query completion. Document the concurrency model (single-threaded LSP process, no concurrent `update_module_source` calls) and whether a queue is needed.

### Phase 36: Document LSP initialization handshake

`milestone_36_3` mentions a "non-CLI editor/automation-facing diagnostic adapter" but does not describe the LSP initialization sequence (`initialize` → `initialized` → `textDocument/didOpen`). This contract should be defined so the thin adapter is implementable without inventing protocol behavior.

### Phase 36: Define the document sync model

LSP supports full document sync, incremental sync, and open-only sync. The thin adapter must declare which mode it implements and why. This is a small but important contract detail.

### Phase 36: Add anti-split-brain enforcement for `sifr_lsp`

Once `sifr_lsp` is created, a lint or test must verify that no LSP handler directly imports `sifr_hir` types for semantic queries. This can be a `#[deny(unused_imports)]` lint gate or a structured test that runs the LSP against the parity corpus and compares results against the `sifr_frontend` API.

### Phase 36: Name the TextMate/Tree-sitter decision point explicitly

Even though it is deferred, Phase 36 should document: "The grammar strategy (TextMate `.tmLanguage.json` generated from `sifr_python_parser` token kinds, or Tree-sitter `.wasm`) will be decided in a follow-up phase with explicit tradeoffs documented. The chosen approach must use `sifr_python_parser` as the tokenization source, not a manually maintained grammar."

### Phase 35: Define cache invalidation for LSP document sync

The `update_module_source` API in Phase 35 returns `InvalidationReport`. For LSP `textDocument/didChange`, the invalidation report must be consumed by the LSP handler to know which query results are stale. Document the consumption contract.

---

## 5. What Would Satisfy the Next Review Round

### Phase 35 satisfaction criteria

1. `crates/sifr_frontend/` exists in `Cargo.toml` with the Phase 35 public API (`FrontendContext`, `ModuleId`, `ModuleGraphView`, `QueryResult`, `diagnostics_for_*`, `analysis_for_*`)
2. `sifr_driver` is updated to call `sifr_frontend` — the old `sifr_driver/src/frontend/api.rs` functions are removed or re-exported through `sifr_frontend`
3. Split-brain guardrail is implemented: `check_split_brain.sh` runs in `scripts/run_all_tests.sh --profile quick` and fails on new parse/lower/type-check entrypoints outside `sifr_frontend` or `sifr_hir`
4. Benchmark corpus includes `lsp-query` group with cold-start, completion, and hover cases
5. `GraphRevision` and `InvalidationReport` consumption contract is documented for LSP use

### Phase 36 satisfaction criteria

1. `sifr_analysis` query boundary is defined: what queries it owns, what `sifr_lsp` may call, what `sifr_hir` interactions are forbidden from LSP handlers
2. VS Code extension architecture is documented: separate repo, grammar strategy, LSP launcher, settings surface, and validation gates
3. Parity corpus includes editor query cases (completion, hover, go-to-definition) — not just diagnostics
4. Thin adapter definition is concrete: which LSP handler maps to which `sifr_analysis` query, and how error responses (parse failure, type-check failure) propagate as LSP diagnostic notifications

### Cross-phase satisfaction criteria

1. The `sifr_syntax` wrapper layer is named and its boundary documented: does it wrap `sifr_python_parser` token kinds, or does it also wrap `sifr_python_ast` node construction? This is relevant because LSP needs tokenization (for semantic tokens and syntax highlighting) and the tokenization source must be `sifr_python_parser`, not a parallel implementation.
2. The LSP stdio protocol version is declared (currently LSP 3.17 is the stable target)
3. `sifr lsp` CLI subcommand is documented in Phase 36 — it is the entry point for the LSP server, and its flag surface (`--stdio`, `--tcp`, `--port`) should be defined

---

## Summary Scorecard

| Dimension | Phase 35 | Phase 36 | Notes |
|---|---|---|---|
| Crate boundaries | ⚠️ Incomplete — `sifr_frontend` missing | ⚠️ Incomplete — `sifr_analysis`/`sifr_lsp` missing | Both reference non-existent crates |
| Anti-split-brain | ⚠️ Defined but not enforced | ⚠️ Defined but enforcement mechanism unclear | Guardrail scope needs detail |
| LSP query contract | ❌ Missing | ⚠️ Partial — diagnostics covered, editor queries not | BLOCKER-2 |
| VS Code extension | ❌ Not mentioned | ❌ Not mentioned | BLOCKER-3 |
| Benchmark scope | ⚠️ CLI-only, no LSP workloads | N/A | BLOCKER-4 |
| Parity corpus | N/A | ⚠️ Diagnostics only, no editor queries | BLOCKER-5 |
| `sifr_syntax` layer | ❌ Not defined | ❌ Not defined | Grammar source of truth undefined |
| `sifr_ide`/`sifr_analysis` | ❌ Not defined | ❌ Not defined | Editor query ownership gap |
| Performance budgets | ⚠️ CLI budgets defined, LSP budgets missing | N/A | LSP UX thresholds not defined |
| Ruff fork hygiene | ✓ Recognized in architecture.md | ✓ Recognized | Must not become Sifr semantic authority |

**Recommendation:** Phase 35 and Phase 36 should not proceed to implementation until the edits in section 3 are incorporated. The phases are directionally correct but underspecified for the target end state. The most important immediate action is defining `sifr_frontend`'s public API in Phase 35 and defining the editor query boundary (`sifr_analysis`/`sifr_ide`) in Phase 36.