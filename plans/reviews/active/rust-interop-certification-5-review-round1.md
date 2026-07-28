# Rust Interop Certification 5 Review — Round 1

Reviewer: Claude Opus 5 (`--effort medium`)

Scope: uncommitted certification 5 working-tree implementation, excluding the
unrelated `editor_integrations` submodule change.

## Reviewer output

### 1. `alias-use-after-close=rejected` is false — the alias is fully usable

`resource_lifecycle_runtime/src/bridges/resources.rs:419-439`

`Handle::clone` (`crates/sifr_runtime/src/interop.rs:387-398`) deep-copies the
slot, so `mark_closed` on `original` leaves `alias.slot == Open`. The check then
requires `alias.inner_ref()` to succeed and reads a bridge-owned shared Boolean.
No `HandleStateError::Closed` is observed for the alias, and the probe does not
touch the real certified resources.

This falsifies the alias-rejection claims in the compatibility matrix, public
docs, internal architecture, and fixture README. Double-close is genuinely
proven; only the alias claim is unsupported.

### 2. The `expected_runtime_state: "closed"` provenance gate is satisfied by prose

`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:560`

`_rust_test_outcomes.py` looks for `HandleStateError::Closed` in assertion
source, but the token appears only inside an assertion failure-message string.
The gate intended to require a real closed-state observation is bypassed.

### 3. The negative fixture is a copy of the positive — no independent negative coverage

`fixtures/opaque_resource_matrix/negative/invalid_resource_aliasing.sifr`

Both directions call the same bridge entry point and execute the same package
path. The negative exits successfully with a summary rather than reaching an
independent error state, so the two directions amount to the same run twice.

### 4. Poison redaction bypasses the generated glue it claims to certify

`resource_lifecycle_runtime/src/bridges/resources.rs:441-464`

The test catches an unrelated panic and calls `mark_poisoned` manually. It does
not exercise the authentic generated-glue `PoisonOnPanic` path, so it proves a
setter/getter rather than bridge-unwind poisoning and redaction.

### 5. "Zero active tasks" is not a sound cleanup gate

`resource_lifecycle_runtime/src/bridges/resources.rs:244-251, 93-97, 509-513`

The PostgreSQL negative-probe connection task is spawned outside `TrackedTask`
and is neither counted nor aborted on timeout. `TaskActivity` is constructed
inside the spawned future, so a task aborted before its first poll can increment
without ever decrementing.

### 6. SQLite-cleanup failure path skips task shutdown and handle closure

`resource_lifecycle_runtime/src/bridges/resources.rs:501-504`

An early return from database removal occurs before task joins and before
marking the handle closed. Cleanup errors can leave the handle open with drained
state and rely only on drop abortion.

### 7. Low: catalog `rusqlite` downgrade re-scopes an already-certified row

The rationale is recorded, but `rusqlite` also participates in the previously
certified `blocking_offload` row; that row needs explicit revalidation under the
downgraded graph.

The loopback harnesses, locked/offline profile wiring, protocol round trips, and
double-close evidence are otherwise sound.

**VERDICT: NOT SATISFIED**
