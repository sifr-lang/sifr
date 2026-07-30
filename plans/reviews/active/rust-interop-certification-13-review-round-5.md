Repository files unmodified; all probes ran in `/tmp` and were removed. (I rebuilt `target/debug/sifr`, an ignored artifact.) The round-5 review file is a pre-existing empty untracked placeholder — I did not write to it.

---

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 5

## Verdict: **NOT SATISFIED**

Both round-4 blockers are genuinely fixed, and I reproduced every fix end to end. R4-1 and R4-3 are closed by a real module-graph resolver; R4-2 is closed decisively (925 → 1 SQLx `cargo metadata` spawn). R4-4 and R4-5 were also addressed beyond what was required.

One blocker remains, in the same defect class that gated rounds 1–4: `#[path]` on a **content-bearing** `mod` is ignored when computing the child module directory, so Sifr scans a directory Cargo never compiles. I reproduced a hard `SIFR-RUST-CARGO-0001` on a crate that `cargo check --offline` compiles clean, and the mirror-image miss on the file Cargo actually compiles. This is much narrower than prior rounds' blockers, and the fix is two lines — but it violates the invariant the architecture doc states for this subsystem in its own words ("fall through to offline Cargo as the authority instead of becoming a false Sifr diagnostic").

---

## Scope reviewed

Committed delta `origin/main..HEAD` (`b231daf81`, `7a27b7896`, `6ec0742b6`, `0e53989be`, `bfa7f27c6`, `96a56b7f1`), 54 files, focused on `96a56b7f1`. Excluded and not attributed: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.claude/`, the two stray webp files, `plans/phases/43_interoperability.md`.

---

## Findings (severity order)

### R5-1 — MEDIUM · `#[path]` on an inline module with a body is ignored, producing a false `SIFR-RUST-CARGO-0001`
`crates/sifr_driver/src/build/rust_interop_sqlx_modules.rs:97-104` (`collect_declared_modules`)

When a `mod` declaration carries a body, the traversal takes the `module.content` branch and recurses with `module_dir.join(module.ident)`, unconditionally discarding any `#[path]` attribute. Rust's rule is the opposite: `#[path = "alt"]` on a content-bearing module sets the directory in which **its children** are resolved. `resolve_declared_module`/`declared_path` handle `#[path]` correctly, but they are only reached on the file-module branch.

Reproduced with `target/debug/sifr` built at `96a56b7f1`, on a `/tmp` copy of the certified fixture (baseline: `no errors found`, 29.3 s):

```rust
// src/bridges/mod.rs
#[path = "alt"]
pub mod a { pub mod b; }

// src/bridges/alt/b.rs   <- the file Cargo compiles
pub fn compiled() {}

// src/bridges/a/b.rs     <- never compiled by anyone
pub fn q() { let _ = sqlx::query!("SELECT 34::INT4 AS value"); }
```

```
cargo check --offline --lib → Finished `dev` profile ... in 14.20s   (clean)
sifr check src/main.sifr    → error[SIFR-RUST-CARGO-0001]: Rust bridge package SQLx
  offline metadata failed: `SQLX_OFFLINE=true` but there is no cached data for
  this query: SELECT 34::INT4 AS value
```

The inverse confirms the resolution is simply wrong rather than merely conservative — with the query moved into the compiled `alt/b.rs` and no `a/` directory present, the preflight collects nothing and the failure surfaces only from rustc:

```
sifr check → error[SIFR-RUST-CARGO-0001]: Rust bridge SQLx offline metadata failed
  for `main.route_probe`
  = note: rustc stderr: `SQLX_OFFLINE=true` but there is no cached data ...
```

Severity is MEDIUM, not HIGH: the common real-world shape of this pattern is `#[cfg(unix)] #[path = "unix"] mod sys { … }` or the `cfg_attr` form, and both are already skipped conservatively by `module_declaration_may_vary` (I verified the `cfg_attr` + `path` case defers to Cargo — probe Q). Only the *unconditional* `#[path]`-on-a-body form is affected. But when a user does hit it, there is no workaround short of deleting valid code, and the same `.sqlx`-prepare escape hatch does not exist.

**Fix:** in the `module.content` branch, use `declared_path(module)` to compute the child directory (`module_dir.join(path)`) when present, falling back to `module_dir.join(ident)` otherwise. Add a regression test pinning a `#[path = "dir"] mod a { mod b; }` layout in both directions.

