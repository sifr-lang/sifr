## SATISFIED

Certification 11 is PR-ready. All three round-1 blockers are genuinely fixed, seven of eight non-blocking findings are resolved, and every gate I ran passes. The only live failure in the shared worktree is the explicitly preserved, unrelated `ecosystem_backend_certification` hunk, and I verified cert-11's own matrix hunk is independently valid.

---

## Gate results (all run against the current worktree)

| Gate | Result |
|---|---|
| `cargo test -p sifr --bin sifr …test_lockfile_feature_and_frozen_drift_rejected_without_network -- --ignored --exact` | **pass**, 0.47 s — 5 distinct reasons observed |
| `…test_locked_offline_sifr_commands_and_warm_cache -- --ignored --exact` | **pass**, 103.63 s (issue doc reports 65.01 s — machine variance) |
| `cargo test -p sifr_package --lib` | **pass** 142/142 |
| `cargo test -p sifr_driver --lib` | **pass** (435 passed, 61 ignored in `sifr_tests`) |
| `cargo clippy -p sifr_package --lib --no-deps -- -D warnings` | **clean** |
| `cargo clippy -p sifr_driver --lib --no-deps -- -D warnings` | **clean** |
| `cargo clippy -p sifr --bin sifr --no-deps -- -D warnings` | **clean** |
| `cargo clippy --workspace -- -D warnings` (AGENTS.md/CI command) | **clean** |
| `cargo fmt --check` / `git diff --check` | **clean** |
| `check_fixture_matrix.py` / `--self-test` | `fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18` / `cases=190` |
| `check_tiers.py` / `--self-test` | `tiers=5 fixtures=36` / `cases=6` |
| `check_stable_support_claims.py` | `claims=34` |
| `check_stale_drafts.py` | ok |
| `check_typescript_go_transfer_guardrails.py` | PASS |
| `check_hir_maintainability_guardrails.py` | PASS |
| `check_file_size_guardrails.py` | PASS (2998 files, limit 900) |
| `check_compatibility_matrix.py`, backend row reverted to HEAD in a hardlink copy | `rows=36 fixture_rows=36 categories=4`, **exit 0** |

Counts on the reverted baseline recompute **exactly** as the issue doc claims: 21 `supported`, 12 `supported-through-bridge`, 1 `unsupported-by-design`, 2 `future-owned`; **68 passing / 4 planned**.

I also ran `cargo clippy --workspace --all-targets --no-deps -- -D warnings`, which fails — but only on `crates/sifr_ipc/src/**` and `crates/sifr_stdlib_manifest/src/lib.rs:29` `clippy::expect_used` in *lib-test* code. Both files are unmodified by this change and this is not the AGENTS.md/CI command. Not attributable to cert-11.

---

## Round-1 findings, reinspected individually

**B1 — Clippy failures — FIXED.** `type RegistryEntry = (String, String, String, String)` at `cargo_resolution.rs:15` retires `type_complexity`; `probe_resolution_diagnostics` now takes `&[RenderedDiagnostic]` (`rust_interop_probe.rs:126`, `:147`), retiring `needless_pass_by_value`. The trailing `use super::cargo_resolution::…` after `mod tests` is gone — `rust_interop_probe.rs` now has all imports at lines 1–22 and `mod tests` at 597. Verified by four independent Clippy runs.

**B2 — lexical entry-file classification — FIXED, and fixed at the right authority.** `entrypoint_declares_rust_interop` is deleted. `package_graph_context.rs:41-44` now derives the boundary hint from `session.manifest.as_ref().is_some_and(SifrManifest::declares_rust_backend)`. This is not merely "some parsed state" — it is *the same predicate* `crates/sifr_package/src/graph/derive.rs:92` uses to classify a package as `PackageClassification::RustBackedSifr`, and `derive.rs:146` enforces `validate_pure_markers` on packages that fail it. So a package carrying `@rust` declarations necessarily satisfies it, and the reclassification is gated additionally on `cargo_lock_failure_reason(&excerpt).is_some()`. `load_package_graph_context_from_root` and the non-entrypoint path pass `false`, so the hint is scoped to the entrypoint boundary. The comment at `:38-40` records why.

**B3 — negative test asserted a fabricated value — FIXED.** The synthetic `cargo metadata` + direct-classifier block is gone. `diagnostic_test_sink.rs` is a thread-local `#[cfg(test)]`-only sink recorded from inside `render_diagnostics` (`diagnostic_rendering_and_run.rs:58-59`), so what the test sees is the command's real rendered output. `assert_drift_rejected` now asserts `exit_code == EXIT_USER_DIAGNOSTIC`, **exactly one** diagnostic, `code == RUST_CARGO_METADATA`, a **case-specific reason**, and lock byte-identity. And it runs `move_rust_declarations_to_imported_module` first (`cargo_lock_mode_certification_tests.rs:381`, `:454-475`) — the `@rust` declarations live in `src/bridge.sifr`, imported by an entry module with no decorators. This is exactly the configuration that produced `SIFR-PACKAGE-0101` in round 1; the observed output is now:

