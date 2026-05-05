## `milestone_diag_4a` slice 2b.8 — HIR invalid type-annotation shape migration to active `SIFR-TYPE-0007` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-invalid-annotations`.
- Target: migrate every HIR call site inside [`resolve_annotation_expr`](../crates/sifr_hir/src/lower/typing_and_functions.rs:384) that today emits an annotation-shape error string through the legacy `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge ([sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)) onto active `SIFR-TYPE-0007` via `LowerCtx::error_with_code` ([sifr_hir/src/lower/mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228)), and add representative e2e fixtures pinning the new code+substring contract.
- Files changed:
  - [crates/sifr_hir/src/lower/typing_and_functions.rs](../crates/sifr_hir/src/lower/typing_and_functions.rs:380) — introduces a private `invalid_type_annotation` helper (line 380–382) and re-routes 11 call sites inside `resolve_annotation_expr` (lines 419, 423, 437, 453, 463, 498, 518, 542, 555, 574, 699).
  - [crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr:1) — `dict type annotation requires [K, V] syntax` (also the registry's representative fixture for `SIFR-TYPE-0007` at [codes.rs:630](../crates/sifr_diagnostics/src/codes.rs:630)).
  - [crates/sifr/tests/e2e/fail/invalid_float_literal_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_float_literal_type_annotation.sifr:1) — non-integer literal in annotation position.
  - [crates/sifr/tests/e2e/fail/invalid_type_annotation_expression.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_expression.sifr:1) — catch-all unsupported expression (`int + str`).
  - [crates/sifr/tests/e2e/fail/invalid_type_annotation_base.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_base.sifr:1) — non-name subscript base (`make_type()[int]`).
  - [crates/sifr/tests/e2e/fail/dict_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/dict_type_annotation_wrong_arity.sifr:1) — `dict[int, str, bool]`.
  - [crates/sifr/tests/e2e/fail/result_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_arity.sifr:1) — `Result[int, ValueError, ValueError]`.
  - [crates/sifr/tests/e2e/fail/result_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_syntax.sifr:1) — `Result[int]` (non-tuple slice).
  - [crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_arity.sifr:1) — `Callable[[int], str, bool]`.
  - [crates/sifr/tests/e2e/fail/callable_type_annotation_param_list_required.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_param_list_required.sifr:1) — `Callable[int, str]` (first slice element not a list).
  - [crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_syntax.sifr:1) — `Callable[int]` (non-tuple slice).
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) — slice 2b.7 flipped to merged with PR #1679; slice 2b.8 added with "Started" status.
- Validation already executed by the implementer: `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=85.60s`).

## Findings

### F1 — Taxonomy choice: `SIFR-TYPE-0007` is the correct active code for all 11 migrated sites

`SIFR-TYPE-0007` ("Invalid type annotation shape.", [codes.rs:625-635](../crates/sifr_diagnostics/src/codes.rs:625)) carries the family-level meaning "an annotation expression *was provided* but is structurally malformed". Each migrated emission inside `resolve_annotation_expr` is exactly that semantic — the user wrote *something* in annotation position, but the shape is unsupported, has the wrong arity, or uses the wrong slice form. The split against the adjacent diagnostics is clean:

- "annotation absent" → `SIFR-TYPE-0004` (slice 2b.7, callers' responsibility — `resolve_annotation_expr` is reached only when an annotation is present).
- "annotation references an unknown name" → `SIFR-NAME-0003`, still on the bridge at [typing_and_functions.rs:400](../crates/sifr_hir/src/lower/typing_and_functions.rs:400) and [692](../crates/sifr_hir/src/lower/typing_and_functions.rs:692).
- "annotation present and parsed, but has the wrong arity / slice form / literal kind / base shape" → `SIFR-TYPE-0007` (this slice).
- "annotation references a Result error type that doesn't extend `Error`" → semantic validity, deliberately on the bridge at [typing_and_functions.rs:509](../crates/sifr_hir/src/lower/typing_and_functions.rs:509).
- "generic alias / generic class arity mismatch" → still on the bridge at [595](../crates/sifr_hir/src/lower/typing_and_functions.rs:595), [636](../crates/sifr_hir/src/lower/typing_and_functions.rs:636), [642](../crates/sifr_hir/src/lower/typing_and_functions.rs:642), correctly out of scope.

Each migrated site returns `Type::Any` as the recovery payload, exactly mirroring prior bridge behavior — the diagnostic identity is the only behavior change.

### F2 — Centralized helper `invalid_type_annotation` is a clean DRY pattern

[typing_and_functions.rs:380-382](../crates/sifr_hir/src/lower/typing_and_functions.rs:380):

```rust
fn invalid_type_annotation(ctx: &mut LowerCtx, message: impl Into<String>) {
    ctx.error_with_code(DiagnosticCode::TYPE_INVALID_ANNOTATION, message.into());
}
```

`impl Into<String>` lets every call site pass a `&'static str` literal (10 of 11 sites) without an explicit `.to_string()`, while still accepting the one `format!` call shape if one were ever needed. The helper is `fn` (not `pub(super)`), correctly scoped to the file. This is a small but real readability gain over inlined `ctx.error_with_code(DiagnosticCode::TYPE_INVALID_ANNOTATION, "...".to_string())` — it also gives a single-point grep target if the code identity ever needs to be retargeted.

