## `milestone_diag_4a` slice 2b.7 — HIR missing type-annotation migration to active `SIFR-TYPE-0004` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-annotation-types`.
- Target: migrate every HIR call site that today emits the free-form "missing a type annotation" string through the legacy `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge ([sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)) onto active `SIFR-TYPE-0004` via `LowerCtx::error_with_code` ([sifr_hir/src/lower/mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228)), and add representative e2e fixtures pinning the new code+substring contract.
- Files changed:
  - [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:316) — three sites: regular positional, vararg, keyword-only.
  - [crates/sifr_hir/src/lower/classes.rs](../crates/sifr_hir/src/lower/classes.rs:405) — two sites: `__init__` params, regular method params.
  - [crates/sifr_hir/src/lower/nested_function_inference.rs](../crates/sifr_hir/src/lower/nested_function_inference.rs:439) — one site: nested-function inference failure.
  - [crates/sifr/tests/e2e/fail/missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/missing_type_annotation.sifr:1) — regular positional fixture (also the registry's representative fixture for `SIFR-TYPE-0004` at [codes.rs:597](../crates/sifr_diagnostics/src/codes.rs:597)).
  - [crates/sifr/tests/e2e/fail/missing_vararg_type_annotation.sifr](../crates/sifr/tests/e2e/fail/missing_vararg_type_annotation.sifr:1) — vararg fixture.
  - [crates/sifr/tests/e2e/fail/missing_keyword_only_type_annotation.sifr](../crates/sifr/tests/e2e/fail/missing_keyword_only_type_annotation.sifr:1) — keyword-only fixture.
  - [crates/sifr/tests/e2e/fail/class_init_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/class_init_missing_type_annotation.sifr:1) — `__init__` fixture.
  - [crates/sifr/tests/e2e/fail/class_method_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/class_method_missing_type_annotation.sifr:1) — class regular method fixture.
  - [crates/sifr/tests/e2e/fail/nested_function_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/nested_function_missing_type_annotation.sifr:1) — nested-function fixture.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) — slice 2b.6 flipped to merged with PR #1678; slice 2b.7 line added.
- Validation already executed by the implementer: `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=71.17s`).

## Findings

### F1 — Taxonomy choice: `SIFR-TYPE-0004` is the correct active code for all six sites

`SIFR-TYPE-0004` ("A required type annotation is missing.", [codes.rs:592-602](../crates/sifr_diagnostics/src/codes.rs:592)) carries the family-level meaning "a *required* annotation was *not provided*". Each migrated emission is exactly that semantic: `param.parameter.annotation` was `None` for a callable parameter where Sifr's surface-language contract requires an explicit annotation. None of the migrated sites is doing annotation-shape parsing (which would belong to `TYPE_INVALID_ANNOTATION = SIFR-TYPE-0007` at [codes.rs:34](../crates/sifr_diagnostics/src/codes.rs:34)/[625-635](../crates/sifr_diagnostics/src/codes.rs:625)) or type-name resolution (`NAME_UNKNOWN_TYPE = SIFR-NAME-0003`). The split is clean: the migrated branch is `if Some(ann) → resolve; else → emit TYPE-0004`, with `Type::Any` as the recovery payload so downstream lowering still produces useful diagnostics for the rest of the function. The `Type::Any` recovery exactly mirrors the prior bridge behavior, so this slice does not change downstream behavior beyond the diagnostic identity.

### F2 — All six sites in stated scope are migrated, with 1:1 fixture coverage

Stated scope is "top-level function regular params, varargs, keyword-only params, class `__init__` params, class method params, and nested local function parameters that cannot be inferred". The six call sites match the six fixtures one-to-one:

| Site | Code path | Fixture | Fixture's `expect-error` substring |
|---|---|---|---|
| Top-level regular positional | [typing_and_functions.rs:316](../crates/sifr_hir/src/lower/typing_and_functions.rs:316) | [missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/missing_type_annotation.sifr:1) | `parameter 'value' in function 'identity' is missing a type annotation` |
| Top-level vararg | [typing_and_functions.rs:335](../crates/sifr_hir/src/lower/typing_and_functions.rs:335) | [missing_vararg_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/missing_vararg_type_annotation.sifr:1) | `vararg parameter 'values' in function 'total' is missing a type annotation` |
| Top-level keyword-only | [typing_and_functions.rs:355](../crates/sifr_hir/src/lower/typing_and_functions.rs:355) | [missing_keyword_only_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/missing_keyword_only_type_annotation.sifr:1) | `parameter 'verbose' in function 'display' is missing a type annotation` |
| Class `__init__` param | [classes.rs:405](../crates/sifr_hir/src/lower/classes.rs:405) | [class_init_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/class_init_missing_type_annotation.sifr:1) | `parameter 'value' in Box.__init__ is missing a type annotation` |
| Class regular method param | [classes.rs:457](../crates/sifr_hir/src/lower/classes.rs:457) | [class_method_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/class_method_missing_type_annotation.sifr:1) | `parameter 'value' in Tool.scale is missing a type annotation` |
| Nested function inference failure | [nested_function_inference.rs:439](../crates/sifr_hir/src/lower/nested_function_inference.rs:439) | [nested_function_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/nested_function_missing_type_annotation.sifr:1) | `parameter 'n' in function 'helper' is missing a type annotation and could not be inferred` |

Each fixture's substring is a verbatim slice of the new emitted text (the only formatting change is moving from `ctx.error(...)` to `ctx.error_with_code(DiagnosticCode::TYPE_MISSING_ANNOTATION, ...)`; the format string itself is unchanged). The e2e harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) requires both the code AND the optional substring to match a single emitted diagnostic — every fixture's `expect-error` line satisfies both halves.

### F3 — A `grep` for "missing a type annotation" finds exactly the six migrated sites

```
crates/sifr_hir/src/lower/typing_and_functions.rs:319    (regular positional)
crates/sifr_hir/src/lower/typing_and_functions.rs:338    (vararg)
crates/sifr_hir/src/lower/typing_and_functions.rs:358    (keyword-only)
crates/sifr_hir/src/lower/classes.rs:408                 (__init__)
crates/sifr_hir/src/lower/classes.rs:460                 (regular method)
crates/sifr_hir/src/lower/nested_function_inference.rs:442 (nested inference failure)
```

There is no remaining `ctx.error(...)` call in the HIR lowering tree carrying any "missing a type annotation" / "missing type annotation" substring, so no in-scope site escapes onto the legacy `SIFR-TYPE-0001` bridge. The legacy bridge at [diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is correctly left untouched — bridge deletion is explicitly out of scope per the issue checklist at [issue:43](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43).

### F4 — Nested-function migration site is correctly the *inference-failure* site, not the *missing-annotation* site

The `infer_nested_function_types` pipeline at [nested_function_inference.rs:108](../crates/sifr_hir/src/lower/nested_function_inference.rs:108) treats unannotated nested-function params as inference candidates rather than immediate errors. The candidate is seeded with `Type::Unknown` at [nested_function_inference.rs:172](../crates/sifr_hir/src/lower/nested_function_inference.rs:172) (`explicit = false`), then iterative inference attempts to bind it. The error is emitted only at finalize time, gated by `!param.explicit && param.ty.is_unknown()` at [nested_function_inference.rs:437](../crates/sifr_hir/src/lower/nested_function_inference.rs:437). The fixture at [nested_function_missing_type_annotation.sifr:3](../crates/sifr/tests/e2e/fail/nested_function_missing_type_annotation.sifr:3) is `def helper(n): return 1` whose body provides no constraint on `n`, so inference legitimately fails and the new code surfaces. This is the right site to migrate — the migrated message text retains the "and could not be inferred" suffix that distinguishes this from the top-level "annotation missing" cases, and that suffix is in the fixture substring. There is no second migrated site in the nested-function path, and there does not need to be: the only other emission in this module ([line 454](../crates/sifr_hir/src/lower/nested_function_inference.rs:454)) is the return-type inference failure, which is conceptually a different diagnostic (it would belong to a future return-type code, not `SIFR-TYPE-0004`).

### F5 — Class-method migration is at the type-collection pass, not the body-lowering pass — and that is correct

`collect_class_type` ([classes.rs:169](../crates/sifr_hir/src/lower/classes.rs:169)) is the type-collection pass invoked from [mod.rs:584](../crates/sifr_hir/src/lower/mod.rs:584) and [mod.rs:594](../crates/sifr_hir/src/lower/mod.rs:594). Class-method bodies are subsequently lowered through `lower_class` ([classes.rs:588](../crates/sifr_hir/src/lower/classes.rs:588)), where regular-class method param iteration at [classes.rs:862-868](../crates/sifr_hir/src/lower/classes.rs:862) silently defaults to `Type::Any` when the annotation is missing — but that body-lowering pass is reached *after* `collect_class_type` has already emitted the migrated `SIFR-TYPE-0004`, so the silent fallback is just absorbing the same already-flagged condition rather than swallowing it. Migrating the body-lowering pass too would have been redundant and would have produced a third copy of the same diagnostic. Picking only the type-collection pass for the migration is the right call.

### F6 — Out-of-scope sites are correctly *not* migrated

The slice scope explicitly defers:

- *Annotation shape* errors (`SIFR-TYPE-0007`). These remain on the bridge at [typing_and_functions.rs:415](../crates/sifr_hir/src/lower/typing_and_functions.rs:415), [419](../crates/sifr_hir/src/lower/typing_and_functions.rs:419), [430](../crates/sifr_hir/src/lower/typing_and_functions.rs:430), [456](../crates/sifr_hir/src/lower/typing_and_functions.rs:456), [511](../crates/sifr_hir/src/lower/typing_and_functions.rs:511), [683](../crates/sifr_hir/src/lower/typing_and_functions.rs:683) — all six remain `ctx.error(...)` and are correctly *not* repointed to `TYPE-0004`. Repointing them would conflate "missing required annotation" (`TYPE-0004`) with "annotation present but malformed" (`TYPE-0007`).
- *Result error-type validity*, *unknown type names*, and *generic arity errors*. None of those sites are touched in this branch's diff.
- *Bridge deletion*. The `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` arm is intact, which is required for the still-unmigrated annotation-shape and Result-validity paths to surface anything at all.
- ***kwargs / posonlyargs***. Sifr does not currently lower `**kwargs` or positional-only params (the type-extraction loops at [typing_and_functions.rs:311-366](../crates/sifr_hir/src/lower/typing_and_functions.rs:311) cover only `args`, `vararg`, and `kwonlyargs`; classes.rs covers only `args`). [phase 02 docs:387](../internal_docs/phases/02_type_system_power.md:387) confirms `*args` and `**kwargs` are slated for `milestone_decorators`. So there is no pre-existing emission site for missing annotations on those param shapes to migrate; the slice's call-site set is correctly closed against current Sifr surface area.

### F7 — Issue checklist transitions are clean

[issue:41](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:41) flips slice 2b.6 from "Started" / "implementation complete and reviewer-satisfied" to `[x] merged ... PR: ...pull/1678`, matching the merged PR identifier from the prior review's verdict. [issue:42](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) adds slice 2b.7 with "Started" status and an accurate scope statement. The slice 2b.6 / slice 2b.7 wording mirrors the established cadence used for slices 2b.3, 2b.4, and 2b.5. No unrelated checklist drive-bys.

### F8 — Diff is tightly scoped; no orphan fixtures, no stale callers

`git status` shows exactly four modified files (the three HIR sources and the issue) plus six untracked fixtures — nothing else. A repo-wide `grep` for any `expect-error` line containing "missing a type annotation" or "missing type annotation" returns only the six new fixtures, so there is no pre-existing fixture left pinning the old `SIFR-TYPE-0001` bridge for missing-annotation messages that would now silently fail or pass against the wrong code. No baselines, no `verification/`, no docs, no schema changes — the diff is genuinely just emission migration plus fixtures plus the checklist edit.

## Residual risks

### R1 — Class missing-annotation diagnostic likely fires twice per param due to the dual `collect_class_type` invocation

[mod.rs:584](../crates/sifr_hir/src/lower/mod.rs:584) and [mod.rs:594](../crates/sifr_hir/src/lower/mod.rs:594) both invoke `collect_class_type` for each class (the second pass refreshes class shapes after type-alias resolution per the comment at [mod.rs:580-581](../crates/sifr_hir/src/lower/mod.rs:580)). `collect_class_type` has no idempotency check on its error-emission paths, so the migrated `SIFR-TYPE-0004` calls at [classes.rs:405](../crates/sifr_hir/src/lower/classes.rs:405) and [classes.rs:457](../crates/sifr_hir/src/lower/classes.rs:457) will each push a second identical `LoweringError` onto `ctx.errors` on the second pass. Pre-migration, this same path was emitting the same legacy bridge error twice, so this is not a regression introduced by the slice — it is a pre-existing double-firing carried forward.

The downstream impact is partly muted: `apply_diagnostic_recovery_limits` at [sifr_driver/src/diagnostics.rs:179-202](../crates/sifr_driver/src/diagnostics.rs:179) groups by `(severity, code, message, file)` but retains up to `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` per group — it does not collapse exact duplicates within a group, so a user can still see the identical `SIFR-TYPE-0004: parameter 'value' in Box.__init__ is missing a type annotation` message twice. The e2e harness uses `errors.iter().any(...)` at [e2e.rs:2561](../crates/sifr/tests/e2e.rs:2561), so duplicate emissions don't fail the new fixtures, but they do bloat the user-facing output for class missing-annotation cases. Out of scope for this slice; would naturally fall to the future builder-migration / dedupe-introduction slice that resolves the registry's `dedupe_args` metadata into runtime suppression.

### R2 — Several pre-existing class-method emission paths still default to `Type::Any` *without* emitting any diagnostic

These sites exist on this branch and were not in scope:

- Enum-method type collection at [classes.rs:265-269](../crates/sifr_hir/src/lower/classes.rs:265).
- Protocol-method type collection at [classes.rs:299-303](../crates/sifr_hir/src/lower/classes.rs:299).
- Enum-method body lowering at [classes.rs:658-662](../crates/sifr_hir/src/lower/classes.rs:658).
- Newtype-method body lowering at [classes.rs:736-740](../crates/sifr_hir/src/lower/classes.rs:736).

Unlike the regular-class case (F5 above), enum/protocol/newtype methods do **not** route through `collect_class_type`'s migrated branches at lines 405/457 — they have their own type-collection paths above (lines 256-283 for enums, 290-313 for protocols), and those paths silently fall back to `Type::Any` for missing annotations with no error emission at all. So an enum or protocol or newtype method written with an unannotated param today compiles silently and the param is treated as `Type::Any`. This is a pre-existing gap (the legacy bridge would not have fired here either), but it is conceptually within "class method params" if read broadly. The slice's scope statement says "class method params" which the implementer interpreted as `class C: def m(self, x): ...`, not enum/protocol/newtype methods — defensible given the gap pre-dates this work, but worth surfacing for the followup slice that broadens diagnostic coverage. Recommended next step: either extend the migration to those sites in a small follow-up, or explicitly capture this as a tracked scope item for the bridge-deletion slice (since the bridge can't be deleted while these silent-fallback paths exist on enum/protocol/newtype methods).

### R3 — Nested-function inference path covers only `func.parameters.args`

`collect_nested_function_states` at [nested_function_inference.rs:167](../crates/sifr_hir/src/lower/nested_function_inference.rs:167) iterates only `func.parameters.args` to seed inference candidates. A nested function with `*args` or `*, x` keyword-only params would not be seeded, would not be inferred, and would not be flagged by the migrated finalize-time error at line 437. In practice this means the migration's nested-function scope is "regular positional only" — narrower than the top-level scope, which covers vararg and kwonly via [typing_and_functions.rs:330-366](../crates/sifr_hir/src/lower/typing_and_functions.rs:330). Pre-existing nested-function design decision, not introduced here. The slice's scope statement says "nested local function parameters that cannot be inferred", which is consistent with the implementation's "regular positional only" reality (since varargs/kwonly are not currently inferrable in nested form to begin with).

### R4 — Registry `message_template` for `SIFR-TYPE-0004` does not match any of the six emitted format strings

The registry at [codes.rs:598](../crates/sifr_diagnostics/src/codes.rs:598) declares `"missing type annotation for {name}"` with `declared_args = [name, declaration_kind]` ([codes.rs:600](../crates/sifr_diagnostics/src/codes.rs:600)). The six migrated emissions are:

1. `parameter '{name}' in function '{func}' is missing a type annotation` (regular positional, kwonly).
2. `vararg parameter '{name}' in function '{func}' is missing a type annotation`.
3. `parameter '{name}' in {class}.__init__ is missing a type annotation`.
4. `parameter '{name}' in {class}.{method} is missing a type annotation`.
5. `parameter '{name}' in function '{func}' is missing a type annotation and could not be inferred`.

None of these match the template, and the template's second declared arg (`declaration_kind`, JSON-only per [codes.rs:600](../crates/sifr_diagnostics/src/codes.rs:600)'s `json_arg!`) is never threaded through `ctx.error_with_code` because the current emission API at [mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228) takes only a pre-formatted `String` — declared args are documentation-only at this state. This is structurally identical to slice 2b.6 R2 (registry args not yet runtime-exercised) and is consistent with the decision to defer the `DiagnosticBuilder` placeholder pipeline to a later slice ([issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432)). The fixtures correctly assert on the rendered text rather than the template, so e2e contracts remain stable. Non-blocking, but the question of whether `declaration_kind` should be `"function-param" | "vararg" | "kwonly-param" | "init-param" | "method-param" | "nested-inferred-param"` (or some narrower enum) is now a concrete design question for the builder-migration slice — five of the six emissions render slightly different shapes that would each map to a distinct `declaration_kind` value if the template is to subsume them.

### R5 — No registry-level test guards `SIFR-TYPE-0004`'s `representative_fixture_path` against fixture rename or substring drift

[codes.rs:597](../crates/sifr_diagnostics/src/codes.rs:597) hardcodes `"crates/sifr/tests/e2e/fail/missing_type_annotation.sifr"` as the representative fixture. The registry tests at [codes.rs:1465](../crates/sifr_diagnostics/src/codes.rs:1465) only assert that `representative_fixture_path` is `Some(...)` for active codes — not that the path exists, not that the fixture contains a matching `expect-error` line. If the fixture were renamed or its `expect-error` re-keyed in a future slice, the registry would silently desync (the same risk pattern flagged as R3 for slice 2b.5 / closed in slice 2b.6 by manual repointing). Out of scope for this slice; ideally absorbed by `scripts/check_diagnostic_code_coverage.py` planned in `milestone_diag_11` per [issue:1236](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1236).

### R6 — Fixture style intentionally minimal; no negative test for "annotation present, code does NOT fire"

Each new fixture exercises exactly one missing-annotation site and asserts exactly one `expect-error`. There is no companion fixture that proves a *correctly annotated* version of the same code emits no `SIFR-TYPE-0004` (and no other diagnostic) — i.e., the `expect-pass` direction. Because the six annotated equivalents are essentially the universal happy path of every Sifr program, the existing pass suite implicitly covers this — but a future regression that, say, made `extract_function_type` always emit `TYPE-0004` regardless of `annotation.is_some()` would only be caught by the wide pass suite, not by anything fixture-local to this code. Non-blocking; consistent with the established style of prior slices.

## Verdict

Satisfied / no blocking findings. Slice 2b.7 closes the six in-scope HIR call sites for missing parameter annotations onto active `SIFR-TYPE-0004` with one fixture per site (each pinning both the new code and a verbatim substring of the unchanged emitted text), correctly leaves the annotation-shape, unknown-type-name, generic-arity, Result-error, and bridge-deletion paths untouched, and respects the pre-existing nested-function and class-shape (enum/protocol/newtype) gaps that were already present on the bridge. Residual risks are either pre-existing (R1 dual-pass duplicate emission, R2 enum/protocol/newtype silent fallbacks, R3 nested-inference scope), structural across the milestone (R4 template/runtime drift, R5 registry-fixture binding, R6 negative-test gap), or both — all are correctly deferred to follow-up slices. The local validation set the implementer reports (`report_signature=e1bf653aaa770517`, matching the signature pinned across slices 2b.3-2b.6) is the established gate, and the diff is exactly the surface area the scope statement promises.
