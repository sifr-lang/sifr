Repository files unmodified; all probes ran in `/tmp` and were removed. (I rebuilt `target/debug/sifr`, a gitignored artifact.) `plans/reviews/active/rust-interop-certification-13-review-round-6.md` was already present as an untracked empty placeholder before this session — I did not write to it.

---

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 6

## Verdict: **NOT SATISFIED**

## Executive summary

The round-5 blocker's **literal** required fix landed and is correct: `#[path = "dir"]` on a content-bearing `mod` now redirects its children, both directions are unit-pinned, and the mandatory `.env`-armed generated-package test carries a never-compiled default-directory query and reaches Cargo. Every optional round-5 suggestion except R5-7's two micro-optimizations was also addressed (parse tolerance de-vacuumed, `[lib].path`/`main.rs` pinned, memo bounded, opt-outs documented, split sizes corrected). All recorded validation figures reproduced.

But the round-5 blocker's **invariant** — "a crate offline Cargo compiles clean must never produce a false Sifr diagnostic" — is still violated by the same `#[path]` base-directory defect, in three further layouts I reproduced end to end on the certified fixture itself. The fix used `module_dir` as the `#[path]` base. `module_dir` is correct for plain `mod` lookups but is *not* rustc's `#[path]` base whenever the enclosing file is a non-`mod.rs` file module — which is exactly the file kind (`src/bridges/backend.rs`) that holds this fixture's real SQLx queries. I got a hard `SIFR-RUST-CARGO-0001` on three crates that `cargo check --offline --lib` finishes clean, plus the mirror-image silent miss on the file Cargo actually compiles.

This is one finding, narrow, and the same two-state fix closes all three cases. Nothing else in the branch is blocking.

---

## Scope reviewed

Committed delta `origin/main..a191d7202` (`b231daf81`, `7a27b7896`, `6ec0742b6`, `0e53989be`, `bfa7f27c6`, `96a56b7f1`, `a191d7202`), 55 files, focused on `a191d7202`. Excluded and not attributed: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.claude/`, the two stray webp files, `plans/phases/43_interoperability.md`.

---

## Findings (severity order)

### R6-1 — MEDIUM · BLOCKING · `#[path]` is resolved against the wrong base directory in non-`mod.rs` parent files, producing false `SIFR-RUST-CARGO-0001`
`crates/sifr_driver/src/build/rust_interop_sqlx_modules.rs:98-101` (the new inline branch), `:112-118` (`resolve_declared_module`), `:152-161` (`module_dir_for_explicit_path`)

`PendingModule` carries a single `module_dir`. For a file module `mod m;` resolving to `<dir>/m.rs`, the code sets `module_dir = <dir>/m` and then uses that same value as the base for **both** plain child lookups and `#[path]` lookups. rustc keeps two pieces of state (`dir_path` plus a pending `relative` component) and **skips the `relative` component for `#[path]`**. I established the real rules empirically with `rustc`/`cargo`, not from memory:

| Construct, declared in… | rustc resolves to | Sifr computes | Same? |
|---|---|---|---|
| `#[path="alt"] mod a { mod b; }` in a **mod-rs** file (`lib.rs`/`main.rs`/`mod.rs`) | `<dir>/alt/b.rs` | `<dir>/alt/b.rs` | ✅ (this is what `a191d7202` fixed) |
| `mod outer { #[path="x.rs"] mod y; }` in a mod-rs file | `<dir>/outer/x.rs` | `<dir>/outer/x.rs` | ✅ |
| `#[path="foo.rs"] mod c;` in **`<dir>/m.rs`** (non-mod-rs) | `<dir>/foo.rs` | `<dir>/m/foo.rs` | ❌ **A** |
| `#[path="alt"] mod a { mod b; }` in **`<dir>/m.rs`** (non-mod-rs) | `<dir>/alt/b.rs` | `<dir>/m/alt/b.rs` | ❌ **B** |
| child `mod kid;` inside a file loaded by `#[path="p.rs"]` | `<parent-of-p>/kid.rs` | `<parent-of-p>/p/kid.rs` | ❌ **C** |

All three mismatches reproduce as **hard false diagnostics** with `target/debug/sifr` built at `a191d7202`, on a `/tmp` copy of the certified `backend_feature_package` (baseline `sifr check src/main.sifr` → `no errors found`, 21.3 s):

