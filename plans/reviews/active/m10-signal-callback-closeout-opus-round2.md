**PR-ready.**

Both round‑1 findings are resolved and the round‑2 material is internally consistent:

1. **Tracker row** — M10 now reads `merged` with dual-PR evidence: `PR #2892 · sha=4a5b16c` (M10a signal native-boundary migration, Opus round 6) and `PR #2894 · sha=330e277` (certification split, Opus round 1). Both shas match the recent commit log (`4a5b16cf3` migrate, `330e27765` split). Ordering with M11/M12 `planned` is correct.

2. **Split is coherent across the manifest** — `callback_subscription_matrix` split into:
   - `callback_subscription_core` → claimed by `_sifr.signal` (`state = "closing"`, M10-owned). Matches the reason text ("M10b split and claimed the stdlib-owned callback_subscription_core evidence").
   - `callback_subscription_ecosystem` → carried by `_sifr.python` (`state = "retained"`, future-owned). Matches "ecosystem subscription crates remain future-owned."
   
   No row references the now-split `callback_subscription_matrix` name, so the rename is clean on the evidence shown.

3. **Opaque reference** — `opaque_resource_matrix` confirmed present in both `rust_interop_compatibility_matrix.json` and `rust_interop_fixture_matrix.json`, so the `_sifr.python` certification reference is live, not dangling, and correctly M11-scoped.

4. **Retained/closing rows are appropriately scoped for a pilot closeout** — `_sifr.signal` (closing, records intrinsic-name closure), `_sifr.runtime::observability_glue` (retained-by-design), and `_sifr.python` (retained, `removal_criteria` defers to M11 Python Interop Adapters). None of these block M10.

One thing I could not independently confirm without tools (and which was *not* included in the packet, unlike the opaque verification): that `callback_subscription_core` and `callback_subscription_ecosystem` actually resolve to rows in the certification matrices. Everything referencing them is consistent, so I'm treating this as satisfied — but if you want the same rigor applied to the split as to the opaque row, a one-line grep for both ids in the certification data would close the loop before merge.
