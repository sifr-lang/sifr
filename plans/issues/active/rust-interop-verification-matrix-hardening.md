# Rust Interop Verification Matrix Hardening

## Status

Active follow-up created by Phase 39 final closeout review.

This issue is an entry prerequisite for
[`rust-interop-runtime-ecosystem-certification.md`](rust-interop-runtime-ecosystem-certification.md).
Each ordered item below is one PR and must be reviewed and merged before the
next item starts.

Phase 40 `milestone_40_1` may not claim its required release-profile
Rust-interop suite execution until `hardening_1` has merged. Its stable-claim
gate additionally depends on the certification issue's `certification_0`;
`hardening_1` owns execution wiring, while `certification_0` owns claim
derivation and stable-candidate validation.

## Objective

Make Rust-interop compatibility claims mechanically no stronger than the
evidence an authoritative local validation lane executes.

The completed implementation must prevent:

- tier labels that contradict `execution_kind`;
- diagnostic-only crate examples being presented as linked runtime evidence;
- `supported`, `supported-through-bridge`, or `unsupported-by-design` rows
  relying on README prose or an unexecuted test name;
- accepted examples passing because incidental nearby wording happens to look
  like rejection prose; and
- profile JSON accepting `rust_interop` selections that the legacy facade does
  not execute.

## Current Baseline and Frozen Semantics

At issue start there are 34 fixture rows and 34 compatibility rows. Eleven are
`future-owned-by-separate-phase`. The following tier/execution-kind
combinations are the only valid combinations after `hardening_2`:

| Tier | Allowed `execution_kind` | Meaning |
| --- | --- | --- |
| 0 | `compiler-diagnostic` | parser, lowering, metadata, and diagnostic behavior; no Cargo build claim |
| 1 | `cargo-probe` | direct/local/shared bridge package build or value probe |
| 2 | `contract-only`, `cargo-probe`, `runtime-observed` | safety contracts, build probes, or executed lifecycle/runtime behavior; the row capability and docs must name which one |
| 3 | `cargo-probe` | locked Cargo, trust, proc macro, build script, or native-link build behavior |
| 4 | `contract-only`, `cargo-probe`, `runtime-observed` | production ecosystem contract, build, or runtime certification, explicitly scoped by the row |

Tier is breadth/subject ownership; `execution_kind` is evidence strength. A
contract-only tier-2/tier-4 row is valid only as a contract-only claim and must
not satisfy a runtime claim.

The two current tier-1 `contract-only` rows, `same_workspace_crate` and
`shared_bridge_crate`, must become `cargo-probe` rows with actual package-build
evidence. Do not weaken tier 1 to permit contract-only evidence.

For compiler-diagnostic rows, non-empty `required_crates` is allowed only with
a non-empty structured `diagnostic_crate_rationale` in the fixture matrix,
compatibility row, and `fixture.json`. The rationale means the named crate API
is used only to exercise diagnostics and is not linked or executed. Initially:

- `direct_crate_negative_type` explains that `regex` supplies the rejected
  unsupported direct-signature shape; and
- `blocking_diagnostics` explains that `rusqlite`, `rayon`, and `flate2`
  supply blocking/CPU classification examples only.

The three copies must match exactly. The validator rejects the field on any
non-`compiler-diagnostic` row and rejects non-empty diagnostic-row crates
without it. Feature/example metadata on such a row is illustrative and may
not be cited as compiled evidence.

## Executable Evidence Provenance Schema

`hardening_3` raises each fixture manifest to `schema_version: 2`. Every
`evidence.positive` and `evidence.negative` record with `status: "passing"`
must contain:

```json
{
  "validation": {
    "profile": "create-pr",
    "step": "crate_tests",
    "suite_id": "sifr_driver_lib",
    "test_file": "crates/sifr_driver/src/build/rust_interop_contract_tests.rs",
    "test_name": "exact_rust_test_function_name"
  }
}
```

Rules:

- `profile` is one of `create-pr`, `merge`, `nightly`, or `release` and names
  the weakest mandatory local profile that executes the evidence.
- `step` is exactly `crate_tests`; Rust-interop metadata checks validate the
  mapping but do not masquerade as behavior evidence.