**A** — appended to `src/bridges/backend.rs`: `#[path = "redirect_child.rs"] mod redirect_child;`, with the compiled file at `src/bridges/redirect_child.rs` and an unprepared query in the never-compiled `src/bridges/backend/redirect_child.rs`:
```
cargo check --offline --lib → Finished `dev` profile ... in 16.60s   (clean, 1 dead_code warning)
sifr check src/main.sifr    → error[SIFR-RUST-CARGO-0001]: Rust bridge package SQLx offline
  metadata failed: `SQLX_OFFLINE=true` but there is no cached data for this query:
  SELECT 77::INT4 AS value
```

**B** — the *same construct `a191d7202` added*, moved into the non-mod-rs `backend.rs`: `#[path = "alt_b"] mod inline_b { mod child; }`, compiled child at `src/bridges/alt_b/child.rs`, unprepared query at `src/bridges/backend/alt_b/child.rs`:
```
cargo check --offline --lib → Finished ... (clean)
sifr check                  → error[SIFR-RUST-CARGO-0001] ... SELECT 78::INT4 AS value
```

**C** — `#[path = "loaded.rs"] mod loaded;` in `src/bridges/mod.rs`, `loaded.rs` declaring `mod kid;`, compiled child at `src/bridges/kid.rs`, unprepared query at `src/bridges/loaded/kid.rs`:
```
cargo check --offline --lib → Finished ... (clean)
sifr check                  → error[SIFR-RUST-CARGO-0001] ... SELECT 80::INT4 AS value
```

The mirror direction confirms this is mis-resolution, not conservatism — with the query in the file rustc actually compiles (`src/bridges/redirect_child.rs`) and no `src/bridges/backend/` present, the preflight collects nothing and the failure surfaces only from rustc:
```
error[SIFR-RUST-CARGO-0001]: Rust bridge SQLx offline metadata failed for `main.route_probe`
  = note: rustc stderr: `SQLX_OFFLINE=true` but there is no cached data for this query...
 --> /private/tmp/certrev13/pkg/src/bridges/redirect_child.rs:1:18
```

Why this is blocking rather than a nit:
- It is the **same defect class and the same severity** the round-5 review used to return NOT SATISFIED, in the same function, and case **B** is literally the construct the round-5 fix introduced coverage for — just relocated one file kind over.
- `src/bridges/backend.rs` — the non-mod-rs file where cases A and B bite — is precisely where this fixture's SQLx macros live, so the affected layout is reachable inside the certified package, not a synthetic corner.
- A false hard `SIFR-RUST-CARGO-0001` has no user workaround short of deleting valid code; `cargo sqlx prepare` cannot help, because the file is not part of the compiled crate.
- `internal_docs/rust_interop_architecture.md:1224-1226` claims source discovery follows the module graph "including active `#[path]` redirects" and that anything outside the recognizer "fall[s] through to offline Cargo as the authority instead of becoming a false Sifr diagnostic." Both halves are currently inaccurate for non-mod-rs parents.

**Fix:** replace the single `module_dir` with rustc's two-state model — `dir_path` plus `relative: Option<String>`:
- plain `mod m;` → `<dir_path>/<relative?>/m.rs` (or `.../m/mod.rs`); child state is `dir_path` unchanged with `relative = Some("m")` for the flat form, and `dir_path = <dir>/m`, `relative = None` for the `mod.rs` form;
- inline `mod m { … }` → `dir_path = <dir_path>/<relative?>/m`, `relative = None`;
- any `#[path = p]` → base is `<dir_path>` **without** `relative`; the loaded module's child state is `dir_path = parent(p)` (or `parent(p)` for `mod.rs`), `relative = None`.

**Required regression coverage:** unit tests for A, B, and C in both directions (redirected child recognized; default-directory sibling never scanned), with the declaring file a *file* module rather than `src/lib.rs`. The existing `write_source` helper only writes `src/lib.rs` and `install_cfg_gated_query_regression` only appends to `src/bridges/mod.rs`, so no current test can observe any of these three cases.

