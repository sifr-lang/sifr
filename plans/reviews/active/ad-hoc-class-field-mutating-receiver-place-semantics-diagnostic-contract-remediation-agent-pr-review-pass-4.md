## Findings

### 1 — LOW (actionable, record accuracy): the scope figures baked into shipped code and the plan are not reproducible at this head

`crates/sifr/tests/e2e_support/e2e_entrypoints.rs:347-349`

```rust
// Deliberately receiver/place-scoped: generalizing this to every active source fixture
// exposes 120 pre-existing argument gaps across 11 unrelated families. The
// diagnostics owner must complete that separate migration before broadening it.
```

Same claim in `plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md:697-698` ("exposed 120 pre-existing missing-argument failures across 11 unrelated diagnostic families") and restated at `:1170`.

I ran exactly that generalization at this head — every `Active` registry entry with a `.sifr` representative fixture (101 codes with declared args), compiling each fixture and applying the guardrail's own criteria (declared arg present on every emitted diagnostic of that code; non-empty for string args):

- **110** missing declared-arg occurrences (0 empty-string values), across **65** codes and **12** families: `CALL, CLASS, DECIMAL, FLOW, IMPORT, INT, MATCH, NAME, OWN, PROTO, RESULT, TYPE`.
- 0 fixtures failed to emit their own code, so nothing is hidden behind an "absent diagnostic" bucket.
- None of the five phase codes appear in the gap set — independent confirmation that the remediation itself is complete.

The one methodological deviation I can identify (I drove the CLI's rendered stream, which passes through `apply_diagnostic_recovery_limits`) can only *reduce* my occurrence count, so it does not explain the family count going the other way: I find **12** gapped families where the record asserts **11 unrelated** ones, with no stated exclusion rule that yields 11 (excluding both receiver/place families `OWN` and `PROTO` would give 10). The substance of the rationale — a large pre-existing migration exists and belongs to the diagnostics owner — is true and well-founded; the specific numbers are not.

Fix is cheap: recompute the figures at this head, or state them as approximate and name the methodology/head they were measured on. Because the numbers are shipped in source (not just a plan note) and are the stated justification for the guardrail's scope, I treat this as actionable rather than cosmetic.

## Non-actionable observations

- `class.implements_protocols` is built by iterating `ctx.class_types: HashMap` (`method_receiver_analysis.rs:114-135`, unchanged from `origin/main`), so a class implementing two protocols that both declare the same shared-receiver method would emit its PROTO-0005 diagnostics in randomized *HIR* order. No user-visible effect: `protocol` is in `dedupe_args`, every render path regroups through the `BTreeMap` in `apply_diagnostic_recovery_limits`, and the new test sorts before asserting (`method_receiver_diagnostics_tests.rs:112`). Pre-existing, correctly out of scope.
- `e2e_entrypoints.rs` is now 829/900 lines — passes, but has limited headroom for the next test added there.
- The plan does not yet link PR #3096; it says only "The focused diagnostic-contract remediation is in review." That is chronologically unavoidable — the last commit `e1b1ab82b` predates publication — and the merge-time record will carry the link.

## What I verified independently (all clean)

- **Head/scope**: published `headRefOid` `e1b1ab82b` == local HEAD; merge base `bf5d82a7a` == `origin/main`. 16 files, +887/−102, 7 commits, no unrelated changes. `e1b1ab82b` is plan-only; `efcbc3d5e` is the one-line comment correction only.
- **Structured args, end to end**: PROTO-0006 gains `class_name` in template, `declared_args`, and `dedupe_args` (`calls_flow_and_protocols.rs:426-435`), mirrored in `internal_docs/diagnostic_codes.md:222` and the plan (`:670-676`); PROTO-0005 and OWN-0005 populate every declared arg. Emitters live in the new `method_receiver_diagnostics.rs` (66 lines) and `ownership_diagnostics.rs:214-228`; their format strings match the registry templates byte-for-byte.
- **Deterministic HIR source order**: `validate_fixed_receiver_mutations` replaces `HashSet` iteration with a `(range.start(), class_name, method)` total sort (`method_receiver_analysis.rs:227-247`); the unit test declares `Zulu` first and asserts `["Zulu","Bravo",…]`, which an alphabetical sort could not satisfy.
- **Recovery/class identity**: `test_fixed_receiver_diagnostics_survive_similar_recovery_cap` drives real `apply_diagnostic_recovery_limits` and asserts six distinct `class_name` values in `BTreeMap` group order — correctly the *other* order from the HIR test, matching `sifr_driver/src/diagnostics.rs:166-186`.
- **Root binding / fallback**: `root_binding_name` (`method_receiver_places.rs:603-611`) traverses exactly the same Name/FieldAccess/Index/Slice shapes as `root_binding_id` (`:633-641`); `InvalidPlace::ImmutableParameter` is only returned when `retained_binding` is `Some` (`:354-396`), so the OWN-0014 branch is defensive-only and contract-correct (`place` arg, matching `registry:214-224`). `place_display` no longer reaches the `binding` arg.
- **Guardrail taxonomy is accurate**: the five constants resolve to `SIFR-OWN-0002/0005/0014` and `SIFR-PROTO-0005/0006` (`registry.rs:112,115,126,163,165`) — exactly the set the plan names at `:692-696`, and all five are receiver/place codes. The pass-3 → pass-4 correction from "phase-scoped" to "receiver/place-scoped" is the accurate descriptor.
- **Rendered envelope**: byte-exact — CLI compact stderr/stdout/exit-code for `e2e_operator_receiver_mutation_rejected` diff clean against the updated baseline. `gen-error-docs --check` clean, so no `.mdx` regeneration is owed.
- **Tests run here**: `method_receiver_diagnostics_tests` 5 passed; both new `crates/sifr` e2e tests pass; `test_e2e_fail` passes (7.29s); `registry_skeleton_is_internally_consistent` passes; `cargo fmt --check`, `check_docs_error_code_links.py`, `check_file_size_guardrails.py` (3079 files, limit 900), `check_hir_maintainability_guardrails.py` all pass. The two relocated unit tests were moved and strengthened, not dropped.
- **Prerequisite isolation**: #3095's bare-`defaultdict` correction is already on `origin/main` and reaches this branch only through merge `6f735ffc6`; the branch's only `verification/` change is the PROTO-0006 baseline.
- **PR metadata**: title/body match the diff; "957 passed / 1 ignored" is consistent with the 958 tests I observed in `sifr_lowering`; pass-1/2/3 review artifacts are present and their verdicts match the plan ledger.

## Verdict

**NOT SATISFIED** — one low-severity actionable finding (unreproducible 120-gaps/11-families scope figures asserted in shipped source at `e2e_entrypoints.rs:348` and in the plan at `:697-698, :1170`; measured 110 gaps across 12 families at this head). Everything else in the implementation, tests, diagnostics contract, docs/tracking, validation claims, commit scope, and PR metadata verified clean.
