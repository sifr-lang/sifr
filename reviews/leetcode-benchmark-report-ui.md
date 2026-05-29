# Benchmark Report UI/UX Plan

## Status
- Reviewed existing: `report.py`, `report.html`, `problems.json`
- Current pain points identified (see below)
- Implementation guidance follows

---

## Pain Points in Current Report

| Issue | Severity | Detail |
|-------|----------|--------|
| Table favors Sifr-only columns | **Critical** | All CPU/memory/time columns say "Sifr". Comparison requires squinting or mental gymnastics. |
| No visual trend representation | **Major** | Bar shows speedup magnitude but not how speedup changes with size. Engineer's mental model is broken. |
| Category headings require scroll | Medium | 50+ problems will make category navigation painful. |
| No sorting/filtering | Medium | Can't isolate "regress" tier or sort by worst speedup. |
| Speedup bar normalization | Low | Bar width is % of global max. 2x in a 10x problem looks weak. Should be self-normalized. |

---

## Recommended Layout

### Structure: Category Accordion + Problem Cards

```
+-- Hero Stats (unchanged, just 4 KPIs)
+-- Filter Controls Bar
+-- Category Accordion
|   +-- [Category Header] "Arrays & Hashing (12 problems)"
|       +-- Category Summary Sparkline
|       +-- [Problem Card 1] expanded table
|       +-- [Problem Card 2] collapsed by default
```

**Why accordion:** At 50 problems across 10 categories, without accordion you'll have 80+ scroll distance. Sticky category headers during scroll help orientation.

**Why cards still:** Problem tables are self-contained; engineers can mentally isolate each benchmark result without cross-referencing.

---

## Recommended Table Redesign

### Column Headers

| Column | Current | Recommended | Rationale |
|--------|---------|-------------|-----------|
| Input | Input | Input | Keep |
| Speedup | Speedup | Speedup | Keep (computed, not raw) |
| Sifr Avg | Sifr Avg | **Mean: Py / Sifr** | Side-by-side, not stacked. Shows both and the ratio. |
| Python Avg | Python Avg | **Drop merged** | Covered by above |
| Sifr Unit Cost | Sifr Unit Cost | **Time/op**: Py / Sifr | Unit cost is what engineers care about; normalizes for loop count |
| Sifr Throughput | Sifr Throughput | **Throughput**: Py / Sifr | Ops/sec is the canonical performance metric |
| Sifr Peak RSS | Sifr Peak RSS | **Peak RSS**: Py / Sifr | Memory comparison |
| Sifr CPU User/System | Sifr CPU User/System | **Drop** or keep hidden | u/s split rarely changes the narrative; it's CPU total that matters |
| Sifr p50/min/max | Sifr p50/min/max | **Drop** or collapse into tooltip | Useful for debugging variance, not for summary |
| Variance | Variance | **CV**: Py / Sifr | Coefficient of variation for both is what "noisy" verdict is based on |

### Result: 6 columns instead of 11

New table columns: `Input | Speedup | Mean (ms) | Time/op | Throughput | Peak RSS | CV`

Each metric shows `Py value / Sifr value` in a compact divider format, e.g.:

```
Mean:   115.3ms / 46.2ms  →  2.49x
```
or abbreviated column headers if 3 values is too wide:

```
Mean Py  |  Mean Sf  |  Speedup
115.3ms |  46.2ms  |  2.49x  (colored)
```

**Decision:** Use dual-column if width allows (1600px+), abbreviate headers if cramped. Engineer-friendly means readable without horizontal scroll on 1080p.

### Speedup Column Specifics

- **Color tiers (keep existing):** strong (≥3x green), good (≥2x teal), watch (≥1x amber), regress (<1x red)
- **Bar width normalization:** Bar should be `self/1` at 1x, `self/max_self_in_problem` at problem level, NOT global max. Reason: 2x speedup in palindrome (where Sifr is 10x faster overall) should NOT visually dominate 2x in two_sum (where Sifr is only 2x faster). Self-normalization lets engineers see within-problem trends.
- **Hide bar for regress tier:** A regress or neutral (~1x) bar at 10% width is visually noisy. Only show bar when speedup ≥ 1.5x.

---

## Recommended Graphs (Vanilla JS)

### 1. Sparkline: Speedup vs Input Size (per problem)

Embed in the problem card header, small inline SVG:

```
two_sum: 2.49x ────╮
                (n=1K) (n=10K) (n=100K) (n=1M)
```

- **Implementation:** Pure SVG, 120px wide x 24px tall. One polyline per problem.
- **Why:** Engineers can instantly see if speedup degrades at scale (a key Rust vs Python gotcha — memory allocator pressure at high N).
- **Hint color:** Use speedup tier color. If line crosses tiers, use dominant tier.

### 2. Horizontal Bar Chart: Speedup by Problem (category view)

Show when category is **collapsed** (accordion header):

```
0001_two_sum           ████████████░░░░  2.23x
0002_add_two_numbers  ██████████████░░░  2.87x
...
```

- **Implementation:** CSS-only bars via `width: calc(speedup * base)` where base = 50px per 1x. No JS, no canvas.
- **Why:** Category-level scan without opening every problem. Fastest to slowest.
- **Max width:** Cap at 200px (representing 4x+) so 10x problems don't crush 1x ones visually.

### 3. Scatter Plot: Memory vs Speedup (optional, if valuable)

If engineers want to find the "fast-but-heavy" outliers:

```
Speedup (x-axis) →  1x    2x    3x    5x   10x
Memory (y-axis) →  50MB  100MB  25MB 200MB 80MB
```

