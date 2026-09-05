# TypeScript-Go Architecture Transfer: Project Residency, Watchers, And Build Info

project-residency surface keeps long-lived compiler-service sessions bounded without making generated
metadata authoritative.

## Residency

`WorkspaceSession` snapshots now carry `WorkspaceResidencySnapshot`. The
snapshot records project residency entries, config registry entries, deduped
watch registrations, and verified build-info metadata.

Project residency states are represented by `ProjectResidencyKind`:
`OpenFileOwner`, `AncestorSolution`, `ReferencedByOpenProject`,
`ExplicitApiOpen`, and `Evictable`. `WorkspaceSession::project` remains lazy:
it creates evictable state before loading, while `open_project`/`reload` marks
the explicit API project as loaded.

## Watchers And Configs

Watch registrations are deterministic strings with ref counts and
`WatchRegistrationReason` values. They are derived from seen files,
directories, package roots, config files, stdlib roots, generated artifacts, and
failed lookups. Config registry entries track reverse retention through ref
counts plus a `pending_reload` flag.

Closing an overlay releases its open-file residency and re-derives
watcher/config snapshot state from the remaining retained session inputs.

## Build Info

`SifrBuildInfoCandidate` models `.sifrbuildinfo` as non-authoritative metadata.
`WorkspaceSession::verify_build_info` accepts it only when the current compiler
fingerprint, package/config identity, and source hashes match the active
workspace. Rejected metadata clears the retained build-info snapshot and never
hides source, config, package, or compiler-option changes.