- `suite_id` must exist in that profile's `crate_test_membership`, be blocking,
  and be enabled by the profile's selected smoke/full crate-test mode.
- `test_file` is a repository-relative `.rs` file inside the package named by
  the suite, and `test_name` occurs exactly once as a Rust test in that file.
- the profile suite command plus `test_name` is the focused reproduction
  command; if the test is ignored, the selected suite command must itself
  include `--ignored`.
- each evidence side has its own test. A test may share helpers, but one test
  name may not certify both sides or multiple compatibility rows.
- planned/failing evidence must not carry passing validation provenance.
- all claimed-support categories require both sides to be passing and to have
  valid provenance. `unsupported-by-design` remains a claimed, passing
  diagnostic contract and follows the same rule.

`check_fixture_matrix.py` validates the schema and file/test existence.
`check_compatibility_matrix.py` loads the fixture manifests and rejects a
claimed-support row without two valid provenance bindings. Tests use temporary
mutated matrix/fixture/profile data and do not edit checked-in files.

## Structured Rejected-Syntax Contract

`hardening_4` removes lexical prefix inference from
`check_stale_drafts.py`.

- A fenced rejected example uses exactly ```` ```sifr-rejected ````.
- An inline prose/code occurrence of a stale spelling is allowed only when the
  same physical line contains
  `<!-- rust-interop-rejected -->`.
- Accepted Sifr examples use ```` ```sifr ```` and never inherit rejection
  state from surrounding paragraphs or headings.
- `python` fences containing Sifr Rust decorators remain errors.
- Review/archive path exclusions remain as currently defined.

The migration updates every accepted rejection example under `docs/`,
`internal_docs/`, and non-archived/non-review `plans/`. Tests must prove that
nearby words such as "no", "stale", or "rejected" no longer suppress a stale
example, while both explicit marker forms work.

## Ordered Items

### hardening_1: Execute the Rust-Interop Area in Authoritative Profiles

One PR:

- adds a `rust_interop_checks` step to the fixed
  `verification/runner/sifr_verify/profile_runner.py` legacy-facade sequence;
- implements it by reading the suites selected for `rust_interop` and calling
  the existing area runner, exactly as other explicit area steps do;
- adds `rust_interop` selections for `matrix`, `tiers`,
  `compatibility-matrix`, and `stale-drafts` to `create-pr`, `merge`,
  `nightly`, and `release`;
- adds a blocking positive create-PR step budget using a measured warm run;
  merge/nightly/release remain governed by their existing lane budgets unless
  those profiles adopt per-step budgets generally;
- adds profile-runner self-tests that fail if a normal legacy-facade profile
  selects `rust_interop` but omits the executable step, omits a required suite,
  or reports no Rust-interop result JSON; and
- updates the area README with direct and profile commands.

Exit gate: direct area execution passes, an emitted plan contains the area
selection, and all four profile dry/plan tests prove the step is scheduled.
`scripts/run_all_tests.sh --profile create-pr` must print
`name=rust_interop_checks ... status=pass`.

### hardening_2: Enforce Tier and Diagnostic-Crate Semantics

One PR:

- encodes the allowed-pair table above in `check_fixture_matrix.py` and its
  self-tests;
- adds a real `check_tiers.py --self-test` entrypoint that exercises temporary
  tier data and fails on missing/duplicate assignments, matrix/TOML mismatch,
  invalid tier names, and empty fixture lists rather than silently running the
  ordinary checked-in-data path;
- adds and cross-validates `diagnostic_crate_rationale`;
- migrates the two diagnostic rows named above;
- turns `same_workspace_crate` and `shared_bridge_crate` into real
  `cargo-probe` rows, including generated package builds and observed
  positive/negative results; and
- updates the fixture matrix, compatibility matrix, fixture manifests,
  evidence headers, tier descriptions, READMEs, architecture, and public docs
  in the same PR.

Mutation tests must reject every disallowed pair, missing/mismatched rationale,
a rationale on a non-diagnostic row, and a tier-1 row downgraded to
contract-only.

### hardening_3: Bind Every Passing Claim to an Executed Test

One PR:

- implements fixture manifest schema version 2 and the exact `validation`
  object above;
- adds distinct positive and negative Rust tests where a current README points
  only at a broad test module/filter;
