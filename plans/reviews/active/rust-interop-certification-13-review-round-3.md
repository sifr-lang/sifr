I completed the round‑3 audit. Repository files were not modified (all probes ran in `/tmp` and were removed; `git status` is unchanged apart from the pre‑existing parallel‑agent paths).

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 3

## Verdict: **NOT SATISFIED**

All three round‑2 required fixes are genuinely and correctly implemented — I confirmed the `.env` sentinel mechanism empirically in both directions, so the "Sifr's forcing is load‑bearing" claim is now true, workspace‑root `.sqlx` is accepted, and final‑build cache identity now folds every resolved bridge backend. One new blocker: the preflight **falsely rejects valid packages whose `sqlx::query!` sites are `cfg`-disabled** — code that Cargo accepts in the exact shape Sifr's own probe uses. I reproduced it end‑to‑end with the shipped binary. This is the same "valid code now fails" class that blocked rounds 1 (F1) and 2 (R2), and it is squarely inside the mechanism the round‑2 disposition itself named.

---

## Scope reviewed

Committed delta `origin/main..HEAD` (`b231daf81`, `7a27b7896`, `6ec0742b6`, `0e53989be`), 50 files, focused on `0e53989be`. Excluded and not attributed to this milestone: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.claude/`, the two stray webp files, and `plans/phases/43_interoperability.md`. `plans/reviews/active/rust-interop-certification-13-review-round-3.md` exists as an empty untracked placeholder; per instructions I did not write to it.

---

## Round‑2 required-finding re-audit

**R1 — RESOLVED, and now empirically load-bearing.** `configure_dotenv_database_sentinel` (`package_rust_interop_backend_ecosystem_support.rs:130-145`) writes `DATABASE_URL=postgres://…@127.0.0.1:<ephemeral>/sifr` into the copied package root's `.env`, which is `CARGO_MANIFEST_DIR` for the bridge crate. I verified the full counterfactual out-of-repo, reproducing Sifr's probe shape exactly (a crate in `/tmp` path-depending on a copy of the fixture package, `cargo check --offline`):

```
SQLX_OFFLINE unset, DATABASE_URL unset, fixture .env armed at 127.0.0.1:1
  → error: error communicating with database: Connection refused (os error 61)
    at pkg/src/bridges/backend.rs:38  (note: sqlx_macros::expand_query)
SQLX_OFFLINE=true, DATABASE_URL removed        → Finished, no connection
SQLX_OFFLINE=true, ambient DATABASE_URL armed  → Finished, no connection
```

This matches sqlx 0.8.6 source exactly: `init_metadata` takes `database_url` from `env("DATABASE_URL").ok().or(<from .env>)` and takes the live branch only when `offline == false` (`sqlx-macros-core-0.8.6/src/query/mod.rs:118-163`). So absent Sifr's forcing the build *does* dial out, and with it the build cannot. The fixture `.cargo/config.toml` is down to `[net] offline = true` — no SQLx override. The probe cache cannot mask this: `probe_cache_key` digests the whole backend root (`rust_interop_probe_cache.rs:63`, `:117-129`), and the sentinel port changes `.env` every run, so the control genuinely re-spawns Cargo each time (my negative run: 27.48s).

One precision issue survives (**R3‑3** below): only the **valid control** carries this proof. Both mutations are rejected before Cargo spawns, so they would observe no connection with or without forcing.

**Inherited `DATABASE_URL` removal — proportionate.** Because sqlx short-circuits on `offline`, an ambient `DATABASE_URL` is already inert under forced offline mode (my third case above proves it). `env_remove` is therefore defense-in-depth, not a load-bearing control, and asserting it directly on the `Command` (`rust_interop_sqlx_offline_tests.rs:13-31`, `environment.get("DATABASE_URL") == Some(&None)`) is the right level of evidence. Adding a fixture case for it would prove nothing extra. The published wording ("removes inherited `DATABASE_URL`") is a factual statement about the command, not an unsupported behavioural claim — accurate as written.