### F3 — All 11 in-scope emission sites are migrated, with 10 1:1 fixture matches

Stated scope is "malformed literal annotations, unsupported annotation base/expression shapes, malformed dict/Result/Callable annotation arity or syntax". The 11 `invalid_type_annotation` call sites map onto fixtures as follows:

| # | Code site | Emitted message | Fixture | Fixture's `expect-error` substring |
|---|---|---|---|---|
| 1 | [419](../crates/sifr_hir/src/lower/typing_and_functions.rs:419) | `integer literal too large for type annotation` | *(none — see R3)* | — |
| 2 | [423](../crates/sifr_hir/src/lower/typing_and_functions.rs:423) | `only integer literals are supported in type annotations` | [invalid_float_literal_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_float_literal_type_annotation.sifr:1) | `only integer literals are supported in type annotations` |
| 3 | [437](../crates/sifr_hir/src/lower/typing_and_functions.rs:437) | `unsupported type annotation base` | [invalid_type_annotation_base.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_base.sifr:1) | `unsupported type annotation base` |
| 4 | [453](../crates/sifr_hir/src/lower/typing_and_functions.rs:453) | `dict type annotation requires exactly 2 type parameters` | [dict_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/dict_type_annotation_wrong_arity.sifr:1) | `dict type annotation requires exactly 2 type parameters` |
| 5 | [463](../crates/sifr_hir/src/lower/typing_and_functions.rs:463) | `dict type annotation requires [K, V] syntax` | [invalid_type_annotation.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr:1) (also the registry's representative fixture, [codes.rs:630](../crates/sifr_diagnostics/src/codes.rs:630)) | `dict type annotation requires [K, V] syntax` |
| 6 | [498](../crates/sifr_hir/src/lower/typing_and_functions.rs:498) | `Result type annotation requires exactly 2 type parameters` | [result_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_arity.sifr:1) | `Result type annotation requires exactly 2 type parameters` |
| 7 | [518](../crates/sifr_hir/src/lower/typing_and_functions.rs:518) | `Result type annotation requires [T, E] syntax` | [result_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/result_type_annotation_wrong_syntax.sifr:1) | `Result type annotation requires [T, E] syntax` |
| 8 | [542](../crates/sifr_hir/src/lower/typing_and_functions.rs:542) | `Callable type requires exactly 2 type parameters: [[param_types], return_type]` | [callable_type_annotation_wrong_arity.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_arity.sifr:1) | full message |
| 9 | [555](../crates/sifr_hir/src/lower/typing_and_functions.rs:555) | `Callable parameter types must be a list: Callable[[int, str], bool]` | [callable_type_annotation_param_list_required.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_param_list_required.sifr:1) | full message |
| 10 | [574](../crates/sifr_hir/src/lower/typing_and_functions.rs:574) | `Callable type requires [[param_types], return_type] syntax` | [callable_type_annotation_wrong_syntax.sifr](../crates/sifr/tests/e2e/fail/callable_type_annotation_wrong_syntax.sifr:1) | full message |
| 11 | [699](../crates/sifr_hir/src/lower/typing_and_functions.rs:699) | `unsupported type annotation expression` | [invalid_type_annotation_expression.sifr](../crates/sifr/tests/e2e/fail/invalid_type_annotation_expression.sifr:1) | `unsupported type annotation expression` |

Each fixture is a minimal valid Sifr program (`def consume(value: <bad>) -> int: return 0` plus a trivial `main`), the bad annotation triggers exactly one of the 11 sites, and the `expect-error` line pins both the code (`SIFR-TYPE-0007`) and a verbatim slice of the emitted text. The e2e harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) (`failure.code == expected.code && failure.message.contains(message)`) is satisfied for each. Site #1 (`integer literal too large …`) is the one migration without a dedicated fixture — see R3.

