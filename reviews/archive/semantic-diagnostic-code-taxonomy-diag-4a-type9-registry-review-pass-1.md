# `milestone_diag_4a` slice 2b.6 — `SIFR-TYPE-0009` registry hygiene — review pass 1

## Scope under review

- Branch: `codex/semantic-diagnostics-diag-4a-type9-registry`
- Target: tighten the active `SIFR-TYPE-0009` registry entry so that its `representative_fixture_path`, `message_template`, `declared_args`, and `dedupe_args` align with the canonical tuple-unpack arity emission at [tuple_unpack.rs:64](../crates/sifr_hir/src/lower/tuple_unpack.rs:64) after slices 2b.4 and 2b.5 fully migrated the unpack/shape sites onto this code. Generated docs are refreshed via `gen-error-docs`; no compiler emission behavior is touched.
- Files changed:
  - [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs:647) — registry entry updated (fixture, template, declared/dedupe args).
  - [docs/errors/SIFR-TYPE-0009.md](../docs/errors/SIFR-TYPE-0009.md) — regenerated public error page.
  - [internal_docs/diagnostic_codes.md](../internal_docs/diagnostic_codes.md:85) — refreshed registry digest table row.
  - [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:40) — slice 2b.5 marked merged with PR #1677, slice 2b.6 marked started.
- Validation already executed by the implementer: `cargo run -q -p sifr_diagnostics --bin gen-error-docs`, `cargo fmt --check`, `python3 scripts/check_diagnostic_docs_sync.py`, `python3 scripts/check_diagnostic_schema_sync.py`, `cargo test -p sifr_diagnostics`, `cargo clippy --workspace -- -D warnings`, `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`, `wall_time=101.38s`).

## Findings

### F1 — Representative fixture pointer is now genuinely on-code (closes slice 2b.5 R3)

The previous registry entry advertised `crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr`. That fixture's `expect-error` line at [tuple_dynamic_list_shape.sifr:2](../crates/sifr/tests/e2e/fail/tuple_dynamic_list_shape.sifr:2) actually pins `SIFR-TYPE-0001` (the `tuple()`-constructor TypeCheck-bridge diagnostic), not `SIFR-TYPE-0009` — so the registry's "representative fixture" was previously mis-aimed and would not have demonstrated the active code at all. The new pointer `crates/sifr/tests/e2e/fail/tuple_unpack_shape_mismatch.sifr` ([tuple_unpack_shape_mismatch.sifr:1](../crates/sifr/tests/e2e/fail/tuple_unpack_shape_mismatch.sifr:1)) declares `# expect-error: SIFR-TYPE-0009: tuple unpacking: expected 2 values, got 3` and is the slice 2b.4 fixture for the canonical arity-mismatch path through [tuple_unpack.rs:62-72](../crates/sifr_hir/src/lower/tuple_unpack.rs:62), so the e2e harness's joint code+substring contract at [crates/sifr/tests/e2e.rs:2561](../crates/sifr/tests/e2e.rs:2561) confirms the fixture genuinely exercises this code. Slice 2b.5's R3 ("registry's own canonical-fixture pointer is currently mis-aimed for `SIFR-TYPE-0009`") is now resolved.

### F2 — `message_template` now matches the canonical emission verbatim (closes slice 2b.5 R1 for the chosen representative)

The previous template `cannot unpack {actual_count} value(s) into {expected_count} target(s)` did not match any of the five live `TYPE_UNPACK_SHAPE_MISMATCH` emission sites; it was a synthetic stand-in that drifted from rendered text. The new template `tuple unpacking: expected {expected_count} values, got {actual_count}` is character-for-character identical to the format string at [tuple_unpack.rs:66-70](../crates/sifr_hir/src/lower/tuple_unpack.rs:66) once the placeholders are substituted (`expected_count` ← `targets.len()`, `actual_count` ← `elems.len()`). The placeholder order in the template (`{expected_count}` first, then `{actual_count}`) also matches the order in the format string's argument list. This satisfies the design rule at [issue:430](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:430) that `message_template` is the stable grouping key and "must not contain dynamic identifiers, type names, counts, paths, or literal values" — the literal counts are placeholders, not concrete values.

