# agent Review: TypeScript-Go Prerequisite Discovery

Date: 2026-05-29

Command:

```bash
{ cat reviews/ad-hoc-typescript-go-prerequisite-discovery.md; printf '\n\nReview the decisions above as an independent senior compiler/LSP reviewer. Focus on blockers before implementation starts. Return: verdict, blocking findings, important non-blocking findings, and any ordering changes. Keep it under 120 lines. Do not edit files.\n'; } | agent review ""
```

## Verdict

agent found the contract direction sound, with review findings to fold into the phase docs before implementation approval.

## Findings

Blocking findings:

- Dirty-scope and module-signature absence needed explicit repo-search evidence, not only narrative.
- Direct filesystem reads needed concrete file/line citations instead of a broad summary.
- Scheduler/cancellation limitations should be treated as a blocker for any concurrent/background work before captured snapshot identity exists.

Important non-blocking findings:

- `AnalysisHost::update_document` exists but the LSP does not use it as a long-lived workspace service; the plan should call out that this reverses current LSP behavior.
- Symbol ranges currently being `None` should be tied directly to the M0 source-map prerequisite.

Ordering feedback:

- M0 -> M1 -> M2 -> M3 -> M4 is defensible.
- Event compaction is not a phase-start correctness blocker while the server remains synchronous, but it becomes required before asynchronous scheduler behavior or fine-grained watcher invalidation.

## Incorporated Changes

- Added explicit no-match evidence for `DirtyScope`, `ModuleSignature`, `ExportSignature`, `can_replace_module_in_project`, `CowProjectState`, `WorkspaceSession`, and `WorkspaceSnapshot`.
- Added direct filesystem read examples with file/line references for frontend, driver, lint, format, and package paths.
- Updated scheduler wording so M1-M3 remain serialized unless snapshot identity and cancellation tokens move earlier.
- Reframed event compaction as conditional on async scheduling/fine-grained watcher invalidation.
- Tied symbol/navigation range correctness to M0 source-map completion.
