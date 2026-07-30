Verification complete. Here is my review.

---

# Rust Interop certification_12 — Implementation Review (CLI and Tooling Ecosystem Bridge)

## Verdict

**NOT SATISFIED**

One blocking finding: the staged tree regresses the mandatory 900-line file-size guardrail, which is a blocking step in core validation. Every other element of the acceptance contract verified green, including both mandatory generated-package tests, which I ran.

---

## Scope isolation proof

Candidate = `git write-tree` → `c69c7a1256bfa6d277b4ca9e2f1fd13dbc41d3ec` (37 paths vs `HEAD` `d5a4b294d`).

| Excluded item | Staged? | Evidence |
| --- | --- | --- |
| `ecosystem_backend_certification` → `supported` promotion | **No** | The only unstaged content hunk; `git diff` shows it worktree-only. Staged row remains `"category": "future-owned-by-separate-phase"` with `future_owner` intact (compat matrix `:396-402`). |
| `editor_integrations` submodule | **No** | unstaged `M` only |
| `leetcode` corpora submodule | **No** | untracked only; absent from `git diff --cached --name-status HEAD` |
| `.cert5probe/`, `.claude/` | **No** | untracked |
| `logo*.webp` images | **No** | untracked |
| `plans/phases/43_interoperability.md` | **No** | untracked |

I extracted the candidate independently (`git archive c69c7a1 | tar -x -C /tmp/cert12stage`) and diffed it against the live worktree: the **only** content difference is the excluded backend hunk (`1 insertion, 2 deletions`). All validators below were run against that extracted staged tree, not the live worktree.

The staged CLI hunk (in scope) promotes exactly one row and nothing else.

Staged-tree category/evidence recount (independent of the checkers): `supported 21 / supported-through-bridge 13 / unsupported-by-design 1 / future-owned-by-separate-phase 1`, `70 passing / 2 planned`. Matches the plan's claim exactly.

---

## Blocking findings

### B1 — File-size guardrail regression: `check_fixture_matrix.py` 899 → 904 lines

`verification/areas/rust_interop/checks/check_fixture_matrix.py:717-723`

The staged hunk replaces a one-line conditional expression with a five-branch `if/elif/else`:

```python
717	    crate_token = crate.replace("-", "_")
718	    if fixture_id == "proc_macro_trust":
719	        binding_token = "bridge.generated"
720	    elif fixture_id == "ecosystem_cli_certification":
721	        binding_token = "bridge.cli"
722	    else:
723	        binding_token = crate_token
```

Net `+5` lines pushes the file from **899** (at `HEAD`, passing) to **904** (limit 900).

Evidence:
- `git show HEAD:…/check_fixture_matrix.py | wc -l` → `899`
- `git show c69c7a1:…/check_fixture_matrix.py | wc -l` → `904`
- On the extracted staged tree: `python3 scripts/check_file_size_guardrails.py` →
  `file-size guardrails: FAIL` / `check_fixture_matrix.py: 904 lines (limit 900, category python-verification)`, real exit code **1**.

This is not advisory. `verification/policy/guardrails.json:13` registers it, and `verification/runner/sifr_verify/profile_runner.py:404-408` invokes it inside `run_core_guardrails` via `run_python`, which raises on non-zero exit — so `scripts/run_all_tests.sh` fails on the staged tree. `AGENTS.md` also names this cap as a hard rule ("If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module").

The prior-evidence claim that the "file-size … check passes" is **not reproducible**; it fails deterministically both in the live worktree and on the exact staged tree, and it is caused by this change (not pre-existing).

**Required fix:** move the fixture→binding-token selection out of `check_fixture_matrix.py` — e.g. a `FIXTURE_BINDING_TOKENS = {"proc_macro_trust": "bridge.generated", "ecosystem_cli_certification": "bridge.cli"}` mapping in the existing `_binding_helpers.py`, reducing the call site to `binding_token = FIXTURE_BINDING_TOKENS.get(fixture_id, crate_token)`. Then re-run `scripts/check_file_size_guardrails.py` and confirm exit 0.

---

## Contract items verified (all green)

**Exact pins under authoritative lock.** `Cargo.toml:5-9` pins `=1.0.102 / =4.6.1 / =0.1.44 / =0.3.23` with `features = ["env-filter"]`. Fixture `Cargo.lock` carries real registry entries with checksums (`anyhow:65-68`, `clap:83-86`, `tracing:340-343`, `tracing-subscriber:383-386`). I recomputed the root-lock subset independently: **48 external packages, 0 missing** on exact `(name, version, source, checksum)` identity; only 3 local packages (`cli-feature-package`, `sifr-anyhow-surface-probe`, `sifr_runtime`); sole source is crates.io; no `[patch]`/`[replace]` in either lock. Root `Cargo.lock` carries the same four versions (`:127`, `:1057`, `:8196`, `:8240`), so offline resolution is not a new untracked machine dependency.