- migrates all currently passing evidence records across all 34 rows;
- makes `check_fixture_matrix.py` validate suite/profile/file/test ownership;
- makes `check_compatibility_matrix.py` require valid two-sided provenance for
  all claimed-support categories;
- removes README text as validator input; READMEs remain explanatory and repeat
  the canonical structured test names; and
- adds self-tests for missing suites, wrong profile modes, non-blocking suites,
  missing/duplicate test names, ignored-test command mismatch, path escape,
  shared evidence tests, status/provenance mismatch, and a README-only passing
  claim.

Exit gate: every currently claimed row resolves to two executable tests, all
mutation tests pass, the full Rust-interop area passes through the
`rust_interop_checks` profile step, and no planned evidence is falsely bound.

### hardening_4: Replace Lexical Rejection Context

One PR:

- implements the `sifr-rejected` fence and same-line HTML marker contract;
- gives `check_stale_drafts.py --self-test` a real isolated temporary-tree
  entrypoint instead of falling through to the ordinary checked-in-data scan;
- migrates all current stale-syntax mentions in scan scope;
- removes `_is_rejection_context` and its broad lexical markers;
- adds isolated temporary-tree tests for accepted, rejected, malformed,
  nested-fence, adjacent-prose, `python`-fence, and scan-exclusion cases; and
- updates the Rust-interop README, architecture, and public authoring docs.

Exit gate: the scanner rejects an unmarked stale spelling even when nearby
prose says "no", "not", "stale", or "rejected"; both explicit rejected forms
pass; accepted Sifr and Python-fence behavior remains correct.

### hardening_5: Closeout

One documentation/review PR after `hardening_1` through `hardening_4`:

- records merged PRs and final row/schema counts here;
- confirms the runtime certification issue's entry criteria are unblocked;
- updates Phase 39 and roadmap follow-up links/status;
- runs a final Opus implementation-readiness review to satisfaction; and
- archives this issue only after all gates below pass.

## Implementation Progress

| Item | Status | Evidence |
| --- | --- | --- |
| `hardening_1` | merged | [PR #3018](https://github.com/sifr-lang/sifr/pull/3018), with the profile-evidence correction in [PR #3019](https://github.com/sifr-lang/sifr/pull/3019) |
| `hardening_2` | merged | [PR #3020](https://github.com/sifr-lang/sifr/pull/3020); final Opus review approved in round 7 |
| `hardening_3` | review approved; PR pending | all 34 fixture manifests are schema v2; all 47 passing evidence directions have distinct structured Rust-test provenance; Opus round 6 satisfied; create-PR lane passed |
| `hardening_4` | pending | starts only after `hardening_3` merges |
| `hardening_5` | pending | starts only after `hardening_4` merges |

## Acceptance Criteria

- Invalid tier/execution-kind pairs are rejected exactly per the frozen table.
- Diagnostic-only crate examples require matching structured rationale and
  cannot be cited as runtime/build evidence.
- Every passing evidence direction in every claimed-support row resolves to
  one exact Rust test executed by a blocking mandatory local profile.
- README-only evidence, broad module names, missing/ignored tests, and profile
  selections not executed by the legacy facade are rejected.
- Rejected stale syntax is explicit and structural; nearby lexical wording has
  no effect.
- Public and internal docs state the execution scope of contract-only,
  cargo-probe, and runtime-observed claims.
- The Rust-interop area runs in create-PR, merge, nightly, and release profiles.
- No user-triggerable panic path, fallback, test skip, or external-network
  dependency is introduced.

## Required Validation

Every item runs its focused self-test and the common local gates:

```bash
uv run --project verification --locked python -m sifr_verify areas run --area rust_interop
python3 verification/areas/rust_interop/checks/check_fixture_matrix.py --self-test
python3 verification/areas/rust_interop/checks/check_compatibility_matrix.py --self-test
python3 verification/areas/rust_interop/checks/check_tiers.py --self-test
python3 verification/areas/rust_interop/checks/check_stale_drafts.py --self-test
uv run --project verification --locked python -m sifr_verify --self-test
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_file_size_guardrails.py
git diff --check
```

Before opening a PR, run the create-PR lane. Before merge, run the default
merge gate. Do not wait on CI; local validation is authoritative.
