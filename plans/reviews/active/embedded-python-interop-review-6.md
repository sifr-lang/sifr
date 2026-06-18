## Review (round 6)

**1. New policy clarity (line 43): strong.**

The Core Decisions bullet is well-placed, names the four forbidden patterns (back-compat shims, legacy compat layers, fallback paths, degraded modes), and gives the positive framing ("correct production contract directly"). It pairs cleanly with the existing no-fallback rules at lines 44, 142, and 895.

**2. Conflict scan — one wording fix needed.**

| Phrase | Location | Conflict? | Reasoning |
|---|---|---|---|
| "as compatibility paths only" | line 479 (Array Interface) | **Yes** | "compatibility paths only" reads as "legacy-only fallback," which contradicts the new bullet. The intent is "interchange protocol for producers that don't expose Py_buffer/Arrow/DLPack" — a peer protocol, not a legacy/degraded mode. |
| "CPython-compatible packages" | line 553 | No | Means *compatible with CPython*, not Sifr back-compat. |
| "No host-global Python fallback" | line 142 | No | Forbids a fallback — reinforces the new bullet. |
| "No silent fallback from zero-copy…" | line 44 | No | Same — explicit "no fallback" rule. |
| "no silent zero-copy fallback to copying" | line 895 | No | Same — exit-gate reinforcement. |

**3. Recommended one-line change before commit.**

Line 479 — drop "as compatibility paths only" and re-frame as a peer protocol; preference is already stated separately at line 486 ("Prefer `Py_buffer`, Arrow, or DLPack when available.").

```
Support `__array_interface__`, `__array_struct__`, and `__cuda_array_interface__` as additional zero-copy interchange protocols.
```

Same line count (file stays at 899 lines, well under 900).

**4. Status: not ready to commit yet.**

Make the line 479 rewording, then commit. Everything else is consistent with the new policy and the wording at line 43 is clear and strong.
