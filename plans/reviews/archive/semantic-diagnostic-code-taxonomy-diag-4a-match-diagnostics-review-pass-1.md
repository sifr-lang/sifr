# Review — `milestone_diag_4a` slice 2b.18 (match diagnostics) — pass 1

Branch: `codex/semantic-diagnostics-diag-4a-match-diagnostics`
Scope: migrate match exhaustiveness, guard, and class-pattern-field diagnostics
to active `SIFR-MATCH-0001` / `SIFR-MATCH-0002` / `SIFR-MATCH-0003`, add
`crates/sifr_hir/src/lower/match_diagnostics.rs` helpers + unit tests, re-key
six e2e fail fixtures off `SIFR-TYPE-0001`, and align registry/doc templates
for `SIFR-MATCH-0002` / `SIFR-MATCH-0003`.

Review style: read-only audit, no files modified.

## TL;DR

Slice is correct, narrowly scoped, and consistent with prior structurally-coded
slices (RESULT, OWN, FLOW). Migration is complete for the four raw `ctx.error`
call sites covered by `SIFR-MATCH-0001..0003`. Unit tests pin both code and
message for every helper. Fixture re-keys are exhaustive across the match
domain. Local validation evidence is comprehensive.

One hygiene concern (registry message-template drift for `SIFR-MATCH-0001`),
two minor design observations, and a few nits — none blocking.

**Recommendation: ready for PR**, with the option to address the
`SIFR-MATCH-0001` template drift either now or as a deferred follow-up.

## What was reviewed

- [crates/sifr_hir/src/lower/match_diagnostics.rs](crates/sifr_hir/src/lower/match_diagnostics.rs)
- [crates/sifr_hir/src/lower/match_diagnostics_tests.rs](crates/sifr_hir/src/lower/match_diagnostics_tests.rs)
- [crates/sifr_hir/src/lower/mod.rs](crates/sifr_hir/src/lower/mod.rs:42) — module registration
- [crates/sifr_hir/src/lower/statements.rs](crates/sifr_hir/src/lower/statements.rs:644) — `lower_match` / `lower_pattern` migration
- [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs:951) — registry message-template alignment
- [docs/errors/SIFR-MATCH-0002.md](docs/errors/SIFR-MATCH-0002.md), [docs/errors/SIFR-MATCH-0003.md](docs/errors/SIFR-MATCH-0003.md)
- [internal_docs/diagnostic_codes.md](internal_docs/diagnostic_codes.md)
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- 6 re-keyed fixtures under `crates/sifr/tests/e2e/fail/`.

## Correctness — migration completeness

Confirmed: all four raw `ctx.error(...)` call sites in the match-lowering path
that are covered by `SIFR-MATCH-0001..0003` are migrated to the new helpers.
A repo-wide grep for the original message strings finds them only inside the
new helper bodies and the new unit-test file:

- `match guard must be a bool expression, got '...'` →
  [match_diagnostics.rs:5](crates/sifr_hir/src/lower/match_diagnostics.rs:5),
  call site [statements.rs:664](crates/sifr_hir/src/lower/statements.rs:664).
- `non-exhaustive match: type '...' has uncovered variants: ...` (union) →
  [match_diagnostics.rs:12](crates/sifr_hir/src/lower/match_diagnostics.rs:12),
  call site [statements.rs:803](crates/sifr_hir/src/lower/statements.rs:803).
- `non-exhaustive match: enum '...' has uncovered variants: ...` →
  [match_diagnostics.rs:21](crates/sifr_hir/src/lower/match_diagnostics.rs:21),
  call site [statements.rs:841](crates/sifr_hir/src/lower/statements.rs:841).
- `non-exhaustive match: type '...' cannot be fully covered by literal patterns ...` →
  [match_diagnostics.rs:30](crates/sifr_hir/src/lower/match_diagnostics.rs:30),
  call site [statements.rs:861](crates/sifr_hir/src/lower/statements.rs:861).
- `class '...' has no field '...' — available fields: ...` →
  [match_diagnostics.rs:39](crates/sifr_hir/src/lower/match_diagnostics.rs:39),
  call site [statements.rs:967](crates/sifr_hir/src/lower/statements.rs:967).

