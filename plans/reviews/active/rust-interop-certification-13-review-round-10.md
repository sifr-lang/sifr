# Published-Head Merge-Readiness Review — Rust Interop Runtime Ecosystem Certification 13 (PR #3078)

## Verdict: **SATISFIED**

`HEAD = f8ab7080cbec82f651476801e989c66449c6c939`, matching `gh pr view 3078 --json headRefOid` exactly. Base `main`, draft, 59 files. `git status` shows only the excluded working-tree items — no production or test file is dirty, so every gate below reflects the reviewed commit.

---

## Scope

Full published delta `origin/main..f8ab7080c` — 12 commits, 59 files, +6471/−147. Excluded and not attributed: `editor_integrations`, the leetcode corpora submodule, `.cert5probe/`, `.agent/`, the two stray webp files, `plans/phases/43_interoperability.md`.

Delta since round 9 (`49020688d..f8ab7080c`): **2 Markdown files, +76/−0**; `git diff --name-only | grep -v '\.md$'` returns **0 files**. I therefore re-audited the whole implementation rather than only the last docs commit.

---

## 1. Resolver semantics — independently established against rustc, not accepted structurally

I did not take rounds 7–8's rustc claims on trust. Built both layouts under **rustc 1.94.0 (4a4ef493e)** and read which files it actually compiled.

**`#[path]` from a genuine non-`mod.rs` file module** (`mod outer;` → `src/outer.rs`, pending `relative = Some("outer")`):

| Declaration in `src/outer.rs` | rustc compiled | Never compiled | Sifr model |
|---|---|---|---|
| `#[path = "direct.rs"] mod direct;` | `src/direct.rs` | `src/outer/direct.rs` | `dir_path.join(path)` ✅ |
| `#[path = "alt_inline"] mod m { mod child; }` | `src/alt_inline/child.rs` | `src/outer/alt_inline/child.rs` | `dir_path.join(path)`, skips `relative` ✅ |
| `#[path = "loaded.rs"] mod l;` then `mod child;` | `src/child.rs` | `src/loaded/child.rs` | `source_path.parent()`, `relative: None` ✅ |

This is exactly rustc's "all `#[path]` files are treated as though they are a `mod.rs` file" rule, and confirms `ModuleDirectory { dir_path, relative }` (`rust_interop_sqlx_modules.rs:57-69, 120-127, 144-156`) reproduces the two-state model in both the inline and file branches. Round 6's blocker class is genuinely closed.

**Raw identifiers** (`rust_interop_sqlx_modules.rs:124, 158, 161`):

| Declaration | rustc compiled | Decoy on disk | Result |
|---|---|---|---|
| `mod r#async;` | `src/outer/async.rs` | `src/outer/r#async.rs` **never compiled** | `unraw()` ✅ |
| `mod r#match;` → `match/mod.rs`, child `mod deep;` | `src/outer/match/deep.rs` | — | unrawed `dir_path` ✅ |

The nested-`mod.rs` raw form — round 8's N-1 coverage nit — is behaviorally correct; only the unit pin is absent.

**Fail-open direction is preserved and documented.** `regular_source_path` refuses symlinks and package-escaping canonical paths; `visit_item_mod` is a no-op so fn-body inline modules are not scanned; `module_declaration_may_vary` conservatively skips both `cfg` and `cfg_attr` module declarations. Every one of these produces a *miss* (Cargo remains the authority), never a false rejection — and `internal_docs/rust_interop_architecture.md:1229-1238` states each boundary explicitly, including symlinks, package escape, and fn-body declarations. The architecture doc matches the code.

**cfg/cfg_attr.** `rust_interop_sqlx_cfg.rs` overrides 13 visit hooks (item, stmt, expr, arm, field, field-value, variant, generic-param, fn-arg, impl/trait/foreign item) and returns `true` (skip) for unknown `syn` non-exhaustive variants — fail-open on future syntax. `cfg_attr_arguments_may_disable` parses the argument list and skips the predicate, only inspecting whether a nested `cfg`/`cfg_attr` may be added; an unparseable list is treated as "may disable". Correct direction throughout.

---

## 2. Dependency/feature lock, real service, offline macro

