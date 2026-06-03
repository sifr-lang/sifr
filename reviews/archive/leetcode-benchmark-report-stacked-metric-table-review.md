

## Recommendation: Stacked-by-Metric Layout

The stacked format you proposed is the right direction. Here's my take:

**Structure** (6 columns regardless of implementation count):
```
Input | Runtime lead | Memory lead | Mean | Time/op | Peak RSS
```

**Cell content for 3+ implementations**:
```
Mean:
● Py: 120.4ms
● Sf:  50.7ms
● No:  60.0ms

Peak RSS:
● Py: 23.5 MB
● Sf:  2.0 MB
● No:  3.1 MB
```

**Actionable specifics:**

| Aspect | Recommendation |
|--------|----------------|
| **Labels** | Abbreviated colored dot + short code: `● Py` `● Sf` `● No` `● Bu` `● Rs` |
| **Colors** | Reuse existing `--impl-color` CSS variable from current code |
| **Row height** | `min-height` with auto-growth; consistent per-row (taller when N=5, shorter when N=2) |
| **Alignment** | Stack top-aligned within cells; values right-aligned, labels left |
| **Responsive** | Horizontal scroll fallback on narrow viewports (table-wrap already exists) |
| **Future proof** | Add impls without adding columns; just grows vertically in cells |
| **Diagnostics** | Keep the expandable row pattern — doesn't change |

**Why this works better:**
- Current: 9 columns (Python+Sifr) → 15+ with Rust+Node+Bun
- Proposed: 6 columns always — grows down, not across
- Each row stays visible; no horizontal scroll for typical cases
- Comparison across implementations is visual (stacked) not column-skipping

**One concern**: Cells with 5 stacked implementations get tall. Consider a "compact mode" toggle that shows just the leader per metric when N > 3, with a "show all" expand. But start simple — the flat stacked view is already a big improvement.
