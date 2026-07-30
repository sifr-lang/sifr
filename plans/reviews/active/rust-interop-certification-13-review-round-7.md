Repository files unmodified; all probes ran in `/tmp` and were removed. (I rebuilt `target/debug/sifr` and test artifacts — gitignored.) `plans/reviews/active/rust-interop-certification-13-review-round-7.md` was already present as an untracked placeholder before this session; I did not write to it.

---

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 7

## Verdict: **SATISFIED**

## Executive summary

Round-6 blocker **R6-1 is completely and correctly resolved**. The commit replaces `PendingModule`'s single `module_dir` with rustc's actual two-state model (`ModuleDirectory { dir_path, relative }`), and `#[path]` lookup now anchors to `dir_path` while deliberately skipping the pending `relative` component — which is precisely what `rustc_expand::module` does.

I did not take this on structural resemblance. I established the ground truth empirically against **rustc 1.94.0** across eleven layouts, then reproduced both directions of layouts A, B, and C end to end on the certified fixture from its real non-`mod.rs` parent (`src/bridges/backend.rs`). All three of round 6's false `SIFR-RUST-CARGO-0001` rejections are gone; all three compiled-file queries are now caught pre-Cargo instead of leaking to rustc stderr. Every recorded validation figure reproduced.

One residual fail-open edge exists (raw-identifier module names), but it is pre-existing, not in R6-1's class, produces a miss rather than a false rejection, and is consistent with the documented "Cargo is the authority" policy. It is an optional suggestion, not a blocker.

---

## Scope reviewed

Committed delta `origin/main..7c37a86da` (`b231daf81`, `7a27b7896`, `6ec0742b6`, `0e53989be`, `bfa7f27c6`, `96a56b7f1`, `a191d7202`, `7c37a86da`), 56 files, focused on `7c37a86da`. Excluded and not attributed: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.claude/`, the two stray webp files, `plans/phases/43_interoperability.md`.

---

## 1. Resolver re-audit against actual rustc/Cargo semantics

`crates/sifr_driver/src/build/rust_interop_sqlx_modules.rs:51-178`

The implemented model:

| Construct | Sifr next-state | rustc equivalent |
|---|---|---|
| crate root (`[lib].path` / `lib.rs` / `main.rs`) | `dir_path = parent(entry)`, `relative = None` | root `ModuleData`, `Owned { relative: None }` |
| flat `mod m;` → `<base>/m.rs` | `dir_path = base`, `relative = Some("m")` | `default_submod_path` → `Owned { relative: Some(ident) }` |
| nested `mod m;` → `<base>/m/mod.rs` | `dir_path = base/m`, `relative = None` | `Owned { relative: None }` |
| inline `mod m { … }` | `dir_path = plain_base()/m`, `relative = None` | pushes `relative` **then** ident, clears relative |
| inline `#[path = p] mod m { … }` | `dir_path = dir_path.join(p)`, `relative = None` | `mod_file_path_from_attr(attrs, &module.dir_path)` used *as* the dir (the documented "historical reasons" quirk) |
| `#[path = p] mod m;` (loaded file) | `source = dir_path.join(p)`, child `dir_path = parent(source)`, `relative = None` | `#[path]` files "treated as though they are a `mod.rs` file" |

Both `#[path]` forms read `directory.dir_path` directly, never `plain_base()`. That is the exact distinction round 6 required.

**Empirical verification (rustc 1.94.0, `--crate-type lib`, dead-code warnings used to observe which file was actually compiled).** In every case rustc compiled the file Sifr's model predicts, and never the decoy:

| Probe | Layout | rustc compiled | Sifr model | Match |
|---|---|---|---|---|
| **A** | `#[path="direct.rs"] mod d;` in non-mod-rs `src/outer.rs` | `src/direct.rs` | `src/direct.rs` | ✅ |
| **B** | `#[path="alt"] mod i { mod child; }` in `src/outer.rs` | `src/alt/child.rs` | `src/alt/child.rs` | ✅ |
| **C** | `#[path="loaded.rs"] mod l;` in `src/sub/mod.rs`, `l` declares `mod kid;` | `src/sub/kid.rs` | same | ✅ |
| **C2** | same, from non-mod-rs `src/outer.rs` | `src/kid.rs` | same | ✅ |
| **D** | `#[path="x.rs"]` two file-module levels deep (`src/outer/a.rs`) | `src/outer/x.rs` | same | ✅ |
| **E** | inline `mod inner { mod kid; }` in `src/outer.rs` | `src/outer/inner/kid.rs` | same | ✅ |
| **F** | inline `mod ins { mod kid; }` inside a `#[path]`-loaded file | `src/ins/kid.rs` | same | ✅ |
| **H** | `#[path="alt.rs"] mod a { mod b; }` (dir-path quirk) | `src/alt.rs/b.rs` | same | ✅ |
| **I** | `#[path="x.rs"]` inside `src/a/mod.rs` | `src/a/x.rs` | same | ✅ |
| **J** | `#[path="sub/loaded.rs"]`, then plain child | `src/sub/kid.rs` | same | ✅ |
| **K** | three-deep relative chain, `#[path]` inside an inline mod | `src/p/q/ins/z.rs` | same | ✅ |

