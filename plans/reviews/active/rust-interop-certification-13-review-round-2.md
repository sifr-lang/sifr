# Milestone Review — Rust Interop Runtime Ecosystem Certification 13, Round 2

## Verdict: **NOT SATISFIED**

Round 1's substantive code defects are genuinely fixed. F1 (trailing comma / `+`-concatenation false positives), F2's import- and file-macro blindness, F4's substring manifest gate, F5's traversal, F8, and F9 are all resolved, and the new preflight is now a careful, conservative, fall-through-on-unknown recognizer that I checked line-by-line against sqlx 0.8.6's real grammar. Two things block publication: the F6 remediation **does not work** — the armed sentinel cannot reach any compiler-spawned Cargo process, so the newly published "Sifr's forcing is load-bearing" claim is false and I proved it empirically — and the metadata-location model introduces a **new false rejection of a valid, documented sqlx layout**, which is the same class round 1 blocked on.

---

## Scope reviewed

Full committed delta `origin/main..HEAD` (`b231daf81`, `7a27b7896`, `6ec0742b6`), 46 files, with focus on `6ec0742b6`. The unstaged worktree paths (`editor_integrations`, leetcode corpora, `.cert5probe/`, `.agent/`, two stray webp files, `plans/phases/43_interoperability.md`) were excluded and are not attributed to this milestone. I made no repository modifications; all probes ran in `/tmp` scratch trees and were cleaned up. The untracked `rust/{axum,sqlx,tower_http}/` residue and `target/`+`.DS_Store` under the fixture are empty/ignored working-tree leftovers, absent from the commit.

### Independently reproduced

| Check | Result |
|---|---|
| `cargo test -p sifr_driver --lib` | **444 passed, 0 failed, 65 ignored** — matches recorded evidence |
| Both mandatory generated-package tests (`--ignored --exact`) | **2 passed in 82.58s** |
| `verification/areas/rust_interop/runner.py` | **10/10 variants, 0 failures**; 36 rows, 36 fixture_rows, 3 categories, **228 mutation cases**, 36 stable claims, 33 claim self-tests, 20 stale-draft cases, 7 matrix self-test cases |
| `cargo clippy --workspace -- -D warnings` | pass |
| `cargo fmt --check` | pass |
| `check_file_size_guardrails.py` | pass (3008 files, limit 900) |
| `check_sifr_driver_maintainability_guardrails.py` | pass |
| `check_sysroot_stdlib_resource_certification_gate.py` (+`--self-test`) | pass; `future_runtime_rows=0` |
| Empirical sqlx-0.8.6 behavior probes (out-of-repo copies of the fixture) | reproduced **R1**, **R2** — see below |

---

## Round-1 finding re-audit

**F1 — RESOLVED.** `parse_inline_query` (`rust_interop_sqlx_offline.rs:390-397`) now uses `Punctuated::<LitStr, Token![+]>::parse_separated_nonempty` — byte-identical to sqlx's own source parsing (`sqlx-macros-core-0.8.6/src/query/input.rs:54-58`). `parse_bind_arguments` (`:399-409`) returns early on empty input, on a bare trailing comma, and otherwise uses `parse_terminated`. Critically, every `syn::parse2` is now `.ok()` (`:300-317`), so unrecognized macro input yields `None` and falls through to Cargo rather than aborting. `syn::parse_file` failures are likewise skipped (`:120-122`). Both directions are covered by `supported_macro_forms_include_aliases_files_concatenation_and_trailing_commas` and `syntax_outside_preflight_understanding_falls_through_to_cargo` (`:496-550`).

**F2 — RESOLVED for import/alias/macro-family surface; residual gaps R2/R3/R4.** I verified recognition of: fully-qualified `sqlx::…!` and `::sqlx::…!` (leading colon lives on `path.leading_colon`, not `segments`); Cargo aliases via `package = "sqlx"` (`:98-112`); `use sqlx as db` (`:215-217`); `use sqlx::{self as db, …}` (`:230-232`); `use sqlx::query` / `use sqlx::query as q` / grouped / globbed (`:225-250`); `extern crate sqlx as db` (`:194-198`); and all six inline plus all six `query_file*` families (`:15-30`, `:296-320`). File-query path resolution matches sqlx exactly: sqlx joins `CARGO_MANIFEST_DIR` (`sqlx-macros-core/src/common.rs:5-36`) and hashes untrimmed file contents; the preflight joins `backend_root` (`:333`) — the same directory, since `probe_planning.rs:89` sets `cargo_manifest_path = package_root/Cargo.toml`.

