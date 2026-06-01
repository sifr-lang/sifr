# TypeScript-Go Architecture Transfer M1 Guardrails

status: M1 pre-flight gate

This document is the implementation guardrail for
`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` M1. It records
the actual pre-session state after M0 and before M2-M5 behavior migration. Later
milestones may update this file and its checker when they intentionally replace
one of these current limitations.

## Locked Terms

M1 locks the following terms before behavior migration starts:

- `sifr_source`: bottom-of-graph source text, line-map, source-file metadata,
  source hash, and UTF-8/UTF-16/UTF-32 position conversion authority.
- `SourceProvider`: future semantic file-system boundary in `sifr_frontend` for
  disk, overlay, tracking, package, directory, canonicalization, and failed
  lookup reads. Not implemented in M1.
- `WorkspaceSession`: future mutable owner of workspace compiler-service state.
  Not implemented in M1.
- `WorkspaceSnapshot`: future immutable captured state shared by analysis, LSP,
  lint, format, diagnostics, package tooling, and future API handles. Not
  implemented in M1.
- `DirtyScope`, `DirtyReason`, `ImportSignature`, `ExportSignature`,
  `ModuleSignature`, `CompilerFingerprint`, `CacheKeyFingerprint`,
  `FlowGraph`, `QueryReadiness`, and `.sifrbuildinfo`: locked architecture terms
  for later milestones, not M1 behavior.

## Current Direct-Read, Probe, And Documented Effect Inventory

These production filesystem reads and path probes are semantic inputs, tooling
inputs, package identity inputs, or command-surface inputs. M2 must route
semantic entries through a typed source provider or explicitly reclassify a row
as a non-semantic exception. Path probes are listed alongside content and
directory reads so M2 can track successful and failed lookup dependencies
without treating probes as source content reads.

