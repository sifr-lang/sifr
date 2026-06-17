I have enough evidence to deliver the review. Read-only inspection complete — taxonomy guard self-test passes, but the cleanup leaks synthetic delivery labels via patterns the guard's regex doesn't catch.

## Review verdict: another round required

The taxonomy guard passes only because its regexes require trailing digits or specific tokens (`contract_psp_*`, `contract_<digit>`, `M\d+`). The cleanup replaced delivery-plan labels with **digit-less synthetic substitutes** that the guard cannot catch but that match the spirit of what the user rejected.

### Blocking issues

**1. `internal_docs/architecture.md` — wholesale `milestone_<name>` → `contract_<name>` substitution.**
At least 30+ instances on lines 65, 175–198, 298, 318, 321, 402, 408, 411, 412, 440, 447, 452–458, 462, 478, 479, 730–732. Examples: `contract_safe_indexing`, `contract_borrow_default`, `contract_ergonomics`, `contract_protocols`, `contract_generics`, `contract_metaprogramming`, `contract_error_handling`, `contract_collection_safety`, `contract_auto_init`, `contract_classes`, `contract_ffi`, `contract_ecosystem`, `contract_core_stdlib`, `contract_ext_collections`. These are not API/schema/diagnostic contracts — they are 1:1 renames of the old `milestone_*` anchors. This is the exact "synthetic contract labels as a disguised replacement" pattern the user rejected. The taxonomy guard misses them because its `contract_` regexes all require a trailing digit.

**2. `internal_docs/architecture.md` — "task ownership surface0"…"task ownership surface7" (lines 309–317).** Mechanically substituted for the original M10…M17 labels. These are synthetic delivery labels disguised as architecture nouns. `grep` finds 12 occurrences in this one file.

**3. "task ownership surface" / "architecture guardrails" leakage across 41 active-surface files** (verification reports, internal_docs, demos). Examples that produce nonsense:
- `demos/compiler_api/main.sifr:4` → `print("driver task ownership surface api spine demo: 42")`; the same string is embedded in `emitted.rs` and `idiomatic.rs`.
- `demos/rooted_entrypoint/shared.sifr:4` → returns `"adhoc task ownership surface rooted entrypoint demo: pass"` (and matching `idiomatic.rs`).
- `demos/generic_functions_and_iterators/main.sifr:4` → `# Milestone task ownership surface9: Audit Fix-Up Demo` — still has "Milestone" right next to the substituted phrase.
- `demos/nested_functions/main.sifr:7` → `# by the Sifr compiler after the Language Hardening task ownership surface1.`
- `demos/core_language/main.sifr:4` → `# Sifr task ownership surface Demo: Core Language`.
- `verification/areas/stdlib_parity/reports/network_http_cpython_evidence_matrix.md:9` → `task ownership surface mines loopback TCP, address formatting…` (subject of a sentence).
- `internal_docs/typescript_go_architecture_transfer_guardrails.md` is the worst offender: bulk substitution made `M1` → `architecture guardrails`, producing strings like `the actual pre-session state after architecture guardrails and before architecture guardrails-architecture guardrails behavior migration`, `Not implemented in architecture guardrails.`, `Permitted architecture guardrails exceptions:`, `## architecture guardrails Disposition`, `## Historical architecture guardrails LSP Reality And architecture guardrails Update`.

**4. `internal_docs/architecture.md:134` heading broken by substitution.** `### Phase 31.5 Governance Artifact` became `### driver/package architecture.5 Governance Artifact` — `architecture.5` is no longer a coherent section title.

**5. Taxonomy guard gap (false negative).** `verification/areas/coverage_matrix/checks/verification_taxonomy.py`'s `contract_*` patterns all require trailing digits, so the new `contract_<word>` rename wave passes silently. If the cleanup's intent is to reject all synthetic delivery labels, the guard needs a rule like `\bcontract_[a-z][a-z0-9_]*\b` with an explicit allowlist of real contract surfaces (e.g. `diagnostic_contract_*`, `contract_backlog`, `format_expectation_contract_errors`, validation-manifest `contract_id` field, etc.). Self-test should add a `contract_safe_indexing`/`contract_ergonomics`-style fixture.

### Non-blocking observations

- `scripts/distribution/generate_dispatchers.sh:132` — `fail "stable channel installs are disabled until the stable release channel is enabled"` is tautological; original was `until Phase 39`. Suggest `until a stable channel is supported` or similar.
- `scripts/distribution/create_new_version.sh:138` — error message is grammatically truncated: `stable-looking versions are disabled until the stable release channel: ${VERSION}`. Should likely read `…until a stable channel is supported: ${VERSION}`.
- `scripts/distribution/create_new_version.sh:16` — `Plan or execute a preview release.` gained a two-space indent vs. surrounding usage lines.
- `verification/areas/core_language/checks/lowering_layer_inventory.py:1` docstring `Validate contract-set lowering-layer snapshot inventory rows.` — "contract-set" reads like another delivery-label euphemism; the `contract_id` fields it validates are real semantic contracts, so the docstring could just say `Validate lowering-layer snapshot inventory rows.`

### What is clean

- `git mv` rename of `.cursor/skills/phase-closure-loop/` → `.cursor/skills/codebase-closure-loop/` is detected as `R` by `git status`; the new file is taxonomy-clean and contains no leftover `phase|milestone|wave` tokens.
- `.cursor/commands/create-new-version.md`, `.github/workflows/preview-release.yml`, `verification/areas/project_workspace/data/validation_contracts/manifest.json` assertion strings line up with the updated demo prints (`mode_consistency`, `graph_isolation`, `single-file-graph_isolation_single_file`). No fixture/manifest drift.
- `crates/sifr/tests/e2e/pass/*.sifr` fixture renames (`bytes-wave2` → `bytes-constructor-sample`, etc.) are internal-only string constants — no companion `.stdout` or snapshot asserts on these strings.
- `verification_taxonomy.py` self-test exercises the digit-bearing rejection cases correctly and passes.
- Renamed `internal_docs/typescript_go_architecture_transfer_m*.md` files: all 16 references from `internal_docs/architecture.md:300–317` and the `…_guardrails.py` checker point to the new digit-less filenames; no broken refs outside `plans/`.

### Recommendation

Another review round is needed. The dominant problem is bulk text substitution that traded one disguise (`M1`, `milestone_x`, `Phase 27`) for another (`architecture guardrails`, `contract_x`, `task ownership surface0`). The guard didn't notice because its rules target the *old* shape. Two things have to happen before this should land:

1. Strip the synthetic `contract_<word>` / `task ownership surface[N]?` / `architecture guardrails` substitutes from `internal_docs/architecture.md`, `internal_docs/typescript_go_architecture_transfer_guardrails.md`, the 6 demo files, and the ~30 traceability reports — replace with real descriptive nouns or just drop the label.
2. Extend `verification_taxonomy.py` to forbid digit-less `contract_<word>` outside an explicit allowlist (real API/schema/diagnostic contracts), and add `task_ownership_surface\d*` to the synthetic-label pattern list. Add a self-test case for each.
