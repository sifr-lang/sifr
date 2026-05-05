## `milestone_diag_4a` slice 2b.9 — HIR container literal conflict migration to active `SIFR-TYPE-0008` — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-container-conflicts`.
- Target: migrate the four HIR call sites that today emit a free-form "list/set element / dict key / dict value type mismatch" string through the legacy `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge ([sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137)) onto active `SIFR-TYPE-0008` via `LowerCtx::error_with_code` ([sifr_hir/src/lower/mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228)), and add representative e2e fixtures pinning the new code+substring contract.
- Files changed:
  - [crates/sifr_hir/src/lower/expressions.rs](../crates/sifr_hir/src/lower/expressions.rs:1979) — introduces a private `container_literal_type_conflict` helper (lines 1979–1993) and re-routes four conflict-emission sites: list element ([expressions.rs:2004](../crates/sifr_hir/src/lower/expressions.rs:2004)), set element ([expressions.rs:2030](../crates/sifr_hir/src/lower/expressions.rs:2030)), dict key ([expressions.rs:2059](../crates/sifr_hir/src/lower/expressions.rs:2059)), dict value ([expressions.rs:2074](../crates/sifr_hir/src/lower/expressions.rs:2074)).
  - [crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr](../crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr:1) — list element conflict (also the registry's representative fixture for `SIFR-TYPE-0008` at [codes.rs:641](../crates/sifr_diagnostics/src/codes.rs:641)).
  - [crates/sifr/tests/e2e/fail/container_set_literal_type_conflict.sifr](../crates/sifr/tests/e2e/fail/container_set_literal_type_conflict.sifr:1) — set element conflict.
  - [crates/sifr/tests/e2e/fail/container_dict_key_type_conflict.sifr](../crates/sifr/tests/e2e/fail/container_dict_key_type_conflict.sifr:1) — dict key conflict.
  - [crates/sifr/tests/e2e/fail/container_dict_value_type_conflict.sifr](../crates/sifr/tests/e2e/fail/container_dict_value_type_conflict.sifr:1) — dict value conflict.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) — slice 2b.8 flipped to merged with PR #1680; slice 2b.9 added with "Started" status.
- Validation already executed by the implementer: `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=71.04s`).
- Reviewer-side reproduction: `cargo test -p sifr --test e2e -- test_e2e_fail` (1 passed), `cargo test -p sifr_hir diagnostic_transport_tests` (2 passed), `cargo fmt --check` (clean), `python3 scripts/check_hir_maintainability_guardrails.py` ("HIR maintainability guardrails: PASS"), `cargo clippy --workspace -- -D warnings` (clean).

## Findings

### F1 — Taxonomy choice: `SIFR-TYPE-0008` is the correct active code for all four sites

`SIFR-TYPE-0008` ("Container literal elements, keys, or values have conflicting types.", [codes.rs:636-646](../crates/sifr_diagnostics/src/codes.rs:636)) is the family-level meaning of *exactly* the four migrated emissions. Each site fires when a homogeneous-container literal (`[…]`, `{…}` for sets, `{k: v, …}`) declares two members whose static types are mutually non-assignable:

- list element: [expressions.rs:2003](../crates/sifr_hir/src/lower/expressions.rs:2003) — `!ty.is_assignable_to(expected)` inside `lower_list_literal`.
- set element: [expressions.rs:2029](../crates/sifr_hir/src/lower/expressions.rs:2029) — same predicate inside `lower_set_literal`.
- dict key: [expressions.rs:2058](../crates/sifr_hir/src/lower/expressions.rs:2058) — same predicate inside `lower_dict_literal`'s key arm.
- dict value: [expressions.rs:2073](../crates/sifr_hir/src/lower/expressions.rs:2073) — same predicate inside `lower_dict_literal`'s value arm.

The split against adjacent diagnostics is clean:

- "container method receiver/argument has wrong element type" (e.g., `list.extend()` iterable element-type mismatch at [method_call_args.rs:482](../crates/sifr_hir/src/lower/method_call_args.rs:482), `dict.update()` at [method_call_args.rs:509](../crates/sifr_hir/src/lower/method_call_args.rs:509), `set.update()` at [method_call_args.rs:541](../crates/sifr_hir/src/lower/method_call_args.rs:541)) — different semantic ("call argument is wrong shape", `SIFR-CALL-*` / `SIFR-TYPE-*`), correctly out of scope.
- "subscript assignment has wrong key/value/index type" ([container_literal_specialization.rs:49,55,95-99,101-106,113,135,141,147-151,166,181](../crates/sifr_hir/src/lower/container_literal_specialization.rs:49)) — semantic is "assignment-target compatibility against an established container type", separate from "the literal itself has internally conflicting elements", correctly left on the bridge.
- "empty literal previously specialized; later subscript writes incompatible types" ([container_literal_specialization.rs:30](../crates/sifr_hir/src/lower/container_literal_specialization.rs:30)) — also assignment-shape, not literal-shape, and the user explicitly excluded "empty collection specialization" from this slice.
- "tuple unpacking shape" → `SIFR-TYPE-0009` (slice 2b.5/2b.6) — different family-level meaning ("count mismatch", not "element-type conflict").

Each migrated site retains the prior recovery behavior — the elem/key/value type lock to whatever was set first and the literal continues to lower with the locked type as its specialization. The diagnostic identity is the only behavior change.

### F2 — Centralized helper `container_literal_type_conflict` is a clean, scope-correct DRY pattern

[expressions.rs:1979-1993](../crates/sifr_hir/src/lower/expressions.rs:1979):

```rust
fn container_literal_type_conflict(
    ctx: &mut LowerCtx,
    element_kind: &str,
    expected: &Type,
    actual: &Type,
) {
    ctx.error_with_code(
        DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
        format!(
            "container literal has conflicting {element_kind} types: {} and {}",
            expected.display_name(),
            actual.display_name()
        ),
    );
}
```

The signature is right-sized for the four call sites: `element_kind: &str` because every caller passes a `&'static str` (`"list element"`, `"set element"`, `"dict key"`, `"dict value"`); `expected` and `actual` taken by `&Type` because the callers already have `&Type` references in hand (one borrowed from `Option<Type>`, one borrowed from a freshly cloned `expr.ty()`). Visibility is `fn` (private to the file), matching the slice 2b.8 helper convention ([typing_and_functions.rs:380-382](../crates/sifr_hir/src/lower/typing_and_functions.rs:380)). The helper sits adjacent to the three literal-lowering functions that consume it ([expressions.rs:1995](../crates/sifr_hir/src/lower/expressions.rs:1995), [2021](../crates/sifr_hir/src/lower/expressions.rs:2021), [2047](../crates/sifr_hir/src/lower/expressions.rs:2047)), giving a single grep target for any future retargeting.

