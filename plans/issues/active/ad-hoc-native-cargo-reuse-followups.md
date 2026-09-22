# Native Cargo reuse follow-ups from DX.9

Status: active follow-up work; no DX.9 acceptance blocker.
Owners: driver native/probe/test orchestration, runtime Python, storage/platform support.
Source: [read-only Opus review of candidate 2f1f3fb215d5263c0d1fb49d8c3346aafa299b33](https://github.com/sifr-lang/sifr/pull/3856#issuecomment-5723374425).

These are separate maintenance suggestions, not new DX.9 requirements.
No implementation belongs to the phase-record update.

| ID | Owner | Bounded follow-up |
| --- | --- | --- |
| DX9-F1 | Driver probes | Remove the unused successful probe receipt, or define its explicit observability purpose. Cargo freshness remains mandatory. |
| DX9-F2 | Driver native capture | Remove the redundant caller-style mirror from the cached build flow, or key it by root identity. The existing family lease prevents concurrent aliasing and finalized capture uses the Cargo artifact directly. |
| DX9-F3 | Test orchestration | Canonicalize test cache scopes and reconcile native family inputs with the test runner's actual environment/resolution contract. Current non-Python test behavior is not an omission requiring new Python test support. |
| DX9-F4 | Native capture / Python runtime | Measure whole-file identity/copy and interpreter-library hashing cost; investigate streaming or mapped reads while preserving byte-content validation. Metadata-only freshness must not replace that guarantee. |
| DX9-F5 | DX.3 storage owner | Define a bounded lease wait and useful ownership diagnostic for wedged sibling processes, consistently across existing storage primitives. |
| DX9-F6 | Platform support | Handle Windows storage, DLL capture and Python loader qualification together if Windows support is explicitly scoped; the current driver storage is already Unix-only. |

Existing metadata and feature-enabled Python Clippy errors remain owned by
[the metadata follow-up issue](ad-hoc-dx-metadata-review-followups.md) and
[the Python qualification issue](ad-hoc-python-interop-qualification-dependencies.md).
DX.9's supplemental run found 498 pre-existing errors and zero errors in changed
source files. The full phase-end gate remains outstanding.

## DX9-F7 generated native root lock drift — closed 2026-09-22

Owner: driver native storage and Cargo resolution. Retained emitted-Rust Item 12 is
blocked on this externally owned cache contract; no DX implementation is included
in the Item 12 candidate.

The Item 12 native 411-case run on compiler `b27322843de1a49a8a6e9c42f633ce94ffa7223daa4aaf681e6ac9f24724d7c4`
and source candidate `e7fe5cf1f19a7a09bc70a9ba8a1153000f9baf9e` passed its first 65
programs. `0071_simplify_path.sifr` then failed `SIFR-RUST-CARGO-0001` before
execution. The owned DX.9 family root at
`target/sifr-cache/native/families/1a4a2afa0ddbdd835915cc99de2bdb82d65d20fddfd2e23457ca78fe3148a29f/roots/e56034eea35ef70c419f3ad96110cba267f1e8dea310912ab1c6e9783734eb1a`
had a generated `Cargo.toml` with no dependencies while its retained `Cargo.lock`
still listed `sifr_runtime`. Moving that obsolete lock to external evidence let
`0071` pass. The next program, `0072_edit_distance.sifr`, changed the same root
manifest to require `sifr_runtime`; its lock then contained no dependencies and
failed with the same diagnostic. Moving that second obsolete lock let `0072`
pass. The family metadata binds `owner_scope` to the Item 12 worktree. No other
session owned or used this family during the observations.

The editable root identity is keyed by scope and project name, while generated
manifest dependencies change among programs in that scope. Constrained Cargo
resolution only prepares a lock when none exists, so the retained lock and new
manifest disagree. The DX owner should reconcile lock preparation with changes
to the generated manifest under the family lease and add a regression that
alternates dependency-free and runtime-dependent programs in one scope under
locked resolution. Preserve Cargo target warmth and the no-stale-executable
failure rule. This is a diagnosis and requested owner action, not an accepted
Item 12 workaround or a claim of full native qualification.

Evidence: `/home/yaser5/projects/sifr/emitted-rust-retained12-evidence/algorithmic-native-compiler43-full411.log`,
`native-full-1790108632432039626/native-matrix.json`, both `0071`/`0072`
failed logs, `0071-stale-generated-Cargo.lock`, `0072-stale-generated-Cargo.lock`,
and both `0071-cache-repair.log`/`0072-cache-repair.log`. The failed 411 run
remains failed; no broad gate or review was used.

Resolution: [PR #3903](https://github.com/sifr-lang/sifr/pull/3903) merged as
`439b5ebd5c6917872f58288453917a2427509192` from final implementation candidate
`c76312b894d86218ebcd6b27ce43fa3271388077`. The generated lock now records
the prepared resolution identity and lock digest. A manifest or authority change
restores or prepares the matching lock while the native family lease is held;
the family Cargo target is retained. The existing no-stale-executable rule is
unchanged.

Acceptance evidence: the leased same-root locked alternation regression and
six adjacent native reuse tests passed (7/7), and Cargo resolution tests passed
(15/15). Logs: `/home/yaser5/projects/sifr/dx9-f7-evidence/native-reuse-tests.log`
and `cargo-resolution-tests.log`. Formatting, file-size, driver maintainability
and diff checks passed. The [scoped Opus review](https://github.com/sifr-lang/sifr/pull/3903#issuecomment-5784211177)
returned SATISFIED with no blocking findings on the final candidate.

Deferred review suggestions, outside DX9-F7 acceptance: measure actual target
artifact reuse more directly in the test, evaluate a markerless prepared-lock
comparison, and profile added manifest/authority hashing on hot builds.

## DX9-F6 actual automatic CI evidence — 2026-09-20

Restored workflow admission reaches Windows native SQL qualification and exposes
55 driver errors from the existing Unix-only storage/process APIs, after all
16 component tests pass. Exact evidence, platform contracts and the separately
scoped acceptance criteria are owned by
[Windows driver portability](ad-hoc-windows-driver-portability.md).
The CI admission fix does not disable the Windows job or claim it passed.
