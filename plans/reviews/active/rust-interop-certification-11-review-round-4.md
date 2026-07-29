Worktree is byte-identical to how I found it; I modified nothing.

# Merge-readiness review — Sifr rust-interop `certification_11` (PR #3075)

## Verdict: `SATISFIED`

Exact head `68c5f1a43091ddac40aa230d76d76bca308dc7fc` (PR #3075, base `main`, `MERGEABLE` / `mergeStateStatus: CLEAN`, **not** a draft, 2 commits, 70 files, +3430/−519) is ready to merge. No blocking findings.

---

## Blocking findings

**None.**

Prior blockers, re-verified individually at this head:

| Round-1 blocker | Status at head — independently confirmed |
|---|---|
| **B1** `cargo clippy --workspace -- -D warnings` failed | **FIXED** — exit 0, zero warnings |
| **B2** `SIFR-RUST-CARGO-0001` classification came from a lexical scan of the *entry file only* | **FIXED** — `package_graph_context.rs:26-45` derives the boundary hint from the parsed manifest (`SifrManifest::declares_rust_backend`), not source text. The negative test calls `move_rust_declarations_to_imported_module` first, so a regression to lexical entry-file scanning would fail the test. |
| **B3** negative test asserted a *directly invoked* classifier, not the command's emitted diagnostic | **FIXED** — `assert_drift_rejected` now asserts on `diagnostic_test_sink::capture(...)`. The sink's `record` is called inside the real `render_diagnostics` (`diagnostic_rendering_and_run.rs:59`) on the same `canonical_diagnostic_stream` that reaches stderr. No direct classifier call remains in any test. |

Round-1 non-blocking N1–N8 are resolved or consciously dispositioned (N3 authority precedence, N4 kind-specific reasons, N5 manifestless rejection, N2 scenario token, N6 file-size splits all genuinely fixed; N7/N8 documented — see observations).

## Required-scrutiny results

**Isolation of the unstaged parallel-agent edit — PROVEN.** The sole `run_all_tests.sh --profile create-pr` failure is fully attributed and is not a PR defect:

| Tree | `check_compatibility_matrix.py` |
|---|---|
| **Exported committed HEAD** (`git archive 68c5f1a43`) | `rows=36 fixture_rows=36 categories=4` — **exit 0** |
| **Live worktree** (unstaged hunk present) | **exit 1** — `ecosystem_backend_certification: supported rows require passing positive and negative fixture evidence` |

The unstaged diff is exactly one hunk at `@@ -396,8` promoting `ecosystem_backend_certification` to `supported` and deleting `future_owner`, while both its evidence directions remain `planned`. In the committed blob at PR head that row is still `future-owned-by-separate-phase` with `future_owner` present, tier 4, both directions `planned`. `git diff 3c9601d26 HEAD` on the matrix contains only the `cargo_locked_offline` promotion. Every other rust-interop checker passes in the live worktree too, so the backend hunk is the single cause. No excluded artifact leaked into the commit (no `*.webp`, `.cert5probe/`, `.claude/`, `editor_integrations`, leetcode corpus, `43_interoperability.md`, or cert-11 round-4 draft).

**Committed-matrix counts reproduce exactly as documented:** 68 `passing` / 4 `planned`; 21 supported, 12 supported-through-bridge, 1 unsupported-by-design, 2 future-owned; 36 fixtures, 44 crates, 61 package examples, 18 scenario examples.

**Exact Cargo argv propagation.** All four resolution-mode-bearing spawns are instrumented immediately before `.output()` with no intervening mutation: `cargo/load.rs:62` (`package-metadata`), `cargo_resolution.rs:192` (`resolution`), `rust_interop_probe.rs:142` (`rust-probe`), `materialize.rs:284` (`final-build`). I audited every remaining `.output()`/`.status()` in the package/driver Cargo paths myself — each is a `cargo -V`/`rustc -Vv`/`--version` toolchain-signature probe, a Python probe, or test-only code, none of which carries resolution semantics. `assert_cargo_invocations_preserve_lock_modes` **panics on an unclassified phase**, requires per-mode `final-build` *and* `package-metadata`, ties `--offline` to `is_network_disallowed()`, and requires frozen strength on probes — so it cannot silently pass on a new uninstrumented phase.

**Real emitted diagnostics, five distinct drift causes.** The mandatory negative test's actual output, observed in my run: `missing lockfile`, `stale selected version`, `checksum drift`, `dependency source drift`, `requested feature selection drift` — each `SIFR-RUST-CARGO-0001`, exactly one diagnostic per case, `EXIT_USER_DIAGNOSTIC`, lock byte-identical (or absent) after each.

**Lock authority precedence is fail-closed.** `entrypoint_resolution.rs:15-30` orders package lock first, sysroot second. `seed_lockfile_from_authorities` seeds from the lowest-priority authority and overlays higher-priority registry packages last, so package pins win. `validate_authoritative_registry_entries:392-403` enforces that if the *primary* owns a name, the exact entry must be in the primary — the sysroot may only authorize remaining names. Validation runs on **both** the cold-prepare and cache-restore paths, so the prepared-lock cache is never a trust boundary. Missing/empty authority is a hard error.

**Cache identity and atomic publication.** Key = `sifr-cargo-resolution-v5` + manifest with path-dependency roots normalized to `sifr-path-dependency-manifest:{digest}` + prefix args + per-authority digests + vendor roots. Publication is copy-to-temp + `rename`, tolerating a concurrent winner. Constrained probes use a separate `lock-constrained` cache bucket, so a `Normal` probe result can never satisfy a `--frozen` request.

**Constrained manifestless behavior fails closed.** Verified for all three commands: `sifr check/build/run --frozen requires a package Cargo.lock`, code `SIFR-RUST-CARGO-0001`, exit `EXIT_USER_DIAGNOSTIC`. `--locked --offline` correctly normalizes to `Frozen` without dropping `--locked`.

**Extra check beyond prior rounds.** The certified negative direction binds only to `sifr check`, so I independently tested `build` and `run` under real stale-version drift in a temp copy of the fixture. All three emit `SIFR-RUST-CARGO-0001: stale selected version` — no silent resolver fallback on any command. No gap.

## Commands and evidence

| Gate | Result |
|---|---|
| Mandatory positive + negative (`-p sifr --bin sifr -- --ignored`) | **2 passed**, 95.44 s |
| `constrained_modes_reject_manifestless_check_build_and_run` + flag-parse test | **2 passed**, 0.02 s |
| `cargo test -p sifr_package --lib` | **142 passed, 0 failed** |
| `cargo test -p sifr_driver --lib` | **435 passed, 0 failed**, 61 ignored |
| `cargo clippy --workspace -- -D warnings` | **exit 0** |
| `cargo fmt --check` / `git diff --check` | exit 0 / exit 0 |
| compatibility / tiers / stable-claims / stale-drafts / fixture-matrix (committed HEAD) | all exit 0 |
| Self-tests | matrix `cases=5`, fixture `cases=190`, tiers `cases=6`, claims `cases=33` |
| `check_file_size_guardrails.py` | PASS (2998 files, limit 900); largest touched `cli_model_and_entrypoint.rs` 888, `entrypoint.rs` 885 |
| HIR + sifr_driver maintainability, docs error-code links, submodule ownership | PASS |
| TypeScript-Go transfer guardrails | PASS (inventory row added for the new Cargo-lock reads) |
| Drift fail-closed for `build`/`run` (my own extra test) | `SIFR-RUST-CARGO-0001` on all three commands |

**Provenance is real, not decorative.** `suite_id: sifr_cli_generated_builds` is a **blocking** entry in `merge.json:71`, `create-pr.json:88`, `release.json:72`, `nightly.json:73`, with command exactly `test -p sifr --bin sifr -- --ignored --test-threads=1` — the suite that actually executes these `#[ignore]`d tests. Both referenced test names exist. The `rust_interop_checks` / `compatibility-matrix` step is blocking in `create-pr`, consistent with the reported single-lane failure.

**Stable claims are not overclaimed.** One new claim row (`cargo_locked_offline`, `supported`, `cargo-probe`), 34 total. Docs scope the guarantee to the exact pinned dependency and state that `--offline`/`--frozen` require every selected source to be present in the applicable cache or vendor tree. The scenario token `IndexMap::<String, u32>::new()` pins genuinely executable source (`locked_bridge/src/lib.rs:2`), not prose.

## Non-blocking observations

1. **`drift.rs:219` `feature_selection_seen` is a global OR**, consulted only as the last fallback for Cargo's ambiguous `"cannot update the lock file"` stderr. It could mislabel an otherwise-unclassified stale lock as feature drift when any dependency declares features. The certified guarantee is the stable *code* `SIFR-RUST-CARGO-0001`, which is unaffected, and all five reasons are empirically distinct. Carried forward from round 3.
2. **`drift.rs` (229 lines) has no fast unit tests** — bound to evidence only through the `--ignored` negative test, so a regression surfaces slowly.
3. **Prepared-lock store has no eviction** (`cargo_resolution.rs:317`). Entries are tiny and the key is now path-stable, so growth is bounded in practice.
4. **N8 disposition is a deliberate strictness choice, documented in code** (`cargo_resolution.rs:149-152`): standalone `--offline` *can* legitimately update a source lock, but generated interop projects treat all constrained modes as byte-immutable because the prepared lock is a validated cache artifact. Sound, and worth keeping visible.
5. **`internal_docs/rust_interop_architecture.md`** loosely says Sifr seeds "from the package lock when one is present, otherwise from the sysroot lock"; the code seeds from the lowest-priority authority and overlays. The next sentence states the correct net effect.
6. `cli_model_and_entrypoint.rs` (888) and `entrypoint.rs` (885) sit within 15 lines of the 900 cap — the next touch on either will need a split.

## Merge recommendation

**Merge.** PR #3075 is ready, mergeable, and scope-clean. Every certification_11 acceptance criterion is backed by evidence I reproduced independently rather than taking on trust: real emitted diagnostics, exact argv capture across all four Cargo phases, fail-closed authority precedence validated on both cache paths, five genuinely distinct drift causes, and constrained manifestless rejection on all three commands. The single `create-pr` lane failure is entirely caused by an unstaged parallel-agent edit that is provably absent from the commit, and the committed matrix passes with the documented counts exactly.

Two post-merge follow-ups, neither gating: the final checklist box in the issue doc closes at merge (correctly still unchecked), and observations 1–2 above are worth a small hardening item to give `drift.rs` fast unit coverage and a kind-specific fallback.
