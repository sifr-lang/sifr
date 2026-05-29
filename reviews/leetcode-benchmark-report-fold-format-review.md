

## Review: `format_fold` thresholding at 2x

**Finding1 — The `rstrip("0").rstrip(".")` edge case silently kills precision near 1x.**

`format_fold(1.01)` → `"1x"` due to trailing-zero stripping. This defeats the whole stated goal of keeping "1.68x" meaningful —1.01 is also a valid improvement and shouldn't collapse to plain "1x faster."

Fix: add an explicit floor or clamp before stripping, or a minimum decimal count for values closer to 1x.

**Finding 2 — The 2x threshold is defensible but1.5x feels tighter.**

Comparing against the tier boundaries (`speedup >= 3 = strong`, `>= 2 = good`, `>= 1 = marginal`), thresholding fold rounding at 2x aligns with the tier cutoffs. That said, `1.9x` rounding to `"2x"` is a5% rounding error for what the tier labels call "marginal." Lowering the threshold to **1.5x** would round marginal/edge cases more faithfully.

**Finding 3 — Nearest integer is the right choice over floor.**

`int(value + 0.5)` (round half-up) is already in place and correct. Floor (`int(value)`) would systematically undercount — `"5.31x"` → `"5x"` (floor) vs `"5x"` (round half-up). Half-up is more honest.

**Finding 4 — No `~` approximation marker needed at2x.**

The rounding is an honest display choice, not a coarse estimate. Adding `~` would add visual noise without resolving any ambiguity that matters in this context. If a reader needs more precision they can open the table cell — the bar chart already provides visual granularity.

---

## Actionable recommendations (priority order)

1. **Add precision guard near1x** to fix the `1.01 → "1x"` bug:
   ```python
   def format_fold(value: float) -> str:
       if value >= 2:
           return f"{int(value + 0.5)}x"
       if value >= 1.5:
           return f"{value:.1f}x"
       return f"{value:.2f}" + "x"  # keep at least 1 decimal for near-1x   ```
   (Tune thresholds to taste — the key is you no longer silently drops `1.01` to `"1x"`.)

2. **Consider lowering threshold from 2x to 1.5x** to avoid rounding marginal improvements into the next tier.

3. **Deduplicate `format_fold`** — it's defined identically in `report.py:228` and `report_table.py:41`. Pull into a shared module.
