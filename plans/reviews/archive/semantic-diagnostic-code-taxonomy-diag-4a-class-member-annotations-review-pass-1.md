## `milestone_diag_4a` slice 2b.11 — enum/protocol/newtype method missing param annotations migration to active `SIFR-TYPE-0004` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-class-member-annotations`.
- Target: close the gap surfaced as R2 in [the slice 2b.7 review](semantic-diagnostic-code-taxonomy-diag-4a-missing-annotations-review-pass-1.md) — namely that enum, protocol, and newtype method parameters that lacked an annotation were silently defaulting to `Type::Any` with no diagnostic. After this slice each such site emits active `SIFR-TYPE-0004` via `LowerCtx::error_with_code` ([sifr_hir/src/lower/mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228)) with a fixture pinning both the code and a verbatim substring of the rendered text.
- Files changed (per `git status`):
  - [crates/sifr_hir/src/lower/classes.rs](../crates/sifr_hir/src/lower/classes.rs:53) — new private helper plus four migrated/new emission sites.
  - [crates/sifr/tests/e2e/fail/enum_method_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/enum_method_missing_type_annotation.sifr:1) — enum method fixture.
  - [crates/sifr/tests/e2e/fail/protocol_method_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/protocol_method_missing_type_annotation.sifr:1) — protocol method fixture.
  - [crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr](../crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr:1) — newtype method fixture.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:45) — slice 2b.10 flipped to merged with PR #1682; slice 2b.11 line added.
- Validation rerun by reviewer: `cargo test -p sifr --test e2e -- test_e2e_fail` (1 passed), `cargo fmt --check` (clean), `python3 scripts/check_hir_maintainability_guardrails.py` (PASS), `cargo test -p sifr_hir diagnostic_transport_tests` (2 passed), `cargo clippy --workspace -- -D warnings` (clean).

## Findings

### F1 — Taxonomy choice: `SIFR-TYPE-0004` is correct for all three new sites

`SIFR-TYPE-0004` ("A required type annotation is missing.", [codes.rs:31](../crates/sifr_diagnostics/src/codes.rs:31)) carries the family-level meaning "a *required* annotation was *not provided*". Each new emission is exactly that semantic: `param.parameter.annotation` was `None` for an enum/protocol/newtype method parameter where Sifr's surface-language contract requires an explicit annotation. This is identical in shape to the existing class-method `__init__`/regular emissions migrated in slice 2b.7 ([classes.rs:431](../crates/sifr_hir/src/lower/classes.rs:431), [classes.rs:483](../crates/sifr_hir/src/lower/classes.rs:483)), and the `Type::Any` recovery payload mirrors them. None of the new sites is doing annotation-shape parsing (which would be `TYPE_INVALID_ANNOTATION = SIFR-TYPE-0007`) or type-name resolution (`NAME_UNKNOWN_TYPE = SIFR-NAME-0003`), so `TYPE-0004` is the right code rather than any sibling type-system code.

### F2 — Helper extraction is appropriate and faithfully replaces the existing two emission sites

The new `missing_method_param_annotation` helper at [classes.rs:53-65](../crates/sifr_hir/src/lower/classes.rs:53) factors the format string `"parameter '{param_name}' in {class_name}.{method_name} is missing a type annotation"` and the `error_with_code(TYPE_MISSING_ANNOTATION, ...)` call into a single function with a `&mut LowerCtx`, three `&str` arguments, and no return value.

Replacing the two pre-existing inline emissions (the `__init__` site formerly at [classes.rs:405](../crates/sifr_hir/src/lower/classes.rs:405) and the regular-method site formerly at [classes.rs:457](../crates/sifr_hir/src/lower/classes.rs:457)) with helper calls produces byte-identical rendered text — verified by the prior fixture's `expect-error` substrings continuing to match (`cargo test -p sifr --test e2e -- test_e2e_fail` still passes, including the existing `class_init_missing_type_annotation.sifr` and `class_method_missing_type_annotation.sifr` fixtures from slice 2b.7). The helper takes `&str` rather than `String`, so the `__init__` literal call site at [classes.rs:434](../crates/sifr_hir/src/lower/classes.rs:434) can pass `"__init__"` directly without an allocation, while the other call sites pass `&class_name`/`&method_name`/`&param_name` references to existing owned `String`s — no additional clones introduced. The helper signature is the minimum surface area the four call sites need; nothing speculative.

