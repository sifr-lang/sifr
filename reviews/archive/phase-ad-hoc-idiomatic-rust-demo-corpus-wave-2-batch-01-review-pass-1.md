## Review: Idiomatic Rust Demo Corpus — Wave 2 Batch 01

### Verdict: **No blocking issues. One actionable improvement recommended.**

---

### statistics/idiomatic.rs

**Actionable issues:**

1. **`mode` (lines 89–104) performs a second unnecessary pass over `data`** with hash lookups (`counts[value]`). After building the `counts` map, iterate over `counts.values()` directly to find the max — no second `data` traversal and no redundant hash lookups.

   ```rust
   // Current: O(n) pass over data + O(n) hash lookups
   let mut best_value = data[0];
   let mut best_count = 0_usize;
   for value in data {
       let count = counts[value];  // redundant hash lookup
       ...
   }

   // Better: single pass over counts
   let mut best_value = data[0];
   let mut best_count = 0_usize;
   for (&value, &count) in counts.iter() {
       if count > best_count {
           best_count = count;
           best_value = value;
       }
   }
   ```

2. **`multimode` (lines 112–129) has the same issue**: iterating `data` again with `counts[value]` hash lookups instead of iterating over `counts` directly.

3. **Type inconsistency**: `mode`/`multimode` take `&[i64]` while all other statistical functions operate on `&[f64]`. This is defensible (mode for continuous data is unusual) but worth documenting or homogenizing.

**Non-blocking observations:**
- Error type is manually implemented — acceptable, though `thiserror` is the more common modern choice for new projects.
- `near` helper is private but widely reused — consistent with demo scope.
- `quantiles` linear interpolation is correct.
- Bessel's correction in variance/stdev is properly applied.

---

### json/idiomatic.rs

**No actionable issues.** Clean, idiomatic wrapper around `serde_json`. Error mapping is precise (preserves line/column). `is_ok_and` usage is modern Rust. Naming (`loads`/`json_dumps`) matches Python parity goals appropriately.

---

### datetime/idiomatic.rs

**No actionable issues.** `from_timestamp` f64 is reasonable for a demo (Sifr likely uses float timestamps). `ValueError` is manually implemented but acceptable. `UtcOffset` display formatting is correct. `chrono` API usage is idiomatic.

---

### Summary

| File | Issues |
|------|--------|
| `statistics/idiomatic.rs` | 3 actionable (mode/multimode efficiency, type inconsistency) |
| `json/idiomatic.rs` | None |
| `datetime/idiomatic.rs` | None |

The statistics file has a legitimate efficiency issue in `mode` and `multimode`: they traverse the input slice twice when a single pass over the counts map suffices. The other two files are solid. All three files are readable, well-structured, and appropriate for educational demos.
