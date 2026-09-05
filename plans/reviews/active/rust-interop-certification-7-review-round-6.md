# Rust Interop Certification 7 Review — Round 6

- Reviewed commit: `71194c983`
- Base: `origin/main`
- Reviewer: agent, medium effort
- Verdict: finding; not satisfied

## Prior closure

The reviewer confirmed every round-1 through round-5 finding closed, including
direct provenance for all four contract-only positive and negative fixture
directions. It reproduced the full driver library, mandatory generated
packages, Rust-interop validators, Clippy, formatting, guardrails, counts,
safe-Rust audit, and unrelated-path preservation.

## New finding

### Low — copy-fallback assertion did not name the rejected key

All unsupported zero-copy decorator keys shared one generic diagnostic, so the
checked-in `copy_fallback=True` token could drift to a typo or a different
unsupported key without failing the provenance test. Carry the exact rejected
key into both zero-copy and view unsupported-key reasons, assert
`copy_fallback` in the manifest-bound negative test, and assert the legacy
`mutable` view key in its focused regression.
