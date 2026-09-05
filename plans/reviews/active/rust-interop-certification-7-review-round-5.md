# Rust Interop Certification 7 Review — Round 5

- Reviewed commit: `c48034f21`
- Base: `origin/main`
- Reviewer: agent, medium effort
- Verdict: findings; not satisfied

## Prior closure

The reviewer confirmed every round-1 through round-4 implementation finding
closed. In particular, both positive fixture sources now return their declared
opaque handles and their manifest-bound tests lower and validate the exact
checked-in sources. Generated bridge-path recognition is owned by codegen and
the full driver, area validators, guardrails, inventory, and unrelated-path
checks remained green.

## New findings

### 1. Medium — zero-copy bytes negative provenance used a synthetic source

The checked-in `copy_fallback_rejected` source omitted both
`copy_fallback=True` and the paired view decorator, so it failed for an
unrelated missing-view reason. Its manifest-bound test injected copy fallback
into the synthetic unit-test source instead of reading the fixture. Make the
fixture express a complete opaque view contract with explicit copy fallback
and bind the test directly to that file.

### 2. Low — mutable-alias negative provenance used a synthetic source

The checked-in `mutable_alias_rejected` source was semantically aligned, but
its manifest-bound test mutated the synthetic unit-test source instead of
lowering the fixture. Bind the test directly to the checked-in negative source
so future drift is observable.
