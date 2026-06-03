

Now I have a complete picture. Let me give you a concrete recommendation.

---

## Table Redesign: Transposed Column Layout

**Core problem:** The current layout (one row per size, Py/Sifr jammed into every cell) fails at scale. A 4th language means 4 values per cell. The fix is to flip the axis: **one column group per implementation, not one cell slice per metric.**

### Layout

```
| Input | Speedup | Mem Δ | [Python]        | [Sifr]           | [Node]...  |
|       |         |       | Mean | t/op | RSS | Mean | t/op | RSS | ...
| 1K    | 2.5x █  | -12%  | 53ms | 53ns| 80MB | 21ms | 21ns| 25MB | ...
| 10K   | 3.1x █  | -18%  | 510ms| 51ns| 320MB| 165ms| 17ns| 262MB| ...
```

Each row = one input size. Each column group = one implementation with its own metric values.

**Column groups:**
- **Summary:** Input, Speedup (bar + badge), Memory (badge)
- **Per implementation:** Mean, Time/op, Peak RSS — plus an expandable details row

**Expandable details row (per input size):**
```
| Median | Stddev | CV | CPU u/s |
| 51ms   | 4.2ms  | 8% | 40/13ms |
```

### Why this scales

| Languages | Current layout (values per cell) | Transposed (column groups) |
|-----------|----------------------------------|-----------------------------|
| 2 (Py+Sifr) | 2 values, cramped | 1 group + 3 cols |
| 3 (+Node) | 3 values, unusable | 2 groups, still clean |
| 5 (+Bun,Rust) | 5 values, scroll-to-read | 4 groups, horizontal scroll |
| N | N-uple per cell | N × 3 cols + group headers |

You can also do **"show top N only"** with a toggle — collapse lower-priority implementations into a single "+N more" column that expands inline.

### Metric drop/shrink decisions

Drop from the default visible table (keep in expandable details):
- **Median** — redundant with Mean for comparing two impls; mean already shows the story
- **Throughput** — inverse of Time/op, always redundant
- **Stddev** — diagnostic, useful only when CV triggers "noisy"
- **CPU User/System** — total CPU is the useful number; split is not
- **Min/Max range** — only matters for variance debugging

**Default visible:** Input, Speedup, Memory Δ, Mean, Time/op, Peak RSS per impl.

**Expandable details:** Median, Stddev, CV, CPU total (u+s merged).

### Speedup bar normalization

Bar width = `speedup / max_speedup_in_this_problem`, not global max. A 2x speedup in a 5x-problem should look half the bar of the 5x, not ~same as the 5x in another problem where the max is 10x.

Only show the bar when speedup ≥ 1.5x. Regress/neutral rows skip the bar — a 5% bar on a 0.95x row is visual noise.

### Variance indicator

Not a column — inline dot in the expandable row or per-impl cell. E.g.:

```
Mean: 53ms  (Py: ●stable)  →  green dot
Mean: 51ms  (Py: ●noisy)   →  amber dot
```

### CSS shape

```css
/* Column groups */
.col-impl-header { border-bottom: 3px solid var(--impl-color); }
.col-impl-mean   { border-left: 1px solid var(--line); }

/* Compact pill values */
.val { 
  font-variant-numeric: tabular-nums; 
  font-size: 13px;
}
.val-py { color: var(--indigo); }
.val-sf { color: var(--teal); }

/* Expandable details */
.detail-row td { 
  background: var(--soft); 
  font-size: 12px;
  color: var(--muted);
}
```

### Scroll ergonomics

- Sticky left panel: `position: sticky; left: 0` on the Input + Speedup + Memory columns (the summary view)
- The rest scrolls horizontally
- Total default width: ~1080px for 2 languages with 3 metrics each (no horizontal scroll on 1080p)
- 3 languages: ~1400px (horizontal scroll kicks in, but summary panel stays visible)

### Implementation notes

1. **Dynamic column generation** — current code hardcodes `python`/`sifr` pairs. Refactor `comparison_row()` to iterate over `sorted(impls.keys())` and generate columns dynamically. This is the key to supporting N languages.

2. **Expandable details** — one `<tr class="detail-row">` per input size, CSS `[open] .detail-row { display: table-row }` toggled by a button or the problem card's `<details>` state.

3. **Sticky summary panel** — wrap Input + Speedup + Memory in a `<colgroup>` with `position: sticky; left: 0; z-index: 2; background: white`.

4. **Per-impl color coding** — assign colors from a palette (indigo for Python, teal for Sifr, orange for Node, purple for Bun, etc.) and propagate via `data-impl` attributes. Current code hardcodes `.py` and `.sf` — needs generalization.

### Summary

| Aspect | Current | Proposed |
|--------|---------|----------|
| Columns | 12 (Py/Sifr in every cell) | 3 summary + N×3 impl cols |
| Width (2 lang) | 1660px min | ~1080px |
| Width (3 lang) | — | ~1400px |
| Variance | Full column | Inline dot |
| Scalability | Breaks at 3+ | Horizontal scroll, summary sticky |

This is a medium refactor of `comparison_row()` + `render_html_report()` in `report.py`. The charts stay unchanged — they're already the primary visual comparison layer.
