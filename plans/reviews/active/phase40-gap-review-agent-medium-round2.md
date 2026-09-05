I now have enough verified evidence to render the review. Let me compile the findings.

## Round-2 Implementation-Readiness Review — Rust Interop Follow-Up Plans

Re-inspected against the live repository: both revised target plans, `check_fixture_matrix.py`, `check_compatibility_matrix.py`, `check_tiers.py`, `check_stale_drafts.py`, both matrices + `tiers.toml`, `profile_runner.py`, all four authoritative profile JSONs, `check_sysroot_stdlib_resource_certification_gate.py`, `stdlib_retained_compiler_intrinsics.toml`, the `sifr_driver` contract-test files, the Native Pydantic-Sifr plan (`ps_2`/`ps_3`/bridge-v2), and the concurrent Phase 40 rewrite (read-only). No files modified.

---

### 1. Disposition of every round-1 finding

| R1 finding | Disposition | Evidence verified live |
|---|---|---|
| **BLOCKER 1** — no milestone/PR decomposition | **CLOSED** | Certification now has a Track A Row Contract table (`certification_1`–`13`) with exact positive/negative evidence IDs, tier, execution_kind, expected category, and special gate; ordered item blocks with dependency ordering; `certification_0` and `certification_14` bracket them. The 13 items map exactly onto the 11 live future-owned rows + 2 new runtime rows — no invented, no omitted rows (verified against the 11 future-owned IDs). |
| **BLOCKER 2** — runner wiring unspecified | **CLOSED** | `hardening_1` adds a `rust_interop_checks` step to the fixed legacy-facade sequence, reading `selected_suites_for_area("rust_interop")` and calling the area runner — exactly the pattern of `run_sysroot_release_checks` (`profile_runner.py:503-512`). All four profiles are `legacy-facade` (confirmed), so a step added to the `steps` list runs in all four. The four area suites `matrix/tiers/compatibility-matrix/stale-drafts` exist in `manifest.json`. Exit-gate string `name=rust_interop_checks ... status=pass` matches the actual `[sifr-lane-step]` output format (`:126`). |
| **BLOCKER 3** — completion breaks live gates | **CLOSED** | `certification_14` replaces the future-owned backstop in `check_sysroot_...:80-86`, relaxes the "all four categories used" rule in `check_compatibility_matrix.py:67`, and removes stale `future_owner`. My earlier manifest worry is moot: the live `stdlib_retained_compiler_intrinsics.toml` references only `opaque_resource_core` in `certification_rows` (not the ecosystem rows), so promoting ecosystem rows does **not** trip the per-surface check (`:66-72`). `REQUIRED_FIXTURES`/inventory updates are scoped in `certification_0`/`pkg` items. |
| **BLOCKER 4** — Pydantic blockers had no work item | **CLOSED** | `certification_2` (panic_boundary_wrapper_emission) and `certification_3` (callbacks_call_scoped) are first-class items; `certification_pkg_resource_core` owns `opaque_resource_package_core`. Ordering is acyclic and matches the Pydantic plan: certification_3 uses bridge-v1 → `ps_2` (bridge-v2) → `certification_pkg_resource_core` → `ps_3`. `ps_3` table (`:1960`) and `ps_2` (`:2038-2047`) line up with the certification dependency block (`:259-291`). |
| **BLOCKER 5** — hardening under-specified | **CLOSED** | (a) The frozen `(tier, execution_kind)` table is given and is coherent with all 34 live rows *after* the two named tier-1 migrations (`same_workspace_crate`, `shared_bridge_crate` → cargo-probe). (b) `diagnostic_crate_rationale` is named, cross-validated in three files, and applied to exactly the two live compiler-diagnostic rows carrying crates (`direct_crate_negative_type`, `blocking_diagnostics`). (c) The `validation` provenance object is concrete; `sifr_driver_lib` is a real blocking smoke+full suite and `crates/sifr_driver/src/build/rust_interop_contract_tests.rs` exists (plus sibling panic/callback/async/zero-copy/advanced-data contract-test files). (d) `sifr-rejected` fence + `<!-- rust-interop-rejected -->` marker replace `_is_rejection_context`, with migration scope + self-tests. |
| **MEDIUM 6** — contract-only overclaim untracked | **CLOSED** | `certification_0` adds future-owned `zero_copy_runtime_matrix` + `advanced_data_runtime_matrix`; the contract-only rows now carry `"scope":"contract-only"` in live evidence (verified), preserved as contract-only; `stable_support_claims.json` + a stable-candidate check are added. |
| **MEDIUM 7** — Phase 40 stable enforcement was prose | **CLOSED (residual, see NEW-1)** | Phase 40 (now `implementation-ready`) references the matrix in `milestone_40_0/1/4`, Validation Contract (`:520`), Quality Contract, and Exit Gate; `milestone_40_1` fails on advertising a future-owned surface. `certification_0` supplies the executable check + data. |
| **MEDIUM 8** — `unsupported-by-design` escape vs passing pos+neg | **CLOSED** | Objective now requires "passing positive and negative compiler-diagnostic evidence" and a dedicated rule block (`:34-37`, `:93-103`), matching `CLAIMED_SUPPORT_CATEGORIES` semantics (`check_compatibility_matrix.py:21,123`). |
| **LOW 9** — stale/retrospective terminology | **CLOSED** | "Completed Stdlib Native-Boundary Handoff" is past tense ("was split into"); `certification_14` converts it to durable historical wording. |
| **LOW 10** — hardcoded inventories drift | **CLOSED** | `certification_0` and `certification_pkg_resource_core` both name validator-inventory / `REQUIRED_FIXTURES` updates. No new `REQUIRED_CRATES` entries are needed for the two runtime rows (their crates already appear via the contract-only rows). |

