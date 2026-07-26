## Round-3 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only. I ran checks, self-tests, guardrails, and read-only Cargo commands
(`cargo fetch --locked --offline`, `cargo tree -i`, `cargo clippy -p
sifr_rust_interop_catalog`, `cargo fmt --check`); no file was edited, staged, or
committed. I re-derived every round-1 and round-2 finding from the tree rather
than trusting the round-3 prompt.

### Findings

**1. HIGH — the recorded create-PR budget evidence contradicts the only
create-PR artifact in the tree, and real headroom is roughly half what is
claimed.**
`verification/areas/rust_interop/README.md:142-147` states "The
`certification_0` create-PR gate measured 4,022 ms on 2026-07-26, leaving 978 ms
of enforced headroom." The newest create-PR lane artifact
(`target/validation_lane_reports/create-pr.latest.log:311-312`, mtime 19:58, and
the matching `create-pr.latest.json` entry `{"name":"rust_interop_checks",
"elapsed_ms":4442,"budget_ms":5000,"budget_enforcement":"blocking"}`) records
**4,442 ms — 558 ms of headroom (11%)**. No artifact anywhere under
`target/validation_lane_reports/` contains 4,022; the only other
`rust_interop_checks` records are 538 ms (06:10, pre-change) and 3,264 ms in the
stale merge log. So the number the README publishes as the authoritative
enforced measurement is not reproducible from the recorded evidence, and it
understates the risk it exists to document.

The understatement matters because this change made the step's cost
cache-dependent: the `matrix` case alone is now 2,084 ms
(`create-pr.latest.log:284`) because `_crate_catalog.py:39-47` shells out to
`cargo fetch --locked --offline` inside a blocking suite, and that subprocess
contends on the cargo package-cache lock. Failure scenario: a developer or CI
runner whose cache lock is briefly contended, or whose registry index is larger,
crosses 5,000 ms and gets a blocking `step budget exceeded`
(`profile_runner.py:395-401`) on an unrelated PR. Either re-measure and record
the real number with its variance, or raise the budget in this change — round 1
(finding 2) and round 2 (finding 8) both flagged this line, and it is still
wrong.

**2. HIGH — the merge, nightly, and release profiles have never been executed
against this change, although it rewrites `ProfileRunner.run()` for every
profile.**
`profile_runner.py:270-284` moved offline enablement out of `__init__` and
inserted a new first step for *all* profiles. The only merge evidence in the
tree is `target/validation_lane_reports/merge.latest.{json,log}` (mtime 17:20),
which reports `rust interop verification ok: variants=8`, contains **zero**
occurrences of `cargo_cache_setup`, and contains zero occurrences of
`stable-candidate` — it predates both the suite registration and the setup
prelude. There is no nightly or release report at all. The item's own exit gate
requires "all four local profiles report the Rust-interop checks step as
executed"
(`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:242`), and
AGENTS.md makes `scripts/run_all_tests.sh` (merge) the default authoritative
gate. Checklist line `:183-184` is honestly left unchecked, so this is an
unfinished gate rather than a false claim — but combined with finding 1 the
README asserts a create-PR measurement while promising that "the merge-gate
measurement is recorded after that profile runs" (`README.md:144-145`), and
neither is currently backed by an artifact for this tree.

**3. MEDIUM — the new runner self-test writes a fake record into the production
lane-timing channel and corrupts the accounting the same change introduced.**
`selftest.py:310-341`'s `RecordingProfileRunner` subclasses `ProfileRunner` and
calls the real `run()`, which calls the real `timed_step`
(`profile_runner.py:214-228`) and therefore prints
`[sifr-lane-step] name=cargo_cache_setup elapsed_ms=0 status=pass`. Reproduced
directly: `uv run --project verification --locked python -m sifr_verify
--self-test` emits that line as its first output. Because
`verification_runner_foundation` runs inside the lane, `reports.py:202-210` and
`:350-356` ingest it — `create-pr.latest.json` now lists `cargo_cache_setup`
twice, at 506 ms (real, `create-pr.latest.log:7`) and 0 ms (fake,
`create-pr.latest.log:437`), with the fake record decorated with the real
advisory budget. The one step this change added to lane accounting is the one it
poisons, and a governance consumer of the report (Phase 40 qualification
records) sees a duplicated setup step with a 0 ms measurement. Fix by asserting
ordering without invoking the printing path (override `timed_step`, or record
into a list from a stub rather than driving real `run()` through real
`timed_step`). The ordering coverage itself is good and should be kept — see the
round-2 verification below.

