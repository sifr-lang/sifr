# `milestone_diag_4a` slice 2b.24 — class diagnostics migration (pass 2)

Re-review of uncommitted changes on
`codex/semantic-diagnostics-diag-4a-class-diagnostics` after the pass-1
follow-ups.

Pass-1 review:
[reviews/semantic-diagnostic-code-taxonomy-diag-4a-class-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-class-diagnostics-review-pass-1.md).

## Verdict

**Reviewer-satisfied / approved for PR.** All three pass-1 observations were
addressed; no new blocker or correctness issue surfaced; sync scripts, fmt
check, focused HIR unit tests, and the e2e fail suite are clean against the
current tree.

## Pass-1 follow-up status

### 1. (Low, pass 1) `SIFR-CLASS-0004` scope mismatch — **resolved**

Pass 1 flagged the ambiguity between the constant's `MEMBER`-shaped framing
and a template that only covers field access. Pass 2 narrows the scope on the
registry side instead of broadening the template:

- Registry description now reads `"Missing class field."`
  ([crates/sifr_diagnostics/src/codes.rs:1071](crates/sifr_diagnostics/src/codes.rs:1071)),
  matching the active template `type '{type_name}' has no field '{field}'`.
- Generated artifacts updated in lockstep:
  [docs/errors/SIFR-CLASS-0004.md:5](docs/errors/SIFR-CLASS-0004.md:5) and
  the index row at
  [docs/errors/diagnostic-codes.md:93](docs/errors/diagnostic-codes.md:93).
- Internal index row at
  [internal_docs/diagnostic_codes.md:122](internal_docs/diagnostic_codes.md:122)
  reflects the new owner `sifr_hir::lower::expressions` and the new
  template/args.

The Rust constant name `CLASS_MISSING_MEMBER`
([codes.rs:91](crates/sifr_diagnostics/src/codes.rs:91)) is still
broader-sounding than the now-narrowed registry description, but pass 1
explicitly framed the fix at the registry-description level (not as a
constant rename) so the scope is now unambiguous on the user-facing surface.
Renaming the constant would create churn for no behavioral benefit; leaving
it is the right call. The reserved code space (`SIFR-CLASS-0004` covers
"missing class field") is now a stable contract for future method-lookup
migrations to either share or get a new code, rather than ambiguously claim
already-implemented coverage.

### 2. (Nit, pass 1) Stale source-area in emission inventory — **resolved**

[internal_docs/diagnostic_emission_inventory.md:342](internal_docs/diagnostic_emission_inventory.md:342)
now reads
`| SIFR-CLASS-0004 | missing class field | attribute lowering | … |`,
matching the new emission site in `lower::expressions::lower_attribute` and
aligning with the owner column in the internal diagnostic-codes index.

### 3. (Nit, pass 1) Mixed `format!` capture style — **resolved**

All four migration sites now use fully explicit named arguments:

- [classes.rs:255-264](crates/sifr_hir/src/lower/classes.rs:255) (CLASS-0003)
  passes `enum_name`, `value`, `existing_variant`, `duplicate_variant`.
- [classes.rs:574-581](crates/sifr_hir/src/lower/classes.rs:574) (CLASS-0002)
  passes `class_name`, `field`.
- [classes.rs:600-607](crates/sifr_hir/src/lower/classes.rs:600) (CLASS-0001)
  passes `class_name`.
- [expressions.rs:2269-2276](crates/sifr_hir/src/lower/expressions.rs:2269)
  (CLASS-0004) passes `type_name`, `field`.

This is the convention pass 1 noted other 2b.* slices have favored, so the
slice now matches the rest of the migrated codes.

## What I re-verified end-to-end

### Registry ↔ emitted-message byte-equality

For each migrated code I re-confirmed the registry template byte-equals the
post-line-continuation `format!` output, and that arg/dedupe lists match
the placeholders one-for-one:

- **SIFR-CLASS-0001**: emitter at
  [classes.rs:600](crates/sifr_hir/src/lower/classes.rs:600) ↔ registry at
  [codes.rs:1036](crates/sifr_diagnostics/src/codes.rs:1036). Args:
  `class_name` only, matches the single placeholder.
- **SIFR-CLASS-0002**: emitter at
  [classes.rs:574](crates/sifr_hir/src/lower/classes.rs:574) ↔ registry at
  [codes.rs:1047](crates/sifr_diagnostics/src/codes.rs:1047). Args:
  `class_name`, `field`.
