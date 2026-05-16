# Tooling Reuse Strategy

status: planning-approved

## Objective

Use as much of the existing Sifr Ruff fork and ty/Ruff tooling architecture as is actually cheaper and more maintainable than rebuilding it, while preserving one Sifr semantic authority.

This document records the Phase 35/36 reuse audit. It replaces a future open-ended audit milestone: Phase 36 implementation must follow this strategy unless a reviewed PR updates this file first.

## Sources Reviewed

Primary external docs reviewed on 2026-05-16:

- `https://docs.astral.sh/ty/features/diagnostics/`
- `https://docs.astral.sh/ty/features/language-server/`
- `https://docs.astral.sh/ty/rules/`
- `https://docs.astral.sh/ty/suppression/`
- `https://docs.astral.sh/ty/exclusions/`
- `https://docs.astral.sh/ty/reference/rules/`

Local fork code reviewed:

- `third_party/ruff/crates/ty_server/`
- `third_party/ruff/crates/ty_ide/`
- `third_party/ruff/crates/ty_project/`
- `third_party/ruff/crates/ty_python_semantic/`
- `third_party/ruff/crates/ty_completion_eval/`
- `third_party/ruff/crates/ruff_server/`
- `third_party/ruff/crates/ruff_db/`
- `third_party/ruff/crates/ruff_diagnostics/`
- `third_party/ruff/crates/ruff_python_*`
- `third_party/ruff/crates/ruff_source_file/`
- `third_party/ruff/crates/ruff_text_size/`

Current Sifr code reviewed:

- `crates/sifr_diagnostics/`
- root `Cargo.toml`
- Phase 35 and Phase 36 planning docs

## Non-Negotiable Boundary

Sifr's editor and CLI must share one compiler brain.

Allowed:

- Reuse generic protocol, source-position, scheduling, glob, suppression-parser, testing, and benchmark ideas from ty/Ruff.
- Adapt local fork code when the extracted dependency graph is clean and Sifr owns the final API.
- Use ty/Ruff behavior as UX benchmark evidence.

Forbidden:

- Using `ty_server`, `ruff_server`, Pyright, Ruff, or Python semantics as Sifr's semantic authority.
- Depending on `ty_python_semantic` from production `sifr_analysis` or `sifr_lsp`.
- Depending on Python module resolution, Python environment discovery, Python type rules, Python exception rules, or Python diagnostic rules for Sifr semantic answers.
- Suppressing Sifr hard correctness errors that are required for "if it compiles, it works."

## Strategic Decision

Use a reuse-first, semantics-owned architecture:

```text
Sifr Ruff fork parser/AST/trivia/source ranges
        |
        v
sifr_syntax
        |
        v
sifr_frontend
        |
        v
sifr_analysis
        |
        v
sifr_lsp
  - depends directly on lsp-server and lsp-types
  - adapts selected ty_server/ruff_server protocol shell patterns
  - does not depend on ty_python_semantic or Python project semantics
```

The smart path is not to fork ty wholesale. The smart path is to reuse the generic shell and UX patterns while replacing the Python semantic/project core immediately with Sifr-owned crates.

## Decision Matrix