**End-to-end on the certified fixture** (`/tmp` copy, baseline `sifr check src/main.sifr` → `no errors found`, 23.4 s), with A/B/C installed in the real `src/bridges/backend.rs`:

- *False-rejection direction* (unprepared queries in the never-compiled default directories `src/bridges/backend/…`): `cargo check --offline --lib` → **clean, 17.20 s**; `sifr check` → **`no errors found`**. All three of round 6's false `SIFR-RUST-CARGO-0001` diagnostics are gone.
- *Active-query direction* (each unprepared query moved into the file rustc actually compiles): `sifr check` → **`error[SIFR-RUST-CARGO-0001]` … `SELECT 881` / `SELECT 882` / `SELECT 883`**, each with the package-scoped message and note — i.e. rejected *pre-Cargo*, not leaked through rustc stderr. Round 6's mirror-image miss is closed.

**Other transitions re-verified against the rewritten resolver:**

- Nested `foo/mod.rs` from a non-mod-rs parent: `mod nested;` in `backend.rs` → `src/bridges/backend/nested/mod.rs` correctly scanned (`SELECT 901` rejected pre-Cargo).
- Cycle refusal: self-referential `#[path = "mod.rs"] mod selfref;` — no hang, resolved in 10.2 s, query still rejected. The `visited` `BTreeSet` on canonical paths bounds it.
- Ambiguity: `(true, true)` for both `foo.rs` and `foo/mod.rs` returns `None` (skip), matching rustc's `MultipleCandidates` error being someone else's problem — fail-open, no false rejection.
- Containment: `#[path]` escaping the root (relative or absolute) is rejected by `regular_source_path`'s `canonicalize().starts_with(canonical_root)`; children of an escaped module are consequently unreachable. Fail-open.
- Symlinks: `is_regular_file` (`symlink_metadata` + `!is_symlink`) unchanged by this commit; documented opt-out.
- Malformed `#[path(...)]` (`Meta::List`) → `declared_path` returns `None` → default lookup; rustc rejects such a crate outright, so no false-rejection path.
- Multiple `#[path]` attributes → `find_map` takes the first, matching `first_attr_value_str_by_name`.
- `cfg`/`cfg_attr`, gated/orphan sources, parse tolerance, `[lib].path`, `main.rs` fallback: `rust_interop_sqlx_cfg.rs` and `crate_entry_path` are untouched by `7c37a86da`; the four unit tests covering them pass, and round 5/6's probe findings stand.

**No remaining false preflight rejection found.** One remaining active-query miss — see N-1.

---

## 2. Required coverage verification

**`explicit_paths_from_file_modules_follow_rust_directory_rules` (`rust_interop_sqlx_offline_tests.rs:280-337`) — pins A/B/C in both directions.** The declaring file is `src/outer.rs`, a genuine *file* module (not `src/lib.rs`, not a `mod.rs`), which is the file kind round 6 required. It installs six decoys/targets and asserts `collect_sqlx_queries == ["SELECT 13", "SELECT 14", "SELECT 15"]` — exact vector equality, so the never-compiled `SELECT 90/91/92` siblings are pinned as *absent* while the compiled ones are pinned as *present*. `queries.sort()`/`dedup()` at `rust_interop_sqlx_offline.rs:371-372` makes the expected order deterministic.

I confirmed the test's layout is **real Rust, not a fantasy**: replicated verbatim under rustc 1.94, which compiled exactly `q13`, `q14`, `q15` and none of `q90`, `q91`, `q92`. The test therefore pins actual rustc semantics.

**Mandatory generated-package negative (`package_rust_interop_backend_ecosystem_support.rs:202-259`) — installs A/B/C in the real `src/bridges/backend.rs`.** I verified `backend.rs` is reached as `src/lib.rs` → `src/bridges/mod.rs` (mod-rs) → `pub mod backend;` → `src/bridges/backend.rs`, i.e. exactly the non-mod-rs file module carrying this fixture's SQLx macros. The three appended declarations pair each compiled target with a never-compiled `src/bridges/backend/…` sibling holding `SELECT 97` / `96` / `95`.