**R2 — RESOLVED.** `sqlx_metadata_roots` (`:201-215`) now returns `[backend_root/.sqlx, workspace_root/.sqlx]`, and `validate_query_metadata` (`:579-604`) takes the first existing file across roots. I diffed this against sqlx's real resolver (`query/mod.rs:165-175`): `SQLX_OFFLINE_DIR` → `manifest_dir/.sqlx` → `workspace_root().join(".sqlx")`, selected by `.find(|p| p.exists())`. Order and semantics match; Sifr's `.is_file()` vs sqlx's `.exists()` differs only in the fail-open direction. `cargo_workspace_root` mirrors sqlx's own mechanism (`Metadata::workspace_root` shells out to `cargo metadata --format-version=1 --no-deps`) with an ancestor-scan fallback. Explicit `SQLX_OFFLINE_DIR` disengages the preflight *and* the digest, covered by `explicit_offline_directory_disengages_conservative_preflight`; `dotenv_defines_offline_dir` correctly rejects `#`-commented lines and `SQLX_OFFLINE_DIRECTORY`-style prefixes while accepting `export ` and `KEY = value`. `workspace_metadata_and_workspace_dependency_renames_are_resolved` pins the accepted workspace-root layout. No new false rejection from this change.

**R3 — RESOLVED.** `sqlx_offline_metadata_digest` now has a dedicated field (`RustInteropCargoInputs::sqlx_offline_metadata_digest`, emitted at `rust_interop_plan.rs:590-597`, so it is genuinely part of `cache_key_fragment` → `binary_project_cache_key`), and `rust_interop.rs:236-245` unions `package.package_root` with `cargo_manifest_path.parent()` for **every** `pending_direct_probes` entry. I confirmed `pending_direct_probes` is populated for all resolved direct backends at `:462` regardless of later probe-cache hits, and that `self.probes` is only a codegen descriptor list with no backend root — so there is no second class of backend that escapes. The assignment happens *after* `combined_cargo_inputs`, so the sysroot fold cannot clobber it. Probe-side identity uses the per-backend digest (`rust_interop_probe_cache.rs:76-79`) and now includes the workspace root too. `complete_metadata_directory_participates_in_cache_identity` pins a `describe`-only mutation across two roots.

**R4 — Partially closed; the stated rationale is now self-contradicted.** `[workspace.dependencies]` renames are handled (`:169-199`, `:150-161`). `dev-dependencies`/`build-dependencies` were deliberately skipped on the grounds that "the source scanner only covers normal crate `src/` and activating preflight from test/build-only tables would create false positives." That reasoning is correct — and it is exactly the bug in **R3‑1**: the identical false-positive mechanism already fires today for `#[cfg(test)]` code when sqlx is a *normal* dependency. The disposition is acceptable **only** once the cfg guard exists; without it, the milestone shipped the half of the hazard it had already identified.

**R5 — RESOLVED.** The `hash`-equality check is gone; `validate_query_metadata` compares only `query`, matching sqlx (`data.rs:117-119`). The stale message now names the real condition ("saved SQLx query text does not match query identity") and the negative test asserts on it.

**R6 — RESOLVED.** `_scenario_backend.py:319-324` type-checks `describe` and returns a clean failure line; the new `"metadata describe shape"` mutation (`:249-252`) pins it. No traceback path remains.

**R7 — RESOLVED.** Dedicated `sqlx_offline_metadata_digest` field; `cargo_metadata_digest` no longer doubles as the SQLx carrier on the package path, and `combine_optional_digest` folds it independently with `assert_ne!` coverage on both sides (`rust_interop_cargo_inputs.rs:491-498`).

**File split / inventory.** `rust_interop_sqlx_offline.rs` 618 lines, `_tests.rs` 297, `rust_interop_probe.rs` 868, `rust_interop.rs` 883, `_scenario_backend.py` 386 — all under 900 (`rust_interop.rs` has only 17 lines of headroom; worth watching). The TypeScript-Go direct-read guard passes; see **R3‑5** for an accuracy caveat in how it passes.

---

## Findings (severity order)

### R3‑1 — HIGH · New false rejection: `cfg`-disabled `sqlx::query!` sites are treated as required metadata
`crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:326-347` (`collect_module_queries`), `:438-447` (`SqlxQueryVisitor`)

