

The heap math is verified: `count=-100` (newer) produces key `39,995,996,100,900` and `count=-10` (older) produces `39,999,996,099,900`. Since 99,900 < 99,990, the newer tweet pops first — correct.

The pass-1 reviewer's arithmetic was inverted. Your reasoning is sound.

**M1b Re-Review:**

1. **Heap ordering:** Correctly reasoned and covered by assertions. `key = ((count + offset) * base * base)` preserves the natural order because `count + offset` stays positive and monotonically related to recency. No negation needed.

2. **Overflow warnings:** Acceptable. Max key ≈ 40B, well within i64. Warnings are compiler-conservative for the general case.

3. **Metadata/seed updates:** Previously validated in pass-1. M1b rows (0355, 0295) correctly marked as `mixed/equivalent`.

**M1b is approved.** No further review rounds needed.
