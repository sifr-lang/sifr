## Findings

**M1 — partially addressed; one residual gap.** The `@rust(<crate_token>` prefix check in `verification/areas/rust_interop/checks/check_fixture_matrix.py:387` matches by `startswith` without a boundary, so a sibling crate whose normalized name starts with the target's name slips through. Concretely, in `tracing.sifr` (crate_token `tracing`), an `@rust(tracing_subscriber.fmt.init, …)` line `startswith("@rust(tracing")` → True; `_rust_bound_function_name` then returns the subscriber's `def` name, and the verifier-call check at line 379 is satisfied if `verify_tracing_package` calls that same wrong function. Five collision pairs in `REQUIRED_CRATES` (after `-`→`_` normalization) are exposed:
- `serde` ↔ `serde_json`, `serde_derive`
- `http` ↔ `http_body`
- `tokio` ↔ `tokio_postgres`, `tokio_tungstenite`
- `tracing` ↔ `tracing_subscriber`
- `tower` ↔ `tower_http`

The fix is small: require the char following `crate_token` in the `@rust(...)` head to be a non-identifier delimiter (e.g., one of `.`, `,`, `)`, `:`, whitespace) — `not (next_char.isalnum() or next_char == "_")`. The current example files all happen to use the correct crate, so the suite passes today; the gap only fires on a future copy-paste mistake, which is exactly what M1 was meant to catch.

**M2 — effectively addressed.** `verify_<crate_token>_package` presence + bound-function call site are both enforced (lines 374–380). Two minor edge cases, neither actionable given current fixture shape:
- Substring match: `f"{bound_function}("` could match a longer suffix-bearing identifier (e.g., `my_foo(` satisfies `foo(`). Not currently realizable in the small fixtures.
- The "verifier body" is the entire file tail from the `def verify_…` line onward, not just that function's block; a stray call in a later definition would satisfy the check. Again not realizable in the current 1-function-per-file shape.

If M1 is tightened to require a boundary character, these residual M2 edges become even less reachable, and I would not block on them.

**URL literals — fixed.** Verified `redis://` (both fixtures), `postgres://` (opaque_resource_matrix), and `ws://` (callback_subscription_matrix) are in place.

## Summary
One actionable finding remains: tighten the `@rust(<crate_token>` match in `_rust_bound_function_name` to require a delimiter after the crate token, closing the sibling-crate bypass for the five collision pairs above. Everything else in this follow-up looks good.
