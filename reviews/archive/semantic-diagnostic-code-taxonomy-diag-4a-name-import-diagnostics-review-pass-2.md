## Review — `milestone_diag_4a` slice 2b.19 (name and import diagnostics) — pass 2

Branch: `codex/semantic-diagnostics-diag-4a-name-import-diagnostics`
Scope: confirm the pass-1 blocker is resolved and re-audit the slice for any
remaining blockers, behavioral regressions, or taxonomy inconsistencies before
PR.

Review style: read-only audit, no files modified.

## TL;DR

The pass-1 blocker is fixed and the registry/doc drift called out as
non-blocking in pass 1 has also been tightened. Three of the four
non-blocking items from pass 1 are now resolved or actively improved:

| Pass-1 item | Status in pass 2 |
| --- | --- |
| 1 — `sifr_driver` test asserted on old "unknown module" message | **Fixed** ([project_graph.rs:383–386](crates/sifr_driver/src/tests/project_graph.rs:383)). Strengthened to also assert `code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)`. |
| 2 — three import miss messages collapsed into one | Unchanged. Intentional taxonomy choice (registry already carried the consolidated template); no follow-up needed. |
| 3 — registry templates drifted from emitted messages | **Fixed** for all five codes ([codes.rs:507–572](crates/sifr_diagnostics/src/codes.rs:507)) and propagated to `docs/errors/SIFR-{NAME,IMPORT}-*.md` and `internal_docs/diagnostic_codes.md`. |
| 4 — call-site coverage gaps (stdlib `missing_member`, intrinsic `unknown_import_target`) | Unchanged. Stdlib `missing_member` still has e2e coverage only; intrinsic `unknown_import_target` still has no test. Non-blocking — intrinsic path is reachable only from stdlib `.sifr` sources with `allow_intrinsic_imports`. |
| 5 — unrelated raw `lower_module_impl` import errors | Unchanged. Out of scope for slice 2b.19. |
| 6 — issue tracker entry | Unchanged. Already correct in pass 1. |
| 7 — combined `name_import_diagnostics_tests.rs` deviating from 1:1 convention | Unchanged. Defensible — flagged for awareness only. |
| 8 — pre-existing `enum_invalid_variant.sifr` quirk | Unchanged. Pre-existing; not introduced by this slice. |

The pass-1 nit about the SIFR-NAME-0002 representative fixture
(`stdlib_invalid_module.sifr` did not in fact trigger SIFR-NAME-0002 after the
migration) is also addressed by the new
[`undefined_function.sifr`](crates/sifr/tests/e2e/fail/undefined_function.sifr)
fixture, which is a clean three-line demonstration of the code.

**Recommendation: ready for PR.** No remaining blockers. Optional follow-ups
listed below are not gating.

## What was reviewed in pass 2

- Pass-1 blocker fix: [crates/sifr_driver/src/tests/project_graph.rs](crates/sifr_driver/src/tests/project_graph.rs:354)
- Registry and doc alignment:
  - [crates/sifr_diagnostics/src/codes.rs:507–572](crates/sifr_diagnostics/src/codes.rs:507)
  - [docs/errors/SIFR-NAME-0001.md](docs/errors/SIFR-NAME-0001.md), [-0002](docs/errors/SIFR-NAME-0002.md), [-0004](docs/errors/SIFR-NAME-0004.md)
  - [docs/errors/SIFR-IMPORT-0001.md](docs/errors/SIFR-IMPORT-0001.md), [-0002](docs/errors/SIFR-IMPORT-0002.md)
  - [internal_docs/diagnostic_codes.md:72–77](internal_docs/diagnostic_codes.md:72)
- New fixture: [crates/sifr/tests/e2e/fail/undefined_function.sifr](crates/sifr/tests/e2e/fail/undefined_function.sifr)
- Re-confirmed migrated helpers and call sites unchanged since pass 1:
  - [crates/sifr_hir/src/lower/name_diagnostics.rs](crates/sifr_hir/src/lower/name_diagnostics.rs)
  - [crates/sifr_hir/src/lower/import_diagnostics.rs](crates/sifr_hir/src/lower/import_diagnostics.rs)
  - [crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs](crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs)
  - Migrated call sites in [aug_assign_lowering.rs:299](crates/sifr_hir/src/lower/aug_assign_lowering.rs:299), [expressions.rs:248](crates/sifr_hir/src/lower/expressions.rs:248), [expressions.rs:1718](crates/sifr_hir/src/lower/expressions.rs:1718), [statements.rs:1542](crates/sifr_hir/src/lower/statements.rs:1542), [tuple_unpack.rs:102](crates/sifr_hir/src/lower/tuple_unpack.rs:102), and `lower_module_impl` ([mod.rs:795](crates/sifr_hir/src/lower/mod.rs:795), [:807](crates/sifr_hir/src/lower/mod.rs:807), [:817](crates/sifr_hir/src/lower/mod.rs:817), [:967](crates/sifr_hir/src/lower/mod.rs:967), [:978](crates/sifr_hir/src/lower/mod.rs:978), [:987](crates/sifr_hir/src/lower/mod.rs:987), [:1105](crates/sifr_hir/src/lower/mod.rs:1105))
