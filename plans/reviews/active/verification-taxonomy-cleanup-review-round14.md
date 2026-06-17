# Review: terminology hygiene cleanup

## 1. BLOCKING findings

### A. Mass-substitution garbles in active source / fixtures
Several term swaps were applied without checking the surrounding sentence. The result is broken comments, demo headers, and even live TODO markers:

- `crates/sifr_driver/src/stdlib/bootstrap.rs:33` and `:50` — `TODO(diag_4a behavior group 2):` (was `diag_4a slice 2` — "slice" → "behavior group" inside a parenthesized ID does not belong here).
- `demos/control_flow/main.sifr:4` — `# Sifr synchronization primitives Demo: Control Flow and Data Structures` (literal `M2` → `synchronization primitives` inside a header).
- `demos/codegen_output/main.sifr:4` — `#  Codegen Polish` (lost `Phase 2`; doubled space).
- `crates/sifr/tests/e2e/pass/cpython_difflib_subset.sifr:2` — `the simplified SequenceMatcher(a, b) surface in this surface` (`wave` → `surface` collided with the existing word "surface").
- `verification/areas/stdlib_parity/reports/stdlib_parity_cpython_b1_traceability.md:22` — same "surface in this surface" garble.
- `verification/areas/stdlib_parity/reports/concurrency_runtime_readiness_traceability.md:1` — title double-words: `# Concurrency Runtime readiness Readiness Traceability`. Line 5: `This readiness artifact is the readiness audit surface…`.
- `verification/areas/stdlib_parity/reports/concurrency_runtime_inventory_closure.md:4` — `the final readiness readiness was completed by PR #2488`. Line 65 same defect.
- `internal_docs/architecture.md:202` — `IndexMap may be considered in a future future capability`.
- `internal_docs/architecture.md:277-278` — `design decisions that span multiple future capability. They must be resolved early to prevent future capability from diverging` (mass-noun substitution broke grammar).
- `crates/sifr/tests/e2e/pass/numeric_sentinels.sifr:2`, `reverse_range_narrowing.sifr:2` — `Narrowing regression behavior group N` (awkward but parses).

These are clear defects from regex sweeps that should be hand-fixed before the cleanup lands.

### B. Demo payload renames invalidated their own hard-coded byte-integer assertions

Three demos rename the `b"…"` payload but still assert against the OLD payload's `to_ints()` representation. These will fail at runtime:

- `demos/bytes_file_io/main.sifr:22` — payload now `b"bytes_file_io"` but `ints_ok == "[119, 97, 118, 101, 51]"` (= `wave3`). Mirror: `emitted.rs:381`, `idiomatic.rs:26`.
- `demos/binary_storage/main.sifr:22,25-27,48` — payload now `b"binary_storage"` (14 bytes) but expects `expected_second = 97`, `sum_bytes == 487`, `contains(119)` ("w"), and `"[119, 97, 118, 101, 52]"` (= `wave4`). Mirrors: `emitted.rs:481`, `idiomatic.rs:72`.
- `demos/binary_files/main.sifr:23` — payload now `b"runtime-binary_files"` but expects `"[114, 117, 110, 116, 105, 109, 101, 45, 119, 97, 118, 101, 48]"` (= `runtime-wave0`). Mirror: `emitted.rs:381`, `idiomatic.rs:30`.

These three demos need both their payload and their derived byte/sum assertions regenerated as a unit.

### C. AGENTS.md still leaks `Milestone` and `Roadmap`
- `AGENTS.md:81` — `- demos/ — Milestone demo files (*.sifr) showcasing language features`
- `AGENTS.md:84` — `- plans/ — Roadmap, phase plans, issue plans, and review artifacts`

These slipped past the gate because the taxonomy regex in `verification/areas/coverage_matrix/checks/verification_taxonomy.py` only bans `Milestone\s+\d+` (digit-anchored) and never lists bare `Roadmap`. Tighten the regex OR rewrite these lines.

