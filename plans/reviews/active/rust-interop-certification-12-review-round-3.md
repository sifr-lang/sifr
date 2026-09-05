# Rust Interop certification_12 — Round 3 Exact-Head Review (PR #3076)

## Verdict

**NOT SATISFIED**

The certification_12 milestone itself is technically sound and fully reproduces: the round-2 delta is exactly the three described changes, the durability observation is genuinely closed with a load-bearing mutation, and every count, checker, guardrail, and mandatory test passes on the exact published head. One actionable defect remains in committed content that **this PR introduces**: a named review artifact whose body is a 3-line conversational fragment, cited by the issue plan as a completed review.

---

## Head resolution

| Item | Value |
| --- | --- |
| Published head | `3867b21d56dc961b944c9259c632de2fc1d9d3c4` (== `headRefOid`, == branch tip) |
| Merge base / `origin/main` | `b3f663a174d170a99656e3221ffd952b81c4d51c` |
| Mergeable | `MERGEABLE` / `mergeStateStatus: CLEAN` |
| PR state | **`isDraft: true`**, `reviewDecision: ""` |
| Diff | 40 files, +1532 / −122 |

## Round-2 → round-3 delta — verified exactly as specified

`git show 3867b21d5` is precisely three changes, nothing more:

1. `_scenario_cli.py:105` adds `'target: "sifr_cli_noise"'` to the required-token tuple.
2. `_scenario_cli.py:248-254` adds the `"tracing excluded emission drift"` mutation (`sifr_cli_noise` → `sifr_cli_probe`, expecting `must contain target: "sifr_cli_noise"`).
3. Issue plan: `208` → `209`, plus the round-1/round-2 artifact records; new `rust-interop-certification-12-review-round-2.md`.

**The new guard is load-bearing, proven independently.** I extracted the head, deleted the `tracing::warn!(target: "sifr_cli_noise", …)` block from `src/bridges/cli.rs:69-73`, and the checker now fails where before it would have stayed green:

```
rust interop fixture matrix error: ecosystem_cli_certification:
  examples/cli_feature_package/cli.rs must contain target: "sifr_cli_noise"
```

Round-2 observation 1 is closed at the root. Counts moved consistently: fixture self-test `cases=209` (was 208), `_scenario_checks.run_self_test()` → `(117, None)` (was 116). No stale `208` remains anywhere in `plans/`, `docs/`, or `internal_docs/`. Mutation uses `replace(before, after, 1)`; the static emission requirement and the runtime `!trace.contains("excluded bridge event")` `ensure!` now form a complete pair.

## Blocking finding

**B1 — `plans/reviews/active/rust-interop-certification-11-review-round-5.md` is a 3-line conversational fragment committed as a review artifact, and the issue plan cites it as a completed review.**

The file is **added by this PR** (`git cat-file -e b3f663a17:<path>` → not on base/main) and its entire content is:

```
That was the leftover wait job from earlier; its result is already folded into the review
(certification suite: 6 passed, 0 failed, exit 0). No background work remains and no findings change.

Review stands: **`SATISFIED`** — head `4452643a94deb28068ea994780878f540b2e88bf` is safe to merge,
with the four non-blocking observations noted above.
```

It is a chat transcript tail, not a review: it references "the four non-blocking observations noted above" when there is nothing above, and carries no verdict body, evidence, commands, or findings. Every sibling artifact is a real review — 129, 130, 100, 57, 84, 147, 149 lines; this one is 3.

Meanwhile the issue plan (`:1327-1332`) links it as load-bearing provenance:

> `- [Final exact-head round 5](../../reviews/active/rust-interop-certification-11-review-round-5.md) reviewed published head 4452643a94…, returned SATISFIED, and recommended merge.`

So a tracking document asserts a completed exact-head review whose linked artifact does not contain one. No gate catches this — `check_stale_drafts.py` passes. This is doubly out of scope: it is certification-11 closeout content in a certification-12 PR. Round 2 noted the file as "not certification-12 work" (its observation 4) but did not inspect the body.