### F3 — Message template alignment is exact

Registry template at [codes.rs:642](../crates/sifr_diagnostics/src/codes.rs:642):

```
container literal has conflicting {element_kind} types: {expected} and {actual}
```

Helper format string at [expressions.rs:1988](../crates/sifr_hir/src/lower/expressions.rs:1988):

```
container literal has conflicting {element_kind} types: {} and {}
```

The placeholders `{}` substitute `expected.display_name()` and `actual.display_name()`, both `String` renderings whose values are the type's Sifr surface name ([types.rs:500-545](../crates/sifr_type_system/src/display.rs:500); e.g., `Type::Int → "int"`, `Type::Str → "str"`, `Type::List(Box::new(Type::Int)) → "list[int]"`). The four `element_kind` strings the call sites pass match the spelling that the registry's representative fixture and three siblings encode in their `# expect-error` lines. The only template-vs-template delta against the legacy bridge text is intentional and registry-aligned: the legacy text was per-shape (`"list element type mismatch: expected '<T>', got '<U>'"`), the new text is template-driven (`"container literal has conflicting list element types: <T> and <U>"`); the new text drops the single-quote wrapping around type names so it matches the registry verbatim. No other diagnostic in the active registry quotes type names, so this brings TYPE-0008 in line with the family-wide style.

