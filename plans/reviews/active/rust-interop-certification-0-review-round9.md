# Round-9 final PR-boundary audit — `certification_0`

> Editor's provenance note (2026-07-26): the sentence below describes the
> audit-time state. The reviewer emitted its report out of tree; a concurrent
> coordinator later captured that output in this file and committed it while a
> separate final audit was still running. This note makes that capture sequence
> explicit and resolves the apparent self-reference without changing the
> reviewer's findings or verdict.

Read-only. I modified nothing in the repository; every command above was a check, a report read, or a diff. `plans/reviews/active/rust-interop-certification-0-review-round9.md` does not exist during this audit (only the out-of-tree `.claude.log` companion), so no writer race is possible.

## Evidence-integrity re-check (rounds 1–8): coherent

| Artifact | Terminal lines | Verdict |
| --- | --- | --- |
| round1–round7 | exactly one each | all `NOT SATISFIED` |
| round8 | exactly one | `SATISFIED` |

`grep -cE '^(NOT )?SATISFIED$'` returns 1 for every artifact and each file's last line is that verdict — no spliced bodies, no orphaned verdicts, no forged `SATISFIED`. Round 6 is one coherent `NOT SATISFIED`, round 7 one `NOT SATISFIED`, round 8 one `SATISFIED`, exactly as the brief states.

## Authoritative reports: every claimed number reproduces from the artifacts

**create-PR** (`create-pr.latest.json`, 00:24–00:25):
- 23 lane steps, zero non-pass.
- `cargo_cache_setup elapsed_ms=578 budget_ms=300000 enforcement=advisory status=pass`.
- `rust_interop_checks elapsed_ms=4732 budget_ms=10000 enforcement=blocking status=pass` (5,268 ms headroom, matching the README prose exactly).
- E2E `131 pass tests completed (131 passed, 0 failed)`; hardening 6 variants, 0 failures.

**merge** (`merge.latest.json`, generated 2026-07-27 01:38:29):
- 23 lane steps, zero non-pass; `real_seconds=4337.38`.
- `cargo_cache_setup elapsed_ms=658` advisory pass; `rust_interop_checks elapsed_ms=4394 status=pass`.
- hardening `variants=261, failures=0, blocking_failures=0, non_blocking_failures=0, skipped=0`.
- E2E `674 pass tests completed (674 passed, 0 failed)`.
- Advisories exactly the two disclosed: `warm wall-time budget exceeded`, `group skew is high…`. No blocking advisory.

**nightly / release** (exact-state): `rust_interop_checks` 4,161 ms and 3,880 ms, both pass, cache setup 495/812 ms; both then stop at `algorithmic_compatibility_checks` with `variants=412, failures=20, blocking_failures=20`. Identical 20 fixture slugs in both logs, all `bucket=algorithmic_compatibility`, diagnostics are `Any`/typing/container-surface classes with no relation to this diff. The README and the certification issue disclose this inline as pre-existing and out of scope, and the separately-owned follow-up issue preserves the taxonomy evidence. Accurate.

## Independent re-validation I ran on the exact current tree

- `check_stable_support_claims.py --self-test` → `cases=33`, exit 0; real run → `claims=23`, exit 0.
- Complete area: `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0` (matrix, tiers self-test `cases=6`, compat `rows=36 fixture_rows=36 categories=4` + self-test `cases=5`, stale-drafts `cases=20`, stable-candidate both cases).
- Runner `--self-test`: all eight sections pass, including the Rust-interop profile-execution and profile-schema sections.
- `cargo fmt --check` clean; `cargo clippy --workspace -- -D warnings` exit 0; `cargo check --locked --offline -p sifr_rust_interop_catalog` clean; `git diff --check 7554f89b5` clean; HIR guardrails PASS; `sifr_driver` guardrails PASS; file-size guardrails PASS (2833 files, limit 900).

## Substance spot-checks