### R5-2 — NIT · Recorded warm-check wall-clock (2.67 s) did not reproduce; 14.5 s here, and the residual cost is outside SQLx
The load-bearing R4-2 claim reproduced exactly (see validation table): **1** SQLx `cargo metadata` spawn, no subprocess storm, `sqlx_dependency_crate_names` no longer spawns Cargo at all. But my traced warm check took **14.49 s**, not 2.67 s. I profiled it with `sample`: 2658 of 3946 samples sit in `probe_cache_key → cached_digest_path → digest_path`, recursively hashing the path-dependency tree — nothing SQLx-related, and unchanged by this branch (`rust_interop_probe_cache.rs` gained only the 5-line `.sqlx` digest block). `cargo metadata` on the fixture is 0.09 s, so it is not the cost either.

I attribute the delta to environment, not to an overclaim: my `/tmp` copy had to rewrite the `sifr_runtime` path dependency to an absolute path into the repo, which enlarges the digested tree. Flagging it so the "2.67 s" figure is not treated as a portable characteristic of a warm check.

### R5-3 — NIT · `syntax_outside_preflight_understanding_falls_through_to_cargo` now has a vacuous assertion
`rust_interop_sqlx_offline_tests.rs:117-119`

The test writes `src/unparseable.rs` and asserts validation still passes, which used to prove parse-failure tolerance under the `src/**/*.rs` glob. Under the module graph that file is never declared as a `mod`, so it is never opened, and the assertion now passes for the wrong reason. `syn::parse_file(...)` failure tolerance in `reachable_rust_modules:36` is real but no longer covered. Declare the module (`mod unparseable;`) to restore the coverage.

### R5-4 — NIT · Reported split sizes are off by a line or two
The issue doc records "661 lines for offline policy, 221 for cfg-aware visitation, and 200 for module-graph traversal". `wc -l` gives **662 / 219 / 200**. Cosmetic, but the round-4 review pinned exact numbers, so keep them exact.

### R5-5 — NIT · Symlinked module sources silently drop out of the preflight
`rust_interop_sqlx_modules.rs:84-87` (`is_regular_file` rejects symlinks)

Confirmed empirically (probe H): a `mod linked;` backed by a symlink is not scanned and the missing query surfaces only from rustc. This is fail-open and matches the documented "Cargo is the authority" rule, and it is the right trade for the containment guarantee — but symlinked sources are a real layout in vendored/generated trees, so it is a coverage reduction worth recording deliberately rather than incidentally. Same category: a `mod` declared inside a function body is not discovered (`visit_item_mod` is a no-op and `collect_declared_modules` only walks item-level `mod`s).

### R5-6 — NIT · An external `.env` `SQLX_OFFLINE_DIR` has no cache identity, and warm probe hits no longer run Cargo either
When `.env` declares `SQLX_OFFLINE_DIR`, `sqlx_metadata_roots` returns `None`, so the digest is `None` and the external directory contributes nothing to `probe_cache_key`. Combined with the preflight now running *after* the cache-hit check, a change to the external metadata invalidates neither the preflight nor the probe cache. This is a pre-existing consequence of the round-3 opt-out (nothing in `96a56b7f1` widened it) and the opt-out is explicit, but the limitation is undocumented.

### R5-7 — NIT · Two small residual inefficiencies
`resolve_cargo_workspace_root:311` spawns `cargo metadata` before trying `nearest_declared_workspace_root`, so the fixture — whose own `Cargo.toml` carries `[workspace]` — still pays one subprocess that a TOML read would answer. And `validate_sqlx_offline_metadata` parses `Cargo.toml` twice (directly, then again inside `backend_may_resolve_sqlx_metadata`). Also: the `ROOTS` memo is keyed on `(root, content-fingerprint)` and never evicts, so a long-lived driver editing manifests accumulates entries indefinitely — bounded-size entries, but unbounded count.

---

## Required-finding re-audit

**R4-1 — RESOLVED.** `collect_sqlx_queries` now iterates `reachable_rust_modules(backend_root)` instead of globbing. The resolver starts at `[lib].path` if declared, else `src/lib.rs`, else `src/main.rs` (`crate_entry_path:53-74`); refuses gated declarations *before* opening the file (`module_declaration_may_vary` is checked in `collect_declared_modules:94` prior to `resolve_declared_module`); honours inner file attributes (`has_conditional_compilation_attribute(&syntax.attrs):39`); rejects symlinks and canonicalizes with a `starts_with(canonical_root)` containment check; and dedups via `visited`. Independently reproduced on the certified fixture, all from a clean `no errors found` baseline:

| Probe | Layout | Cargo | `sifr check` | Verdict |
|---|---|---|---|---|
| A | `#[cfg(test)] mod tests;` + `src/bridges/tests.rs` | clean | **no errors found** | round-4 blocker fixed |
| B | `#[cfg(feature = "mysql-variant")] mod gated_variant;` (feature undeclared) | clean | **no errors found** | round-4 blocker fixed |
| C | orphan `src/bin/tool.rs` | clean (lib target) | **no errors found** | R4-3 fixed |
| D | ungated `mod extra;` → `src/bridges/extra/mod.rs` | fails | **rejected pre-Cargo** | nested layout still recognized |
| E | active `#[path = "redirected/active.rs"] mod …;` | fails | **rejected pre-Cargo** | active redirect followed |
| F | one file, gated + ungated query | fails on ungated | **rejected on `SELECT 22` only** | mixed file handled |
| G | `#[path = "../../../outside.rs"]` escaping the root | fails | deferred; rustc surfaced it | containment holds, fails open |
| I | `[lib] path = "src/entry.rs"` | fails | **rejected pre-Cargo** | Cargo `[lib].path` honoured |
| K | `mod outer { mod inner; }` → `outer/inner.rs` | fails | **rejected pre-Cargo** | inline→file nesting correct |
| O | `mod cyc;` + `#[path = "cyc.rs"] mod again;` (cycle) | fails | **rejected, no hang** | cycle guard works |
| L | `#[path = "alt"] mod a { mod b; }` | **clean** | **falsely rejected** | **R5-1** |

The mandatory negative test now injects a **file-based** `#[cfg(test)] mod cfg_gated_sqlx_regression;` plus its source file, so the boundary that broke in round 4 is pinned in generated-package integration coverage, not just units. `crate_module_graph_skips_gated_and_orphan_source_files` pins seven layouts at the unit level. Not covered by tests: `[lib].path`, the `main.rs` fallback, symlink/cycle refusal, and R5-1.

**R4-2 — RESOLVED, verified by measurement.** `sqlx_dependency_crate_names` now calls `workspace_dependency_packages`, which returns immediately when no dependency carries `workspace = true` (`:203-205`) and otherwise resolves via `declared_workspace_root` — pure TOML reads, no subprocess. `cargo_workspace_root` reinstates a cache keyed on an ancestor-manifest **content** fingerprint, and I confirmed the subprocess resolution happens outside the mutex: the read guard's scope closes at `:285`, `resolve_cargo_workspace_root` runs at `:286`, and the write lock is re-taken at `:287`. Traced with a PATH shim logging `cargo` argv, on a warm check:

| Metric | Round 4 | Round 5 (mine) |
|---|---|---|
| `cargo metadata` for the SQLx path | 925 | **1** |
| total `cargo metadata` | 925 | **2** (1 SQLx + 1 pre-existing package resolution) |
| spawns for a no-SQLx root (`crates/sifr_stdlib`) | 913 | **0** |
| total `cargo` invocations | — | 5 (`2× metadata`, `2× --version`, `1× -V`) |

The recorded "one cargo metadata subprocess total" is accurate for the SQLx path; "total" is imprecise by one, since a non-SQLx `cargo metadata --format-version 1` also runs. The issue-doc sentence round 4 called false is now corrected and matches what I measured.

**Preflight-after-cache-hit move — SOUND.** `validate_probe_sqlx_offline_metadata` moved from `rust_interop_probe.rs:71` to after the cache-hit check at `:85`. This is safe because `probe_cache_key` covers both channels through which the preflight's answer can change: `cached_digest_path(backend_root)` (all bridge sources, `.sqlx`, and `.env`) plus `sqlx_offline_metadata_digest(backend_root)`, which resolves package **and workspace** `.sqlx` roots — so metadata living outside `backend_root` still participates. A cache hit therefore implies an earlier successful build of byte-identical inputs. Confirmed empirically: the mandatory negative test's post-control mutations all produce a miss and a rejection. Caveat R5-6 for the external-directory opt-out.

**R4-4 — RESOLVED (beyond requirement).** `attribute_may_disable:173-184` now distinguishes `cfg` (always gating) from `cfg_attr`, which gates only when a non-predicate argument could itself be a `cfg`/`cfg_attr` (`cfg_attr_arguments_may_disable:186-192` skips argument 0 and recurses). Unit coverage pins both sides: `#[cfg_attr(any(), allow(dead_code))]` stays preflighted (`SELECT 96` collected), `#[cfg_attr(feature = "…", cfg(test))]` defers. Unparseable `cfg_attr` token streams return `true` (defer) — fail-open. The one exception is `module_declaration_may_vary:167-171`, which treats **any** `cfg_attr` on a `mod` as varying; that is correct rather than inconsistent, because `cfg_attr` can inject `#[path]` and change which file is loaded. I verified `#[cfg_attr(feature = "alt", path = "other.rs")] mod qmod;` defers to Cargo (probe Q).