The recognizer walks every `Item` and every macro in every `.rs` file under `src/` with **no attribute awareness at all** — `grep cfg` in the module matches only the `#[cfg(test)]` on its own test module. `syn` has no cfg evaluation, so queries inside `#[cfg(test)] mod …`, `#[cfg(feature = "…")] fn …`, or any never-enabled cfg are collected and their metadata demanded. Cargo never expands them, so they legitimately have no `.sqlx` entry — and `cargo sqlx prepare`'s documented default does not cover test targets.

Reproduced end-to-end with the built `target/debug/sifr` on a `/tmp` copy of the certified fixture (baseline copy: `no errors found`):

```
# appended to src/bridges/backend.rs
#[cfg(test)]
mod cfg_gated_tests { #[test] fn t() { let _ = sqlx::query!("SELECT 99::INT4 AS value"); } }

cargo check --offline  (path-dep probe crate, SQLX_OFFLINE=true)  → Finished, clean
sifr check src/main.sifr                                          → error[SIFR-RUST-CARGO-0001]:
  Rust bridge SQLx offline metadata failed for `main.route_probe`:
  `SQLX_OFFLINE=true` but there is no cached data for this query: SELECT 99::INT4 AS value
```

Same result for `#[cfg(feature = "mysql-variant")] pub fn …` where the feature is not even declared in `Cargo.toml` — i.e. unconditionally dead code that no Cargo invocation can ever compile still hard-fails `sifr check`. Multi-database sqlx crates with per-driver `#[cfg(feature = …)]` query variants, and bridge crates with `#[cfg(test)]` query tests, are both ordinary patterns; there is no override, and the error text misdescribes the cause.

**Fix:** fall through for any item, inline module, statement, or expression carrying a `cfg`/`cfg_attr` attribute the preflight cannot prove active — the same "unknown ⇒ Cargo is the authority" rule the rest of this module already follows. Add regression tests for `#[cfg(test)] mod`, `#[cfg(test)] fn`, and `#[cfg(feature = …)]`. Once that guard exists, the R4 dev/build-dependency disposition becomes internally consistent and can stay as-is.

### R3‑2 — LOW · Ambient `SQLX_OFFLINE_DIR` silently removes `.sqlx` from cache identity, though sqlx ignores it for reads
`rust_interop_sqlx_offline.rs:201-206`

`sqlx_metadata_roots` returns `None` when the process env defines `SQLX_OFFLINE_DIR`, disengaging both the preflight and the digest. But in sqlx 0.8.6 the read-path `offline_dir` is populated **only** from `.env` — `init_metadata` (`query/mod.rs:118-135`) overrides `offline` and `database_url` from the process env but passes `offline_dir` straight through from `load_dot_env`; the process variable is consulted only on the *save* path (`:346`). So with `SQLX_OFFLINE_DIR` exported in a developer's shell, Cargo still reads the package `.sqlx` while Sifr drops it from cache identity, leaving a warm final build reusable across `.sqlx` edits. The published claim ("Package- and workspace-root `.sqlx/` directories … participate in probe and generated-build cache identity") is unconditional. Restrict the disengagement to the `.env`-declared form (which is what sqlx honours), or keep the digest even when disengaging the preflight.

### R3‑3 — LOW · "Both mutations complete without contacting the sentinel" is not evidence of forcing
`docs/rust-interop.mdx:255-258`, `internal_docs/rust_interop_architecture.md:1263-1268`, `plans/phases/39_rust_interop.md:352-355`, `plans/issues/…certification.md:1552-1556`, fixture `README.md:12-16`, `rust_interop_compatibility_matrix.json:406`

The two mutations are rejected by the preflight *before any Cargo process is spawned*, so their no-connection outcome is independent of `SQLX_OFFLINE`. Only the valid control exercises the forced environment. The current phrasing ("The valid control and both metadata mutations complete without contacting that sentinel, so the compiler's offline forcing is load-bearing"; and in the issue, "both mutations pass without a connection only because Sifr forces `SQLX_OFFLINE=true`") attributes to all three what one establishes. Attribute the proof to the control and describe the mutations as pre-Cargo rejections.