### F4 — Migrating `resolve_annotation_expr` covers every annotation-shape entry point in the HIR

`resolve_annotation_expr` is the single funnel through which annotation expressions are lowered for every annotation-bearing surface in the language. Repo-wide grep for callers shows:

```
crates/sifr_hir/src/lower/typing_and_functions.rs:314,333,353,369   (function param/return)
crates/sifr_hir/src/lower/expressions.rs:3296,3343                   (expression-position annotations)
crates/sifr_hir/src/lower/type_aliases.rs:136                        (type alias RHS)
crates/sifr_hir/src/lower/classes.rs:266,273,300,307,377,403,455,469,659,674,737,751,865,882
                                                                     (class/enum/protocol/newtype field, method param, method return)
crates/sifr_hir/src/lower/mod.rs:1112                                (top-level annotated assignment)
```

Centralizing the migration inside `resolve_annotation_expr` means **every** annotation in the language — function params, function returns, class fields, class method params/returns, enum/protocol/newtype methods, type alias RHS, statement-level annotated assignments — now emits `SIFR-TYPE-0007` for shape errors without per-callsite changes. This is the right architectural choice for the slice's scope statement ("HIR invalid type annotation shape diagnostics inside `resolve_annotation_expr`") and avoids the multi-site duplication that the missing-annotation slice (2b.7) had to perform because *that* check fires *before* `resolve_annotation_expr` is called.

### F5 — Out-of-scope sites inside `resolve_annotation_expr` are correctly *not* migrated

The slice scope explicitly defers the following adjacent emissions, all of which remain `ctx.error(...)` and are correctly *not* repointed onto `SIFR-TYPE-0007`:

- [typing_and_functions.rs:400](../crates/sifr_hir/src/lower/typing_and_functions.rs:400): `unknown type: '<name>'` — belongs to `SIFR-NAME-0003` (unknown type name).
- [typing_and_functions.rs:509](../crates/sifr_hir/src/lower/typing_and_functions.rs:509): `'<E>' is not a valid error type in Result` — Result error-type semantic validity, conceptually separate from "shape".
- [typing_and_functions.rs:595](../crates/sifr_hir/src/lower/typing_and_functions.rs:595): `generic type alias '<name>' expects N type argument(s), got M` — generic alias arity.
- [typing_and_functions.rs:636](../crates/sifr_hir/src/lower/typing_and_functions.rs:636): `class '<name>' does not declare type parameters; use class C[T]: ...` — generic class shape.
- [typing_and_functions.rs:642](../crates/sifr_hir/src/lower/typing_and_functions.rs:642): `generic class '<name>' expects N type argument(s), got M` — generic class arity.
- [typing_and_functions.rs:692](../crates/sifr_hir/src/lower/typing_and_functions.rs:692): `unknown generic type: '<base_name>'` — unknown generic type name.

Each of these is conceptually distinct from "the annotation expression is structurally malformed" — repointing them to `TYPE-0007` would conflate "shape error in the annotation grammar" with "the annotation refers to an unknown / mis-arity'd / mis-extended *named* type". The split matches the issue's checklist scope.

The legacy `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is correctly left untouched; bridge deletion is the explicit out-of-scope item per [issue:43](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43), and is required for the still-unmigrated sites in the list above to surface anything at all.

### F6 — Issue checklist transitions are clean

[issue:42](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) flips slice 2b.7 from "Started" / "implementation complete and reviewer-satisfied" to `[x] merged ... PR: ...pull/1679`, matching the merged PR identifier from the prior review's verdict. [issue:43](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43) adds slice 2b.8 with "Started" status and an accurate scope statement (`HIR invalid type-annotation shape diagnostics migration to active SIFR-TYPE-0007 with fixture coverage`). The wording mirrors the established cadence used for slices 2b.3 through 2b.7. No unrelated checklist drive-bys.

### F7 — Diff is tightly scoped; no orphan fixtures, no stale callers

`git status` shows exactly two modified files (`typing_and_functions.rs` plus the issue) and ten untracked fixtures — nothing else. A repo-wide grep for any `expect-error` line containing `SIFR-TYPE-0001` returns no fixture pinning any of the 11 migrated annotation-shape messages, so there is no pre-existing fixture left pinning the legacy bridge code for these sites that would now silently fail or pass against the wrong code. No baselines, no `verification/`, no docs, no schema changes — the diff is genuinely just the helper, the 11 emission rewrites, the ten fixtures, and the checklist edit.

### F8 — Registry's `representative_fixture_path` for `SIFR-TYPE-0007` lines up

[codes.rs:630](../crates/sifr_diagnostics/src/codes.rs:630) hardcodes `"crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr"` as the representative fixture for `SIFR-TYPE-0007`. That file now exists with `# expect-error: SIFR-TYPE-0007: dict type annotation requires [K, V] syntax`, which is one of the 11 migrated emissions — the registry pin is satisfied without any registry edit. The pre-existing pin from slice 2b.0/2b.1 was sized correctly for this slice's scope; no metadata drift.

