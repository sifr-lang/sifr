# Review Pass 4: `hardening_2` (Rust-interop verification matrix hardening)

Scope: staged diff for
`plans/issues/active/rust-interop-verification-matrix-hardening.md` item
`hardening_2`, re-reviewed after the pass-3 clean-checkout blockers. The
`ad-hoc-class-field-mutating-receiver-place-semantics` issue and its review
files are out of scope and were ignored.

## Pass-3 blockers: both resolved, independently verified

`.gitignore:30` now carries `!verification/areas/rust_interop/**/Cargo.lock`
after the `**/Cargo.lock` rule, and all **11** fixture locks on disk under the
area are staged — `git ls-files --cached` and `find` return identical sets:

- `same_workspace_crate/examples/workspace_hash_crate/Cargo.lock`
- `same_workspace_crate/negative/Cargo.lock`
- `shared_bridge_crate/examples/shared_hash_bridge/Cargo.lock`
- `cargo_locked_offline/examples/locked_offline_cache/Cargo.lock`
  (pass-3 finding 3, the pre-existing latent gap)
- plus `bridge_version_mismatch`, `ecosystem_backend_certification`,
  `ecosystem_cli_certification`, `local_bridge_blake3`, `native_build_script`,
  `panic_abort_profile`, `proc_macro_trust`.

Nothing else under the area is untracked or ignored except `__pycache__/`.

**Finding 1 (compile-time `include_str!`) — resolved.** All eight
`include_str!` targets in
`crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:3-26` are
tracked, including `negative/Cargo.toml`, `negative/Cargo.lock`,
`negative/sifr.toml`, and `negative/shared_bridge_lib.rs`. No `include_str!`,
`env!("CARGO_MANIFEST_DIR")` join, or `cargo metadata` probe reaches a
local-only artifact.

**Finding 2 (`cargo metadata --locked`) — resolved.** All 11 locks contain only
path packages with no registry entries, and each matches its sibling
`Cargo.toml` dependency set (checked individually), so `--locked --offline`
resolves without network and without regenerating.

**Independent clean-checkout re-verification.** I built my own
tracked-files-only export — `git archive $(git write-tree)` (HEAD plus index)
into `/tmp/sifr-clean-export-p4`, with `third_party/ruff` archived separately
from its own `HEAD` — and ran, in that export:

- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1`
  → cold compile 54.45s, then **4 passed; 0 failed** in 204.11s. The test target
  compiles, so the pass-3 build break is gone, not merely relocated.
- `uv run --project verification --locked python -m sifr_verify areas run --area rust_interop`
  → **variants=4, failures=0** (34 fixtures, 34 rows, 4 categories, 10 scenario
  examples). This is the check that would have failed on pass-3 finding 3.

## Gates run in this pass

- Working tree: area run 4/4 failures=0; `check_fixture_matrix.py --self-test`
  (27 cases), `check_compatibility_matrix.py --self-test` (3),
  `check_tiers.py --self-test` (6) — all ok.
- `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --check` clean;
  `check_file_size_guardrails.py` PASS (2822 files, limit 900);
  `git diff --check` clean.
- Confirmed the area run generates no in-tree artifacts: `git status --ignored`
  on the area is unchanged afterwards.
- Not re-run: `scripts/run_all_tests.sh --profile create-pr` and the default
  merge gate. Unlike pass 3, the untracked-artifact caveat no longer applies —
  the export runs above prove those lanes do not depend on local-only files.

## Exit criteria for `hardening_2`

Re-evaluated against all five bullets plus the mutation requirement:

- **Frozen allowed-pair table encoded** — `ALLOWED_EXECUTION_KINDS`
  (`check_fixture_matrix.py:123-129`) is exactly the plan's table; enforced in
  `_validate_execution_semantics:245-252`. The self-test sweeps all 5×4 pairs
  two-sidedly (`is_allowed == has_pair_failure` fails the run). An out-of-range
  tier cannot slip the pair check silently, because `tier must be 0..4` is
  validated separately at `:208`. **Met.**
- **Real `check_tiers.py --self-test`** — mutated TOML/JSON rendered into
  `tempfile.TemporaryDirectory` and re-validated through `_load_and_validate`,
  with an unmutated control asserting zero failures, plus missing, duplicate,
  matrix/TOML mismatch, invalid-name, and empty-fixture-list cases. Does not
  fall through to the checked-in data path. **Met.**
- **`diagnostic_crate_rationale` added and cross-validated** — three identical
  copies each for `direct_crate_negative_type` and `blocking_diagnostics`
  (fixture matrix, compatibility row, `fixture.json`);
  `_validate_manifest_alignment:539-556` and
  `check_compatibility_matrix.py:125` close both directions; exact key set
  `{purpose, linked, executed}` with `linked`/`executed` pinned via
  `is not False` (so `0` or `"false"` is rejected); the shape is validated
  whenever the field is present on a diagnostic row, and rejected outright on
  any non-`compiler-diagnostic` row. **Met.**
- **Both diagnostic rows migrated** — including matching prose in
  `blocking_diagnostics/README.md:6-9` and
  `direct_crate_negative_type/README.md:6-8` stating the crates are not linked
  or executed. **Met.**
- **Tier-1 rows are real `cargo-probe` rows** — and now genuinely reproducible
  by anyone who clones the branch. The graph comes from real
  `cargo metadata --locked --offline` over checked-in manifests plus
  `sifr_package::derive_package_graph`
  (`package_rust_interop_build_tests.rs:59-94`); positive tests build **and
  run** both the evidence source and the scenario source, asserting observed
  values `1451903697411170458`, `17`, `73696672`, `b"rfis"`, and
  `"736966722d727573742d696e7465726f70"`; both negatives run against genuine
  Cargo graphs (in-workspace-but-undeclared member; checked-in backend source
  carrying the `crate::__sifr_bridge` violation) and observe
  `SIFR-RUST-RESOLVE-0001`. The pass-3 conditional is discharged. **Met.**
- **Docs/matrices/manifests updated in the same change** — fixture matrix,
  compatibility matrix, both `fixture.json`s, all evidence headers, tier
  descriptions (`rust_interop_tiers.toml:3,13,25,47,56`), both fixture READMEs,
  the scenario README, `internal_docs/rust_interop_architecture.md:968-1022`,
  `docs/rust-interop.mdx:49-62`, and
  `verification/areas/rust_interop/README.md` §Tier And Execution Semantics.
  `diagnostic_family` drift is mechanically closed by
  `_validate_diagnostic_family_alignment:566`, which is why
  `same_workspace_crate/fixture.json:3` is now `SIFR-RUST-RESOLVE-0001`.
  **Met.**
- **Mutation coverage** — disallowed pair (all 20 combinations), missing
  rationale, mismatched rationale, malformed rationale with and without crates,
  rationale on a `cargo-probe` row, tier-1 downgrade to `contract-only`,
  diagnostic-family drift. **Met.**
- **No skip/fallback/network dependency** — the `#[ignore]` markers are claimed
  honestly: `sifr_driver_generated_builds`
  (`verification/profiles/merge.json:67`, `nightly.json:69`, `release.json:68`)
  is blocking, `modes: ["full"]`, and its command includes `--ignored`, which
  both fixture READMEs name. All fixture locks are path-only, so no lane
  acquires a network dependency. **Met.**

