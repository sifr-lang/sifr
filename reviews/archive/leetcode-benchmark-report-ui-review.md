

## UI/UX Review: LeetCode Benchmark Report

### 1. Hero Summary — Engineering Story

**What's good:**
- Median Runtime and Median Peak RSS are the two most important headlines and are prominently displayed.
- Avg Runtime adds useful context (15.95x vs 5.91x median shows right-skewed distribution).
- Reliable Comparisons (65/108) helps calibrate trust in the numbers.

**Issue:** The hero headline "LeetCode runtime comparison" doesn't communicate the key message — it should say something like "Sifr is 5.91x faster on average" as the primary takeaway. The description paragraph is verbose and doesn't call out the headline numbers.

**Recommendation:** Move the core claim ("Sifr is Xx faster, uses Y% less memory") into the title or a prominent callout above the description. Let the description be a one-line explainer.

---

### 2. Runtime vs Memory Color Semantics

**Runtime tiers (color + meaning):**
- `strong` (green): ≥3x faster — **correct**, feels like a win
- `good` (teal/green): ≥2x faster — **acceptable**
- `watch` (amber): ≥1x faster — **ambiguous**: "Sifr is faster but you should watch it" is confusing. "Watch" sounds like a warning.
- `regress` (red): slower — **correct**

**Issue with `watch`:** The amber/warning color on "Sifr 1.25x faster" feels like a caution sign on good news. The word "watch" in the filter checkboxes (showing "watch" tier) also makes it sound like a problem.

**Memory tiers:**
- `strong`/`good` (green): ≥2%/10% less memory — **correct**
- `neutral` (gray): ±2% — **correct**
- `regress` (red): more memory — **correct**

**Potential confusion:** Both runtime `watch` and memory `neutral` use amber/gray respectively, but they mean different things. A reader might think `watch` is bad. Consider renaming `watch` to something like `marginal` to reflect that the speedup exists but is small.

---

### 3. Category Overview Readability

**What's good:**
- One row per problem with bar + text + memory badge + variance dot is scannable.
- Bar width normalized to max speedup makes comparisons visual.
- Variants sorted by speedup (descending) so fastest problems float to top.

**Issues:**

The grid layout is tight:
```
grid-template-columns: minmax(150px, 1fr) minmax(120px, 260px) minmax(150px, auto) minmax(110px, auto) auto
```
With 5 columns and 12px gaps, this may overflow on narrow containers.

**Misleading bar color for `watch`:** The `watch` tier (amber) bar makes slow problems appear to have a warning color even when Sifr is still faster. The bar color should represent magnitude, not caution — consider using teal for `watch` since it's still a win.

**Memory badge can wrap:** At small widths, `Sifr 88% less` in a 106px min-width badge will wrap to two lines, breaking the row alignment.

---

### 4. Detailed Table Column Order and Labeling

**Current order:**
1. Input
2. Runtime (bar + text)
3. Memory (badge)
4. Mean (Py / Sifr)
5. Median (Py / Sifr)
6. Min to Max (Py / Sifr)
7. Stddev (Py / Sifr)
8. CPU User / System (Py / Sifr)
9. Time/op (Py / Sifr)
10. Throughput (Py / Sifr)
11. Peak RSS (Py / Sifr)
12. CV (Py / Sifr)
13. Variance

**Issues:**

- **Min to Max** is confusing — "Min to Max" could mean "range" but it shows `Xms to Yms` which is actually the per-impl range, not a cross-impl comparison. This is redundant with mean/median.
- **CPU User / System** is low-value — shows Python's overhead breakdown which isn't actionable for Sifr evaluation. Consider dropping this column.
- **Throughput** header doesn't explain it's problem-specific — a footnote or tool-tip would help.
- **Variance** column at the end is sparse (just "stable" or "noisy") — could be a badge or inline indicator instead of a full column.

**The key columns for Sifr evaluation are:** Input, Runtime advantage, Memory advantage, Mean, Median, Peak RSS. The rest are diagnostic. Consider grouping them as "Detailed" in a collapsed section or reducing horizontal density.

---

### 5. CSS/Layout Risks

**Desktop (≥920px):** Looks solid. Two-column visual grid and hero layout work well.

**Mobile (<920px):**
- The table has `min-width: 1660px` — this will always cause horizontal scroll on mobile. A horizontal scroll container is fine, but the 13 columns make it a painful scroll experience.
- `.category-bar` collapses to single column, which is good.
- Filter bar becomes sticky (position: static at mobile) which loses utility.

**Risk:** The chart SVG width is `100%` but `viewBox` is 720px. At narrow widths, the charts may render at a cramped scale. The 310px height is fixed, so aspect ratio could distort on very narrow viewports.

**Font scaling:** `14px/1.5` base body font is readable but dense for data-heavy tables. The `12px` for muted/secondary text hits accessibility minimums.

---

## Priority-Ordered Recommendations

| Priority | Issue | Fix |
|----------|-------|-----|
| **P1** | `watch` tier color feels like a warning on good news | Rename to `marginal` and use teal instead of amber |
| **P1** | Hero title doesn't call out the key claim | Add "Sifr Xx faster, Y% less memory" to the title or as an eyebrow |
| **P2** | Table has 13 columns — horizontal scroll painful on mobile | Collapse CPU, Throughput, Stddev, CV into a "Details" expandable section |
| **P2** | Memory badge wraps on narrow widths | Allow badge to shrink, or use `flex-shrink: 0` |
| **P2** | `Min to Max` column is confusing and low-value | Remove or rename to clarify it's per-impl range, not comparison |
| **P3** | Variance column takes a full column for a dot+word | Convert to inline status indicator, drop the column |
| **P3** | Charts may compress badly on narrow viewports | Add `overflow: hidden` with a minimum chart width or stack vertically |
