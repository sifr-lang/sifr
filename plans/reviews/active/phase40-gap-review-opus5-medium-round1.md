I have enough to complete the review. Here is my assessment.

---

# Implementation-Readiness Review — Rust Interop Follow-Up Plans

Reviewed against the live repository (matrix data, validators, profile runner, sysroot gate, Phase 39/40 plans, stdlib-boundary archive, and Pydantic-Sifr plan). No files were modified.

## Summary of state verified

- Compatibility matrix: 34 rows; fixture matrix: 34 fixtures; `REQUIRED_FIXTURES` hardcodes the same 34. Row↔fixture correspondence is currently 1:1 and enforced (`check_compatibility_matrix.py:66`, `:110`).
- 11 `future-owned-by-separate-phase` rows, all owned by the certification issue: `bridge_type_matrix`, `opaque_resource_matrix`, `panic_boundary_wrapper_emission`, `async_runtime_reqwest`, `callbacks_call_scoped`, `callback_subscription_ecosystem`, `ecosystem_backend_certification`, `ecosystem_cli_certification`, `native_build_script`, `proc_macro_trust`, `cargo_locked_offline`.
- **The `rust_interop` verification area is not wired into any authoritative gate profile.** `create-pr`, `merge`, `nightly`, `release` are all `legacy-facade`, none list `rust_interop` in `selected_areas`, and `profile_runner.py`'s step list (`:160-187`) has no rust_interop step. `check_fixture_matrix.py` / `check_compatibility_matrix.py` / `check_tiers.py` / `check_stale_drafts.py` are executed by no gate. Only `check_sysroot_stdlib_resource_certification_gate.py` (run in `core_guardrails`) reads the matrix, and it validates only stdlib-core rows plus a future-owned backstop.

---

## Findings (severity-ordered)

