# Windows driver portability blocks native SQL CI qualification

Status: open; separate platform scope, not implemented by CI admission repair
Owner: driver storage/process execution and Windows platform support
Related: [DX9-F6](ad-hoc-native-cargo-reuse-followups.md)

Restored automatic CI exposes an existing compiler-platform failure. On candidate
`f8164f788185e40cef32f5a8c2b7c4e5dc4a748b`,
[automatic run 35532810784, Windows job 106136353671](https://github.com/sifr-lang/sifr/actions/runs/35532810784/job/106136353671)
passes all 16 native compiler-component tests, then actually executes the locked,
offline native SQL build qualification and fails compiling `sifr_driver` with
55 errors. The admission/provisioning fix changes no driver source or lockfile.
This is not a workflow admission failure or missing Cargo dependency.

The first failures are `std::os::unix` imports in
`build/rust_interop_probe_cache.rs:55`, `cache_storage.rs:4`,
`metadata_producer/ensure.rs:3`, `process_execution.rs:3`,
`project_cache/housekeeping.rs:9`, and `project_cache/storage.rs:12` under
`crates/sifr_driver/src`. Further failures include Unix file descriptors,
`libc::O_NOFOLLOW`, file modes, inode/device identity, and process-group APIs.
The complete log is preserved outside Git at
`/home/yaser5/projects/sifr/ci-admission-evidence/f8164f788185e40cef32f5a8c2b7c4e5dc4a748b/windows.log`.

A separate approved implementation must define the Windows equivalents for
private storage permissions, symlink/reparse-point protection, file identity,
lease lifetime, atomic publication and process-tree termination. Preserve the
existing safety contracts; do not merely cfg out checks, skip Windows jobs,
or add permissive fallbacks. Scope should include each driver module above and
its owning helpers, while excluding unrelated compiler semantics and lint debt.

Acceptance requires focused Windows storage/process/metadata tests (including
failure, concurrent ownership, cleanup, and descendant lifetime cases), the
unchanged 16 component tests, and actual Windows SQL clean/incremental/locked/
offline/reproducible qualification. Reuse unaffected Unix evidence and rerun
affected Unix contracts to prove platform factoring preserves their behavior.
No Windows implementation is authorized by recording this issue.

Final admission candidate `05c72e32a2b66fc1414929e40b59f6c6e619e454`
reproduced exactly 16 component passes and 55 driver errors in
[run 35533124294, job 106137284398](https://github.com/sifr-lang/sifr/actions/runs/35533124294/job/106137284398).
Its log is under the matching candidate evidence directory as `windows.log`.
The user explicitly chose to record Windows separately on 2026-09-20; no
portability implementation or disabled CI coverage belongs to the admission fix.

## 2026-09-23 SQL Item 3 CI retriage and delivery split

[SQL Item 3 PR #3975](https://github.com/sifr-lang/sifr/pull/3975), merged as
`8bd81fcddd2226daf98a38c6e58e32a4d4ae4680`, used candidate
`ddce80ca0655bc5b724d585a092b1de61a0260fe` again passed all 16
compiler-component cases, then failed the Windows native SQL build while
compiling `sifr_driver` with 55 errors in
[run 35914571116, job 107362677007](https://github.com/sifr-lang/sifr/actions/runs/35914571116/job/107362677007).
The exact diagnostic inventory is:

| Driver source under `crates/sifr_driver/src` | Errors | Contract exposed |
| --- | ---: | --- |
| `cache_storage.rs` | 21 | Private ownership, no-follow entry access, inherited leases, available space. |
| `process_signals.rs` | 9 | Scoped cancellation and signal-handler lifetime. |
| `project_cache/housekeeping.rs` | 7 | Workspace file identity, safe pruning and generation leases. |
| `project_cache/storage.rs` | 5 | Private no-follow file creation/read and generation publication. |
| `metadata_producer/ensure.rs` | 5 | Private staged metadata and atomic output publication. |
| `process_execution.rs` | 5 | Owned process tree, cancellation and deadline termination. |
| `build/rust_interop_probe_cache.rs` | 3 | Private probe marker publication. |

The earlier six-area inventory omitted `process_signals.rs`. PR #3975 changed
none of these seven files. The Unix-only operations in
`project_cache/storage.rs` predate the N02c/F08 serialization change: that
change only passed the already serialized record bytes to `publish`. Therefore
the 55 errors are the existing Windows driver portability failure, not an N02c
serialization regression. The [PR #3975 blocker comment](https://github.com/sifr-lang/sifr/pull/3975#issuecomment-5802776564)
records this attribution; no Windows fix SHA or passing SQL Windows
qualification exists from this retriage.

The 2026-09-23 assignment authorizes a safety-preserving Windows remediation.
The W1 storage and W2 process contracts below remain distinct ownership areas,
but their implementation and native acceptance must share one coupled W1+W2
candidate because the real Windows crate compiles both areas unconditionally.
Retain the existing acceptance above:

1. **W1 — Windows secure storage.** Implement the shared Windows equivalents
   across `cache_storage.rs`, `project_cache/housekeeping.rs`,
   `project_cache/storage.rs`, `metadata_producer/ensure.rs` and
   `build/rust_interop_probe_cache.rs`. Enforce private owner/ACL checks,
   reject symlinks and all reparse-point traversal, preserve lock identity and
   descendant lease lifetime, and publish staged files and generations
   atomically with required durability. Test malicious aliases, foreign or
   permissive ownership, concurrent readers/writers/GC, abandoned stages,
   failure injection and intact published winners on Windows. Rerun affected
   Unix storage contracts. A compile-only `cfg` branch or permissive fallback
   does not satisfy W1.
2. **W2 — Windows process ownership.** In the same candidate as W1,
   implement Windows cancellation and process-tree lifetime in `process_signals.rs` and
   `process_execution.rs`. Preserve bounded output, deadline and cancellation
   causes, partial output, normal user-program streaming, and cleanup of
   descendants holding pipes. Run focused Windows cancellation, timeout,
   descendant, cleanup and concurrency cases plus affected Unix process cases.
3. **W3 — integrated Windows qualification.** After the coupled W1+W2
   candidate passes its named native Windows storage, metadata and process tests
   plus affected Unix storage and process contracts, run the unchanged
   16 compiler-component tests and actual Windows native SQL clean,
   incremental, locked, offline and reproducible qualification on the exact
   candidate. Record the Windows job, candidate SHA and raw outcome; rerun
   affected Unix contracts. Only a passing W3 can clear this SQL CI blocker.

The Linux execution host had 20 GiB free with another session actively using
its separate Cargo target at retriage. It had no installed Windows Rust target
and no Windows runtime for the required safety tests. No target cleanup or
cross-target build was attempted, and the SQL owner’s target was untouched.

## 2026-09-23 W1 native-test dependency blocker

W1 was inspected on `origin/main` `9da86125c` in an isolated worktree, with no
implementation changes or passing W1 claim. The available native Windows host is
the `windows-2025` job in `.github/workflows/local-first-validation.yml`. That
job compiles the complete `sifr_driver` crate before its storage and metadata
tests can execute. `crates/sifr_driver/src/lib.rs` unconditionally includes both
`process_execution.rs` and `process_signals.rs`, which still contain the 14
Windows compile errors assigned to W2 in the SQL Item 3 inventory above.
Consequently, even a complete W1-only storage patch cannot compile the real
crate on Windows or execute the required W1 malicious-alias, owner/ACL,
concurrent lease/GC, abandoned-stage, failure and winner cases. A standalone
`cache_storage` harness would leave the project-generation, metadata and probe
publication paths untested; stubbing W2 in the product would not satisfy the
safety contract.

The Linux host has no Windows runtime or installed Windows Rust target. At this
audit it had 16-18 GiB free and no active Cargo process; no target was cleaned,
and the SQL worker's target was untouched. W1 is **needs dependency/scope
adjudication**, not implemented or validated. The owner must authorize either
W2 process portability before W1's native acceptance, or one coupled W1+W2
Windows candidate with bounded reviews before W3. Keep the Windows SQL CI
blocker attributed to driver portability until W3 passes on an exact candidate.

## 2026-09-23 W1+W2 coupled implementation rescope after PR #3980

[Merged PR #3980](https://github.com/sifr-lang/sifr/pull/3980) recorded a
**failed W1-only validation dependency**, not a storage implementation or a
Windows pass. Its candidate `987dd680f254a3bddcbbe65eb36f62c15491492b`
merged as `d54d333dd265c7c40916d4548e6d747d8082b762`. The W1-only
blocker above remains historical evidence. This rescope selects the coupled
option from that adjudication. `sifr_driver::lib` includes the
storage and process modules without Windows cfg gates, so neither W1-only nor
W2-only code can compile the real crate and execute native acceptance while
the other area retains its Windows errors.

The next implementation item is **one coupled W1+W2 candidate** covering all
seven driver source areas in the SQL Item 3 inventory and their owning helpers.
It must preserve the complete W1 private ownership/ACL, no-reparse traversal,
identity/lease, atomic publication and durability contract, and the complete W2
process-tree, cancellation/deadline, bounded capture, partial-output, normal
streaming and descendant cleanup contract. Review the storage and process
changes within this candidate as bounded areas; do not declare either area
accepted until the real crate and both sets of native tests execute. Run W1's
Windows malicious-alias, owner/ACL, concurrent reader/writer/GC, abandoned-stage,
failure-injection and published-winner tests; run W2's Windows cancellation,
timeout, descendant, cleanup and concurrency tests; rerun affected Unix storage,
metadata and process contracts. A cfg exclusion, stub or permissive fallback
does not satisfy either contract.

**W3 remains a separate integrated qualification item** after the coupled
implementation candidate. It runs the unchanged 16 compiler-component tests and
the actual Windows native SQL clean, incremental, locked, offline and
reproducible qualification on the exact candidate, records the job, SHA and raw
outcome, and reruns affected Unix contracts. Neither #3980 nor this docs-only
rescope supplies a Windows test pass or clears the SQL CI blocker.