- Clean control reaches Cargo: `check_package_project` → `control.is_empty()` **asserted and passing**, so no false diagnostic from any of the three layouts.
- Never-compiled sibling queries ignored: the same assertion is what would fail if they were scanned.
- Missing and stale remain **independently** rejected: `remove_file` then `assert_sqlx_metadata_mutation_is_rejected(Missing)`, then a distinct `replacen`-based stale write then `…(Stale)`, each requiring `SIFR-RUST-CARGO-0001` plus its own detail string (`there is no cached data for this query` vs `saved SQLx query text does not match query identity`).
- `SQLX_OFFLINE` forced and no database used: the missing case additionally asserts the rendered diagnostic contains `SQLX_OFFLINE=true`; `configure_hermetic_build_environment:36-38` sets `SQLX_OFFLINE=true` and `env_remove("DATABASE_URL")`; `resolve_cargo_workspace_root:323-324` does the same for its own metadata call; `assert_database_sentinel_unused` passes at all three checkpoints against a live armed `.env` `DATABASE_URL`. The fixture's `.cargo/config.toml` contains only `[net] offline = true` — no SQLx override — so the compiler's forcing is genuinely load-bearing.

**Test passes: 43.33 s** in my environment.

---

## 3. Re-audit of all prior findings and cross-cutting concerns

**R6-1 — RESOLVED** (production + coverage + docs), per sections 1–2.

**R6-2 — RESOLVED.** The issue doc's new bullet no longer says "both directions" unqualified: "Unit coverage proves all three affected layouts in both directions: a redirected file module, a redirected inline module, and children of a path-loaded file." Accurate.

**R6-3 — NOT TAKEN (was optional, remains optional).** `resolve_cargo_workspace_root:311-329` still spawns `cargo metadata` before trying `nearest_declared_workspace_root`; `validate_sqlx_offline_metadata:91` and `backend_may_resolve_sqlx_metadata:255` still parse `Cargo.toml` twice. No correctness impact.

**R6-4 — UNCHANGED.** `rust_interop.rs` is still 883/900.

**R5-1 → R5-7, R4-x, R3-x, R2-x, R1-x — all remain closed.** Re-verified the load-bearing mechanisms directly at current line numbers rather than by reference:

- **Subprocess/memo/cache.** `cargo_workspace_root:271-291`: the read guard's `if let Ok(roots)` block closes before `resolve_cargo_workspace_root(backend_root)` runs, and the write lock is re-taken after — subprocess work is outside the mutex. The memo is bounded by `roots.retain(|(root, _), _| root != backend_root)` before insert, giving one live fingerprint per backend root. `workspace_resolution_fingerprint` hashes every ancestor manifest's bytes, so manifest edits invalidate. `workspace_dependency_packages:199-235` and `nearest_declared_workspace_root`/`declared_workspace_root` are pure TOML reads with an early return when no alias uses the workspace — no subprocess.
- **Panic safety — clean.** `grep -nE '\.unwrap\(\)|\.expect\(|panic!|unreachable!|todo!|unsafe|\[[0-9]+\]'` across all three new production files returns exactly one hit: `segments[0]` at `rust_interop_sqlx_offline.rs:501`, guarded by `segments.len() >= 2` in the same `&&` condition. All parses are `.ok()`, all I/O `Result`/`Option`-handled. `rust_interop_sqlx_cfg.rs` non-exhaustive arms still return `true` (⇒ skip ⇒ defer to Cargo), so new `syn` variants fail open.
- **Direct-read inventory — current and accurate.** The three renumbered anchors updated in `7c37a86da` land on real sites: `:36` = `fs::read_to_string(&canonical_source)`, `:71` = `crate_entry_path`'s manifest read, `:103` = `is_regular_file`'s `symlink_metadata`. The guardrail scans all of `crates/sifr_driver/src` and **PASSes**, so completeness is enforced, not asserted.
- **Multi-backend cache identity.** `combined_sqlx_offline_metadata_digest:63-89` folds package and workspace `.sqlx` roots for every resolved backend into one path-keyed digest, so a two-backend divergence changes the result; `complete_metadata_directory_participates_in_cache_identity` passes.
- **Exact dependency/feature lock.** `assert_exact_backend_dependency_graph` runs `cargo tree --workspace --edges features --locked --offline` and pins `axum v0.8.9`, `tower-http v0.7.0` + `set-header`, `sqlx v0.8.6` + `runtime-tokio-rustls`/`postgres`/`macros`; the fixture manifest pins `=0.8.9`/`=0.8.6`/`=0.7.0`/`=1.52.3` with `default-features = false`; the area runner re-verifies 44 aliases. The planning-only shadow crates are fully untracked — `git ls-files` on the fixture shows 10 real files and nothing under `rust/`.
- **Docs.** `internal_docs/rust_interop_architecture.md:1222-1235` no longer overclaims: it now states the two-state model explicitly ("retains Rust's separate declaration-directory and pending flat-module-relative state, so explicit paths remain anchored to the declaring file's directory while ordinary child modules use the pending module directory"), which I verified is what the code does. The round-6 documentation finding is closed. `docs/rust-interop.mdx:244-262` and `plans/phases/39_rust_interop.md:343-358` remain fixture-scoped, explicitly disclaiming arbitrary Axum/tower-http APIs, live SQLx connectivity, and product-level web-framework support.
- **Matrix / provenance / stable claims.** Area runner: 36 fixtures, 10 diagnostics, 44 crates, 61 package examples, 18 scenario examples, 229 mutation cases, 5 tiers, compat 36 rows / 36 fixture rows / 3 categories + 7 self-tests, 20 stale-draft cases, 36 stable claims + 33 self-tests. Resource gate `surfaces=1`, `future_runtime_rows=0`; `--self-test` PASS.
- **File-size / responsibility split.** 665 / 219 / 235 — matches the recorded figures exactly. Split is by compiler concern (offline policy / cfg-aware visitation / module-graph traversal), all three well under the 900 cap.

