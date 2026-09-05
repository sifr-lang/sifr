# Rust Interop Certification 7 Review — Round 1

- Reviewed commit: `7fb5c3d6d`
- Base: `origin/main`
- Reviewer: agent, medium effort
- Verdict: findings; not satisfied

## Verified

The reviewer independently passed all three mandatory generated-build tests,
the full 10-variant Rust-interop area, all five checkers and self-tests,
formatting, workspace Clippy, and both maintainability guardrails. It confirmed
that the negative diagnostics precede Cargo, the obligation keys align, the
bytes/memmap observations are self-verifying, release is deterministic, the
bridge is unsafe-free, the package lock is a root-lock subset, and all
documented inventory counts are accurate.

## Findings

### 1. High — file-size guardrail failure

`verification/areas/rust_interop/checks/_scenario_checks.py` reached 920 lines,
so the authoritative file-size guardrail fails. Move the zero-copy token table
and safe-Rust scan into the responsibility-specific `_scenario_zero_copy.py`.

### 2. High — delimiter-unaware view/return matching

`zero_copy_validation.rs` used substring containment over a rendered return
type. That accepts prefix aliases such as `Bytes` versus `BytesView`, and
container returns such as `list[CrateBackedView]`. Compare the exact Ok-slot
opaque handle type structurally (or by exact canonical rendering), diagnose
missing return renderings, and add regressions for both false-negative forms.

### 3. Medium — fragile Send/Sync diagnostic classification

Classification depended on rustc echoing `__SifrView` in a source snippet.
Renderer changes can therefore fall through to `SIFR-RUST-TYPE-0001` and leak
raw rustc output and temporary paths. Classify from the owned zero-copy
obligation state plus trait-failure phrases and suppress raw stderr for the
recognized diagnostic.

### 4. Medium — probe-plan assertions do not cover enforcement source

The unit tests asserted digest/cache metadata, while the emitted direct probe
source had only one ignored integration direction. Add direct unit coverage
for all four Send/Sync obligation combinations.

### 5. Medium — `zerocopy[derive]` policy is not frozen

The matrix declared the feature, but `EXPECTED_FEATURE_POLICIES` and the
catalog did not. Bind `zerocopy = ["derive"]` in the feature-policy inventory
and exact catalog entry, align the contract-only zero-copy row, and mutation
test removal/drift.

### 6. Low — weak bytemuck/zerocopy observation and mutation coverage

The pointer checks borrowed the same objects and could not demonstrate a
falsifiable parse. Reinterpret the sealed mmap through bytemuck and use
`Packet::ref_from_bytes` through zerocopy, validating both address and values
after a pre-seal mutation. Extend scenario mutations for the crate-view and
release tokens. Scope docs to the buffer received by the bridge and label
compile-time obligations as type-probed rather than runtime-observed.

## Reviewer conclusion

Findings 1 and 2 block closure. The final certification checklist correctly
remains open.