### R6-2 — NIT · Round-5 regression coverage is mod-rs-only, and the issue doc states it unqualified
`rust_interop_sqlx_offline_tests.rs:209-214`, `package_rust_interop_backend_ecosystem_support.rs:164-168`

Both new `#[path = "alt_inline"] mod inline_redirected { mod child; }` fixtures are installed into mod-rs files (`src/lib.rs`, `src/bridges/mod.rs`), the one arrangement the fix gets right. The issue doc's "Unit coverage proves both directions" is true as far as the test goes but reads as covering the construct generally. Fold this into R6-1's coverage requirement.

### R6-3 — NIT · R5-7's two micro-optimizations were not taken (optional then, optional now)
`rust_interop_sqlx_offline.rs:312-329` still spawns `cargo metadata` before trying `nearest_declared_workspace_root`, so a package whose own `Cargo.toml` carries `[workspace]` (this fixture) still pays one subprocess a TOML read would answer. `validate_sqlx_offline_metadata:93,259` still parses `Cargo.toml` twice, since `backend_may_resolve_sqlx_metadata` re-derives `sqlx_dependency_crate_names` instead of receiving the already-computed `sqlx_crates`. Neither is a correctness issue; the memo half of R5-7 *was* fixed.

### R6-4 — NIT · `rust_interop.rs` remains at 883/900 lines
17 lines of headroom, unchanged by `a191d7202`. Flagged in round 5, still worth watching before the next change lands there.

---

## Required-finding re-audit

**R5-1 (round-5 blocker) — PARTIALLY RESOLVED.** The prescribed change is present and correct: `collect_declared_modules:98-101` now computes `declared_path(module).map_or_else(|| module_dir.join(ident), |path| module_dir.join(path))` before recursing. I confirmed with rustc that this is the right answer for mod-rs parents (`#[path="alt"] mod a { mod b; }` in `lib.rs` → `src/alt/b.rs`), and the mandatory negative test's control now proves it in integration. The residual non-mod-rs cases are **R6-1**.

**R5-2 — CLOSED (wording).** The issue doc now reads "a local traced warm fixture check completed in 2.67 seconds with one SQLx workspace-metadata subprocess instead of 925; an independent environment measured a slower wall clock from pre-existing recursive dependency hashing and one additional general package-resolution metadata invocation." That is an accurate, non-portable-scoped statement of both measurements.

**R5-3 — RESOLVED.** `mod unparseable;` is now declared at `rust_interop_sqlx_offline_tests.rs:103`, and `src/unparseable.rs` (`fn unfinished(`) is written *after* the first assertion, so the second `validate_sqlx_offline_metadata` genuinely exercises the `syn::parse_file(...).ok()` fallthrough at `rust_interop_sqlx_modules.rs:36`. The assertion is no longer vacuous.

**R5-4 — RESOLVED.** `wc -l` gives **665 / 219 / 200**, matching the issue doc exactly.

**R5-5 / R5-6 — RESOLVED (documented).** `internal_docs/rust_interop_architecture.md:1231-1236` now records symlinked and package-escaping module sources, function-body `mod` declarations, and the external `.env` `SQLX_OFFLINE_DIR` cache-identity opt-out as deliberate limitations, and states that the default package/workspace search is required for the certified warm-cache guarantee.

**R5-7 — PARTIALLY RESOLVED.** The unbounded-memo half is fixed: `cargo_workspace_root:290` now does `roots.retain(|(root, _), _| root != backend_root)` before inserting, bounding the memo to one live fingerprint per backend root. The two micro-optimizations were not taken (**R6-3**).

**"Add unit coverage for `[lib].path` and the `main.rs` fallback" — RESOLVED.** `cargo_entrypoint_selection_follows_lib_path_then_main_fallback` pins both: a `[lib] path = "source/entry.rs"` manifest collects `SELECT 13`, and a fixture with `src/lib.rs` deleted collects `SELECT 14` from `src/main.rs`. Symlink and cycle refusal remain uncovered by tests (documented opt-out / verified only by ad-hoc probe).