- **Exact lock.** `assert_exact_backend_dependency_graph` runs `cargo tree --workspace --edges features --locked --offline` and asserts `axum v0.8.9`, `tower-http v0.7.0` + feature `set-header`, `sqlx v0.8.6` + `runtime-tokio-rustls`/`postgres`/`macros`. `.cargo/config.toml` adds `[net] offline = true`. The planning-only shadow crates (`rust/axum`, `rust/sqlx`, `rust/tower_http`) are deleted from git; only empty directories remain on disk.
- **Real Axum/tower-http loopback and deterministic shutdown** (`src/bridges/backend.rs:53-103`): `TcpListener::bind("127.0.0.1:0")`, real `SetResponseHeaderLayer::if_not_present`, raw HTTP exchange, `oneshot` + `with_graceful_shutdown`, then the join handle is awaited under `timeout(Duration::from_secs(2), server)` and both nested results are checked. Shutdown is observed, not assumed. No `unwrap`/`expect`/`panic!` anywhere in the bridge.
- **Real SQLx macro, no DB.** `sqlx::query!("SELECT 13::INT4 AS value")` is expanded at compile time and only `.sql()` is read — the query is never executed. I verified `sha256("SELECT 13::INT4 AS value") = f2d6fe08…f508`, matching both the `.sqlx` filename and the file's `hash` field, and real sqlx 0.8.6 accepted it in the passing positive build — so the hashing convention is empirically validated by sqlx itself, not merely asserted.
- **Offline forcing is Sifr's, and load-bearing.** `configure_hermetic_build_environment` sets `SQLX_OFFLINE=true` and `env_remove("DATABASE_URL")` on the probe cargo (`rust_interop_probe.rs:146`), the final build (`materialize.rs:283`), and `cargo metadata` (`rust_interop_sqlx_offline.rs:323-324`). The scenario validator *rejects* a fixture-supplied `[env]` in `.cargo/config.toml`, so the fixture cannot silently take over. The negative test arms a real bound `.env` `DATABASE_URL` sentinel and asserts non-connection at all three checkpoints.

---

## 3. Diagnostics, cache identity, multi-backend

- **Independent missing vs stale.** `remove_file` and a `replacen` of the SQL text are separate mutations, each asserted against its own detail string (`there is no cached data for this query` / `saved SQLx query text does not match query identity`), both under stable `SIFR-RUST-CARGO-0001`, with `SQLX_OFFLINE=true` proven present in the rendered missing-metadata diagnostic. Rejection happens in `validate_probe_sqlx_offline_metadata` **before** the probe temp root is created (`rust_interop_probe.rs:88`) — pre-Cargo, not stderr scraping. The clean control still reaches Cargo with `errors.is_empty()` while carrying the never-compiled `SELECT 95/96/97/98/99` siblings.
- **Complete cache identity.** `combined_sqlx_offline_metadata_digest` digests the whole `.sqlx` directory (`digest_path`), keyed by path, for package **and** workspace roots; it feeds `probe_cache_key` (`rust_interop_probe_cache.rs:76-79`) and `RustInteropCargoInputs.sqlx_offline_metadata_digest`, which is serialized into the plan cache string (`rust_interop_plan.rs:590-597`) and combined for dual-package builds (`rust_interop_cargo_inputs.rs:77-80`, with an `assert_ne!` test in both directions).
- **Multi-backend identity.** `rust_interop.rs:236-245` unions the package root with every pending direct probe's manifest parent into a `BTreeSet` before digesting — deterministic ordering, all resolved backends covered.
- **Workspace resolution.** `cargo metadata --no-deps --offline` is spawned outside the memo mutex; the memo is keyed by an ancestor-manifest content fingerprint and retains one entry per backend, so a long-lived driver invalidates on workspace change without unbounded growth. `nearest_declared_workspace_root` is the fallback on failure or missing field.
- **Panic safety.** `grep -nE '\.unwrap\(\)|\.expect\(|panic!|unreachable!|todo!|unsafe|\[[0-9]+\]'` over all three new production files returns exactly one hit: `segments[0]` at `rust_interop_sqlx_offline.rs:501`, short-circuit-guarded by `segments.len() >= 2` in the same `&&`. `unraw()` is infallible.

---

## 4. Matrix, provenance, claims, inventory, docs

Every claimed figure reproduced by direct computation and by re-running each checker:

| Claim | Verified |
|---|---|
| 36 fixture rows / 36 compat rows / 36 schema-v2 manifests | ✅ `fixtures=36`, `rows=36 fixture_rows=36` |
| 72 passing, 0 planned evidence directions | ✅ `{'passing': 72}`, no other status |
| 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design` | ✅ exact |
| Execution kinds 13/4/10/9 | ✅ `cargo-probe 13, compiler-diagnostic 4, contract-only 10, runtime-observed 9` |
| 44 crate aliases, 61 package examples, 18 scenario examples | ✅ `crates=44 package_examples=61 scenario_examples=18` |
| 36 structured stable claims | ✅ `claims=36` |
| Only `ecosystem_backend_certification` promoted | ✅ it was the sole `future-owned` row on `origin/main`; 0 remain |
| Inventory anchors `rust_interop_sqlx_modules.rs:37/72/104` | ✅ land on the `read_to_string(&canonical_source)`, manifest read, and `.is_file()` lines; guardrail scans all of `crates/sifr_driver/src` and PASSes |
| Production split 665 / 219 / 240 | ✅ exact; all under 900 |

**Gate relaxations are legitimate, not weakenings.** Two guards were relaxed because this milestone promotes the last future-owned row: the resource gate's "at least one future-owned row must remain" backstop (whose own message said *"update this guard when resource certification lands"*) and `VALID_CATEGORIES`' requirement that `future-owned-by-separate-phase` be in use. In both cases the load-bearing per-row guard survives — `_validate` still fails any surface certification row that is neither an allowlisted core row with passing positive+negative evidence nor future-owned by the certification issue — and both were replaced with *positive* self-tests asserting a completed matrix is accepted while a still-required active category being unused is still rejected. The PR body discloses this as "accept the completed zero-future-row transition".

**Scenario policy is real, not decorative.** `_scenario_backend.py` pins the exact workspace and root dependency tables, frozen features, trust lists, `[net] offline`, absence of fixture `[env]`, the sole `.sqlx` filename plus SQL/hash/PostgreSQL description, and ten load-bearing `backend.rs` tokens — then proves each with **19 mutations plus a baseline**, every one asserted to be caught.

**Docs are fixture-scoped and precise.** `docs/rust-interop.mdx:244-262`, `internal_docs/rust_interop_architecture.md:1262-1288`, `plans/phases/39_rust_interop.md:343-361`, and both READMEs each explicitly disclaim arbitrary Axum/tower-http APIs, live SQLx connectivity, and product-level web-framework support. The stable-claims table row and error-code links check clean.

---

## 5. Validation evidence

### Recorded authoritative gate — reproduced exactly

`target/validation_lane_reports/create-pr.latest.json` at the warm rerun:

| Step | Recorded in issue/PR | Report | Budget | Status |
|---|---|---|---|---|
| `python_interop` | 527.06 / 600 s, 19 variants | 527058 ms | 600000 blocking | pass |
| `rust_interop_checks` | 7.62 / 10 s, 10/10 | 7616 ms | 10000 blocking | pass |
| `developer_tooling_checks` | 132.07 / 180 s, 18 variants | 132066 ms | 180000 blocking | pass |
| `crate_tests` | 144.83 / 600 s, driver 450/65 | 144832 ms | 600000 blocking | pass |
| `runtime_platform_suites` | 69.87 / 120 s, 28 variants | 69873 ms | 120000 blocking | pass |
| `e2e_pass_suite` | 131/131, 399.17 / 600 s | 399166 ms | 600000 blocking | pass |

All **24** lane steps `status=pass`, all budget statuses `pass`. Log confirms the variant counts (`variants=19`, `variants=10`, `variants=18`, `variants=28`, `131 passed, 0 failed`, `450 passed; 0 failed; 65 ignored`). `advisories` contains exactly one entry — `"warm wall-time budget exceeded"` — with `within_warm_budget: false`; no blocking advisory. Wall clock 1423.27 s. Every recorded figure is exact.

The report is timestamped 09:25, between `059ea83d9` (08:45) and `f8ab7080c` (09:26); the only difference from the reviewed head is the 14-line issue-doc append, which is inert to every gate.

### My reproduction at `f8ab7080c`

| Check | Result |
|---|---|
| `cargo test -p sifr_driver --lib` | **450 passed, 0 failed, 65 ignored** (37.93 s) |
| Focused `build::rust_interop_sqlx_offline` | **13 passed, 0 failed** (0.12 s) |
| Mandatory positive (Axum loopback + tower-http + SQLx offline + clean shutdown) | **pass** (79.52 s) |
| Mandatory negative (armed `.env`; missing + stale + clean control) | **pass** (30.13 s) |
| Rust-interop area (all 5 checks + all 5 self-tests) | **PASS** — 36/10/44/61/18; tiers 5/36 + 6; compat 36/36/3 + 7; stale-drafts + 20; claims 36 + 33 = **229 mutation cases** |
| `cargo clippy --workspace -- -D warnings` | **PASS** |
| `cargo fmt --check` | **PASS** |
| File-size guardrail | **PASS** (3011 files, limit 900) |
| `sifr_driver` maintainability guardrail | **PASS** |
| HIR maintainability guardrail | **PASS** |
| Resource certification gate + `--self-test` | **PASS** (`surfaces=1`, `future_runtime_rows=0`); self-test PASS |
| Docs error-code link guardrail | **PASS** |
| `git diff --check origin/main..f8ab7080c` | **clean** |
| rustc 1.94 `#[path]` probe (3 layouts, both directions) | **exact agreement** |
| rustc 1.94 raw-identifier probe (flat + nested `mod.rs` + decoy) | **exact agreement** |
| Query hash `sha256(query)` == filename == `hash` field | **confirmed** |

