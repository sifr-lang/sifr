# Review — `milestone_diag_4a` slice 2b.19 (name and import diagnostics) — pass 1

Branch: `codex/semantic-diagnostics-diag-4a-name-import-diagnostics`
Scope: migrate undefined-variable, undefined-function, missing-module-member,
forbidden-intrinsic-import, and unknown-import-target HIR errors onto the active
codes `SIFR-NAME-0001`, `SIFR-NAME-0002`, `SIFR-NAME-0004`, `SIFR-IMPORT-0001`,
and `SIFR-IMPORT-0002`; introduce the
[`name_diagnostics`](crates/sifr_hir/src/lower/name_diagnostics.rs) and
[`import_diagnostics`](crates/sifr_hir/src/lower/import_diagnostics.rs) helper
modules with shared unit tests
([`name_import_diagnostics_tests.rs`](crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs)),
re-key eight e2e fail fixtures off `SIFR-TYPE-0001`, and update the slice
tracker.

Review style: read-only audit, no files modified.

## TL;DR

The migration itself is correct and consistent with prior structurally-coded
slices (RESULT, OWN, FLOW, MATCH). All five helper constructors carry the right
codes, the eight e2e fixture re-keys land on the matching codes, and there are
no remaining raw `ctx.error(format!(...))` call sites in production HIR code
for any of the five diagnostic kinds in this slice.

However, one downstream test was missed: the migration consolidates three
distinct user-visible messages (`unknown intrinsic module 'X'`,
`unknown stdlib module 'X'`, `unknown module 'X'`) into a single
`unknown import target: 'X'` form, and a `sifr_driver` test still asserts on
the old `unknown module '...'` substring.