**4. MEDIUM-LOW — the docs-wide discovery in `main()` is still untested; round-2
finding 3 is only half fixed.**
The new case at `check_stable_support_claims.py:503-523` passes
`{"docs/secondary.md": ...}` straight into `_validate`, so it proves the
`public_documents` merge path at `:186-195` and the secondary-document overclaim
rule. Nothing exercises the actual discovery at `:539-543`. Deleting those five
lines leaves all 18 self-test cases green while collapsing the gate back to the
single marker block in `docs/rust-interop.mdx` — verbatim the failure scenario
round 2 described. Contrast with round-2 finding 2, which was fixed properly by
driving the real call site. Extracting a `_collect_public_documents(root)`
helper and pointing one case at a temp tree would close it.

**5. MEDIUM-LOW — the contract-only narrowing in the compatibility matrix is
prose that no check binds.**
The item requires that the five preserved rows' "notes and public claims must
say `contract-only`" (`certification.md:212-213`). The public-claims side is
mechanically gated. The notes side is not:
`check_compatibility_matrix.py:182-183` only requires `notes` to be non-empty,
and nothing relates a row's notes to its `execution_kind`. Failure scenario: a
later row PR restores `rust_interop_compatibility_matrix.json:326` to "Zero-copy
bytes view contracts are supported." and every gate — area run, all four
profiles, stable-candidate — stays green while the authoritative matrix
overclaims again. A rule of the shape "`execution_kind == contract-only` ⇒ notes
contain `contract-only`" would make the narrowing durable rather than
decorative.

**6. LOW — the prelude's cold cost is unmeasured, unrecorded, and budgeted in
only one profile.**
`cargo_cache_setup` measured 506 ms warm. This change adds 324 packages
(~100 MB of `.crate` payload) to a cold `cargo fetch --locked`, charged inside
the lane against the create-PR 15-minute cold budget — and the newest run
already consumed 814 s (13.6 min, `create-pr.latest.time`) and raised the
wall-time advisory. Only `create-pr.json:10` carries a `cargo_cache_setup`
budget (300,000 ms advisory); merge, nightly, and release have none, so the
prelude is unbudgeted in the three heavier profiles. Recording one cold
measurement in the README alongside the warm one would make the 15-minute
envelope auditable.

**7. LOW — `cargo_setup.py` validates one representation of the setup command
and executes another.**
`cargo_setup.py:19-23` rejects any profile whose `setup_command` differs from
`CANONICAL_SETUP_COMMAND` (`:7`), then returns a separately hardcoded
`["cargo", "fetch", "--locked"]`. Changing the constant (say to add `--frozen`)
would pass validation for updated profiles while still executing the old argv.
`shlex.split(CANONICAL_SETUP_COMMAND)` removes the second authority.

**8. LOW — dead second guard.** `enable_offline_cargo`
(`profile_runner.py:292-296`) re-tests `cargo_policy.offline`, which `run()`
already tested at `:285`. One of the two is unreachable-by-construction; the
self-test's override hides which.

**9. LOW — ten pre-existing pins moved version as a side effect; worth one line
in the item.** The lock diff is genuinely reachable-minimal (see verification
below), but `hashbrown 0.17.0→0.17.1`, `rand 0.10.1→0.10.2`,
`js-sys 0.3.95→0.3.103`, `wasm-bindgen 0.2.118→0.2.126` (+3 macro crates), and
`futures-core/-channel/-sink 0.3.32→0.3.33` are in-place bumps, and `hashbrown`
/ `rand` are consumed by first-party crates (`get-size2 → ruff_db`,
`sifr_stdlib`, `ruff_notebook` per `cargo tree -i`). Each is *forced* — I
verified the requirements in the cached manifests: `datafusion-common 54.1.0`
requires `hashbrown 0.17.1`, `tungstenite 0.30.0` requires `rand 0.10.2`,
`web-sys 0.3.103` and `wasm-bindgen-futures 0.4.76` require `js-sys =0.3.103`
and `wasm-bindgen =0.2.126`, and `futures 0.3.33` pins its own family at
0.3.33 — so this is inherent to hosting the catalog in the same workspace, not a
stray `cargo update`. AGENTS.md treats lock diffs as intentional, so the item
should say so explicitly instead of leaving a reader to re-derive it.

**10. LOW — "exact-pinned" is alias-level only.** `arrow = "=58.3.0"`
(`crates/sifr_rust_interop_catalog/Cargo.toml:11`) while every arrow
implementation crate resolved to 58.4.0 (`arrow-array`, `arrow-schema`,
`parquet`, …). Reproducibility is fine — the lock is exact — but the inventory
line "44 required crate aliases, each exact-pinned" (`certification.md:197-198`)
should not be read as pinning the arrow/datafusion/polars families.