The two mandatory tests are `#[ignore]`d and bound in `fixture.json` to `profile: merge`, `suite_id: sifr_driver_generated_builds`, whose command carries `--ignored`; that suite is `modes: ["full"]`, so the create-pr lane legitimately does not execute it. The provenance binding is honest about this, and `_provenance_checks` enforces the `--ignored`/`#[ignore]` correspondence and weakest-executing-profile. I executed both directly at this head.

---

## 6. Chronology, checklist, PR scope

- **Chronology accurate.** The round-9 bullet's five claims all check out against the artifact; the `denied.` restoration is byte-identical to the pre-regression text (round 8's R8-1 is closed). The cert-12 closeout bullet's factual claim is verifiable: `ea119724e325b3900ccca81db766114d76eb4efd` exists, is an ancestor of `origin/main`, and is the `#3076` merge commit. Verdict lines across nine artifacts read NOT SATISFIED ×6, SATISFIED (7), NOT SATISFIED (8), SATISFIED (9) — logged accurately.
- **Checklist honest.** Six `certification_13` items `[x]`, each independently substantiated above; the final "focused and authoritative local gates, agent review rounds, merge, unblock only `certification_14`" item correctly still `[ ]`. That item is the author's PR-lane step and is precisely what this round unblocks.
- **PR metadata.** Title "Certify the Rust backend ecosystem bridge" matches the delta. The body's Summary, Root cause, and Validation sections are accurate against everything I measured, and it discloses both the cold-run overrun and the nonblocking warm advisory rather than eliding them.
- **Unrelated changes preserved.** All excluded working-tree items remain untracked/unstaged and outside the delta; nothing stray entered any commit.

---

## Findings

**None.** No correctness, coverage, documentation, PR-scope, or certification-blocking finding.

## Optional nits (not blocking)

- **N-1 (carry-over).** The raw nested-`mod.rs` form (`mod r#match;` → `match/mod.rs`) is verified only by rustc probe, not unit-pinned. One more decoy pair in `explicit_paths_from_file_modules_follow_rust_directory_rules` would close it.
- **N-2 — cold-run evidence not retained.** The first cold `create-pr` report was overwritten by the warm rerun, so the PR body's "899 s / 600 s Python step" and the issue's "completed all 19 Python-interop variants" cannot be reproduced from surviving artifacts. The only surviving cold temp log (`lane.create-pr.log.rn9jm532`, 02:05) is a *different*, aborted attempt that terminates mid-area with a `readonly-check-doctor` subprocess timeout, so it neither confirms nor contradicts the recorded sentence. The authoritative claim — the warm rerun's exit-0 result and every figure in it — is fully verified. Consider preserving superseded lane reports under distinct names.
- **N-3 (carry-over).** `resolve_cargo_workspace_root` still spawns `cargo metadata` before trying `nearest_declared_workspace_root`; `backend_may_resolve_sqlx_metadata` re-parses `Cargo.toml`; `rust_interop.rs` sits at 883/900; symlink and cycle refusal remain probe-verified rather than unit-covered.
- **N-4 — cosmetic.** The two new self-test assertions in `check_compatibility_matrix.py:401-420` are indented inside the per-case loop, so these two static conditions re-evaluate once per case while being counted as `+2` in `cases={len(cases) + 4}`. No behavioral effect.
- **N-5 — scope note.** `plans/reviews/active/rust-interop-certification-12-review-round-5.md` (a prior-milestone artifact) and its chronology link land in this PR via `b231daf81`; neither exists on `origin/main`, so this completes rather than contradicts the record, but the PR body doesn't mention it.
- **N-6 (carry-over, not attributable).** `cargo clippy --workspace --all-targets -- -D warnings` fails repo-wide on pre-existing test-code lints. The documented gate in `AGENTS.md` is `cargo clippy --workspace -- -D warnings`, which passes.

## Merge readiness

Ready to merge once the author takes the PR out of draft and marks the final checklist item. Every acceptance criterion is met and independently reproduced; the sole remaining open item is the author's own gates/review/merge step, which this round closes out. If the merge lane is preferred as the landing gate, note that it — not create-pr — is the profile that executes the two mandatory backend tests; I ran both directly and both pass at this exact head.

## Verdict

**SATISFIED**