Function-local `use` statements, `mod` items nested inside function bodies, and `sqlx::query!` appearing inside another macro's token tree are all invisible to the recognizer (aliases are collected per-module-item-list at `:164`, `visit_item_mod` is a no-op at `:278`, and `syn::visit` does not descend into `Macro::tokens`). Each of these fails **open**, which is the correct direction.

On the structural question you asked: **yes, complete `.sqlx` digest participation genuinely protects the forms the preflight cannot resolve — but only at the final-build layer, and only for the entrypoint-package-root layout.** I traced `sqlx_offline_metadata_digest` → `cargo_metadata_digest` (`rust_interop_cargo_inputs.rs:53`) → `cache_key_fragment` (`sifr_codegen/src/rust_interop_plan.rs:586-588`) → `binary_project_cache_key` (`materialize.rs:447`) → `prepare_cached_artifact`. A metadata change therefore misses the artifact cache and materializes a brand-new project root with a fresh `target/`, forcing full macro re-expansion. That is real, load-bearing protection. The probe layer is weaker by construction: the probe cache key changes (`rust_interop_probe_cache.rs:76-79`), but the probe re-runs against the shared `artifact_cache_root()/rust_bridge_probe_target` (`rust_interop_probe_paths.rs:16`), where Cargo will not recompile the bridge crate whose sources did not change. So `sifr check` catches query-text drift via the preflight only; `describe`-only drift is caught by the final build. The docs don't overclaim probe-time `describe` rejection, so this is a boundary worth noting, not a defect.

**F3 — RESOLVED for the certified layout**, structurally, via the path above. `digest_path` (`rust_interop_digest.rs:4-14`) hashes relative name + full bytes of every file, and `metadata_root.is_dir().then(...)` (`:60`) distinguishes absent / empty / populated `.sqlx`. Qualified by **R3**.

**F4 — RESOLVED.** `sqlx_dependency_crate_names` (`:74-96`) parses `[dependencies]` and `[target.*.dependencies]` as real TOML tables and honours `package = "sqlx"` renames; `sqlx.workspace = true` in the fixture resolves because the alias itself is `sqlx`. Unparseable non-bridge sources are non-fatal. Manifest read/parse failure remains fatal (`:76-87`) — acceptable, since Cargo would fail on the same manifest. Coverage gaps in **R4**.

**F5 — RESOLVED.** `collect_rust_sources` (`:130-156`) is an explicit worklist using `fs::symlink_metadata` and skipping every symlink, so cycles are impossible and broken symlinks and unreadable directories are skipped rather than fatal.

**F6 — NOT RESOLVED.** See **R1**. This is the primary blocker.

**F7 — Adequate as-is.** `stderr_reports_sqlx_offline_metadata_failure` (`rust_interop_probe_diagnostics.rs:113-119`) now has four positive and two negative discrimination cases (`:151-177`), including the load-bearing check that a bare `hash collision` string without the `sqlx_macros::expand_query` note is *not* Cargo-classified. Round 1 listed this as optional; the coverage added is proportionate.

**F8 — RESOLVED.** `examples/tower-http.sifr:11` now declares `tower_http_response_header`, matching the certified `set-header` middleware.

**F9 — RESOLVED.** `rust_interop_probe.rs` is 868 lines (was 881); the preflight moved behind a single `ProbeExecutionFailure`-returning call at `:71`.

---

## Findings (severity order)

### R1 — HIGH · The database sentinel cannot reach any compiler Cargo process; the "load-bearing forcing" claim is false
`crates/sifr_driver/src/tests/package_rust_interop_backend_ecosystem_support.rs:130-145`, `:147-155`; claims at `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1531-1533`, `docs/rust-interop.mdx:251-255`, `internal_docs/rust_interop_architecture.md:1253-1258`, `plans/phases/39_rust_interop.md:348-352`, fixture `README.md:11-15`, `rust_interop_compatibility_matrix.json:406`.

`configure_database_sentinel` arms the sentinel by writing `[env] DATABASE_URL = { …, force = true }` into `<copied scenario root>/.cargo/config.toml`. No Cargo invocation in the pipeline is launched from at or below that directory:

- probe: `command.current_dir(&probe_root)` (`rust_interop_probe.rs:136`) where `probe_root = std::env::temp_dir().join("sifr_rust_probe_…")` (`:88-97`);
- final build: `command.current_dir(project_path)` (`materialize.rs:274`) where `project_path = pending.workspace_root().join(project_name)` under `artifact_cache_root() = std::env::temp_dir().join(…)` (`workspace.rs:272-274`).

Cargo discovers config from the invocation cwd upward; it never reads a **path dependency's** own `.cargo/config.toml`. I verified this directly and then replicated the exact pipeline shape:

```
D) probe crate in /tmp/sifrprobe2 path-depends on the fixture copy;
   fixture/.cargo/config.toml arms DATABASE_URL force=true at a live sentinel;
   SQLX_OFFLINE and DATABASE_URL both unset in the environment
   → cargo check exit 0, NO sentinel connection
   (control: a manual connect to the same port did register a HIT, so the listener was live)
```

And the forcing has nothing to bite on even in principle, because sqlx only goes live when `database_url` is `Some` (`sqlx-macros-core-0.8.6/src/query/mod.rs:156-163`); with `DATABASE_URL` absent it uses `.sqlx` whether or not `SQLX_OFFLINE` is set:

```
A) fixture copy, SQLX_OFFLINE unset, DATABASE_URL unset      → cargo check exit 0
B) fixture copy, SQLX_OFFLINE unset, DATABASE_URL=sentinel   → "error communicating with
   database: Connection reset by peer" at backend.rs:38, SENTINEL HIT
C) fixture copy, SQLX_OFFLINE=true, DATABASE_URL=sentinel    → exit 0, NO HIT
```

So `configure_hermetic_build_environment` could be deleted outright and both mandatory tests would still pass — exactly the round-1 F6 condition. Removing `SQLX_OFFLINE` from the fixture config and adding the `"fixture SQLx environment"` validator mutation (`_scenario_backend.py:192-198`, `:279-282`) is a fine *fixture-policy* assertion, but it does not make the compiler's forcing load-bearing, and the milestone now publishes the stronger claim in five places, including the structured matrix row that feeds the public stable-claims table.

**Verified remediation:** sqlx reads `<CARGO_MANIFEST_DIR>/.env` via dotenvy (`query/mod.rs:390-408`), and that path *does* survive a path-dependency build from another directory:

```
E) fixture/.env contains DATABASE_URL=sentinel; no [env] config;
   SQLX_OFFLINE unset; cargo check from /tmp/sifrprobe2
   → live-connect error at backend.rs:38, SENTINEL HIT
```

Arm the negative test with a `.env` at the copied package root (proves `SQLX_OFFLINE` forcing) and, separately, with an ambient `DATABASE_URL` in the test process (proves `env_remove`). Then `assert_database_sentinel_unused` becomes genuine evidence. Alternatively, narrow every claim listed above to "Sifr forces `SQLX_OFFLINE=true` and strips inherited `DATABASE_URL`" without asserting that the sentinel demonstrates it.

### R2 — MEDIUM · New false rejection: workspace-root `.sqlx` and `SQLX_OFFLINE_DIR` layouts
`crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:411-418`, `:58-61`

`validate_query_metadata` looks only in `backend_root/.sqlx`. sqlx resolves three locations in order — `SQLX_OFFLINE_DIR`, `manifest_dir/.sqlx`, then **`workspace_root/.sqlx`** (`query/mod.rs:165-175`). The third is not exotic: it is what `cargo sqlx prepare --workspace` produces and the reason the fallback exists. Verified empirically — a member package with no `.sqlx` of its own and `.sqlx` at the Cargo workspace root compiles cleanly:

```
/tmp/sifrws/{Cargo.toml [workspace] members=["pkg"], .sqlx/…}, /tmp/sifrws/pkg/ (no .sqlx)
→ cargo check --offline, SQLX_OFFLINE and DATABASE_URL unset → exit 0
```