**11. LOW — two catalog validator branches remain unmutated.**
`_crate_catalog.py` now covers missing crate, non-exact version, absent from
lock, wrong features, both `default-features` directions, a `[features]` table,
unexpected dependency, `optional: false`, wrong `package` alias, and missing
metadata (12 cases + control). Still uncovered: a non-dict dependency value
(`:101-103`) and a non-table `[dependencies]` (`:60-63`).

**12. LOW — hermeticity debt carried into the new row.**
`fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:14` hardcodes
`/tmp/sifr-rust-interop.bin`. Inert while the row is `planned` and never
executed, but it violates the temp-dir rule the issue imposes at
`certification.md:262-270` and must be replaced when `certification_7` executes
it. Flagged in rounds 1 and 2; still present.

**13. LOW — the docs deferral prose has no promotion guard.**
`_validate_public_document_scope` (`check_stable_support_claims.py:228-237`)
constrains only rows still in the derived deferral set, and `DEFERRAL_TERMS`
(`:35-43`) makes `_validate_unstructured_advertisements` skip any line
containing "planned" or "future-owned". When `certification_1`+ promotes a row,
`docs/rust-interop.mdx:80-84` can keep describing it as future-owned and planned
with every gate green. The inverse direction (overclaiming) is covered; this one
is not.

### Round-2 findings, re-checked independently

1. **Resolved, and verified in depth.** The lock diff is now genuinely
   reachable-minimal: **0 packages removed** (round 2's 12 dropped families,
   including `wit-bindgen`/`wasmparser`/`wit-component`, are back), 324 new
   package names of which **all 324 are reachable from
   `sifr_rust_interop_catalog`** in the lock graph and **0 are orphans**, and
   none of round 2's unreachable bumps survive — `insta` is still 1.47.2,
   `ignore` 0.4.25, `time` 0.3.47, `toml`/`toml_edit`/`winnow`/`camino`/
   `serde_with`/`ref-cast`/`thin-vec`/`console`/`rust_decimal`/`globset`/`bstr`
   all unchanged. 44 names have changed version *sets*; 34 of those merely gain
   a coexisting older/newer major, and the 10 in-place bumps are each forced (see
   finding 9). `version = 4`, and every non-workspace entry carries a checksum
   (the 37 without one are the 22 workspace members plus 15 path-based ruff
   crates). `CARGO_NET_OFFLINE=true cargo fetch --locked --offline`: pass. No
   vendor conflict: `.cargo/config.toml` is a placeholder with no source
   replacement, and vendor substitution is invocation-scoped
   (`CargoVendorMode::SysrootOnly`), which `README.md:64-66` correctly disclaims.
2. **Resolved.** `selftest.py:310-341` drives the real `run()` and asserts
   `["header","cargo-cache-setup","offline","selected-areas"]`; deleting
   `prepare_cargo_cache()` at `profile_runner.py:271` now breaks the self-test,
   which is exactly the regression round 2 asked for. Error propagation is
   correct: `cargo_setup_command`'s `ValueError` → `ProfileRunnerError` →
   `timed_step` status 2 → `run()` aborts before any validation step;
   `run_command` non-zero → `CommandFailed` → the returncode. `setup_env` is a
   copy of a full `os.environ` copy with `CARGO_NET_OFFLINE` popped, so the
   fetch is genuinely online-capable while every later step is not. See findings
   3 and 8 for what the fix introduced.
3. **Partly resolved** → finding 4.
4. **Resolved.** `runtime_deferral_ids` and `compile_scope_ids` are derived from
   the compatibility matrix (`check_stable_support_claims.py:101-111`), the
   hardcoded `RUNTIME_DEFERRAL_IDS`/`COMPILE_SCOPE_IDS` constants are gone, and
   `runtime_deferrals` must match the derived set exactly, duplicate-free
   (`:112-124`), with a mutation case at `:483-487`. The seven ids in
   `stable_support_claims.json:6-14` reproduce the derived set.
5. **Resolved.** The `if crate == "candle"` special case is gone;
   `_crate_catalog.py:120-124` reads `policy.get("default_features", True)`, and
   `EXPECTED_FEATURE_POLICIES["candle"]` (`_matrix_inventory.py:124`) now carries
   `default_features: False`. The same table is the single authority for both the
   catalog (`_crate_catalog.py:125-129`) and every fixture row
   (`check_fixture_matrix.py:382-403`), so `advanced_data_matrix` (existing) and
   `advanced_data_runtime_matrix` (new) both had to gain
   `{"backend":"cpu-only","default_features":false}` in matrix *and*
   `fixture.json`, and `Cargo.toml:18` states `default-features = false`. No
   duplicate authority remains here.