The two remaining raw `ctx.error(...)` calls in `lower_pattern` —
`"class pattern class name must be a simple name"`
([statements.rs:947](crates/sifr_hir/src/lower/statements.rs:947)) and
`"tuple pattern requires subject of tuple type, got '...'"`
([statements.rs:996](crates/sifr_hir/src/lower/statements.rs:996)) — are
**out of scope** for this slice (they are not part of `SIFR-MATCH-0001..0003`
and don't yet have allocated codes). The slice is correctly bounded; just
calling it out so this isn't read as an oversight.

## Correctness — registry / docs alignment

`crates/sifr_diagnostics/src/codes.rs` and the generated docs are now
consistent for the migrated codes:

| Code | Template (registry, `codes.rs`) | Helper-emitted message |
|------|----|----|
| `SIFR-MATCH-0002` | `match guard must be a bool expression, got {actual}` | `match guard must be a bool expression, got '{actual}'` |
| `SIFR-MATCH-0003` | `class {class_name} has no field {field}` | `class '{class_name}' has no field '{field_name}' — available fields: ...` |

The slight quoting/elaboration delta (single quotes around values, "available
fields" tail) is normal across the registry — templates are documentation-only
strings consumed by `gen-error-docs` and the placeholder-declaration test, not
runtime format strings (verified at
[codes.rs:1633](crates/sifr_diagnostics/src/codes.rs:1633) and the registry
sync script). All e2e fixtures only assert a message *substring*
([e2e.rs:2563](crates/sifr/tests/e2e.rs:2563)), so the quoting delta is safe.

## Tests

[match_diagnostics_tests.rs](crates/sifr_hir/src/lower/match_diagnostics_tests.rs)
adds five tests, exactly one per helper. Each asserts both the structured
`code` and the exact message. The shared `lower_errors` helper goes through
the public `lower_module` entry point, so these tests also cover the
`statements.rs` wiring, not just the helpers in isolation. The module is
correctly gated under `#[cfg(test)]` at
[mod.rs:43](crates/sifr_hir/src/lower/mod.rs:43).

E2E coverage is exhaustive across the match domain: a grep across
`crates/sifr/tests/e2e/fail/*.sifr` for `SIFR-TYPE-0001` confirms no remaining
match-shaped fixture uses the legacy bucket. The six re-keyed fixtures map to
codes as follows:

| Fixture | New code |
|---|---|
| `enum_match_non_exhaustive.sifr` | `SIFR-MATCH-0001` |
| `match_non_exhaustive_literal.sifr` | `SIFR-MATCH-0001` |
| `match_non_exhaustive_optional.sifr` | `SIFR-MATCH-0001` |
| `match_non_exhaustive_union.sifr` | `SIFR-MATCH-0001` |
| `match_type_mismatch_guard.sifr` | `SIFR-MATCH-0002` |
| `match_invalid_field_name.sifr` | `SIFR-MATCH-0003` |

All four `SIFR-MATCH-0001` shapes (union, optional, literal, enum) are
exercised end-to-end.

## Findings

### F1. Registry template drift for `SIFR-MATCH-0001` is left unaligned

**Severity: low (hygiene), non-blocking.**

The slice description explicitly aligned templates for `SIFR-MATCH-0002` and
`SIFR-MATCH-0003`, but `SIFR-MATCH-0001` still carries the legacy placeholder:

```
non-exhaustive match for {subject_type}
```
([codes.rs:946](crates/sifr_diagnostics/src/codes.rs:946))

The actually-emitted messages for that code are three distinct, more
informative strings:

- `non-exhaustive match: type '{subject_type}' has uncovered variants: {uncovered} — add matching case(s) or \`case _:\``
- `non-exhaustive match: enum '{enum_name}' has uncovered variants: {uncovered} — add matching case(s) or \`case _:\``
- `non-exhaustive match: type '{subject_type}' cannot be fully covered by literal patterns — add \`case _:\` to handle remaining values`

The current template doesn't document `uncovered`, doesn't reflect any of the
three real shapes, and is now the only `MATCH` template that diverges from
emitted text. The asymmetry will be a near-certain reviewer comment in the
next pass.

Two reasonable resolutions:

1. Pick the most representative shape — most likely the union form — and
   update template + declared args (add `uncovered (message+json)`). Document
   the enum/literal alternates as message variants only. This matches what
   `SIFR-MATCH-0002` / `SIFR-MATCH-0003` did.
2. Explicitly acknowledge in the issue tracker (or as a follow-up entry) that
   `SIFR-MATCH-0001` carries multiple concrete messages under one code and
   defer template alignment until per-shape codes are split.

Either is fine; the inconsistency itself is the issue, not which way it gets
resolved. The diagnostic-docs sync script doesn't enforce template-vs-emitted
parity, so this won't fail validation, but it shows up in any human read.

### F2. One code, three message shapes (`SIFR-MATCH-0001`)

**Severity: discussion-only, non-blocking.**

The non-exhaustive-match path emits three distinct human-readable messages
under one code, with three different argument vocabularies (`subject_type`
+ `uncovered`, `enum_name` + `uncovered`, `subject_type` only). This is
consistent with prior precedent (`SIFR-OWN-0001` etc.) so it isn't out of
character for this phase. Worth flagging as something the structured-args
work in a later milestone will need to disambiguate (either three sub-codes
or a `kind` discriminator arg). Not actionable now.

### F3. Helpers take pre-joined `&str` for multi-value args

**Severity: nit.**

`non_exhaustive_union` and `non_exhaustive_enum` accept `uncovered: &str` —
a comma-joined list pre-built at the call site
([statements.rs:805](crates/sifr_hir/src/lower/statements.rs:805),
[statements.rs:842](crates/sifr_hir/src/lower/statements.rs:842)). Future
structured-arg JSON output will likely want a `Vec<String>` or `&[String]`
so a renderer can format the list itself. For this slice the current shape
is fine and matches the existing helper style, but it's worth noting if
slice 2c or later moves toward typed structured args — these helpers will
be the obvious place to widen the parameter type.

### F4. Style: full-path call vs `use` import

**Severity: nit.**

`statements.rs` already does `use super::ownership_diagnostics;` at line 27
and uses the bare module name at call sites, while it keeps
`super::flow_diagnostics::*` fully qualified. The new code follows the
`flow_diagnostics` style:

```rust
super::match_diagnostics::guard_not_bool(ctx, &guard_ty.display_name());
```

Both styles already coexist in the file; no action needed. If a future
cleanup unifies on the `use super::...;` form, this can be folded in then.

### F5. Test fragility — guard subject expression

**Severity: nit, mitigated.**

`match_guard_type_error_has_match_code` constructs the failing guard via
`n + 1`, relying on `int + int` evaluating to `int`. If a future slice
refines arithmetic to a literal type or otherwise narrows the result, the
test could fall through to a different diagnostic. The risk is mitigated
because the parallel e2e fixture `match_type_mismatch_guard.sifr` exercises
the exact same shape — both would break together and be obvious.

### F6. Tracker entry — wording is fine; PR URL placeholder

The entry on
[issues/...md:53](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53)
matches the prior in-progress slice template verbatim. The "PR: pending"
placeholder will need to be flipped once the PR opens, plus the standard
review-completed entry added once review wraps. No action now.

## Behavioral regressions / panics / ownership

None observed:

- No new `unwrap` / `expect` / `panic!` in non-test code.
- All five helpers route through `ctx.error_with_code`, preserving the same
  emission-once semantics as the raw `ctx.error` calls they replaced (no
  early-return changes — note the explicit `return None` at
  [statements.rs:977](crates/sifr_hir/src/lower/statements.rs:977) is
  preserved).
- Helper bodies do nothing besides format and forward; no dataflow change.
- `LowerCtx` borrow patterns at call sites are unchanged (`&mut LowerCtx` in,
  string-typed args by value/`&str`).
- The `cfg(test)` gating is correct, so no test code leaks into release
  builds.

## Validation evidence (as reported)

User-supplied report-signature `e1bf653aaa770517` from
`scripts/run_all_tests.sh --profile quick` (wall_time 141.16s) plus the full
suite of focused checks (`gen-error-docs`, `check_diagnostic_docs_sync.py`,
`check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`,
`cargo test -p sifr_diagnostics`, `cargo test -p sifr_hir
match_diagnostics_tests`, `cargo test -p sifr --test e2e -- test_e2e_fail`,
`cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace --
-D warnings`). I did not re-execute these locally; the change set is small
and consistent with what those gates already cover.

## Verdict

Slice is implementation-complete and ready for PR. The single hygiene point
worth addressing before opening — or explicitly deferring — is the
`SIFR-MATCH-0001` registry template drift in finding **F1**. Everything
else is nit-level and can ride along in a future cleanup pass.

## Suggested action items (in priority order)

1. **F1** — update or explicitly defer the `SIFR-MATCH-0001` registry
   template + declared args. Mirror what was done for `SIFR-MATCH-0002` /
   `SIFR-MATCH-0003`, or add a tracker line acknowledging the deferral.
2. **F6** — populate the PR URL on the tracker line once the PR is opened.
3. (Optional, future slice) Decide whether `SIFR-MATCH-0001`'s three message
   shapes should split into sub-codes or stay unified with a `kind` arg
   (**F2**); revisit helper signatures (**F3**) when structured args land.
