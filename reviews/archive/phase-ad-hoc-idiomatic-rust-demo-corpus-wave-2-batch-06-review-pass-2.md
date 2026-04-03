
Reviewing each file against the production-grade bar: behavioral equivalence, ownership/API/design quality, readability, and Rust norms.

---

## `demos/collections/idiomatic.rs`

**No actionable issues.**

Key observations:
- `Counter` correctly tracks insertion order via `encounter_order` for stable `most_common` sorting
- `most_common` sorts by count descending, then by encounter order ascending for ties — correct
- `Deque::append` early-return for `maxlen == 0` is explicit; capacity check uses `>=` — consistent with pass 1 follow-up
- `append_all` using `extend_from_slice` is clean
- All assertions match the Sifr reference semantics

---

## `demos/itertools/idiomatic.rs`

**No actionable issues.**

Key observations:
- `chain` lifetime and clone semantics are correct
- `pairwise` via `windows(2)` is the canonical Rust idiom
- `batched` correctly rejects `size == 0` with a typed `ValueError`
- `accumulate` uses `scan` with state correctly; `T: Copy + Add` bound is appropriate for numeric accumulation
- `cycle_n` via `cycle().take(n)` is clean
- Error message validation in `collect_negative_actual` is correct

---

## `demos/heapq/idiomatic.rs`

**No actionable issues.**

Key observations:
- `MinHeap<T>` via `BinaryHeap<Reverse<T>>` is the standard Rust min-heap pattern
- `heapify`/`heappush`/`heappop` are correctly implemented with ownership semantics
- `nsmallest`/`nlargest` allocate a temp vector (sort + truncate) — acknowledged as non-blocking per prior discussion; this is a small-scope performance tradeoff, not a design problem
- The `items` slice is not mutated by `nsmallest`/`nlargest` calls — assertion `items == vec![9, 3, 7, 1, 5]` is correct
- `collect_actual` uses independent type inference paths for each heap, which is fine

---

**No actionable issues.**
