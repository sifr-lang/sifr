I have completed the audit. Here are my findings.

---

# Whole-Phase Implementation Review Pass 5 — Class-Field Mutating-Receiver Place Semantics

**Exact head reviewed:** `7f6546a80538a168fe3cf81a9b195cbbcd825647` (== PR #3088 `headRefOid`, draft, `MERGEABLE`, base `main`)
**Scope:** merged PRs #3065, #3082, #3087, #3090, #3092, #3094; archive `plans/issues/archive/ad-hoc-class-field-mutating-receiver-place-semantics.md`; 40+ review artifacts; diagnostics registry/docs; fixtures and manifest. Read-only; no builds, tests, or sweeps run.

## Findings

### BLOCKING 1 — Final authoritative merge gate is still non-green (carried, correctly recorded)

Archive lines 918–929 record the integrated default merge-profile attempt at code-identical head `738402910a22932eb98c267be27fd919ab408821`: all functional lanes green (coverage/core, diagnostics, CPython differential 2/2, Python interop 25/25, Rust interop 10/10, frontend/syntax 4/4, developer tooling 32/32), but representative budget checking rejected project graph (`1394.933ms > 1357.524ms`), arithmetic (`1378.214ms > 1334.139ms`), and JSON diagnostics (`1344.143ms > 1335.954ms`), with an unrelated nightly profile overlapping the measurement window. The archive honestly declines to accept this as a green gate and requires an uncontended exit-0 run. That requirement is unmet at this head, so closure cannot be granted regardless of the implementation audit.

### BLOCKING 2 — Both diagnostics newly reserved by this phase never populate their declared message+JSON arguments

This is the same defect class that whole-phase pass 1 and pass 2 treated as blocking for `SIFR-OWN-0002` — and it is unclosed for the two codes this phase itself reserved.

- `crates/sifr_lowering/src/lower/method_receiver_analysis.rs:222-229` emits `PROTO_FIXED_RECEIVER_MUTATION` (`SIFR-PROTO-0006`) via `ctx.error_with_code_at`.
- `crates/sifr_lowering/src/lower/method_receiver_analysis.rs:290-294` emits `PROTO_RECEIVER_CONVENTION_MISMATCH` (`SIFR-PROTO-0005`) via `ctx.error_with_code_at`.
- `crates/sifr_lowering/src/lower/mod_context.rs:343-350`: `error_with_code_at` forwards `BTreeMap::new()` — the emitted diagnostics carry **zero** structured arguments.

Yet the arguments are declared as required in three places:

- `crates/sifr_diagnostics/src/codes/registry/registry_entries/calls_flow_and_protocols.rs:420-421` — `[arg!("class_name"), arg!("method"), arg!("protocol")]` / `["class_name","method","protocol"]`
- `crates/sifr_diagnostics/src/codes/registry/registry_entries/calls_flow_and_protocols.rs:431-432` — `[arg!("method"), arg!("trait_name")]` / `["method","trait_name"]`
- `internal_docs/diagnostic_codes.md:221-222` documents them as `class_name (message+json)`, `method (message+json)`, `protocol (message+json)` and `method (message+json)`, `trait_name (message+json)`

And the phase plan reserves both codes with an explicit `message+JSON arguments` clause (archive lines 687 and 699).

The codebase demonstrably knows the correct pattern — `ownership_diagnostics.rs:247-261` (`borrow_conflict`, the single funnel for all `SIFR-OWN-0002` paths), `:262-279` (`unsupported_mutable_receiver_place`, `place`), and `:305-311` (constructor path, `place=self`) all use `error_with_code_args_help_at` with populated args. Only the two new PROTO codes were missed.

Failure scenario: a program with a mutable class implementation of a shared-receiver protocol method emits `SIFR-PROTO-0005` whose JSON `args` object is empty, so LSP/tooling consumers cannot extract `class_name`, `method`, or `protocol`. Additionally, `crates/sifr_driver/src/diagnostics.rs:268-283` (`recovery_dedupe_args`) returns an empty key-arg vector when declared args are absent, so `SimilarDiagnosticGroupKey` (`:208-215`, keyed on code + template + dedupe args + primary *file*) collapses two distinct protocol mismatches in the same file into one group — losing a diagnostic that populated args would have kept distinct.

### BLOCKING 3 — `SIFR-OWN-0005` `binding` argument is likewise never populated, including at the phase's new emit site

`crates/sifr_lowering/src/lower/ownership_diagnostics.rs:212-222` (`immutable_parameter_mutation`) emits `OWN_IMMUTABLE_PARAMETER_MUTATION` through `error_with_code_at`, so no `binding` argument reaches the diagnostic — while `crates/sifr_diagnostics/src/codes/registry/registry_entries/calls_flow_and_protocols.rs:123-124` declares `[arg!("binding")]` / `["binding"]` and the template is `cannot mutate through immutable parameter {binding}`.

The phase plan assigns this explicitly (archive lines 664-666): "`SIFR-OWN-0005`: … Populate `binding` with the root binding and point at the receiver expression."

The phase adds a new emit site that makes the omission concrete: `crates/sifr_lowering/src/lower/method_receiver_places.rs:587-592` (`report_immutable_root`) resolves the root binding name from retained binding facts and then discards it into an unstructured message. Both phase fail fixtures (`crates/sifr/tests/e2e/fail/immutable_borrowed_parameter_field_receiver.sifr`, `immutable_owned_parameter_field_receiver.sifr`) exercise this path and therefore produce argument-less `SIFR-OWN-0005`.

In fairness to scope attribution: the helper itself predates the phase (introduced by PR #1689, per `git log -L`), and three of its four call sites (`binding_mutability.rs:14`, `mutating_methods.rs:71`, `:118`) are pre-existing. I still classify this as blocking because the phase's own Diagnostics section takes ownership of the requirement and the phase adds a fourth emit site, but it should be recorded as pre-existing debt formally adopted by this phase, not as a regression introduced by it.

### NON-BLOCKING 4 — No guardrail asserts that emitted diagnostics carry their registry-declared arguments

`crates/sifr_diagnostics/src/codes/registry_tests.rs` enforces only *static* registry self-consistency: `assert_dedupe_args_are_declared` (`:188-201`) and `assert_template_placeholders_are_declared` (`:203-224`). Nothing checks that a code's actual emission populates its declared args. The lowering tests for these codes (`method_receiver_analysis_tests.rs:622`, `:645`) assert only `error.code == …` and never inspect `error.args`.

This is the direct reason findings 2 and 3 survived twelve Item-2 PR reviews and four prior whole-phase reviews. A guardrail comparing each active code's declared args against the args observed when compiling its `representative_fixture_path` would close the class rather than the instances.

### NON-BLOCKING 5 — PR #3088 body omits the integrated merge-gate evidence added by this very head

The head commit under review is `7f6546a80 docs: record integrated receiver merge gate attempt`, which added archive lines 918–929. The PR body's "Current validation" section still lists #3094's create-PR gate as the latest evidence and reduces the integrated run to a single line: "final integrated default merge profile: pending an uncontended host window."

That line is not false, but it omits the phase's strongest integrated functional evidence (7 lanes green at a code-identical head) and the specific budget misses and nightly-profile contention that disqualified it. Index/slice remediation pass 5 flagged this same class of PR-body/archive divergence. The archive is complete and honest here; only the PR record lags.

## Verified closed / no findings

I re-audited the following end-to-end and found no additional actionable issue:

- **Ambient counter removal:** `pending_self_field_clone_suppression`, `method_call_needs_field_clone_suppression`, `method_mut_arg_needs_field_clone_suppression`, and `body_contains_field_assign_codegen` return zero hits across `crates/`. `crates/sifr_codegen/src/class_method_receiver_analysis.rs` is reduced to a 32-line unrelated helper.
- **Clone-free, fail-closed place emission:** `crates/sifr_codegen/src/place_emitter.rs:60-96` contains no `.clone()`/`.cloned()`/`take()`/temporary, verifies `binding_id == place.root` and each projection's field name, and returns `None` on any unproven shape. All generic and registry method paths route receivers through `lower_method_receiver_place_for_{stmt,registry}` (`:98-122`, `:159-180`), including `recursive_method_calls.rs:63-95`. An unproven `MutableBorrow` shape reaches `class_method_emitter.rs:149-156` and panics as a programmer invariant rather than silently falling back to the value/clone path.
- **Index/slice footprint remediation (#3094):** the two added `collect_footprint(object, …)` recursions at `footprint.rs:108` and `:121` match the archive's description exactly. All three arms that emit `Footprint::Dynamic` (Index `:104-110`, Slice `:111-125`, FieldAccess `:240-249`) now both push the conservative root and recurse into the object subtree. `collect_footprint`'s match is exhaustive with no `_` arm, so new `HirExpr` variants cannot silently escape footprint collection.
- **Callable-field precision (#3092):** `footprint.rs:54-96` preserves exact `FieldIdentity` when the base resolves statically, excludes actual methods via the `methods.iter().any(…)` shadowing guard, and falls back through the object subtree (reaching the conservative root) for dynamic bases.
- **Overlap rule:** `places_overlap` (`footprint.rs:24-28`) is symmetric prefix-on-same-root; `validate_call_overlaps` (`method_receiver_places.rs:208-280`) covers receiver-vs-arg, shared/owned-receiver-vs-mut-arg, and arg-vs-arg with legacy-pair suppression. All five `SIFR-OWN-0002` paths funnel through the single `borrow_conflict` helper, which does populate `binding`.
- **Audited evaluation-order exception:** `indexed_storage.rs:35-54` restricts `borrow_follows_argument_evaluation` to `__sifr_defaultdict_list.extend` and the four `__sifr_defaultdict_set` update-family methods; every other specialized indexed mutation retains the conservative rule.
- **Root eligibility is binding-kind-based, not name-based:** `prove_mutable_place` (`:354-396`) and `receiver_rooted` (`method_receiver_analysis.rs:482-497`) both test `BindingKind`/final convention; no literal `"self"` test appears in eligibility.
- **Fixed-point inference and fixed-trait contract:** `infer_and_annotate_class_receivers` (`method_receiver_analysis.rs:17-97`) iterates to convergence, excludes protocols from seeding, honors `Owned` and the fixed-trait registry, then persists, propagates inheritance, validates PROTO-0005/0006, and re-annotates calls from final metadata.
- **Validation coverage of all bodies:** `module_body_lowering.rs:21-51` validates class methods, operator impls, and module functions; `type_visit.rs:636` confirms `transform_hir_function` descends into `HirStmt::NestedFunction`, and lambda bodies are visited as expressions.
- **Optimizer protection:** `protected_mutable_place_roots` is populated during checked place emission (`place_emitter.rs:47`, `:81`) and passed to `remove_unneeded_mutability_in_items` in both production (`lib_modules_and_codegen.rs:630`) and test-module assembly (`entrypoints.rs:159`).
- **Docs and manifest:** `docs/errors/SIFR-OWN-0002.mdx` scope is widened to overlapping shared reads and owned moves as required; `SIFR-OWN-0014`, `SIFR-PROTO-0005`, `SIFR-PROTO-0006` pages exist and are registry-generated; `verification/areas/core_language/data/create_pr_e2e_manifest.json` contains all seven prior native phase pass fixtures plus #3094's `class_field_dynamic_index_base_disjoint`.
- **Debt attribution is honest:** the pre-existing CFG panic-hook, value-codegen move/borrow, and match-lowering debts (archive lines 1072-1076, 1227-1232, 1321-1325) are each recorded as reproducing on untouched bases and explicitly not treated as closure exceptions.

## Recommended remediation

1. Route `SIFR-PROTO-0005` and `SIFR-PROTO-0006` through `error_with_code_args_help_at` with their declared args, following `unsupported_mutable_receiver_place` as the model; extend `method_receiver_analysis_tests.rs:615-647` to assert the arg values, not just the code.
2. Populate `binding` in `immutable_parameter_mutation`, and record in the archive that this closes pre-existing `SIFR-OWN-0005` debt adopted by the phase.
3. Add a registry guardrail that compiles each active code's `representative_fixture_path` and asserts the emitted diagnostic carries every `declared_args` entry — this closes finding 4 and prevents recurrence.
4. Refresh the PR #3088 body with the integrated attempt's lane results, the three budget misses, and the contention cause, so the PR record matches the archive.
5. Re-run `scripts/run_all_tests.sh` on an uncontended host and require exit 0 before closure.

NOT SATISFIED
