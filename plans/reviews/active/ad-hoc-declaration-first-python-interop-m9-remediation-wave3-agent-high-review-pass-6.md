# M9 remediation Wave 3 — agent 5.6 Sol high review pass 6

Reviewer configuration: `agent`, high reasoning, fast service tier, read-only full-wave review.

## Verdict: blocked

1. **High — consuming async owner close begins before acquiring its cancellation-finalization claim.** In `crates/sifr_codegen/src/python_interop_async/conversions.rs`, the semantic-close request consumes and marks the opaque owner before `append_submission`; `crates/sifr_codegen/src/python_interop_async/callback_frame.rs` emits that preamble before claiming the child scope. Cancellation racing in this window makes the claim fail and returns before `submit_async_declaration_with_callbacks`, so retained callbacks are not drained and `CallbackOwnerSlot` has no fallback close. The runtime test creates the scope before its request, so it cannot expose this generated ordering.

   Acquire the scope before semantic-close request construction, execute the entire fallible consuming-close preamble and submission inside the protected finalization block, and assert `scope claim < request construction < awaited owner close < parent resumption`.

