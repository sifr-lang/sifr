# Review Pass 5: `hardening_2` (Rust-interop verification matrix hardening)

Scope: staged diff for
`plans/issues/active/rust-interop-verification-matrix-hardening.md` item
`hardening_2`, re-reviewed after the pass-4 LOW findings. The
`ad-hoc-class-field-mutating-receiver-place-semantics` issue and its review
files are out of scope and were ignored.

## Pass-4 findings: all four resolved, plus the informational item

**Finding 1 (`.gitignore` negation too wide) — resolved.** `.gitignore:30-31`
now carries exactly the two suggested patterns:

```
!verification/areas/rust_interop/fixtures/*/examples/*/Cargo.lock
!verification/areas/rust_interop/fixtures/*/negative/Cargo.lock
```

Verified two-sidedly:

- `git check-ignore -v .../workspace_hash_crate/sifr_output/Cargo.lock`
  → `.gitignore:27:**/Cargo.lock` (still ignored — a gitignore `*` does not
  cross `/`), so a generated in-place build can no longer be committed by
  `git add -A`.
- `git ls-files` and `find` over the area return the identical set of **11**
  locks (10 under `examples/*/`, 1 under `negative/`). Nothing under the area is
  untracked or ignored except `__pycache__/`.

**Finding 2 (`digest`/`digest_hex` no longer digest) — resolved with a real
digest, not a rename.**
`.../shared_hash_bridge/rust/sifr_shared_hash_bridge/src/lib.rs:11-15` restores
FNV-1a-64 (`0xcbf2_9ce4_8422_2325` offset basis, `0x0000_0100_0000_01b3` prime),
with `digest` returning the big-endian bytes and `digest_hex` the `{:016x}`
form. I recomputed FNV-1a independently and every asserted constant is the true
digest:

| Input | Expected | Asserted at |
| --- | --- | --- |
| `b"sifr-rust-interop"` | `142632b86444e09a` | `examples/shared_hash_bridge/src/main.sifr:15` |
| `b"sifr"` | `4e138d18e63ba405` | `package_rust_interop_build_tests.rs:196` |
| `b"sifr-rust-interop"` (u64) | `1451903697411170458` | `package_rust_interop_build_tests.rs:128` |
| `b"sifr"` (u64) | `5625995497597281285` | `package_rust_interop_build_tests.rs:140` |

`workspace_hash::hash` (`rust/workspace_hash/src/lib.rs:1-5`) is the same real
FNV-1a, and `hash_pair` is a genuine mixing function. No stale `reverse`,
`rfis`, `73696672`, or `17` reference survives anywhere under either fixture or
in `docs/rust-interop.mdx`.

**Finding 3 (dead `verified_hash` binding) — resolved.**
`examples/workspace_hash_crate/src/main.sifr:18-19` is now
`print(verify_workspace_hash_crate())`, which both prints the probed value and
keeps the `first_hash != combined_hash` assertion (and therefore
`workspace_hash.hash_pair`) load-bearing. The scenario run asserts the printed
value `5625995497597281285`.

**Finding 4 (negative overlays had no drift check) — resolved, and the checks
bite.** `_validate_negative_overlays`
(`verification/areas/rust_interop/checks/_scenario_checks.py:305-417`) is called
from `validate_scenario_examples:116` and pins, mechanically:

- `same_workspace_crate/negative/Cargo.toml` — `[workspace]` and `[package]`
  must equal the canonical scenario manifest at the parsed-TOML level, and
  `[dependencies]` must be absent/empty;
- `same_workspace_crate/negative/Cargo.lock` — package-name set must equal the
  canonical lock's;
- `shared_bridge_crate/negative/sifr.toml` — `[package]`, `[source]`, `[rust]`
  must equal the canonical scenario manifest, and `[trust] rust-no-panic` must
  be exactly `["sifr_shared_hash_bridge.generated_private_type"]`;
- `shared_bridge_crate/negative/shared_bridge_lib.rs` — required to exist and to
  contain `use crate::__sifr_bridge::`.

I ran nine mutations against `check_fixture_matrix.py` (working tree restored
after each; `git status` confirmed clean):