### F3 — `declared_args` / `dedupe_args` reordered to match the new template

`declared_args` and `dedupe_args` were both flipped from `[actual_count, expected_count]` to `[expected_count, actual_count]`, which now matches the placeholder order of the new template. Both args remain `arg!` (i.e., `MessageAndJson`) which is the right format choice: counts are user-facing in the rendered message and stable enough to expose in the JSON envelope. The dedupe-args list mirroring the declared-args list also keeps `(severity, code, message_template, primary display file)` and `(code, message_template, primary SourceSpan range, dedupe args)` ([issue:676](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:676), [issue:682](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:682)) coherent for the canonical site; nothing that the bridge currently surfaces depends on the old order, so this is a metadata-only swap.

### F4 — Choice of canonical site among five is defensible

`SIFR-TYPE-0009` carries five distinct format strings across five emission sites:

1. Tuple-unpack arity (canonical) — [tuple_unpack.rs:66](../crates/sifr_hir/src/lower/tuple_unpack.rs:66): `"tuple unpacking: expected {} values, got {}"`.
2. Tuple-unpack non-tuple — [tuple_unpack.rs:78](../crates/sifr_hir/src/lower/tuple_unpack.rs:78): `"cannot unpack non-tuple type '{}'"`.
3. Star-unpack non-list — [tuple_unpack.rs:171](../crates/sifr_hir/src/lower/tuple_unpack.rs:171): `"star unpacking requires a list type"`.
4. For-loop tuple arity — [statements.rs:2107](../crates/sifr_hir/src/lower/statements.rs:2107): `"for loop tuple target expects {} element(s), iterable yields {}"`.
5. For-loop tuple non-tuple element — [statements.rs:2123](../crates/sifr_hir/src/lower/statements.rs:2123): `"for loop tuple target expects iterable elements of tuple type, got '{}'"`.

The slice picks site #1 as canonical. That is the right choice for three reasons:

- Site #1 is owned by the module the registry already names (`sifr_hir::lower::tuple_unpack`); sites #4 and #5 live in `sifr_hir::lower::statements`, and slice 2b.5 confirmed the for-loop tuple-target paths are conceptually a tuple-unpack carrier. Choosing the namesake module's primary site keeps `owner_module` semantically tight.
- Site #1 is the closest analog of the family summary `"Tuple or list unpacking shape mismatch."` — it is the canonical shape mismatch (count vs. count), whereas sites #2 and #3 are *kind* mismatches (non-tuple, non-list) and sites #4 and #5 are an iterator-element overlay of the same idea.
- Site #1 has the cleanest two-arg shape (`expected_count`, `actual_count`) that maps to the design spec's expected/actual idiom used by `SIFR-TYPE-0002` ([codes.rs:78](../crates/sifr_diagnostics/src/codes.rs:78)) and `SIFR-TYPE-0008`, so the registry stays consistent with the rest of the TYPE family.

The drift between this canonical template and the four non-canonical sites (R1 below) is a structural property carried over from earlier slices — it is narrower after this slice (the canonical site is now exact), not wider.

### F5 — Generated docs faithfully reflect the registry edit

`docs/errors/SIFR-TYPE-0009.md` was regenerated by `gen-error-docs`; its rows mirror the registry exactly:

