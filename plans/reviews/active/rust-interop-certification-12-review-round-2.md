## Rust Interop certification_12 — Round 2 Exact-Head Review (CLI/Tooling Ecosystem Bridge)

## Verdict

**SATISFIED**

Round 1's sole blocking finding (B1) is genuinely fixed at the root, the post-round-1 hardening is real and load-bearing, the merge into current `origin/main` preserved the certification diff bit-for-bit with zero unrelated content, and both mandatory ignored tests pass on the integrated head.

---

## Head resolution and merge integrity

| Item | Value |
| --- | --- |
| Reviewed head | `e2c321a788142bdf0da02967efee076c985a3d7c` |
| Parents | `6afd74646d…` (cert commit) + `b3f663a174d170a99656e3221ffd952b81c4d51c` |
| `origin/main` | `b3f663a174d170a99656e3221ffd952b81c4d51c` — **identical to parent 2**, so merge base is the exact merge target |

Nothing was lost and nothing leaked in the merge — proven two ways:

- `git diff origin/main..HEAD` and `git diff 6afd74646^..6afd74646` produce **byte-identical patches** (`diff /tmp/p1.diff /tmp/p2.diff` → no output) over the **same 39 paths**.
- `git diff --name-only 6afd74646..HEAD` returns exactly the 17 files of `b3f663a17` (#3074) and nothing else — no parallel work entered via the merge.

The 39-file diff is entirely `verification/areas/rust_interop/**`, the new `sifr_driver` test module + its 2-line `mod` registration, and docs/plans. No compiler-crate behavior change.

## B1 verification — fixed at the root

`check_fixture_matrix.py:719` now reads `binding_token = package_example_binding_token(fixture_id, crate_token)`; the fixture→token table moved to `verification/areas/rust_interop/checks/_binding_helpers.py:7-15` as `FIXTURE_BINDING_TOKENS` + a single accessor. Net effect on the checker is `+1` line (the import).

```
git show HEAD:…/check_fixture_matrix.py | wc -l          → 900   (limit is `lines > 900` → fail)
python3 scripts/check_file_size_guardrails.py            → file-size guardrails: PASS (3005 files, limit 900 lines), exit 0
python3 scripts/check_hir_maintainability_guardrails.py           → PASS
python3 scripts/check_sifr_driver_maintainability_guardrails.py   → PASS
```

The gate passes on both the live tree and the extracted exact head. The refactor is a real responsibility move (fixture-specific policy now lives with the other binding helpers), not line-shuffling.

## Post-round-1 hardening — verified load-bearing

**Excluded-target event now exists and is proven excluded.** `src/bridges/cli.rs:63-86` emits `tracing::info!(target: "sifr_cli_probe", …)` **and** `tracing::warn!(target: "sifr_cli_noise", …)` inside `with_default`, captures the subscriber output through a `MakeWriter`, then asserts:

```rust
anyhow::ensure!(
    trace.contains("cli bridge event") && trace.contains(mode)
        && !trace.contains("excluded bridge event"),
    "filtered tracing event was not observed"
);
```

`EnvFilter::try_new("sifr_cli_probe=trace")` carries no global directive, so `sifr_cli_noise` is off — and the assertion now *proves* it rather than inferring it. This addresses round-1 observation 2 exactly.

**Both requested mutations exist** (`_scenario_cli.py`):
- `"tracing exclusion drift"` (`:247-253`) flips `!trace.contains("excluded bridge event")` → `trace.contains(…)` and requires the failure.
- `"direct binding policy drift"` (`:198-204`) flips `direct-crate-bindings = true` → `false` and requires `_scenario_checks.py:406`'s `must enable [rust] direct-crate-bindings`. This addresses round-1 observation 3.

Mutation counts moved consistently: `_scenario_checks.run_self_test()` → `(116, None)` (was 114), fixture self-test `cases=208` (was 206). Round 1 *predicted* 206; 208 is the correct post-hardening figure and matches every doc claim.

## Mandatory ignored tests — re-run on the integrated head

```
cargo test -p sifr_driver -- --ignored --exact \
  tests::package_project_build_check::rust_interop_build_tests::cli_ecosystem_support::test_build_cli_tooling_probe_and_anyhow_adapter \
  tests::package_project_build_check::rust_interop_build_tests::cli_ecosystem_support::test_check_direct_anyhow_surface_rejected

test …::test_check_direct_anyhow_surface_rejected ... ok
test …::test_build_cli_tooling_probe_and_anyhow_adapter ... ok
test result: ok. 2 passed; 0 failed; finished in 23.68s
```

(Round-1 observation 4 discharged: verified against a base that *is* the merge target.)

## Acceptance contract re-audit

**Exact lock / root-lock subset.** Independently recomputed on the extracted exact head: **48 external packages, 0 missing** from root `Cargo.lock` on full `(name, version, source, checksum)` identity; local packages are only `cli-feature-package`, `sifr-anyhow-surface-probe`, `sifr_runtime`. Enforcement is wired generically — `_scenario_checks.py:421-429` calls `require_root_lock_subset` for every scenario, and `_scenario_lock_checks.py:41-67` compares source **and** checksum, not just version.

**Real upstream crates, no shadow stubs.** The four fake local crates (`rust/anyhow`, `rust/clap`, `rust/tracing`, `rust/tracing_subscriber`) are deleted; `rust/` contains only `anyhow_surface`. Fixture lock carries genuine crates.io checksums (`anyhow 1.0.102` `7f202df8…`, `clap 4.6.1` `1ddb117e…`, `tracing 0.1.44` `63e71662…`, `tracing-subscriber 0.3.23` `cb7f578e…`), all matching root lock. `assert_exact_cli_dependency_graph` gates the graph with `cargo tree --workspace --edges features --locked --offline`, requiring `tracing-subscriber feature "env-filter"`.

**anyhow boundary.** `execute_cli_probe` → `anyhow::Result<String>` internally; `parse_and_trace:21-29` collapses it into `CliErrorBridge` before the boundary. Sifr signatures declare `Result[str, CliError | RustPanicError]` — no anyhow type crosses. `--mode invalid` surfaces only `"clap parse failed through the anyhow adapter"`, asserted by the positive test.

**SIFR-RUST-TYPE-0001 negative evidence.** The negative test first proves the adapter is *accepted*, then swaps in `anyhow_surface::direct_error` → `anyhow::Error`, requiring (a) `RUST_TYPE_PROBE_FAILURE` on `main.expose_anyhow_error`, (b) rendered rustc evidence naming both `anyhow_surface::direct_error` and `anyhow::Error`/`anyhow :: Error`, and (c) `all(code != RUST_TRUST_MISSING)` — so the diagnostic isolates representation, not trust. The negative overlay declares `rust-no-panic = ["anyhow_surface.direct_error"]` precisely so trust cannot mask it.

**Provenance.** `fixture.json` binds both directions to `suite_id: sifr_driver_generated_builds`, `step: crate_tests`, `profile: merge`, with the real file/test names. That suite is `"status": "blocking", "executed_in_merge": true` with `["test","-p","sifr_driver","--lib","--","--ignored","--test-threads=1"]` in `verification/profiles/merge.json:73` (also `create-pr.json:90`, `nightly.json:75`, `release.json:74`) — `#[ignore]`d tests are genuinely selected.

**Exact committed matrix.** Verified on the exact head, independent of the checkers:

```
rows 36
categories: supported 21 / supported-through-bridge 13 / unsupported-by-design 1 / future-owned-by-separate-phase 1
evidence:   passing 70 / planned 2
ecosystem_backend_certification  → future-owned-by-separate-phase, future_owner intact
ecosystem_cli_certification      → supported-through-bridge
```

**Full area suite on the exact head** (`git archive HEAD | tar -x`, submodules symlinked):

```
check_compatibility_matrix.py   → rows=36 fixture_rows=36 categories=4          (self-test 5)
check_fixture_matrix.py         → fixtures=36 diagnostics=10 crates=44
                                  package_examples=61 scenario_examples=18      (self-test 208)
check_stable_support_claims.py  → claims=35                                     (self-test 33)
check_tiers.py                  → tiers=5 fixtures=36                           (self-test 6)
check_stale_drafts.py           → ok                                            (self-test 20)
runner.py                       → variants=10, failures=0, blocking_failures=0
```

**Docs / claims / counts.** The `docs/rust-interop.mdx` generated table gains exactly one row in `stable_support_claims.json` order and explicitly disclaims direct support for arbitrary CLI crate APIs and `anyhow::Error` values. `internal_docs/rust_interop_architecture.md:1216-1235` and `plans/phases/39_rust_interop.md:339-345` state the same bounded scope and record backend certification as separately owned. Every count the issue plan asserts (36/44/61/18, 208, 70+2, 21/13/1/1, 35) reproduces exactly.

**No shortcuts.** `grep -rE "unwrap\(\)|expect\(|panic!|unsafe|todo!|unimplemented!"` over the fixture's `src/` and `rust/` trees is clean; lock poisoning is handled via `map_err` into `anyhow`/`io::Error::other`. `cargo fmt --check` exit 0; `cargo clippy --workspace -- -D warnings` (the documented gate) clean.

## Scope isolation proof

Every previously enumerated unrelated change is worktree-only. `git cat-file -e HEAD:<path>` → **absent from HEAD** for all of: `.cert5probe`, `.claude`, `plans/phases/43_interoperability.md`, `logo 06.48.53.webp`, `docs/logo/logo.webp 08-03-09-514.webp`, `plans/reviews/active/rust-interop-certification-12-review-round-2.md`.

The parallel-agent promotion is unstaged only. HEAD's committed hunk promotes `ecosystem_cli_certification` **and nothing else**; the unstaged worktree hunk (`ecosystem_backend_certification` → `"supported"`, dropping `future_owner`) is confined to `git diff` and is not in the tree I validated. Submodule pointers (`editor_integrations`, leetcode corpora) are **unchanged vs `origin/main`** at HEAD.

## Non-blocking observations

1. **Exclusion evidence is not itself guarded against deletion.** `_scenario_cli.py:101-111` requires `target: "sifr_cli_probe"` and `!trace.contains("excluded bridge event")` but never requires the `tracing::warn!(target: "sifr_cli_noise", …)` emission. Deleting `cli.rs:69-73` would leave every gate green while the exclusion assertion becomes vacuously true. Adding `'target: "sifr_cli_noise"'` to the token tuple would close this cheaply. The current committed evidence is genuine — this is guard durability, not a live defect.
2. **Per-crate package examples remain degenerate duplicates** (carried from round-1 observation 1, unresolved). `clap.sifr`, `tracing.sifr`, `tracing-subscriber.sifr`, and `anyhow.sifr` all bind the identical `bridge.cli.parse_and_trace` with identical args, differing only in wrapper name — permitted by the new `FIXTURE_BINDING_TOKENS` entry. Honest (the old stubs were the overclaim) but per-crate granularity is nominal, and `anyhow_context(args)` is a misleading name for a CLI parse.
3. **`check_fixture_matrix.py` sits at exactly 900 lines — zero headroom.** The very next line added to that file re-triggers B1. Consider proactively relocating one more responsibility.
4. **`plans/reviews/active/rust-interop-certification-11-review-round-5.md`** is a certification-11 closeout artifact on this branch. It is referenced by the staged issue-plan diff (`:1327-1332`), so it is coherent, but it is not certification-12 work.
5. **`cargo clippy --workspace --all-targets -- -D warnings` fails**, entirely in pre-existing untouched files (`sifr_ipc/src/ipc_connection.rs` and others — none appear in `git diff --name-only origin/main..HEAD`). The documented gate (`--workspace` without `--all-targets`) is clean. Pre-existing, out of scope.
6. **Pre-existing, do not fix here:** `internal_docs/sifr_sysroot_and_stdlib_architecture.md:915-916` still calls `opaque_resource_matrix` future-owned though it is `supported-through-bridge` in both main and this head.

## Commands run

```
git rev-parse HEAD; git log -1 --format='%H %P'; git rev-parse origin/main
git diff --name-status origin/main..HEAD ; diff <(git diff 6afd74646^..6afd74646) <(git diff origin/main..HEAD)
git diff --name-only 6afd74646..HEAD           # == b3f663a17 only
git archive HEAD | tar -x -C /tmp/cert12head   # exact committed head; third_party symlinked
(exact head) check_{compatibility_matrix,fixture_matrix,stable_support_claims,tiers,stale_drafts}.py [--self-test]
(exact head) PYTHONPATH=verification/runner python3 verification/areas/rust_interop/runner.py  → variants=10 failures=0
(exact head) _scenario_checks.run_self_test()  → (116, None)
(exact head) root-lock subset recount → 48 external / 0 missing ; category+evidence recount → 21/13/1/1, 70+2
(exact head) scripts/check_{file_size,hir_maintainability,sifr_driver_maintainability}_guardrails.py → PASS
cargo test -p sifr_driver -- --ignored --exact <both cli_ecosystem_support tests>  → 2 passed
cargo fmt --check ; cargo clippy --workspace -- -D warnings  → exit 0
git cat-file -e HEAD:<each excluded path>      → all absent
```

## Merge / PR recommendation

**Merge.** Head `e2c321a788142bdf0da02967efee076c985a3d7c` is already integrated with the exact merge target, all blocking round-1 findings are resolved at the root, both proactive hardening items landed with real mutation coverage, and the full rust-interop area plus the mandatory generated-package tests pass on the exact committed tree. Open the PR from this head and merge on green.

Two items for the author after merge, neither gating: check off the final plan checklist item (`Run focused and authoritative local gates … merge the PR, and unblock only certification_13`) and land the round-2 review artifact — and note that the worktree's unstaged `ecosystem_backend_certification` promotion must stay out of this PR, as it correctly is.

No repository files were modified, staged, or created during this review; all validation ran against `/tmp/cert12head` (an extract of the exact committed head) except the cargo test/lint runs, whose inputs are untouched by the worktree's unstaged changes.
