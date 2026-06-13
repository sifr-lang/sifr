

**Review: Stacked-by-metric table UI**

**Verdict: Solid foundation, low-risk polish needed.**

**What works well:**
- Fixed6-column structure with stable width (`min-width: 920px`)
- Color-coded dots + short labels (`Py 120.4ms`) make scanning fast
- Implementation legend row keeps header self-documenting
- Sticky first column preserves context during horizontal scroll
- Tier-colored bars in "Runtime lead" give at-a-glance verdict
- Expandable diagnostics are well-structured**Actionable polish (in priority order):**

1. **Visual grouping between verdict vs. data columns**  
   Add a subtle left border or `border-left` on "Mean" `<th>` to separate the summary columns (Runtime lead, Memory lead) from the data columns (Mean, Time/op, Peak RSS). Currently all 6 columns blend together.

2. **Legend row needs visual separation**  
   The legend row blends into the header. Add `background: #fff` or a lighter shade to make it clearly optional/readable content, not a data column. Alternatively, move it above the `<thead>` as a standalone row.

3. **Consider alternating row shading**  
   With 4+ input sizes per problem, distinguishing rows helps. `tr:nth-child(even) td { background: var(--soft) }` is a one-liner that pays off in readability.

4. **"Runtime lead" bar is redundant with stacked cells**  
   The bar visualizes the same speedup the Mean column shows numerically. If you keep the bar, consider removing the percentage width label on mobile or keeping it only on wider screens to reduce clutter.

5. **No row hover state**  
   A subtle `tr:hover td { background: #f8fafc }` helps users track the current row across stacked cells. Low effort, meaningful UX.

**Scalability check:** Stacking works for any number of implementations — the grid auto-sizes the value column. The6-column cap (Input + 2 verdict + 3 metric columns) is reasonable. If you add more metrics (e.g., "Allocations"), you'd expand to 7 columns; the `overflow-x: auto` handles it gracefully.

**One concern:** The "Time/op" and "Peak RSS" columns may be redundant for most readers — "Mean" already captures runtime, and Peak RSS is the memory story. Time/op appeals to power users. Consider making it a toggle or keeping it but ensuring the column headers make the distinction clear.
