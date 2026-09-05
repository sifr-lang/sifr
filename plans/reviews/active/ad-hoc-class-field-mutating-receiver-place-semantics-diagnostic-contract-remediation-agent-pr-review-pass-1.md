I inspected the implementation, tests, registry, driver dedup/grouping path, fixtures, and the originating pass-5 review independently. No files were modified and no builds/tests were run.

---

# Focused Diagnostic-Contract Remediation Review — Class-Field Mutating-Receiver Place Semantics

**Exact head reviewed:** `87e35414b22e8500ba4199b59f34ef0ca5ba1a68` (sole commit in `origin/main..HEAD`, branch `codex/class-field-diagnostic-args-remediation`)
**Diff scope:** 7 files, +215/−58. Read-only audit.

## What the remediation gets right

- **`SIFR-PROTO-0005` args** (`method_receiver_analysis.rs:305-317`): `class_name`/`method`/`protocol` are populated via `error_with_code_args_help_at`, matching the registry declaration (`registry_entries/calls_flow_and_protocols.rs:420-421`), the template at `:418`, `internal_docs/diagnostic_codes.md`, and the phase plan's reservation (archive lines 690-700). Values are exact-asserted in `method_receiver_diagnostics_tests.rs:21-42`.
- **`SIFR-PROTO-0006` args** (`method_receiver_analysis.rs:223-241`): `method`/`trait_name` populated; exact values asserted at `method_receiver_diagnostics_tests.rs:114-126`, including that `__eq__` maps to `PartialEq` through `fixed_trait_name` (`:245-256`).
- **`SIFR-OWN-0005` `binding`** (`ownership_diagnostics.rs:212-229`): populated at the single funnel. I verified all four emit sites route through it — `binding_mutability.rs:14`, `mutating_methods.rs:71`, `:118`, `method_receiver_places.rs:591` — so there is no residual `error_with_code_at` path for this code.
- **Arg propagation is real end-to-end**, not just at the HIR layer: `query_diagnostics.rs:150-167` forwards `structured_args` as `extra_args` into `diagnostic_with_source_range_args_help` (`query_diagnostic_rendering.rs:45-62`), which merges them into the rendered envelope. The new e2e assertion at `e2e_entrypoints.rs:369-381` therefore exercises the same `args` map that JSON consumers and `recovery_dedupe_args` (`sifr_driver/src/diagnostics.rs:268-283`) read.
- **Distinct identity for multiple PROTO-0005**: `validate_protocol_receiver_conventions` collects into a declaration-ordered `Vec` (`:259`, `:291-300`) and now emits `class_name`, so two mismatches in one file get distinct `SimilarDiagnosticGroupKey` values (`sifr_driver/src/diagnostics.rs:209-215`) instead of collapsing into one capped group. `method_receiver_diagnostics_tests.rs:46-95` pins this and is order-insensitive (sorts `class_names`).
- **Test-module split is justified, not cosmetic**: `method_receiver_analysis_tests.rs` was 846 lines; adding the ~127-line diagnostics suite inline would have landed near 923 lines, over the 900 cap. Post-split: 796 and 127. All touched files are under the cap (`e2e_entrypoints.rs` 744, `method_receiver_analysis.rs` 539, `ownership_diagnostics.rs` 486, `own_mut_semantics_tests.rs` 202, `mod.rs` 254). Registration at `lower/mod.rs:91-92` is `#[cfg(test)]`-gated. `check_hir_maintainability_guardrails.py` only enforces banned monoliths and checklist snippets, so it is unaffected.
- **Guardrail scoping judgment: I agree with the phase's decision.** Enforcing the assertion across `active_registry_entries()` would convert a five-diagnostic contract fix into a ~120-failure, eleven-family diagnostics-program migration touching NAME/IMPORT/TYPE/DECIMAL/CALL/OWN/FLOW/MATCH/PROTO/CLASS/RESULT emitters. That is unambiguously a separate program, and the scoped guardrail does close pass-5 finding 4 *for the diagnostics this phase owns*: it is a live emission check (compile the `representative_fixture_path`, assert every `declared_args` entry is present), which is exactly the drift class that let findings 2 and 3 survive sixteen prior reviews. The placement in `e2e_entrypoints.rs` is also correct — the check needs `sifr_driver`, which `sifr_diagnostics` cannot depend on — and there is precedent in the same file (`test_decimal_fail_fixtures_do_not_emit_retired_pseudo_codes`, `:386`).

