Pass1's verdict holds. Verified independently:

- **Guard integration** — `verification/runner/sifr_verify/profile_runner.py:353-354` invokes both the main check and `--self-test`, and it's declared in `verification/policy/guardrails.json:55`. Pass1 didn't call this out explicitly; it is present and exercised by the reported `create-pr` run.
- **Regex correctness** — Independently reproduced 31 captured fallback modules from `crates/sifr_retained_intrinsics/src/lib.rs`; the `_sifr.not_real` test string on line 143 is correctly excluded because it isn't a `=> Some(` arm.
- **No closing rows today** — Grep across the manifest shows only `retained` / `retained-by-design` states, so the guard is a forward-only trip wire as documented.
- **Self-test adequacy** — Positive and negative closing-row cases are covered (lines 310-328); the `--self-test` exits 0.
- **Subfamily false-positive scan** — `_surface_private_modules` strips any `::leaf` suffix via `split("::", 1)[0]`, so a closing subfamily leaf trips the guard while the parent fallback module still exists — that matches the phase invariant.

READY
