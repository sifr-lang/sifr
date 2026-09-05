Repository files unmodified — all probes ran in `/tmp` and were removed; I rebuilt `target/debug/sifr` and test artifacts (gitignored). `plans/reviews/active/rust-interop-certification-13-review-round-8.md` was already present as an empty untracked placeholder; I did not write to it.

---

# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 8

## Verdict: **NOT SATISFIED**

Substantively this branch is done. The round-7 conclusion holds in full, and the optional raw-identifier hardening is correct, complete, and now verified against real rustc in every position I could construct. The verdict turns on **one dropped word** in the certification issue's evidence log, introduced by the very commit under review (`aa70a96d9`), which leaves a validation claim garbled and, as written, wrong. That is a documentation finding, and the stated SATISFIED bar admits none. It is a one-word repair — not a regression, not a coverage gap, not a correctness problem.

---

## Scope

Committed delta `origin/main..aa70a96d9` (9 commits, 57 files), focused on `aa70a96d9`. `HEAD` is exactly `aa70a96d9`; `git status` shows only the excluded working-tree items (`editor_integrations`, leetcode corpora submodule, `.cert5probe/`, `.agent/`, two stray webp files, `plans/phases/43_interoperability.md`, the empty round-8 placeholder). No production or test file is dirty, so everything I ran reflects the reviewed commit.

Delta since round 7's SATISFIED: two `unraw()` call sites, 24 test lines, three documentation edits. `rust_interop_sqlx_offline.rs` and `rust_interop_sqlx_cfg.rs` are untouched by `aa70a96d9`.

---

## 1. Resolver re-audit — the raw-identifier change

`rust_interop_sqlx_modules.rs:124` (inline branch) and `:158` (flat/nested branch) now call `module.ident.unraw().to_string()`. Both `#[path]` branches (`:126` inline, `:146` file) still read `directory.dir_path` directly and never `plain_base()`, so round 6's two-state model (`dir_path` + pending `relative`) is intact and untouched.

**Does `IdentExt::unraw` match rustc's filename?** rustc's `default_submod_path` formats `ident.name`, which for `r#async` is the symbol `async`; `unraw` strips the `r#` prefix. I did not accept that from structure — I built the layouts under **rustc 1.94.0** and read which files it actually compiled (dead-code warnings as the observation channel):

| Probe | Declaration | rustc compiled | Sifr model | Match |
|---|---|---|---|---|
| raw keyword, flat | `mod r#async;` in `src/outer.rs` | `src/outer/async.rs` | same | ✅ |
| raw keyword, decoy present | `src/outer/r#async.rs` also on disk | **never compiled** | never probed | ✅ |
| raw keyword, inline | `mod r#type { mod child; }` | `src/outer/type/child.rs` | same | ✅ |
| raw keyword, inline decoy | `src/outer/r#type/child.rs` | **never compiled** | never probed | ✅ |
| raw **non**-keyword, flat | `mod r#foo;` | `src/foo.rs` | same | ✅ |
| raw keyword, **nested** `mod.rs` | `mod r#match;` → `src/match/mod.rs`, child `mod deep;` | `src/match/deep.rs` | same | ✅ |

The nested-`mod.rs` raw form matters because `unraw()` also feeds `base.join(&module_name)` for the child `dir_path` at `:173`; rustc uses the unrawed directory name there too, so it agrees.

**Fail-open direction preserved.** If only a literal `r#async.rs` exists and `async.rs` does not, both `flat` and `nested` miss, `resolve_declared_module` returns `None`, and the module is skipped — Cargo remains the authority. rustc errors on such a crate anyway. No path exists by which the decoy can produce a false rejection: the resolver never constructs an `r#`-prefixed candidate.

**End-to-end on the certified fixture** (`/tmp` copy, path dep repointed absolutely; baseline `sifr check src/main.sifr` → `no errors found`, 26.0 s), with declarations installed in the real non-`mod.rs` file module `src/bridges/backend.rs`:

- **Flat raw, active:** `mod r#async;` + unprepared query in `src/bridges/backend/async.rs` → `cargo check --offline --lib` fails, and `sifr check` now reports `error[SIFR-RUST-CARGO-0001] … no cached data for this query: SELECT 771::INT4 AS value` with the package-scoped note. Round 7's N-1 miss is **closed at the integration level**, not just in units — the failure is caught pre-Cargo rather than leaking as `rustc stderr`.
- **Flat raw, decoy:** valid `async.rs`, unprepared query moved into `src/bridges/backend/r#async.rs` → `cargo check --offline` **clean**, `sifr check` → **`no errors found`**. No false rejection.
- **Inline raw, both directions at once:** `mod r#type { mod child; }` with `SELECT 773` in `type/child.rs` and `SELECT 774` in `r#type/child.rs` → rustc compiles and fails on 773 only; `sifr check` reports 773 and `grep -c "SELECT 774"` returns **0**. Exact agreement with rustc in both directions simultaneously.

Ordinary and `#[path]` resolution are unregressed: 13 focused SQLx tests and 450 driver tests pass, and the fixture baseline is clean.

**Panic safety.** `grep -nE '\.unwrap\(\)|\.expect\(|panic!|unreachable!|todo!|unsafe|\[[0-9]+\]'` across the three new production files returns exactly one hit — `segments[0]` at `rust_interop_sqlx_offline.rs:501`, guarded by `segments.len() >= 2` in the same `&&`. `unraw()` is infallible and allocation-only.

---

## 2. Independent verification of `explicit_paths_from_file_modules_follow_rust_directory_rules`

The test declares `mod outer;` from `src/lib.rs`, so `src/outer.rs` is a genuine non-`mod.rs` file module — the file kind round 6 required. I replicated its layout **verbatim** under rustc 1.94 (all eleven files, both decoy directories, `--crate-type lib --edition 2021`). rustc emitted dead-code warnings for exactly:

```
src/direct.rs (q13)  src/alt_inline/child.rs (q14)  src/child.rs (q15)
src/outer/async.rs (q16)  src/outer/type/child.rs (q17)
```

and for **none** of `src/outer/direct.rs` (90), `src/outer/alt_inline/child.rs` (91), `src/loaded/child.rs` (92), `src/outer/r#async.rs` (93), `src/outer/r#type/child.rs` (94).

So the test's `assert_eq!` on the exact vector `["SELECT 13".."SELECT 17"]` pins precisely rustc's semantics: 16 and 17 are the compiled raw-identifier sources, 93 and 94 are literal `r#`-named decoys rustc never opens. Because the assertion is exact vector equality (with `sort`/`dedup` at `rust_interop_sqlx_offline.rs:371-372` making order deterministic), the decoys are pinned as *absent*, not merely unchecked. The follow-up `validate_sqlx_offline_metadata(&fixture.0) == Ok(())` after writing metadata for exactly those five confirms the positive direction too. The pin is faithful, not a fantasy layout.

Coverage nit only: the raw **nested-`mod.rs`** form (`mod r#match;` → `match/mod.rs`) is not unit-pinned, though I verified it against rustc by hand.

---

## 3. Reconfirmation of everything else