### F4 — All four in-scope sites are migrated, with 1:1 fixture coverage

Stated scope is "list element conflicts, set element conflicts, dict key conflicts, dict value conflicts inside `lower_list_literal` / `lower_set_literal` / `lower_dict_literal`". The four call sites map onto the four fixtures one-to-one:

| # | Site | element_kind | Fixture | `expect-error` substring |
|---|---|---|---|---|
| 1 | [expressions.rs:2004](../crates/sifr_hir/src/lower/expressions.rs:2004) | `list element` | [container_literal_type_conflict.sifr:1](../crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr:1) (registry rep, [codes.rs:641](../crates/sifr_diagnostics/src/codes.rs:641)) | `container literal has conflicting list element types: int and str` |
| 2 | [expressions.rs:2030](../crates/sifr_hir/src/lower/expressions.rs:2030) | `set element` | [container_set_literal_type_conflict.sifr:1](../crates/sifr/tests/e2e/fail/container_set_literal_type_conflict.sifr:1) | `container literal has conflicting set element types: int and str` |
| 3 | [expressions.rs:2059](../crates/sifr_hir/src/lower/expressions.rs:2059) | `dict key` | [container_dict_key_type_conflict.sifr:1](../crates/sifr/tests/e2e/fail/container_dict_key_type_conflict.sifr:1) | `container literal has conflicting dict key types: int and str` |
| 4 | [expressions.rs:2074](../crates/sifr_hir/src/lower/expressions.rs:2074) | `dict value` | [container_dict_value_type_conflict.sifr:1](../crates/sifr/tests/e2e/fail/container_dict_value_type_conflict.sifr:1) | `container literal has conflicting dict value types: int and str` |

Each fixture is a minimal valid Sifr program (`def main(): values = <bad literal>; print(len(values))`), the bad literal triggers exactly one of the four sites, and the `expect-error` line pins both the code (`SIFR-TYPE-0008`) and a verbatim slice of the emitted text. The e2e harness contract at [e2e.rs:2561-2566](../crates/sifr/tests/e2e.rs:2561) (`failure.code == expected.code && failure.message.contains(message)`) is satisfied for each: I traced the matchers by reproducing `cargo test -p sifr --test e2e -- test_e2e_fail` (passes).

For the dict fixtures, the constructed inputs are surgical:

- `{1: "one", "two": "two"}`: first iteration sets `key_ty=int` and `val_ty=str`; second iteration's key conflicts (`str` vs. `int`), value matches (`str` vs. `str`). Exactly one `dict key` emission, no `dict value` emission. Fixture's single `expect-error` line is satisfied without depending on the unrelated value path.
- `{"one": 1, "two": "two"}`: first iteration sets `key_ty=str` and `val_ty=int`; second iteration's key matches (`str`), value conflicts (`str` vs. `int`). Exactly one `dict value` emission, no `dict key` emission. Same property.

This means the dict-key and dict-value fixtures are independent regression sentinels — a regression that selectively broke one path but not the other would be caught by exactly the right fixture rather than both.

### F5 — Out-of-scope sites are correctly *not* migrated

The slice statement explicitly defers "other container method diagnostics, unhashable keys, empty collection specialization, bridge deletion, registry/docs generation". Every adjacent emission that conceptually borders TYPE-0008's territory remains on the bridge:

- `emit_empty_literal_type_conflict` ([container_literal_specialization.rs:30](../crates/sifr_hir/src/lower/container_literal_specialization.rs:30)): empty-literal post-specialization check — out of scope, retains `ctx.error(...)`.
- `validate_subscript_assignment_target` and `validate_subscript_augassign_target` ([container_literal_specialization.rs:39-188](../crates/sifr_hir/src/lower/container_literal_specialization.rs:39)): subscript assignment/augassign — out of scope, retains `ctx.error(...)`.
- `lower_tuple_literal` ([expressions.rs:2093](../crates/sifr_hir/src/lower/expressions.rs:2093)): tuples are heterogeneous and have no element-uniformity check; correctly has no migrated emission.
- Method-arg element-type emissions in `method_call_args.rs` (list.extend / dict.update / set.update / set.intersection_update etc.): different semantic ("argument shape mismatch"), correctly out of scope.

The legacy `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge at [sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is correctly left untouched; bridge deletion is the explicit out-of-scope item per [issue:43](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43), and is required for the unmigrated emissions above to surface anything at all.

### F6 — Issue checklist transitions are clean

[issue:42](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:42) flips slice 2b.8 from "Started" / "implementation complete and reviewer-satisfied" to `[x] merged ... PR: ...pull/1680`, matching the merged PR identifier from the prior review's verdict. [issue:43](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:43) adds slice 2b.9 with "Started" status and an accurate scope statement (`HIR container literal element/key/value conflict diagnostics migration to active SIFR-TYPE-0008 with fixture coverage`). The wording mirrors the established cadence used for slices 2b.3 through 2b.8. No unrelated checklist drive-bys.

### F7 — Diff is tightly scoped; no orphan fixtures, no stale callers

`git status` shows exactly two modified files (`expressions.rs` plus the issue) and four untracked fixtures — nothing else. A repo-wide grep for the legacy phrases (`list element type mismatch`, `set element type mismatch`, `dict key type mismatch`, `dict value type mismatch`) returns no live emission and no fixture pinning the legacy text against the legacy bridge code; the only matches are in `issues/archive/`. A grep for the new phrase confirms it only appears in the helper, the registry, and the four new fixtures (plus generated docs at [docs/errors/SIFR-TYPE-0008.md:13](../docs/errors/SIFR-TYPE-0008.md:13) and the family table at [internal_docs/diagnostic_codes.md:84](../internal_docs/diagnostic_codes.md:84)). No baselines, no `verification/`, no schema changes.

### F8 — Registry's `representative_fixture_path` for `SIFR-TYPE-0008` already lined up

[codes.rs:641](../crates/sifr_diagnostics/src/codes.rs:641) hardcodes `"crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr"` as the representative fixture for `SIFR-TYPE-0008`. That file now exists with `# expect-error: SIFR-TYPE-0008: container literal has conflicting list element types: int and str`, satisfying the registry pin without any registry edit. The pre-existing pin from slice 2b.0/2b.1 was sized correctly for this slice's scope.

## Residual risks

### R1 — Inventory line 310 still says "fixture pending in `milestone_diag_7`"

[internal_docs/diagnostic_emission_inventory.md:310](../internal_docs/diagnostic_emission_inventory.md:310) reads `| SIFR-TYPE-0008 | container literal element/key/value type conflict | container literal specialization | fixture pending in milestone_diag_7 |`, but `crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr` now exists and the registry knows about it. Same drift exists for `SIFR-TYPE-0004` (line 306) and `SIFR-TYPE-0007` (line 309), both of which were also migrated in earlier slices without flipping their inventory text. Pre-existing pattern across slices 2b.7 and 2b.8, carried forward — non-blocking, but a one-line edit on line 310 would resync this slice's inventory row with reality. Same recommendation flagged in `reviews/semantic-diagnostic-code-taxonomy-diag-2b-review-pass-2.md` N3 for the 0004/0007/0008 trio when the inventory was previously corrected. Recommended follow-up: update line 310 to `crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr` either in this slice or in a dedicated inventory-hygiene slice.

### R2 — Registry's `owner_module` for `SIFR-TYPE-0008` is `container_literal_specialization`, but the actual emission lives in `expressions.rs`