**R4-5 — RESOLVED.** The message is now `Rust bridge package SQLx offline metadata failed: {reason}` with the note "This preflight is package-scoped." No arbitrary probe target is blamed. Every probe above confirms it. The mandatory test's assertion was re-pointed from `main.query_compile_time` to this stable prefix, and still independently pins missing vs. stale detail strings and `SQLX_OFFLINE=true`.

**Attribute coverage / fail-open / panic safety — good.** `rust_interop_sqlx_cfg.rs` overrides `visit_item`, `visit_stmt`, `visit_expr`, `visit_arm`, `visit_field`, `visit_field_value`, `visit_variant`, `visit_generic_param`, `visit_fn_arg`, `visit_impl_item`, `visit_trait_item`, `visit_foreign_item` — the full set the requirement names. Every non-exhaustive match arm returns `true` (⇒ skip ⇒ defer), so future `syn` variants fail open; `Verbatim` arms return `false`, harmless because `syn::visit` yields no `Macro` nodes inside a `TokenStream`. No `unwrap`/`expect`/panic/indexing in the three new production files; the only `unwrap_or_else` is `Path::parent().unwrap_or_else(|| Path::new(""))`. `segments[0]` at `rust_interop_sqlx_offline.rs:498` is guarded by `segments.len() >= 2`. All parses are `.ok()`, all I/O `Result`/`Option`-handled, recursion bounded by what `syn` already accepted, and the cycle guard was verified live (probe O).

**Ambient vs `.env` `SQLX_OFFLINE_DIR` (R3-2) — holds.** Probe P: `SQLX_OFFLINE_DIR=/external/nope` in the process environment does **not** disengage the preflight (the query is still rejected). Probe P2: the `.env` form does disengage, and the failure then surfaces from Cargo. `grep -rn SQLX_OFFLINE_DIR crates/` finds no process-env read.

**Package-scoped diagnostics, responsibility split, file sizes.** The split is clean by concern: offline policy 662, cfg-aware visitation 219, module traversal 200 — the two new files are well under the cap and `rust_interop_sqlx_offline.rs` dropped from 823 to 662, restoring headroom. `rust_interop.rs` is unchanged at 883 (17 lines of headroom — still worth watching). File-size guardrail PASS (3011 files, limit 900); driver maintainability PASS.

**TypeScript-Go direct-read inventory — current and accurate.** I spot-verified all fourteen new/updated references line by line: `rust_interop_probe.rs:85` (`cache_file.is_file()`), `rust_interop_sqlx_modules.rs:33/54/86`, `rust_interop_sqlx_offline.rs:72/108/209/256/263/340/349/542/626/630` — all real probe sites, no stale entries. The guard scans all of `crates/sifr_driver/src` and PASSes, so completeness holds.

**Multi-backend final cache identity — holds.** `rust_interop.rs:243` feeds `combined_sqlx_offline_metadata_digest` over every resolved backend root into the generated-build input; `rust_interop_cargo_inputs.rs:77-79` merges the field across backends with unit coverage at `:492-497`; `complete_metadata_directory_participates_in_cache_identity` proves a `describe` mutation in one of two backends changes the combined digest.

**Sentinel attribution and earlier rounds (R3-3, R3-5, and R1–R7 of round 2) — all hold.** I re-read `docs/rust-interop.mdx:249-259`, `internal_docs/rust_interop_architecture.md:1220-1231`, `plans/phases/39_rust_interop.md:345-359`, the fixture READMEs, and the compatibility-matrix `notes`. Every claim is fixture-scoped and precise; "Missing or stale metadata is rejected before Cargo is spawned" remains true after the cache-hit move (on a hit, no Cargo runs at all). The architecture doc now explicitly documents the module-graph scoping and the memo semantics — which is what makes R5-1 a documented-invariant violation rather than an unstated gap.

---

## Validation assessment

Every recorded figure reproduced except the warm wall-clock (R5-2). Nothing was overstated.

