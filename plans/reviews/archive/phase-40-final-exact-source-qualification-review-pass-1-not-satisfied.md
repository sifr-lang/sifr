## Review: Phase 40 final exact-source qualification evidence — PR #3054 @ `3e965ea4d`

I modified no tracked files. (Running the documentation area runner rewrote a gitignored `target/` artifact only; `git status` remains clean apart from the pre-existing untracked active review file.)

### Exact head and scope
`gh pr view 3054` → `headRefOid` = `3e965ea4dc9b5093455110aac690536090d5cf7f` == local HEAD; OPEN, MERGEABLE, base `main`, 3 files **+210/−5** — matches the local diff exactly. Documentation-only: one ledger file, two new archive files. No Rust/Python/script/config touched, so the Rust-interop prohibition is respected. `git diff --check` clean; both new files end with a newline; `check_file_size_guardrails.py` → PASS (2952 files).

### Independently reproduced
| Claim | Result |
|---|---|
| Release report SHA-256 `faa68444…6b0e03` | ✓ matches; also byte-identical to `/tmp/sifr-phase40-release-profile-c9d611fb7c-final/` original |
| `report_id` `release-c9d611fb7c7c-fa3d95c04f8a`, `overall_status: pass`, `source.clean: true`, commit `c9d611fb7c` | ✓ all four |
| Rust result SHA-256 `be24b69a…ba87a`, variants 10, failures 0 | ✓; digest binds in all 5 `rust_interop_checks` suite rows **and** the `result_artifacts` row, and is byte-identical to `target/verification/areas/rust-interop-release-results.json` in the source checkout — custody chain intact |
| 24 lane steps, all `pass` | ✓ |
| 7,610.91 s | ✓ exactly (`release.latest.time`); sum of step `elapsed_ms` = 7610.71 s, consistent |
| Python interop 25/25, perf 8/8, dev tooling 48/48, docs 2/2, E2E 674/674, hardening 290/0 | ✓ all from persisted artifacts (`hardening_summary.variants: 290`, log line `674 pass tests completed (674 passed, 0 failed)`) |
| Sequential-vs-parallel equivalence | ✓ DET-0002 pass, signature `1f8b1cadc4f48ec8` both ways |
| Source checkout clean, detached at exact SHA, submodules match report **and** index | ✓ all 10 submodule SHAs identical across three sources |
| Index SHA-256 `503f4fcc…04703`, 20 artifacts, 533,998,429 bytes, expiry `02:17:30Z`–`02:32:17Z` | ✓ all |
| Custody replay: 20 payloads by name, size, SHA-256 | ✓ **20 verified, 0 bad** |
| 6 upload ids + index id `8710544640` | ✓ exact match against `gh api .../artifacts` (7 total) |
| Run 30416219284: success, `workflow_dispatch`, headSha `c9d611fb7c` | ✓; workflow enforces `dispatch ref must resolve to source_commit`, so the "dispatched through `main`" note is honest and the identity binding is sound |
| PR #3052 merge = `c9d611fb7c…`; PR #3047 merge = `8a23f9086…` | ✓ both, and both ancestors of `origin/main` |
| Live `channels.json` still schema v1 @ `71b32439…4bf9ef` | ✓ re-downloaded from the `channels` release tag; identical. No production mutation |
| `stable-release` env: `protection_rules: []`, no reviewers; sole collaborator `yaseralnajjar` | ✓ — the "blocked on a genuinely distinct human reviewer" conclusion is correct and does not bypass the boundary |

Checklist flips are all justified: milestone 40.4's three items are backed by merged `sifr-vscode` PR #12 / `editor-integrations` PR #10 (both submodule pointers present at `c9d611fb7c`) and approved pass 5; isolation by merged PR #3039 (`d8dd28a80`); rollback wiring by merged PR #3047. Status stays **"In progress"**, all five Final Phase Closure boxes stay unchecked, and 11 items remain open — no overstatement of Phase 40 completion.

### Actionable finding

**1. Ledger states the authoritative run had one advisory; it had two.** `plans/issues/active/phase-40-stable-channel-ga-execution.md:1412` asserts unconditionally: *"The only advisory was the already indexed warm wall-time target."* The canonical lane report for that exact run records:

```
advisories: ['warm wall-time budget exceeded',
             'group skew is high; investigate batching balance or fixture clustering']
```

`observations.group_skew_ratio` = 16.0 (`largest_group_fixtures` 16 vs `median_group_fixtures` 1), which trips the emitter at `verification/runner/sifr_verify/reports.py:171`. The group-skew advisory is non-blocking and long-precedented in this repo ("warm-cache/group-skew advisories" appears throughout earlier archives), so nothing is being hidden — but in a document whose sole function is exact evidence custody, "the only advisory was X" is a false completeness claim about the final GA-qualifying run. The archived evidence file's variant at line 101 ("the 7,610.91-second cold wall time produced *only the declared warm-target advisory*") scopes "only" to the wall time and is defensible; the ledger's does not and is not.

### Nonblocking observations

- The `release` budget records `cold_wall_time_target_minutes: 60`, and the run took 126.85 min. No cold advisory fired because `build_advisories` only ever checks the warm target (`reports.py:144`); the report exposes `within_warm_budget` with no cold analogue. The evidence doesn't claim otherwise, but "every blocking gate and budget passed" sits next to a recorded `within_warm_budget: false` — the distinction between blocking gates and advisory budgets is left implicit.
- The index artifact's own expiry is `2026-08-28T02:32:52Z`, slightly later than the quoted "latest expiry" `02:32:17Z`. The evidence scopes that range to *indexed payloads*, so it is correct as written; worth knowing that the index itself outlives the range.
- `generated_code_quality` release-full carries `expected_failures: 3` of 7 variants (blocking 0). The evidence names the lane without a count, so nothing is overstated, but the governed expectations aren't surfaced in the archive.
- All payload custody lives under `/tmp` with a 2026-08-28 GitHub retention expiry; after that date the digests in this archive become unreplayable from the run. Recording that is arguably out of scope for this PR.

Finding 1 is a wording correction to a single sentence, but it is a factual inaccuracy in the evidence record, so the satisfaction bar is not met.

VERDICT: NOT SATISFIED
