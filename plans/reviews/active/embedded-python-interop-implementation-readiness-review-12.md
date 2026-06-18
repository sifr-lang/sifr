Read line 184 (the updated `PyValue` enum) and line 316 (the Sifr-to-Python arguments paragraph).

1. **Resolved.** `IntExact(SifrExactInt)` and `IntFixed(SifrFixedInt)` cleanly remove the `PyInt` placeholder and mirror the established Python-to-Sifr terminology in the Conversion Rules table (line 301: "exact `int`" vs "fixed-width `py.to[int32]`/etc."). The two integer flavors are now explicit on both directions of the boundary. No new implementation ambiguity — `SifrExactInt`/`SifrFixedInt` are recognizable references to the Sifr integer kinds already named in this phase, and this is the conceptual runtime surface (line 169), not the final type signature.

2. **Yes.** The phase is implementation-ready and self-contained: lines 11–25 enumerate the contracts this phase owns without deferring, lines 43–45 forbid fallback/legacy/degraded modes, lines 850 and 891 reassert no silent zero-copy fallback, and the milestone DoDs each carry positive + negative fixtures with explicit gates. Trust, blocking, callbacks, zero-copy, cleanup, and certification all have binding rules plus a concrete verification surface.

3. None required.

4. **Ready to commit.**
