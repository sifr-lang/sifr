# Claude Review: TypeScript-Go Architecture Decisions

Date: 2026-05-29

Scope: iterative Claude review of the locked architecture decision section in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`.

## Rounds

1. Initial decision review found missing contract details for the new `sifr_source` dependency position, `DirtyReason`, and LSP overlay transfer into snapshots.
2. Follow-up review clarified that the document is a planning contract, not implementation closeout.
3. Planning-contract review found remaining gaps in flow graph shape, budget harness verification, cancellation propagation, dirty-scope merge priority, and `sifr_source` migration blast radius.
4. Final decision review confirmed those gaps were resolved in the contract, with per-request budget implementation correctly left as phase work.

## Incorporated Changes

- Locked `sifr_source` as the bottom dependency crate for source text, line maps, encodings, spans, and source-file metadata.
- Defined `DirtyReason`, dirty-scope merge priority, and `Unknown`/`WatcherStorm` degradation.
- Specified LSP overlay transfer through explicit `WorkspaceSession` document-event/update-overlay APIs before snapshot capture.
- Added cancellation token API shape and required phase-boundary checks.
- Added minimum flow graph shape and flow cache invalidation coupling.
- Added cache fingerprint types and deterministic cache-key requirements.
- Added `sifr_source` migration sequence across syntax, diagnostics, frontend, LSP, and performance benchmarks.
- Made per-request LSP budget harness requirements explicit.