- **Inventory prose is exact.** Recomputed from the data files: 36 compatibility rows / 36 fixture manifests; categories 17/5/1/13; execution kinds 13 `cargo-probe`, 4 `compiler-diagnostic`, 10 `contract-only`, 9 `runtime-observed`; 47 `passing` / 25 `planned` evidence directions; 7 runtime deferrals; 23 claims; 44 catalog aliases, all optional and `=`-pinned; `REQUIRED_FIXTURES` is 36 and contains both new runtime rows.
- **Lockfile policy claim verified independently.** No package removed. Exactly ten sole-version advances — `hashbrown` 0.17.0→0.17.1, `rand` 0.10.1→0.10.2, `js-sys` 0.3.95→0.3.103, `wasm-bindgen`+3 macro/support crates 0.2.118→0.2.126, `futures-core`/`futures-channel`/`futures-sink` 0.3.32→0.3.33 — matching the issue text one-for-one. Of 324 added packages, all but `cxxbridge-cmd` are reachable from `sifr_rust_interop_catalog` in the `--all-features` resolve graph; `cxxbridge-cmd` is a direct lock dependency of catalog-pinned `cxx =1.0.198` (cargo's resolve graph omits its bin edge), so the "nothing unreachable" claim holds.
- **Profile budgets.** All five profiles declare `cargo_cache_setup {300000, advisory}`; only create-PR declares the blocking 10,000 ms `rust_interop_checks` budget, which is precisely how the README scopes it. `stable-candidate` is selected in create-pr, merge, nightly, and release (not in `python-interop-live`, which is not one of the four authoritative profiles).
- **Runner change is sound.** The prelude runs as step 1 with `CARGO_NET_OFFLINE` popped, aborts the profile on failure, and offline is forced only afterward; `cargo_setup.py` accepts exactly one canonical command and rejects non-locked/non-boolean/non-canonical policy. `profile_policy.md` documents the prelude as the only registry-network opportunity.
- **Phase 40 handoff is one-directional and consistent.** Phase 40 now *confirms/consumes* the registration; `certification_0` *owns* it. No duplicate-registration instruction survives in either document.

## Findings — all LOW, none blocking

1. **LOW (inherent) — the authoritative merge report predates two doc-only edits by ~1 minute.** `merge.latest.json` is 01:38:29; `verification/areas/rust_interop/README.md` and `plans/issues/active/rust-interop-runtime-ecosystem-certification.md` are 01:39:27, because they record that run's own numbers. Unavoidable ordering. Coverage of the final bytes comes from the focused post-merge bundle, which I re-ran here in full (file-size guardrail, complete area 10/10, fmt, clippy, diff-hygiene) — all green. No action.
2. **LOW — round-8 findings 2/3/4 remain open** (item-line-then-plain prose merge; connective allowlist gaps; `docs/`-only sweep root). Reconfirmed same class and same standard: the canonical claims table is marker-delimited, unique, ordered, table-only, and canonical-doc-only, and the README now discloses the docs-wide sweep as a keyword/Markdown-structure tripwire rather than a claim authority. No live violation in any real document.
3. **LOW — round7.md finding 2 still reads as self-contradictory** against the file a reader holds (round-8 optional 1, unaddressed). Editorial only; the verdict trail is unambiguous.
4. **LOW — `/tmp` hardcode** in `fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:13`. Inert while both evidence directions are `planned`; owned by `certification_7` per the brief.
5. **LOW — direct area invocation depends on a warm cargo cache.** `_crate_catalog.validate_crate_catalog` runs `cargo fetch --locked --offline`; outside a profile the prelude has not run, so a cold machine gets an actionable "not cacheable offline" failure. Documented behavior in the README, correct step ordering inside all profiles.
6. **LOW — maintainability headroom.** `check_stable_support_claims.py` 836/900, `profile_runner.py` 869/900. Guardrail PASS; a `_prose_scope.py` split is the natural next change.

None of these requires an implementation change before the PR. Every blocker raised in rounds 1–7 is closed and defended, the exact-state authoritative merge evidence is complete and internally consistent with the recorded prose, and the extended-profile failures are correctly scoped out with preserved evidence and a separately-owned issue.

SATISFIED