**No shadow crates.** The four fake stubs (`rust/anyhow`, `rust/clap`, `rust/tracing`, `rust/tracing_subscriber`) are deleted in the staged tree; `git ls-tree` on the candidate shows `rust/` contains only `anyhow_surface`. That crate is distinctly named `sifr-anyhow-surface-probe` and depends on the real workspace `anyhow` — a legitimate surface holder, not a masquerade.

**Real execution.** `src/bridges/cli.rs:37-84` builds a real `clap::Command` with `value_parser(["check","build"])`, installs a real `tracing_subscriber::fmt` layer with `EnvFilter::try_new("sifr_cli_probe=trace")` over a capturing `MakeWriter`, emits the event, reads the captured bytes back, and `anyhow::ensure!`s the event was observed. `EnvFilter` is only in scope because `env-filter` is enabled, so the feature is compile-load-bearing, not decorative.

**anyhow stays internal.** `execute_cli_probe` returns `anyhow::Result<String>`; `parse_and_trace:21-29` collapses it into `CliErrorBridge` before the boundary. `main.sifr:6-10` / `positive/…:6-10` declare `Result[str, CliError | RustPanicError]` — no `anyhow` type crosses.

**Both mandatory tests execute and pass — I ran them:**
```
cargo test -p sifr_driver --lib -- --ignored --test-threads=1 cli_ecosystem_support
test …cli_ecosystem_support::test_build_cli_tooling_probe_and_anyhow_adapter ... ok
test …cli_ecosystem_support::test_check_direct_anyhow_surface_rejected ... ok
test result: ok. 2 passed; 0 failed; finished in 38.32s
```
The positive test asserts the full versioned marker `clap=4.6.1;mode=check;tracing=0.1.44;subscriber=0.3.23;env-filter=enabled;event=observed;anyhow=1.0.102;adapter=CliError`, asserts the `--mode invalid` path surfaces only `clap parse failed through the anyhow adapter`, asserts empty stderr, and independently gates the feature graph via `cargo tree --workspace --edges features --locked --offline` requiring `tracing-subscriber feature "env-filter"` (`…cli_ecosystem_support.rs:100-131`).

**Negative direction is correctly isolated.** `…cli_ecosystem_support.rs:66-96` first proves the explicit adapter is *accepted*, then swaps in the direct surface and requires (a) `RUST_TYPE_PROBE_FAILURE` on `main.expose_anyhow_error`, (b) rendered evidence naming `anyhow_surface::direct_error` **and** `anyhow::Error`/`anyhow :: Error` — the spaced form is rustc's own pretty-printer, so this is real signature-probe output, not a hardcoded table — and (c) `all(code != RUST_TRUST_MISSING)`, which directly discharges "without a trust diagnostic masking it". `DiagnosticCode::RUST_TYPE_PROBE_FAILURE` is the `SIFR-RUST-TYPE-0001` code (`crates/sifr_driver/src/build/rust_interop.rs:849/857/862`; asserted equal to the literal string in existing contract tests).

**Merge profile provenance is real, not nominal.** `fixture.json` binds both sides to `suite_id: sifr_driver_generated_builds`, `step: crate_tests`, `profile: merge`, with correct `test_file`/`test_name`. That suite is `"status": "blocking", "executed_in_merge": true, "modes": ["full"]` with command `["test","-p","sifr_driver","--lib","--","--ignored","--test-threads=1"]` in `verification/profiles/merge.json:73` (also `create-pr.json:90`, `nightly.json:75`, `release.json:74`). `create-pr` runs `crate_tests` at `smoke`, so `merge` is correctly the weakest executing profile for these `#[ignore]`d tests. `_provenance_checks.py` rejects README-only claims, and `validation` blocks are present on both passing sides.

**Validator and mutation coverage.** All checkers pass on the extracted staged tree:
```
check_compatibility_matrix.py  → rows=36 fixture_rows=36 categories=4
check_fixture_matrix.py        → fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18
check_stable_support_claims.py → claims=35
check_tiers.py                 → tiers=5 fixtures=36
check_stale_drafts.py          → ok
--self-test: 206 / 5 / 33 / 6 cases, all ok
```
`_scenario_checks.run_self_test()` → `(114, None)`. `_scenario_cli.py` contributes 15 mutations + 1 baseline, and I confirmed each mutation is genuinely load-bearing: all four exact pins, `env-filter` removal, workspace membership, wrapper path, both trust lists, bridge path, and four bridge/source token drifts (`clap::Command::new`, `EnvFilter::try_new`, the observed-event assertion, the `clap parse failed` context) plus `-> anyhow::Error` on the surface crate. Mutations are applied to a `tempfile` copy and restored — no checked-in edits.

**Docs match structurally.** The `docs/rust-interop.mdx` generated claims table is 35 rows in exact `stable_support_claims.json` order (script-verified, including the new `ecosystem_cli_certification | supported-through-bridge | cargo-probe` row). `internal_docs/rust_interop_architecture.md:1216-1235`, `plans/phases/39_rust_interop.md:339-345`, both fixture READMEs, and the issue-plan checklist all state the same scope, explicitly disclaim direct support for arbitrary CLI/`anyhow` APIs, and explicitly record backend certification as separately owned. No overclaim found. Every count the issue plan asserts (36/44/61/18, 206, 70+2, 21/13/1/1, 35 claims) I reproduced exactly.

