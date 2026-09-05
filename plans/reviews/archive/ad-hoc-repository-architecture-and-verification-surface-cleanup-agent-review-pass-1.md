I have everything I need. Here is the review.

---

# Review: Ad Hoc Repository Architecture And Verification Surface Cleanup

**Verdict up front: approve with changes.** The core architecture — areas own fixtures, lanes select areas, one public facade, stdlib-only runner, generated reports under `target/` — is sound and genuinely production-grade. But the plan has one missing load-bearing PR (the facade cutover), one internal contradiction it cannot land without resolving (transitional coexistence vs. "no duplicate layouts"), and one named-but-undispositioned surface (`crates/sifr/tests/verification/`). These are fixable in the doc; nothing requires redesigning the shape.

## 1. Severity-ordered findings

### Blocker-level (the plan cannot land as written)

**F1. The `run_all_tests.sh` cutover has no PR.** `run_all_tests.sh` is today a 481-line shell orchestrator that hard-wires dozens of individual check invocations (`scripts/run_all_tests.sh:128-189`) and sources lane config via `scripts/validation_lane.py`. The acceptance criteria demand it "delegates directly to the verification runner," but no PR in the sequence performs that rewiring. PR 6 builds the runner, PR 7 splits lanes, PRs 8–N migrate areas — and at no defined point does the facade stop being the orchestrator and become a thin wrapper over `sifr_verify`. This is the single riskiest change in the whole phase (it redefines the merge gate and CI mirrors it), and it's invisible. Add an explicit **Facade Cutover PR** (logically after the first few area migrations, before PR N+2) with side-by-side equivalence evidence: same checks executed, same exit-code semantics, same report shape, per lane.

**F2. Internal contradiction: "No duplicate old and new verification layouts" (Non-Goals) vs. a 15+ PR incremental migration.** From PR 6 until PR N+2, the repo necessarily contains both the new runner/areas and the legacy `scripts/` + loose `verification/` material — that *is* a duplicate layout. As written, every migration PR violates a stated non-goal. Scope the non-goal to the end state and explicitly authorize transitional coexistence, governed by a migration-status table (which area has cut over, which legacy path still gates). Without this, reviewers of mid-train PRs have no rule to apply.

**F3. `crates/sifr/tests/verification/` is named in the Problem statement and then never dispositioned.** The current `verification/suites/manifest.json` points its case entries at `crates/sifr/tests/verification/diagnostics/...`, `project/...`, `crashes/`, `package/`. The plan's "every retained fixture has exactly one owning area manifest" rule is unsatisfiable while those fixtures live in a crate the plan doesn't mention. Decide explicitly: either (a) crate-local fixtures move into `verification/areas/<area>/fixtures/` and the Rust tests reference them by relative path, or (b) crate-internal unit-test fixtures are formally exempted from area ownership and the suites manifest entries migrate. Either answer is defensible; silence is not.

### High

**F4. Stdlib-only Python vs. JSON Schema validation is an unacknowledged conflict.** The plan commits to five `*.schema.json` files *and* a runner with "no external Python package dependency" — but JSON Schema validation in Python requires the third-party `jsonschema` package. Hand-rolling a full JSON Schema validator is its own project. Resolve in the doc: commit to a small, explicitly enumerated schema subset (required keys, types, enums) validated by hand-written stdlib code, and say so in the schemas section. Otherwise PR 6's "schema self-tests" validation step is undefined.

**F5. PR 3 (Cursor cleanup) targets a `plans/` layout that doesn't exist until PR 5.** PR 3 says Cursor commands are updated "for the planned `plans/` layout," leaving commands pointing at nonexistent paths for two PRs — and PR 5's own validation ("all AGENTS/Cursor references resolve") forces a second Cursor edit anyway. Split it: PR 3 becomes portability-only (personal paths, `.DS_Store`, `.obsidian`, `.cursor/.rules/`, skill consolidation); path retargeting moves into PR 5 atomically with the tree move.