Under this milestone that package now aborts with `SIFR-RUST-CARGO-0001: … SQLX_OFFLINE=true` but there is no cached data for this query`, and `sqlx_offline_metadata_digest` returns `None`, so the metadata is absent from cache identity as well. Outer-workspace layouts are a supported shape elsewhere in this very module (`rust_interop_cargo_inputs.rs:283`, `:370-376` both walk ancestors for `Cargo.lock` and profile tables). This is the same fail-closed-on-an-unmodelled-configuration class as round-1 F1: correct code that compiled before this milestone now fails, with a message that misdescribes the cause. Resolve the workspace root (or fall through when the package-root `.sqlx` directory is absent entirely), and skip the preflight when `SQLX_OFFLINE_DIR` is set.

### R3 — MEDIUM · Final-build cache identity covers only the entrypoint package root, not the bridge backend root
`crates/sifr_driver/src/build/rust_interop_cargo_inputs.rs:53`, `crates/sifr_driver/src/build/rust_interop.rs:224-234`

The probe key digests `backend_root` (`rust_interop_probe_cache.rs:76`), but `cargo_inputs` digests `package.package_root` of the **entrypoint package only**. Two supported layouts therefore get no `.sqlx` participation in the final artifact cache key: a bridge living in a sub-Cargo-package (`rust/db/.sqlx` — precisely the layout this fixture used before this milestone), and `.sqlx` owned by a non-entrypoint Sifr package in the graph. The preflight still fail-closes on query-text drift in those layouts, so the primary guarantee holds; what is lost is warm-final-build invalidation for `describe`-only tampering. The published wording is unconditional — `rust_interop_compatibility_matrix.json:406` "includes the complete .sqlx directory in probe and generated-build cache identity", `docs/rust-interop.mdx:250-251`, `internal_docs/rust_interop_architecture.md:1218-1219`, `plans/phases/39_rust_interop.md:350`. Either digest the resolved backend roots as well, or qualify the claim to the package-root layout.

### R4 — LOW · Dependency detection ignores dev/build dependency tables and workspace renames
`rust_interop_sqlx_offline.rs:89-94`

`[dev-dependencies]`, `[build-dependencies]`, `[target.*.dev-dependencies]`, and `[workspace.dependencies]` renames (`db = { workspace = true }` where the workspace entry carries `package = "sqlx"`) are all invisible, so the preflight silently disengages. Each fails open, which is right; but combined with R3 a warm final build can reuse a binary produced from stale metadata. Worth closing while the code is fresh.

### R5 — LOW · Preflight is stricter than sqlx on the `hash` field, with an inverted message
`rust_interop_sqlx_offline.rs:425-429`

sqlx compares only `query` (`data.rs:117-119`) and merely asserts `hash` non-empty (`:130-131`). Sifr rejects when `metadata["hash"]` differs from the recomputed digest — a `.sqlx` file that Cargo accepts. The emitted reason, `"hash collision for saved query data"`, describes the opposite condition (matching filename, differing query). Low reachability, but it is a diagnostic-stability inaccuracy in a message the negative test asserts on (`package_rust_interop_backend_ecosystem_support.rs:103`).

### R6 — LOW · Validator crashes rather than fails on `"describe": null`
`verification/areas/rust_interop/checks/_scenario_backend.py:313-314`

`data.get("describe", {})` returns `None` for an explicit JSON null, so `describe.get("nullable")` raises `AttributeError`. Not reachable from the checked-in fixture, but it is reachable from exactly the mutation style `_run_mutation` uses elsewhere, and would surface as a traceback instead of a clean failure line.

### R7 — NIT · `cargo_metadata_digest` repurposed as the SQLx digest carrier
`rust_interop_cargo_inputs.rs:53` vs `:107`. The same field means "sysroot content digest" on one path and "`.sqlx` directory digest" on the other, and `combined_cargo_inputs` overwrites it with an unrelated combined digest at `:76`. Information is preserved (the combined digest derives from `format!("{primary:?}")`), so there is no correctness bug — but a dedicated field would keep the cache-identity contract legible.

### Observation (not a defect)
`validate_probe_sqlx_offline_metadata` runs at `rust_interop_probe.rs:71`, deliberately **before** the cache-hit early return at `:83-86`. That is required for the fail-closed-on-warm-cache goal, and it means every warm build re-`syn::parse_file`s every `.rs` under the bridge package's `src/`. It is gated on the package actually depending on sqlx (`:65-67`), so the cost is scoped. No user-triggerable panics in the new production code: no data-dependent `unwrap`/`expect`, all I/O and parse results are `Result`/`Option`-handled, and the only recursion (`collect_module_queries` over inline modules, `syn::visit`) is bounded by what `syn::parse_file` already accepted.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** Shadow crates gone; `cargo tree --workspace --edges features --locked --offline` asserts `axum v0.8.9`, `tower-http v0.7.0` + `set-header`, `sqlx v0.8.6` + the three frozen features (`…support.rs:157-190`). |
| Hermetic `127.0.0.1:0` Axum service through real tower-http, deterministic shutdown | **Met.** `src/bridges/backend.rs:59-98` — real `axum::serve`, `SetResponseHeaderLayer::if_not_present`, raw loopback exchange asserting `HTTP/1.1 200 OK` + `x-sifr-tower` + body, `with_graceful_shutdown` joined under a 2s timeout. Marker observed in my own run. |
| Real SQLx macro from checked-in `.sqlx` under `SQLX_OFFLINE=true`, no `DATABASE_URL`/live DB | **Met for the fixture**, but the "prove neither `DATABASE_URL` nor a live database is required" sub-clause is not actually proven — see **R1**. |
| Mandatory generated-package diagnostic: independent missing + stale mutation, stable `SIFR-RUST-CARGO-0001`, database/network access disabled | **Met for the fixture.** Control accepts unmutated metadata, then both mutations on the same package root with distinct asserted details. The "with database access disabled" framing rests on the inert sentinel (**R1**). |
| Bind to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met**, except that three of the published statements are inaccurate (**R1**, and **R3**'s unconditional cache-identity wording). Counts verified to the unit: 36 rows / 36 fixture_rows / 3 categories, 72 passing / 0 planned, 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`, 36 stable claims, `future_runtime_rows=0`. |
| Validator self-tests without weakening earlier rows | **Met at the validator level.** `_scenario_backend.py` contributes 19 cases inside the 228 total; no prior row regressed (10/10 reproduced). The `"offline env"` case was correctly inverted to `"fixture SQLx environment"`, but see **R1** for what it does and does not establish. |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The `certification_14` transition edits (`:1546-1552`) and the `future-owned` backstop removals are handled correctly, and the compatibility self-test is properly inverted with a companion assertion that an *active* category may still not be empty (`check_compatibility_matrix.py:401-418`). The round-1 LSP disposition was accepted in round 1 and is unchanged; no `lsp` or editor-integration source is in this delta.