Pass-5 code findings 2, 3, and 4 are therefore substantively addressed. Findings 1 (merge gate) and 5 (PR body) are not code findings and are outside this branch.

---

## BLOCKING 1 — Two `SIFR-PROTO-0006` diagnostics in one module have byte-identical structured identity and are emitted in per-run randomized order, so the similar-group cap can drop a *nondeterministically chosen* diagnostic

`validate_fixed_receiver_mutations` iterates a `HashSet`:

- `crates/sifr_lowering/src/lower/method_receiver_analysis.rs:209` — `fixed_receivers: &HashSet<MethodKey>`
- `:213` — `for key in fixed_receivers { … }` emits directly inside the loop (`:232-241`)

`HashSet` with the default `RandomState` randomizes iteration order per process. Nothing downstream restores order: I grepped `crates/sifr_lowering/src/lower/mod.rs`, `sifr_lowering/src/lib.rs`, `sifr_driver/src/frontend/*.rs`, and `sifr_driver/src/diagnostics.rs` — there is no sort of diagnostics by span, and `apply_diagnostic_recovery_limits` (`sifr_driver/src/diagnostics.rs:162-188`) preserves input order *within* a group (only the group keys are `BTreeMap`-ordered).

The identity problem compounds it. Per the registry (`registry_entries/calls_flow_and_protocols.rs:431-432`), `SIFR-PROTO-0006` declares and dedupes on `method`/`trait_name` only — no class discriminator. Because HIR diagnostics always carry `message_template == "{message}"` (`query_diagnostics.rs:155`), the `SimilarDiagnosticGroupKey` (`sifr_driver/src/diagnostics.rs:209-215`) for two mutating `__eq__` methods in different classes is identical in *every* field except `primary_file`, which is also identical. Even the human message is byte-identical, since the message omits the class.

This is not hypothetical for this phase — its own representative fixture already contains exactly this shape:

```
crates/sifr/tests/e2e/fail/operator_receiver_mutation_rejected.sifr
  Counter.__eq__            → SIFR-PROTO-0006  (method=__eq__, trait_name=PartialEq)
  DelegatingCounter.__eq__  → SIFR-PROTO-0006  (method=__eq__, trait_name=PartialEq)
```

**Failure scenario A (diagnostic loss, nondeterministic).** A module with six classes each defining a receiver-mutating `__eq__`. All six diagnostics land in one group; `apply_diagnostic_recovery_limits` (`:174-186`) retains `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` and replaces the rest with an omission summary. `apply_diagnostic_recovery_limits` is on the real user path (`crates/sifr/src/diagnostic_rendering_and_run.rs:27`, `sifr_driver/src/project/frontend.rs:272`), so the user loses one real error — and *which* class is dropped changes between runs, so a fix-one-error-at-a-time loop can see errors appear and disappear. Six PROTO-0005 mismatches in one file, by contrast, now group by `class_name` and are all retained; the asymmetry is caused precisely by the missing discriminator.

**Failure scenario B (order flake).** Any future fail fixture with two PROTO-0006 diagnostics at *different* columns will flake: `crates/sifr/tests/e2e.rs` documents "Expectation annotations preserve declaration order", and the two emitted diagnostics arrive in randomized relative order. The existing fixture escapes this only by accident — both its annotations are `# expect-error[col=9]: SIFR-PROTO-0006`, so swapping them is unobservable.

Two-line fix for the determinism half: collect the loop's `(key, trait_name, range)` triples into a `Vec` and sort by `(range.start(), key.class, key.method)` — or iterate `classes` in declaration order as `validate_protocol_receiver_conventions` already does — before emitting. The commit rewrote this exact function and mirrored the `Vec` pattern for PROTO-0005 while leaving PROTO-0006 on the randomized `HashSet`, so the inconsistency is squarely in the changed surface. (Attribution: the `HashSet` iteration predates this commit, having arrived with the original PROTO-0006 reservation earlier in this phase; it is phase-owned, not a regression introduced here.)

Related non-blocking note under the same root cause: the missing `class_name` on PROTO-0006 is a *contract* limitation — archive lines 703-711 reserve only `method` and `trait_name`, so the emitter faithfully implements what was declared. Closing the identity gap properly means widening the declaration (registry entry `:429-432`, template, `internal_docs/diagnostic_codes.md`, `docs/errors/SIFR-PROTO-0006.mdx`) to include the implementing class, then populating it. I list it here rather than as its own finding because it shares the failure scenario above.