### BLOCKER 1 — Certification plan has no milestone/PR decomposition, ordering, or per-row acceptance
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md` is an inventory + handoff-contract, not an executable plan. It has a flat "Scope" bullet list (`:14-31`), a generic "Required Evidence" list (`:106-115`), and exactly **one** concretely decomposed item (`certification_pkg_resource_core`, `:86-103`). For the other 10 future-owned rows there is: no milestone, no PR-sized breakdown, no dependency ordering, no per-row positive/negative evidence spec, and no per-row promotion-vs-`unsupported-by-design` decision criterion. An implementer cannot execute rows like `bridge_type_matrix`, `native_build_script`, or `proc_macro_trust` without inventing scope, sequencing, and exit gates. **Blocks readiness for question 1, 3, 4.**

### BLOCKER 2 — Runner wiring is asserted but unspecified, and today's gate runs nothing
The `certification_pkg_resource_core` exit gate requires "adds the `rust_interop` area to the authoritative legacy profile-runner path used by create-PR, merge, nightly, and release" and that "`--profile create-pr` reports the Rust-interop area as executed" (`:96-103`). But the plan gives **no mechanism**: legacy-facade mode has a fixed step list (`profile_runner.py:160-187`) with no rust_interop step, and `selected_areas` is only consulted by specific hardcoded steps — a `rust_interop` selection would be silently ignored (exactly the trap the plan names but does not solve). Implementing this requires a new `run_rust_interop_area` step method, a `step_budgets` entry in all four profile JSONs, and `selected_areas` entries — none of which are scoped. The plan also does not state which tiers run in which profile. **Blocks readiness for question 5.**

### BLOCKER 3 — Completing the certification issue breaks two live gates; no guardrail update is planned
Two enforcement rules fail the moment the last future-owned row is promoted:
- `check_sysroot_stdlib_resource_certification_gate.py:80-86` fails with *"expected at least one runtime/resource compatibility row to remain future-owned; update this guard when resource certification lands"* once `_future_runtime_rows` is empty.
- `check_compatibility_matrix.py:67` fails with *"compatibility category is unused: future-owned-by-separate-phase"* because it requires all four categories to be used.

Neither plan mentions updating/removing these guards, updating the hardcoded `REQUIRED_FIXTURES`/`REQUIRED_CRATES` sets in `check_fixture_matrix.py:21-116` for the new `opaque_resource_package_core` row, or the closeout bookkeeping (Phase 39 file, roadmap, archive move). **Blocks readiness for question 8.**

### BLOCKER 4 — Pydantic-Sifr blockers exist with no work item that delivers them
The Pydantic handoff (`:70-84`) states `ps_3` is blocked until `opaque_resource_package_core`, `callbacks_call_scoped`, and `panic_boundary_wrapper_emission` land passing evidence here. Only `opaque_resource_package_core` has a decomposed item (`certification_pkg_resource_core`). `callbacks_call_scoped` and `panic_boundary_wrapper_emission` have no milestone/PR in the plan, so a documented downstream blocker has no owning work item. Additionally the ordering is subtle (`certification_pkg_resource_core` begins after Pydantic `milestone_ps_2`, yet Pydantic `ps_3` blocks on it) and the bridge-v2 dependency (`ad-hoc-native-pydantic-sifr-architecture.md:385-465`) is not reflected in this plan's sequencing. **Blocks readiness for question 3.**

### BLOCKER 5 — Hardening validator rules are under-specified (schema, tier mapping, provenance, marker)
`plans/issues/active/rust-interop-verification-matrix-hardening.md` names four goals but leaves each unimplementable-as-written:
- **Tier↔execution_kind** (`:16-17`): no allowed matrix is given. Live data has no clean mapping (tier 2 spans `runtime-observed`/`contract-only`/`cargo-probe`; tier 4 spans `contract-only`/`cargo-probe`). The plan must enumerate allowed pairs, or every existing row is at risk.
- **compiler-diagnostic rows with runtime crates** (`:18-20`): needs a concrete structured exemption field, but none is named. Existing `direct_crate_negative_type` (`required_crates:["regex"]`) and `blocking_diagnostics` (`["rusqlite","rayon","flate2"]`) would be rejected without a defined marker.
- **executable-evidence provenance** (`:21-23`): requires support rows to "point at executable evidence owned by a local validation lane." No such lane runs the rust_interop area (see Blocker 2), the evidence object has only an `id`/`status`, and contract-only supported rows (`zero_copy_bytes`, `callbacks_threadsafe`) have no runtime execution. The provenance mechanism (new field? lane assertion?) is undefined and currently unsatisfiable.
- **stale-draft structured marker** (`:24`): no marker syntax is specified to replace `_is_rejection_context` (`check_stale_drafts.py:69-87`), and there is **no migration section** for existing docs that rely on the lexical heuristic. No self-test/mutation requirement, no validation commands, no PR decomposition. **Blocks readiness for question 7.**

### MEDIUM 6 — Contract-only "supported" zero-copy/advanced-data rows have an untracked runtime deferral
Phase 39 `milestone_39_9`/`milestone_39_10` (`39_rust_interop.md:216`, `:231`) say runtime-observed certification for `bytes`/`memmap2`/`bytemuck`/`zerocopy` and `arrow`/`datafusion`/`polars`/`ndarray`/`candle` is "future-owned by the certification issue," but:
- the matrix rows (`zero_copy_bytes`, `zero_copy_view_matrix`, `arrow_record_batch`, `tensor_dlpack_bridge`, `advanced_data_matrix`) are categorized `supported`/`supported-through-bridge` (contract-only), **not** `future-owned`, so no row tracks the deferral; and
- the certification plan's Scope (`:14-31`) does not list zero-copy or advanced-data runtime certification at all.

Consequence: the Phase 40 stable constraint (`:118-121`) keys only on `future-owned-by-separate-phase`, so these contract-only rows would pass the stable gate and could be advertised as "supported" with no runtime evidence — an overclaim path the plans do not close. **Answers question 2: they coexist but the deferral is untracked and unowned.**

### MEDIUM 7 — Phase 40 stable enforcement is prose, not an executable gate
The certification plan states Phase 40 "must not claim support for any surface that remains future-owned" (`:118-121`), but Phase 40 (`40_stable_channel_ga_promotion...md`) never references the compatibility matrix in any milestone or DoD, and no gate script enforces the constraint (the existing sysroot gate enforces the stdlib invariant only). There is no executable check that fails stable promotion when a future-owned Rust surface is advertised. **Blocks readiness for question 6.**

### MEDIUM 8 — `unsupported-by-design` is treated as a light escape but the validator demands passing pos+neg evidence
The objective (`:9-12`) offers "or an explicit unsupported-by-design decision" as an alternative to passing support evidence, but `check_compatibility_matrix.py:21,123` includes `unsupported-by-design` in `CLAIMED_SUPPORT_CATEGORIES` — it requires **passing positive and negative** fixture evidence. So choosing `unsupported-by-design` still requires authored, passing diagnostic fixtures both directions. The plan's evidence semantics do not reflect this. **Contributes to question 4.**

### LOW 9 — Stale/retrospective terminology in the "Handoff to Stdlib" section
The certification plan's stdlib handoff (`:33-68`) is written in present/future tense ("splits into", "may split", "async_runtime_reqwest split in M6") but the stdlib-boundary phase is **completed and archived** (`archive/ad-hoc-stdlib-native-boundary-completion.md:5`); those splits already exist in the matrix (`opaque_resource_core`, `async_runtime_core`, `callback_subscription_core`). An implementer could read still-pending work into an already-finished handoff. Recommend converting to past tense / "already split" notes.

### LOW 10 — No row-count drift, but hardcoded inventories will drift on the one planned addition
Current counts are consistent (34/34/34). The single planned new row `opaque_resource_package_core` will require synchronized edits to `REQUIRED_FIXTURES` and `REQUIRED_CRATES` (`check_fixture_matrix.py:21-116`); the plan's package_core item mentions "updates the durable Rust-interop fixture inventory" (`:97-98`) but not the Python guard sets, risking a validator failure mid-implementation.

---

## Proposed plan structure / edits to close every material finding

**A. Restructure the certification issue into ordered milestones (Blocker 1, 4).** Add a `## Milestones` section with one milestone per future-owned row (or tight cluster), each specifying: the row id, exact positive fixture (path `fixtures/<id>/positive/<evidence-id>.sifr`), exact negative fixture, execution_kind, target category after promotion, the crate feature pins (already in Phase 39 `:334-346`), and the promotion-vs-`unsupported-by-design` decision criterion. Give explicit ordering and a dependency table. Add first-class milestones for `callbacks_call_scoped` and `panic_boundary_wrapper_emission` (the Pydantic blockers) and state their required-before relationship to Pydantic `ps_3`. Record the bridge-v2 dependency for `opaque_resource_package_core`.