## Validation assessment

Every recorded figure I re-ran matched: 444/65 driver tests, both mandatory tests green (82.58s combined vs. the recorded 62.52s + 18.41s — timing variance only), 10/10 area with 228 mutation cases, all counts, Clippy/rustfmt/file-size/driver-maintainability/resource-gate. The seven focused SQLx unit tests exist and cover what the evidence says they cover. The one inaccurate evidence statement is `:1531-1533` ("Sifr's environment forcing is therefore load-bearing in the positive final build and in the valid control"), which R1 disproves in both halves.

---

## Required fixes

1. **R1** — Make the sentinel real or narrow the claim. Verified working mechanism: arm `DATABASE_URL` through `<copied package root>/.env` (reaches sqlx across a path-dependency build; case E above) so that `SQLX_OFFLINE` forcing is what prevents the connection, and add a second arming via ambient `DATABASE_URL` in the test process so `env_remove` is covered too. Then correct or remove the "load-bearing"/"without contacting the armed database sentinel" language at `rust-interop-runtime-ecosystem-certification.md:1531-1533`, `docs/rust-interop.mdx:251-255`, `internal_docs/rust_interop_architecture.md:1253-1258`, `plans/phases/39_rust_interop.md:348-352`, the fixture `README.md:11-15`, and `rust_interop_compatibility_matrix.json:406`.
2. **R2** — Do not reject a package whose metadata sqlx would legitimately resolve elsewhere. Resolve `workspace_root/.sqlx` (and honour `SQLX_OFFLINE_DIR` by disengaging), or fall through to Cargo when no package-root `.sqlx` directory exists at all. Add a unit test per resolution location documenting the boundary.
3. **R3** — Either extend the final-build digest to the resolved bridge backend roots, or narrow the four unconditional "complete `.sqlx/` directory participates in probe and generated-build cache identity" statements to the package-root layout actually covered.

## Optional suggestions

- **R4** — include `dev-dependencies`, `build-dependencies`, `target.*.dev-dependencies`, and `[workspace.dependencies]` package renames in `sqlx_dependency_crate_names`.
- **R5** — drop the `hash`-field equality check (or keep it with a message that names the actual condition), so the preflight is never stricter than sqlx.
- **R6** — make `_validate_query_metadata` tolerate `"describe": null` instead of raising.
- **R7** — give the SQLx digest its own field on `RustInteropCargoInputs` rather than reusing `cargo_metadata_digest`.
- Consider a unit test pinning the preflight's fall-through for `sqlx::query!` nested inside another macro invocation and for function-local `use sqlx::query;`, so those conservative boundaries are documented rather than incidental.
