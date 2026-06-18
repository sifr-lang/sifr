I reviewed the round-8 file.

1. **Backward-compat/fallback conflicts:** None. The policy at line 43 ("No backward-compatibility shims, legacy compatibility layers, fallback paths, or degraded modes") is consistent with line 44 (no silent zero-copy fallback), line 142 (no host-global Python fallback), line 354 (no `py.blocking` alias), and the Exit Gate's "no silent zero-copy fallback to copying." Nothing contradicts.

2. **Wording:** One minor leftover — the Conversion Rules table at line 311 still says "array-interface compatibility" for numpy arrays, while Core Decisions (line 39) and Exit Gate (line 895) now name the protocols precisely. For full parity you could replace it with `__array_interface__`/`__array_struct__`/`__cuda_array_interface__` (or just "array-interface protocols"). Optional, not blocking.

3. **Otherwise: ready to commit.**