| Area | Decision | Rationale |
|---|---|---|
| `lsp-server` | reuse-direct | Generic Rust JSON-RPC/LSP transport. No Python semantics. Used by ty and Ruff server. |
| `lsp-types` | reuse-direct | Generic LSP data model. No Python semantics. |
| `ty_server` initialization and capability negotiation | adapt | Strong implementation pattern for client capabilities, position encodings, pull/push diagnostics, semantic tokens, inlay hints, rename, and workspace configuration. Must rename commands/settings and remove Python-specific capabilities. |
| `ty_server` request dispatch traits and response discipline | adapt | The code enforces one response per request, structured protocol errors, cancellation retry paths, and clear sync/background handler separation. Sifr should adapt the pattern, not blindly copy all handlers. |
| `ty_server` main loop and scheduler | adapt-with-review | Useful event loop, request queue, latency-sensitive worker split, cancellation, and snapshot model. However, current `Session` is coupled to `ty_project::ProjectDatabase`; Sifr should implement a Sifr-owned `Session` over `AnalysisHost` and may copy generic scheduler structure. |
| `ty_server` document model | adapt | `TextDocument`, `DocumentVersion`, full/incremental change application, URI keys, UTF-8/UTF-16/UTF-32 position encoding, and stale-version discipline are directly useful. Remove notebook behavior from MVP unless explicitly added later. |
| `ty_server` location/range conversion | adapt | Range conversion and URI-carrying location helpers are valuable. Replace `ruff_db::File` and notebook mapping with Sifr `FileId`, `SourceUri`, and `SourceMapView`. |
| `ty_server` diagnostics publication lifecycle | adapt | Pull and push diagnostics, document-version tagging, dynamic registration, related information support, and settings diagnostics are good patterns. Diagnostic payloads must be generated from `sifr_diagnostics`. |
| `ty_server` settings model | reference-only | Useful split between global/workspace/editor settings, unknown-option diagnostics, and dynamic updates. Current implementation imports Python versions, Python extension environment, and `ty_project` options. |
| `ty_ide` public query surface | reference-only | The surface is a strong checklist: completion, hover, goto, references, document symbols, workspace symbols, semantic tokens, inlay hints, signature help, document highlights, folding, rename, code actions. Direct code is coupled to Python AST, Python semantic model, and Python module resolver. |
| `ty_ide` completion ranking/evaluation | adapt pattern | Completion ranking and `ty_completion_eval` mean-reciprocal-rank evaluation are useful for Sifr completion quality gates. The semantic candidates must be Sifr-native. |
| `ty_ide` semantic-token categories | reference-only | Useful LSP category benchmark. Sifr token meanings differ because of ownership, mutability, Result/Option, and Rust-codegen concepts. |
| `ty_project` project database | reject for production | Deeply Python-specific: Python module resolution, Python settings, Python source types, Python environment discovery, vendored stubs/typeshed concepts. |
| `ty_project` glob include/exclude implementation | adapt or vendor into Sifr-owned crate | The portable glob and include/exclude behavior is useful for Sifr workspaces. Extract only generic glob code if dependency graph stays clean. |
| `ty_project` file watcher/change classification | reference-only | Useful architecture, but tied to Python config and project database. Sifr should build watcher integration over `sifr_frontend` revisions. |
| `ty_python_semantic::lint` rule metadata and severity model | adapt concept, reject dependency | Rule metadata, default levels, status, docs URL, and `ignore`/`warn`/`error` levels are valuable. Production Sifr must define `sifr_diagnostics`-owned rules because ty lints are Python semantic rules. |
| `ty_python_semantic::suppression` parser | adapt concept, possible code extraction | The parser for `ty: ignore[...]` and unused suppression diagnostics is useful. Sifr should design `sifr: ignore[...]` or `sifr: allow[...]`, using Sifr rule ids and forbidding suppression of hard correctness errors. Direct dependency on `ty_python_semantic` is rejected. |
| `ruff_db::diagnostic` | reference-only | Strong diagnostic shape: annotations, subdiagnostics, concise messages, fixes, docs URLs, secondary codes, renderers. Sifr already has `sifr_diagnostics`; replacing it would break schema and renderer contracts. Adopt missing concepts selectively. |
| `ruff_diagnostics` fixes/edits | reference-only | Sifr already has structured suggestions. Compare edit/applicability behavior before adding code-action plumbing. |
| `ruff_server` architecture | reference-only or adapt selected shell | Its contributing guide explicitly supports `lsp-server`/`lsp-types`, sync/background tasks, and Arc snapshots. Useful for architecture. Ruff diagnostics/formatting are not Sifr semantic authority. |
| Ruff parser/AST/trivia/text crates | reuse-direct through `sifr_syntax` | Already Sifr's syntax substrate. Phase 35 must wrap them behind Sifr-owned API. |

## Diagnostic Strategy

Sifr keeps `crates/sifr_diagnostics/` as the canonical diagnostic model.

Reasons:

- Sifr already has stable diagnostic codes such as `SIFR-TYPE-*`, `SIFR-OWN-*`, `SIFR-RESULT-*`, and `SIFR-WORKSPACE-*`.
- Sifr's rendered JSON envelope is schema-checked and used as the canonical renderer source.
- Sifr already has severities, child notes/help, source spans, structured args, docs URLs, suggestions, edit applicability, deterministic ordering, and renderer parity.
- Phase 27 requires stable diagnostic schema, codes, severities, spans, URLs, suggestions, renderer views, and exit-code behavior.

Adopt from ty/Ruff diagnostics:

- Diagnostic UX benchmark: source snippets, annotations, contextual reference spans, and fix suggestions.
- Concise-message behavior for compact/LSP contexts where primary annotation text carries essential context.
- Rule metadata shape for suppressible diagnostics: rule id, summary, docs URL, default level, status, source location, and configured level.
- LSP diagnostic modes: `off`, `open-files`, and `workspace`, when they map cleanly to Sifr project analysis.
- Pull diagnostics support in addition to push diagnostics once protocol smoke tests cover both.

Do not adopt:

- `ruff_db::Diagnostic` as Sifr's core diagnostic type.
- ty/Python lint ids as Sifr rule ids.
- Ruff `noqa` semantics as Sifr's suppression syntax.
- Blanket suppression for Sifr hard correctness errors.

## Rule, Suppression, and Exclusion Strategy

ty's rules/suppression/exclusion model is worth copying as a product shape, not as a semantic dependency.

Sifr should split diagnostics into two categories:

1. Hard correctness diagnostics:
   - Parse errors.
   - Type errors required for soundness.
   - Ownership/move/borrow errors.
   - `Result`/`Option` safety errors.
   - Runtime-panic-prevention errors.
   - Workspace/import errors that would make compilation ambiguous or unsound.
   - These are not suppressible and cannot be downgraded to warning.