```
E SIFR-RUST-CARGO-0001 … missing lockfile
E SIFR-RUST-CARGO-0001 … stale selected version
E SIFR-RUST-CARGO-0001 … checksum drift
E SIFR-RUST-CARGO-0001 … dependency source drift
E SIFR-RUST-CARGO-0001 … requested feature selection drift
```

The test would now fail if B2 regressed. Evidence binding is genuine.

**N1 — no argv evidence — FIXED, and the "every invocation" claim is real.** `sifr_package/src/cargo/invocation_trace.rs` records the fully-built `Command` immediately before `.output()`. I enumerated every `Command::new("cargo")` spawn in `sifr_driver`/`sifr_package`/`sifr` and confirmed all four are instrumented with no post-record mutation: `cargo/load.rs:62` (`package-metadata`), `cargo_resolution.rs:192` (`resolution`), `rust_interop_probe.rs:142` (`rust-probe`), `materialize.rs:284` (`final-build`). The only uninstrumented spawn is `test_runner/execution.rs:136` (`sifr test`), outside this certification's check/build/run scope. `assert_cargo_invocations_preserve_lock_modes` fails closed on an unclassified phase (`:307`), requires per-mode `final-build` *and* `package-metadata`, ties `--offline` on the resolution phase to `is_network_disallowed()`, requires frozen strength on probes, and asserts `warm_invocations.is_empty()` — a warm binary cache hit launches no Cargo at all.

**N2 — token pinned to false README prose — FIXED at the root.** `rust/locked_bridge/src/lib.rs:2` now genuinely is `indexmap::IndexMap::<String, u32>::new()`, so `CARGO_LOCKED_SCENARIO_TOKENS` pins executable source, not prose. Both READMEs are corrected: network denial is attributed to `--frozen` ("Every mutation runs through Cargo `--frozen`, which denies network and lockfile writes"), and the false `CARGO_NET_OFFLINE=true` claims are gone. The `--frozen` attribution matches the code — `cmd_check` sets no such env var. The dead `"cache miss"` / `"cache hit"` prose tokens were also dropped from the policy.

**N3 — sysroot could override the package lock — FIXED with a unit test.** `seed_lockfile_from_authorities` seeds from the lowest-priority authority and overlays higher-priority registry packages by name (`cargo_resolution.rs:209-280`); `validate_authoritative_registry_entries` now enforces that if the *primary* lock owns a name, the entry must be in the primary — the sysroot may only authorize remaining names (`:393-403`). `package_lock_registry_pin_overrides_sysroot_pin` (`:533`) asserts 1.0.0 wins and 2.0.0 is absent.

**N4 — reasons not kind-specific — FIXED.** New `sifr_package/src/cargo/drift.rs` compares the checked-in lock against reachable manifests (workspace members + path deps) *without retrying or mutating Cargo resolution*, and `cli_lock_modes.rs:41-46` consults it only for the ambiguous `"stale lockfile or feature/source drift"` branch. All five reasons are distinct in observed output.

**N5 — run/check silently downgraded — FIXED for all three, with a fast test.** `check_and_package_commands.rs:42-49`, `diagnostic_rendering_and_run.rs:103-110` (build), `:226-234` (run). `constrained_modes_reject_manifestless_check_build_and_run` is a **non-ignored** test asserting one diagnostic, `RUST_CARGO_METADATA`, and `"sifr {command} --frozen requires"` for all three. `docs/cli/build-run.mdx` now documents the choice for all three, not just build.

**N6 — headroom exhausted — SUBSTANTIALLY FIXED.** `check_and_package_commands.rs` 900→**776**, `diagnostic_rendering_and_run.rs` 898→**864**, `rust_interop_probe.rs` 894→**863**, `package_rust_interop_build_tests.rs` 900→**770** (via `package_rust_interop_test_support.rs`). Nine new focused modules. `cli_model_and_entrypoint.rs` (888) and `entrypoint.rs` (885) remain tight; see non-blocking note 5.

**N7 — prepared-lock store growth — ADDRESSED as requested.** `prepared_lock_path` now hashes `normalized_manifest_cache_input`, which rewrites path-dependency roots to `sifr-path-dependency-manifest:{digest}` (`:346-376`), retaining Cargo prefix args, per-authority lock digests, and vendor roots, and bumping to `v5`. Keys now repeat across temp roots; `prepared_resolution_cache_identity_ignores_ephemeral_path_roots` proves it. No eviction — see note 4.

**N8 — misleading predicate at the `Offline` call site — FIXED.** Explanatory comment at `cargo_resolution.rs:150-153`.

---

## Independent assessment

