All three pass-3 polish suggestions are resolved in the current document:

1. **`metrics` default features** (line 273): The binding notes now read *"Default features are accepted because they expose only the metrics facade API; no exporter, recorder, or integration features are enabled or implied."* — fully addressed.

2. **"executor-like features" in futures-util** (line 265): The trailing "or executor-like features" phrase has been dropped. The row now reads *"do not enable `channel`, `compat`, `io`, or `sink` features"* — four concrete named exclusions, nothing ambiguous.

3. **"add/directly use" for tracing** (line 272): Now reads *"add `tracing = 0.1.44` as a direct workspace dependency"* — unambiguous, matches the resolved register at line 896.

No new blockers or meaningful polish items found. Technical correctness, cross-document consistency, and lock discipline remain intact.

**PASS**
