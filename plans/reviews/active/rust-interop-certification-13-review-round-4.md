Repository files were not modified — all probes ran in `/tmp` and were removed; `git status` matches the starting snapshot apart from the pre-existing parallel-agent paths. (I did rebuild `target/debug/sifr`, an ignored build artifact, because the checked-out binary predated `bfa7f27c6`.)

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 4

## Verdict: **NOT SATISFIED**

The round‑3 required fix (R3‑1) is **partially** implemented. The `syn`-level attribute walk is well built and closes the inline forms I could construct — but the recognizer still has no module-graph awareness, so a `cfg`-gated **file** module (`#[cfg(test)] mod tests;` + `src/tests.rs`, the single most idiomatic Rust layout there is) is still hard-rejected by `sifr check` while Cargo compiles the crate clean. I reproduced this end to end with the binary built from `bfa7f27c6`. That is the same "valid code now fails" class that blocked rounds 1, 2, and 3.

Separately, the R3‑4 fix removed the workspace-root memo without gating the call site that actually dominates: a single warm `sifr check` of the certified fixture now spawns **925 `cargo metadata` subprocesses** (913 of them for `crates/sifr_stdlib`, which has neither `.sqlx` nor any sqlx dependency), costing ~41 s of a 54 s warm check. The issue doc's claim "no-SQLx roots avoid `cargo metadata`" is false as written.

R3‑2, R3‑3, R3‑5 are correctly and fully resolved.

---

## Scope reviewed

Committed delta `origin/main..HEAD` (`b231daf81`, `7a27b7896`, `6ec0742b6`, `0e53989be`, `bfa7f27c6`), 51 files, focused on `bfa7f27c6`. Excluded and not attributed: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.agent/`, the two stray webp files, `plans/phases/43_interoperability.md`. `plans/reviews/active/rust-interop-certification-13-review-round-4.md` is an untracked empty placeholder; I did not write to it.

---

## Findings (severity order)

### R4‑1 — HIGH · `cfg`-gated **file** modules still produce a false `SIFR-RUST-CARGO-0001`
`crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:318-332` (`collect_sqlx_queries`)

The new attribute guards all live inside `collect_module_queries` / `SqlxQueryVisitor`, which operate on the item tree of one already-parsed file. But `collect_sqlx_queries` walks **every** `.rs` file under `src/` via `collect_rust_sources` and treats each one as an independent root module. There is no link from a file back to the `mod` declaration that includes it, so `#[cfg(...)] mod foo;` suppresses the declaration item — and then `src/foo.rs` is scanned anyway, from scratch, with no inherited cfg context. Its queries are collected and their metadata demanded.

Reproduced with `target/debug/sifr` built at `bfa7f27c6`, on a `/tmp` copy of the certified fixture (baseline: `no errors found`):

```
# src/bridges/mod.rs
pub mod backend;

#[cfg(test)]
mod tests;

# src/bridges/tests.rs
#[test]
fn probe_query() { let _ = sqlx::query!("SELECT 88::INT4 AS value"); }

cargo check --offline   → Finished, clean (lib target never compiles tests.rs)
sifr check src/main.sifr → error[SIFR-RUST-CARGO-0001]: Rust bridge SQLx offline
  metadata failed for `main.route_probe`: `SQLX_OFFLINE=true` but there is no
  cached data for this query: SELECT 88::INT4 AS value
```

Identical result for a feature module that is not even declared in `Cargo.toml`:

```
# src/bridges/mod.rs:  #[cfg(feature = "mysql-variant")] pub mod gated_variant;
# src/bridges/gated_variant.rs:  sqlx::query!("SELECT 77::INT4 AS value")

cargo check --offline   → Finished (only an unexpected_cfgs warning)
sifr check src/main.sifr → error[SIFR-RUST-CARGO-0001]: … SELECT 77::INT4 AS value
```

The multi-driver feature-module pattern (`#[cfg(feature = "postgres")] mod pg;` / `mod mysql;`) and file-based `#[cfg(test)] mod tests;` are the two most common shapes this hazard takes in real sqlx crates, and both are unfixable by the user short of deleting the code. `cargo sqlx prepare`'s default does not cover test targets, so "just prepare them" is not an escape hatch.

The new unit test `cfg_gated_queries_fall_through_to_cargo` and the injected regression in the mandatory negative test both use **inline** `#[cfg(test)] mod { … }`, so neither exercises this boundary. The fixture, having a single-file bridge, structurally cannot.

**Fix:** resolve the module graph from the crate root (`src/lib.rs` / `src/main.rs`, honouring `#[path]`) instead of globbing `src/**/*.rs`, and stop descending at any `mod` declaration carrying a `cfg`/`cfg_attr`. A file reachable only through a gated declaration must not be scanned at all. Add regression coverage for file-based `#[cfg(test)] mod tests;` and a file-based undeclared-feature module.

