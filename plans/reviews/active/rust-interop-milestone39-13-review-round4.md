# Rust Interop M39.13 — Round 4 Review

Scope: re-verify M39.13 closeout after the post-round-3 taxonomy wording fixes
and validation-stability helper edits. All local validations the user listed are
re-checked here against the actual diffs rather than re-run.

## Findings (severity-ordered)

### No blockers

Round 4 introduced no new structural, taxonomy, or evidence regressions. The
remaining items below are all low-severity carry-overs or cleanup
recommendations.

### Low — Architecture `Panic Surface Policy` section still lacks an active-issue redirect

`internal_docs/rust_interop_architecture.md:522` ends the panic surface section
with: "Full generated wrapper emission owns mapper signature validation and
mapper-panic fallback behavior." Unlike the zero-copy (`:661`), advanced data
(`:678-682`), and callbacks (`:738-744`) sections — all of which now route
future-owned scope through
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md` — this
sentence does not name the active issue. The phase file M39.8 status
(`plans/phases/39_rust_interop.md:196`) already does the redirect via
`panic_boundary_wrapper_emission`, so the compatibility matrix and phase plan
remain consistent. Not blocking M39.13 closeout, but worth aligning before
phase-level review so the architecture doc parallels the other three sections.

Suggested wording (advisory): replace `:522` with `"The initial compile-time
panic contract validates the public panic surface and `panic=map_error(path)`
shape. Full generated wrapper emission, mapper signature validation, and
mapper-panic fallback behavior are future-owned by
[`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`](../plans/issues/active/rust-interop-runtime-ecosystem-certification.md)
through the `panic_boundary_wrapper_emission` compatibility row."`

### Low — `editor_integrations` submodule pointer is `-dirty`

`git status` reports `editor_integrations` at
`eab2cca55799654396c311f1f1709216301f5404-dirty`; the nested `vscode` submodule
inside it has an uncommitted `c5b1e0d` bump. This is unrelated to M39.13 but
should not ride along on the closeout commit. Either commit the nested bump
deliberately (in a separate PR for editor integrations) or restore the
submodule to the recorded commit before opening the M39.13 PR. Not a M39.13
correctness blocker, but the PR diff will look noisy and may trip submodule
checks on CI.

### Carry-over from round 3 (unchanged, explicitly not in round 4 scope)

- `same_workspace_crate` / `shared_bridge_crate`: `tier=1` with
  `execution_kind=contract-only`. The fixture matrix check still does not
  cross-validate tier against execution_kind.
- `blocking_diagnostics`: `tier=0`, `execution_kind=compiler-diagnostic`, but
  `required_crates=["rusqlite", "rayon", "flate2"]` with feature pins.
- `check_stale_drafts.py` rejection-context detector remains permissive.
- README-only fixtures with `status: "passing"` are still treated as ground
  truth for `supported` rows by the compatibility validator (it only verifies
  that fixture-matrix evidence is `passing` and that future-owned rows have an
  active owner; it does not invoke cargo for `supported` rows). For the rows
  marked `supported`/`supported-through-bridge`, the round-3 review confirmed
  the evidence pointers either name concrete driver tests
  (e.g. `zero_copy_bytes` README cites
  `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs::package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract`)
  or the `runtime-observed` / `contract-only` execution kinds the matrix is
  comfortable claiming. No new round-4 regressions on this front.

These were medium/low in round 3 and explicitly carried as not-blocking
phase-level review.

## Direct answers to the asked inspection items

### 1. Compatibility matrix and docs avoid overclaiming runtime/ecosystem support

Confirmed.

- `verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`
  lists 10 rows as `future-owned-by-separate-phase`
  (`bridge_type_matrix`, `opaque_resource_matrix`,
  `panic_boundary_wrapper_emission`, `async_runtime_reqwest`,
  `callbacks_call_scoped`, `callback_subscription_matrix`,
  `ecosystem_backend_certification`, `ecosystem_cli_certification`,
  `native_build_script`, `proc_macro_trust`, `cargo_locked_offline`). Every one
  declares `future_owner` =
  `plans/issues/active/rust-interop-runtime-ecosystem-certification.md`, and
  `check_compatibility_matrix.py:127-138` validates that path exists, lives
  under `plans/issues/active/` or `plans/phases/`, and that the row does not
  already have passing positive+negative evidence.
- `docs/rust-interop-compatibility.mdx` has a dedicated `## Future-Owned` table
  (`:50-69`) that names every future-owned surface, explicitly says they are
  "not listed as verified support" and "must not be advertised as stable Rust
  interop support while they remain future-owned." The `## Supported` and
  `## Supported Through Bridges` tables match the compatibility JSON rows
  one-to-one with `category in {supported, supported-through-bridge}`.
- `docs/rust-interop.mdx` examples (async HTTP, zero-copy Arrow/DLPack,
  callbacks) document declaration *shape* — none of them claim runtime
  certification against `reqwest`, `arrow`, `tokio-tungstenite`, etc. They are
  pinned by the contract-level evidence the matrix actually has. The
  cross-link to `/rust-interop-compatibility` at `docs/rust-interop.mdx:11-13`
  routes readers to the future-owned disclosures before they reach the
  examples.

### 2. Forbidden delivery-plan wording in verification/docs

Confirmed clean.