**Recommendation: not ready for PR.** Fix the failing `sifr_driver` test
([item 1](#1-blocker-sifr_driver-test-still-asserts-on-the-old-unknown-module-message)),
then merge. Everything else listed below is informational or non-blocking.

## What was reviewed

- [crates/sifr_hir/src/lower/name_diagnostics.rs](crates/sifr_hir/src/lower/name_diagnostics.rs)
- [crates/sifr_hir/src/lower/import_diagnostics.rs](crates/sifr_hir/src/lower/import_diagnostics.rs)
- [crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs](crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs)
- [crates/sifr_hir/src/lower/mod.rs](crates/sifr_hir/src/lower/mod.rs:39) — module registration and migrated import call sites
- Migrated call-site files:
  - [crates/sifr_hir/src/lower/aug_assign_lowering.rs:299](crates/sifr_hir/src/lower/aug_assign_lowering.rs:299)
  - [crates/sifr_hir/src/lower/expressions.rs:248](crates/sifr_hir/src/lower/expressions.rs:248), [1718](crates/sifr_hir/src/lower/expressions.rs:1718)
  - [crates/sifr_hir/src/lower/statements.rs:1542](crates/sifr_hir/src/lower/statements.rs:1542)
  - [crates/sifr_hir/src/lower/tuple_unpack.rs:101](crates/sifr_hir/src/lower/tuple_unpack.rs:101)
- Re-keyed e2e fail fixtures (8):
  - [enum_invalid_variant.sifr](crates/sifr/tests/e2e/fail/enum_invalid_variant.sifr) → `SIFR-NAME-0001`
  - [import_intrinsic.sifr](crates/sifr/tests/e2e/fail/import_intrinsic.sifr) → `SIFR-IMPORT-0001`
  - [import_nonexistent_local.sifr](crates/sifr/tests/e2e/fail/import_nonexistent_local.sifr) → `SIFR-IMPORT-0002`
  - [stdlib_intrinsic_direct_import.sifr](crates/sifr/tests/e2e/fail/stdlib_intrinsic_direct_import.sifr) → `SIFR-IMPORT-0001`
  - [stdlib_invalid_import.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_import.sifr) → `SIFR-NAME-0004`
  - [stdlib_invalid_module.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr) → `SIFR-IMPORT-0002`
  - [stdlib_missing_function.sifr](crates/sifr/tests/e2e/fail/stdlib_missing_function.sifr) → `SIFR-NAME-0004`
  - [undefined_var.sifr](crates/sifr/tests/e2e/fail/undefined_var.sifr) → `SIFR-NAME-0001`
- Issue tracker: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- Cross-referenced registry definitions in
  [crates/sifr_diagnostics/src/codes.rs:21](crates/sifr_diagnostics/src/codes.rs:21)
  through [:27](crates/sifr_diagnostics/src/codes.rs:27) and active registry
  entries at [:507–:572](crates/sifr_diagnostics/src/codes.rs:507).

## Findings

### 1. Blocker — `sifr_driver` test still asserts on the old "unknown module" message

[crates/sifr_driver/src/tests/project_graph.rs:382–384](crates/sifr_driver/src/tests/project_graph.rs:382)
checks:

```rust
assert!(errors
    .iter()
    .any(|e| e.message.contains("unknown module 'missing_mod'")));
```

After this slice, the local-module branch in `lower_module_impl` no longer
emits `"unknown module '<name>'"`; it now routes through
[`import_diagnostics::unknown_import_target`](crates/sifr_hir/src/lower/import_diagnostics.rs:12)
and produces `"unknown import target: '<name>'"`
([crates/sifr_hir/src/lower/mod.rs:987](crates/sifr_hir/src/lower/mod.rs:987)).

I confirmed both the regression and that it is caused by this slice:

```
test tests::project_graph::test_collect_project_modules_reports_unknown_module_in_non_main ... FAILED

thread 'tests::project_graph::test_collect_project_modules_reports_unknown_module_in_non_main' panicked at crates/sifr_driver/src/tests/project_graph.rs:382:5:
assertion failed: errors.iter().any(|e| e.message.contains("unknown module 'missing_mod'"))
```

The test passes on `367b10a8` (the merge base) and fails on the working tree.

**Why it was missed:** the validation list in the task description does not
include any `sifr_driver` invocation. `cargo test -p sifr -- --skip test_e2e_pass`
does not exercise `sifr_driver` tests, and `cargo clippy --workspace` does not
run tests. The canonical pre-PR gate per `AGENTS.md`,
`scripts/run_all_tests.sh --profile quick`, runs
`cargo test -p sifr_driver --lib` at
[scripts/run_all_tests.sh:112](scripts/run_all_tests.sh:112) and would have
caught this. Recommend running it before opening the PR.

**Fix:** update the assertion (and ideally the test name and comment) to the
new message, e.g. `"unknown import target: 'missing_mod'"`. Strengthening to
also assert `e.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)`
would lock in the structured contract while you are there.

### 2. Quality regression — three distinct messages collapsed into one

[`unknown_import_target`](crates/sifr_hir/src/lower/import_diagnostics.rs:12)
is now invoked from three previously-distinct call sites in `lower_module_impl`:

| Path | Old message | New message |
| --- | --- | --- |
| Intrinsic import miss ([mod.rs:817](crates/sifr_hir/src/lower/mod.rs:817)) | `unknown intrinsic module '<name>'` | `unknown import target: '<name>'` |
| Stdlib import miss ([mod.rs:978](crates/sifr_hir/src/lower/mod.rs:978)) | `unknown stdlib module '<name>'` | `unknown import target: '<name>'` |
| Local import miss ([mod.rs:987](crates/sifr_hir/src/lower/mod.rs:987)) | `unknown module '<name>'` | `unknown import target: '<name>'` |

Similarly, [`name_diagnostics::missing_member`](crates/sifr_hir/src/lower/name_diagnostics.rs:19)
flattens the intrinsic-specific phrasing
`intrinsic module '<X>' has no member '<Y>'`
([prior mod.rs](crates/sifr_hir/src/lower/mod.rs:807) line) into the generic
`module '<X>' has no member '<Y>'`.

The user can usually still infer the kind from the module name's prefix
(`_sifr.*` is intrinsic; `sifr.*` is stdlib), so this is not a correctness
issue — and the registry's canonical template at
[crates/sifr_diagnostics/src/codes.rs:568](crates/sifr_diagnostics/src/codes.rs:568)
explicitly says `"unknown import target: {module}"`, so consolidation is in
fact the *intended* taxonomy. Calling this out so the loss of context is
documented in the PR description and so reviewers expect the change.

If the slice author wants to preserve information density without re-fanning
helpers, a one-line note attachment ("source: stdlib", "source: intrinsic")
would slot in cleanly with the diagnostic-note infrastructure used by other
families. Not required for this slice.

### 3. Nit — registry templates and emitted messages don't match literally

The registry's `message_template` for these codes
([crates/sifr_diagnostics/src/codes.rs:507–572](crates/sifr_diagnostics/src/codes.rs:507))
omits the surrounding single quotes that the helpers emit:

| Code | Registry template | Emitted (helper) |
| --- | --- | --- |
| `SIFR-NAME-0001` | `undefined variable: {name}` | `undefined variable: '<name>'` |
| `SIFR-NAME-0002` | `undefined function: {name}` | `undefined function: '<name>'` |
| `SIFR-NAME-0004` | `member {member} does not exist on {container}` | `module '<container>' has no member '<member>'` |
| `SIFR-IMPORT-0001` | `cannot import forbidden intrinsic module {module}` | `cannot import from '<module>' — _sifr.* modules are internal compiler intrinsics` |
| `SIFR-IMPORT-0002` | `unknown import target: {module}` | `unknown import target: '<module>'` |

The HIR-side `LoweringError` carries a free-form `String`, so the registry
templates are documentation/render-side metadata at present and are not
enforced against runtime messages — this is the same posture as prior slices
(MATCH had the same drift; see the
[match-diagnostics review pass 1](semantic-diagnostic-code-taxonomy-diag-4a-match-diagnostics-review-pass-1.md)
for the analogous note). Worth flagging since two of the templates
(`SIFR-NAME-0004` and `SIFR-IMPORT-0001`) drift not just in punctuation but in
wording, and the slice is a natural opportunity to tighten them. Non-blocking.

### 4. Test coverage gaps in the unit tests

[`name_import_diagnostics_tests.rs`](crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs)
adds one test per code, which pins both the message and the code for each
helper. Each helper, however, is invoked from multiple call sites in
`lower_module_impl`:

- `name_diagnostics::missing_member` is called from three paths — intrinsic
  ([mod.rs:807](crates/sifr_hir/src/lower/mod.rs:807)), stdlib
  ([mod.rs:967](crates/sifr_hir/src/lower/mod.rs:967)), and local
  ([mod.rs:1105](crates/sifr_hir/src/lower/mod.rs:1105)). The unit test only
  exercises the local path. Stdlib is covered by
  [stdlib_invalid_import.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_import.sifr)
  and [stdlib_missing_function.sifr](crates/sifr/tests/e2e/fail/stdlib_missing_function.sifr);
  the intrinsic path has no direct fixture but is reachable only from stdlib
  `.sifr` files (via `allow_intrinsic_imports`), which is internal-only.
- `import_diagnostics::unknown_import_target` is called from three paths —
  intrinsic ([mod.rs:817](crates/sifr_hir/src/lower/mod.rs:817)), stdlib
  ([mod.rs:978](crates/sifr_hir/src/lower/mod.rs:978)), and local
  ([mod.rs:987](crates/sifr_hir/src/lower/mod.rs:987)). The unit test exercises
  the local path; e2e covers the stdlib path
  ([stdlib_invalid_module.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr))
  and the local path
  ([import_nonexistent_local.sifr](crates/sifr/tests/e2e/fail/import_nonexistent_local.sifr)).
  The intrinsic-unknown path has no test.

Coverage by call site is acceptable for this slice — the helper is the unit
under test and is exercised at least once. Adding a unit test for the stdlib
context-manager path (so the SIFR-NAME-0004 stdlib branch has a fast fixtureless
test) would harden the suite, but it is not required.

### 5. Other unrelated `lower_module_impl` import errors are still raw

The slice intentionally migrates exactly the five codes scoped in the task
description. The same `lower_module_impl` body still calls
`ctx.error(format!(...))` directly for several import-adjacent diagnostics
that do not yet have active codes:

- `unsupported relative import level <N> for module '<X>'` ([mod.rs:741](crates/sifr_hir/src/lower/mod.rs:741))
- `unsupported bare relative import; use 'from <module> import ...'` ([mod.rs:748](crates/sifr_hir/src/lower/mod.rs:748))
- `cannot import private name '<name>' from module '<X>'` ([mod.rs:996](crates/sifr_hir/src/lower/mod.rs:996))
- `unsupported import statement 'import <X>'; use 'from <X> import <name>'` ([mod.rs:1117](crates/sifr_hir/src/lower/mod.rs:1117))

These are out of scope for slice 2b.19 — flagging here so the next slice in
the import family knows where to start. None block this PR.

### 6. Issue tracker update is correct

The slice 2b.18 entry is correctly flipped from `[ ]` to `[x]` and now
references PR #1690; slice 2b.19 is added as `[ ]` "in progress" with
`PR: pending`. Format matches surrounding slices. Good.

### 7. Module registration and naming

[crates/sifr_hir/src/lower/mod.rs:39](crates/sifr_hir/src/lower/mod.rs:39)
inserts `import_diagnostics` and
[:50](crates/sifr_hir/src/lower/mod.rs:50) inserts `name_diagnostics` in the
correct alphabetical positions. The `#[cfg(test)] mod
name_import_diagnostics_tests;` registration directly follows `name_diagnostics`,
mirroring the `match_diagnostics` / `match_diagnostics_tests` pairing.

The single combined `name_import_diagnostics_tests.rs` covers both helper
modules, which is a small deviation from the existing 1:1 convention
(`match_diagnostics_tests.rs` ↔ `match_diagnostics.rs`). Defensible — the
tests share infrastructure (`lower_errors` helper, parser setup) and are
narrowly scoped — but worth noting in case a strict 1:1 rule is preferred for
the diagnostic family layout. Non-blocking.

### 8. Pre-existing fixture quirk: `enum_invalid_variant.sifr`

[enum_invalid_variant.sifr](crates/sifr/tests/e2e/fail/enum_invalid_variant.sifr)
asserts `undefined variable: 'Color'` — but `Color` is in fact defined in the
fixture (the actual problem is the missing `YELLOW` variant). The slice is
faithfully rekeying the existing message-and-code from `SIFR-TYPE-0001` to
`SIFR-NAME-0001`, so this is not a regression caused by 2b.19. Worth filing
as a follow-up: the diagnostic should ideally surface
`enum 'Color' has no variant 'YELLOW'`, not pin the blame on `Color`. Out of
scope here.

## Validation

I re-ran the pieces most likely to be sensitive to this slice:

- ✅ `cargo test -p sifr_hir name_import_diagnostics_tests` — passes (5/5).
- ✅ Spot-checked unit tests for each migrated call site in
  `expressions_tests.rs` / `nested_function_tests.rs` for "undefined
  variable" / "undefined function" assertions; all retain matching messages.
- ❌ `cargo test -p sifr_driver` — 1 failure, captured in
  [item 1](#1-blocker-sifr_driver-test-still-asserts-on-the-old-unknown-module-message).
- Two pre-existing failures in
  `lower::expressions_tests::test_empty_dict_literal_conflicting_write_reports_deterministic_error`
  and
  `lower::expressions_tests::test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`
  reproduce on `367b10a8` (the merge base) — **not caused by this slice**.

The validation log in the task description omitted `cargo test -p sifr_driver`
(or `--lib`). Strongly recommend wiring this into the local validation step,
or running `scripts/run_all_tests.sh --profile quick` per `AGENTS.md`.

## Recommendation

Not ready for PR until [item 1](#1-blocker-sifr_driver-test-still-asserts-on-the-old-unknown-module-message)
is fixed and `cargo test -p sifr_driver --lib` passes locally. Items 2–8 are
informational; the slice author can choose to address any subset of them in
this PR or defer to a follow-up.

After the `sifr_driver` test is updated, the slice is otherwise
correctly scoped, faithful to the migration patterns set by RESULT, OWN, FLOW,
and MATCH slices, and free of remaining raw call sites for the five codes it
claims.
