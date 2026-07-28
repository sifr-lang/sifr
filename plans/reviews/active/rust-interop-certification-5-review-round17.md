# Rust Interop Certification 5 Review — Round 17

## Verdict

**SATISFIED**

## Reviewed revisions

- Head: `16d794fc3516f9cf560acdd0771771569770b26e`
- Base: `f9837adb105f048ed56624c148ee83ecbd2a3d03`
- PR: `#3042`
- Diff: 90 files, 6,456 insertions, 397 deletions

The merge base equals the reviewed base exactly.

## Findings

None actionable.

## Round 16 finding disposition

1. **Callback subscription positive fixture ownership — closed.**
   `callback_subscription_ecosystem/positive/subscription_cancel_shutdown.sifr`
   now consumes `own subscription`, matching the sibling lifecycle fixtures.
2. **Three passing negative fixtures reached stale close/config errors —
   closed.** All three now use owned, fallible cleanup declarations. The
   reviewer type-checked all touched fixtures with a freshly built compiler
   and observed zero stale configuration or ownership diagnostics. Each
   fixture can therefore reach its declared cargo-probe or runtime-observed
   evidence stage.
3. **Authoritative merge evidence was not pinned — closed and independently
   corroborated.** The plan pins exact base `f9837adb10` and records 57/57
   distribution, 50/50 generated builds, 674/674 E2E, and 261/261 hardening.
   The reviewer independently reproduced all five distribution suites:
   57 variants, zero failures and zero blocking failures.
4. **Non-union poisoned-state fallback lacked codegen coverage — closed.**
   `emitted_opaque_self_method_maps_poison_to_plain_declared_error` uses a
   plain `ResourceError`, asserts the poisoned-handle mapping and message
   conversion, and asserts that no panic-error union variant is emitted.

## Correctness assessment

- Consuming receiver metadata is set during lowering, retained in Rust interop
  IR, propagated through imports and re-exports, and enforced at call sites.
- Opaque close-policy validation requires exactly one matching Rust-bound
  owned cleanup member, rejects other consuming methods, and fails closed.
- Panic payloads are unconditionally discarded by the runtime bridge. The
  fallback conversion can expose only the stable `Rust bridge panicked`
  message.
- Resource, tracked-task, and temporary-database cleanup is deterministic.
  Joins are timeout-bounded and failures become typed errors rather than
  panics.
- The only new bridge panic is the deliberate redaction probe inside
  `catch_unwind`; the remaining `unreachable!` is a local programmer
  invariant.
- Declared error mapping initializes every declared string field, including
  multi-field message errors.
- The diff is scoped to Certification 5. The unrelated modified
  `editor_integrations` submodule and untracked `.cert5probe/` directory are
  not part of the diff.

## Independent validation

- Eleven touched fixture type-checks: clean.
- Distribution release merge suites: 57/57, zero failures.
- Rust interop area: 10/10 variants, zero failures, 28 stable claims.
- `cargo test -p sifr_lowering -p sifr_codegen -p sifr_runtime`: zero
  failures.
- Targeted opaque tests: 5 passed, zero failed.
- File-size guardrail: 2,908 files, all within 900 lines.
- HIR maintainability guardrail: passed.

## Residual risks

All residual risks are non-blocking:

- The reviewer did not independently rerun the full 674-fixture E2E and
  261-variant hardening suites; the author-owned exact-base authoritative run
  records both as passing.
- The future-owned callback subscription positive fixture is not yet included
  in the lowering guard test; that is coverage polish for Certification 6,
  not a gap in Certification 5.
- The exact `rusqlite 0.39.0` pin constrains both the new and a prior certified
  row; its stable-toolchain rationale is recorded in the plan.
- Fixture-to-test linkage remains convention-based in the existing evidence
  model.
