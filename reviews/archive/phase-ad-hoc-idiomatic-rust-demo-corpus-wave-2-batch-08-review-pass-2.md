## Pass 2 Review: phase ad-hoc-idiomatic-rust-demo-corpus-wave2-batch08

### bisect/idiomatic.rs

No new issues. `partition_point`-based bisect is correct. Insort correctly uses the corresponding bisect function. Edge cases (empty slice) handled. The `collect_actual()` aggregator pattern is unchanged from pass 1.

### defaultdict/idiomatic.rs

One subtle API semantics issue worth noting: `get_mut` takes `key: K` by value (line 26). Each call to `get_mut` in `main()` passes `"hit".to_string()` (or `1`), which creates a fresh owned `String`/`i64` per call. `HashMap::entry` accepts `Q` by value, so this works — but it means callers paying for repeated allocations if they hold the key in a variable and reuse it across multiple `get_mut` calls. This is observable in the `groups` demo where `"hit".to_string()` is allocated three times to access the same logical key. The Python defaultdict idiom of `d[k].append(x); d[k].append(y)` is not expressible without either storing the returned reference across statements (which is safe but only allows one pending borrow at a time) or using index-based access. The demo avoids exposing this constraint because all chained mutations are written as separate `get_mut` calls. Not a regression from pass 1, but the allocation behavior is a meaningful difference from Python defaultdict worth noting in API semantics.

### max_heap/idiomatic.rs

No issues. `heapreplace_max` correctly pops-before-push for max-heap semantics. Drain pattern is sound. Trace verification:
- `[2, 7, 4, 1, 8, 1]` heapifies to max-heap; drain yields `[8, 7, 4, 2, 1, 1]` — correct.
- `[4, 10, 7]` → `replace(6)`: pop-max `10`, push `6`, heap is `[6, 4, 7]`, drain yields `[10, 7, 6]` — correct.

### Final Verdict

**✅ Approved** — unchanged from pass 1. No behavioral regressions, no new correctness or maintainability issues beyond the pass 1 observations. The defaultdict `get_mut` allocation behavior and single-borrow chaining constraint are inherent Rust API characteristics, not demo defects.
