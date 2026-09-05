# TypeScript-Go Architecture Transfer: Trace And Status Surfaces

trace/status normalizes compiler-service trace and status output so stale requests,
cache state, invalidation, and editor-facing readiness can be explained from a
single snapshot.

## Trace

`WorkspaceTracePhase` defines the deterministic trace-stage vocabulary:
`SourceUpdate`, `Parse`, `Lower`, `TypeCheck`, `Ownership`, `Flow`, `Cache`,
`Invalidation`, `Scheduler`, `Cancellation`, `StaleRejection`, and `LspTiming`.
`WorkspaceSession` records source updates, compiler-stage summaries,
invalidation details, cache summaries, and stale rejections into
`WorkspaceTraceLog`. Workspace trace retention is bounded to the newest 256
events.

LSP request scheduling, cancellation, stale diagnostic rejection, and timing
markers use the same trace-stage vocabulary in the LSP session trace buffer,
which is also bounded to the newest 256 events and exposed through the custom
`sifr/debugTrace` request. Analysis snapshot stale rejection records the
captured/current workspace and graph/source revisions before returning
`StaleSnapshot`.

## Status

`WorkspaceDebugSnapshot` combines `WorkspaceStatusSnapshot` with the trace log.
Status output includes open-file, project, source, module, dependency, cache,
index-readiness, last-update latency, watcher/config, build-info, and retained
source-byte counters. The memory counters are deterministic local counts, not
telemetry or heap introspection.

`AnalysisHost::debug_snapshot` enriches frontend status with
`SymbolBucketReadiness` for workspace/package/stdlib index buckets without
building the symbol index on demand. If the index has not been built yet, the
status reports unavailable readiness. LSP analysis open/update paths feed
last-update latency counters into workspace status. `sifr trace <file>` prints
a representative CLI trace/status snapshot for project or single-file inputs.