**B. Specify runner integration concretely (Blocker 2).** Add a milestone that: introduces a `run_rust_interop_area` step in `profile_runner.py` (or a `selected_areas`-driven step that actually executes), adds `rust_interop` to `selected_areas` and a `step_budgets` entry in `create-pr/merge/nightly/release`, and states the tier→profile policy (e.g., tier 0–1 in create-pr; runtime-observed loopback/local-service rows in nightly/release only). Define how `redis`/`tokio-postgres` local services are provisioned under each profile's `network_policy`/`execution_sandbox` (create-pr forbids external network, loopback declared-only), or tier-gate them out of create-pr explicitly.

**C. Add a closeout/guardrail-migration milestone (Blocker 3).** Enumerate the guard edits triggered by promotion: update/remove the future-owned backstop in `check_sysroot_stdlib_resource_certification_gate.py:80-86`; relax the "all four categories used" rule in `check_compatibility_matrix.py:67` (or keep one `unsupported-by-design`/`future-owned` row intentionally); update `REQUIRED_FIXTURES`/`REQUIRED_CRATES`; rewrite `fixture.json` `expected_result`/`status` markers per `check_fixture_matrix.py:496-519`; and the bookkeeping (Phase 39 note, roadmap, archive move).

**D. Rewrite the hardening issue with concrete schema rules (Blocker 5).** For each of the four goals, add: (1) the exact allowed `(tier, execution_kind)` pairs, validated against all 34 current rows; (2) the named exemption field (e.g. `diagnostic_crate_rationale`) and which existing rows carry it; (3) the executable-evidence provenance mechanism (a required field binding each support row to a runner-executed lane — which presupposes Blocker 2's wiring, so state the dependency); (4) the stale-draft structured marker syntax (e.g. an explicit HTML-comment/prefixed token) plus a migration step over existing `docs/`, `internal_docs/`, `plans/` occurrences. Add required `--self-test`/mutation coverage and exact validation commands, and split into ordered PRs.

**E. Close the contract-only overclaim gap (Medium 6, 7, 8).** Either (a) add explicit `future-owned` rows for zero-copy/advanced-data *runtime* certification and list them in certification scope, or (b) add a documented `evidence_scope: contract-only` field the stable gate treats as "not fully supported." Add an executable Phase 40 gate (a script in `milestone_40_1` or `core_guardrails`) that fails stable promotion if any advertised Rust surface is `future-owned` or contract-only-without-runtime. Correct the objective wording so `unsupported-by-design` reflects the passing pos+neg requirement.

**F. De-stale the stdlib handoff section (Low 9, 10)** to past tense and note the guard-inventory edits the one planned row requires.

---

## Readiness verdicts

- **`rust-interop-runtime-ecosystem-certification.md`: NOT READY.** Only one of ~11+ rows is decomposed; no milestones, ordering, per-row evidence/criteria, runner mechanism, network/resource policy, guard-migration, or closeout. Blockers 1–4, 6–8 apply.
- **`rust-interop-verification-matrix-hardening.md`: NOT READY.** Four scope bullets with no schema rules, no tier mapping, no provenance mechanism, no marker syntax, no migration/self-test/validation-command detail, no PR decomposition; its central "local validation lane" requirement is unsatisfiable until the area is wired into a gate. Blocker 5 (and Blocker 2 dependency) apply.

## Overall verdict

`NOT SATISFIED`