Not a code defect and it does not touch the certification evidence — but it is actionable, in committed content, introduced by this PR, and it degrades exactly the provenance integrity this milestone is about. Fix is one of: restore the real round-5 artifact, or drop both the file and the issue-plan link and fold the statement into cert-11's already-merged record.

## Non-blocking observations

1. **`check_fixture_matrix.py` remains at exactly 900 lines — zero headroom** (carried, round-2 obs 3). The guardrail fails on `> 900`, so the next line added re-triggers round-1's B1.
2. **Issue-plan wording imprecision** (`:1381`): "restored the checker **below** the hard cap." It is *at* the cap (900), not below.
3. **Per-crate package examples remain degenerate duplicates** (carried, round-1 obs 1 / round-2 obs 2, unresolved). `clap.sifr`, `tracing.sifr`, `tracing-subscriber.sifr`, `anyhow.sifr` all bind the identical `bridge.cli.parse_and_trace` with identical args; `anyhow_context(args)` is a misleading name for a CLI parse. Honest (the shadow stubs were the overclaim) but per-crate granularity is nominal.
4. **The committed round-2 artifact records `cases=208` and lists the exclusion gap as open**, while the same commit closes it to 209. Historically correct for head `e2c321a78` and reconciled by the issue-plan entry, but a reader diffing artifact against tree sees a discrepancy.
5. **Final checklist item unchecked** (`Run focused and authoritative local gates … merge the PR, and unblock only certification_13`). Legitimately open pre-merge.
6. **`cargo clippy -p sifr_driver --lib --tests -- -D warnings` fails with 18 errors** across 11 files — `tests/single_file_frontend.rs`, `build/rust_interop_panic_contract_tests.rs`, `tests/test_runner.rs`, and others. **None** appear in this PR's diff, and none are in the new `package_rust_interop_cli_ecosystem_support.rs`. Pre-existing; the documented gate `cargo clippy --workspace -- -D warnings` is clean (exit 0).
7. **PR is a draft** — must be marked ready for review before merge.

## Evidence — all on the exact published head

`git archive 3867b21d5 | tar -x -C /tmp/c12h`, `third_party/ruff` symlinked (submodule absent from archive).

```
check_compatibility_matrix.py   → rows=36 fixture_rows=36 categories=4        (self-test 5)
check_fixture_matrix.py         → fixtures=36 diagnostics=10 crates=44
                                  package_examples=61 scenario_examples=18    (self-test 209)
check_stable_support_claims.py  → claims=35                                   (self-test 33)
check_tiers.py                  → tiers=5 fixtures=36                         (self-test 6)
check_stale_drafts.py           → ok                                          (self-test 20)
runner.py                       → variants=10, failures=0, blocking_failures=0, non_blocking=0
_scenario_checks.run_self_test() → (117, None)
```

**Independent matrix recount (JSON parsed directly, not via checkers):**

```
rows 36
categories: supported 21 / supported-through-bridge 13 / unsupported-by-design 1 / future-owned-by-separate-phase 1
evidence:   passing 70 / planned 2
```

**Guardrails:**

```
check_file_size_guardrails.py              → PASS (3005 files, limit 900 lines)
check_hir_maintainability_guardrails.py    → PASS
check_sifr_driver_maintainability_guardrails.py → PASS
wc -l check_fixture_matrix.py → 900 ; _scenario_cli.py → 326 ; _binding_helpers.py → 101
cargo fmt --check --all → exit 0 ; git diff --check b3f663a17 3867b21d5 → clean
cargo clippy --workspace -- -D warnings → exit 0
```

**Both mandatory generated-package tests, on the live tree (inputs untouched by the unstaged change):**