**Safety / no silent fallback.** The design is fail-closed at every branch I traced. Constrained mode with no authority → error (`:59-63`). Missing authority file → error (`:64-71`). `validate_authoritative_registry_entries` runs on **both** the cold and warm (cache-restore) paths (`:84-88`, `:100-104`), so even a stale or tampered prepared-lock cache entry cannot smuggle a non-authoritative package — the cache is an optimization, never a trust boundary. Absent checksums normalize to `"<missing>"` and only validate against an authority that also lacks one. `cache_prepared_lock` publishes via temp + rename with a benign lost-race branch. `assert_unchanged` runs after both the probe and the final build.

**Documentation honesty.** `docs/package_management.md`, `docs/cli/build-run.mdx`, `docs/rust-interop.mdx`, and `internal_docs/rust_interop_architecture.md` each disclose the non-obvious behaviours rather than hiding them: probes run at frozen strength while the final build preserves the requested flag; all three constrained modes share one probe cache bucket; `PackageOwned` probes preserve package Cargo sources; the prepared lock must stay byte-identical; and `supported` explicitly "does not claim that an uncached package can resolve in offline or frozen mode." No overclaim found.

---

## Non-blocking observations (none block this PR)

1. **`drift.rs` has no fast unit tests.** 229 lines of new heuristic, bound to evidence only through the `--ignored` negative test. A handful of table-driven tests over synthetic lock/manifest pairs would make regressions cheap to catch. `crates/sifr_package/src/cargo/drift.rs:19`.
2. **`feature_selection_seen` is a global OR** (`drift.rs:219-222`, consumed at `:63`). Any dependency anywhere with a non-empty `features` array makes an otherwise-unclassified stale-lock failure report `"requested feature selection drift"`. The doc comment says "best-effort", and the certified guarantee is the stable **code**, not the reason string — but the label can mislead.
3. **One imprecise doc sentence.** `internal_docs/rust_interop_architecture.md` says Sifr "seeds a generated resolution from the package lock when one is present, otherwise from the sysroot lock." The code seeds from the *lowest*-priority authority and overlays the package lock's registry packages. The very next sentence states the correct net effect, so the record is not wrong — just loosely worded.
4. **Prepared-lock store still has no eviction** (`cargo_resolution.rs:317-321`). Entries are tiny and now key-stable; a bounded/GC'd store or a note remains worthwhile.
5. **`cli_model_and_entrypoint.rs` (888) and `entrypoint.rs` (885)** are within 15 lines of the 900 cap. The next touch forces a split.
6. **`nearest_ancestor_file`** (`entrypoint_resolution.rs:57-62`) can select a `Cargo.lock` above the package root. Unreachable on the certified path — package `cargo metadata` fails first in constrained modes — but a tighter bound to the package/workspace root would remove the possibility.

---

## Shared-worktree commit hygiene

**Must stage** — untracked and load-bearing: `cargo_diagnostics.rs`, `cargo_lock_mode_certification_tests.rs`, `cli_lock_modes.rs`, `diagnostic_test_sink.rs`, `package_graph_context.rs`; `cargo_invocation_trace.rs`, `cargo_resolution.rs`, `entrypoint_artifact.rs`, `entrypoint_resolution.rs`, `rust_interop_probe_features.rs`, `rust_interop_probe_nonce.rs`, `rust_interop_probe_paths.rs`, `package_rust_interop_test_support.rs`; `cargo/drift.rs`, `cargo/invocation_trace.rs`; `_scenario_cargo_locked.py`; `fixtures/cargo_locked_offline/examples/indexmap.sifr`. Plus the `.gitkeep` deletion (correct — the directory now holds real files).

**Must also stage** the three review docs the issue-doc hunk links, all currently untracked with content: `rust-interop-certification-10-review-round-4.md` (5.5 KB), `-round-5.md` (5.7 KB), `rust-interop-certification-11-review-round-1.md` (18.5 KB — the round-1 0-byte placeholder is now populated). Without them the merged doc has dead links.

**Must exclude:** the `ecosystem_backend_certification` hunk at `rust_interop_compatibility_matrix.json:396-400` (preserve in worktree — I confirmed it is the *sole* reason the live checker fails, and that reverting only it yields exit 0); `editor_integrations` submodule pointer; untracked `verification/areas/algorithmic_compatibility/corpora/leetcode`, `.cert5probe/`, `.claude/`, `"logo 06.48.53.webp"`, `"docs/logo/logo.webp 08-03-09-514.webp"`, `plans/phases/43_interoperability.md`. None touch cert-11 code, data, fixtures, or docs.

**One item to resolve before opening the PR:** `plans/reviews/active/rust-interop-certification-11-review-round-2.md` is a 0-byte untracked placeholder. Populate it with this verdict or leave it unstaged — do not commit an empty file.

The last issue-doc checkbox (`Run focused and authoritative local gates, complete Opus review rounds…, merge the PR`) is correctly still unchecked; it closes at merge.