6. **Resolved** for the four named branches; two remain → finding 11.
7. **Resolved as documented scoping.** `profile_policy.md:20-28` now names the
   prelude as the only registry-network opportunity, states that a failed fetch
   aborts rather than letting later steps self-heal online, scopes
   `execution_sandbox.external_network` to post-prelude commands, and `:38-40`
   demotes `doctor` to diagnosis; `create-pr.json:51` was rewritten to match.
   Residual, acceptable: `network_policy.live_network_allowed: false` and
   `execution_sandbox.external_network: "forbidden"` remain unqualified booleans
   in all four profiles (and `coverage_matrix.py:329` asserts the former is
   `false`), so the reconciliation lives only in prose.
8. **Not resolved** → finding 1.
9. **Resolved.** `cargo_cache_setup` is a real `timed_step`
   (`profile_runner.py:271`) with a create-PR advisory budget
   (`create-pr.json:10`), is added to `PROFILE_STEP_NAMES`
   (`profiles.py:25`, which also enforces the budget key), and is picked up
   generically by `reports.py`. Findings 3 and 6 are the residuals.
10. **Resolved.** `crates/sifr_rust_interop_catalog/src/lib.rs` is a six-line
    doc comment; `CERTIFICATION_CRATE_COUNT` is gone.
11. **Resolved.** Round-1 (9,840 bytes) and round-2 (9,521 bytes) artifacts are
    populated; this file is round 3.
    Round-1 residuals also confirmed fixed: `docs/rust-interop.mdx:76` says "is
    validated against", the duplicated README paragraph is gone, both new rows
    are named in the public doc *and required* to be
    (`check_stable_support_claims.py:212-216`), `README.md:128-130` says "five",
    the blank-line style regression at `check_fixture_matrix.py:33-35` is fixed,
    and the file is 758 lines (was 880) after the `_matrix_inventory.py`
    extraction.

### What verifies clean

Recorded inventory reproduces exactly and independently: 36 compatibility rows /
36 fixture rows / 36 schema-v2 manifests, 47 `passing` + 25 `planned` in *both*
matrices, categories 17 `supported` / 5 `supported-through-bridge` / 1
`unsupported-by-design` / 13 `future-owned-by-separate-phase`, execution kinds 13
`cargo-probe` / 4 `compiler-diagnostic` / 10 `contract-only` / 9
`runtime-observed`, 44 crate aliases, 23 claims (= 36 − 13 future-owned), 5
suites / 10 cases. Both new rows are honest:
`future-owned-by-separate-phase` with `future_owner`, both directions `planned`,
legal tier/execution pairs (2 and 4 with `runtime-observed` per
`_matrix_inventory.py:115-121`), and `future-owned` /
`future-owned-diagnostic` manifests matching the
`ecosystem_backend_certification` convention. The five contract-only rows
changed in `notes` only — `category`, `execution_kind`, and evidence are
untouched. Suite selection is identical across create-pr/merge/nightly/release
and `required_rust_interop_suites()` derives from the manifest, so the
all-profiles contract stays mechanical. `stable_support_claims.json` carries the
non-authority `role` and both canonical source paths, and the checker
mutation-tests role/schema/source drift.

Executed here: area run 10 variants / 0 failures; fixture-matrix self-test 81
cases; compatibility 4; tiers 6; stable claims 23 and self-test 18; runner
self-test all eight sections pass; `cargo fetch --locked --offline` pass;
`cargo clippy -p sifr_rust_interop_catalog -- -D warnings` clean;
`cargo fmt --check`, file-size guardrail, HIR guardrail, `git diff --check` all
pass. File sizes: `profile_runner.py` 866, `check_fixture_matrix.py` 758,
`check_stable_support_claims.py` 559, `selftest.py` 587 — all under the 900-line
cap, though `profile_runner.py` has 34 lines of headroom and gained 32 here. No
user-path panic or fallback hazard in the new Python: `_parse_public_claims`
slices rather than indexes, `validate_crate_catalog` catches OSError and
TOMLDecodeError, `_validate_non_cargo_policies` isinstance-guards every level,
and the top-level `json.loads` behaviour matches every sibling checker.
`--all-features` appears only in package-scoped `sifr_stdlib` suites, so the
optional catalog graph is never compiled by an existing lane. No scope creep: the
`Cargo.toml` member, `cargo_metadata_classification.json` entry, and Phase 40
ownership edits are all required by this item, and no unrelated user change was
touched.

Blocking/actionable: 1, 2, 3 (and 4, 5 should land with them — both are drift
holes in gates this item exists to make honest).
Optional: 6, 7, 8, 9, 10, 11, 12, 13.

NOT SATISFIED
