All three fixes are clean. The phase remains ready to close — no blockers introduced.

**What I verified:**

- **SIFR-OWN-0001 / 0005 / 0013** (sampled): Syntax is valid — erroneous block calls `consume(items)` then `len(items)` (use-after-move), fixed block reorders to `len(items)` first. No `taken = own items` expression remains. The "Why It Happens" and "How To Fix It" prose is correctly generic for the OWN family.

- **SIFR-IMPORT-0008**: "Why It Happens" now names the actual policy (bare CPython stdlib names reserved for project modules, use `sifr.*` explicitly). Example is precise: `from math import sqrt` → `from sifr.math import sqrt`.

- **SIFR-PACKAGE-0703**: Erroneous code shows `manifest = "../old/sifr.toml"` (stale pointer drift), fixed code shows `manifest = "sifr.toml"`. Directly illustrates the diagnostic's scenario.

One pre-existing observation (not introduced by these fixes): several OWN pages (e.g. OWN-0005 "Immutable parameter is mutated", OWN-0013 "Non-IPC-serializable value") share the identical use-after-move code example rather than code tailored to their specific error. That mismatch predates this pass and is not a blocker for closure — but worth a future cleanup ticket.

**Phase is ready to close.**