- **Mandatory backend tests — both pass** (71.55 s combined). Negative independently rejects missing and stale metadata with `SIFR-RUST-CARGO-0001` plus its own detail string, asserts `SQLX_OFFLINE=true` in the rendered diagnostic, and its clean control — carrying the A/B/C never-compiled `SELECT 97/96/95` siblings in the real `src/bridges/backend.rs` — still reaches Cargo with `errors.is_empty()`. Positive observes the exact loopback marker. Both are untouched by `aa70a96d9`.
- **Cache / workspace behavior.** Unchanged since round 7 (`rust_interop_sqlx_offline.rs` not in this commit's diff); subprocess-outside-mutex, bounded per-backend memo, ancestor-manifest fingerprint, and the multi-backend `.sqlx` combined digest all re-exercised by the passing focused and driver suites.
- **Inventory anchors — exact and complete.** The three `rust_interop_sqlx_modules.rs` anchors were renumbered `36/71/103 → 37/72/104` to absorb the new `use syn::ext::IdentExt;` line. They land on the checker's actual `DIRECT_FS_PATTERN` hits: `:37` `fs::read_to_string(&canonical_source)`, `:72` the manifest read, `:104` the `.is_file()` call inside `is_regular_file` (round 7 described `:103` as the `symlink_metadata` line; the pattern matches `.is_file()`, one line lower — the anchor is right, that description was off by one). The guardrail scans all of `crates/sifr_driver/src` and **PASSes**, so completeness is enforced rather than asserted.
- **Docs.** `internal_docs/rust_interop_architecture.md:1227` now reads "anchored to the **declaration directory**" — round 7's N-2 is closed, and the wording is now exact for `#[path]` nested inside an inline module. `docs/rust-interop.mdx:244-262` and `plans/phases/39_rust_interop.md:343-358` remain fixture-scoped, explicitly disclaiming arbitrary Axum/tower-http APIs, live SQLx connectivity, and product-level web-framework support.
- **Issue chronology.** Round 6 and round 7 are both logged in order, round 7's `SATISFIED` is recorded accurately, and the raw-identifier bullet's claims all check out: module names are unrawed before lookup; both the file and inline forms prove the compiled source is recognized while a literal `r#name` decoy is ignored; the split is 665/219/240. Two superseded round-5-era bullets were converted to past tense (round 7's N-3, addressed) — and this is where the defect below was introduced.
- **Matrix / provenance / stable claims.** 36 rows, 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`, 72 evidence directions, 0 planned; the `ecosystem_backend_certification` fixture binds positive and negative to the two *distinct* mandatory driver tests I ran; 36 stable claims; `future_runtime_rows=0`.
- **Responsibility split.** `wc -l` = **665 / 219 / 240** — matches the recorded figure exactly. Split is by compiler concern (offline policy / cfg-aware visitation / module-graph traversal); all three well under 900.
- **Checklist honesty.** Six items `[x]`, all independently substantiated above; the final gates/review/merge item correctly `[ ]`.

---

## Findings (severity order)

### R8-1 — LOW · Documentation · blocking under the stated bar
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1515`

`aa70a96d9` deleted the continuation line `denied.` from the create-PR-lane evidence bullet. The bullet now ends mid-sentence:

> …transcript replay, and all self-tests. Workspace Clippy passes with warnings

The intended claim was "passes with warnings **denied**" — every other occurrence in the doc writes it as `-D warnings` (lines 1181, 1781). As committed, the sentence is both truncated and semantically inverted: it reads as though Clippy passes *while emitting* warnings, which understates the gate. I verified the real behavior: `cargo clippy --workspace -- -D warnings` **passes**.

The diff hunk shows it was collateral to the neighbouring tense edit, not an intentional rewording:

```
   transcript replay, and all self-tests. Workspace Clippy passes with warnings
-  denied.
 - [agent round 1](…)
```

Impact is confined to the certification record — no code, gate, or claim about capability is affected. It is nonetheless an inaccurate sentence in the milestone's own evidence log, introduced by the commit under review, in a checklist item marked complete. Fix: restore `denied.`

### N-1 — NIT · Raw nested-`mod.rs` form is not unit-covered
The test pins the flat file form and the inline form. `mod r#match;` → `match/mod.rs`, which also routes through `unraw()` at `:173` for the child `dir_path`, is verified only by my ad-hoc rustc probe. One more decoy pair in the existing test would close it.

### N-2 — NIT · Carry-overs, all optional and unchanged
R6-3 (`resolve_cargo_workspace_root` still spawns `cargo metadata` before trying `nearest_declared_workspace_root`; `backend_may_resolve_sqlx_metadata` still re-parses `Cargo.toml`), R6-4 (`rust_interop.rs` at **883/900**), and the still-absent unit coverage for symlink refusal and cycle refusal — both load-bearing, both verified only by probe across rounds 5–7.

### N-3 — NIT · Not attributable
`cargo clippy --workspace --all-targets -- -D warnings` fails workspace-wide (269 `expect()` on `Result` in tests, 24 `usize→u32` casts, across `sifr_codegen`, `sifr_package`, `sifr_lowering`, `sifr_ipc`, `sifr_lint`, `sifr_stdlib_manifest`, and driver test files). This is pre-existing and repo-wide; the documented gate in `AGENTS.md` is `cargo clippy --workspace -- -D warnings`, which passes. Noted only so the distinction is on record — no action for this milestone.

---

## Validation results

All at `HEAD = aa70a96d9`, working tree as found.

| Check | Recorded | My result |
|---|---|---|
| Focused SQLx tests (`build::rust_interop_sqlx_offline`) | 13 | **13 passed, 0 failed** (0.14 s) |
| `cargo test -p sifr_driver --lib` | 450 / 65 ignored | **450 passed, 0 failed, 65 ignored** (33.41 s) |
| Mandatory negative (`.env`-armed; missing + stale + A/B/C control) | pass | **pass** |
| Mandatory positive (Axum loopback + tower-http + SQLx offline + clean shutdown) | pass | **pass** (both, 71.55 s combined) |
| Rust-interop area runner | 10/10, 229 mutations | **variants=10, failures=0, blocking=0**; fixtures=36, diagnostics=10, crates=44, package_examples=61, scenario_examples=18, **229 mutation cases**, tiers=5+6, compat 36/36/3 + 7, 20 stale-draft, 36 claims + 33 |
| `cargo clippy --workspace -- -D warnings` | pass | **pass** |
| `cargo fmt --check` | pass | **pass** |
| File-size guardrail | pass | **PASS** (3011 files, limit 900) |
| `sifr_driver` maintainability | pass | **PASS** |
| TypeScript-Go transfer guardrails | pass | **PASS** |
| Resource gate + `--self-test` | pass | **PASS** (`surfaces=1`, `future_runtime_rows=0`); self-test **PASS** |
| `git diff --check origin/main..aa70a96d9` | pass | **clean** |
| Production split | 665 / 219 / 240 | **665 / 219 / 240** |
| `unraw` vs rustc filenames (6 layouts, incl. non-keyword + nested `mod.rs`) | claimed | **all 6 match; decoys never compiled** |
| Unit-test layout is real Rust (q13–q17 / q90–q94) | implied | **rustc compiles exactly q13–q17 — pin is faithful** |
| Raw ident, active direction, on the fixture from `backend.rs` | fixed | **rejected pre-Cargo with package-scoped message — N-1 closed** |
| Raw ident, decoy direction | fixed | **`cargo check` clean AND `sifr check` clean — no false rejection** |
| Inline raw module, both directions at once | not run | **773 reported, 774 never reported — exact rustc agreement** |
| Compat categories / evidence directions | 21/14/1, 72/0 | **21/14/1, 72 directions, 0 planned** |

Every recorded figure reproduced. Nothing in the branch is overstated. I did not run `scripts/run_all_tests.sh --profile create-pr` — that aggregate is the still-open final checklist item and belongs to the author's PR lane; every gate it aggregates that is attributable to this delta, I ran individually.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** `cargo tree --locked --offline` pins the exact versions and three frozen SQLx features; 44 aliases re-verified; shadow crates fully untracked. |
| Hermetic `127.0.0.1:0` Axum service, real tower-http, deterministic shutdown | **Met.** Positive test observes the exact marker and asserts empty stderr. |
| Real SQLx macro from checked-in `.sqlx` under forced `SQLX_OFFLINE`, no live DB | **Met.** Query hash bound into the runtime marker; sentinel unused at all three checkpoints against an armed `.env` `DATABASE_URL`; fixture supplies no SQLx override. |
| Mandatory generated-package diagnostic: independent missing + stale, stable code, DB/network disabled | **Met.** Independent `remove_file` and `replacen` paths, each with its own detail string; clean control carries the A/B/C never-compiled siblings in the real `src/bridges/backend.rs`. |
| Bind distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met with one editorial defect.** All counts and bindings re-verified; capability wording is fixture-scoped and precise, and the architecture doc's `#[path]` sentence is now exact. The truncated Clippy evidence sentence (**R8-1**) is the sole flaw. |
| Validator self-tests without weakening earlier rows | **Met.** 229 cases, 10/10, no prior row regressed. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist is honestly marked. For the first time across eight rounds there is **no remaining gap in the resolver's class**: `#[path]` and raw identifiers match rustc in every layout I could construct, in both directions, including inside the fixture's own real file module.

---

## Required fixes

1. **R8-1** — Restore `denied.` at `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1515` so the bullet reads "Workspace Clippy passes with warnings denied."

## Optional suggestions

- **N-1** — Add a `mod r#match;` → `match/mod.rs` decoy pair to `explicit_paths_from_file_modules_follow_rust_directory_rules`.
- **N-2** — Carry-overs: try `nearest_declared_workspace_root` before spawning `cargo metadata`; thread `sqlx_crates` into `backend_may_resolve_sqlx_metadata`; add unit coverage for symlink and cycle refusal; watch `rust_interop.rs` at 883/900.

Once R8-1 lands, this milestone is certifiable — a re-verify of a single-word documentation change is all round 9 would need.