All ten round-1 findings are genuinely closed, not merely mentioned.

---

### 2. Remaining / new findings (severity order)

**MEDIUM — NEW-1: Phase 40 ↔ certification/hardening stable-gate ownership and reverse-dependency is unreconciled.**
Phase 40 is `status: implementation-ready` and sequences `milestone_40_0 → 40_1` with **no declared dependency** on the hardening/certification issues. Yet `milestone_40_1` requires running the rust_interop `matrix/tiers/compatibility-matrix/stale-drafts` suites *in the release profile* (`40_...md:269-271`) and a stable-claim validation. Live release profile does **not** select `rust_interop` and `profile_runner` has no such step — that wiring only exists after `hardening_1`. Additionally, Phase 40 already binds "the exact public support claims derived from" the matrix into `stable-release-plan.json` (`:159-160`), while `certification_0` separately creates `stable_support_claims.json` + a stable-candidate check and says it will "Update Phase 40 `milestone_40_1` … to execute the stable-candidate check" (`cert:151-154`). Neither plan states which mechanism is authoritative, nor that Phase 40 `milestone_40_1` depends on `hardening_1` + `certification_0`.
*Concrete edit:* In both plans, state explicitly that Phase 40 `milestone_40_1`'s "rust_interop suites in release" + stable-claim gate depend on `hardening_1` (profile wiring) and `certification_0` (stable-candidate check + `stable_support_claims.json`), and that `certification_0` **extends** — not duplicates — Phase 40's release-plan claim binding. (Phase 40 itself is user-owned; do not edit it — reconcile from the two plans' side.)

**MEDIUM — NEW-2: `check_tiers.py --self-test` is a mandatory gate that no item implements and that currently passes vacuously.**
Both plans list `python3 verification/areas/rust_interop/checks/check_tiers.py --self-test` in "Required Validation" (`cert:302`, `hardening:245`). `check_tiers.py` has no argv handling (`main()` ignores `sys.argv`; `__main__` just `raise SystemExit(main())`), so `--self-test` silently runs the ordinary tier check and exits 0 — a false-pass. Tier/execution-kind enforcement is placed in `check_fixture_matrix.py` (hardening_2), so `check_tiers.py` never gains a self-test.
*Concrete edit:* Either add a real `--self-test` entrypoint to `check_tiers.py` as an explicit `hardening_2` deliverable, or remove the `check_tiers.py --self-test` line from both plans' validation blocks and keep only the plain `check_tiers.py` run.

**LOW — NEW-3: `certification_14`'s sysroot-guard change does not name the guard's own self-test.**
`certification_14` says "Replace the completion-time backstop in `check_sysroot_stdlib_resource_certification_gate.py`," but the guard's `--self-test` asserts that backstop fires (`:254-263`) and is executed on every profile in `core_guardrails` (`profile_runner.py:349-350`). Removing the backstop without updating that self-test breaks the guard.
*Concrete edit:* Add to `certification_14`: "update the guard's `--self-test` (the completed-matrix backstop assertion) in the same PR."

**LOW — NEW-4: `tokio-postgres`/`redis` "deterministic loopback protocol harnesses" (`certification_5`, `certification_6`) leave harness fidelity unspecified.**
The acceptance behavior is defined (close/aclose, double-close, alias/use-after-close rejection, cleanup), and the hermetic/loopback-only constraints are clear and consistent with the create-pr sandbox (`loopback_network: declared-only`) — so this is not a contradiction. But "how much wire protocol must be emulated" for `tokio-postgres` is the one place an implementer has real discretion to under/over-build within a single PR.
*Concrete edit (optional):* Add one sentence bounding the required protocol surface (e.g., "emulate only the request/response frames exercised by the certified operations; no general server compliance").

No contradictions were found in row-count language (34/34/11 verified exact), profile budgets (per-step budgets are opt-in; adding one create-pr key is consistent with `enforce_step_budget`), the schema-version split (fixture *manifests* → v2 with `check_fixture_matrix.py:275` updated in `hardening_3`; matrix files stay v1), the `scope` field (accepted by both validators today), or ignored-test handling (heavy runtime tests bind to `sifr_driver_generated_builds`, `--ignored`, full mode → profile `merge`, which is coherent).

---

### 3. Readiness verdicts

- **`rust-interop-runtime-ecosystem-certification.md`: READY.** Fully decomposed into ordered one-PR items with exact evidence IDs, promotion vs. `unsupported-by-design` criteria, canonical per-PR update list matching the live validators, closeout/guard-migration item, Pydantic and bridge-v2 sequencing that is acyclic and matches `ps_2`/`ps_3`, and a stable-claim gate. Residual issues are MEDIUM/LOW precision gaps (NEW-1/2/3/4), none requiring an implementer to invent structural scope.

- **`rust-interop-verification-matrix-hardening.md`: READY.** The four goals are now concrete and live-verifiable: the `(tier, execution_kind)` table is consistent with all 34 rows after the two named tier-1 migrations; `diagnostic_crate_rationale` maps to exactly the two live rows; the provenance schema binds to real, existing suites/files; the rejected-syntax markers + migration + mutation self-tests are specified; and `hardening_1` wires the area into all four profiles via a mechanism that matches `profile_runner`. Residual issues are NEW-1 (shared) and NEW-2.

---

### 4. Overall verdict

`SATISFIED`