### D. Dangling rename target
- `plans/phases/42_data_science_ml.md:29` links to `verification/areas/core_language/data/integer_dtype_contract.md`. The file is now `integer_dtype_rules.md`. (The user's instruction was "outside ./plans/" but this is a plans-file link OUT to verification/, and the verification target moved — fix the link.)

### E. Process-taxonomy leak baked into a verification harness constant
- `verification/areas/generated_code_quality/generated_code_quality.py:79` and `:346` — Python constant `CONCURRENCY_CLOSEOUT_DEMOS`. Closeout-as-identifier in the harness itself.

### F. Lingering "Conversion Set" identifiers in active reports
The user listed "conversion set" as banned. The taxonomy regex only matches the literal `conversion set N demo` suffix, so the bare term survives in:
- `internal_docs/generated_code_quality.md:82, 108, 120` and several `verification/areas/stdlib_parity/reports/*traceability.md` files — "Conversion Set N" / "Post-Closure Audit Conversion Set N".

## 2. NON-BLOCKING concerns

- **Awkward "backlog" → "signal queue" rewrite** in `verification/policy/ecosystem_compatibility.md:51`, `suite_taxonomy.md:17`, `fuzz_property.md:80` reads as nonsense in context (e.g. "signal queue generation"). "Backlog" in these contexts was the planning sense — the replacement just needs better wording, not a regex.
- **`docs/concurrency_runtime.md:117, 225`** — "The current runtime readiness does not ship" (was "M7 does not ship"). Drops the sequencing fact; reads oddly.
- **`docs/cli/build-run.mdx:81`** uses "phase-aware summary" while sibling `docs/cli_command_semantics.md:66` reads "stage-aware". User-facing inconsistency. (Note: `docs/cli/build-run.mdx:11,33,77` legitimately keep "build phase" — that's correct compiler-pipeline language.)
- **`docs/stdlib/text-encoding.mdx:340`** still references "future phases" in user-facing copy.
- **`internal_docs/architecture.md`** lists in the "Feature Area" column (formerly "Milestone") now read like bare anchor IDs (`safe_indexing`, `collection_safety`, `enum type-system work`, `pattern-matching work`). The table loses ordering information; if M0/M3/M7 sequencing was load-bearing, that information isn't recoverable from the diff.
- **Borderline "ad hoc" prose:** `internal_docs/tooling_analysis.md:149, 159` (`ad hoc production-grade linter work`, `the ad hoc phase keeps…`); demos `recursive_type_part*`/`nested_function_part*`/`self_update_demo` `main.sifr` header (`Ad hoc <feature> part N demo`). These read as templatic stamps rather than English prose.
- **Lingering planning-sense "follow-up":** `internal_docs/typescript_go_architecture_transfer_first_class_flow_graph.md:81-92` ("Loop-else follow-up validation"), `crates/sifr/tests/e2e/pass/{bounded_channel_basic,channel_basic}.sifr` ("follow-up channel slices"), `docs/network_http.md:45`, `internal_docs/network_http_architecture.md:34-39`, `verification/areas/stdlib_parity/data/network_http_substrate_inventory.json:93-124` ("`network-http-serving-scale-follow-up`" identifier in docs).
- **Architecture-lock files lost concrete `plans/issues/*.md` link targets** in favor of bare strings like `first-class-bytes-and-binary-surface-foundation record`, `network-http-serving-scale-follow-up record` (`stdlib_bytes_architecture_lock.md:3-4`, `stdlib_rng_architecture_lock.md:3`, `stdlib_iterator_architecture_lock.md:3`, `generic_clone_hardening_traceability.md:4`, `network_http_baseline_traceability.md:15,25`). Cross-refs no longer navigable.
- **Enforcer hardening to match the user's stated ban list** would close most of these gaps in one shot: add `\bRoadmap\b`, bare `\bMilestone(s?)\b`, `\bConversion Set\s+\d+\b`, bare `\bcloseout\b`/`\bCLOSEOUT\b` (currently only sub-patterns), and possibly `\bfuture future\b` / `\breadiness readiness\b` as anti-double-swap canaries.

## 3. Am I satisfied the pass covers the goal without over-banning?

**Almost — but not yet.** The taxonomy file itself is well-shaped: legitimate compiler/runtime terms (`compiler phase`, `phase=hir`, `closure`, `TCP backlog`, `TLS session ticket`, `contract_id`, `contract_check`, `contract_version`, `build output contract`, `schema contract`, `WorkspaceTracePhase`, `SingleOwnerCompilerPhase`, etc.) are all preserved by `ALLOW_TEXT_PATTERNS` and survive a manual cross-check against active sources. The four profile JSONs and the rename graph for `coverage_matrix:readiness`, `validation_suite_support`, `validation_suites.rs`, `compact_grouping_behavior`, and `.cursor/skills/codebase-readiness-loop` are end-to-end coherent — no over-banning was observed and no rename leaves a stale Rust/TOML/script reference (matches outside ./plans came back empty for `validation_contract`, `phase-closure-loop`, `coverage_matrix_closeout`, `compact_grouping_contract`).

What blocks "yes" is the **execution defects**, not the design: AGENTS.md isn't taxonomy-clean, three demos will execute with broken assertions, `bootstrap.rs` has two broken TODO comments, multiple reports have double-word swaps the eye would have caught, `architecture.md` has grammar regressions from a mass-noun substitution, and the enforcer's bare-word regex set has gaps that let the AGENTS.md and "Conversion Set" leaks through. None of these are scope mistakes — they're sweep collisions. Fix the items in §1 and tighten the regex set (§A/C/F), and the pass meets the goal.