| Area | Current site | Current behavior | M2 expectation |
| --- | --- | --- | --- |
| Frontend project entrypoint read | `crates/sifr_frontend/src/graph_cache_and_queries.rs:312` | `FrontendContext::load_project` reads the entrypoint from disk. | Provider-tracked file read. |
| Frontend project directory read | `crates/sifr_frontend/src/graph_cache_and_queries.rs:322` | `load_project` enumerates sibling `.sifr` files directly. | Provider-tracked directory read. |
| Frontend project module read | `crates/sifr_frontend/src/graph_cache_and_queries.rs:359` | `load_project` reads non-entry modules directly. | Provider-tracked file read. |
| Driver module resolution | `crates/sifr_driver/src/project/discovery.rs:90`, `crates/sifr_driver/src/project/discovery.rs:118`, `crates/sifr_driver/src/project/discovery.rs:180` | Resolution probes physical paths with `is_file`. | Provider-tracked successful and failed lookup dependencies. |
| Driver project discovery | `crates/sifr_driver/src/project/discovery.rs:332`, `crates/sifr_driver/src/project/discovery.rs:574` | Project discovery enumerates `.sifr` files and reads selected modules. | Provider-tracked directory and file reads. |
| Driver workspace manifest discovery | `crates/sifr_driver/src/workspace/mod.rs:32`, `crates/sifr_driver/src/workspace/mod.rs:49`, `crates/sifr_driver/src/workspace/mod.rs:156` | Workspace root discovery probes `sifr.toml`, reads it, and checks configured source roots. | Provider-tracked config reads and directory probes. |
| Driver package module materialization | `crates/sifr_driver/src/project/package_discovery.rs:53` | Package build/check reads resolved package module source directly. | Provider-tracked package file read. |
| Linter standalone input | `crates/sifr_lint/src/engine.rs:134` | Lint engine reads each target file directly. | Short-lived provider with shared source-map authority. |
| Linter config and discovery | `crates/sifr_lint/src/config.rs:48`, `crates/sifr_lint/src/config.rs:72`, `crates/sifr_lint/src/discovery.rs:29`, `crates/sifr_lint/src/discovery.rs:33`, `crates/sifr_lint/src/discovery.rs:79` | Linter config lookup and target discovery probe files/directories and read `sifr.toml`. | Short-lived provider with tracked config and discovery reads. |
| Formatter standalone input | `crates/sifr_format/src/lib.rs:177`, `crates/sifr_format/src/lib.rs:180`, `crates/sifr_format/src/lib.rs:197`, `crates/sifr_format/src/lib.rs:215`, `crates/sifr_format/src/lib.rs:446`, `crates/sifr_format/src/lib.rs:456` | Formatter checks path shape, walks directories, reads source, and writes formatted files. | Short-lived provider for reads; writes remain command-output effects. |
| Formatter config | `crates/sifr_format/src/config.rs:85`, `crates/sifr_format/src/config.rs:109` | Formatter config lookup probes candidate files and reads `sifr.toml`. | Short-lived provider with tracked config reads. |
| Package offline availability | `crates/sifr_package/src/cargo/lock_modes.rs:46` | Offline dependency validation probes whether package roots are available. | Provider-tracked package metadata probe or reviewed package-management exception. |
| Package manifest read | `crates/sifr_package/src/manifest/sifr.rs:55` | Package identity reads `sifr.toml` directly. | Provider-tracked package/config identity input. |
| Package manifest validation | `crates/sifr_package/src/manifest/validate.rs:14`, `crates/sifr_package/src/manifest/validate.rs:43`, `crates/sifr_package/src/manifest/validate.rs:44` | Package manifest validation probes source roots and exported source files. | Provider-tracked directory/file probes. |
| Package source-map traversal | `crates/sifr_package/src/imports/source_map.rs:240`, `crates/sifr_package/src/imports/source_map.rs:254` | Package source-map construction recursively reads package source-root directories and probes child directories. | Provider-tracked directory reads and path probes. |
| Package namespace API | `crates/sifr_package/src/imports/namespace_api.rs:32`, `crates/sifr_package/src/imports/namespace_api.rs:264` | Package public API extraction reads `__init__.sifr` and probes child namespaces. | Provider-tracked package source reads and probes. |
| Package source layout | `crates/sifr_package/src/source/layout.rs:30` | Pure-marker validation reads generated package source files. | Reviewed provider-backed package tooling read or non-semantic generated-output exception. |
| Package session discovery and targets | `crates/sifr_package/src/ops/session_discovery.rs:6`, `crates/sifr_package/src/ops/session_discovery.rs:13`, `crates/sifr_package/src/ops/session_discovery.rs:25`, `crates/sifr_package/src/ops/session_targets.rs:17`, `crates/sifr_package/src/ops/session_targets.rs:34`, `crates/sifr_package/src/ops/session_targets.rs:42` | Package CLI/session discovery probes manifests and source roots. | Provider-tracked package session reads and probes where they affect compilation. |
| CLI lint command reads | `crates/sifr/src/lint_cli.rs:308`, `crates/sifr/src/lint_cli.rs:496` | CLI lint command reads individual files for linting and probes path exclusion. | Provider-backed for semantic source reads; CLI target filtering remains a documented command-surface probe until M2 classifies it. |
| CLI check/package command reads | `crates/sifr/src/check_and_package_commands.rs:409`, `crates/sifr/src/check_and_package_commands.rs:415`, `crates/sifr/src/check_and_package_commands.rs:427`, `crates/sifr/src/check_and_package_commands.rs:551`, `crates/sifr/src/check_and_package_commands.rs:554`, `crates/sifr/src/check_and_package_commands.rs:583`, `crates/sifr/src/check_and_package_commands.rs:590`, `crates/sifr/src/check_and_package_commands.rs:601` | CLI package/check command surfaces probe targets and cache paths and read package sources for command output. | Provider-backed for semantic source reads; command-output/cache probes remain documented exceptions where non-semantic. |
| CLI entrypoint probing | `crates/sifr/src/cli_model_and_entrypoint.rs:634`, `crates/sifr/src/cli_model_and_entrypoint.rs:690`, `crates/sifr/src/cli_model_and_entrypoint.rs:716`, `crates/sifr/src/cli_model_and_entrypoint.rs:721` | CLI mode resolution reads manifests/source files and probes sibling modules. | Provider-backed for semantic source reads; CLI mode-selection probes remain tracked lookup dependencies. |

Permitted M1 exceptions:

- CLI stdin reads are not workspace identity until later explicitly modeled.
- Generated-output and test-harness reads under tests, verification, and
  generated artifact checks are outside the M2 semantic source-provider scope.
- Codegen intrinsics that emit `std::fs::*` for user programs are not compiler
  service reads.
- Build artifact cache metadata in `crates/sifr_driver/src/build/workspace.rs:219`,
  `crates/sifr_driver/src/build/workspace.rs:282`, and
  `crates/sifr_driver/src/build/workspace.rs:296` is M15
  `.sifrbuildinfo`/build-metadata territory, not M2 source-provider correctness.
- Package projection writes and repair probes in
  `crates/sifr_package/src/projection.rs:100`,
  `crates/sifr_package/src/projection.rs:109`,
  `crates/sifr_package/src/projection.rs:127`,
  `crates/sifr_package/src/projection.rs:129`,
  `crates/sifr_package/src/projection.rs:169`, and
  `crates/sifr_package/src/projection.rs:187` are package-management output and
  repair-state effects unless a later package-aware snapshot milestone promotes
  a specific read into package identity.

## Current Source-Map Guardrail

M0 replaced the old `SourceMapView` stubs. Current guardrail:

- `SourceMapView::text_position_to_span` delegates to
  `sifr_source::SourceText::byte_offset_with_encoding`.
- `SourceMapView::span_to_text_range` delegates to
  `sifr_source::SourceText::range_at`.
- Valid registered source-file conversions return `Some`; invalid files,
  non-boundary positions, and out-of-range spans return `None`.

## Current LSP Reality

`internal_docs/lsp_server.md` describes both implemented Phase 36 behavior and
future compiler-service layers. As of M1:

- `DocumentStore` still owns per-document analysis hosts.
- `DocumentState::rebuild` calls `AnalysisHost::open_single_file` with
  `FrontendMode::SingleFile` on open/change/save.
- The current `RequestQueue` tracks pending request ids and shutdown state only.
- The current `Scheduler` maps request methods to lane labels only; it does not
  yet provide priority queues, cancellation tokens, debounce, background lanes,
  progress, or worker execution.
- Stale-result rejection is revision-token based inside per-document
  `AnalysisHost` use, not `WorkspaceSnapshot`/document-version publication
  identity.

M1-M4 remain serialized. M5 is the first milestone allowed to remove the
request-local LSP host shape by making LSP consume captured workspace snapshots.
M11/M13 own priority queues, cancellation, progress, debounce, and operational
hardening.

## Current LSP Budget Reality

The enforced protocol-level LSP performance gate is still the aggregate
`lsp.request_families` scenario with budget id `perf.lsp.request_families`.
M12 must split this into per-request cases for cold start, diagnostics,
completion, hover, signature help, navigation, references, rename, semantic
tokens, inlay hints, selection range, type hierarchy, code actions, formatting,
generated Rust preview, and workspace diagnostics. Until M12, aggregate LSP
coverage is smoke coverage only.

## Automation

`verification/tooling/check_typescript_go_m1_guardrails.py` validates:

- M1 documentation contains the locked terms, direct-read inventory, current LSP
  limitations, serialized M1-M4 rule, and aggregate LSP budget status.
- `sifr_source` still has bottom-of-graph dependency direction.
- `SourceMapView` no longer has no-op conversion stubs.
- Current LSP single-file rebuild and shallow scheduler/request queue reality is
  visible in code while M1 is active.
- The performance manifest still has the aggregate-only LSP request-family
  budget, so M12 cannot be accidentally claimed early.

### Future Milestone Update Obligations

- M2 must either route every non-exempt inventory row through `SourceProvider`
  or update this inventory with a reviewed exception before closure.
- M5 must update the current LSP single-file rebuild caveat and the matching
  script checks when `DocumentStore` stops owning request-local
  `AnalysisHost::open_single_file` rebuilds.
- M12 must update the aggregate-only LSP budget caveat and the matching script
  checks when `lsp.request_families` is split into per-request scenarios.
- M13 must update the scheduler/request-queue caveats if cancellation tokens,
  progress, or background queues land in the scheduler modules.
- M15 must update the build-metadata exception when `.sifrbuildinfo` or
  equivalent persistent metadata becomes an explicit compiler-service input.
