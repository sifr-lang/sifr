## Review: phase ad-hoc-idiomatic-rust-demo-corpus-wave2-batch08

### Findings

**bisect/idiomatic.rs — Minor (non-blocking)**

- `collect_actual()` repeats `vec![1, 3, 3, 3, 5]` and `vec![1, 3, 3, 5]` across assertions. Not a behavioral issue; purely a readability/maintainability observation. The self-test structure (`Vec<bool>` aggregator + final assert) is necessary scaffolding to make the demo self-verifying but adds some noise.

No behavioral regression. Correctness verified.

**defaultdict/idiomatic.rs — Minor concern (non-blocking)**

- `get_mut` returning `&mut V` means the borrow must fully expire before the next call. The Python defaultdict pattern of `entry = d[k]; entry.push(x); entry.push(y)` doesn't port directly — chaining fails at compile time. The demo tests avoid this by calling `get_mut` once per key per statement, which is safe but doesn't exercise the limitation.

This is an API shape that's "honest but limited." Users familiar with Python defaultdict will get a borrow-checker-guided education. Not a regression from demo parity since the test cases don't attempt chaining, but the limited API shape is visible from the demo intent.

**max_heap/idiomatic.rs — No issues**

- `heapreplace_max`: pop-then-push correctly restores heap invariant. The `BinaryHeap::pop` removes max, `push` reheapifies, so the final state is sound.
- `drain`: correct consume-until-None pattern.
- Test traces verify correct behavior: `stones` drains `[8, 7, 4, 2, 1, 1]`; `probe` after `replace(6)` drains `[10, 7, 6]`.

### Recommendation

**Accepted** — all three files correctly demonstrate the target behaviors with no regressions.

Rationale:
- `bisect`: correct bisect-left/right semantics, correct insort-left/right semantics, edge cases (empty slice) handled.
- `defaultdict`: core defaultdict behavior (lazy factory, accumulate-into-collection, set insert, counter increment) all correct. The `&mut V` borrow limitation is an inherent Rust tradeoff, not a demo parity failure.
- `max_heap`: heapify, pop-max, replace-max, and drain all correct.

The `collect_actual()` aggregation pattern in bisect and the `get_mut` single-call limitation in defaultdict are observable characteristics of Rust-first companions, not defects. Validation passed with standalone rustc and demo runs, which is the authoritative signal.

### Final Verdict

✅ **Approved** — ready for batch commit.