**F6. Guardrails-vs-verification boundary is fuzzy and will be litigated in every migration PR.** `scripts/` keeps "source and repository guardrails" while "no verification implementation remains in `scripts/`" — but `check_hir_maintainability_guardrails.py`, `check_diagnostic_docs_sync.py`, `check_source_crate_dependency_direction.py` etc. are verification by any reasonable reading, and several `check_diagnostic_*` scripts map directly to the proposed `diagnostics` area. Add a one-sentence tie-breaker, e.g.: *"A check that gates first-party source/repo hygiene and needs no compiled artifact is a guardrail (`scripts/`); a check that executes or inspects compiler behavior or output is verification (area-owned)."* Then classify the ambiguous diagnostics scripts in PR 1B against that rule.

### Medium

**F7. The `determinism` area conflates two different things.** The plan states "sequential/parallel equivalence is a runner self-check, not an ad hoc shell script" — and then creates an *area* that owns "runner determinism self-checks." Runner self-checks belong to `runner/` (tested like any other code); flake/quarantine data and report-signature baselines are policy-adjacent data. Either fold determinism into `runner/` self-tests plus `policy/` data, or narrow the area's charter to *compiler-output determinism* (deterministic emitted Rust, deterministic diagnostics ordering) and move runner self-verification out of it.

**F8. Repeated ownership hedges signal a missing tie-breaker rule.** "Unless a case is specifically stdlib parity" appears three times in the audit disposition table; `runtime_platform` vs `stdlib_parity` and `regression` vs `core_language` boundaries will generate the same argument repeatedly. Add one rule to the area table: *ownership follows the contract being asserted, not the feature being exercised* — a fixture asserting CPython-observable behavior is `stdlib_parity`; one asserting compile-time semantics is `core_language`; a minimized reproducer of a fixed bug is `regression` regardless of domain.

**F9. PR 1 and PR 1B are the same artifact split bureaucratically.** A "disposition table" (PR 1) and a "keep/move/rewrite/delete relevance audit" (PR 1B) over largely the same surfaces, both no-op PRs, differ only in column count. Merge into one inventory PR with the richer row format. Also drop the "1B" numbering — renumber cleanly.

**F10. Two files named `roadmap.md` with different meanings.** `plans/roadmap.md` (execution roadmap) and `plans/phases/roadmap.md` (phase status index) will be confused in links and in conversation. Rename the latter `plans/phases/index.md`.

**F11. `plans/issues/{active,completed,archive}` has two terminal states; reviews get only two states.** The completed/archive distinction (done vs. superseded) is real but will be misfiled constantly, and the asymmetry with `plans/reviews/{active,archive}` is arbitrary. Collapse issues to `active/` + `archive/` and record `status: completed | superseded | abandoned` in the doc header — the plan already establishes header-status as the mechanism for phases, so reuse it.

**F12. Skill-naming rule contradicts its own example.** "Skill names should describe workflow intent, not a transient model brand" — followed by "Keep `agent review`." The review workflow should use one neutral agent-review process rather than model-specific variants.

**F13. `.cursor/plans/` is silently dropped.** The current `.cursor/` contains `plans/` (with embedded `.obsidian/` state); the target shape lists only `commands/`, `references/`, `skills/` and the rules never disposition `.cursor/plans/` itself. Add an explicit row — presumably its contents move to `plans/` or are deleted.

**F14. Cargo.lock wording is factually off.** `Cargo.lock` is currently **ignored and untracked** (`git ls-files Cargo.lock` is empty; `.gitignore` lists it). "Keep `Cargo.lock` in the root tracked set" should read "begin tracking `Cargo.lock`" — and PR 2 should note the consequences the current text omits: a large initial commit, and a contributor-visible behavior change (lockfile diffs now appear in PRs and need a review convention).

### Low

**F15. E2E discovery-reorder risk needs a mitigation, not a bullet.** The Risks section notes fixture moves can reorder lexicographic e2e discovery and invalidate snapshots, but no migration-PR validation step addresses it. Add to the PR 8-N checklist: regenerate/verify snapshot stability after each fixture move, with declaration-order expectations re-pinned.

**F16. Area-local ceremony should declare its minimum.** Eight optional subdirectories per area (`suites/ fixtures/ corpora/ baselines/ data/ runner.py README.md`) is fine for `stdlib_parity` and absurd for `package_management` (five submodules and a manifest). State explicitly: only `manifest.json` is mandatory; create subdirectories on first use.