`grep -inE 'closeout|staged|follow-up'` across `verification/areas/rust_interop/`,
`docs/rust-interop.mdx`, `docs/rust-interop-compatibility.mdx`,
`internal_docs/rust_interop_architecture.md` returns zero matches outside the
allowed plans/issues prefix. The previous "remains staged for ecosystem
closeout" and "remain owned by ..." sentences in the architecture doc (former
`:661`, `:678-682`, `:738-744`) and the M39.6/7/8/9/10/11 status lines were
rewritten to "future-owned by `plans/issues/active/...`." `closeout` and
`follow-up` still appear in `plans/phases/39_rust_interop.md` and
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`, which is
expected: `verification/areas/coverage_matrix/checks/verification_taxonomy.py`
scopes `ACTIVE_ROOTS` to `verification/`, `.github/workflows/`, `crates/`,
`demos/`, `docs/`, `editor_integrations/`, `internal_docs/`, `lib/`, and
`scripts/` only — `plans/` is exempt, so delivery-plan wording there is
intentional and does not trip the guardrail.

### 3. M39.4/6/7/8/10/11/13 statuses route future-owned scope to the active issue

Confirmed.

| Milestone | Phase file line | Redirect verbiage | Compatibility row |
| --- | --- | --- | --- |
| M39.4 | `:142` | "future-owned by [active issue] and is not claimed as Phase 39 support until both evidence directions pass" | `bridge_type_matrix` |
| M39.6 | `:162` | "future-owned by [active issue] through the `opaque_resource_matrix` compatibility row" | `opaque_resource_matrix` |
| M39.7 | `:177`, `:184` | "future-owned by [active issue] through the `async_runtime_reqwest` compatibility row" + explicit defer of borrowed-input wrappers | `async_runtime_reqwest` |
| M39.8 | `:196` | "future-owned by [active issue] through the `panic_boundary_wrapper_emission` compatibility row" | `panic_boundary_wrapper_emission` |
| M39.10 | `:227` | "future-owned by [active issue] and is not claimed as Phase 39 support until both evidence directions pass" | `advanced_data_matrix` (and architecture `:678-682` for `arrow`, `datafusion`, `polars`, `ndarray`, `candle`) |
| M39.11 | `:240` | "future-owned by [active issue] through the `callbacks_call_scoped` and `callback_subscription_matrix` compatibility rows" | both callback rows |
| M39.13 | `:269` (milestone name) + `:385-387` (validation-planning goals tail) | "Future-owned runtime/ecosystem certification rows are tracked by [active issue] and are not Phase 39 supported surfaces until both evidence directions pass" | N/A (it owns the matrix itself) |

The active issue
(`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`) lists
every future-owned surface in its Scope section, asserts the Phase 40 stable
promotion constraint at the bottom, and exists at the path the matrix
validator and architecture doc reference. No dangling milestones.

### 4. Validation helper edits

All three helpers now share the same three-tier resolution: env override →
`target/debug/sifr` if pre-built → `cargo run` fallback.

- `verification/runner/sifr_verify/audit_fixtures.py:169-175`:
  `SIFR_AUDIT_FIXTURE_BIN` → `target/debug/sifr` → `cargo run --locked -q -p sifr --`.
  The fallback preserves `--locked`, matching the pre-edit behavior.
- `verification/areas/stdlib_parity/tools/check_stdlib_module_parity.py:231-237`:
  `SIFR_STDLIB_MODULE_BIN` → `target/debug/sifr` → `cargo run --locked -q -p sifr --`.
  Same shape, `--locked` preserved.
- `verification/areas/developer_tooling/check_rule_suppression_rules.py:28-34`:
  `SIFR_RULE_SUPPRESSION_BIN` → `target/debug/sifr` → `cargo run -q -p sifr --`.
  No `--locked` here, but that matches the pre-edit fallback (confirmed via
  `git show HEAD:.../check_rule_suppression_rules.py` showing the original
  unflagged `cargo run`). Not a regression introduced by this round.

The pattern is acceptable for M39.13 closeout: the env override gives CI/local
runs deterministic targeting, the `target/debug/sifr` short-circuit avoids
re-invoking cargo when a debug build is already present (which is the warm
`create-pr` profile case), and the cargo fallback keeps cold runs working.
All three helpers are called from outside the compiler workspace and have no
ownership of build configuration, so the binary-reuse path is purely an
optimization, not a semantics change.

One observation, not a finding: the suppression helper at `:18` still uses
`subprocess.run(..., capture_output=True)` without an explicit timeout — pre-existing,
unchanged by this round, and outside M39.13 scope.

### 5. `doctest = false` on `sifr_analysis` and `sifr_lint`

Acceptable for M39.13. Re-verified the round-3 finding:

`grep -rn '```' crates/sifr_analysis/src crates/sifr_lint/src` returns zero
matches, so the `[lib] doctest = false` toggle in
`crates/sifr_analysis/Cargo.toml:18-20` and `crates/sifr_lint/Cargo.toml:19-21`
suppresses an empty `rustdoc --test` invocation rather than skipping real
doctests. The narrow scope (only these two crates, both with the
documented `create-pr` wedge as motivation) keeps the change reversible if a
future doc-comment example is added — adding the first ` ``` ` block and
removing the toggle is a single PR.

## Satisfaction and gating

**M39.13 milestone PR**: I am satisfied. Round 4 edits closed the wording
residual from round 3, the helper edits are well-scoped and consistent, and
the compatibility matrix + docs do not overclaim. The two non-blocking items
above (architecture `:522` redirect, editor_integrations submodule) do not
need to land in M39.13's PR.

**Final full Phase 39 review**: Nothing in round 4 blocks proceeding. For
phase-level review I'd still focus on the three items round 3 named:
(a) the active follow-up issue must stay tracked outside Phase 39 so it
survives the phase boundary — currently `plans/issues/active/` lives outside
the phase doc, so this is satisfied; (b) the architecture `:522` redirect
should land before final phase signoff so all four future-owned narrative
sections (panic, zero-copy, advanced data, callbacks) read consistently;
(c) the carry-over tier/execution_kind cross-validation gap should be
captured as a follow-up against verification tooling rather than reopened
against Phase 39.
