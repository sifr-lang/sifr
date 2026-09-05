# Review Pass 3: `hardening_2` (Rust-interop verification matrix hardening)

Scope: current working diff for
`plans/issues/active/rust-interop-verification-matrix-hardening.md` item
`hardening_2`, re-reviewed after the pass-2 fixes. The
`ad-hoc-class-field-mutating-receiver-place-semantics` issue and its review
files are out of scope and were ignored.

## Pass-2 findings: all five resolved

1. **Accurate README wording — resolved.** `same_workspace_crate/README.md:6-15`
   and `shared_bridge_crate/README.md:7-16` now say the positive test "consumes
   the checked-in Cargo workspace, compiles and runs both the positive evidence
   source and the scenario source". That is literally what
   `test_build_*_positive_cargo_probe` does — two `copied_scenario` trees, one
   with `src/main.sifr` overwritten by the evidence file, one left as the
   scenario source, both built and run
   (`package_rust_interop_build_tests.rs:117-144, 178-199`). The scenario
   `main()` bodies now exercise the previously unprobed bindings:
   `workspace_hash.hash_pair` (`examples/workspace_hash_crate/src/main.sifr:14`,
   asserted `!=` in the scenario run) and `digest_hex`
   (`examples/shared_hash_bridge/src/main.sifr:15,20`, asserted
   `"736966722d727573742d696e7465726f70"` and printed as `73696672`).
2. **Explicit `cargo-probe` negative semantics — resolved.**
   `docs/rust-interop.mdx:55` and
   `verification/areas/rust_interop/README.md` §Tier And Execution Semantics
   both now read "positive directions build generated/package Rust code, while
   negative directions may observe a required compiler rejection before Cargo
   execution". The fixture READMEs match ("before a generated build can
   proceed", "before Cargo execution").
3. **Checked-in rejected shared-crate source — resolved, and load-bearing.**
   `shared_bridge_crate/negative/shared_bridge_lib.rs` holds the
   `use crate::__sifr_bridge::app::GeneratedPrivate;` violation and is
   `include_str!`d at `package_rust_interop_build_tests.rs:24`. I confirmed it
   is not decorative: `validate_backend_generated_bridge_imports`
   (`crates/sifr_driver/src/build/rust_interop.rs:545-572`) walks the backend
   crate's `src/` via `first_generated_bridge_import` and emits the
   "package-specific" `SIFR-RUST-RESOLVE-0001` the test asserts. Deleting that
   import from the checked-in file would fail the test.
4. **Negative-only trust manifest — resolved.**
   `shared_bridge_crate/negative/sifr.toml:17` scopes `rust-no-panic` to
   `sifr_shared_hash_bridge.generated_private_type` alone, and the canonical
   scenario manifest (`examples/shared_hash_bridge/sifr.toml:17-20`) is back to
   `digest` and `digest_hex` only — matching `_require_trust_targets`
   (`_scenario_checks.py:258`).
5. **Real checked-in Cargo layouts consumed — resolved in mechanism, broken in
   provenance.** `package_entrypoint_from_cargo_layout`
   (`package_rust_interop_build_tests.rs:59-94`) now shells out to
   `cargo metadata --format-version=1 --locked --offline` against the copied
   fixture `Cargo.toml` and derives the graph through
   `sifr_package::derive_package_graph`, so `package_graph` synthesis is gone
   and the fixture's `path = "rust/workspace_hash"` / `members = [` claims are
   genuinely load-bearing. The negative side installs a dedicated
   `same_workspace_crate/negative/Cargo.toml:4-8` that keeps `workspace_hash` a
   workspace member with no dependency edge — a real Cargo graph, exactly as
   asked. **However, the `Cargo.lock` files this now depends on are not checked
   in** — see findings 1 and 2.

## Gates run in this pass

- `uv run --project verification --locked python -m sifr_verify areas run --area rust_interop`
  → 4 suites, 0 failures (34 fixtures, 34 rows, 10 scenario examples).
- `check_fixture_matrix.py --self-test` (27 cases),
  `check_compatibility_matrix.py --self-test` (3),
  `check_tiers.py --self-test` (6) — all ok.
- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1`
  → 4 passed in 43.5s.
- `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --check` clean;
  `check_file_size_guardrails.py` PASS (2822 files); `git diff --check` clean.
- Not re-run: `scripts/run_all_tests.sh --profile create-pr` and the default
  merge gate. Note that on this machine they would pass, because the missing
  artifacts in finding 1 exist locally as ignored files.

## Actionable findings

### 1. HIGH (blocking) — `sifr_driver` does not compile from a clean checkout

`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:12-14`
`include_str!`s
`verification/areas/rust_interop/fixtures/same_workspace_crate/negative/Cargo.lock`.
That path is matched by `.gitignore:27` (`**/Cargo.lock`, whose only exceptions
are `!/Cargo.lock` and `!/vendor/*/Cargo.lock`), so `git add` will not stage it
and it will never reach the PR. `git ls-files` confirms every fixture
`Cargo.lock` in the repo is untracked.

`include_str!` is a compile-time read, so this is not a test-time failure — the
whole `sifr_driver` test target fails to build. Verified by moving the file
aside:

```
error: couldn't read `.../same_workspace_crate/negative/Cargo.lock`: No such file or directory (os error 2)
  --> crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:12:44
```

That breaks `cargo test -p sifr_driver`, the `crate_tests` step in every
profile, and `cargo clippy --all-targets` for anyone who clones the branch —
not just the `#[ignore]`d evidence.

### 2. HIGH — the scenario Cargo layouts are also unavailable from a clean checkout

Same root cause, distinct failure. `copied_scenario` copies
`fixtures/<id>/examples/<scenario>/` as it exists on disk, then
`package_entrypoint_from_cargo_layout` runs `cargo metadata --locked`. Both
`examples/workspace_hash_crate/Cargo.lock` and
`examples/shared_hash_bridge/Cargo.lock` are likewise gitignored and untracked,
so the copied tree has no lock and `--locked` refuses to create one. Verified
against a tracked-files-only export of the fixture
(`git archive HEAD … | tar -x`):

```
error: cannot create the lock file … because --locked was passed to prevent this
exit=101
```

All four tests fail this way once finding 1 is fixed. The README claim
"consumes the checked-in Cargo workspace" is therefore not yet true of the
lockfile, which is the part `--locked` actually pins.

**Fix for both.** Add a negation to `.gitignore` after line 27 and stage the
locks, e.g.

```
!verification/areas/**/Cargo.lock
```

I verified this negation takes effect (`git check-ignore` reports the negating
rule; the 11 fixture locks become stageable). `**/Cargo.lock` is a file pattern
with no excluded parent directory, so negation is valid here — the existing
`!/Cargo.lock` line is the same construction. Then commit
`same_workspace_crate/negative/Cargo.lock`,
`examples/workspace_hash_crate/Cargo.lock`, and
`examples/shared_hash_bridge/Cargo.lock` at minimum, and re-run the four tests
from a clean export to confirm.

### 3. LOW (pre-existing, not introduced by this diff) — `cargo_locked_offline` has the same latent gap

`_scenario_checks.py:261-263` already hard-requires
`fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`, and
that file is untracked and gitignored too, so the `matrix` suite would fail on
a clean checkout today. Worth folding into the same `.gitignore` fix while it
is being made; it explains why this trap was not caught earlier.

## Exit criteria for `hardening_2`

Re-evaluated against the item's five bullets and the mutation requirement:

- **Frozen table encoded** — `ALLOWED_EXECUTION_KINDS`
  (`check_fixture_matrix.py:123-129`) matches the plan table exactly, enforced
  by `_validate_execution_semantics:245-252`, self-tested two-sidedly across all
  5×4 pairs (`is_allowed == has_pair_failure` fails the run). Met.
- **Real `check_tiers.py --self-test`** — renders mutated TOML/JSON into
  `tempfile.TemporaryDirectory` and re-runs `_load_and_validate` on those paths,
  with a control assertion that unmutated temporary data yields zero failures,
  plus missing/duplicate/mismatch/invalid-name/empty-list cases. Met.
- **`diagnostic_crate_rationale` added and cross-validated** — three
  byte-identical copies for `direct_crate_negative_type` and
  `blocking_diagnostics`; `_validate_manifest_alignment:539-556` covers
  `fixture.json`, `check_compatibility_matrix.py:125` covers the compatibility
  row; `linked` and `executed` pinned to `false`; the shape is validated
  whenever the field is present on a diagnostic row, not only when crates are
  listed. Met.
- **Both diagnostic rows migrated** — met.
- **Tier-1 rows are real `cargo-probe` rows** — met in substance and materially
  stronger than pass 2: the graph now comes from real `cargo metadata` over
  checked-in manifests rather than synthesized `package_graph` data, positive
  and scenario sources both build and run with asserted observed values
  (`1451903697411170458`, `17`, `73696672`), and both negative directions run
  against genuine Cargo graphs — an in-workspace-but-undeclared member for
  `same_workspace_crate`, and a real backend crate whose checked-in source
  carries the boundary violation for `shared_bridge_crate`. Met, **conditional
  on findings 1–2**, since as committed this evidence would not build at all.
- **Docs/matrices/manifests updated in the same change** — fixture matrix,
  compatibility matrix, both `fixture.json`s, all evidence headers, tier
  descriptions (`rust_interop_tiers.toml`), both fixture READMEs, the scenario
  README, `internal_docs/rust_interop_architecture.md:968-1022`, and
  `docs/rust-interop.mdx:49-62`. Consistent; `diagnostic_family` drift is now
  mechanically closed by `_validate_diagnostic_family_alignment:566`. Met.
- **Mutation coverage** — disallowed pair, missing rationale, mismatched
  rationale, malformed rationale with and without crates, rationale on a
  non-diagnostic row, tier-1 downgrade to contract-only, diagnostic-family
  drift. Met.

Claim strength is now honest in both directions: no row asserts more than what
an executed test observes, and the `cargo-probe` negative caveat is stated in
public docs, the area README, and both fixture READMEs rather than only in
fixture prose.

## Informational (not `hardening_2` scope)

`check_stale_drafts.py --self-test` still prints `rust interop stale draft scan
ok` and ignores the flag, giving false self-test signal while listed in this
issue's Required Validation block. `hardening_4` owns the file; passes 1 and 2
both suggested adding an explicit line to that item and the issue text still has
not been updated.

## Verdict

**Not approved.** The design work is done and the pass-2 findings are all
genuinely resolved — finding 5 in particular is fixed more thoroughly than
requested. But the change as it would land does not compile: two required
`Cargo.lock` artifacts are gitignored, so the branch is green only on a working
tree that holds untracked files. Fix findings 1 and 2 (one `.gitignore` line
plus staging the locks), then re-run the four `--ignored` tests from a
tracked-files-only export. No other actionable findings.
