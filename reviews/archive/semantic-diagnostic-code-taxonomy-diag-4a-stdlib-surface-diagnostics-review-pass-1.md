# `milestone_diag_4a` slice 2b.25 — Stdlib surface diagnostics migration

Pass 1 review for the uncommitted tree on branch
`codex/semantic-diagnostics-diag-4a-stdlib-surface-diagnostics`.

## Scope under review

- Mark slice 2b.24 merged in the issue tracker after [sifr-lang/sifr#1696](https://github.com/sifr-lang/sifr/pull/1696).
- Migrate the unsupported `defaultdict()` keyword-constructor diagnostic from
  the generic `SIFR-TYPE-0001` bridge to active `SIFR-STDLIB-0001`.
- Align registry template / args / owner / docs with the emitted message.
- Re-key the fail fixture and add focused HIR unit coverage.
- Repair the slice 2b.24 class format strings so clippy's
  `uninlined_format_args` lint passes on current main.

## Verdict

**Approved — no blockers.** Behavior, registry, fixture, docs, and HIR test
all line up. A few stylistic and follow-up notes are flagged below for the
next slice.

## What was checked

### 1. HIR call site migration
[crates/sifr_hir/src/lower/builtin_calls.rs:407](crates/sifr_hir/src/lower/builtin_calls.rs:407)

- `lower_defaultdict_constructor_call` now emits via
  `ctx.error_with_code(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, ...)` with
  the verbatim message `defaultdict() does not support keyword arguments`.
- `DiagnosticCode` import is added correctly at the top of the file.
- `error_with_code` populates `LoweringError.code = Some(...)` (see
  [crates/sifr_hir/src/lower/mod.rs:237](crates/sifr_hir/src/lower/mod.rs:237)),
  so the active code surfaces through the renderer instead of the
  `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge.
- `STDLIB_UNSUPPORTED_SURFACE` already existed in the constant list and the
  active-code array
  ([crates/sifr_diagnostics/src/codes.rs:97](crates/sifr_diagnostics/src/codes.rs:97),
  [crates/sifr_diagnostics/src/codes.rs:1405](crates/sifr_diagnostics/src/codes.rs:1405)),
  so no enum/registry plumbing is missing.

### 2. Registry entry alignment
[crates/sifr_diagnostics/src/codes.rs:1112](crates/sifr_diagnostics/src/codes.rs:1112)

- Template tightened from `unsupported standard-library surface {symbol}` to
  the verbatim emitted message.
- Owner narrowed from `sifr_hir::lower` to `sifr_hir::lower::builtin_calls`,
  matching the emitting module.
- Declared and dedupe args set to `[]` — consistent with a verbatim message
  having no `{}` placeholders.
- Representative fixture remains the re-keyed
  [crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr](crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr).

### 3. Generated docs
[docs/errors/SIFR-STDLIB-0001.md](docs/errors/SIFR-STDLIB-0001.md),
[internal_docs/diagnostic_codes.md:126](internal_docs/diagnostic_codes.md:126)

- Both are regenerated consistently with the new template/owner/args.
- `Owner` and `Message template` rows match the registry entry verbatim.
- Description in [docs/errors/diagnostic-codes.md:97](docs/errors/diagnostic-codes.md:97)
  remains the family-level `Unsupported standard-library constructor, method,
  or surface.` — unchanged and still accurate.

### 4. Fixture re-keying
[crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr:2](crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr:2)

- `expect-error` line updated to
  `SIFR-STDLIB-0001: defaultdict() does not support keyword arguments`.
- `parse_expected_error`
  ([crates/sifr/tests/e2e.rs:596](crates/sifr/tests/e2e.rs:596))
  splits on the first `:` after the code, so the message substring is
  `defaultdict() does not support keyword arguments` — exact match against the
  emit. The active code passes `is_diagnostic_code`.

### 5. Focused HIR test
[crates/sifr_hir/src/lower/expressions_tests.rs:232](crates/sifr_hir/src/lower/expressions_tests.rs:232)

- Asserts code+message identity using
  `error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)`.
- Uses `iter().any(...)`, so secondary downstream errors from the failed
  constructor (the `_ = groups` use, etc.) cannot mask or break the assertion.
- Style and import (`use sifr_diagnostics::DiagnosticCode;`) match other
  domain tests in this file.

### 6. Slice 2b.24 class format-string repair
[crates/sifr_hir/src/lower/classes.rs:255](crates/sifr_hir/src/lower/classes.rs:255),
[crates/sifr_hir/src/lower/classes.rs:574](crates/sifr_hir/src/lower/classes.rs:574),
[crates/sifr_hir/src/lower/classes.rs:599](crates/sifr_hir/src/lower/classes.rs:599)

- 2b.24 had introduced the keyword-argument form
  `format!("...{enum_name}...", enum_name = class_name, ...)` that trips
  clippy's `uninlined_format_args` on current main (verified via
  `git show 2b2e9fb7 -- crates/sifr_hir/src/lower/classes.rs`).
- The repair adds local aliases (`let enum_name = class_name.as_str();`,
  `let value = val;`, `let existing_variant = existing;`,
  `let duplicate_variant = vname;`, `let field = fname.as_str();`) and removes
  the keyword-argument form so the inline-capture lint is satisfied.
- Display behavior is unchanged: `class_name.as_str()` Display equals
  `String` Display; the rebinds for `i64` (Copy) and `&String` are
  pass-through. Messages are byte-identical to the slice 2b.24 emissions.
- Bundling this in slice 2b.25 is acceptable as a trunk-green fix, but
  worth calling out explicitly in the PR description so the scope is
  documented.

### 7. Issue tracker
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:59](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:59)

- Slice 2b.24 ticked with the [sifr-lang/sifr#1696](https://github.com/sifr-lang/sifr/pull/1696)
  link. New slice 2b.25 in-progress entry added with `PR: pending`.

## Non-blocking findings / follow-ups

1. **Sibling `defaultdict()` diagnostics still uncoded.**
   `lower_defaultdict_constructor_call` keeps four bare `ctx.error(...)`
   sites for positional arity, non-name factory, unsupported factory name,
   and initial-mapping shape — all natural fits for `SIFR-STDLIB-0001`.
   They still fall through the `SIFR-TYPE-0001` bridge.
   See [crates/sifr_hir/src/lower/builtin_calls.rs:415](crates/sifr_hir/src/lower/builtin_calls.rs:415),
   [crates/sifr_hir/src/lower/builtin_calls.rs:425](crates/sifr_hir/src/lower/builtin_calls.rs:425),
   [crates/sifr_hir/src/lower/builtin_calls.rs:431](crates/sifr_hir/src/lower/builtin_calls.rs:431),
   [crates/sifr_hir/src/lower/builtin_calls.rs:459](crates/sifr_hir/src/lower/builtin_calls.rs:459),
   [crates/sifr_hir/src/lower/builtin_calls.rs:467](crates/sifr_hir/src/lower/builtin_calls.rs:467),
   [crates/sifr_hir/src/lower/builtin_calls.rs:484](crates/sifr_hir/src/lower/builtin_calls.rs:484),
   [crates/sifr_hir/src/lower/builtin_calls.rs:494](crates/sifr_hir/src/lower/builtin_calls.rs:494).
   Out of scope for 2b.25; queue for the next stdlib slice.

2. **Registry template now records a single concrete emission.**
   With `Message template = "defaultdict() does not support keyword arguments"`
   and `Owner = sifr_hir::lower::builtin_calls`, the entry effectively
   documents one site. As more sites migrate (the four above, plus
   `Counter()` kwargs, `OrderedCounter()` kwargs, etc.), expect to either
   broaden the template (e.g., `{message}`/`{symbol}`-style placeholder) or
   re-introduce templated args. Not a problem for this slice — just flagging
   for planning.

3. **Counter / OrderedCounter kwargs fixtures lack `expect-error`.**
   [crates/sifr/tests/e2e/fail/counter_kwargs_constructor_unsupported.sifr](crates/sifr/tests/e2e/fail/counter_kwargs_constructor_unsupported.sifr)
   and
   [crates/sifr/tests/e2e/fail/ordered_counter_kwargs_constructor_unsupported.sifr](crates/sifr/tests/e2e/fail/ordered_counter_kwargs_constructor_unsupported.sifr)
   are parallel "stdlib constructor rejects kwargs" surfaces. They live
   outside this slice's scope, but they belong in the same family and would
   be the natural companion fixtures when the sibling diagnostics above are
   migrated.

4. **Stylistic — alias rebinds vs. natural names.**
   The class repair introduces `let enum_name = class_name.as_str(); let
   value = val; let existing_variant = existing; let duplicate_variant =
   vname;` purely to keep the descriptive names in the format string. A
   shorter refactor — used by the pre-2b.24 code — is to drop the aliases
   and write `format!("enum '{class_name}' has duplicate value {val}:
   variants '{existing}' and '{vname}'")`. Equivalent rendering, fewer
   lines. Same applies to the `field`/`fname` and the `class_name = class_name`
   tail in the missing-init branch. Optional cleanup.

5. **`internal_docs/diagnostic_emission_inventory.md`.**
   The existing row for `SIFR-STDLIB-0001` (line 346) is still phrased as
   the family-level placeholder ("unsupported stdlib constructor/method
   surface … stdlib/builtin lowering"). It is consistent with the design
   intent and does not need a change for this slice, but as items 1–3 land
   it should be revisited so the inventory keeps matching the active
   call-site footprint.

## Validation cross-check

User-reported local validation that I did not re-run, but is consistent with
the diff:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs` — registry change
  matches the regenerated `docs/errors/SIFR-STDLIB-0001.md` and
  `internal_docs/diagnostic_codes.md` rows.
- `cargo fmt --check` — diff is well-formatted.
- `python3 scripts/check_diagnostic_docs_sync.py` — `--check` mode of the
  same generator above.
- `python3 scripts/check_diagnostic_schema_sync.py` — schema is
  derive-driven, no model-shape change here.
- `python3 scripts/check_hir_maintainability_guardrails.py` — file sizes
  unaffected.
- `cargo test -p sifr_hir defaultdict_keyword_constructor_unsupported` — new
  HIR test covers code+message identity.
- `cargo test -p sifr_hir class_code` — exercises the slice 2b.24 codes
  whose format strings were repaired.
- `cargo test -p sifr --test e2e -- test_e2e_fail` — picks up the re-keyed
  fixture; `parse_expected_error` extracts the new code/message correctly.
- `cargo test -p sifr -- --skip test_e2e_pass` — broad HIR/driver coverage.
- `cargo clippy --workspace -- -D warnings` — repaired class format strings
  no longer trigger `uninlined_format_args`.

## Recommendation

Ship slice 2b.25 as-is. After merge, the next stdlib slice should:

- Migrate the remaining `lower_defaultdict_constructor_call` `ctx.error(...)`
  sites to `SIFR-STDLIB-0001` and broaden the registry template/args to fit.
- Migrate the `Counter()` / `OrderedCounter()` kwargs rejection paths and
  add `expect-error` directives to the existing parallel fixtures.
- Optionally collapse the class-format-string aliases to natural names
  (item 4) when other class lowering work is open in the same file.