**R4-1 through R4-5, R3-x, R2-x, R1-x — all remain closed** except as narrowed by R6-1. I re-verified the load-bearing mechanisms directly rather than by reference:
- Module graph rooted at `[lib].path` → `src/lib.rs` → `src/main.rs` (`crate_entry_path:53-74`); gated declarations refused *before* the file is opened (`:94`); inner file attributes honoured (`:39`); symlinks rejected via `symlink_metadata` + `!is_symlink` (`:84-87`); containment via `canonicalize().starts_with(canonical_root)` (`:76-82`); cycles bounded by the `visited` `BTreeSet` (`:30`).
- `cfg` always gates; `cfg_attr` gates only when a non-predicate argument could itself be `cfg`/`cfg_attr` (`:173-199`), with unparseable token streams returning `true` (defer). `module_declaration_may_vary` treats *any* `cfg_attr` on a `mod` as varying, which is correct because `cfg_attr` can inject `#[path]`.
- Subprocess work is outside the mutex: the read guard's scope closes at `:285`, `resolve_cargo_workspace_root` runs at `:286`, the write lock is re-taken at `:287`.
- `sqlx_dependency_crate_names` resolves `workspace = true` and `package = "…"` renames by pure TOML reads (`:199-236`), returning early when no alias uses the workspace (`:203-205`) — no subprocess.
- Forced offline: `configure_hermetic_build_environment:36-39` sets `SQLX_OFFLINE=true` and `env_remove("DATABASE_URL")`; `resolve_cargo_workspace_root:323-324` does the same for its own metadata call. `grep -rn SQLX_OFFLINE_DIR crates/` still finds no process-env read — only the `.env` form disengages.
- Multi-backend final cache identity: `combined_sqlx_offline_metadata_digest:63-90` folds package and workspace `.sqlx` roots for every resolved backend into one digest, keyed by path so a two-backend divergence changes the result.

**Panic safety — clean.** No `unwrap`/`expect`/`panic!`/`unreachable!`/slicing/`unsafe` anywhere in the three new production files. The single index, `segments[0]` at `rust_interop_sqlx_offline.rs:501`, is guarded by `segments.len() >= 2` in the same condition. Every non-exhaustive `match` arm in `rust_interop_sqlx_cfg.rs` returns `true` (⇒ skip ⇒ defer to Cargo), so new `syn` variants fail open; `Verbatim` arms return `false`, harmless because `syn::visit` yields no `Macro` nodes inside a `TokenStream`. All parses are `.ok()`, all I/O is `Result`/`Option`-handled.

**Direct-read inventory — current and accurate.** The five renumbered `rust_interop_sqlx_offline.rs` anchors updated in `a191d7202` (343/352/545/629/633) all land on real probe sites: `nearest_declared_workspace_root`'s manifest read, `declared_workspace_root`'s manifest read, `QuerySource::File`'s query-file read, the `.sqlx` `is_file()` probe, and the metadata read. The remaining nine anchors are unchanged and correct. The guard scans all of `crates/sifr_driver/src` and PASSes.

**Matrix / provenance / stable claims / responsibility split — sound.** 36 rows / 36 fixture rows / 3 categories, 44 crates, 61 package examples, 18 scenario examples, 36 stable claims, `future_runtime_rows=0`; the completed-matrix and missing-active-category self-tests pass; the resource gate accepts zero deferrals while still requiring passing supported stdlib-core rows. Docs are fixture-scoped throughout: `docs/rust-interop.mdx:244-260` and `internal_docs/rust_interop_architecture.md:1259-1285` both explicitly disclaim arbitrary framework surfaces, live database connectivity, and product-level web-framework support. The only inaccurate documentation sentence is the `#[path]` claim under R6-1. Split at 665 / 219 / 200 with `rust_interop_sqlx_offline.rs` well under the cap (**R6-4** for `rust_interop.rs`).

---

## Validation results

All commands run at `HEAD = a191d7202` with the working tree in its as-found state.

