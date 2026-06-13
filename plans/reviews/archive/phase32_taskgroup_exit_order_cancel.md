

Reviewing the diff for correctness, cancellation semantics, generated Rust validity, and test adequacy.

## Review Findings

### Bug 1: Abort-triggered cancellation sets `group_failure_seen` incorrectly

**File:** `crates/sifr_codegen/src/preamble.rs:744`

```rust
Ok((observed, Err(join_error))) => {
    group_failure_seen = !join_error.is_cancelled();  // <-- correct check
```

The check `!join_error.is_cancelled()` correctly prevents setting `group_failure_seen` for cancellations. However, tracing through the scenario:

1. Child A fails → `group_failure_seen = true` → `policy_cancelling = true` → `abort_handle_B.abort()`
2. Observer B cancelled → `join_next()` returns `Ok((_, Err(join_error)))` with `is_cancelled() == true`
3. `group_failure_seen = false` (correct — we don't set it)
4. `policy_cancelling = true` → skip abort loop
5. Check `!observed && failure.is_none()` → `failure` already set from step 1 → skip

This path is correct. The bug is that **the condition should also handle the case where `group_failure_seen` IS true but the failure was already recorded**.

Actually wait — re-reading the current code, `group_failure_seen = !join_error.is_cancelled()` means:
- If cancelled: `group_failure_seen = false`
- If not cancelled (panic): `group_failure_seen = true`

This is correct. If the observer panics, we should abort remaining siblings. If it's cancelled by our abort, we should not.

### Bug 2: Missing `!policy_cancelling` guard in Cancelled branch

**File:** `crates/sifr_codegen/src/preamble.rs:737-741`

The Cancelled branch:

```rust
Ok((observed, Ok(__SifrScopeChildOutcome::Cancelled))) => {
    group_failure_seen = true;
    if !observed && failure.is_none() {  // <-- missing !policy_cancelling
```

The original sequential code had:
```rust
if !observed && !policy_observed && failure.is_none() {
```

The `!policy_observed` check prevents creating a `ScopeFailure` entry for a child that was cancelled as a direct result of our abort. In the new concurrent code, once `policy_cancelling = true`, subsequent children are cancelled by our abort, not by external cancellation.

**Fix:**
```rust
if !observed && !policy_cancelling && failure.is_none() {
```

This matches the Err branch's pattern and prevents creating a spurious `ScopeFailure` for a child we ourselves cancelled.

### Bug 3: `Err(_)` (observer panic) unconditionally creates failure entry

**File:** `crates/sifr_codegen/src/preamble.rs:755-758`

```rust
Err(_) => {
    group_failure_seen = true;
    if failure.is_none() {  // <-- missing !policy_cancelling
        failure = Some(ScopeFailure::new("task group child observer failed".to_string()));
    }
}
```

If the observer panics after we've already recorded a failure from another child, this creates a redundant `ScopeFailure`. Should be:
```rust
if !policy_cancelling && failure.is_none() {
```

### Test Adequacy Assessment

The test fixture `task_group_fail_fast_spawn_order.sifr` is correct:
- `fail_fast()` completes instantly (0.0s delay)
- `slow_writes_marker()` takes 0.20s
- Asserts sibling marker doesn't exist, validating cancellation before completion

The 0.20s sleep creates a wide timing window. A `task.yield_now()` would be more deterministic, but the current timing is reliable in practice.

### Rust Validity

The generated code syntax is valid. `self.children.drain(..)` consumes the Vec, `abort_handles` iteration is correct, and async block captures are sound.

---

**Summary of required fixes:**

| # | Location | Issue |
|---|----------|-------|
| 1 | Line ~740 | Add `!policy_cancelling` guard to `Cancelled` match arm |
| 2 | Line ~755 | Add `!policy_cancelling` guard to `Err(_)` (observer panic) arm |

---

NOT SATISFIED. Two concrete bugs require fixes before this slice can be approved.