### R4‑2 — HIGH · The memo removal amplified `cargo metadata` to ~925 subprocesses per check (~41 s)
`rust_interop_sqlx_offline.rs:123` (`sqlx_dependency_crate_names` → `workspace_dependency_packages`), `:275-306`

`bfa7f27c6` deleted the `OnceLock<Mutex<BTreeMap<…>>>` memo that `HEAD~1` had around `cargo_workspace_root`, and added the `backend_may_resolve_sqlx_metadata` short-circuit **only** to `sqlx_metadata_roots`. The other caller is unguarded: `sqlx_dependency_crate_names` calls `workspace_dependency_packages(backend_root)` unconditionally at line 123 — before it knows whether sqlx is even a dependency, and before it knows whether any dependency uses `workspace = true`. And `validate_probe_sqlx_offline_metadata` runs at `rust_interop_probe.rs:71`, i.e. **before** the probe-cache hit check at `:86`, so it fires on every probe on every invocation, warm or cold.

Measured with a PATH shim that logs `cargo` argv, on a warm `sifr check` of the certified fixture:

| Manifest | `cargo metadata` spawns, one warm check |
|---|---|
| `crates/sifr_stdlib/Cargo.toml` | **913** |
| `/tmp/…/pkg/Cargo.toml` | 7 |
| `crates/sifr_runtime/Cargo.toml` | 5 |

`crates/sifr_stdlib` has no `.sqlx` directory and no sqlx dependency (`grep sqlx crates/sifr_stdlib/Cargo.toml` → empty) — precisely the case requirement 5 says must avoid `cargo metadata`. A single such call costs ~45 ms here, so ~41 s of the measured **54.0 s** warm `sifr check` is this subprocess storm. At `HEAD~1` the memo bounded it to one call per distinct root (~3 total).

This also makes the issue-doc statement "workspace resolution no longer uses a process-lifetime memo, no-SQLx roots avoid `cargo metadata`" (`plans/issues/…certification.md:1573-1576`) inaccurate: the second clause does not hold for the preflight path, and the first clause names a change that removed the only thing bounding the cost.

**Fix:** apply the same "possible sqlx" short-circuit before `workspace_dependency_packages` (or make it lazy — it is only needed when a dependency entry actually carries `workspace = true`), and reinstate caching resolved outside any lock. Moving `validate_probe_sqlx_offline_metadata` after the probe-cache hit check would also be safe now that the `.sqlx` digest is part of `probe_cache_key`.

### R4‑3 — LOW · Non-lib targets and orphan files under `src/` are also preflighted
`rust_interop_sqlx_offline.rs:334-360`

Same root cause as R4‑1, different surface. `src/bin/tool.rs` containing `sqlx::query!("SELECT 66::INT4 AS value")` produces `SIFR-RUST-CARGO-0001` even though Sifr's probe builds the bridge crate as a path dependency, where Cargo compiles only the lib target. Confirmed empirically. Lower severity than R4‑1 (bridge crates carrying bins are rarer), but a module-graph-rooted scan fixes both at once.

### R4‑4 — NIT · `cfg_attr` is treated as gating, which silently narrows preflight coverage
`rust_interop_sqlx_offline.rs:533-537`

`#[cfg_attr(…)]` never removes an item — it conditionally *adds* attributes. Treating it as "possibly disabled" means `#[cfg_attr(feature = "x", inline)] fn f() { sqlx::query!(…) }` is skipped entirely, so a genuinely missing `.sqlx` entry there falls to Cargo instead of `sifr check`. This is fail-open and consistent with the documented "Cargo is the authority" rule (`rust_interop_architecture.md:1223-1226` says exactly this), so it is not a defect — but it is a real coverage reduction and the unit test pins it deliberately (`#[cfg_attr(any(), allow(dead_code))]` with an unprepared query), so it should be a conscious call rather than incidental.

### R4‑5 — NIT · R3‑6 unchanged: SQLx failures are still attributed to an arbitrary probe target
Both my reproductions blamed `main.route_probe` for queries in unrelated modules. The preflight is package-scoped; `{target}` is whichever probe validated first. Naming the offending source file (or stating package scope) would make the diagnostic actionable — and would have made R4‑1 self-evident to a user.

---

## Required-finding re-audit