[codes.rs:643](../crates/sifr_diagnostics/src/codes.rs:643) declares the owner as `"sifr_hir::lower::container_literal_specialization"`, propagated to [docs/errors/SIFR-TYPE-0008.md:12](../docs/errors/SIFR-TYPE-0008.md:12) and [internal_docs/diagnostic_codes.md:84](../internal_docs/diagnostic_codes.md:84). However, the four migrated sites and the new helper all live in `crates/sifr_hir/src/lower/expressions.rs`; the `container_literal_specialization` module focuses on subscript-assignment validation, post-lowering empty-literal patches, and `type_contains_unknown_or_any` ([container_literal_specialization.rs:1-261](../crates/sifr_hir/src/lower/container_literal_specialization.rs:1)) — it has never emitted any of the migrated diagnostics. This is a pre-existing registry-vs-actual-owner mismatch that the slice did not introduce (the registry has carried this owner string since the codes table was first populated in `21cde40c`), but the slice is the natural moment to fix it: either repoint the owner to `sifr_hir::lower::expressions` (or `expressions::container_literal_lowering` if the helper is intended to grow), or move the helper plus the four call sites into `container_literal_specialization.rs` (which would also keep the maintainability-guardrails LOC budget for `expressions.rs` slimmer; current run reports PASS so this is not a forced move). Non-blocking — the owner field is documentation-only and no test asserts on it — but worth correcting now while the helper is the only TYPE-0008 emitter, before it gets cited in downstream tooling.

### R3 — Registry `dedupe_args` are documentation-only at the current emission API

[codes.rs:644-645](../crates/sifr_diagnostics/src/codes.rs:644) declares `declared_args = ["element_kind", "expected", "actual"]` and `dedupe_args = ["element_kind", "expected", "actual"]`. The current `error_with_code` signature ([mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228)) takes only a pre-formatted `String`, so the helper has no path to pass these as structured args; the registry assertions only verify that dedupe args are a subset of declared args ([codes.rs:1591-1606](../crates/sifr_diagnostics/src/codes.rs:1591)) and don't enforce a structured-arg construction. As a result, runtime dedup (if any) hashes by message text rather than by arg tuple, and two literals with the same `(element_kind, expected, actual)` tuple but different surface formatting (none, today) would dedup correctly only by accident. Same structural drift flagged across slices 2b.5–2b.8 (R2/R4 in those reviews); this is a pending input for the structured-arg builder migration that resolves [issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432). Non-blocking for this slice.

### R4 — `is_assignable_to`-driven duplicate emissions for `[1, "a", "b"]`-shape inputs

The list/set loops update `elem_ty` only on the first iteration; subsequent non-matching elements emit but do not refresh the locked element type. So `[1, "a", "b"]` emits two errors with the same `(element_kind="list element", expected=int, actual=str)` triple. `LowerCtx::error_with_code` does no deduplication ([mod.rs:228-235](../crates/sifr_hir/src/lower/mod.rs:228)), and `apply_diagnostic_recovery_limits` ([sifr_driver/src/diagnostics.rs:179-202](../crates/sifr_driver/src/diagnostics.rs:179)) caps each `(severity, code, message, file)` group at `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5` but does not collapse exact duplicates inside the group — so the user sees two identical lines for that input. This is the exact behavior the bridge produced before this slice, so it's a pre-existing issue carried forward, identical in shape to slice 2b.7's R1 / slice 2b.8's R3. Non-blocking, deferred to the dedupe-introduction slice that resolves the registry's `dedupe_args` metadata into runtime suppression.

### R5 — No registry-level test guards `SIFR-TYPE-0008`'s `representative_fixture_path` against fixture rename or substring drift