- Issue tracker: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53–54](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53)

## Findings

### 1. Pass-1 blocker is fixed and strengthened

[crates/sifr_driver/src/tests/project_graph.rs:383–386](crates/sifr_driver/src/tests/project_graph.rs:383)
now asserts:

```rust
assert!(errors.iter().any(|e| {
    e.message.contains("unknown import target: 'missing_mod'")
        && e.code == Some(DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE)
}));
```

This is the exact fix recommended in pass 1 (item 1), and the additional code
check is the bonus strengthening that pass 1 suggested. Confirmed by running:

```
cargo test -p sifr_driver --lib test_collect_project_modules_reports_unknown_module_in_non_main
test result: ok. 1 passed; 0 failed; ...
```

The test name (`..._reports_unknown_module_...`) and the surrounding
docstring still say "unknown module" — that is a stylistic choice; the test
behavior is correct. Not blocking.

### 2. Registry and emitted message templates now match for all five codes

The registry, docs, and emitted helper messages are now literally aligned:

| Code | Registry template ([codes.rs:507–572](crates/sifr_diagnostics/src/codes.rs:507)) | Emitted (helper) |
| --- | --- | --- |
| `SIFR-NAME-0001` | `undefined variable: '{name}'` | `undefined variable: '<name>'` |
| `SIFR-NAME-0002` | `undefined function: '{name}'` | `undefined function: '<name>'` |
| `SIFR-NAME-0004` | `module '{container}' has no member '{member}'` | `module '<container>' has no member '<member>'` |
| `SIFR-IMPORT-0001` | `cannot import from '{module}' — _sifr.* modules are internal compiler intrinsics` | `cannot import from '<module>' — _sifr.* modules are internal compiler intrinsics` |
| `SIFR-IMPORT-0002` | `unknown import target: '{module}'` | `unknown import target: '<module>'` |

The corresponding `docs/errors/SIFR-{NAME,IMPORT}-*.md` "Message template"
rows match the registry (verified with
`cargo run -q -p sifr_diagnostics --bin gen-error-docs`, which produced no
further diff against the working tree). `internal_docs/diagnostic_codes.md`
([:72–77](internal_docs/diagnostic_codes.md:72)) also matches.

This addresses pass-1 item 3 in full. Templates with embedded interpolation
markers can never be perfectly enforced against the runtime `format!` strings
(the HIR-side `LoweringError.message` remains a free-form `String`), but the
registry now reflects exactly what users will see — the precondition for
ever wiring up an enforced check later.

### 3. New SIFR-NAME-0002 representative fixture

Pass 1 noted the registry pointed `SIFR-NAME-0002` at
`crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr`, which after the
slice's re-keying actually emits `SIFR-IMPORT-0002`, not `SIFR-NAME-0002`. The
new fixture is correct and minimal:

[crates/sifr/tests/e2e/fail/undefined_function.sifr](crates/sifr/tests/e2e/fail/undefined_function.sifr):

```python
# expect-error: SIFR-NAME-0002: undefined function: 'foo'
def main():
    foo()
```

Registry now points here ([codes.rs:523](crates/sifr_diagnostics/src/codes.rs:523)),
and `docs/errors/SIFR-NAME-0002.md` agrees. The fixture is exercised by
`test_e2e_fail`, which is green.

### 4. Validation re-run

I re-ran the same suite the slice author ran, plus the canonical
`AGENTS.md`-mandated `sifr_driver` lib test, against the working tree:

| Command | Result |
| --- | --- |
| `cargo run -q -p sifr_diagnostics --bin gen-error-docs` | passes; no further diff |
| `cargo fmt --check` | clean |
| `python3 scripts/check_diagnostic_docs_sync.py` | exit 0 |
| `python3 scripts/check_diagnostic_schema_sync.py` | exit 0 |
| `python3 scripts/check_hir_maintainability_guardrails.py` | `HIR maintainability guardrails: PASS` |
| `cargo test -p sifr_hir name_import_diagnostics_tests` | 5/5 |
| `cargo test -p sifr_driver --lib test_collect_project_modules_reports_unknown_module_in_non_main` | 1/1 |
| `cargo test -p sifr_driver --lib` | 100/100 |
| `cargo test -p sifr_diagnostics --lib --tests` | 31/31 |
| `cargo test -p sifr --test e2e -- test_e2e_fail` | 1/1 (25 fixtures filtered in) |
| `cargo test -p sifr -- --skip test_e2e_pass` | passes |
| `cargo clippy --workspace -- -D warnings` | clean |

