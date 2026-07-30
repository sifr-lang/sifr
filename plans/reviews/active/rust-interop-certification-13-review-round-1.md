# Milestone Review — Rust Interop Runtime Ecosystem Certification 13

## Verdict: **NOT SATISFIED**

The certification substance is real and largely excellent — genuine crates.io `axum 0.8.9` / `tower-http 0.7.0` / `sqlx 0.8.6`, a real hermetic loopback, real offline query-macro expansion, and correct matrix/claims/docs transitions. But the new pre-Cargo SQLx verifier — the one novel piece of production compiler code in this milestone — has a **confirmed false positive that breaks valid `sqlx::query!` usage** and a **fail-open detection gap that undercuts the published stable claim**. Both are actionable.

---

## Scope reviewed

Full committed delta `origin/main..HEAD` (`b231daf81`, `7a27b7896`), 43 files. The listed unstaged worktree paths (`editor_integrations`, leetcode corpora, `.cert5probe/`, `.claude/`, stray webp, `plans/phases/43_interoperability.md`) were excluded and are not attributed to this milestone.

### Independently executed

| Check | Result |
|---|---|
| `verification/areas/rust_interop/runner.py` | **10/10 variants, 0 failures**; 36 fixtures, 44 crates, 61 package examples, 18 scenarios, **228 mutation self-tests**, 36 stable claims, 20 stale-draft cases |
| `check_compatibility_matrix.py` (+`--self-test`) | pass; rows=36, categories=3, self-test cases=7 |
| `check_sysroot_stdlib_resource_certification_gate.py` (+`--self-test`) | pass; `future_runtime_rows=0` |
| Mandatory generated-package tests (both, `--ignored --exact`) | **2 passed in 78.20s** |
| `cargo test -p sifr_driver` | **449 passed, 0 failed, 65 ignored** (lib target = 505 total → 440 passing non-generated, matching the plan's figure) |
| `cargo clippy --workspace -- -D warnings` | pass (the `--all-targets` failures are pre-existing in untouched `sifr_lowering` test code) |
| `cargo fmt --check` | pass |
| Fixture `Cargo.lock` audit | 184 packages: **182 registry-sourced, only 2 path deps** (`backend-feature-package`, `sifr_runtime`) — **no shadow crates**; `rust/{axum,sqlx,tower_http}` correctly deleted |
| `cargo tree --locked --offline --edges features` in fixture | `axum v0.8.9`, `tower-http v0.7.0` + `set-header`, `sqlx v0.8.6` + `runtime-tokio-rustls`/`postgres`/`macros` |
| Root-lock provenance | `require_root_lock_subset` enforces name+version+source+**checksum** identity against the repo root lock for every external package; passes |
| File sizes / inventory counts | all touched files < 900; 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`; 13/4/10/9 execution kinds; **0 `"planned"`, 0 `future_owner`** remaining |
| Empirical parser probe (out-of-repo scratch crate, syn 2) | reproduced finding **F1** |

---

## Findings (severity order)

### F1 — HIGH · False positive: valid `sqlx::query!` invocations now fail the build
`crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:212-218` (`parse_bind_arguments`), `:190-196` (`QueryInput`)

```rust
while !input.is_empty() {
    input.parse::<Token![,]>()?;
    let _argument: Expr = input.parse()?;   // <-- empty input after trailing comma
}
```

Empirically confirmed with the identical parser against syn 2:

```
"\"SELECT $1\", value"   -> Ok
"\"SELECT $1\", value,"  -> Err("unexpected end of input, expected an expression")
```

`sqlx::query!` is `macro_rules!($query:expr, $($args:tt)*)` forwarding to `args = [$($args)*]`, an `ExprArray` (`sqlx-0.8.6/src/macros/mod.rs:314-330`) — a **trailing comma is valid sqlx** and compiles today. After this milestone, `sqlx::query!("SELECT $1", value,)` in a bridge aborts the build with `SIFR-RUST-CARGO-0001: SQLx query! must start with a string literal: unexpected end of input`. That is both a false rejection and a misleading message.

Second shape, same cause: sqlx accepts `+`-concatenated literal sources (`Punctuated::<LitStr, Token![+]>::parse_separated_nonempty`, `sqlx-macros-core/src/query/input.rs:54-58`). `sqlx::query!("SELECT " + "1")` parses the first `LitStr`, then `parse_bind_arguments` demands a comma, sees `+`, and hard-errors.

This is a user-visible regression introduced by this milestone: code that compiled before now fails. Fail-closed is the right default for *metadata*, but not for *syntax the verifier does not understand*.

### F2 — HIGH · False negative: the preflight is blind to the common `use sqlx::query;` form and to all `query_file*!` macros
`crates/sifr_driver/src/build/rust_interop_sqlx_offline.rs:165-182`

```rust
if segments.first().map(String::as_str) != Some("sqlx") { return Ok(None); }
```

With `use sqlx::query;` (or `use sqlx as db;`), the macro path's first segment is `query`, not `sqlx`, so the query is silently skipped. Likewise `query_file!`, `query_file_as!`, `query_file_scalar!`, `query_file_unchecked!`, `query_file_as_unchecked!`, `query_file_scalar_unchecked!` fall into the `_ => return Ok(None)` arm — six public sqlx macros that all require `.sqlx` metadata, none handled.

This matters specifically because of *why* the preflight exists. The plan states it "prevents stable Cargo's incomplete `.sqlx/` input tracking from hiding either mutation behind a warm probe cache" (`rust-interop-runtime-ecosystem-certification.md:1478-1482`). I confirmed `.sqlx/*.json` is not a cache input at any layer: the probe cache key (`probe_cache_key`) has no metadata term and the preflight is its only guard; `bridge_source_digests` digests `src/bridges` only (`rust_interop_cargo_inputs.rs:14-31`); and `digest_package_source_map` covers Sifr modules only (`crates/sifr_package/src/graph/digest_source_map.rs:8-11`). So for an unqualified or `query_file!` query, **stale metadata is silently accepted on a warm cache** at both the probe and final-build layers.

The published claim does not carry that caveat — `rust_interop_compatibility_matrix.json:406` asserts flatly: *"rejects missing or stale metadata as SIFR-RUST-CARGO-0001 before database access."* Either the detector must cover the ordinary import forms, or the claim must be narrowed to fully-qualified `sqlx::query{,_as,_scalar}[_unchecked]!`.

### F3 — MEDIUM · Preflight validates only query text and hash; tampered `describe` survives
`rust_interop_sqlx_offline.rs:118-122` compares `metadata["query"]` and `metadata["hash"]` only. A `.sqlx` file with the correct SQL but altered `describe.columns[].type_info` / `nullable` / `db_name` passes the preflight and — per the cache analysis in F2 — will not force a Cargo recompile, so the bridge keeps the previously-inferred column types. The Python scenario validator *does* check `db_name` and `nullable` (`_scenario_backend.py:314-318`), but only for this one fixture; the compiler guard does not. Hashing the full metadata document (or at least `describe`) into the probe cache key would close this.

### F4 — MEDIUM · Substring manifest gate and unconditional `syn::parse_file` hard failure
`rust_interop_sqlx_offline.rs:21-23`:

```rust
if !manifest.contains("sqlx") || !manifest.contains("macros") { return Ok(()); }
```

Fragile in both directions: a manifest mentioning `sqlx` only in a comment alongside any `proc-macros`/`macros` token opts in; a package inheriting sqlx purely from a workspace manifest elsewhere may opt out. Worse, once opted in, `collect_rust_sources` walks **all** of `src/` and any file `syn::parse_file` cannot handle becomes a fatal `SIFR-RUST-CARGO-0001` (`:46-51`) — a build failure caused by the driver's pinned `syn` rather than by anything wrong with the package. Parse the dependency table properly, and treat unparseable non-bridge sources as "no queries found" rather than fatal.

### F5 — LOW · Unbounded recursive traversal follows symlinks
`rust_interop_sqlx_offline.rs:64-102`. `collect_rust_sources` recurses through `is_dir()` (symlink-following) with no depth bound and no visited set; a symlink cycle under `src/` yields stack-overflow abort in the compiler. The sibling traversal in the same crate uses an explicit worklist (`rust_interop_cargo_inputs.rs:136-153`). Separately, a broken symlink under `src/` now makes `read_dir` fail and aborts the build with `failed to inspect Rust package path`.

### F6 — LOW · The armed database sentinel is not load-bearing evidence
`package_rust_interop_backend_ecosystem_support.rs:130-155`. In both mutation cases the preflight rejects *before Cargo is spawned*, so no connection was ever possible; and the sentinel config itself sets `SQLX_OFFLINE = { value = "true", force = true }` (`:140`), as does the checked-in fixture config. So `assert_database_sentinel_unused` would pass with `configure_hermetic_build_environment` deleted. The mechanism is covered only by the string-level unit test at `rust_interop_sqlx_offline.rs:229-248`. The plan (`:1481`), fixture README, `docs/rust-interop.mdx:252`, and `39_rust_interop.md` all present the sentinel as proof of hermeticity; that reads stronger than the test delivers. A case that leaves `SQLX_OFFLINE` unset in the fixture config and relies solely on the driver's forcing would make the claim real.

### F7 — LOW · Downstream classifier is untested end-to-end and now nearly unreachable
`rust_interop_probe_diagnostics.rs:113-119` requires the literal `sqlx_macros::expand_query` in stderr. Coverage is two hand-written string tests (`:151-176`); no test drives a real sqlx compile failure through it. Since the preflight shadows this branch for every case it can see, it is effectively dead for the certified fixture and will rot silently if upstream note wording changes.

### F8 — NIT · Misleading package-example naming
`examples/tower-http.sifr:11` declares `tower_http_trace_layer`, but the certified feature set is `set-header` only and no `Trace` layer exists anywhere in the bridge. `examples/axum.sifr`'s `axum_router` and `examples/sqlx.sifr`'s `sqlx_query` all funnel into the same two bridge functions — acceptable, but the `trace_layer` name actively misdescribes the evidence.

### F9 — NIT · `rust_interop_probe.rs` is at 881/900 lines
19 lines of headroom after this milestone's +18. The preflight call block (`:71-85`) would sit more naturally in `rust_interop_sqlx_offline.rs` as a `ProbeExecutionFailure`-returning helper.

### Observation (not a defect)
Removing the "at least one future-owned row" backstop from `check_sysroot_stdlib_resource_certification_gate.py:80-85` was necessary (`ecosystem_backend_certification` was the last such row) and the supported-stdlib-core invariants are intact, with the completed-matrix self-test properly inverted. The revisit-forcing function it served is now carried only by prose in `certification_14`. Acceptable, but worth a deliberate note when `certification_pkg_resource_core` wakes up.

---

## Checklist assessment — `certification_13`

| Item | Assessment |
|---|---|
| Replace shadow crates with exact-pinned real graph, frozen SQLx features, checked-in lock | **Met.** Shadow crates deleted; 182/184 lock entries registry-sourced with root-lock checksum parity; `cargo tree --locked --offline` confirms `=0.8.9`/`=0.7.0`/`=0.8.6` and exactly the three SQLx features. |
| Hermetic `127.0.0.1:0` Axum service through real tower-http, deterministic shutdown | **Met.** `src/bridges/backend.rs:58-99` — real `axum::serve`, `SetResponseHeaderLayer::if_not_present`, raw loopback exchange asserting `HTTP/1.1 200 OK` + `x-sifr-tower` + body, `with_graceful_shutdown` joined under a 2s timeout. Marker observed in my own run. |
| Real SQLx macro from checked-in `.sqlx` under `SQLX_OFFLINE=true`, no `DATABASE_URL`/live DB | **Met for the fixture.** `sqlx::query!` at `backend.rs:37`, genuine sqlx-shaped metadata, identity bound into the evidence marker. Forcing mechanism itself is only unit-tested (**F6**). |
| Mandatory generated-package diagnostic: independent missing + stale mutation, stable `SIFR-RUST-CARGO-0001`, before DB/network | **Met for the fixture**, with the sentinel caveat in **F6**. Control run accepts unmutated metadata first, then both mutations on the same package root; distinct expected details asserted per direction. |
| Bind positive/negative to distinct mandatory tests; promote only this row; update policy, claims, docs, provenance, inventory | **Met.** `fixture.json` validation blocks resolve through the blocking `sifr_driver_generated_builds` merge-profile suite (`_provenance_checks.py:414-421`, `_weakest_executing_profile`); exactly one row promoted; counts match to the unit. |
| Validator self-tests for versions/features, ownership, loopback/middleware, offline env, metadata identity, both negative directions, provenance, bridge contract — without weakening earlier rows | **Met at the validator level.** `_scenario_backend.py` adds 19 cases (baseline + 16 structural + 2 metadata) inside the 228-case total; no prior row regressed (10/10 reproduced). |
| Gates / review rounds / merge / unblock `certification_14` | **Open, correctly unchecked.** |

The checklist items are honestly marked. **The gap is not in the checklist — it is in the production code the checklist ships** (F1/F2), which the fixture-scoped acceptance criteria do not exercise.

---

## Validation assessment

The recorded evidence is accurate and I reproduced its load-bearing parts. Every count in the plan's "Validation evidence to date" matched exactly (36/44/61/18, 228 mutations, 72 passing / 0 planned, 21/14/1, 13/4/10/9, 36 claims, 440+65 driver tests). Timings differed only as expected (78.20s for both mandatory tests vs. 74.81s + 63.34s cold).

**The isolated LSP rerun is acceptable.** The create-PR lane's single failure was an LSP protocol-shutdown timeout after 23 successful requests in a shared worktree — a resource-contention signature with no code path touching Rust interop, the driver build layer, or the verification area. The full six-variant `lsp-smoke` suite passing immediately in isolation is the right disposition, and it matches how the same class of flake was resolved in the certification-12 rounds. No `lsp` or editor-integration source is in this delta.

---

## Required fixes

1. **F1** — Accept trailing commas in `parse_bind_arguments`, and accept `+`-concatenated `LitStr` sources in `QueryInput`/`QueryAsInput`. Add regression tests for `sqlx::query!("SELECT $1", value,)` and `sqlx::query!("SELECT " + "1")`. Then make unparseable macro input **non-fatal** (fall through to Cargo, which is the authority on sqlx syntax) rather than a hard `SIFR-RUST-CARGO-0001`.
2. **F2** — Either recognize unqualified/aliased `query{,_as,_scalar}[_unchecked]!` (resolve `use sqlx::…` imports, or match on the last segment with a `sqlx` reachability check) **and** handle the six `query_file*!` macros, **or** narrow the claim text in `rust_interop_compatibility_matrix.json:406`, `docs/rust-interop.mdx:249-253`, `internal_docs/rust_interop_architecture.md:1215-1219`, and the fixture README to the fully-qualified inline-literal forms actually guarded. Add a unit test per unhandled form documenting the chosen boundary.
3. **F4** — Replace the `manifest.contains("sqlx") && manifest.contains("macros")` gate with real dependency-table parsing, so opt-in is deterministic in both directions.

## Optional suggestions

- **F3** — hash the full `.sqlx` document (or `describe`) into `probe_cache_key`, closing the tampered-metadata path structurally instead of by field comparison.
- **F5** — convert `collect_rust_sources` to the worklist form already used in `rust_interop_cargo_inputs.rs`, with a visited set; do not let a broken symlink fail the build.
- **F6** — add a negative case where the fixture config does **not** force `SQLX_OFFLINE`, so the sentinel becomes genuine evidence for the compiler's env forcing.
- **F7** — drive one real sqlx offline failure through `stderr_reports_sqlx_offline_metadata_failure` (e.g. a query the preflight intentionally does not see), so the branch is not dead.
- **F8** — rename `tower_http_trace_layer` to reflect the certified `set-header` middleware.
- **F9** — move the preflight call block out of `rust_interop_probe.rs` before it crosses 900.
