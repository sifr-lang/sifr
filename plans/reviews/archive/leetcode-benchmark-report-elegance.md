

Reviewing the charts and layout. Key observations:

**What's working well:**
- Both charts now have explicit axes with labeled ticks, axes lines, grids, and axis titles
- The 1x baseline on the speedup chart provides clear reference
- Point labels on speedup chart show exact values above each data point
- Runtime chart has proper Python/Sifr line separation with legend
- Y-axis labels on the runtime chart are readable with proper log scale formatting

**Concrete issues to fix:**

1. **Missing vertical grid lines**: Only horizontal lines exist. Adding vertical lines aligned to each X tick would make it much easier to read values at specific input sizes.

2. **Speedup Y-axis lacks explicit scale notation**: The runtime chart header shows "log scale" but the speedup chart Y-axis (which is linear) has no equivalent "linear" or "linear zoomed to data range" note. Add `font-style: italic` label "Y: speedup ratio (linear)" below the axis title.

3. **Legend in runtime chart uses absolute positioning**: `cx="542"` is vulnerable to clipping if SVG scales oddly. Wrap the legend in a `<g>` translated to `translate(540, 20)` and clip to the chart area if needed.

4. **Point labels on speedup chart can collide**: The `max(top + 12, y - 10)` offset doesn't prevent label collision when two consecutive points are close vertically. Consider alternating label side placement (above/below) or add `dy` nudge for tight clusters.

5. **Chart card minimum height is 360px but charts are 310px tall**: Creates dead space. Reduce card `min-height` to 344px to fit tighter to chart content.

6. **The "log scale" badge hides the actual range**: The runtime chart badge says `log scale` but doesn't show the actual Y range (e.g., `47.9ms–6.7s`). Engineers often want to know if the scale was autoscaled or if there are outlier values. Consider showing a hint: `log scale · range: 48ms–6.7s` or just let the axis tick labels speak for themselves and remove the `<strong>log scale</strong>` label entirely.

7. **The category overview bar chart lacks labeled axes**: The problem bars show speedup as width but there are no axis labels indicating what the bar length represents. The bar header has "median speedup" implied but not labeled directly on the bars.

8. **No hover/tooltip on chart data points**: For engineers investigating specific data points, SVG `<title>` elements or hidden-by-default tooltips would help without requiring table scrolling.

9. **The baseline label position**: `y="{baseline_y - 7:.1f}"` puts it 7px above the baseline. If the baseline is near the top of the chart, the label clips outside the viewBox. The1x baseline at `y=163.4` is already tight. Add a guard: only render the label if `baseline_y > top + 20`.

10. **`speedup_tier("watch")` for values between 1-2x**: A 1.55x speedup (0001 at1M) is marked "watch" which uses amber, but it's still a positive speedup. The color semantics are confusing—"watch" amber suggests caution, but a positive speedup should use a neutral or green-adjacent color. Consider introducing a fifth tier or adjusting the color mapping.

11. **The runtime chart has no labeled boundary for "noisy" size points**: If Sifr at 1M in two_sum is noisy, the runtime chart should visually distinguish that point (e.g., hollow circle vs filled). Currently only the table shows variance verdict; the chart does not.

**Priority ranking:**
- Issues 1 and 2 are the most impactful for axis clarity (ask stated concern)
- Issues 4, 6, and 9 are moderate polish
- Issues 3, 5, 8, 10, 11 are nice-to-have enhancements

Do you want me to implement any of these?
