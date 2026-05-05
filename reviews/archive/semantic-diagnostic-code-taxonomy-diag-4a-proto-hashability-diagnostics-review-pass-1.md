# `milestone_diag_4a` slice 2b.27 — Protocol hashability diagnostic migration

Pass 1 review of the uncommitted working tree on branch
`codex/semantic-diagnostics-diag-4a-protocol-container-diagnostics`.

## Scope under review

- Mark slice 2b.26 merged in the issue tracker after [sifr-lang/sifr#1698](https://github.com/sifr-lang/sifr/pull/1698) and add the in-progress entry for slice 2b.27.
- Migrate the `hash()` hashability diagnostic from the generic `SIFR-TYPE-0001` bridge to active `SIFR-PROTO-0004`.
- Align the `SIFR-PROTO-0004` registry template / args / owner / representative fixture / generated docs with the emitted message.
- Re-key [crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr](crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr).
- Add focused HIR unit coverage asserting the exact message and exact `DiagnosticCode`.

## Verdict

**Approved — reviewer-satisfied for PR.** Behaviour, registry, fixture, generated docs, internal docs, issue tracker, and HIR unit coverage all line up. No correctness, regression, or alignment blockers were found. Two strictly out-of-scope follow-ups are flagged at the bottom for the next slice; neither blocks this PR.

## What I checked

### 1. HIR call site migration
[crates/sifr_hir/src/lower/expressions.rs:812](crates/sifr_hir/src/lower/expressions.rs:812)

- `lower_call`'s `hash()` special case now emits via
  `ctx.error_with_code(DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED, ...)` instead of the bare `ctx.error(...)`. The call-site refactor preserves the message verbatim — `"hash() argument must be hashable, got '{type_name}'"` — by binding `let type_name = ty.display_name();` before the format and using an inline format-arg, which keeps clippy's `uninlined_format_args` happy.
- The `DiagnosticCode` import is already present at [crates/sifr_hir/src/lower/expressions.rs:56](crates/sifr_hir/src/lower/expressions.rs:56), so no missing/duplicate imports.
- `error_with_code` populates `LoweringError.code = Some(...)` ([crates/sifr_hir/src/lower/mod.rs:237](crates/sifr_hir/src/lower/mod.rs:237)), so the active code surfaces through `compile_errors_to_diagnostics` to the e2e harness's `failure.code` instead of falling through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge. Confirmed against the e2e fail harness logic at [crates/sifr/tests/e2e.rs:572](crates/sifr/tests/e2e.rs:572) and [crates/sifr/tests/e2e.rs:2561](crates/sifr/tests/e2e.rs:2561) — the `failure.code == expected.code` comparison is exactly what the new fixture marker relies on.
- The `PROTO_HASHABLE_OR_COMPARABLE_REQUIRED` constant was already declared at [crates/sifr_diagnostics/src/codes.rs:83](crates/sifr_diagnostics/src/codes.rs:83) and listed in the active codes array at [crates/sifr_diagnostics/src/codes.rs:1402](crates/sifr_diagnostics/src/codes.rs:1402). No new enum/constant plumbing is needed.

### 2. Registry entry alignment
[crates/sifr_diagnostics/src/codes.rs:1024](crates/sifr_diagnostics/src/codes.rs:1024)

The active entry is now coherent with what HIR actually emits:

| Field | Before | After | Emitted at expressions.rs:814 |
| --- | --- | --- | --- |
| Representative fixture | `crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr` (never existed) | `crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr` | n/a |
| Message template | `type {type_name} must satisfy {protocol}` | `hash() argument must be hashable, got '{type_name}'` | matches verbatim |
| Owner | `sifr_hir::lower` | `sifr_hir::lower::expressions` | matches the emitting module |
| Declared args | `[type_name, protocol]` | `[type_name]` | matches |
| Dedupe args | `[type_name, protocol]` | `[type_name]` | matches |

Family code, family description, and severity are unchanged; only message-shape-bound fields moved. Entry indentation, `arg!(...)` shape, and trailing comma all match the surrounding `active_entry!` invocations — no formatting drift.

### 3. Generated docs
[docs/errors/SIFR-PROTO-0004.md](docs/errors/SIFR-PROTO-0004.md), [internal_docs/diagnostic_codes.md:118](internal_docs/diagnostic_codes.md:118)

- Both docs reflect the new template / owner / fixture / args row-for-row, consistent with what `cargo run -q -p sifr_diagnostics --bin gen-error-docs` would regenerate. The `<!-- Generated ... Do not edit by hand. -->` banner is preserved.
- The family-level entry in [docs/errors/diagnostic-codes.md:89](docs/errors/diagnostic-codes.md:89) (`Hashable or comparable protocol is required.`) is unchanged. That description is broader than what is currently enforced (only `hash()` argument hashability) but is forward-compatible with future Comparable enforcement under the same code, and matches the constant name. Not a blocker — see follow-up (B).
- [internal_docs/diagnostic_emission_inventory.md:338](internal_docs/diagnostic_emission_inventory.md:338) was *not* updated and still reads `unhashable/comparable fixtures`. That row is intentionally aspirational across the whole inventory (other rows reference broader surfaces too), and its description does not contradict the new emission. Acceptable to leave for this slice.

### 4. Fixture re-keying
[crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr:1](crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr:1)

- `# expect-error` marker changed from `SIFR-TYPE-0001:` to `SIFR-PROTO-0004:` with the message tail unchanged. No other lines were modified.
- The fixture body still drives the same code path: `m: Measurement = Measurement(3.14); h: int = hash(m)`. `Measurement` has a `float` field which makes `is_hashable_type` return `false` ([crates/sifr_hir/src/lower/classes.rs:1055](crates/sifr_hir/src/lower/classes.rs:1055)) — the diagnostic still fires deterministically.
- I checked that no other fixture file references this code/message: `grep -rn "SIFR-PROTO-0004\|generic_counter_unhashable" crates/ docs/ internal_docs/ issues/` only finds the live SIFR-PROTO-0004 artefacts. The `generic_counter_unhashable` reference at [internal_docs/phases/13_type_system_completion.md:618](internal_docs/phases/13_type_system_completion.md:618) is a planned-but-unimplemented test in the phase plan, not a real fixture, so dropping it from the registry is correct.

### 5. HIR unit coverage
[crates/sifr_hir/src/lower/expressions_tests.rs:307](crates/sifr_hir/src/lower/expressions_tests.rs:307)

- `test_hash_unhashable_argument_has_proto_code` constructs a `Measurement` class with a `float` field and `print(hash(m))` to drive `lower_call`'s hashability gate.
- Asserts both the **exact** message (`"hash() argument must be hashable, got 'Measurement'"`) and the **exact** `DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED` — matching the brief.
- Test placement, `lower_source(...)` helper, and assertion shape mirror the adjacent slice-2b.26 tests (`test_map_callable_arity_mismatch_has_call_code`, etc.), keeping the file consistent.

### 6. Issue-tracker hygiene
[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:61](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:61)

- Slice 2b.26 line flipped from `[ ] ... implementation complete and reviewer-satisfied` to `[x] ... merged ... PR: https://github.com/sifr-lang/sifr/pull/1698.` — wording matches the established merged-line template used by 2b.20 through 2b.25 (same pattern: `merged: ... migration to active <code(s)> with fixture coverage. PR: ...`).
- A new `2b.27 in progress` entry is added on line 62 with the right shape (`hash() hashability protocol diagnostic migration to active SIFR-PROTO-0004 with fixture coverage. PR: pending.`), consistent with how prior in-progress entries were formatted before being flipped to merged.

### 7. Coherence: is `SIFR-PROTO-0004` the right home for `hash()` hashability?

Yes. The PROTO family is split as:

- `SIFR-PROTO-0001` — generic-bound / type-parameter conformance (e.g. `T: Comparable`).
- `SIFR-PROTO-0002` — invalid iterator/reversible protocol signature.
- `SIFR-PROTO-0003` — context-manager protocol missing.
- `SIFR-PROTO-0004` — Hashable / Comparable protocol obligations triggered by callsites that *demand* the protocol (rather than via a generic bound).

`hash(x)` demands Hashable conformance on `x` directly, not via a `T: Hashable` bound, so it doesn't fit `SIFR-PROTO-0001`. Filing it under PROTO-0004 is the right bucket and matches the family description. The constant name `PROTO_HASHABLE_OR_COMPARABLE_REQUIRED` keeps room for an analogous future Comparable surface (e.g. `heapq.heappush` on uncomparable elements) without further code churn.

### 8. Validation surface

The author-listed local validation set (`gen-error-docs`, `cargo fmt --check`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir "hash_unhashable_argument"`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) is the standard slice-2b mix and covers every surface this change touches. No additional gate is missing.

## Concerns / non-blocking follow-ups

- **(A) `hash()` arity error still on the bridge.** The companion error at [crates/sifr_hir/src/lower/expressions.rs:803](crates/sifr_hir/src/lower/expressions.rs:803) (`"hash() takes exactly 1 argument, got N"`) still uses bare `ctx.error(...)` and so flows through `SIFR-TYPE-0001`. That is a `CALL` shape violation — the natural target is `SIFR-CALL-0001` (`CALL_WRONG_POSITIONAL_COUNT`), the same code used by the slice-2b.26 sum-arity migration ([reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-shape-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-call-shape-diagnostics-review-pass-1.md)). Out of scope for this slice (slice scope is specifically the hashability check), but worth filing as the next builtin-arity migration sub-slice.
- **(B) Family description vs. enforced surface.** The PROTO-0004 family description and constant name still mention "comparable", though only Hashable is currently enforced. This is intentional (forward-compatible) and existed before this slice, so no action is required here. If a future slice decides Comparable enforcement should live under a separate code (e.g. `SIFR-PROTO-0005`), the rename can happen there; if it stays here, no change is needed.

Neither follow-up alters the verdict.

## Final word

Implementation is correct, scope is minimal and matches the brief, registry / docs / fixture / unit test all agree on a single code+message, and the migration is coherent with the rest of the PROTO family. **Approved for PR.**