### R3‑4 — LOW · Cache identity now depends on a `cargo metadata` subprocess for every Rust-interop build, with a lock held across it and a never-invalidated memo
`rust_interop_sqlx_offline.rs:229-280`

`combined_sqlx_offline_metadata_digest` calls `sqlx_metadata_roots` → `cargo_workspace_root` unconditionally at `rust_interop.rs:243`, i.e. for every package with any Rust-interop declaration, sqlx or not. Measured cost is small (~39 ms per distinct root, memoized per process; the failure path for a root without `Cargo.toml` is equally fast), but three properties are worth tightening: the memo `Mutex` is held **across** the subprocess, serialising all threads; the memo is process-lifetime with no invalidation, so a long-lived driver (LSP/watch) keeps a stale workspace root after a user adds an outer `Cargo.toml`; and cache identity now varies with whether `cargo metadata` succeeded versus fell back to the ancestor scan. All three are conservative (extra invalidation, never a false hit). Short-circuit when neither a `.sqlx` directory nor an sqlx dependency is present, and resolve outside the lock.

### R3‑5 — NIT · The TypeScript-Go direct-read inventory now passes on a union of stale and fresh line references
`internal_docs/typescript_go_architecture_transfer_guardrails.md:82-101`

`validate_direct_fs_inventory` only checks that every current probe site appears *somewhere* in the doc; it never checks that listed references are real. The main table row still lists `rust_interop_sqlx_offline.rs:60,76,117,140,146,149,333,416,580`, none of which are probe sites after the refactor, and the new append-only paragraph carries the 12 real ones (plus five `rust_interop_cargo_inputs.rs` lines, with `:141` covered only by the older row). The guard is green and the inventory is complete, but the row is now misleading. Fold the fresh numbers into the row and drop the dead ones. (`fs::symlink_metadata` at `:302` is out of the pattern's scope by design.)

### R3‑6 — NIT · SQLx metadata failures are attributed to the first probe declaration, not the query
The diagnostic above blames `main.route_probe` for a query in an unrelated module. The preflight is package-scoped but reported per-probe, so the `{target}` argument is arbitrary. Consider naming the offending source file instead, or stating that the failure is package-wide.

### R3‑7 — NIT · Ancestor fallback can bind an unrelated outer workspace's `.sqlx` into cache identity
`nearest_declared_workspace_root` (`:272-280`) walks ancestors until any `Cargo.toml` with a `[workspace]` table. For a Sifr package root with no `Cargo.toml` of its own inside a user's Rust monorepo, an unrelated top-level `.sqlx` would be digested into Sifr's cache key. Conservative (extra invalidation only), but noise.

### Observations (not defects)
- No user-triggerable panics in the new production code: every `syn::parse2`/`parse_file` is `.ok()`, all I/O is `Result`/`Option`-handled, `Mutex` poisoning is handled by `if let Ok`, `sha256_hex` cannot fail, and the only recursion (`collect_module_queries`, `syn::visit`) is bounded by what `syn` already accepted. `collect_rust_sources` remains an iterative, symlink-skipping worklist.
- This milestone makes compile-time live-database SQLx expansion **impossible** by policy (forced `SQLX_OFFLINE=true` plus `DATABASE_URL` removal). That is a deliberate behaviour change for packages that previously relied on a live `DATABASE_URL`, and it is documented as such in `rust_interop_architecture.md:1214-1219` and `docs/rust-interop.mdx`. Correct for a hermetic-build guarantee; worth keeping visible in release notes.
- The probe-layer boundary noted in round 2 still stands and is still not overclaimed: `describe`-only drift is caught by final-build cache identity, not by `sifr check`.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** `cargo tree --workspace --edges features --locked --offline` asserts `axum v0.8.9`, `tower-http v0.7.0` + `set-header`, `sqlx v0.8.6` + the three frozen features (`…support.rs:157-190`); passed in my run. |
| Hermetic `127.0.0.1:0` Axum service via real tower-http, deterministic shutdown | **Met.** Positive mandatory test passed (49.37s) with the full marker `axum=0.8.9;loopback=…;status=200;tower-http=0.7.0;middleware=response-header;sqlx=0.8.6;offline=true;query-value=13;query-hash=f2d6…;shutdown=clean`. |
| Real SQLx macro from checked-in `.sqlx` under `SQLX_OFFLINE=true`, no `DATABASE_URL`/live DB | **Met, and now actually proven.** The `.env`-armed control plus my counterfactual establish that forcing — not fixture configuration — is what prevents the connection. |
| Mandatory generated-package diagnostic: independent missing + stale mutation, stable `SIFR-RUST-CARGO-0001`, DB/network disabled | **Met for the fixture** (negative test passed, 27.48s), with the attribution caveat in **R3‑3**. |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met**, counts verified to the unit: 36 rows / 36 fixture_rows / 3 categories; 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`; 13 `cargo-probe` / 4 `compiler-diagnostic` / 10 `contract-only` / 9 `runtime-observed`; 72 passing / 0 planned; 44 crates; 61 package examples; 18 scenario examples; 36 stable claims; `future_runtime_rows=0`. Wording issues per **R3‑2**/**R3‑3**. |
| Validator self-tests without weakening earlier rows | **Met.** 229 mutation cases pass, up one for the new `describe`-shape case; no prior row regressed (10/10). |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked. As in round 1, the gap is in the production code the checklist ships — the fixture-scoped acceptance criteria cannot exercise **R3‑1**.

---

## Validation assessment

Everything recorded reproduced, and no recorded figure was overstated.

| Check | My result |
|---|---|
| `cargo test -p sifr_driver --lib` | **446 passed, 0 failed, 65 ignored** (28.46s) — matches |
| `cargo test -p sifr_codegen --lib` | **932 passed, 0 failed** — matches |
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | **9 passed** — matches |
| Positive mandatory generated-package test | **pass, 49.37s** (recorded 46.39s) |
| Negative `.env`-armed missing/stale test | **pass, 27.48s** (recorded 44.78s; warm Cargo target) |
| Rust-interop area runner | **10/10 variants, 0 failures**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5, compat 36/36/3 + 7 self-tests, 20 stale-draft cases, 36 claims + 33 self-tests |
| `cargo clippy --workspace -- -D warnings` / `cargo fmt --check` | pass / pass |
| file-size (3009 files, limit 900) / driver maintainability / HIR maintainability | PASS / PASS / PASS |
| resource gate + `--self-test` | PASS (`surfaces=1`, `future_runtime_rows=0`) |
| TypeScript-Go transfer guardrails | PASS (caveat **R3‑5**) |
| `git diff --check origin/main..HEAD` | clean |
| Empirical sqlx 0.8.6 probes (out-of-repo) | forcing counterfactual **confirmed**; `cfg`-gated false rejection **confirmed** |

The one evidence statement I would now correct is the round‑2-fix bullet at `plans/issues/…certification.md:1552-1556` ("both mutations pass without a connection only because Sifr forces `SQLX_OFFLINE=true`"), per **R3‑3**.

---

## Required fixes

1. **R3‑1** — Make the preflight fall through for `cfg`/`cfg_attr`-gated items, modules, statements, and expressions, so `#[cfg(test)]` and feature-gated `sqlx::query!` sites can no longer produce a false `SIFR-RUST-CARGO-0001`. Add regression tests for `#[cfg(test)] mod`, `#[cfg(test)] fn`, and an undeclared `#[cfg(feature = …)]`. Then re-state the R4 dev/build-dependency disposition, which only holds once this guard exists.

## Optional suggestions

- **R3‑2** — disengage only for `.env`-declared `SQLX_OFFLINE_DIR` (what sqlx 0.8.6 actually honours for reads), or keep the digest when disengaging the preflight.
- **R3‑3** — attribute the load-bearing proof to the valid control in all six places; describe the mutations as pre-Cargo rejections.
- **R3‑4** — skip `cargo metadata` when no `.sqlx` and no sqlx dependency exist, resolve outside the memo lock, and bound the memo's lifetime.
- **R3‑5** — fold the current probe line numbers into the driver inventory row and drop the stale ones.
- **R3‑6** — name the offending source file (or state package scope) instead of an arbitrary probe target.
- **R3‑7** — bound the ancestor workspace scan to the Sifr package/repo boundary.