## Residual risks

### R1 — Site #1 (`integer literal too large for type annotation`, line 419) has no dedicated fixture

The migration is correct in code (the helper is invoked identically to the other 10 sites), but no `.sifr` fixture in this slice exercises the i64-overflow branch of the integer-literal arm. Triggering it requires writing something like `def f(value: 99999999999999999999999) -> int: ...` (a literal larger than `i64::MAX`). The slice's scope statement says "representative e2e fail fixtures" rather than "comprehensive" — strictly speaking, every other site has a fixture and the absence of one for site #1 leaves a coverage hole that would only catch a regression that selectively broke this one branch (e.g., a refactor that changed `i.as_i64()` to `i.as_i128()` on this path but not on others). Non-blocking against the scope language as written, but a one-line fixture add would close the gap completely. Recommended follow-up: add `crates/sifr/tests/e2e/fail/integer_literal_too_large_type_annotation.sifr` with `# expect-error: SIFR-TYPE-0007: integer literal too large for type annotation` and a body of `def consume(value: 99999999999999999999999) -> int: return 0` plus the standard `main` shim.

### R2 — Registry `message_template` for `SIFR-TYPE-0007` does not match any of the 11 emitted format strings

The registry at [codes.rs:631](../crates/sifr_diagnostics/src/codes.rs:631) declares `"invalid type annotation for {annotation_kind}"` with `declared_args = ["annotation_kind"]`. None of the 11 migrated emissions follow that template; they each render their own free-form text. The current emission API at [mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228) takes only a pre-formatted `String`, so declared args are documentation-only at this state — same structural drift flagged in slice 2b.7 R4 and slice 2b.6. The fixtures correctly assert on the rendered text rather than the template, so e2e contracts remain stable. Non-blocking, but the question of how the 11 distinct emission shapes should fold into a single `{annotation_kind}` placeholder (e.g., `"dict-arity"`, `"dict-syntax"`, `"result-arity"`, `"result-syntax"`, `"callable-arity"`, `"callable-param-list"`, `"callable-syntax"`, `"int-overflow"`, `"non-integer-literal"`, `"non-name-base"`, `"unsupported-expression"`) is now a concrete design question for the future builder-migration slice that resolves [issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432).

### R3 — Class-context annotation-shape errors will fire twice per site due to the dual `collect_class_type` invocation

`resolve_annotation_expr` is called from `collect_class_type` ([classes.rs:266, 273, 300, 307, 377, 403, 455, 469](../crates/sifr_hir/src/lower/classes.rs:266) — at least eight call sites for class/enum/protocol field, init param, init return, method param, method return). [mod.rs:584](../crates/sifr_hir/src/lower/mod.rs:584) and [mod.rs:594](../crates/sifr_hir/src/lower/mod.rs:594) both invoke `collect_class_type` for each class (the second pass refreshes class shapes after type-alias resolution). `resolve_annotation_expr` has no idempotency check and `LowerCtx::error_with_code` does no deduplication — so a malformed `dict[int]` annotation on a class field, init param, or method param will push two identical `LoweringError` records onto `ctx.errors`. The same is true of every other migrated site when reached from a class context.

The downstream impact is partly muted — `apply_diagnostic_recovery_limits` at [sifr_driver/src/diagnostics.rs:179-202](../crates/sifr_driver/src/diagnostics.rs:179) caps each `(severity, code, message, file)` group at `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` but does not collapse exact duplicates inside a group — so a user can still see the same `SIFR-TYPE-0007: dict type annotation requires [K, V] syntax` message twice for a class-context annotation. The new fixtures are all top-level `def` shapes, so they don't trip this; the duplication is still a real user-facing bloat for class fixtures. Pre-existing and identical in shape to slice 2b.7's R1 — carried forward, not introduced here, and best deferred to the dedupe-introduction slice that resolves the registry's `dedupe_args` metadata into runtime suppression.

### R4 — TypeVar bound/constraint shape errors arguably belong to `SIFR-TYPE-0007` too