**F17. PR 8-N candidate order is reasonable but unjustified.** Starting with `diagnostics`/`project_workspace` (small, manifest-backed already) before `core_language` (e2e runner, highest blast radius) is the right risk ramp — say so in one sentence, so a future executor doesn't "optimize" the order.

## 2. Specific recommended edits to the phase doc

1. **Add a "Facade Cutover" PR** between the early area migrations and PR N+2: rewrite `run_all_tests.sh` as a thin lane dispatcher over `sifr_verify`, with side-by-side equivalence evidence (checks executed, exit codes, report shape) per lane. Update the acceptance criterion to reference it. (F1)
2. **Rescope the Non-Goal** to: "No duplicate old/new verification layouts *in the end state*; transitional coexistence during PRs 6–N+2 is tracked in a migration-status table." (F2)
3. **Add a disposition subsection for `crates/sifr/tests/verification/`** choosing (and justifying) area-migration vs. explicit exemption, and updating the "exactly one owning area manifest" rule accordingly. (F3)
4. **Amend the Verification Architecture rules**: schemas are validated by a hand-written stdlib validator over an enumerated schema-feature subset; name that subset. (F4)
5. **Split PR 3** into portability-only; move Cursor path retargeting into PR 5. (F5)
6. **Add the guardrail/verification tie-breaker sentence** to the Scripts Cleanup section. (F6)
7. **Narrow or dissolve the `determinism` area**; move runner self-checks under `runner/`. (F7)
8. **Add the ownership tie-breaker rule** ("ownership follows the asserted contract") above the area table; delete the three "unless stdlib parity" hedges in favor of it. (F8)
9. **Merge PR 1 and PR 1B**; renumber the sequence. (F9)
10. **Rename `plans/phases/roadmap.md` → `plans/phases/index.md`.** (F10)
11. **Collapse `plans/issues/` to `active/` + `archive/`** with header status. (F11)
12. **Rename the consolidated review skill** to an intent-based name; drop the "keep agent review" wording. (F12)
13. **Add an explicit `.cursor/plans/` disposition row.** (F13)
14. **Fix PR 2 wording** to "begin tracking Cargo.lock" and note the contributor-facing consequences. (F14)
15. **Add snapshot-stability verification** to every PR 8-N checklist. (F15)
16. **State that `manifest.json` is the only mandatory area file.** (F16)

## 3. Taxonomy assessment

The areas/lanes/schemas/runner/policy five-concept top level is the right shape — better than the current suites/lanes/loose-domain-dirs mix, and the "lanes select, areas own" invariant is the strongest single idea in the doc. Keep it. Two adjustments to the area table:

- **14 areas, not 15**: fold `determinism` per F7. The remaining 14 map cleanly onto existing directories (`fixedbugs`+`crashes`→`regression`, `oss`→`ecosystem_compatibility`, `tooling`→`developer_tooling`, `perf`+`performance`→`performance`, etc.), which is evidence the taxonomy is descriptive rather than aspirational — good.
- I considered merging `algorithmic_compatibility` into `ecosystem_compatibility` as a single external-corpora area, but their gating semantics differ (scored blocking corpus vs. non-blocking signal) and lanes select per-area, so keeping them separate is the simpler lane story. No change recommended.

The `plans/` top level alongside `internal_docs/` is the correct split (state vs. execution), and moving `internal_docs/verification/` policy material under `verification/policy/` eliminates a real half-state — the seven policy docs currently there are exactly the machine-adjacent material the plan says they are.

## 4. Final verdict

**Approve with changes.** The architecture and end-state contract are serious and coherent; the audit disposition table is unusually rigorous (per-path, per-rule, with coverage-proof preconditions for deletion — this is the best section of the doc). The required changes are concentrated in execution sequencing: add the facade-cutover PR (F1), resolve the coexistence contradiction (F2), and disposition `crates/sifr/tests/verification/` (F3). Those three are blocking; the rest are doc edits that prevent predictable churn during the ~18-PR train. No structural redesign is warranted.