- **Implementation:** SVG with 50-100 plotted points, no axes needed (just position).
- **Why:** Memory vs speedup correlation reveals if Sifr's speed comes at memory cost.
- **Skip if** problems < 20 — too few points to show pattern.

---

## Recommended Interaction Controls

### Must-Have

| Control | Behavior |
|---------|----------|
| Category accordion | Click to expand/collapse. Sticky header during scroll within expanded section. |
| Problem sort (within category) | Dropdown: By Speedup (default) / By Problem ID / By Input Size (ascending) |
| Tier filter | Checkboxes: Show [ ] Regress [ ] Watch [ ] Good [ ] Strong. Default: all checked. |
| "Stable only" toggle | Checkbox, unchecked by default. When checked, hides rows where either impl has `verdict == "noisy"`. |

### Nice-to-Have (Low Effort)

| Control | Behavior |
|---------|----------|
| Search | Text input, filters problem IDs by substring match. |
| "Expand all / Collapse all" | Two buttons in filter bar. |
| Copy row as Markdown | Small button per row that copies `| Input | Speedup | ...` for pasting into PRs/docs. |

### Do NOT Add

- Machine learning anomaly detection for outliers — overkill, needs explanation
- Interactive zoom/pan on charts — adds complexity for static report
- Multi-select problems for cross-problem comparison — requires different view model
- Dark mode toggle — adds CSS complexity, most engineers prefer system default

---

## Data Integrity Flags (Misleading Patterns to Prevent)

### Flag 1: Different Operation Counts
**The critical issue.** Python and Sifr benchmarks run the **same benchmark script** but may have different `--warmup` or `--min-runtime` settings. Verify that `operations` field is identical for both impls in each row. If not, speedup comparison is invalid.

**Mitigation:** Add a visible note in the table header: *"Speedup computed only where operation counts match."* If counts differ, show `—` instead of speedup.

### Flag 2: CV Threshold is Arbitrary
`verdict = "noisy" if cv > 0.10` is a 10% threshold chosen without justification. Different workloads tolerate different variance.

**Mitigation:** Show CV column for both impls. Add tooltip: `"CV = stddev/mean. ≤10% = stable per benchmark config."` Allow engineers to re-assess.

### Flag 3: Throughput Units Vary by Problem
`throughput_per_s` is `operations / mean`. Operations vary by problem (two_sum has different loop counts than palindrome). Cross-problem throughput comparison is misleading.

**Mitigation:** In category/global views, always label throughput as "ops/s (problem-specific)" or only compare throughput within the same problem's sizes.

### Flag 4: Memory Measurement Scope
`memory_mb` from hyperfine is **process RSS at end of run**, not peak during run. For problems with high transient allocation, this underreports Python's memory (GC runs after Python completes).

**Mitigation:** Add footnote in environment panel: *"Memory: hyperfine RSS at run completion. May underreport peak usage for GC'd runtimes."*

### Flag 5: Warmup Runs Not Visible
Hyperfine defaults to `--warmup 1`. A single warmup is often insufficient for JIT-allocated caches (not an issue for Sifr, but relevant if benchmarks ever run on PyPy or Numba).

**Mitigation:** Show warmup count in environment panel. If warmup = 1, show warning chip: *"⚠ Single warmup run — consider 3+ for GC'd runtimes."*

---

## Implementation Priority

| Item | Priority | Effort |
|------|----------|--------|
| Side-by-side table columns (Mean, Time/op, Throughput, Peak RSS, CV) | P0 | Medium |
| Self-normalized speedup bars | P0 | Low |
| Category accordion with sticky headers | P1 | Low |
| Tier filter checkboxes | P1 | Low |
| Speedup sparkline per problem | P2 | Medium |
| "Stable only" toggle | P2 | Low |
| Horizontal bar chart in collapsed category header | P2 | Low |
| Copy-row-as-markdown | P3 | Low |

---

## CSS Architecture for New Table

```css
/* Core table: two-column per metric */
.metric-dual {
  display: grid;
  grid-template-columns: 1fr auto 1fr; /* Py value / divider / Sifr value */
  gap: 0;
}
.metric-dual .py { color: #6366f1; font-variant-numeric: tabular-nums; }
.metric-dual .sf { color: #0f766e; font-variant-numeric: tabular-nums; }
.metric-dual .divider { color: var(--muted); padding: 0 6px; }

/* Speedup badge */
.speed-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.speed-badge .bar {
  width: 80px; /* fixed width, self-normalized within problem */
  height: 8px;
  border-radius: 4px;
  background: #e2e8f0;
}
.speed-badge .bar-fill {
  height: 100%;
  border-radius: inherit;
  background: var(--tier-color);
  /* width set inline: calc(min(speedup, 4x) / 4x * 100%) */
}
```

---

## File Changes Required

1. **`report.py`** — `comparison_row()`: Rewire columns. `render_html_report()`: Add accordion JS, filter bar HTML, sparkline SVG generation.
2. **CSS block** — Add `.metric-dual`, `.compact-bar`, `.filter-bar`, `.accordion-header` styles.
3. **JS block** (in `<script>` tag at bottom) — Accordion toggle, filter checkbox handlers, sort handler. All vanilla, no deps.

Target output: Single self-contained `.html` file, same constraint as current.

---

## Open Questions for User

1. **Max comfortable table width:** Current table has `min-width: 1040px`. With 6 columns and dual values, expect ~1200px. Acceptable, or should I optimize for 1080p screens?
2. **Sparkline granularity:** Show one point per size (as sketched above) or aggregate to category-level? (Category-level is easier to implement but less useful per-problem.)
3. **Copy-row-as-markdown:** Useful enough to prioritize over sparklines?