[mod.rs:271](../crates/sifr_hir/src/lower/mod.rs:271), [277](../crates/sifr_hir/src/lower/mod.rs:277), [299](../crates/sifr_hir/src/lower/mod.rs:299), [313](../crates/sifr_hir/src/lower/mod.rs:313), [325](../crates/sifr_hir/src/lower/mod.rs:325), [335](../crates/sifr_hir/src/lower/mod.rs:335), [347](../crates/sifr_hir/src/lower/mod.rs:347) emit "TypeVar constraints must be simple type names", "TypeVar bound must be a type name or tuple of type names", "TypeVar positional constraints must be simple type names", "TypeVar bound must be a simple type name", and similar variants. Conceptually these are "annotation-shape" errors in the broader sense — the user wrote a TypeVar argument whose *expression shape* is malformed. They live outside `resolve_annotation_expr` (TypeVars are parsed in `parse_typevar_bound_expr` / `parse_typevar_declaration_specs`), so the slice's scope statement of "inside `resolve_annotation_expr`" correctly excludes them. Worth surfacing as a candidate for a near-future slice that completes the `TYPE-0007` family rather than leaving these on the bridge indefinitely.

### R5 — No registry-level test guards `SIFR-TYPE-0007`'s `representative_fixture_path` against fixture rename or substring drift

[codes.rs:1465](../crates/sifr_diagnostics/src/codes.rs:1465) only asserts that `representative_fixture_path` is `Some(...)` for active codes — not that the path exists, not that the fixture contains a matching `expect-error` line. If `invalid_type_annotation.sifr` were renamed or its `expect-error` re-keyed in a later slice (and there are now ten siblings with similar names that could be confused for it), the registry would silently desync. Same risk pattern flagged across slices 2b.5 through 2b.7 — out of scope for this slice; ideally absorbed by `scripts/check_diagnostic_code_coverage.py` planned in `milestone_diag_11` per [issue:1236](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1236).

### R6 — `report_signature` matches the prior slice — confirm this is a deterministic test-set hash, not a stale cache

The implementer reports `report_signature=e1bf653aaa770517` for `scripts/run_all_tests.sh --profile quick`, identical to the value reported for slice 2b.7. If the signature is a content hash of the *test set* (paths plus expectations), then matching across slices is expected because the script's coverage list itself didn't change between 2b.7 and 2b.8. If instead it incorporates a hash of *test results* or *invocation timing*, identical signatures across two distinct runs deserve a sanity check. The wall-time delta (`71.17s` for 2b.7 vs. `85.60s` for 2b.8) suggests a fresh run was executed, which weighs in favor of the deterministic-set-hash interpretation. Non-blocking — calling out only because the matching value across two slices invites a misread.

### R7 — Fixture style intentionally minimal; no negative test for "well-formed annotation, code does NOT fire"

Each new fixture exercises exactly one shape-violation site. There is no companion fixture that proves a *correctly shaped* version of the same annotation (e.g., `dict[int, str]`, `Result[int, ValueError]`, `Callable[[int], str]`) emits no `SIFR-TYPE-0007` and no other error — i.e., the `expect-pass` direction. Because well-shaped annotations are essentially the universal happy path of every Sifr program, the existing pass suite implicitly covers this — but a regression that, say, made the `dict` arm always fire `TYPE-0007` regardless of slice arity would only be caught by the pass suite, not by anything fixture-local. Non-blocking; consistent with the established style of prior slices.

## Verdict

Satisfied / no blocking findings. Slice 2b.8 closes 11 in-scope HIR shape-error call sites inside `resolve_annotation_expr` onto active `SIFR-TYPE-0007` via a small private helper, with ten 1:1 fixtures pinning both the new code and a verbatim substring of the unchanged emitted text, and correctly leaves the unknown-type-name, Result error-type validity, generic alias arity, generic class shape, generic class arity, and unknown generic type paths untouched on the bridge per the scope statement. The registry's existing `representative_fixture_path` for `SIFR-TYPE-0007` aligns with the new fixture set without requiring registry edits. Residual risks are either non-blocking coverage gaps (R1 site #1 fixture, R7 negative-test absence), structural across the milestone (R2 template/runtime drift, R3 class-context double-firing, R5 registry-fixture binding), adjacent-scope candidates (R4 TypeVar bound shape errors), or sanity-check observations (R6 matching signature) — all are correctly deferred to follow-up slices. The local validation set the implementer reports is the established gate, and the diff is exactly the surface area the scope statement promises.
