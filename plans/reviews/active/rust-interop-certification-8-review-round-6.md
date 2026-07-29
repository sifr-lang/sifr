# Verdict: SATISFIED TO MERGE

The perf-only failure is rigorously excluded as a certification_8 regression, on four independent legs. No PR-attributable blocker remains.

## 1. Reachability proof — the changed code cannot cost what is being measured

The only production delta (`git diff origin/main...HEAD`) is 9 lines:

- `crates/sifr_driver/src/build/rust_interop.rs:654` — call renamed `record_declared_bridge_native_links` → `record_declared_native_links`.
- `crates/sifr_driver/src/build/rust_interop/trust_validation.rs:11-32` — removes the `if !uses_bridge_root(...) { return; }` gate and reworded evidence text.

Entry is gated twice. `apply_package_rust_interop_metadata` (`rust_interop.rs:90-95`) returns immediately when `generated.interop.rust.declarations.is_empty()`, and it has exactly two non-test callers (`build/entrypoint.rs:611`, `build/single_file_interop_cache.rs:79`), both post-codegen and the latter memoized behind a process-global `OnceLock`. Inside, the removed gate's only downstream work is one `canonical_trust_target_path` call (`target_resolution.rs:73-78` — `declaration_paths().first().dotted()`, a string build) plus a loop over `package.manifest.trust.native_links`, which is **empty for every failing fixture** — only the new `advanced_data_runtime` example declares native links. No I/O, no rustc invocation, no allocation of consequence.

The four failing fixtures (`benchmark_manifest.json`) are `demos/project_graph/main.sifr`, `crates/sifr/tests/e2e/pass/arithmetic.sifr`, `crates/sifr/tests/e2e/fail/type_mismatch.sifr`, and `verification/areas/performance/query_projects/lsp/main.sifr` — none has a `sifr.toml` declaring Rust interop or native links. A microsecond-scale, once-per-process string format cannot produce +164 ms / +205 ms medians.

## 2. Magnitudes and dispersion are incompatible with a code regression

Same compiler build across all six evidence files (identical `compiler_fingerprint`, `cargo_lock_sha256=602c5cc84154`):

| case | 6 medians (ms, same binary) | budget |
|---|---|---|
| check-single-file-001-arithmetic | 2484.8 / 1581.5 / — / 2586.7 / 1738.2 / **1338.9** | 1334.1 |
| check-project-004-project-graph | 2274.3 / 1507.0 / — / 1449.7 / 1817.2 / **1521.2** | 1357.5 |
| diagnostic-non-regression-002 | 2275.8 / 2160.4 / — / 1445.3 / 1540.3 / **1540.8** | 1336.0 |

The arithmetic case missed by **4.7 ms (0.35%)** while the same case, same binary, spans 1339–2587 ms — a ±93% dispersion against a budget with ~10% headroom. Meanwhile the heavy `build-*` cases ran at **~25% of their baselines** (8504 ms vs 33866 ms baseline; 6856 ms vs 32458 ms) — the compiler is measurably *not* slower; only the sub-2 s process-startup-dominated cases miss. Both lanes also emitted the `warm wall-time budget exceeded` advisory, including the fully green create-PR lane.

## 3. The LSP case is main-attributable stale-baseline drift, not load and not this PR

`perf.lsp.diagnostics.document` baseline is `median_ms=0.91`, `captured_at_unix=1778968823` = **2026-05-16**. Main commit `be2df3d913` "fix(perf): isolate local LSP benchmarks" (**2026-07-22**, ancestor of `origin/main`, one week before this branch) changed the measurement itself: `workspace_mode: "isolated"` now synthesizes a temp **Sifr package** (`sifr.toml` + `Cargo.toml` + `Cargo.lock`) per run, so diagnostics now include package/manifest resolution. Baselines were **not** regenerated in that commit. Every sample since is ≥4.4 ms (4.48 / 4.78 / 4.46 / 5.20 / 6.59 / 14.33), i.e. a methodology floor of ~4.5 ms against a 5.91 ms threshold — which is why this case passes 4 of 6 times, including in the green create-PR lane. The 6.59/14.33 excursions are load on top of a pre-existing main-branch floor.

## 4. All functional evidence is green, including the certification-specific surface

`target/validation_lane_reports/merge.latest.json`: 10 of 11 steps `pass`, only `performance_budget_checks` `fail`. Log: `python interop verification ok: variants=25, failures=0` (line 305), `rust interop verification ok: variants=10, failures=0` at **6830 ms** (lines 337-338), `developer tooling verification ok: variants=32, failures=0` (line 453), `performance benchmarks passed` (line 486) — the run itself succeeded; only the budget comparator failed. `create-pr.latest.json` is green across all 24 steps *including* `performance_budget_checks`. The 6.83 s vs 10.54 s spread on an identical 10/10 Rust-interop suite is itself a load fingerprint; `lane.merge.log.evhqrj1m` (17:38, truncated, orphaned `.time` file) corroborates concurrent/aborted lane activity in this worktree.

This matches the certification_2 precedent verbatim in kind (`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:395-403`: "stopped only at three unchanged `check`-mode performance medians under sustained unrelated host load … fixtures contain no Rust interop … environmental timing drift rather than a PR-attributable regression"). One difference, stated plainly: cert_2 used a *retained pre-PR compiler binary* as control; I have no such control here (all six evidence files share one fingerprint, and re-running a control would violate your no-benchmarks constraint). The static reachability proof in §1 is the stronger substitute — it shows the changed code is unexecutable-or-trivial in the failing fixtures, rather than merely showing an older binary was also slow.

## Actionable requirements

**Blocking on this PR: none.**

**Bookkeeping to do at merge (not a code blocker):**
- Record the merge-lane acceptance rationale in the certification_8 evidence row (`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:154`), in the same form as the certification_2 precedent at lines 390-405 — naming the four cases, the shared-host contention, and the §1 reachability argument.

**Repository-wide retrospective — defer to certification_14, fold into the existing `certification_7` "retrospective performance rerun pending" item (line 153):**
1. Regenerate `perf.lsp.*` baselines in `verification/areas/performance/data/baselines.json`. They are stale since 2026-05-16 and were invalidated by main commit `be2df3d913` (2026-07-22). Current state gives `lsp-query-003-diagnostics` a threshold 5.91 ms against a real ~4.5 ms floor — a knife-edge budget that will keep flapping for unrelated PRs.
2. Recalibrate the `check-*` / `diagnostic-*` budgets. ~10% headroom over baselines captured on a differently-loaded host cannot survive the observed ±90% host dispersion.
3. Fix the dead `build-*` budgets. `build-project-001` runs at 8.5 s against a 37.3 s threshold — a 4× headroom that would not catch a real 3× regression.

Do not gate certification_9 on items 1-3; they are pre-existing main-branch calibration debt, independent of this PR's diff.