- Owner: `sifr_hir::lower::tuple_unpack` (unchanged).
- Message template: `tuple unpacking: expected {expected_count} values, got {actual_count}` (matches [codes.rs:653](../crates/sifr_diagnostics/src/codes.rs:653)).
- Representative fixture: `crates/sifr/tests/e2e/fail/tuple_unpack_shape_mismatch.sifr` (matches [codes.rs:652](../crates/sifr_diagnostics/src/codes.rs:652)).
- Declared args: `expected_count (message+json), actual_count (message+json)` (matches [codes.rs:655](../crates/sifr_diagnostics/src/codes.rs:655)).
- Dedupe args: `expected_count, actual_count` (matches [codes.rs:656](../crates/sifr_diagnostics/src/codes.rs:656)).

`internal_docs/diagnostic_codes.md` row 85 ([diagnostic_codes.md:85](../internal_docs/diagnostic_codes.md:85)) now carries the same five fields. `docs/errors/diagnostic-codes.md` has no diff in this slice — that file only carries the family/severity/summary, none of which changed. `docs/schemas/diagnostics.schema.json` likewise has no diff (the schema describes envelope shape, not per-code metadata). Both `check_diagnostic_docs_sync.py` and `check_diagnostic_schema_sync.py` are listed as having passed locally, which is consistent with this set.

### F6 — No compiler emission changes; correctly so

