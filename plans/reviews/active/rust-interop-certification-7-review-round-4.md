# Rust Interop Certification 7 Review — Round 4

- Reviewed commit: `08f813889`
- Base: `origin/main`
- Reviewer: Claude Opus 5, medium effort
- Verdict: findings; not satisfied

## Prior closure

The reviewer reproduced the full driver library, workspace Clippy, formatting,
file-size and maintainability guardrails, all Rust-interop validators and
self-tests, inventory counts, safe-Rust runtime source, and unrelated-path
preservation. It confirmed every round-1 through round-3 finding closed.

## New findings

### 1. High — two supported positive fixtures had drifted from the exact view contract

The `zero_copy_bytes` and `zero_copy_view_matrix` positive sources still
returned `bytes` while their `view=` declarations named opaque Rust view
types. Their provenance tests used a separate synthetic handle-returning
source, so the advertised fixture sources were never lowered and validated.
Make both fixtures return their declared opaque handles and bind the existing
manifest-owned tests directly to the checked-in fixture sources.

### 2. Low — generated bridge-path recognition duplicated codegen policy

The driver parsed the generated bridge module prefix, segment count, and
`Bridge` suffix locally. Move canonical and legacy generated-path recognition
beside codegen's path renderer, reject containers, aliases, and invalid Rust
identifiers there, and have the driver call that shared helper.