**No shortcuts.** Zero `unwrap()`/`expect(`/`panic!`/`unsafe`/`todo!` in the fixture's `rust/` and `src/` trees (grep clean); lock poisoning is handled via `map_err` into `anyhow`/`io::Error::other`. No network fallback path. Other core guardrails pass: HIR, driver-maintainability, source-crate dependency direction, submodule ownership. `cargo fmt --check -p sifr_driver` clean. `cargo clippy --workspace -- -D warnings` (the documented gate) clean — the failures under the non-gate `--tests` flag are all in pre-existing files (`cargo_resolution.rs`, `project_build_check.rs`, etc.) and none in the new test file.

---

## Non-blocking observations

1. **Per-crate examples are now degenerate duplicates.** `examples/clap.sifr`, `tracing.sifr`, `tracing-subscriber.sifr`, and `anyhow.sifr` all bind the identical `bridge.cli.parse_and_trace` with identical `["sifr","--mode","check"]` args, differing only in wrapper-function name. The `binding_token = "bridge.cli"` override at `check_fixture_matrix.py:720-721` is what permits this, so nothing now ties `clap.sifr` to clap. This is *honest* (the old stubs claiming `clap.Command.new` were the overclaim) but per-crate example coverage is now nominal, and `anyhow_context(args)` in `anyhow.sifr` is a misleading name for a CLI parse. Consider four distinct bridge entry points, one per crate concern.

2. **Env-filter exclusion is asserted only positively.** The bridge proves a matching event *is* observed but never proves a non-matching target is *excluded*. `EnvFilter::try_new("sifr_cli_probe=trace")` with no global directive does disable other targets, so the filter is genuinely active — but a second event at an excluded target with an `!trace.contains(...)` assertion would make "filtered" load-bearing rather than inferred.

3. **`direct-crate-bindings = true` is not covered by a `sifr.toml` mutation.** `validate_cli_scenario` pins `[rust] bridges` and the full `[trust]` table but not this key. Low risk — the negative driver test would fail with a different diagnostic if it were dropped.

4. **Branch is one commit behind `origin/main`.** `HEAD` (`d5a4b294d`) lacks `b3f663a17` (#3074), which touches `crates/sifr_type_system/src/types/type_rendering.rs` (118 lines). I do not believe it affects the rustc-sourced `anyhow::Error` rendering the negative test matches on, but the two mandatory tests were verified against a base that is not the merge target. Re-run them after rebase.

5. **`plans/reviews/active/rust-interop-certification-11-review-round-5.md`** is a certification-11 closeout artifact landing on the certification-12 branch. It is referenced by the staged issue-plan diff, so it is coherent, but it is not certification-12 work.

6. **Pre-existing (out of scope, do not fix here):** `internal_docs/sifr_sysroot_and_stdlib_architecture.md:915-916` says `opaque_resource_matrix` "remains future-owned by separate certification work", but that row is `supported-through-bridge` in both `HEAD` and the staged tree. Stale since `12b64b4f89`, untouched by this change.

---

## Required fixes before SATISFIED

1. Resolve **B1**: refactor the `binding_token` selection out of `check_fixture_matrix.py` so the file returns under 900 lines, then confirm `python3 scripts/check_file_size_guardrails.py` exits 0 on the staged tree and re-run `check_fixture_matrix.py` plus `--self-test` (expect `fixtures=36 …` and `cases=206`).
2. Re-run the two `#[ignore]`d generated-build tests after the refactor and after rebasing onto `origin/main`.

## Commands run

```
git rev-parse HEAD; git write-tree; git merge-base HEAD origin/main
git diff --cached --name-status HEAD ; git diff --name-status ; git status --porcelain
git archive c69c7a1 | tar -x -C /tmp/cert12stage      # exact candidate
git diff --no-index <staged>/…compatibility_matrix.json <worktree>/…   # 1 hunk = excluded backend only
(staged tree) check_{compatibility_matrix,fixture_matrix,stable_support_claims,tiers,stale_drafts}.py [--self-test]
(staged tree) python3 -c "_scenario_checks.run_self_test()"            → (114, None)
(staged tree) category/evidence recount; root-lock subset recount (48 external, 0 missing)
cargo test -p sifr_driver --lib -- --ignored --test-threads=1 cli_ecosystem_support   → 2 passed
cargo clippy --workspace -- -D warnings ; cargo fmt --check -p sifr_driver
scripts/check_file_size_guardrails.py                                  → FAIL (exit 1)
scripts/check_{hir_maintainability,sifr_driver_maintainability,source_crate_dependency_direction,submodule_ownership}_guardrails.py → PASS
```

No repository files were modified, staged, or created.