[codes.rs:1465](../crates/sifr_diagnostics/src/codes.rs:1465) only asserts that `representative_fixture_path` is `Some(...)` for active codes — not that the path exists, not that the fixture contains a matching `expect-error` line. With four sibling fixtures now sharing the `container_*_type_conflict.sifr` naming pattern, a rename of `container_literal_type_conflict.sifr` (e.g., to `container_list_literal_type_conflict.sifr` to match its three siblings' explicit-shape naming) would silently desync the registry. Same risk pattern flagged across slices 2b.5 through 2b.8 — out of scope for this slice; ideally absorbed by `scripts/check_diagnostic_code_coverage.py` planned in `milestone_diag_11` per [issue:1236](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1236).

### R6 — Naming asymmetry: representative fixture is `container_literal_type_conflict.sifr`, siblings are `container_<shape>_literal_type_conflict.sifr` / `container_dict_<axis>_type_conflict.sifr`

The list-element fixture is named `container_literal_type_conflict.sifr` (no shape qualifier) because the registry pin from slice 2b.0/2b.1 chose that name before the slice's other three fixtures existed. The siblings are `container_set_literal_type_conflict.sifr`, `container_dict_key_type_conflict.sifr`, `container_dict_value_type_conflict.sifr`. A reader who searches for the list-element conflict by shape pattern (`container_list_*`) would miss it. Pre-existing constraint from the registry pin — renaming would require a registry edit and was correctly avoided in this slice. Non-blocking; surface only as a follow-up candidate.

### R7 — Fixture style intentionally minimal; no companion negative test

Each new fixture exercises exactly one shape-violation site. There is no companion fixture proving a *correctly typed* version of the same literal (e.g., `[1, 2]`, `{"one", "two"}`, `{1: "one", 2: "two"}`) emits no `SIFR-TYPE-0008`. The pass suite implicitly covers this — a regression that always fired TYPE-0008 regardless of element-type compatibility would catastrophically fail the pass suite — but a regression that only fired on, say, mixed `int`-`str` would only be caught by exactly the four new fixtures. Non-blocking; consistent with the established style of prior slices.

### R8 — `report_signature=e1bf653aaa770517` matches slices 2b.7 and 2b.8

The implementer reports the same `report_signature` value as slices 2b.7 and 2b.8. If the signature is a deterministic content hash of the test set (paths plus expected outcomes), matching across slices is expected because the script's coverage list itself didn't change between 2b.7, 2b.8, and 2b.9 — none of those slices added or removed entries from `scripts/run_all_tests.sh`'s reach. The wall-time delta (`85.60s` for 2b.8 vs. `71.04s` for 2b.9) suggests a fresh run, weighing in favor of the deterministic-set-hash interpretation. Same calling-out as slice 2b.8 R6 — non-blocking, surfaced only because the matching value across three slices invites a misread.

## Verdict

Satisfied / no blocking findings. Slice 2b.9 closes the four in-scope HIR container-literal element-conflict call sites onto active `SIFR-TYPE-0008` via a small private helper that exactly mirrors the registry's message template, with four 1:1 fixtures pinning both the new code and a verbatim substring of the rendered text, and correctly leaves the empty-literal post-specialization, subscript-assignment, container-method-arg, and tuple-literal paths untouched on the bridge per the scope statement. The registry's existing `representative_fixture_path` for `SIFR-TYPE-0008` aligns with the new fixture set without requiring registry edits. Residual risks are either pre-existing inventory/owner drift the slice can optionally fix (R1 inventory line 310, R2 owner-module mismatch), structural across the milestone (R3 template/runtime drift, R4 duplicate emissions, R5 registry-fixture binding), naming-asymmetry follow-ups (R6 list-element fixture naming), or sanity-check observations (R7 negative-test absence, R8 matching signature) — all are correctly deferred to follow-up slices. The local validation set the implementer reports is the established gate, I reproduced the relevant subset (`cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr_hir diagnostic_transport_tests`, `cargo fmt --check`, `python3 scripts/check_hir_maintainability_guardrails.py`, `cargo clippy --workspace -- -D warnings`) green, and the diff is exactly the surface area the scope statement promises.
