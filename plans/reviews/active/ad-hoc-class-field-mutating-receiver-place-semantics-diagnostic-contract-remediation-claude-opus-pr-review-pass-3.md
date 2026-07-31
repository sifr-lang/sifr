I completed a read-only review at exact HEAD `5ac500f5175bc07407e498b1f016f819e40ed911` against `origin/main` `bf5d82a7a`, read the full diff and both prior review artifacts, and independently re-ran the validation surface. No files were modified (the only working-tree entry remains the pre-existing zero-byte pass-3 placeholder).

---

# Focused Diagnostic-Contract Remediation Review — Pass 3

**Diff scope vs `origin/main`:** 15 files, +796/−102 across 4 commits (`87e35414b`, `12e6ebbb2`, `fe5fbab48`, merge `6f735ffc6`, `5ac500f51`).

## Prior findings — all closed

**Pass-1 BLOCKING 1 (PROTO-0006 identity + randomized order)** — closed, re-verified independently.
- `class_name` participates in the full contract: template, `declared_args`, `dedupe_args` (`registry_entries/calls_flow_and_protocols.rs:426-435`), `internal_docs/diagnostic_codes.md:222`, plan `:669-676`. `docs/errors/SIFR-PROTO-0006.mdx` embeds no template or arg list (grep: zero hits for `cannot mutate`/`class_name`/`trait_name`), so no regeneration is owed — confirmed by `gen-error-docs --check` exiting clean.
- The `HashSet` iteration is gone: `validate_fixed_receiver_mutations` collects `FixedReceiverMutation` and sorts on `(range.start(), class_name, method)` (`method_receiver_analysis.rs:227-247`) — total ordering even when `method_source_ranges` misses (`unwrap_or_default()` ties break on class/method).
- Recovery-cap survival is genuinely pinned: `test_fixed_receiver_diagnostics_survive_similar_recovery_cap` runs six same-dunder classes through the real `sifr_driver::apply_diagnostic_recovery_limits` and asserts exactly six `class_name` values (an omission summary would shrink the vector). Passes.

**Pass-2 BLOCKING 1 (stale compact baseline)** — closed, byte-verified two ways.
- I reproduced the fixture directly: `cargo run -q -p sifr -- --diagnostic-format compact check verification/areas/diagnostics/fixtures/diagnostics/e2e_operator_receiver_mutation_rejected/main.sifr` → stderr `diff` against `baselines/check-compact.stderr.txt` is empty, exit code `1` matches `check-compact.exit-code.txt`.
- Authoritative suite: `uv run --project verification --locked python -m sifr_verify areas run --area diagnostics --suite baselines` → `cases=150`, `variants=178`, `total_failures=0`, `blocking_failures=0` (`target/verification/areas/diagnostics-results.json`). Both `e2e_operator_receiver_mutation_rejected/check-compact` and `e2e_protocol_receiver_mutability_mismatch/check-compact` report `status=pass`. This exactly matches the ledger's recorded 150/178/0.
- Blast radius re-checked at this head: whole-repo grep for `cannot mutate its receiver` and `requires a mutable receiver but protocol` hits only the registry, the emitter helper, the plan, `internal_docs/diagnostic_codes.md`, and the two compact baselines. `verification/areas/diagnostics/data/{code_catalog,code_baseline_coverage}.json` record no templates or arg lists. The PROTO-0005 message was already class-qualified on `origin/main` (registry `:418` unchanged in the diff), so its baseline correctly needed no edit.

**Pass-2 NON-BLOCKING 2 (alphabetical order cannot pin the range key)** — closed.
- `fixed_receiver_diagnostics_are_declaration_ordered_and_class_distinct` now declares `Zulu` first and asserts `["Zulu", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"]` (`method_receiver_diagnostics_tests.rs:136,193`). A `class_name`-only sort would yield `Bravo…Zulu` and fail, so the `range.start()` primary key is genuinely pinned.
- The e2e test correctly asserts the *other* order — `["Bravo","Charlie","Delta","Echo","Foxtrot","Zulu"]` (`e2e_entrypoints.rs:466`) — which is the `BTreeMap<SimilarDiagnosticGroupKey,_>` group order that `apply_diagnostic_recovery_limits` (`sifr_driver/src/diagnostics.rs:167-179`) produces. The two tests together now distinguish HIR emission order from rendered group order, and the plan text (`:672-679`) states exactly that distinction rather than the earlier over-broad "emitted in source order" claim. I confirmed every user-facing render funnels through `canonical_diagnostic_stream` → `apply_diagnostic_recovery_limits` (`crates/sifr/src/diagnostic_rendering_and_run.rs:24-27,49-55`), so the rendered claim is accurate.