---

## Findings (severity order)

### None blocking. No correctness, coverage, documentation, or certification-blocking findings.

### N-1 — NIT · Raw-identifier module names resolve to the wrong file name (fail-open miss)
`rust_interop_sqlx_modules.rs:120,153` — `module.ident.to_string()`

syn renders a raw identifier as `r#async` (verified directly), while rustc's `default_submod_path` uses `ident.name`, i.e. `async`. So `mod r#async;` makes Sifr probe `<base>/r#async.rs` while rustc compiles `<base>/async.rs`.

Confirmed end to end on the certified fixture: `mod r#async;` in `src/bridges/backend.rs` with an unprepared query in `src/bridges/backend/async.rs` → `cargo check --offline` fails as expected, and `sifr check` surfaces the failure **only** as `= note: rustc stderr: …` rather than as the package-scoped pre-Cargo diagnostic.

This is **fail-open** (a miss, never a false rejection — a false rejection would require a file literally named `r#async.rs` to coexist), it is **pre-existing** rather than introduced by `7c37a86da`, and it matches the documented "fall through to offline Cargo as the authority" contract. Fix would be one call to `unraw()` / stripping the `r#` prefix in both the inline and flat branches; alternatively record it beside the existing symlink and function-body-`mod` limitations.

### N-2 — NIT · Architecture-doc wording is imprecise for `#[path]` nested inside an inline module
`internal_docs/rust_interop_architecture.md:1226-1227`

"explicit paths remain anchored to the declaring file's directory" holds for the common case, but for `mod inner { #[path="x.rs"] mod y; }` the base is the *inline module's* directory (`<dir>/inner`), not the file's. The preceding clause ("separate declaration-directory … state") is correct; "the declaration directory" would make the whole sentence exact.