| Mutation | Result |
| --- | --- |
| negative `sifr.toml` `bridge-version = 2` | rejected (`rust must match the canonical scenario`) |
| negative trust list → `digest` | rejected (`must trust only ...generated_private_type`) |
| re-add `workspace_hash` dependency edge to negative `Cargo.toml` | rejected (`must omit the workspace_hash dependency`) |
| drop the workspace member from negative `Cargo.toml` | rejected (`workspace must match the canonical scenario`) |
| rename a package in negative `Cargo.lock` | rejected (`package set must match the canonical scenario`) |
| strip `use crate::__sifr_bridge::` from the rejected source | rejected (`must exercise the package-generated import rejection`) |
| delete `negative/shared_bridge_lib.rs` | rejected (`is required`) |
| delete `negative/sifr.toml` | area check passes (see note) |
| delete `negative/Cargo.toml` | area check passes (see note) |
| unmutated control | ok |

The pass-4 concern — a canonical `bridge-version`/`sifr-version` bump leaving a
stale negative copy that still passes for the wrong reason — is fully closed.
The two "delete" rows are not a live gap: both files are `include_str!` targets
(`package_rust_interop_build_tests.rs:21-23, 9-11`), so removing either is a
**compile-time** failure of the whole `sifr_driver` test target in every profile,
not a silent pass. `_read_toml` returning `None` for a missing path is
pre-existing shared behavior used by every scenario-manifest check in the file,
so tightening it only here would be inconsistent with the module rather than
safer. Not raised as a finding.

**Informational item from passes 1–4 — resolved.**
`plans/issues/active/rust-interop-verification-matrix-hardening.md:210-211` now
lists, as `hardening_4` PR content: "gives `check_stale_drafts.py --self-test` a
real isolated temporary-tree entrypoint instead of falling through to the
ordinary checked-in-data scan". The false self-test signal is now explicitly
owned, and `hardening_4`'s exit gate already requires isolated temporary-tree
tests.

## Independent clean-checkout re-verification

Built a fresh tracked-files-only export in this pass — `git archive $(git
write-tree)` (HEAD plus index) into `/tmp/sifr-clean-p5`, with
`third_party/ruff` archived separately from its own `HEAD` — and ran there:

- `cargo test -p sifr_driver --lib rust_interop_build_tests -- --ignored --test-threads=1`
  → cold compile 46.80s, then **4 passed; 0 failed** in 187.72s.
- `uv run --project verification --locked python -m sifr_verify areas run --area rust_interop`
  → **variants=4, failures=0**.

Both `cargo metadata --locked --offline` consumers therefore resolve from
checked-in manifests and locks alone, with no network and no lock regeneration.

## Gates run in this pass

- Working tree area run: variants=4, failures=0 (34 fixtures, 34 rows, 10
  scenario examples, 4 categories).
- `check_fixture_matrix.py --self-test` (27 cases),
  `check_compatibility_matrix.py --self-test` (3),
  `check_tiers.py --self-test` (6) — all ok. `check_stale_drafts.py --self-test`
  still ignores the flag; that is `hardening_4` scope and is now written into the
  issue.
- `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --check` clean;
  `check_file_size_guardrails.py` PASS (2822 files, limit 900; largest touched
  file `check_fixture_matrix.py` at 813);
  `check_hir_maintainability_guardrails.py` PASS; `git diff --cached --check`
  clean.
- `git status --porcelain --ignored` over the area after all runs: no artifacts
  beyond `__pycache__/`.
- `scripts/run_all_tests.sh --profile create-pr` executed in this pass.

## Exit criteria for `hardening_2`

Re-evaluated against all six bullets plus the mutation requirement. The pass-4
verdicts still hold and none were weakened by the pass-4 fixes:

- **Frozen allowed-pair table encoded** — `ALLOWED_EXECUTION_KINDS`
  (`check_fixture_matrix.py:125-131`) is exactly the plan's table
  (`0→compiler-diagnostic`, `1→cargo-probe`, `2→{contract-only, cargo-probe,
  runtime-observed}`, `3→cargo-probe`, `4→{contract-only, cargo-probe,
  runtime-observed}`), enforced at `_validate_execution_semantics:245-253`, with
  `execution_kind` validity checked first and tier range checked separately so no
  pair can slip silently. The self-test sweeps all 5×4 pairs two-sidedly. **Met.**
- **Real `check_tiers.py --self-test`** — mutated TOML/JSON rendered into
  `tempfile.TemporaryDirectory` and re-validated through `_load_and_validate`
  (`check_tiers.py:109-135`), with an unmutated control asserting zero failures,
  plus missing, duplicate, matrix/TOML mismatch, invalid-tier-name, and
  empty-fixture-list cases. Never falls through to the checked-in data path.
  **Met.**