**Pass-1 NON-BLOCKING 2 (OWN-0005 phase-owned site + non-binding `binding`)** — closed.
- `immutable_parameter_field_receivers_report_root_binding_argument` (`method_receiver_diagnostics_tests.rs:198-228`) covers both the borrowed and `own` parameter field-receiver shapes reaching `report_immutable_root`, asserts exactly 2 diagnostics, and asserts `binding == "owner"` on both.
- The type/place leak is gone. `report_immutable_root` (`method_receiver_places.rs:587-600`) now resolves the retained binding name, falls back to the *syntactic* root identifier via the new `root_binding_name` (`:601-610`), and only if neither exists routes to `unsupported_mutable_receiver_place` — which is `OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE` (SIFR-OWN-0014) populating `place`, matching that code's declaration (`registry:214-224`). `place_display` no longer reaches the `binding` arg. Given `root_binding_id` (`:633-641`) and `root_binding_name` traverse the identical Name/FieldAccess/Index/Slice shapes, the OWN-0014 branch is defensive-only but contract-correct.

**Pass-1 NON-BLOCKING 3 (guardrail scope + hardening)** — closed. `e2e_entrypoints.rs:347-349` carries the three-line scope comment (120 gaps, 11 families, diagnostics owner); the plan records the same with named ownership (`:686-699`). The guardrail now asserts `state == DiagnosticState::Active` (`:361-366`), iterates **every** diagnostic of each code rather than `.find` (`:378-388`), asserts each `declared_args` key is present (`:389-396`), and rejects empty/whitespace `String` values (`:397-404`). All five codes declare only `arg!` string args, so nothing escapes the emptiness check.

**Pass-1 NON-BLOCKING 6 / decomposition** — closed. Both PROTO emitters now live in `method_receiver_diagnostics.rs` (66 lines), and both helper messages match their registry templates byte-for-byte (I diffed all four strings). `method_receiver_analysis.rs` threads named `FixedReceiverMutation`/`ProtocolReceiverMismatch` records instead of tuples and pre-formatted strings. The two relocated tests were *moved*, not dropped, and strengthened from code-presence to exact-arg assertions (`method_receiver_analysis_tests.rs:-596..-651` → `method_receiver_diagnostics_tests.rs:5-131`) — no coverage lost.

**Pass-1 NON-BLOCKING 4/5 (plan record + named test runs)** — closed. Ledger entries for passes 1 and 2 and the #3095 prerequisite are present (`:1157-1197`); the focused validation block (`:1201-1214`) names both `crates/sifr` tests and the diagnostics baselines suite explicitly. `cargo clippy --workspace -- -D warnings` is recorded in the phase validation block (`:827`).

## Prerequisite isolation

PR #3095 is correctly isolated. `git diff bf5d82a7a 5ac500f51 -- verification/` touches **one** file — the PROTO-0006 compact baseline. The bare-`defaultdict` correction (`3 errors` → `1 error`, dropping the cascaded `SIFR-STDLIB-0001`/`SIFR-TYPE-0002`) is already at `bf5d82a7a:.../e2e_bare_defaultdict_constructor_rejected/baselines/check-compact.stderr.txt`, i.e. merged in main, and appears in this branch only as merge-inherited content via `6f735ffc6`. Both fixtures pass in the integrated baselines run.

## Independent validation on this exact head

