# TypeScript-Go Architecture Transfer M15: Project Residency, Watchers, And Build Info

Status: in progress

M15 keeps long-lived compiler-service sessions bounded without making generated
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

## Validation

- `cargo test -p sifr_frontend workspace_session` -> PASS, 8 tests
- `cargo test -p sifr_frontend` -> PASS, 42 tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_frontend -- -D warnings` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED (`reviews/typescript-go-m15-project-residency-review-pass-1.md`)
- Claude reviewer pass 2 -> SATISFIED (`reviews/typescript-go-m15-project-residency-review-pass-2.md`)
- Claude reviewer pass 3 -> SATISFIED (`reviews/typescript-go-m15-project-residency-review-pass-3.md`)
- `scripts/run_all_tests.sh --profile quick` -> PASS, report `target/validation_lane_reports/quick.latest.json`, wall time 292.53s, advisory: group skew is high