```
cargo test -p sifr_driver --lib -- --ignored --exact --test-threads=1 \
  …cli_ecosystem_support::test_build_cli_tooling_probe_and_anyhow_adapter \
  …cli_ecosystem_support::test_check_direct_anyhow_surface_rejected

test …test_build_cli_tooling_probe_and_anyhow_adapter ... ok
test …test_check_direct_anyhow_surface_rejected ... ok
test result: ok. 2 passed; 0 failed; finished in 33.93s
```

**Provenance verified end-to-end.** `fixture.json` binds both directions to `suite_id: sifr_driver_generated_builds`, `step: crate_tests`, `profile: merge`, naming the real file and the two test names above. That suite is `status: blocking`, `executed_in_merge: true`, command `["test","-p","sifr_driver","--lib","--","--ignored","--test-threads=1"]` in **all four** profiles (`merge`, `create-pr`, `nightly`, `release`) — so the `#[ignore]`d tests are genuinely selected.

**Docs.** `docs/rust-interop.mdx:234-242` describes the exact-pinned build and states `anyhow::Error` stays internal, explicitly disclaiming "support for arbitrary CLI crate APIs or `anyhow::Error` values"; `:281` carries the single `supported-through-bridge` row.

## Scope proof

**Only test modules in `crates/` — no compiler behavior change:**

```
crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs          (2-line mod registration)
crates/sifr_driver/src/tests/package_rust_interop_cli_ecosystem_support.rs (new)
```

**Backend row is committed as future-owned; CLI alone is promoted.** At HEAD:

```
ecosystem_backend_certification → "category": "future-owned-by-separate-phase",
   "future_owner": "plans/issues/active/rust-interop-runtime-ecosystem-certification.md",
   positive/negative evidence both "planned"
ecosystem_cli_certification    → "supported-through-bridge", no future_owner
```

**The parallel promotion is unstaged only and confirmed absent.** `git diff -- …compatibility_matrix.json` shows the worktree hunk flipping backend to `"supported"` and dropping `future_owner`; it is not in HEAD. Substantiating both directions: the exact head passes `check_compatibility_matrix.py`, while the **live** tree fails solely on that hunk —

```
error: ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence
error: compatibility category is unused: future-owned-by-separate-phase
```

**Excluded paths absent from HEAD** (`git cat-file -e`): `.cert5probe`, `.agent`, `plans/phases/43_interoperability.md`, `logo 06.48.53.webp`, `docs/logo/logo.webp 08-03-09-514.webp`, `verification/areas/algorithmic_compatibility/corpora/leetcode`.

**Submodule pointers unchanged vs base:** `git diff b3f663a17 3867b21d5 -- editor_integrations verification/areas/algorithmic_compatibility` → empty, despite both showing ` M` in the live worktree.

The only in-PR scope leakage is B1's certification-11 artifact.

## Publication / merge recommendation

**Hold — one small commit away from merge.** Do not publish as-is: resolve B1, then this is `SATISFIED`. The certification_12 work itself needs nothing further — the CLI/tooling bridge contract, the exact-pinned real-crate graph, the `SIFR-RUST-TYPE-0001` negative evidence, provenance, matrices, docs, guardrails, and both mandatory tests all reproduce exactly on `3867b21d5`, and the round-2 durability observation is closed with genuine mutation coverage.

Before merge: (a) fix B1 — either commit the real cert-11 round-5 review or remove the stub file together with its issue-plan link; (b) mark PR #3076 ready for review; (c) keep the unstaged `ecosystem_backend_certification` promotion out, as it correctly is. Optionally address observations 1–2 (relocate one more responsibility out of the 900-line checker; correct "below the hard cap" to "at the hard cap"). The final checklist item can be ticked as part of the merge.

No repository files were modified, staged, or created during this review. All checker, guardrail, and mutation validation ran against `/tmp/c12h` and `/tmp/c12mut` (extracts of the exact committed head); the cargo test and lint runs used the live tree, whose Rust inputs are untouched by the unstaged worktree change.