Slice scope is registry hygiene. Inspecting the four emission sites (#2–#5 from F4) shows their format strings, `error_with_code` calls, and surrounding control flow are unchanged on this branch — only `crates/sifr_diagnostics/src/codes.rs` is touched in the compiler tree. That is the right call:

- The current emission API at [mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228) takes a pre-formatted `String`, so a registry-side template change is invisible to runtime emission until the slice that migrates emission to the `DiagnosticBuilder` placeholder pipeline at [issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432) lands. Touching emission in this slice would conflate registry hygiene with the larger builder migration and bloat the diff.
- The five fixtures pinning the five emission sites (`tuple_unpack_shape_mismatch.sifr`, `tuple_unpack_non_tuple_shape_mismatch.sifr`, `star_unpack_requires_list_type.sifr`, `for_loop_tuple_target_arity_mismatch.sifr`, `for_loop_tuple_target_non_tuple_element.sifr`) all assert on substrings of the rendered message — none of them would have changed expectations from a registry-only edit, so leaving emission untouched preserves the e2e contract by construction.

The validation block confirms `scripts/run_all_tests.sh --profile quick` passed at `report_signature=e1bf653aaa770517` (the same signature reported on slices 2b.3, 2b.4, and 2b.5), supporting the no-runtime-effect claim.

### F7 — Issue checklist correctly transitioned

[issues/...:40](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:40) flips slice 2b.5 from "Started" with implementation-complete language to `[x] merged ... PR: ...pull/1677`, and adds line 41 marking slice 2b.6 as "Started" with the registry-hygiene scope. This matches the branch name (`codex/semantic-diagnostics-diag-4a-type9-registry`) and the changed-file set. No reviewer-satisfied flag is added for slice 2b.6, which is consistent with this being the first review pass.

### F8 — Diff is tightly scoped; nothing unrelated touched

`git status` shows exactly four files modified in this slice and no untracked paths: the registry, the regenerated public docs page, the regenerated internal digest, and the issue checklist. No fixtures were re-keyed, no emission sites were edited, no schema/baselines were perturbed, and no neighboring registry entries were drive-by'd. The slice respects the milestone's "no unrelated cleanup" guidance.

## Residual risks

### R1 — Four non-canonical `SIFR-TYPE-0009` emission sites still drift from `message_template`

Sites #2–#5 from F4 still render ad-hoc strings that do not share placeholder shape with the canonical template `tuple unpacking: expected {expected_count} values, got {actual_count}`. Once the builder pipeline at [issue:432](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:432) is adopted (`message` rendered from `message_template` + scalar args, no pre-formatted strings), one of three follow-ups is required:

- Split `SIFR-TYPE-0009` into per-shape codes (e.g., a separate code for non-tuple, star non-list, for-loop arity, for-loop non-tuple), each with its own template; or
- Generalize the template to a kind-tagged form like `{kind} unpacking shape mismatch: expected {expected}, got {actual}` and re-shape the four non-canonical sites to that arg surface; or
- Accept divergent rendered text under one canonical grouping key and document the trade-off (this would weaken the equivalence between `message_template` and rendered text that compact grouping at [issue:687](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:687) relies on).

This slice is correctly *narrower* than slice 2b.5's R1 — the canonical site no longer drifts — but the question of how to align the four remaining sites is deferred. Non-blocking for slice 2b.6's stated scope; should be planned as part of the emission-builder migration slice.

### R2 — `declared_args` are not yet exercised at runtime

The current `error_with_code(code, message: String)` API at [mod.rs:228](../crates/sifr_hir/src/lower/mod.rs:228) does not pass scalar arg values into the diagnostic envelope, so the new `declared_args` (`expected_count`, `actual_count`) are documentation-only at this state. JSON consumers won't see the counts as structured fields until emission migrates to `DiagnosticBuilder::message_template(...).arg(...)`. There is no test today that would catch a silent rename of `declared_args` (e.g., flipping `expected_count` to `expected` or vice versa), since neither the e2e harness nor the unit tests assert structured arg presence for this code. A future builder-migration slice will surface this; for slice 2b.6 it is acknowledged but non-blocking.

### R3 — Registry `summary` still says "Tuple or list unpacking shape mismatch."

The summary `"Tuple or list unpacking shape mismatch."` at [codes.rs:650](../crates/sifr_diagnostics/src/codes.rs:650) is broader than the chosen canonical template, which only addresses tuple unpack arity. The summary correctly covers all five emission sites (tuple, list/star, for-loop), but a reader who reads only the registry row may infer that the template is the universal rendered form, when in fact four of five sites render a different shape. Mitigation options if R1 is resolved by splitting codes: tighten this summary to `"Tuple unpacking arity mismatch."` once the non-arity sites move to their own codes. Non-blocking.

### R4 — No registry-level test guards `message_template` ↔ owner-module emission alignment

Nothing in `cargo test -p sifr_diagnostics` cross-checks that the `message_template` for `SIFR-TYPE-0009` matches a format string actually emitted by `sifr_hir::lower::tuple_unpack`. A future regression that edited [tuple_unpack.rs:66](../crates/sifr_hir/src/lower/tuple_unpack.rs:66) (e.g., re-wording to `"tuple unpacking expected {} values, but got {}"`) would silently re-introduce drift between the registry and the canonical site without any test failing — only a manual `gen-error-docs`+diff would catch it. The fixture `tuple_unpack_shape_mismatch.sifr` would still pass because it asserts on a substring (`"tuple unpacking: expected 2 values, got 3"`) that incidentally survives most rewordings. Out of scope for this slice; worth recording as a hygiene check that `scripts/check_diagnostic_code_coverage.py` (planned for `milestone_diag_11`, [issue:1236](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1236)) could absorb.

## Verdict

Satisfied / no blocking findings. Slice 2b.6 does exactly what its scope claims: it repoints the `SIFR-TYPE-0009` representative fixture from a fixture that emits a *different* code (`tuple_dynamic_list_shape.sifr` → `SIFR-TYPE-0001`) to one that genuinely exercises this code (`tuple_unpack_shape_mismatch.sifr` → `SIFR-TYPE-0009`), aligns `message_template` with the canonical [tuple_unpack.rs:66](../crates/sifr_hir/src/lower/tuple_unpack.rs:66) emission verbatim, reorders `declared_args` and `dedupe_args` to follow the new placeholder order, and refreshes the regenerated public/internal docs without touching emission, fixtures, baselines, or the schema. R3 and R1 from slice 2b.5's pass-1 review are now closed at the canonical site; the remaining drift among the four non-canonical sites (R1 above) and the runtime non-use of `declared_args` (R2 above) are correctly carried forward to the future emission-builder migration slice. No compiler emission change is required by, or appropriate to, this slice.
