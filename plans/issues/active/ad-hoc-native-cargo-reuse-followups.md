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
