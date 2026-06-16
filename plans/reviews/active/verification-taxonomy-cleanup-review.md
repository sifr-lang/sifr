## Code Review: verification taxonomy guard

I grounded each finding against the actual repo (guard at `verification/areas/coverage_matrix/checks/verification_taxonomy.py`, wired at `verification/areas/coverage_matrix/manifest.json:72`). Guard placement (`parents[4]` → repo root) is correct, the full scan is non-vacuous (all `ACTIVE_ROOTS` resolve and exist), and both the self-test and full scan pass when invoked manually.

### Blocking

**1. The guard's negative self-test never runs in the gate — guard has zero regression coverage.**
- `verification_taxonomy.py:86-87` — `run_self_test()` only executes under the `--self-test` flag.
- `manifest.json:73` — the case is registered as a `area-check` entry, and the adapter (`verification/runner/sifr_verify/area_adapter.py:196`) invokes it as `[sys.executable, str(entry)]` with **no arguments**. So the harness only ever runs the *positive* full scan; `run_self_test()` is dead code from the gate's perspective. A repo-wide grep confirms nothing else invokes `--self-test`.
- Contrast the sibling `coverage_matrix_self_test.py`, which runs its negatives in its **default** `main()` and is wired as its own case (`coverage_matrix_negative_self_tests`). That is the established convention here.
- Consequence: a future edit that weakens a `TEXT_PATTERN` or broadens an `ALLOW_TEXT_PATTERN` so it stops catching real violations would still go green — the only thing protecting the patterns (the self-test) is the one thing not exercised. Fix: either make the guard run `run_self_test()` by default before/after the scan, or register a second manifest case that invokes it with `--self-test`.

### Non-blocking (worth noting, not gating)

**2. Whole-line allow-skip is a latent false negative.** `verification_taxonomy.py:139` skips the *entire line* when any allow pattern matches. A real label co-located with an allowed token — e.g. `// compiler phase trace; see Milestone 7` — escapes detection, because `compiler phase` whitelists the whole line including `Milestone 7`. Not triggered on current content (I searched the scanned roots for co-located cases and found none), but it will mask violations as files grow. Prefer stripping/masking only the allowed span, then re-checking the remainder.

**3. Scope boundary leaves sibling surfaces unguarded.** `ACTIVE_ROOTS` excludes `internal_docs/`, `plans/`, and `docs/`. The PR's own renames show taxonomy lived there (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` → `…_guardrails.md`), yet nothing prevents it from regressing back. This matches the stated scope ("active verification and crate surfaces"), so it's intentional — just confirm that's the deliberate boundary and not an oversight, since the cleanup itself touched `internal_docs`.

Findings #2 and #3 are judgment calls you can defer; **#1 is the one I'd resolve before merge** since the guard is the deliverable and its correctness is currently unverified in the gate it runs in.
