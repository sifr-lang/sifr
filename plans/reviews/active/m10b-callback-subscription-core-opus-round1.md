## Verdict: PR-ready

No blocking ownership or overclaiming issues. The row split is internally consistent across all six artifacts, and every claim in the stated intent is backed by the data.

### Ownership — clean, no double-claim
- `callback_subscription_core` (supported, stdlib) is owned by the **`_sifr.signal`** closing surface (`certification_rows = ["callback_subscription_core"]`), whose reason explicitly records the M10b split and the stdlib routing through `sifr_stdlib.signals`.
- `callback_subscription_ecosystem` (future-owned) is owned by the **`_sifr.python`** retained surface and points its `future_owner` at the certification issue.
- The two rows are owned by *different* surfaces — no row is claimed by both a supported/closing and a future-owned surface. Both matrix rows are covered by a surface's `certification_rows`, so the gate's coverage requirement holds.

### Overclaiming — none in the matrices
- Core row: `required_crates: []`, notes say "…without claiming ecosystem subscription crates." The three ecosystem crates (tokio-tungstenite, redis, notify) live **only** on the ecosystem row, with evidence `status: "planned"` and `category: future-owned-by-separate-phase`. Scope boundary is respected.
- Compatibility matrix and fixture matrix rows agree on id, tier, `execution_kind`, `required_crates`, and evidence ids/status. Core manifest (`diagnostic_family: SIFR-RUST-CB-0001`) matches the negative fixture's `expected-diagnostic`.
- The renamed directory (`callback_subscription_matrix → callback_subscription_ecosystem`) is consistently referenced — the manifest uses the new `callback_subscription_ecosystem` name and the ecosystem crate examples (notify/redis/tokio-tungstenite) moved into that fixture; core has `package_examples: {}` / `features: {}`.
- Gate self-test seeds both core (SUPPORTED, passing/passing) and ecosystem (FUTURE_OWNED, future_owner) rows, and `callback_subscription_core` is in the `SUPPORTED_STDLIB_CORE_ROWS` allowlist — matching the real matrix categories.

### Non-blocking observations (not fixes required for merge)
1. **Row label vs. notes nuance.** The core row's `execution_kind` is a single `runtime-observed`, but the notes/intent describe shutdown as *contract* evidence (only subscribe + `aclose` cancellation actually run; `shutdown=drain` is a policy declaration, and the positive fixture never delivers a real signal). This is accurately disclosed in the notes, so it isn't an overclaim — just be aware the "runtime-observed" label covers the subscription/cancel lifecycle, not shutdown drain execution.
2. **Negative fixture isolation.** `invalid_subscription_callback_policy_rejected` differs from the positive in more than the missing `shutdown=drain` (also `panic=trusted_no_panic`, handler/return types `None` vs `Result`). The `SIFR-RUST-CB-0001` family is callback-subscription-specific so it plausibly fires on the shutdown-policy gap, but the negative doesn't *isolate* that as the sole trigger. Worth a follow-up to tighten, not a merge blocker.

Given create-pr validation, the rust_interop area, and the sysroot certification gate all passed, this is safe to open.
