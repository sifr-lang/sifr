## SATISFIED

Exact head `4c1fdeae6e774460ab4c4cb3ddbe19c1016c1471` (PR #3075, single commit, draft, `MERGEABLE`/`CLEAN`, base `main`) implements certification_11 correctly. Nothing blocks merge.

### Critical isolation check — PROVEN

The decisive pair of runs:

| Tree | `check_compatibility_matrix.py` |
|---|---|
| Exact head `4c1fdeae6` (exported via `git archive`) | `rows=36 fixture_rows=36 categories=4`, **exit 0** |
| Live worktree (backend hunk present) | **exit 1** — `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence` |

- **The backend hunk is absent from the commit.** `git diff 3c9601d26 4c1fdeae6 -- rust_interop_compatibility_matrix.json` is a *single* hunk at `@@ -446,15` promoting only `cargo_locked_offline`. The `@@ -396,8` backend hunk exists solely as an unstaged worktree change. In the committed blob, `ecosystem_backend_certification` is still `future-owned-by-separate-phase`, `future_owner` present, tier 4, both evidence directions `planned`.
- The only `ecosystem_backend` strings in the diff are prose inside the committed review markdown — no data/code hunk. No file matching `backend` is touched.
- Future-owned set in the committed blob is exactly `{ecosystem_backend_certification, ecosystem_cli_certification}`.
- No excluded artifact leaked in: no `*.webp`, `.cert5probe/`, `.claude/`, `editor_integrations`, leetcode corpus, `43_interoperability.md`, or round-3 draft.
- `crates/`, `docs/`, `internal_docs/`, `scripts/` are **fully clean** in the worktree, so in-place cargo runs bind to the exact head. 2-dot and 3-dot diffs both yield 69 files, so the stale base introduces no drift.

### Independently re-run at the exact head

| Gate | Result |
|---|---|
| Mandatory positive + negative (`--bin sifr -- --ignored`) | **2 passed**, 99.86 s |
| Negative test, 5 distinct reasons observed | missing lockfile / stale selected version / checksum drift / dependency source drift / requested feature selection drift, all `SIFR-RUST-CARGO-0001`, 0.78 s |
| `constrained_modes_reject_manifestless_check_build_and_run` | pass (non-ignored) |
| `sifr_package --lib` | **142/142** |
| `sifr_driver --lib` | **435 passed, 61 ignored** |
| `cargo clippy --workspace -- -D warnings` (AGENTS.md cmd) | **exit 0**, 0 errors |
| `cargo fmt --check`, `git diff --check` | exit 0 |
| compatibility / tiers / stable-claims / stale-drafts / fixture-matrix | all exit 0 — `claims=34`, `tiers=5 fixtures=36`, `fixtures=36 crates=44 package_examples=61 scenario_examples=18` |
| Self-tests | `cases=190`, `cases=6` |
| HIR maintainability, file-size (2998 files, limit 900), TS-Go transfer | PASS |
| Committed-blob counts | **68 passing / 4 planned** exactly; 21 supported / 12 bridge / 1 unsupported / 2 future-owned |

Two initial failures were environmental, not defects: `-p sifr --lib` ran 0 tests (tests live in the `bin` target and are `#[ignore]`d), and `check_fixture_matrix.py` failed in the exported tree because `git archive` omits the `third_party/ruff` submodule. Both passed once corrected.

### Substantive verification

- **Evidence binding is genuine, not fabricated.** `diagnostic_test_sink::record` is called from inside the real `render_diagnostics` (`diagnostic_rendering_and_run.rs:59`) on the same `canonical_diagnostic_stream` that reaches stderr. `assert_drift_rejected` asserts `EXIT_USER_DIAGNOSTIC`, *exactly one* diagnostic, `RUST_CARGO_METADATA`, a case-specific reason, and lock byte-identity — and runs `move_rust_declarations_to_imported_module` first, so B2 can't regress silently.
- **Cargo flag propagation is complete.** All four production `cargo` spawns are instrumented — `cargo/load.rs:62`, `cargo_resolution.rs:192`, `rust_interop_probe.rs:142`, `materialize.rs:284` — each recorded immediately before `.output()` with no intervening mutation (verified by reading each site). Remaining spawns are test-support or the `sifr test` runner, outside check/build/run scope. `assert_cargo_invocations_preserve_lock_modes` panics on an unclassified phase, requires per-mode `final-build` *and* `package-metadata`, ties `--offline` to `is_network_disallowed()`, and requires frozen strength on probes.
- **Authority precedence is fail-closed.** `validate_authoritative_registry_entries:393-403` enforces that if the *primary* (package) lock owns a name, the entry must be in the primary — the sysroot may only authorize remaining names. Validation runs on **both** the cold-prepare and cache-restore paths (`:84-88`, `:100-104`), so the prepared-lock cache is an optimization, never a trust boundary. Missing/empty authority → error (`:59-71`). `assert_unchanged` enforces byte-identity in every constrained mode.
- **Cache identity** normalizes path-dependency roots to `sifr-path-dependency-manifest:{digest}` at `v5` while retaining prefix args, per-authority digests, and vendor roots; publication is temp+rename. Constrained probes use a separate `lock-constrained` bucket (`rust_interop_probe_cache.rs:57-60`).
- **Provenance is real.** `suite_id: sifr_cli_generated_builds` is a **blocking** entry in `merge.json:71` (and create-pr/release/nightly) whose command is exactly `test -p sifr --bin sifr -- --ignored --test-threads=1` — the suite that actually executes these ignored tests. Both referenced test names exist at `cargo_lock_mode_certification_tests.rs:172` and `:357`.
- **Fixture is honest.** `locked_bridge/src/lib.rs:2` genuinely calls `indexmap::IndexMap`, so scenario tokens pin executable source, not prose; `Cargo.toml` pins `=2.14.0, default-features = false`. READMEs correctly attribute network denial to `--frozen`.
- **Docs don't overclaim** — `docs/rust-interop.mdx` explicitly scopes the claim to the exact pinned dependency and states it "does not claim that an uncached package can resolve in offline or frozen mode." The TS-Go transfer inventory adds a row for every new filesystem read.
- **Maintainability**: file-size PASS; largest touched files `cli_model_and_entrypoint.rs` (888) and `entrypoint.rs` (885) remain under the 900 cap.
- All 51 review-artifact links in the issue doc resolve against the committed tree, including cert-11 rounds 1 and 2 and cert-10 rounds 4 and 5. The final checklist box is correctly still unchecked — it closes at merge.

### Non-blocking (carry forward, no remediation required now)

1. `drift.rs:219` `feature_selection_seen` is a global OR consulted only as the last fallback for Cargo's ambiguous `"stale lockfile or feature/source drift"` stderr. It can mislabel an otherwise-unclassified stale lock as feature drift when any dependency declares features. The certified guarantee is the stable **code** `SIFR-RUST-CARGO-0001`, which is unaffected, and the negative test shows all five reasons remain distinct in practice.
2. `drift.rs` (229 lines) still has no fast unit tests — bound to evidence only through the `--ignored` negative test.
3. Prepared-lock store has no eviction (`cargo_resolution.rs:317`); entries are tiny and now key-stable.
4. `internal_docs/rust_interop_architecture.md` loosely says Sifr seeds "from the package lock when one is present, otherwise from the sysroot lock" — the code seeds from the lowest-priority authority and overlays. The next sentence states the correct net effect.

I did not run the full authoritative `scripts/run_all_tests.sh`; per your instruction, merge readiness is a separate review after the PR is marked ready. Draft state is acceptable here and I made no file modifications.