Claim strength is honest in both directions. Every `cargo-probe` label now maps
to a test that builds real Cargo graphs from checked-in manifests **and**
lockfiles; the negative-direction caveat ("may observe a required compiler
rejection before Cargo execution") is stated in public docs, the area README,
and both fixture READMEs; and no diagnostic row can present crate metadata as
compiled evidence.

## Actionable findings (all LOW, none blocking)

### 1. LOW — the `.gitignore` negation is wider than the artifacts it exists for

`.gitignore:30` — `!verification/areas/rust_interop/**/Cargo.lock` also
un-ignores *generated* locks anywhere under the area:

```
$ git check-ignore -v verification/areas/rust_interop/fixtures/same_workspace_crate/examples/workspace_hash_crate/sifr_output/Cargo.lock
.gitignore:30:!verification/areas/rust_interop/**/Cargo.lock   → NOT ignored
```

`target/**` stays safe (the directory rule `**/target/` at line 26 wins), and
nothing generates in-tree output under the area today — the tests copy to
`std::env::temp_dir()` via `mktemp_dir`
(`crates/sifr_driver/src/tests/project_build_check.rs:4-16`). So this is latent,
not live: it becomes a problem the first time someone runs `sifr build` on a
fixture example in place and then `git add -A`, silently committing a generated
lock into a fixture.

Since `.gitignore` is already being touched by this PR, prefer two exact
patterns instead of one wildcard:

```
!verification/areas/rust_interop/fixtures/*/examples/*/Cargo.lock
!verification/areas/rust_interop/fixtures/*/negative/Cargo.lock
```

Verified in a throwaway repo that these match all 11 staged locks (10 under
`examples/*/`, 1 under `negative/`) and still ignore
`examples/*/sifr_output/Cargo.lock`, because a gitignore `*` does not cross `/`.

### 2. LOW — `digest`/`digest_hex` no longer digest anything

`verification/areas/rust_interop/fixtures/shared_bridge_crate/examples/shared_hash_bridge/rust/sifr_shared_hash_bridge/src/lib.rs:1-9`
replaces the FNV-1a implementation with `input.iter().rev().copied().collect()`
and per-byte hex formatting. The crate is still named
`sifr_shared_hash_bridge`, the functions are still `digest`/`digest_hex`, and
`examples/shared_hash_bridge/README.md:4-5` still describes it as a hash bridge.
The motive is understandable — reversal and hex give trivially assertable
outputs (`b"rfis"`, `73696672`) — and it does not weaken the row's claim, which
is about the package boundary and stable `bytes`/`str` types. But the fixture
now models a "shared hash bridge" that does not hash, which is the kind of
name/behavior mismatch this issue is otherwise eliminating. Either rename to
`reverse`/`to_hex` or restore a real hash and assert its constant.

### 3. LOW — dead binding in a checked-in scenario source

`verification/areas/rust_interop/fixtures/same_workspace_crate/examples/workspace_hash_crate/src/main.sifr:19`
binds `verified_hash` and never reads it. The call itself is load-bearing (it
runs the `first_hash != combined_hash` assertion that probes
`workspace_hash.hash_pair`), so only the binding is dead. Call
`verify_workspace_hash_crate()` as a statement, or print its result instead of
recomputing `workspace_hash(b"sifr-rust-interop")` on the next line.

### 4. LOW — negative overlay manifests have no drift check

`same_workspace_crate/negative/Cargo.toml` and
`shared_bridge_crate/negative/sifr.toml` are near-copies of the canonical
scenario manifests (differing only by the removed dependency edge and the
`[trust]` list, respectively). They are consumed solely by the Rust tests via
`include_str!`; no area check validates them or compares them to the canonical
files. If the canonical `sifr.toml` later bumps `bridge-version` or
`sifr-version`, the negative copy silently stays stale and the negative test
still passes for the wrong reason. `hardening_3` is the natural place to bind
these, since it is already adding structured provenance.

## Informational (not `hardening_2` scope)

`check_stale_drafts.py --self-test` still prints `rust interop stale draft scan
ok` and ignores the flag, so it gives false self-test signal while listed in
this issue's Required Validation block. Passes 1, 2, and 3 each suggested adding
an explicit line to `hardening_4`; the issue text at
`plans/issues/active/rust-interop-verification-matrix-hardening.md:205-218`
still does not mention it. Recommend adding the bullet now so it is not lost a
fourth time.

## Verdict

**APPROVED.**

Both pass-3 blockers are genuinely fixed and independently re-verified from a
tracked-files-only export built in this pass: `sifr_driver` compiles, all four
`--ignored` `cargo-probe` tests pass (4/4, 204.11s), and the complete
Rust-interop area passes 4/4 with zero failures. All `hardening_2` exit criteria
are met, the required mutation coverage is present and two-sided, and no row
claims more than an executed test observes.

The four findings above are LOW and non-blocking — hygiene and fixture-realism
nits, none of which affects a compatibility claim or a gate. Finding 1 is worth
folding in before merge only because this PR already edits `.gitignore`.