- **SIFR-CLASS-0003**: emitter at
  [classes.rs:255](crates/sifr_hir/src/lower/classes.rs:255) ↔ registry at
  [codes.rs:1058](crates/sifr_diagnostics/src/codes.rs:1058). Args:
  `enum_name`, `value`, `existing_variant`, `duplicate_variant` — all four
  placeholders present, ordering aligned with the template.
- **SIFR-CLASS-0004**: emitter at
  [expressions.rs:2269](crates/sifr_hir/src/lower/expressions.rs:2269) ↔
  registry at [codes.rs:1074](crates/sifr_diagnostics/src/codes.rs:1074).
  Args: `type_name`, `field`. Owner attributed to
  `sifr_hir::lower::expressions` ✓.

### Bridge bypass still holds

`CompileError::diagnostic_code` short-circuits whenever
`Some(code)` is set, so the four migrated sites no longer reach the
`TypeCheck → SIFR-TYPE-0001` legacy fallback. A grep for `SIFR-TYPE-0001`
in the class-related fail fixtures returns no remaining hits for these four
phenomena (the only surviving `SIFR-TYPE-0001` class-area fixture is the
hashability protocol case `unhashable_dict_key.sifr`, out of scope for
slice 2b.24).

### Fixtures

All four `# expect-error:` headers re-keyed from `SIFR-TYPE-0001` to the
matching `SIFR-CLASS-000{1..4}` and the comment substrings still match the
rendered messages. The e2e harness asserts both code equality and message
substring containment, and the suite now passes (see "Validation" below).

### HIR unit coverage

The four new tests in
[crates/sifr_hir/src/lower/expressions_tests.rs:1870-1921](crates/sifr_hir/src/lower/expressions_tests.rs:1870)
each construct a minimal source that triggers exactly the targeted
diagnostic and assert both `e.message == "<exact rendered message>"` and
`e.code == Some(DiagnosticCode::CLASS_*)`. This locks down both the textual
template and the wiring through `error_with_code` for every migrated
constant. They run in 0.01s as a focused unit-level guard.

### Generated docs and indexes

`docs/errors/SIFR-CLASS-000{1..4}.md` and the row in
`docs/errors/diagnostic-codes.md` are regenerated and match the registry.
`internal_docs/diagnostic_codes.md` rows for all four codes carry the new
templates, owners, and arg lists. `internal_docs/diagnostic_emission_inventory.md:342`
agrees with the new owner module.

### Issue-tracker bookkeeping

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55-59](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55)
flips slice 2b.23 to `[x] merged` with the existing PR link and adds an
in-progress entry for slice 2b.24, consistent with the wording style of
2b.20–2b.23.

### Stale-reference sweep

Greps across `crates/`, `docs/`, `internal_docs/`, and `verification/`
turned up no surviving references to the prior, never-merged registry
templates (`class … requires an initializer for field …`,
`required field … cannot follow a defaulted field`,
`duplicate or invalid class value …`, `class … has no member …`) outside
the pass-1 review file (which references them by quotation, as expected).
Likewise, no remaining `field or member` description fragment or
`missing class field/member` inventory wording outside the pass-1 review
file.

## Validation re-run on the current tree

```
cargo run -q -p sifr_diagnostics --bin gen-error-docs -- --check   # clean
cargo fmt --check                                                  # clean
python3 scripts/check_diagnostic_docs_sync.py                      # clean
python3 scripts/check_diagnostic_schema_sync.py                    # clean
cargo test -q -p sifr_hir class_code                               # 4 passed
cargo test -q -p sifr --test e2e -- test_e2e_fail                  # 1 passed
```

All gates the user listed are passing on the current tree. No additional
breakage observed in adjacent suites.

## Findings (pass 2)

None blocking. The non-blocking constant-name observation captured under
"Pass-1 follow-up status / 1." is informational only — it does not contradict
the pass-1 resolution direction and requires no action for this slice.

## Suggested follow-ups (optional, out of scope)

- If a future slice migrates the still-untagged "has no class/static method"
  / "has no method" emissions in `lower::expressions`
  ([expressions.rs:2384](crates/sifr_hir/src/lower/expressions.rs:2384),
  [:3198](crates/sifr_hir/src/lower/expressions.rs:3198), and the builtin
  method-lookup family around
  [expressions.rs:2694](crates/sifr_hir/src/lower/expressions.rs:2694)),
  decide at that point whether to allocate a new method-specific code or
  rename `CLASS_MISSING_MEMBER` (and its constant) to a new, narrower symbol
  that matches the now-narrowed registry description. Either direction is a
  cosmetic refactor; the current slice does not need it.
