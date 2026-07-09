Working only from the patch summary as instructed (no file inspection), so these are reasoning-level flags, not confirmed defects.

## Risk flags to self-check before merge

1. **Evidence-id ↔ claim coupling (overclaim risk).** `async_runtime_core` is marked *supported* on the strength of exactly two evidence ids: `stdlib_async_resource_lifecycle` and `stdlib_async_hidden_blocking_rejected`. Those cover (a) resource lifecycle and (b) rejection of hidden blocking. Confirm the matrix row's *claimed capabilities* are limited to exactly those two axes. If the row also implies e.g. cancellation/join determinism as a *supported* guarantee, the driving evidence for that is the new `sifr_runtime` tokio test (`async_handle_close_and_cancel_join_are_deterministic`) — make sure that test is actually referenced as evidence for the row, otherwise cancel/join is an overclaim with no cited evidence.

2. **`_sifr.time` certification_rows repoint.** Changing the retained manifest from `async_runtime_reqwest` → `async_runtime_core` is only sound if `_sifr.time` genuinely needs *no* reqwest/loopback-level guarantee. Since `async_runtime_reqwest` is deliberately left `future-owned`, any `_sifr.time` leaf that still depends on network-async semantics would now be certified against a row that doesn't cover it. Verify no time leaf pulls in the reqwest path.

3. **Gate allowlist scope (gate-bug risk).** The sysroot gate now "allows supported core rows {opaque_resource_core, async_runtime_core}." Confirm this is an *allowlist that still rejects* any other row (notably `async_runtime_reqwest`) being marked supported at the core level — i.e., the change tightened, not loosened, the gate. A set-membership check that only *adds* async_runtime_core without keeping the "reject everything else" branch would silently let future overclaims through.

4. **Self-test "select rows by id" fix.** Good direction (positional selection is fragile). Just confirm the ids used in the self-test are asserted to *exist* — if `.get(id)` returns None-equivalent and the assertion is "not rejected," a typo�'d id would pass vacuously.

5. **Sync-vs-async close matrix.** Driver tests accept async `aclose` for `close=async_close` and reject sync `aclose`; the runtime test covers close+cancel/join. Confirm there's negative coverage that a `close=async_close` resource is *rejected* if it only exposes sync `close` — otherwise "hidden blocking rejected" is asserted one-directionally.

None of these are confirmed blocking from the summary alone — each is a "verify this holds" item. If items 1–3 check out (evidence ids match the exact claimed capabilities, no time leaf needs reqwest, and the gate still rejects non-allowlisted supported rows), then:

**READY** — subject to confirming flags 1–3, which I could not verify without inspecting files.
