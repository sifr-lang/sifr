

## UI/UX Review: LeetCode Benchmark Report

### 1. Page-Length / Scannability ✅
**Verdict: Solved.** 18 collapsed category sections + sticky filter bar gives a clean landing page. Initial view shows only the hero and filter bar — scannable.

### 2. Category-Level Summary ✅
**Verdict: Sufficient for first view.** Each collapsed category shows name, speed-chip, memory badge, and problem count. Opening reveals the problem bar chart (name + speed bar + badges + variance dot). This is good progressive disclosure — no need to open a problem to compare within a category.

### 3. Problem-Level Drilldown ✅
**Verdict: Intuitive.** Each problem is a `<details>` card — standard, familiar. Opens to show two charts (runtime + memory) and the full table. No confusion.

### 4. Color / Label / Layout Risks

**Worth fixing before shipping:**

1. **Search doesn't open empty-matching categories** — If I search for a problem ID that belongs to a category with no other matches, only that section opens. But if I search "Arrays" (hoping to see category grouping), no categories open because category names aren't matched. Consider matching against category too, or add a hint in the placeholder ("Search by problem ID").

2. **Regress filter defaults to checked** — Most users care about wins; defaulting to show regressions only on first load may confuse. Consider "show all" default or pre-check all tiers.

3. **Hero "Avg Runtime: Sifr 15.95x" + "Median: Sifr 5.91x" side by side** — These numbers are jarring together. Avg (mean) being ~3x the median indicates a skewed distribution. Add a small note or suppress avg if it's purely informational rather than actionable.

4. **"n/a" in category bars when memory data missing** — The delta-badge shows "n/a" which is fine, but the speed bar still renders. No visual indicator that the row has incomplete data — user might assume both metrics are valid.

5. **No category-level collapse on search** — Search opens matching sections but leaves their collapsed/expanded state unchanged. This means a user who previously opened a category leaves it open, defeating the scannability goal when filtering.

### Minor / Acceptable as-is
- The table is wide (1660px min) — horizontal scroll is expected for this data density
- 65/108 "reliable comparisons" shown in hero — fine to ship as-is; no action needed