### N-3 — NIT · Superseded validation bullets read present-tense
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1638-1643`

The round-5-era bullet still says "The implementation **remains** responsibility-split at … 200 for module-graph traversal" and an earlier bullet says "All 449 non-generated driver tests pass"; the round-6 bullet correctly records 235 and 450. Both older statements were accurate when written and are in a dated append-only log, so this is honest — but marking them historical would remove the apparent contradiction.

### N-4 — NIT · Carry-overs from round 6, all optional
R6-3 (subprocess-before-TOML-read; duplicate manifest parse), R6-4 (`rust_interop.rs` at 883/900), and the still-absent unit coverage for symlink and cycle refusal — both load-bearing, both verified only by ad-hoc probe (mine and round 5/6's).

---

## Validation results

All commands run at `HEAD = 7c37a86da` with the working tree in its as-found state.

| Check | Recorded | My result |
|---|---|---|
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | 13 pass | **13 passed, 0 failed** (0.13 s) |
| `cargo test -p sifr_driver --lib` | 450 / 65 ignored | **450 passed, 0 failed, 65 ignored** (33.28 s) |
| Mandatory negative (`.env`-armed; gated + inline-path + A/B/C file-module regressions) | pass, 29.30 s | **pass, 43.33 s** |
| Mandatory positive (Axum loopback + tower-http + SQLx offline + clean shutdown) | pass | **pass, 50.76 s** |
| Rust-interop area runner | 10/10, 229 mutations | **variants=10, failures=0, blocking=0**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5+6, compat 36/36/3 + 7, 20 stale-draft, 36 claims + 33 |
| `cargo clippy --workspace -- -D warnings` | pass | **pass** |
| `cargo fmt --check` | pass | **pass** |
| File-size guardrail | pass | **PASS** (3011 files, limit 900) |
| `sifr_driver` maintainability | pass | **PASS** |
| TypeScript-Go transfer guardrails | pass | **PASS** |
| Resource gate + `--self-test` | pass | **PASS** (`surfaces=1`, `future_runtime_rows=0`); self-test **PASS** |
| `git diff --check origin/main..7c37a86da` | pass | **clean** |
| Production split sizes | 665 / 219 / 235 | **665 / 219 / 235** |
| R6-1 layouts A/B/C vs rustc 1.94 | fixed | **11 layouts verified; all match** |
| R6-1 A/B/C, false-rejection direction, on the fixture from `backend.rs` | fixed | **`cargo check` clean AND `sifr check` clean — ✅ fixed ×3** |
| R6-1 A/B/C, active-query direction | fixed | **rejected pre-Cargo ×3 with the package-scoped message — ✅ fixed ×3** |
| Unit-test layout is real Rust | implied | **rustc compiles exactly q13/q14/q15 — pin is faithful** |
| Nested `foo/mod.rs` from non-mod-rs parent | not run | **correctly scanned** |
| Cycle refusal after the rewrite | not run | **no hang (10.2 s), query still rejected** |
| Raw-identifier module | not run | **fail-open miss confirmed (N-1)** |

Every recorded figure reproduced. The negative-test wall clock came in at 43.33 s against a recorded 29.30 s (the issue doc separately records 63.34 s for an earlier run) — ordinary cold/warm and machine variance in the same band round 5 already documented, not an overclaim. Nothing in the branch is overstated.

I did not run `scripts/run_all_tests.sh --profile create-pr`; that gate is the still-open final checklist item and belongs to the author's PR lane. Every gate it aggregates that is attributable to this delta, I ran individually above.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** `cargo tree --locked --offline` pins the exact versions and the three frozen SQLx features; shadow crates are fully untracked; 44 aliases re-verified by the area runner. |
| Hermetic `127.0.0.1:0` Axum service, real tower-http, deterministic shutdown | **Met.** Positive test observes the exact marker including `status=200;tower-http=0.7.0;middleware=response-header;shutdown=clean` and asserts empty stderr. |
| Real SQLx macro from checked-in `.sqlx` under forced `SQLX_OFFLINE`, no live DB | **Met.** Query hash bound into the runtime marker; `assert_database_sentinel_unused` passes at all three checkpoints against an armed `.env` `DATABASE_URL`; the fixture supplies no SQLx override. |
| Mandatory generated-package diagnostic: independent missing + stale, stable `SIFR-RUST-CARGO-0001`, DB/network disabled | **Met**, and strengthened again — the clean control now additionally carries never-compiled `SELECT 97/96/95` queries from A/B/C installed in the **real** `src/bridges/backend.rs`, so the round-6 regression is pinned in generated-package integration, not only in units. |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met.** All counts re-verified. The round-6 documentation inaccuracy is corrected and the "both directions" claim is now properly qualified. Wording is fixture-scoped and precise throughout (N-2/N-3 are wording polish, not inaccuracy). |
| Validator self-tests without weakening earlier rows | **Met.** 229 cases, 10/10, no prior row regressed. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked. Unlike every prior round, the remaining gap is no longer in the class that gated rounds 1–6: the `#[path]` resolver now matches rustc on every layout I could construct, in both directions, and the coverage reaches the fixture's own real file module.

---

## Required fixes

**None.**

## Optional suggestions

- **N-1** — Unraw module identifiers (or document the raw-identifier gap alongside the symlink and function-body-`mod` limitations).
- **N-2** — Say "the declaration directory" rather than "the declaring file's directory" at `rust_interop_architecture.md:1226`.
- **N-3** — Mark the superseded 449-test / 200-line validation bullets as historical.
- **N-4** — Carry-overs: try `nearest_declared_workspace_root` before spawning `cargo metadata`; thread `sqlx_crates` into `backend_may_resolve_sqlx_metadata`; add unit coverage for symlink and cycle refusal; watch `rust_interop.rs` at 883/900 before the next change lands there.
