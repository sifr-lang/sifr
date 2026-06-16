# Ecosystem Compatibility Limitations

This document records what the pinned local ecosystem corpus represents and what it intentionally does not claim.

## Represented Project Types

- Small command-line style entrypoints with deterministic stdout.
- Multi-module local-import projects.
- Collection and control-flow heavy examples that exercise core language lowering.
- Known-failure signal cases that produce structured compiler failures without internal panics.

## Absent Project Types

- Large public repositories with external package graphs.
- Projects that require live registry access, Git fetches, network services, databases, or shell installers.
- GUI, notebook, browser, and long-running daemon projects.
- Cross-language package ecosystems that require Python, Node, or system package manager resolution at test time.

## Host And Platform Limits

- The merge-owned curated corpus is local and host-independent except for the Rust/Sifr toolchain used by the normal validation lane.
- The broader signal lane is still local by default; live-network ecosystem checks belong only in nightly or release profiles.
- Host-specific behavior belongs in `runtime_platform`, not this corpus.

## Dependency And Network Limits

- Merge and create-pr ecosystem checks must not require live network.
- Pinned local fixtures may model package or import patterns, but they do not prove registry behavior.
- Package-manager resolution, lockfile determinism, archive checksums, and offline registry behavior are owned by `package_management`.

## Unsupported Package-Manager Scenarios

- Live registry yanks, index updates, authentication, and cache eviction are outside this corpus.
- Transitive package graph edge cases are represented only if they are explicitly mirrored as local fixtures.
- A pass here does not imply package publish, vendor, or install correctness.

## Known False Negatives

- Small local fixtures can miss scale-sensitive regressions that appear only in large projects.
- Expected known-failure cases prove diagnostic stability only for the specific failure surface they encode.
- Runtime stdout checks here are intentionally coarse; detailed semantic parity belongs in core, stdlib, and algorithmic compatibility suites.

## Adding Or Removing Projects

- Add a project when it covers a distinct language, workspace, or compatibility behavior not already represented.
- Every entry must include owner, rationale, SPDX license, pinned revision, source checksum, commands, timeouts, and expected classification.
- Local first-party fixture `LICENSE` files are SPDX marker files locked by checksum; imported third-party corpora must preserve upstream license text and attribution metadata.
- Remove or replace a project when it duplicates coverage, becomes flaky, depends on unsupported host state, or no longer matches the documented compatibility goal.
- Any source edit must update `source_checksum_sha256`; any new project root must be pinned after the root has commit history.