---

## NON-BLOCKING 2 — The phase's own `SIFR-OWN-0005` emit site has no argument assertion, and its fallback can place a non-binding string into `binding`

Pass-5 finding 3 named one specific site as the phase's contribution: `crates/sifr_lowering/src/lower/method_receiver_places.rs:587-592` (`report_immutable_root`), reached by the two phase fail fixtures `immutable_borrowed_parameter_field_receiver.sifr` and `immutable_owned_parameter_field_receiver.sifr`. Nothing in this commit asserts args for that site:

- The e2e guardrail uses `SIFR-OWN-0005`'s single `representative_fixture_path`, `own_parameter_mutation_requires_mut.sifr` — an `own items: list[int]` subscript assignment, which reaches the funnel via `binding_mutability.rs:14`, not `report_immutable_root`.
- The three exact-value assertions added at `own_mut_semantics_tests.rs:157-158`, `:176-177`, `:195-196` all use `Crate`/`Depot` argument-position sources, reaching `mutating_methods.rs:71`/`:118`.
- `own_mut_semantics_tests.rs:66-90` (the plain-parameter mutation tests) were left without arg assertions.

So no test would fail if a future refactor re-routed `report_immutable_root` through `error_with_code_at`, or introduced a place-based variant that skipped `binding` — which is the exact regression class the remediation exists to prevent.

Second, the value itself is not contract-clean. `method_receiver_places.rs:588-590`:

```rust
let name = root_binding_id(expr)
    .and_then(|id| ctx.scope.retained_binding(id))
    .map_or_else(|| place_display(expr, ctx), |fact| fact.name.clone());
```

`place_display` (`:594-599`) returns either `display_checked_place(...)` (a projected place such as `owner.helper`) or `expr.ty().display_name()` (a *type* name such as `Helper`). Whenever the root binding fact is not retained, the `binding` argument — declared as the root binding (`registry_entries/calls_flow_and_protocols.rs:123`, plan text at archive line 674-676: "Populate `binding` with the root binding") — holds a place expression or a type name. **Failure scenario:** an LSP consuming `args.binding` as an identifier to offer "add `mut` to parameter `X`" produces `add mut to parameter Helper`, or a rename/quick-fix keyed on the arg targets a type. Previously this only degraded prose; the remediation promotes it into the machine-readable contract, which is what makes it worth fixing (or explicitly documenting the fallback) now.

Suggested closure: add one exact-value test over the `owner.helper.bump()` shape asserting `binding == "owner"`, and either narrow the fallback or record the degraded-value case.

---

## NON-BLOCKING 3 — The guardrail's deliberate five-code scope is recorded nowhere the next contributor will look

`e2e_entrypoints.rs:347-353` hardcodes five `DiagnosticCode` constants with no comment. The rationale you supplied for this review — that generalizing to `active_registry_entries()` surfaces ~120 pre-existing missing-argument failures across eleven families, making repository-wide migration a separate program — appears in neither the test, the archive, nor `internal_docs/diagnostic_codes.md`.

**Failure scenario:** the next contributor reads a five-element hardcoded array next to a public `sifr_diagnostics::codes::active_registry_entries()` helper (`registry.rs:861-865`), reasonably concludes it was an oversight, generalizes the loop, and hits 120 opaque failures across unrelated families with no recorded explanation of the scope decision or the migration program that owns them. A three-line comment naming the count, the affected families, and the follow-up owner converts this from apparent oversight into a recorded decision. The same note belongs in the archive's diagnostics section.

Minor items in the same test, worth folding into one pass: the `expect` messages claim "active receiver-place diagnostic" but `state == DiagnosticState::Active` (`registry.rs:371`) is never asserted; the assertion checks key presence only, so an empty-string or placeholder value passes; and `.find` at `:369-372` inspects an arbitrary one of the two PROTO-0006 diagnostics that `operator_receiver_mutation_rejected.sifr` emits (arbitrary in the literal sense, per BLOCKING 1).

---

## NON-BLOCKING 4 — No planning/archive record lands with the remediation