- **`diagnostic_crate_rationale` added and cross-validated** — three identical
  copies each for `direct_crate_negative_type` and `blocking_diagnostics`
  (fixture matrix, compatibility row, `fixture.json`); exact key set
  `{purpose, linked, executed}` with `linked`/`executed` pinned so `0`/`"false"`
  are rejected; `check_compatibility_matrix.py:125` closes the row↔fixture
  direction; the field is rejected outright on any non-`compiler-diagnostic`
  row, and required whenever a diagnostic row names crates. **Met.**
- **Both diagnostic rows migrated** — data plus matching prose in
  `blocking_diagnostics/README.md` and `direct_crate_negative_type/README.md`.
  **Met.**
- **Tier-1 rows are real `cargo-probe` rows** — `same_workspace_crate` and
  `shared_bridge_crate` are `cargo-probe` in all three places; both graphs come
  from real `cargo metadata --format-version=1 --locked --offline` plus
  `sifr_package::derive_package_graph`
  (`package_rust_interop_build_tests.rs:59-94`); positive tests build **and run**
  both the evidence source and the scenario source and assert the four real FNV
  constants above plus `len == 8`; both negatives run against genuine Cargo
  graphs (in-workspace-but-undeclared member; checked-in backend source carrying
  the `crate::__sifr_bridge` violation) and observe `SIFR-RUST-RESOLVE-0001`.
  Reproducible from a clean checkout, verified above. **Met.**
- **Docs/matrices/manifests updated in the same change** — fixture matrix,
  compatibility matrix, all four `fixture.json`s, evidence headers, tier
  descriptions (`rust_interop_tiers.toml:3,13,25,47,56`), both fixture READMEs,
  the scenario README, `internal_docs/rust_interop_architecture.md:968-1022`,
  `docs/rust-interop.mdx:49-62`, and
  `verification/areas/rust_interop/README.md` §Tier And Execution Semantics.
  `diagnostic_family` drift stays mechanically closed by
  `_validate_diagnostic_family_alignment`. **Met.**
- **Mutation coverage** — all 20 tier/kind combinations, missing rationale,
  mismatched rationale, malformed rationale with and without crates, rationale on
  a `cargo-probe` row, tier-1 downgrade to `contract-only`, diagnostic-family
  drift, and now the six negative-overlay drift mutations above. **Met.**
- **No skip/fallback/network dependency** — the `#[ignore]` markers are honest:
  `sifr_driver_generated_builds` is blocking, `modes: ["full"]`, and its command
  includes `--ignored` (`verification/profiles/merge.json:67`,
  `nightly.json:69`, `release.json:68`), which both fixture READMEs name. All 11
  locks are path-only (no registry entries), so no lane gains a network
  dependency. **Met.**

## Claim strength

Honest in both directions, and stronger than pass 4:

- Every `cargo-probe` label maps to a test that builds a real Cargo graph from
  checked-in manifests **and** locks, then executes the binary and asserts a
  value independently derivable from the fixture's Rust source.
- The negative-direction caveat ("negative directions may observe a required
  compiler rejection before Cargo execution") is stated in
  `docs/rust-interop.mdx`, the area README, and both fixture READMEs.
- No diagnostic row presents crate metadata as compiled evidence; both carry
  `linked = false`, `executed = false`, and README prose saying so.
- The fixture no longer contains a name/behavior mismatch: a crate called
  `sifr_shared_hash_bridge` with functions `digest`/`digest_hex` now really
  hashes, and the README's "hash bridge" description is accurate.

## Actionable findings

None.

## Verdict

**APPROVED.**

All four pass-4 LOW findings are resolved at the root rather than papered over —
the `.gitignore` negation is exact and demonstrably excludes generated
`sifr_output` locks, the shared bridge performs a real FNV-1a digest whose every
asserted constant I recomputed independently, the dead scenario binding is gone,
and the negative Cargo/Sifr overlays plus the rejected Rust source are
mechanically drift-checked by the area with six verified mutations.
`hardening_4` now explicitly owns the real `check_stale_drafts.py --self-test`
fix. `hardening_2`'s exit criteria are all met, the change is reproducible from a
tracked-files-only export (4/4 `cargo-probe` tests, area 4/4), and no row claims
more than an executed test observes. No actionable findings.
