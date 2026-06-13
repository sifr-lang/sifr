

Let me think through this from an engineering perspective, considering what questions developers actually need answered when evaluating performance.

**The primary questions engineers ask first:**

1. "Will this make my code faster?" → Need to see absolute runtimes, not just ratios
2. "How much faster?" → Speedup is secondary confirmation
3. "Does it scale?" → Runtime vs input size shows scaling behavior
4. "What's the memory cost?" → Valid concern, but noisy data
5. "Can I trust these numbers?" → CV/stddev matters for reproducibility

---

## Recommended Primary Charts

**Chart 1: Mean Runtime vs Input Size (Primary)**
- Overlay Python + Sifr on same plot- Y-axis: absolute time (ms), linear scale unless range requires log
- X-axis: input size
- Engineers can directly read: "At size N, Python takes X ms, Sifr takes Y ms"
- Speedup is implicitly visible as the gap between lines

**Chart 2: Memory vs Input Size (Primary)**
- Same dual-line format as above
- Label clearly: "Peak RSS (includes OS overhead)"
- Caveat in chart footnote or tooltip**Why not make speedup the primary chart:**
- Ratios are derived from raw times. Raw times are ground truth.
- A 100x speedup looks impressive but means nothing if both are under 1ms
- A 1.5x speedup on a 1-hour job saves 20 minutes — that's significant in a way speedup alone can't convey
- Engineers calibrate expectations against absolute scale, not ratios

---

## Where to Put Speedup

Don't make speedup the central graph — include it as:

1. **Annotation overlay on the runtime chart** — horizontal dashed lines at speedup = 2x, 10x, 100x where the gap is meaningful
2. **Below-fold secondary chart** — Speedup vs input size as a supporting chart
3. **Summary badge** — "Mean 5.2x faster across all sizes" with max/min shown

This way speedup answers "how much?" without obscuring the "is it fast enough?" question from raw times.

---

## Memory: Strong Enough to Graph Prominently?

**Yes, but with caveats.** Memory is a legitimate concern for compiled languages:

- Show the chart. Engineers will look for it.
- Label axis as "Resident Set Size (MB)" with footnote: "RSS measured by hyperfine; includes shared library overhead and OS allocator variance"
- Don't suppress it because it's noisy — acknowledge the noise and let engineers evaluate

**What RSS doesn't capture well:**
- Heap vs stack usage within the program
- Memory freed but not returned to OS (common in Rust)
- Allocation patterns that affect long-running processes

For a static benchmark report, RSS is fine — just be transparent.

---

## CPU/Stability: Table + Badges, Not Charts

**CPU breakdown (user% vs system%):**
- Good for debugging but not primary decision data
- Put in the detailed table per-problem
- Not worth a chart — it confuses the main story

**Stability (CV, stddev):**
- Use as a filter or badge, not a chart
- Example: Show "CV < 5%" badge in green, "CV > 15%" badge in yellow
- Engineers want to know "can I reproduce this?" — a badge communicates "yes/no" faster than a chart
- Optionally: small jitter plot or box plot below the main chart for problems with high variance

---

## Recommended Layout Per Problem Card

```
┌─────────────────────────────────────────────────────────────┐
│ Problem: Two Sum                                            │
│ Mean Speedup: 4.2x │ Peak Speedup: 8.1x │ CV: 2.3%           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [CHART 1: Runtime vs Input Size]                           │
│  — Sifr line (primary, thicker)                             │
│  — Python line (overlay, distinct style)                    │
│  — Annotation: "4.2x faster at N=10k"                        │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ [CHART 2: Memory vs Input Size]                            │
│  — Sifr RSS line │
│  — Python RSS line                                          │
│  Footnote: "RSS includes shared library overhead"           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  [OPTIONAL: Speedup vs Input Size]                          │
│  — Only if speedup varies significantly with size           │
│  — Otherwise, the badge above suffices │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Table: Mean │ Median │ Min │ Max │ StdDev │ CV │ Speedup    │
│  ─────────────────────────────────────────────────────────  │
│  Size  Sifr Py   Sifr Py  Sifr Py ...  │  Speedup            │
│100   0.1  0.4  0.1  0.4  ...       │  4.0x               │
└─────────────────────────────────────────────────────────────┘
```

## Charts to Avoid

1. **Speedup-only chart without context** — hides whether the improvement matters
2. **Log-scale time axis by default** — log is fine, but show linear too if range allows; log compresses the visual story
3. **Bar charts for time comparison** — line charts show scaling better; bars are for discrete comparisons
4. **Normalized time (Python =100%)** — same issue as speedup-only: hides absolute scale
5. **Too many overlaid lines** — if comparing 5+ implementations, use a small-multiples layout instead

---

## Summary: Two Primary Charts

| Chart | Purpose | Design Notes |
|-------|---------|--------------|
| Mean Runtime vs Input Size | "Is Sifr faster?" | Both languages overlaid, annotate key speedup moments |
| Memory vs Input Size | "What's the memory cost?" | RSS labeled, footnote for caveat |

Speedup lives as: badge + annotation + below-fold chart (if interesting).

What do you want to refine — the per-card layout, the category overview, or the underlying data model? Bringing in any example problems and their typical size ranges would help calibrate axes and scale choices.
