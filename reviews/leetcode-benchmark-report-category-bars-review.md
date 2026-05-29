

## UI/UX Review: Category Overview Two-Bar Layout

**Overall**: The change achieves the goal — exact values are now visible alongside the derived badge. The approach is sound.

### What works well
- Exact values (236.5ms, 103.7ms, 34.8 MB, 9.2 MB) are readable and aligned right in a tabular-nums font
- Bars are scaled within the Python/Sifr pair, so visual ratio matches the numeric ratio
- Indigo (Python) / Teal (Sifr) color coding is consistent with the rest of the report
- Badge "Sifr 2.28x faster" and "Memory: Sifr 76% less" gives the headline verdict
- Tier-colored badges reinforce hierarchy (strong > good > marginal > regress)
- Variance dot at the end provides quick signal quality awareness

### Actionable improvements

1. **Collapse "Median runtime" → "Runtime"** (line 734 in report.py)
   - "Median" is redundant here since the detail table shows raw per-input values; "Runtime" and "Memory" are cleaner section labels

2. **Add a subtle left-border separator** between the Runtime and Memory blocks in `.category-bar`
   - Currently they blend together; a 2px border-left on `.summary-metric` (or the second one) would create a visual break

3. **Right-align the "Python"/"Sifr" labels** in `.value-row` instead of left-align
   - Labels are short (6 chars), left-align creates uneven whitespace against longer problem IDs; right-align would pair better visually with the right-aligned numeric values

4. **Consider swapping the bar fill direction** so Sifr fills left-to-right and Python is shown as a reference outline
   - This matches how speedup is mentally modeled ("Sifr is this much fast**er**")
   - Current: both fill left-to-right with Python always larger = visually redundant

5. **Tighten `.value-row` gap from 5px to 4px**
   - The row height (8px bar + text) feels slightly loose at 14px total row height; 4px gap would tighten the block

### Low-priority note
The "noisy" variance dot (amber) on 0001_two_sum may warrant a tooltip explaining what "noisy" means — the legend explains it but only once at the top.