The two `sifr_hir` lib failures
(`test_empty_dict_literal_conflicting_write_reports_deterministic_error`,
`test_empty_list_specialization_optional_append_in_loop_rejects_return_annotation`)
that pass 1 already classified as pre-existing reproduce on `367b10a8` (the
merge base) **and** on the working tree, with the working tree adding 5 new
passing tests on top — i.e. the delta is purely additive, the failures are
not caused or exacerbated by this slice. Out of scope.

### 5. No remaining raw call sites for the migrated diagnostics

`grep` for the pre-migration messages across the repo:

- `unknown intrinsic module` — 0 hits
- `unknown stdlib module` — 0 hits
- `unknown module '` — 0 hits in production code
- `intrinsic module .* has no member` — 0 hits
- `member .* does not exist on` — 0 hits
- `cannot import forbidden intrinsic module` — 0 hits

The only remaining occurrences of the new messages outside helper files are
in the colocated and existing unit tests (`name_import_diagnostics_tests.rs`,
`nested_function_tests.rs`, `expressions_tests.rs`), and in the new
`project_graph.rs` assertion. All are intentional.

### 6. Non-blocking observations carried over from pass 1

These are unchanged in pass 2 and remain non-blocking. Listing for the PR
description's awareness only:

- **Coverage holes (item 4 from pass 1).** The unit test for
  `name_diagnostics::missing_member` exercises the local-module path; stdlib
  is covered via e2e
  ([stdlib_invalid_import.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_import.sifr),
  [stdlib_missing_function.sifr](crates/sifr/tests/e2e/fail/stdlib_missing_function.sifr));
  the intrinsic missing-member branch ([mod.rs:807](crates/sifr_hir/src/lower/mod.rs:807))
  has no direct test. The unit test for
  `import_diagnostics::unknown_import_target` exercises the local path; the
  stdlib path is covered via
  [stdlib_invalid_module.sifr](crates/sifr/tests/e2e/fail/stdlib_invalid_module.sifr);
  the intrinsic-unknown branch ([mod.rs:817](crates/sifr_hir/src/lower/mod.rs:817))
  has no test. Both intrinsic branches require `allow_intrinsic_imports` and
  are reachable only from stdlib `.sifr` files (compiler-internal callers),
  which is why fixtureless coverage is awkward. Acceptable for this slice.

- **Unrelated raw call sites in `lower_module_impl` (item 5 from pass 1).**
  Still present at [mod.rs:741](crates/sifr_hir/src/lower/mod.rs:741),
  [:748](crates/sifr_hir/src/lower/mod.rs:748),
  [:996](crates/sifr_hir/src/lower/mod.rs:996),
  [:1117](crates/sifr_hir/src/lower/mod.rs:1117). Out of scope; flagging for
  the next import-family slice.

- **Combined `name_import_diagnostics_tests.rs` (item 7 from pass 1).** Still
  a defensible deviation from the 1:1 convention used by
  `match_diagnostics_tests.rs ↔ match_diagnostics.rs`. Not blocking.

- **`enum_invalid_variant.sifr` quirk (item 8 from pass 1).** The fixture
  asserts `undefined variable: 'Color'` even though `Color` is defined; the
  underlying issue is the missing `YELLOW` variant. The slice faithfully
  re-keys the existing message-and-code; the misleading wording is
  pre-existing. Worth filing as a separate diagnostic-quality follow-up.

### 7. Issue tracker is consistent

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53–54](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:53)
correctly flips slice 2b.18 to merged (PR #1690) and adds slice 2b.19 as
in-progress with `PR: pending`. The format mirrors surrounding entries.

## Optional follow-ups (do in this PR or punt)

None of these block the PR; pick any subset, or punt all to a follow-up
slice:

1. Update the test name and docstring at
   [project_graph.rs:354](crates/sifr_driver/src/tests/project_graph.rs:354)
   to reflect the new "unknown import target" wording — the assertion is
   correct, only the surrounding text still says "unknown module".
2. Add a unit test for the stdlib `name_diagnostics::missing_member` branch
   reachable from the context-manager path (closes the last unit-level gap
   for SIFR-NAME-0004).
3. File a separate issue to revisit `enum_invalid_variant.sifr` — the
   diagnostic should ideally pin the missing variant, not the (defined) enum
   class. Out of scope here.

## Recommendation

**Ready for PR.** Pass-1 blocker is fixed, registry/template alignment is
tightened beyond what pass 1 asked for, and the SIFR-NAME-0002 representative
fixture is now a clean dedicated case rather than a shared file that no
longer triggers the code. The full local validation suite is green, the
clippy gate is clean, and no remaining raw call sites exist for the five
migrated codes. The slice is faithful to the migration patterns set by the
RESULT, OWN, FLOW, and MATCH slices and ready to land.