| Check | Result |
|---|---|
| `cargo test -p sifr_lowering method_receiver_diagnostics_tests` | 5 passed |
| `cargo test -p sifr_lowering` (full) | **957 passed, 0 failed, 1 ignored** |
| `cargo test -p sifr --test e2e test_receiver_place_representative_diagnostics_populate_declared_args` | ok |
| `cargo test -p sifr --test e2e test_fixed_receiver_diagnostics_survive_similar_recovery_cap` | ok |
| `cargo test -p sifr --test e2e test_e2e_fail` | ok (7.30s) |
| `cargo test -p sifr_diagnostics registry_skeleton_is_internally_consistent` | 1 passed |
| `gen-error-docs -- --check` | clean |
| diagnostics `baselines` suite | cases=150, variants=178, failures=0 |
| operator fixture compact stderr/exit-code | byte-exact vs baseline |
| `check_docs_error_code_links.py` / `check_hir_maintainability_guardrails.py` / `check_file_size_guardrails.py` | PASS (3079 files, limit 900) |
| `cargo fmt --check` | clean |
| `cargo clippy --workspace -- -D warnings` | clean (exit 0) |

Largest touched files: `e2e_entrypoints.rs` 829, `method_receiver_places.rs` 703, `method_receiver_analysis_tests.rs` 796, `method_receiver_analysis.rs` 546, `ownership_diagnostics.rs` 486, `method_receiver_diagnostics_tests.rs` 228, `method_receiver_diagnostics.rs` 66 — all under 900.

## Two observations, neither actionable

I record these for completeness; neither is a defect at this head nor gates publishing.

- **`cargo clippy -p sifr_lowering --all-targets -- -D warnings` reports 29 errors**, all pre-existing test-only lints in unrelated files (`compiler_intrinsics_tests.rs`, `expressions_tests/support.rs`, `python_buffer_contract_tests.rs`, `own_mut_semantics_tests.rs:21,24` — the untouched `range_for_after` helper). Zero hits in `method_receiver_*` or `ownership_diagnostics.rs`. `--all-targets` is not the repository's documented gate (`AGENTS.md`, plan `:827`), which passes. Likewise `sifr_ipc/tests/ipc_process_pipe_fixture.rs:37` under `--all-targets`. Nothing here belongs to this branch.
- **Two latent, currently-unreachable symmetries.** `fixed_trait_name` (`method_receiver_analysis.rs:260-271`) duplicates the dunder list in `fixed_trait_receiver_convention` (`classes/parameter_conventions.rs:27-35`); I verified the two are exhaustively aligned today, so its `_ => "Rust trait"` arm is dead and `trait_name` is always an exact trait. Separately, `class.implements_protocols` is built from a `HashMap` iteration (`refresh_protocol_implementations:114-136`, unchanged from main), so a class implementing two protocols that both declare the same shared-receiver method would emit its two PROTO-0005 diagnostics in randomized *HIR* order — with no user-visible effect, since `protocol` is in `dedupe_args` and every render path regroups through the `BTreeMap` in `apply_diagnostic_recovery_limits`, and no test asserts that order.

## Summary

All pass-1 and pass-2 findings are closed and independently re-verified. PROTO-0005, PROTO-0006, and OWN-0005 populate every registry-declared argument with exact meaningful values on the real rendered-envelope path; PROTO-0006's `class_name` participates in template, `declared_args`, and `dedupe_args` end-to-end, HIR emission is deterministically source-ordered through a range-primary sort that a non-alphabetical test now actually pins, and six same-dunder violations provably survive the five-per-group recovery cap. The phase-owned immutable parameter field-receiver path reports the root binding, with the impossible missing-root branch narrowed to the correct SIFR-OWN-0014 `place` contract. The five-code guardrail checks Active state, all occurrences, key presence, and non-empty values, with its deliberate scope recorded in both code and plan. The widened compact baseline is byte-exact and the integrated diagnostics baselines suite passes 150 cases / 178 variants with zero failures; the bare-`defaultdict` prerequisite is correctly isolated to `origin/main` via #3095. Decomposition is responsibility-based with no coverage loss, every touched file is under the cap, and the plan ledger and validation commands are accurate and sufficient — I executed every command in the focused block plus the full lowering suite, the annotated fail suite, and workspace clippy, and all pass.

Zero actionable findings. This candidate is ready to publish as a PR.

SATISFIED