| Check | Recorded | My result |
|---|---|---|
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | 11 pass | **11 passed, 0 failed** (0.11 s) |
| `cargo test -p sifr_driver --lib` | 448 / 65 ignored | **448 passed, 0 failed, 65 ignored** (31.84 s) |
| Mandatory negative (`.env`-armed, file-based gated query) | pass, 27.83 s | **pass** |
| Mandatory positive (loopback + SQLx offline) | pass, 55.77 s | **pass**; both together 75.93 s |
| Rust-interop area runner | 10/10, 229 mutations | **variants=10, failures=0**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5+6, compat 36/36/3 + 7, 20 stale-draft, 36 claims + 33 |
| `cargo clippy --workspace -- -D warnings` | pass | **pass** |
| `cargo fmt --check` | pass | **pass** |
| file-size guardrail | pass | **PASS** (3011 files, limit 900) |
| `sifr_driver` maintainability | pass | **PASS** |
| TypeScript-Go transfer guardrails | pass | **PASS** |
| Resource gate + `--self-test` | pass | **PASS** (`surfaces=1`, `future_runtime_rows=0`) |
| `git diff --check origin/main..HEAD` | pass | **clean** |
| `cargo metadata` spawns, warm check | 1 | **1 SQLx / 2 total CONFIRMED** (was 925) |
| Warm fixture check wall-clock | 2.67 s | **14.49 s** — residual is pre-existing `digest_path`, not SQLx (**R5-2**) |
| Production split sizes | 661 / 221 / 200 | **662 / 219 / 200** (**R5-4**) |
| `#[path]` on a content-bearing `mod` | not run | **false rejection CONFIRMED** (**R5-1**) |

`cargo clippy --workspace --all-targets` fails with 28 pre-existing `semicolon_if_nothing_returned` errors in `sifr_lowering` **(lib test)**. That crate is untouched by `origin/main..HEAD` and `--all-targets` is not the documented gate in `AGENTS.md`; I am not attributing it to this milestone, but it will bite whoever tightens the gate.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** Area runner re-verifies 44 crate aliases; fixture manifest pins `=0.8.9`/`=0.8.6`/`=0.7.0`/`=1.52.3` with `default-features = false`. |
| Hermetic `127.0.0.1:0` Axum service, real tower-http, deterministic shutdown | **Met.** Positive mandatory test passed. |
| Real SQLx macro from checked-in `.sqlx` under forced `SQLX_OFFLINE`, no live DB | **Met.** Sentinel-unused assertions pass at all three checkpoints. |
| Mandatory generated-package diagnostic: independent missing + stale, stable `SIFR-RUST-CARGO-0001`, DB/network disabled | **Met**, and materially strengthened — the injected regression is now a *file-based* gated module, closing the structural gap the fixture had in round 4. |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met.** Counts re-verified: 36 rows / 36 fixture_rows / 3 categories, 229 mutations, 44 crates, 61 package examples, 18 scenario examples, 36 claims, `future_runtime_rows=0`. The round-4 inaccurate `cargo metadata` claim is corrected and matches measurement. Wording is accurate everywhere I checked, modulo **R5-4**. |
| Validator self-tests without weakening earlier rows | **Met.** 229 cases, 10/10, no prior row regressed. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked, and — unlike every prior round — the fixture-scoped criteria now do exercise the boundary that broke last time. The one remaining gap (**R5-1**) is again in production code outside what the fixture can reach.

---

## Required fixes

1. **R5-1** — In `collect_declared_modules`, use `declared_path(module)` when a content-bearing `mod` carries `#[path]`, so children resolve in the declared directory. Add a regression test covering `#[path = "dir"] mod a { mod b; }` in both directions (the never-compiled sibling must not be scanned; the compiled child must be).

## Optional suggestions

- **R5-3** — Declare `mod unparseable;` so the parse-tolerance assertion is no longer vacuous.
- **R5-4** — Correct the recorded split sizes to 662 / 219 / 200.
- **R5-5 / R5-6** — Record the symlinked-source and function-body-`mod` coverage gaps, and the fact that an external `.env` `SQLX_OFFLINE_DIR` participates in no cache identity, as deliberate documented limitations.
- **R5-7** — Try `nearest_declared_workspace_root` before spawning `cargo metadata`; pass `sqlx_crates` into `backend_may_resolve_sqlx_metadata` to avoid the second manifest parse.
- Add unit coverage for `[lib].path`, the `main.rs` fallback, and symlink/cycle refusal — all three are load-bearing and currently only verified by my ad-hoc probes.
- Watch `rust_interop.rs` (883/900) before the next change lands there.