| Check | Recorded | My result |
|---|---|---|
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | 12 pass | **12 passed, 0 failed** (0.20 s) |
| `cargo test -p sifr_driver --lib` | 449 / 65 ignored | **449 passed, 0 failed, 65 ignored** (27.17 s) |
| Mandatory negative (`.env`-armed, gated + inline-path regressions) | pass, 42.28 s | **pass, 41.08 s** |
| Mandatory positive (Axum loopback + SQLx offline) | pass, 55.77 s | **pass, 48.95 s** |
| Rust-interop area runner | 10/10, 229 mutations | **variants=10, failures=0, blocking=0**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5+6, compat 36/36/3 + 7, 20 stale-draft, 36 claims + 33 |
| `cargo clippy --workspace -- -D warnings` | pass | **pass** |
| `cargo fmt --check` | pass | **pass** |
| File-size guardrail | pass | **PASS** (3011 files, limit 900) |
| `sifr_driver` maintainability | pass | **PASS** |
| TypeScript-Go transfer guardrails | pass | **PASS** |
| Resource gate + `--self-test` | pass | **PASS** (`surfaces=1`, `future_runtime_rows=0`); self-test PASS |
| `git diff --check origin/main..a191d7202` | pass | **clean** |
| Production split sizes | 665 / 219 / 200 | **665 / 219 / 200** |
| `#[path]` on inline mod in a mod-rs file | fixed | **fixed, confirmed against rustc and via `sifr check`** |
| `#[path]` in a non-mod-rs parent (3 layouts) | not run | **false rejection CONFIRMED ×3 on clean-compiling crates (R6-1)** |

Every recorded figure reproduced or came in faster. Nothing in the branch is overstated except the architecture doc's `#[path]` sentence.

I did not run `scripts/run_all_tests.sh --profile create-pr`; that gate belongs to the still-open final checklist item and the blocker below makes it premature.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** `assert_exact_backend_dependency_graph` runs `cargo tree --workspace --edges features --locked --offline` and pins `axum v0.8.9`, `tower-http v0.7.0` + `set-header`, `sqlx v0.8.6` + `runtime-tokio-rustls`/`postgres`/`macros`; the area runner re-verifies 44 aliases. |
| Hermetic `127.0.0.1:0` Axum service, real tower-http, deterministic shutdown | **Met.** Positive test observes the exact marker string incl. `status=200;tower-http=0.7.0;middleware=response-header;shutdown=clean`, and asserts empty stderr. |
| Real SQLx macro from checked-in `.sqlx` under forced `SQLX_OFFLINE`, no live DB | **Met.** Query hash bound in the runtime marker; `assert_database_sentinel_unused` passes at all three checkpoints. |
| Mandatory generated-package diagnostic: independent missing + stale, stable `SIFR-RUST-CARGO-0001`, DB/network disabled | **Met**, and further strengthened — the control now also carries a never-compiled inline-`#[path]` query (`SELECT 98`) and still reaches Cargo with `errors.is_empty()`, so the round-5 regression is pinned in integration, not only in units. |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met with one inaccuracy.** All counts re-verified. Wording is fixture-scoped and precise except the architecture doc's `#[path]`-redirect claim (**R6-1**), and the unqualified "both directions" unit-coverage claim (**R6-2**). |
| Validator self-tests without weakening earlier rows | **Met.** 229 cases, 10/10, no prior row regressed. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked. As in every prior round, the remaining gap sits in production code the fixture's own layout does not currently reach — though this time the affected file kind (`src/bridges/backend.rs`) *is* part of the certified package, so a regression test can reach it without inventing a new scenario.

---

## Required fixes

1. **R6-1** — Replace `PendingModule`'s single `module_dir` with rustc's `dir_path` + `relative: Option<String>` model so `#[path]` resolves against `dir_path` without the pending `relative` component, and so a `#[path]`-loaded non-`mod.rs` file's children resolve in that file's own directory. Add regression coverage for layouts **A**, **B**, and **C** in both directions, declared from a *file* module (not `src/lib.rs`/`mod.rs`).
2. **R6-1 (docs)** — Correct `internal_docs/rust_interop_architecture.md:1224-1226` once the resolver is fixed, or narrow the "including active `#[path]` redirects" claim to what the resolver actually guarantees.

## Optional suggestions

- **R6-2** — Qualify or broaden the issue doc's "Unit coverage proves both directions" once R6-1's tests land.
- **R6-3** — Try `nearest_declared_workspace_root` before spawning `cargo metadata`; thread `sqlx_crates` into `backend_may_resolve_sqlx_metadata` to drop the duplicate manifest parse.
- Add unit coverage for symlink and cycle refusal — both are load-bearing and currently verified only by ad-hoc probes.
- **R6-4** — Watch `rust_interop.rs` (883/900) before the next change lands there.