`origin/main..HEAD` contains code and tests only; no `plans/` or `internal_docs/` change. The archive still describes this work in the future tense — `plans/issues/archive/ad-hoc-class-field-mutating-receiver-place-semantics.md:57-58` (in the closure worktree): "A focused diagnostic-contract remediation **is in progress**; the pre-existing `SIFR-OWN-0005` helper debt is explicitly adopted by this phase". Pass-5's remediation step 2 specifically asked for the archive to record that this closes pre-existing `SIFR-OWN-0005` debt adopted by the phase, and `AGENTS.md` requires docs updated with status, checklist state, and PR links per item. Neither the commit SHA, the guardrail's scope decision, nor the debt-adoption closure is recorded anywhere at this head.

(The `plans/reviews/active/…diagnostic-contract-remediation-agent-pr-review-pass-1.md` placeholder is present and zero-byte; that is the slot for this review, not a finding.)

---

## NON-BLOCKING 5 — Reported validation evidence never names the new guardrail's test binary

The supplied evidence lists focused diagnostic tests, `sifr_lowering` 955 passed / 1 ignored, the annotated fail suite, both guardrail scripts, `fmt`, and `clippy`. The new guardrail lives in the `crates/sifr` `e2e` test binary (`tests/e2e.rs` → `mod e2e_support` → `mod e2e_entrypoints`), and "focused diagnostic tests" plus "annotated fail suite" are both consistent with filtered runs (`test_e2e_fail`, `-p sifr_lowering`) that would never execute `test_receiver_place_representative_diagnostics_populate_declared_args`.

Static review says it should pass: `sifr_diagnostics` and `sifr_driver` are real `[dependencies]` of `crates/sifr` (`Cargo.toml:16-17`), `e2e.rs:15` allows `clippy::expect_used`/`unwrap_used`, `CARGO_MANIFEST_DIR.parent().parent()` correctly resolves the repo root, `String == &str` compares fine at `:371`, all five `representative_fixture_path` files exist and their annotations pin the expected codes, and all five emitters populate their declared args. But a newly added guardrail whose whole purpose is to fail loudly should have its own execution named in the record. Please add the explicit line, e.g. `cargo test -p sifr --test e2e test_receiver_place_representative_diagnostics_populate_declared_args`.

---

## NON-BLOCKING 6 — Emit-site style drifts from the crate's diagnostics-helper convention

`ownership_diagnostics.rs` is the established single-funnel module for this crate's ownership diagnostics (`borrow_conflict` `:254-267`, `unsupported_mutable_receiver_place` `:269-286`, `immutable_parameter_mutation` `:212-229`), and it is precisely that funnel structure that made the OWN-0005 fix a one-place change covering four call sites. The two PROTO emitters instead build `BTreeMap`s inline in the analysis pass (`method_receiver_analysis.rs:223-231`, `:306-309`), duplicating each value between the `format!` message and the arg map — the exact divergence risk the registry template exists to prevent. Relatedly, `validate_protocol_receiver_conventions` now threads a five-element `(String, TextRange, String, String, String)` tuple (`:291-300`, destructured at `:305`), where a small named struct or a `protocol_receiver_mismatch(ctx, class, method, protocol, range)` helper alongside the ownership helpers would be both shorter and drift-proof. Not a defect at this head; it is the shape that lets message and args diverge later.

---

## Summary

Pass-5 code findings 2, 3, and 4 are substantively closed: all five phase-owned diagnostics populate their registry-declared arguments on the real rendered-envelope path, exact-value tests exist for PROTO-0005 (single and multiple), PROTO-0006, and three OWN-0005 paths, and a live emission guardrail now prevents registry/emitter drift for the phase's five codes. The scoped guardrail is the correct call and does not smuggle in a repository-wide diagnostics migration. The test split is necessary rather than cosmetic and leaves every touched file well under 900 lines.

One blocking defect remains in the changed surface: `SIFR-PROTO-0006` still emits from a randomized `HashSet` with no class discriminator, so the phase's own representative fixture produces two diagnostics that are indistinguishable in code, template, message, and dedupe args, and beyond the five-per-group cap the compiler drops a nondeterministically chosen one on the real user path. Five non-blocking findings cover the untested phase-owned OWN-0005 site and its non-binding `binding` fallback, the unrecorded guardrail scope, the absent planning/archive record, the unnamed guardrail test run, and the emit-site helper drift.

NOT SATISFIED