2. Policy rules:
   - Warnings, style-adjacent static analysis, migration advisories, unused code/imports, unreachable code, optional strictness checks, and future lint-like diagnostics.
   - These can have `ignore`/`warn`/`error` configuration if doing so does not violate Sifr's core guarantee.

Recommended Sifr configuration shape:

```toml
[diagnostics.rules]
unused-import = "warn"
unreachable-code = "warn"
bigint-transition-alias = "warn"
all-policy = "warn"

[diagnostics]
mode = "workspace" # off | open-files | workspace, for LSP diagnostics only

[src]
include = ["src", "tests"]
exclude = ["src/generated"]
respect-ignore-files = true
```

Recommended suppression shape:

```sifr
value = legacy_call()  # sifr: ignore[unused-import]
```

Rules:

- Require explicit rule ids for Sifr suppression comments.
- No bare `sifr: ignore` in production code unless a reviewed phase explicitly permits it.
- `sifr: ignore[...]` can suppress only policy rules.
- Unknown rule ids produce a warning or error according to the policy-rule configuration.
- Unused suppression comments are reported by a policy rule similar to ty's `unused-ignore-comment`.
- Existing Python `type: ignore` comments should not suppress Sifr diagnostics by default. If compatibility is added, it must require explicit `type: ignore[sifr:<rule>]`.

Exclusions:

- Reuse ty's product model: include/exclude patterns, default ignored directories, explicit CLI targets overriding excludes, and respect for `.gitignore`/`.ignore`.
- Extract or reimplement the portable glob matcher from `ty_project` only if it can be isolated without Python project state.
- File exclusions must affect discovery scope, not the semantics of files already explicitly passed to the compiler.

## LSP Feature Strategy

ty's language-server docs show the target class of editor experience Sifr should aim for:

- diagnostics updated as users type
- both pull and push diagnostics
- go to definition/declaration/type definition
- references
- document and workspace symbols
- completion with auto-import style actions
- quick fixes and rename
- hover, inlay hints, signature help, document highlight
- semantic highlighting
- folding
- fine-grained incrementality

Phase 36 MVP remains smaller, but the architecture must not block the full list.

MVP:

- initialize/shutdown/exit
- full document sync
- diagnostics
- completion
- hover
- definition
- document symbols
- semantic tokens

Near-follow-up:

- pull diagnostics
- references
- inlay hints
- signature help
- document highlight
- folding
- rename
- code actions from Sifr diagnostic suggestions
- workspace symbols
- auto-import only after Sifr import/module indexing is stable

Explicit deferrals:

- notebook support
- formatting
- type hierarchy/call hierarchy
- generated Rust preview
- test explorer

## Accepted Dependency Graph

Production Phase 36 may depend on:

- `lsp-server`
- `lsp-types`
- generic crates already used by ty/Ruff server shell patterns, after normal dependency review
- Sifr-owned crates: `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, `sifr_diagnostics`
- selected copied/adapted modules from `ty_server`, `ruff_server`, or `ty_project` after they are moved behind Sifr-owned APIs and cleaned of Python assumptions

Production Phase 36 must not depend on:

- `ty_python_semantic`
- `ty_project` as a whole
- `ty_module_resolver` as Sifr module resolver
- `ty_vendored` or Python typeshed/stub semantics
- `ty_site_packages`
- Python environment discovery
- `ruff_server` diagnostics or formatting as Sifr semantic behavior
- `ruff_python_semantic` for Sifr semantic answers

## Implementation Guidance

Phase 35:

- Create `sifr_syntax` as planned and keep Ruff parser/AST dependencies isolated there.
- Create `sifr_frontend` as planned and keep Sifr diagnostics canonical.
- Make split-brain guardrails configurable enough for Phase 36 to forbid Python semantic dependencies.
- Add or reserve diagnostic-rule metadata fields only if needed without destabilizing the existing schema.

Phase 36:

- Do not start with a blank LSP server.
- Start from the selected protocol shell patterns in `ty_server` and `ruff_server`.
- Implement a Sifr-owned session:
  - open document index
  - URI and version mapping
  - source-map conversion
  - `AnalysisHost` ownership
  - request queue and cancellation state
- Implement LSP handlers as protocol adapters over `sifr_analysis`.
- Build dependency guardrails before the first LSP MVP merge.
- Add completion quality evaluation inspired by `ty_completion_eval` once Sifr completion ranking exists.

## Verification Requirements

The planning decision is complete only when these are reflected in Phase 35/36:

- `internal_docs/tooling_reuse_strategy.md` is a source-of-truth planning input, not a future artifact.
- Phase 36 no longer defers the reuse audit as an open-ended milestone.
- Phase 36 requires implementation to follow this decision matrix or update it by reviewed PR.
- Tooling guardrails reject forbidden Python semantic dependencies.
- Diagnostics planning explicitly keeps `sifr_diagnostics` canonical and adopts only selected ty/Ruff concepts.
- Rule/suppression/exclusion planning distinguishes hard correctness diagnostics from configurable policy rules.