### F3 — All three out-of-scope-from-2b.7 sites are now covered, with 1:1 fixture coverage

The slice 2b.7 R2 finding listed four pre-existing silent-fallback paths. This slice handles three of them and intentionally skips the fourth (see F4):

| Site | Code path | Fixture | Fixture's `expect-error` substring |
|---|---|---|---|
| Enum-method type collection | [classes.rs:282-287](../crates/sifr_hir/src/lower/classes.rs:282) | [enum_method_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/enum_method_missing_type_annotation.sifr:1) | `parameter 'other' in Direction.same is missing a type annotation` |
| Protocol-method type collection | [classes.rs:322-327](../crates/sifr_hir/src/lower/classes.rs:322) | [protocol_method_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/protocol_method_missing_type_annotation.sifr:1) | `parameter 'value' in Sink.accept is missing a type annotation` |
| Newtype-method body lowering | [classes.rs:765-770](../crates/sifr_hir/src/lower/classes.rs:765) | [newtype_method_missing_type_annotation.sifr:1](../crates/sifr/tests/e2e/fail/newtype_method_missing_type_annotation.sifr:1) | `parameter 'amount' in UserId.add is missing a type annotation` |

Each fixture's substring is a verbatim slice of the helper's rendered text. The e2e harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) requires the diagnostic code AND the optional substring to match a single emitted diagnostic; I confirmed each fixture's `expect-error` line satisfies both halves by running `cargo run -p sifr -- check` against each fixture and observing the rendered output:

- `enum_method_missing_type_annotation.sifr` → emits `parameter 'other' in Direction.same is missing a type annotation` (twice, see R1).
- `protocol_method_missing_type_annotation.sifr` → emits `parameter 'value' in Sink.accept is missing a type annotation` (twice, see R1).
- `newtype_method_missing_type_annotation.sifr` → emits `parameter 'amount' in UserId.add is missing a type annotation` (once, see R1).

### F4 — The skipped fourth R2 site (enum-method body lowering) is correctly *not* migrated

The 2b.7 R2 finding listed enum-method body lowering at the time-of-2b.7 [classes.rs:658-662](../crates/sifr_hir/src/lower/classes.rs:658) (now at [classes.rs:684-688](../crates/sifr_hir/src/lower/classes.rs:684)) as a silent-fallback site. The implementer correctly chose *not* to emit there because:

1. Enum methods *do* route through `collect_class_type`'s newly-migrated branch at [classes.rs:282-287](../crates/sifr_hir/src/lower/classes.rs:282) — the same R2 line in 2b.7 was wrong about that path being silent, because the enum branch's loop at [classes.rs:270-303](../crates/sifr_hir/src/lower/classes.rs:270) was the silent path. After the migration, enum signatures *are* checked at type-collection time and the diagnostic *does* fire there.
2. `lower_class`'s enum branch is reached *after* `collect_class_type` has already pushed the diagnostic. Emitting again from `lower_class` would produce a third copy on every run (in addition to the dual-pass duplication described in R1) — strictly worse than the existing pattern for regular classes ([classes.rs:894-909](../crates/sifr_hir/src/lower/classes.rs:894), which is also silent for the same dedupe reason). The implementer's stated rationale ("enum signatures are already checked in collect_class_type") matches this F5 finding from the slice 2b.7 review verbatim.

So the protocol-method body-lowering pass at [classes.rs:622-650](../crates/sifr_hir/src/lower/classes.rs:622) — which doesn't iterate parameters at all because it reuses the already-collected `methods_sigs` — is simultaneously correct (no second emission point exists). Newtype-method body lowering is the lone exception (F5 below) because newtype's `collect_class_type` path returns early before iterating methods.