**R3‑1 — NOT RESOLVED (partial).** The `syn` layer is genuinely good: `collect_module_queries` skips `Item::Mod` with a cfg attribute before recursing; `SqlxQueryVisitor` overrides `visit_item`, `visit_stmt`, `visit_expr`, `visit_arm`, `visit_impl_item`, `visit_trait_item`, `visit_foreign_item`, each short-circuiting on `cfg`/`cfg_attr`. I verified the round‑3 inline repro now returns `no errors found`. Ungated queries in the same file are **not** hidden: `collect_module_queries` skips only the gated item and visits every sibling individually — my fixture repro kept the active `SELECT 13` query validated (it was the only query the unit test collects, and the fixture's real query still passes). Alias collection (`module_sqlx_aliases:399-412`) skipping cfg-gated `use`/`extern crate` items can only *shrink* the recognized set, so it cannot introduce false positives; `sqlx_crates` from `Cargo.toml` is always seeded, so fully-qualified `sqlx::query!` stays recognized. The unit test and the mandatory negative test's injected `#[cfg(test)] mod cfg_gated_sqlx_regression { … }` both pass and both exercise the *inline* boundary only. The file-module boundary (**R4‑1**) is untested and still broken.

**Attribute-handling completeness / fail-open / panic safety — good.** Every non-exhaustive match arm in `item_has_cfg_attribute`, `expr_has_cfg_attribute`, `impl_item_…`, `trait_item_…`, `foreign_item_…` returns `true` (⇒ skip ⇒ defer to Cargo), so a future `syn` variant fails open rather than being scanned with unknown attributes. `Verbatim` arms return `false`, but `syn::visit` on a `TokenStream` yields no `Macro` nodes, so nothing is collected either way — harmless. No `unwrap`/`expect`/indexing in the new production code; all parses are `.ok()`, all I/O `Result`/`Option`-handled; the only recursion is bounded by what `syn` already accepted. Queries inside `macro_rules!` bodies are opaque tokens and are correctly not collected.

**R3‑2 — RESOLVED.** `std::env::var_os("SQLX_OFFLINE_DIR")` is gone from `sqlx_metadata_roots`; the only remaining check is `dotenv_defines_offline_dir(&backend_root.join(".env"))` (`:204`), matching sqlx 0.8.6, which populates the read-path `offline_dir` only from `.env`. `grep -rn SQLX_OFFLINE_DIR crates/` confirms no process-env read survives anywhere. Ambient exports no longer drop default `.sqlx` roots from cache identity; `explicit_offline_directory_disengages_conservative_preflight` now writes the `.env` form and pins both the preflight bypass and `digest == None`.

**R3‑3 — RESOLVED.** All six sites re-attributed: `docs/rust-interop.mdx` ("the valid control reaches Cargo without contacting that sentinel … Missing or stale metadata is rejected before Cargo is spawned"), `internal_docs/rust_interop_architecture.md:1265-1268`, `plans/phases/39_rust_interop.md:352-357`, `plans/issues/…:1548-1556`, fixture `README.md:14-17`, and the compatibility-matrix `notes`. Wording is now precise about what each case proves.

**R3‑4 — REGRESSED, not resolved.** The memo and cross-subprocess lock are genuinely gone (verified against `HEAD~1`, which had `static ROOTS: OnceLock<Mutex<…>>`), and `backend_may_resolve_sqlx_metadata` correctly short-circuits the **digest** path. But the **preflight** path was left unguarded and is now uncached — see **R4‑2**. This is a net performance loss relative to `HEAD~1`.

**R3‑5 — RESOLVED.** The append-only paragraph is deleted and the driver row carries current line numbers. I spot-verified all fourteen `rust_interop_sqlx_offline.rs` references against the file: `:75` (`is_dir`), `:111`, `:175`, `:221` (`is_dir`), `:224`, `:264`, `:310`, `:321`, `:344` (`is_file`), `:350` (`is_dir`), `:353` (`read_dir`), `:706`, `:790` (`is_file`), `:794` — all real probe sites, no stale entries. `rust_interop_cargo_inputs.rs:141` is now in the row. The guard scans all of `crates/sifr_driver/src`, so the entries dropped from the row (`rust_interop.rs:795+`, `sysroot_interop.rs:398,543`, etc.) cannot have left the inventory incomplete — the guard would fail. It passes.

**Earlier rounds (R1–R7 of round 2).** All still hold; nothing in `bfa7f27c6` touches the `.env` sentinel, the workspace-root lookup, the dedicated `sqlx_offline_metadata_digest` field, the hash-equality removal, the stale-message wording, or the `describe` type check. Both mandatory tests pass, which re-exercises the sentinel path.

---

## Validation assessment

Every recorded figure reproduced; none was overstated.

| Check | Recorded | My result |
|---|---|---|
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | 10 pass | **10 passed, 0 failed** (0.25 s) |
| `cargo test -p sifr_driver --lib` | 447 / 65 ignored | **447 passed, 0 failed, 65 ignored** (147.08 s) |
| Mandatory negative (`.env`-armed, cfg-gated injection) | pass, 195.61 s cold | **pass, 175.34 s** |
| Mandatory positive (loopback + SQLx offline) | — (not re-recorded after `bfa7f27c6`) | **pass, 147.79 s** |
| Rust-interop area runner | 10/10, 229 mutations | **variants=10, failures=0**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5, compat 36/36/3 + 7 self-tests, 20 stale-draft cases, 36 claims + 33 self-tests |
| `cargo clippy --workspace -- -D warnings` | pass | **pass** |
| `cargo fmt --check` | pass | **pass** |
| file-size guardrail | pass | **PASS** (3009 files, limit 900) |
| `sifr_driver` maintainability | pass | **PASS** |
| TypeScript-Go transfer guardrails | pass | **PASS** |
| Resource gate + `--self-test` | pass | **PASS** (`surfaces=1`, `future_runtime_rows=0`) |
| `git diff --check origin/main..HEAD` | pass | **clean** |
| Empirical `cfg`-gated file module | not run | **false rejection CONFIRMED** |
| `cargo metadata` spawn count, warm check | not run | **925 spawns / ~41 s of 54.0 s CONFIRMED** |

File sizes: `rust_interop_sqlx_offline.rs` 823, `_tests.rs` 355, `rust_interop.rs` 883, `rust_interop_probe.rs` 868 — all under 900, but `rust_interop.rs` has 17 lines of headroom and the sqlx module grew 205 lines this commit. The R4‑1 fix (module-graph resolution) will not fit in `rust_interop_sqlx_offline.rs` without a split.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** Unchanged since round 3; area runner re-verifies 44 crate aliases. |
| Hermetic `127.0.0.1:0` Axum service, real tower-http, deterministic shutdown | **Met.** Positive mandatory test passed (147.79 s). |
| Real SQLx macro from checked-in `.sqlx` under forced `SQLX_OFFLINE`, no live DB | **Met.** Round‑3's counterfactual still stands; nothing in this commit weakens it. |
| Mandatory generated-package diagnostic: independent missing + stale, stable `SIFR-RUST-CARGO-0001`, DB/network disabled | **Met for the fixture** (175.34 s), now with an inline cfg-gated regression injected. The fixture's single-file bridge cannot reach **R4‑1**. |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met**, counts re-verified: 36 rows / 36 fixture_rows / 3 categories, 229 mutations, 44 crates, 61 package examples, 18 scenario examples, 36 claims, `future_runtime_rows=0`. Wording is now accurate everywhere **except** the `cargo metadata` claim at `plans/issues/…:1573-1576` (**R4‑2**). |
| Validator self-tests without weakening earlier rows | **Met.** 229 cases, 10/10, no prior row regressed. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked. As in every prior round, the gap is in production code that the fixture-scoped acceptance criteria structurally cannot exercise.

---

## Required fixes

1. **R4‑1** — Root the source scan at the crate entry (`src/lib.rs` / `src/main.rs`), follow `mod` declarations (honouring `#[path]`), and refuse to descend through any declaration carrying `cfg`/`cfg_attr`. Stop globbing `src/**/*.rs`. Regression tests: file-based `#[cfg(test)] mod tests;`, file-based `#[cfg(feature = "…")] mod variant;` with the feature undeclared, and a `#[path]`-redirected gated module. This also resolves **R4‑3**.
2. **R4‑2** — Gate `workspace_dependency_packages` behind the same "possible sqlx dependency" check (or make it lazy on `workspace = true`), restore a cache for resolved workspace roots computed outside any lock, and correct `plans/issues/…certification.md:1573-1576`, which currently claims a short-circuit that does not exist on the preflight path. Verify by re-measuring the `cargo metadata` spawn count on a warm `sifr check`.

## Optional suggestions

- **R4‑4** — Decide explicitly whether `cfg_attr` should gate. Narrowing it to "only `cfg_attr` whose expansion could plausibly be a `cfg`" would restore preflight coverage for the common `#[cfg_attr(feature=…, inline)]` shape.
- **R4‑5** — Name the offending `.rs` file in `SIFR-RUST-CARGO-0001`, or state that the preflight is package-scoped, instead of attributing to an arbitrary probe target.
- Consider moving `validate_probe_sqlx_offline_metadata` after the probe-cache hit check now that `.sqlx` participates in `probe_cache_key` — this removes the preflight from the warm path entirely.
- Watch `rust_interop.rs` (883/900) and `rust_interop_sqlx_offline.rs` (823/900) before landing the R4‑1 module resolver.