### F5 — Newtype emission must be at body-lowering time because `collect_class_type` returns early for newtypes

`collect_class_type` returns at [classes.rs:234](../crates/sifr_hir/src/lower/classes.rs:234) for newtypes after registering the wrapper type and constructor, *before* iterating the class body for method signatures. So newtype methods have no type-collection pass at all — `lower_class` is the only place where their parameter annotations are inspected. Emitting from [classes.rs:765-770](../crates/sifr_hir/src/lower/classes.rs:765) is therefore the correct placement; emitting from `collect_class_type` would require restructuring the early return, which would expand the diff well beyond the minimal "close the gap" scope.

A side effect is that newtype methods emit the `TYPE-0004` diagnostic *once*, while enum/protocol/regular-class methods emit *twice* due to the dual-pass `collect_class_type` invocation (R1). This is asymmetric, but it is downstream of the architectural choice that newtypes get a single-pass collection. Acceptable given dedup is explicitly out of scope per the slice scope statement.

### F6 — Out-of-scope sites are correctly *not* migrated

The slice scope statement explicitly defers:

- **Return annotations.** The newtype path at [classes.rs:782-786](../crates/sifr_hir/src/lower/classes.rs:782), enum path at [classes.rs:292-296](../crates/sifr_hir/src/lower/classes.rs:292) and [classes.rs:699-703](../crates/sifr_hir/src/lower/classes.rs:699), protocol path at [classes.rs:332-336](../crates/sifr_hir/src/lower/classes.rs:332), and regular-class path at [classes.rs:911-917](../crates/sifr_hir/src/lower/classes.rs:911) all default a missing return annotation to `Type::None` without diagnostic. None of those sites is touched in this branch's diff.
- **Normal class behavior already covered by slice 2b.7.** The two pre-existing emission sites at [classes.rs:431-436](../crates/sifr_hir/src/lower/classes.rs:431) (`__init__`) and [classes.rs:483-488](../crates/sifr_hir/src/lower/classes.rs:483) (regular method) are migrated to use the new helper — code change only, no message-text or code-emission change.
- **Dual-pass deduplication.** The duplication described in R1 below is acknowledged and deferred.
- **Bridge deletion.** `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is intact.
- **Registry/docs generation.** No changes under `crates/sifr_diagnostics/`.

I confirmed each by `git diff --stat` showing only the four expected files and a repo-wide `grep` for the existing emission strings showing no other call sites adopted or migrated.

### F7 — Issue checklist transitions are clean

[issue:45](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:45) flips slice 2b.10 from "implementation complete and reviewer-satisfied" to `[x] merged ... PR: ...pull/1682`, matching the merged PR identifier from the prior review's verdict. [issue:46](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:46) adds slice 2b.11 with "Started" status. The wording mirrors the established cadence used for slices 2b.6 through 2b.10. No unrelated checklist drive-bys.

### F8 — Diff is tightly scoped

`git status` shows exactly two modified files (the HIR source and the issue) plus three untracked fixtures — nothing else. A repo-wide `grep` for `expect-error.*missing a type annotation` returns the six pre-existing fixtures from slice 2b.7 plus the three new ones, and no orphaned bridge-keyed fixture survives. No baselines, no `verification/`, no docs, no schema changes — exactly emission migration plus fixtures plus the checklist edit.

## Residual risks

### R1 — Enum and protocol diagnostics fire twice per param (pre-existing dual-pass behavior, now extended)

Same dual-pass duplication pattern that R1 of the slice 2b.7 review flagged: [mod.rs:595](../crates/sifr_hir/src/lower/mod.rs:595) and [mod.rs:605](../crates/sifr_hir/src/lower/mod.rs:605) both invoke `collect_class_type` per class to refresh class shapes after type-alias resolution, and `error_with_code` does no idempotency check ([mod.rs:228-235](../crates/sifr_hir/src/lower/mod.rs:228)). Each new emission at [classes.rs:282](../crates/sifr_hir/src/lower/classes.rs:282) (enum) and [classes.rs:322](../crates/sifr_hir/src/lower/classes.rs:322) (protocol) therefore pushes a second identical `LoweringError` on the second pass. Empirically confirmed by `cargo run -p sifr -- check`:

- `enum_method_missing_type_annotation.sifr` → 2× identical `parameter 'other' in Direction.same is missing a type annotation`.
- `protocol_method_missing_type_annotation.sifr` → 2× identical `parameter 'value' in Sink.accept is missing a type annotation`.
- `newtype_method_missing_type_annotation.sifr` → 1× `parameter 'amount' in UserId.add is missing a type annotation` (newtype only emits at body-lowering time per F5).

The downstream impact is partly muted because `apply_diagnostic_recovery_limits` at [sifr_driver/src/diagnostics.rs:179](../crates/sifr_driver/src/diagnostics.rs:179) groups by `(severity, code, message, file)` and retains up to `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` per group, but it does not collapse exact duplicates within a group — so a user can still see the identical `SIFR-TYPE-0004` message twice. The e2e harness uses `errors.iter().any(...)` at [e2e.rs:2561](../crates/sifr/tests/e2e.rs:2561), so duplicates do not fail the new fixtures.

Behavior is consistent with what slice 2b.7 already shipped for normal class `__init__` and regular methods (also 2×). Out of scope for this slice; would naturally fall to the future builder-migration / dedupe-introduction slice that resolves the registry's `dedupe_args` metadata into runtime suppression.

### R2 — Newtype fixture's call-site triggers an unrelated pre-existing dispatch error

`newtype_method_missing_type_annotation.sifr:9` calls `user_id.add(2)`. Running `cargo run -p sifr -- check` on it emits two errors:

```
type error: [main] type 'int' has no method 'add'
type error: [main] parameter 'amount' in UserId.add is missing a type annotation
```

The first error is a pre-existing newtype method-dispatch bug — the fixture's call to `user_id.add(2)` resolves the receiver as the inner `int` rather than the `UserId` wrapper, so the `add` method isn't found. The fixture still passes the e2e contract because `errors.iter().any(...)` at [e2e.rs:2561](../crates/sifr/tests/e2e.rs:2561) finds at least one `SIFR-TYPE-0004` match, but the secondary noise dilutes the fixture's signal. The minimal repro for the missing-annotation diagnostic does not require calling the method — the fixtures for enum and protocol both call (or skip) the method without triggering an unrelated diagnostic, and the simplest fix is to drop the `user_id.add(2)` call from `main()` (or replace it with `print(user_id)`). Non-blocking — the fixture *does* exercise the migrated code path correctly — but addressing it would tighten the fixture's signal and surface the orthogonal newtype dispatch bug for separate triage.

### R3 — Enum, protocol, and newtype `__init__` parameters remain silent on missing annotations

For each special class shape, `__init__` is intentionally skipped:

- Enum: `collect_class_type` body iteration skips `__init__` at [classes.rs:273-275](../crates/sifr_hir/src/lower/classes.rs:273); enum's `lower_class` doesn't iterate `__init__` either.
- Protocol: `collect_class_type` skips at [classes.rs:313-315](../crates/sifr_hir/src/lower/classes.rs:313); protocol `lower_class` reuses already-collected sigs.
- Newtype: `lower_class` skips at [classes.rs:753-755](../crates/sifr_hir/src/lower/classes.rs:753); `collect_class_type` returns early before reaching method iteration.

So an enum/protocol/newtype written with an unannotated `__init__` parameter compiles silently and produces no diagnostic. In each case the `__init__` is functionally inert (enum/newtype constructors are auto-generated; protocols have no runtime), so this is a low-priority gap rather than a soundness hole. The slice's scope statement explicitly says "method parameters", which can be read as either including or excluding `__init__`. The implementer's interpretation (exclude, because `__init__` is a no-op for these shapes) is defensible and I would not block on it. Worth surfacing for the bridge-deletion slice's scope review, since the bridge can technically still be deleted — these `__init__`s never produced any diagnostic, legacy-coded or otherwise.

### R4 — Vararg / keyword-only parameters of enum/protocol/newtype methods remain uncovered

The new emission sites all iterate `func.parameters.args.iter().skip(1)` (or `.skip(skip_count)` for the regular-method path). None inspects `func.parameters.vararg` or `func.parameters.kwonlyargs`. So an enum/protocol/newtype method written like `def m(self, *args, key)` would have its `args` and `key` parameters silently default to `Type::Any` with no diagnostic.

This matches the pre-existing class-method behavior in slice 2b.7 (which also covers only `args`, per [the slice 2b.7 R3-equivalent finding F6](semantic-diagnostic-code-taxonomy-diag-4a-missing-annotations-review-pass-1.md)). Top-level functions cover all three param shapes via [typing_and_functions.rs:330-366](../crates/sifr_hir/src/lower/typing_and_functions.rs:330), but class members do not. Pre-existing structural gap, not introduced by this slice. Out of scope.

### R5 — No registry-level binding / no negative test for "annotation present, code does NOT fire"

[codes.rs:597-602](../crates/sifr_diagnostics/src/codes.rs:597) (slice 2b.7's settling) hardcodes `missing_type_annotation.sifr` as the representative fixture for `SIFR-TYPE-0004`. The three new fixtures are not registered as additional representative fixtures — appropriate, because the registry currently supports only one representative fixture per code. The registry tests at [codes.rs:1316](../crates/sifr_diagnostics/src/codes.rs:1316) only confirm that the representative is `Some(...)`, not that any specific shape is exercised. So the three new fixtures are protected only by the `test_e2e_fail` harness — a future rename or substring re-key would silently desync from any registry-level expectation, but no such expectation currently exists. Same risk pattern as slice 2b.7 R5; deferred to `milestone_diag_11`. Each new fixture exercises exactly one missing-annotation site and asserts exactly one `expect-error`; there is no companion fixture for the annotated-and-passes path. Non-blocking.

### R6 — Helper is private and currently has only intra-module reuse

`missing_method_param_annotation` is `fn` (not `pub(super)` or `pub(crate)`), so it cannot be reused by the analogous emissions in [typing_and_functions.rs:316](../crates/sifr_hir/src/lower/typing_and_functions.rs:316) / [336](../crates/sifr_hir/src/lower/typing_and_functions.rs:336) / [356](../crates/sifr_hir/src/lower/typing_and_functions.rs:356) (top-level positional, vararg, kwonly) or [nested_function_inference.rs:440](../crates/sifr_hir/src/lower/nested_function_inference.rs:440) (nested). Those use distinct format strings (`"in function '{name}'"` rather than `"in {class}.{method}"`), so direct reuse would not work without additional parameterization — the helper's narrow signature is appropriate for its current four call sites. If a future slice consolidates the message templates (per R4 of the slice 2b.7 review, which flagged the registry template / runtime drift), the helper would need to be refactored or moved; that's a milestone-scoped concern, not a blocker here.

## Verdict

**Satisfied / no blocking findings.** Slice 2b.11 closes the three in-scope silent-fallback paths (enum-method type collection, protocol-method type collection, newtype-method body lowering) onto active `SIFR-TYPE-0004` with one fixture per site (each pinning both the new code and a verbatim substring of the rendered text), correctly leaves the dual-pass deduplication, return annotations, vararg/kwonly class params, special-class `__init__`, and bridge-deletion paths untouched, and consolidates the existing two slice-2b.7 emission sites onto a shared private helper without changing rendered text. Residual risks are either pre-existing structural (R1 dual-pass duplication, R3 special-class `__init__` silence, R4 vararg/kwonly class-member gap), milestone-scoped (R5 registry-fixture binding, R6 helper visibility), or fixture-quality (R2 newtype dispatch noise) — all are correctly deferred. The local validation set the implementer reports (`report_signature=e1bf653aaa770517`, matching the signature pinned across slices 2b.3-2b.10) is the established gate, and my own re-run of the in-tree validators (e2e fail suite, fmt, HIR guardrails, diagnostic transport tests, clippy) all pass on this branch.
