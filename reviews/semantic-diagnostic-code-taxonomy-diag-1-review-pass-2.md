# Review: Semantic Diagnostic Code Taxonomy — milestone_diag_1 (pass 2)

**Scope reviewed:** uncommitted working tree at `branch=main` against (a) the `milestone_diag_1` definition in [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) and (b) the pass-1 review at [reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-1.md). This is a pre-production review; existing emission paths are intentionally not migrated in this wave.

**Files re-inspected since pass 1:**

- `crates/sifr_diagnostics/src/{lib,model/mod,codes/mod,source_map/mod,render/mod,schema/mod,bin/gen-diagnostic-schema}.rs`
- `crates/sifr_hir/src/{lib.rs,lowering_outcome.rs}`
- Workspace and per-crate `Cargo.toml`s (`sifr`, `sifr_codegen`, `sifr_driver`, `sifr_hir`, `sifr_type_system`)
- `docs/schemas/diagnostics.schema.json`
- `scripts/check_diagnostic_schema_sync.py`, `scripts/run_all_tests.sh`
- `internal_docs/architecture.md` (diagnostic mapping section), `internal_docs/roadmap.md`, `internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md`
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
- Existing review artifacts under `reviews/`

**Validation re-run by reviewer (this pass):**

- `cargo test -p sifr_diagnostics` → 25 passed, 0 failed.
- `python3 scripts/check_diagnostic_schema_sync.py` → OK.
- `cargo check --workspace` → clean.
- `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` → **fails with 12 errors** (`unwrap_used` / `unwrap_err` in test modules). See finding §C.1.

**Verdict:** all six pass-1 must-fix items are resolved at the code level. The new tests are well-targeted and the JSON contract is now genuinely round-trip stable. Residual blockers are limited to one re-confirmed clippy regression (pass-1 #21) and the schema-vs-writer asymmetry (pass-1 #20). With those two fixed, the milestone is mergeable. Pass-1 should-fix items #7 (model decomposition), #9 (severity_rank redundancy), #17 (RelatedKind ordering decision), #22 (architecture wording), and #23 (blocked-review file disposition) are still open but are clearly scoped follow-ups, none of which should block `milestone_diag_1` if explicitly acknowledged in the issue's `Execution Status`.

---

## §A. Pass-1 must-fix resolution

### A.1 — pass-1 #1 (`LoweringOutcome` missing) → ✅ Resolved

[crates/sifr_hir/src/lowering_outcome.rs:1](crates/sifr_hir/src/lowering_outcome.rs:1) adds:

```rust
pub struct LoweringOutcome {
    pub result: LoweringResult,
    pub diagnostics: Vec<SifrDiagnostic>,
}
```

…re-exported from [crates/sifr_hir/src/lib.rs:21](crates/sifr_hir/src/lib.rs:21). `sifr_hir` now genuinely consumes its `sifr_diagnostics` workspace dep (`grep -rn "use sifr_diagnostics" crates/` returns the new file), which also resolves part of pass-1 #15: at least one downstream crate now imports the type, so the dep edge isn't purely cosmetic anymore.

Note (informational, not a finding): the type has no constructor, no `Default` impl, no `record(...)` / `extend(...)` helpers, and no test. That matches the spec's "alongside the existing `LoweringError`" intent — actual routing lands in `milestone_diag_4a` — but the empty surface means a future PR that wires it up has no precedent for whether `result` should be `Option<LoweringResult>`, `Result<LoweringResult, ErrorEmitted>`, or stay as today. Not a blocker for this milestone; flag for `milestone_diag_4a` planning.

### A.2 — pass-1 #2 (`DiagnosticArg` round-trip) → ✅ Resolved

[crates/sifr_diagnostics/src/model/mod.rs:26-34](crates/sifr_diagnostics/src/model/mod.rs:26) switches `DiagnosticArg` to a tagged representation:

```rust
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum DiagnosticArg { String, Signed, Unsigned, Float, Bool }
```

The schema reflects this as five `oneOf` branches with required `kind` + `value` ([docs/schemas/diagnostics.schema.json:94-180](docs/schemas/diagnostics.schema.json:94)). Round-trip identity is locked by `rendered_diagnostic_json_round_trips_without_losing_arg_kinds_or_nulls` ([crates/sifr_diagnostics/src/render/mod.rs:467-493](crates/sifr_diagnostics/src/render/mod.rs:467)), which:

- Emits an `Unsigned(5_u64)` arg, asserts `"kind":"unsigned"` survives the round trip.
- Asserts `"help":null` and `"label":null` are present in the writer output (locking the explicit-null contract).
- Asserts the deserialized `DiagnosticEnvelope == envelope`.

This is the canonical bytes contract that `compare_diagnostics` and `canonical_args_bytes` depend on; the round-trip test catches any future serde override that flips Option fields to `skip_serializing_if`.

Side benefit worth recording: `Signed(5)` and `Unsigned(5)` are now lexicographically distinct under `canonical_args_bytes`, so dedupe and ordering keys can no longer alias across signedness — the bug class pass-1 explicitly called out.

### A.3 — pass-1 #3 (`DiagnosticBuilder::source/internal` signature) → ✅ Resolved

[crates/sifr_diagnostics/src/model/mod.rs:312-332](crates/sifr_diagnostics/src/model/mod.rs:312) restores the documented surface:

```rust
pub fn source(code: DiagnosticCode, severity: Severity, primary_span: SourceSpan) -> Self {
    assert_eq!(severity, code.declared_severity(),
        "diagnostic severity must match the registry-declared severity");
    ...
}
```

Same shape on `internal`. The runtime assertion locks the implementer-facing constraint without losing the strong tie between code-and-severity declaration. This matches option (1) in pass-1's "two ways forward" and is the smallest diff against the documented API.

### A.4 — pass-1 #4 (release-mode drop discipline) → ✅ Resolved (with one noted gap)

The pass-2 implementation:

1. Adds a process-global `UNEMITTED_DIAGNOSTIC_DROP_COUNT: AtomicUsize` and a queryable hook `take_unemitted_diagnostic_drop_count() -> usize` ([model/mod.rs:43-48](crates/sifr_diagnostics/src/model/mod.rs:43)).
2. Increments that counter from `Drop` impls on `DiagnosticBuilder`, `SourceDiagnostic`, and `InternalDiagnostic` *before* the `debug_assert!(false, …)` panic ([model/mod.rs:268-289, 432-441](crates/sifr_diagnostics/src/model/mod.rs:268)).
3. Adds `#[should_panic]` tests:
   - `dropping_builder_without_consumption_panics_in_debug` ([model/mod.rs:720-727](crates/sifr_diagnostics/src/model/mod.rs:720))
   - `dropping_diagnostic_without_consumption_panics_in_debug` ([model/mod.rs:729-739](crates/sifr_diagnostics/src/model/mod.rs:729))
4. Gates the panic tests with `#[cfg(debug_assertions)]` so `cargo test --release` doesn't false-positive.

That fully satisfies pass-1's "either a release-mode reporting hook or an explicit DoD amendment." The hook exists, is queryable from a future error-boundary, and is tested for compilability.

**Noted gap (should-fix in §B, not a blocker):** the hook's *behavior* in release mode isn't asserted. `release_drop_violation_hook_is_queryable` ([model/mod.rs:741-745](crates/sifr_diagnostics/src/model/mod.rs:741)) only proves the function is callable and starts at zero — it doesn't drop a builder and observe the counter increment. A small `#[cfg(not(debug_assertions))]` test that drops a builder and asserts the counter incremented would make the release contract testable. Worth doing, but not in scope to block the milestone.

**Subtle race risk (record-only):** `UNEMITTED_DIAGNOSTIC_DROP_COUNT` is process-global and `cargo test` runs tests in parallel. A `should_panic` test increments the counter (then panics, which is caught), and `release_drop_violation_hook_is_queryable` does `take(); assert_eq!(take(), 0)`. If a parallel `should_panic` test increments between those two `take` calls, the second take returns >0 and the assertion fails. The probability is low — there are only two `should_panic` drop tests and both run quickly — but it is a pre-existing flakiness vector. Use `serial_test::serial`, a `Mutex<()>` guard, or run drop-discipline tests with `cargo test -- --test-threads=1` if a flake ever surfaces. Not a `milestone_diag_1` blocker.

### A.5 — pass-1 #5 (`SourceSpan::new` validation) → ✅ Resolved

[crates/sifr_diagnostics/src/source_map/mod.rs:26-34](crates/sifr_diagnostics/src/source_map/mod.rs:26) adds:

```rust
pub fn new_validated(
    source_map: &SourceMap,
    source_id: SourceId,
    range: TextRange,
) -> Result<Self, SourceMapError> { ... }
```

…with a unit test (`new_validated_rejects_invalid_construction`, [source_map/mod.rs:243-254](crates/sifr_diagnostics/src/source_map/mod.rs:243)) that constructs an out-of-range span and asserts `InvalidSpan`. The unvalidated `SourceSpan::new` is retained as `pub const fn` for already-validated paths, which matches pass-1 option 1 verbatim.

The render-boundary half (`SourceMap::validate_span` returning `SourceMapError`, the `Err` flowing up through `render_span`) is unchanged from pass 1; it still doesn't translate to `SIFR-INTERNAL-*`. That conversion is correctly deferred until a calling crate exists, but the spec text at [issue line 687](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:687) is unambiguous about the eventual end state — the conversion needs to land in the same PR that introduces the first non-test caller. Note for `milestone_diag_4a` planning, not a `milestone_diag_1` blocker.

### A.6 — pass-1 #6 (DoD source-map test coverage) → ✅ Resolved

Three new tests cover the previously-missing branches:

- `renders_three_line_span_highlights_middle_line_fully` ([render/mod.rs:389-412](crates/sifr_diagnostics/src/render/mod.rs:389)): asserts the middle line text is `"two"`, `highlight_start == 1`, `highlight_end == 4`. This locks the byte/char-column math for fully-covered intermediate lines (the previously-suspect branch in `render_line`).
- `eof_zero_length_span_has_exclusive_end_position` ([render/mod.rs:414-440](crates/sifr_diagnostics/src/render/mod.rs:414)): asserts `byte_start == byte_end == 2`, `column == end_column == 1`, `line == end_line == 2`. Locks the "end is exclusive" contract from [issue line 677](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:677) at the EOF boundary.
- `crlf_source_has_stable_line_and_column_positions` ([render/mod.rs:442-465](crates/sifr_diagnostics/src/render/mod.rs:442)): pins the current behavior of treating `\r\n` as a single line break (line_starts only tracks `\n`), with the trailing `\r` retained inside `lines[0].text`.

The CRLF lock is correct as a regression-locking move, but it deserves a comment somewhere in `source_map/mod.rs` near `line_starts(...)` saying "CRLF sources keep `\r` inside line text; the human renderer is responsible for stripping or normalizing it before stdout". As-is, a future human renderer reading `lines[0].text == "b\r"` will emit a `\r` to stdout that returns the cursor to column 1 and overwrites earlier output. Not a milestone blocker — the renderer doesn't exist yet — but worth a one-line comment so the trap is visible to whoever writes it. (See §B.5.)

The pre-existing `renders_multibyte_utf8_columns_as_char_offsets` test ([render/mod.rs:340-364](crates/sifr_diagnostics/src/render/mod.rs:340)) covers the multibyte case at a non-boundary position. Multibyte at a *line* boundary is still only implicitly covered (the three-line test uses ASCII-only middle text). Not a blocker — the math doesn't depend on whether the multibyte sits at the boundary or mid-line — but if a hardening test is cheap, it's worth adding.

---

## §B. Pass-1 should-fix status

| # | Pass-1 finding | Status |
|---|---|---|
| 7 | `model/mod.rs` is a soft monolith | **Not addressed** (756 lines, +80 since pass 1 — see §B.1) |
| 8 | `\u{10ffff}` path sentinel + per-comparison allocation | **Resolved** — `path_rank: u8` field + `sort_by_cached_key` (§B.2) |
| 9 | `severity_rank` duplicates derived `Ord` | **Not addressed** (§B.3) |
| 10 | `static_assertions` negative-impl checks | **Resolved** ([model/mod.rs:714-718](crates/sifr_diagnostics/src/model/mod.rs:714)) |
| 11 | Builder duplicate / empty-name args | **Resolved for builder** — `assert!(insert(...).is_none())` + `assert_valid_placeholder` in `arg(...)` ([model/mod.rs:354-361](crates/sifr_diagnostics/src/model/mod.rs:354)) + `duplicate_args_are_rejected` test. Unused-args validation correctly deferred to `milestone_diag_2a`. |
| 12 | `format_arg` for `Float` divergent from JSON | **Resolved** ([model/mod.rs:605-606](crates/sifr_diagnostics/src/model/mod.rs:605) — `serde_json::to_string(value)`). One sub-nit: §B.4 |
| 13 | `PARSE_OPAQUE_ERROR` reserved-code constant | **Resolved** (constant deleted; only `INTERNAL_COMPILER_PANIC`, `NAME_UNDEFINED_VARIABLE`, `TYPE_ASSIGNMENT_MISMATCH`, and `cfg(test) TEST_NOTE` remain — [codes/mod.rs:11-19](crates/sifr_diagnostics/src/codes/mod.rs:11)) |
| 14 | `render_line` highlight clamp tests | **Resolved** by §A.6 |
| 15 | Cargo deps wired without imports | **Resolved** — comments added at each `Cargo.toml`'s `sifr_diagnostics` entry ("Wired early by milestone_diag_1; compiler emission migration starts in milestone_diag_4a"); also `sifr_hir` now genuinely imports the crate (§A.1) |
| 16 | `display_path_for_path` dead code | **Resolved** (helper removed; `display_path(SourceId)` retained, which is the policy-correct accessor) |
| 17 | `RelatedKind` / `DiagnosticSuggestion` ordering decision | Not addressed; nit |

### B.1 — `model/mod.rs` is now 756 lines (was 676 at pass 1)

The file added ~80 lines (drop counter + hook, `assert_valid_placeholder` argument-side check, three new tests, two `static_assertions`, a duplicate-arg test). The decomposition recommended in pass-1 #7 wasn't taken, and the file has grown in the direction the no-monolith rule was created to prevent. The current contents:

- `Severity`, `ChildSeverity` enums
- `DiagnosticArg`, `From` impls, `canonical_json_bytes`
- Drop-discipline counter + queryable hook
- `DiagnosticChild`, `RelatedSpan`, `RelatedKind`, `DiagnosticSuggestion`, `SuggestionEdit`, `SuggestionApplicability`
- `SifrDiagnostic`, `SourceDiagnostic`, `InternalDiagnostic` (with `Drop` impls)
- `DiagnosticBuilder` + `DiagnosticBuilderKind`
- `ErrorEmitted`, `AdmittedDiagnostic`, `DiagnosticSink`
- Template-renderer helpers (`validate_template_args`, `render_message_template`, `extract_placeholders`, `assert_valid_placeholder`, `format_arg`)
- 700-line test module

That's nine independent concerns. AGENTS.md is unambiguous: "All crates should be decomposed into small, focused files — monolithic files are banned." The pass-1 split sketch (severity / arg / related / suggestion / diagnostic / builder / sink, plus tests) still applies and is a mechanical refactor with zero API surface change. Strongly recommend doing it before `milestone_diag_2a` doubles the file again with registry/codes work.

A guardrail script (`scripts/check_diagnostics_maintainability_guardrails.py`, modeled on `check_hir_maintainability_guardrails.py`) added in `milestone_diag_2a` would prevent regrowth in `codes/` once the registry lands. Pass-1 already flagged this; reiterating.

### B.2 — Path sentinel + ordering key construction

The `\u{10ffff}` sentinel is gone. [render/mod.rs:172-211](crates/sifr_diagnostics/src/render/mod.rs:172) now:

- Computes `(path_rank, path)` where `path_rank` is `0` if the diagnostic has a primary span with a known display path and `1` otherwise; the `path` defaults to `String::new()` when no path is available.
- Adds `byte_start, byte_end = (u32::MAX, u32::MAX)` for spanless diagnostics so they sort after spanned ones within the same rank tier.
- Adds `kind_rank: u8::from(primary.is_none())` (still redundant with `path_rank` for the source-vs-internal distinction; see §B.3).
- Switches the renderer call site to `sort_by_cached_key`, so `DiagnosticOrderingKey` is computed once per diagnostic instead of once per comparison.

This addresses both halves of pass-1 #8 (no Unicode sentinel, no per-comparison allocation). The new test `orders_by_path_then_span_then_args_then_insertion_order` ([render/mod.rs:495-529](crates/sifr_diagnostics/src/render/mod.rs:495)) and the existing `diagnostics_differing_only_in_args_sort_by_canonical_json_bytes` lock the ordering against regressions.

One residual asymmetry: `path_rank` (0 for has-span, 1 for spanless) and `kind_rank` (0 for has-span, 1 for spanless) compute exactly the same value today because spanless diagnostics are exclusively internal. Either delete `kind_rank` or document that `kind_rank` is a future-proofing field for if/when source diagnostics ever lose their span requirement. Cosmetic.

### B.3 — `severity_rank` duplicates derived `Ord` (still open)

[render/mod.rs:213-219](crates/sifr_diagnostics/src/render/mod.rs:213) still has the manual mapping. `Severity` is `pub enum Severity { Error, Warning, Note }` with `derive(Ord)`, so `Error < Warning < Note` already holds via declaration order. Either:

```rust
// Option A: drop the helper and use the enum's derived Ord directly
severity_rank: diagnostic.severity(),
// (`DiagnosticOrderingKey` would then have `severity: Severity` instead of `severity_rank: u8`)
```

```rust
// Option B: pin the contract with a static assert
const _: () = {
    use std::mem;
    const fn rank(s: Severity) -> u8 { match s { Severity::Error => 0, ... } }
    assert!(rank(Severity::Error) == 0);
    assert!(rank(Severity::Warning) == 1);
    assert!(rank(Severity::Note) == 2);
};
```

Either fix is a couple of lines. As-is, a future PR that reorders `Severity` (e.g. for an `Info` insertion) would silently desync the rank function from the enum order — and the existing tests don't catch it because they only assert relative ordering across diagnostics with the same severity. Not a `milestone_diag_1` blocker but worth taking before the human renderer writes its own severity-based logic against the enum.

### B.4 — `format_arg` Float canonicalization (informational follow-up)

`format_arg` now uses `serde_json::to_string(value).unwrap_or_else(|err| panic!(...))` for floats. That guarantees the human-rendered message and the JSON `args.value` carry the same string representation for finite f64s. Two notes:

- The `unwrap_or_else(|err| panic!(...))` is unreachable for finite f64 (which `From<f64>` enforces) but is a load-bearing safety net if a caller bypasses `From<f64>` and directly constructs `DiagnosticArg::Float(f64::NAN)`. The panic message is "failed to render finite diagnostic float arg" — fine.
- `DiagnosticArg::canonical_json_bytes` ([model/mod.rs:38-41](crates/sifr_diagnostics/src/model/mod.rs:38)) still uses `unwrap_or_default()`, so a non-finite `Float` directly constructed via the public enum variant would silently produce empty canonical bytes — i.e., two distinct `Float(NaN)` values would dedupe equal. Pass-1 already noted this risk; the public enum variant is still the sharp tool. Worth tightening in `milestone_diag_2b` either by making the variant private (`pub(crate)`) and forcing construction through `From`, or by making `canonical_json_bytes` panic on non-finite. Not a blocker today.

### B.5 — CRLF rendering trap (informational)

The CRLF test locks `lines[0].text == "b\r"`. That's fine for JSON consumers, but a future human renderer that writes `lines[i].text` to stdout will emit a literal `\r` and overwrite earlier output. Add a `// HUMAN RENDERER NOTE: line text may contain \r on CRLF sources; strip or normalize before printing.` comment near `span_lines` / `render_line` in [render/mod.rs:270-319](crates/sifr_diagnostics/src/render/mod.rs:270). Cheaper than a follow-up bug.

---

## §C. Pass-1 schema/sync findings

### C.1 — pass-1 #21 (clippy `--all-targets`) → **Still failing** (must-fix before merge)

Pass-1 #21 explicitly asked for `cargo clippy -p sifr_diagnostics --all-targets --tests -- -D warnings` to be re-run. The author's pass-2 validation list at the top of the prompt records `cargo clippy -p sifr_diagnostics -- -D warnings` (no `--all-targets`), which checks the library only and skips test code.

Re-running with `--all-targets`:

```
cargo clippy -p sifr_diagnostics --all-targets -- -D warnings
…
error: could not compile `sifr_diagnostics` (lib test) due to 12 previous errors
```

All 12 errors are `clippy::unwrap_used` / `clippy::unwrap_err` (workspace lint at warn → promoted to error by `-D warnings`) inside `#[cfg(test)]` modules:

- `crates/sifr_diagnostics/src/render/mod.rs` lines 358, 384, 406, 432, 460, 486, 487, 491, 526, 559
- `crates/sifr_diagnostics/src/source_map/mod.rs` lines 247, 260

Fix is one line at the top of [crates/sifr_diagnostics/src/lib.rs](crates/sifr_diagnostics/src/lib.rs:1), mirroring the carve-out already in [crates/sifr_hir/src/lib.rs:7](crates/sifr_hir/src/lib.rs:7):

```rust
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
```

That's a strict drop-in port of the pattern AGENTS.md already validates. With it, pass-2's lint claim becomes accurate.

While there: `scripts/run_all_tests.sh` doesn't run `cargo clippy` at any profile — it only runs the schema sync, `cargo test`, and the e2e suite. The clippy gate exists in the `cargo clippy --workspace -- -D warnings` documented in `AGENTS.md` but has no automation. Either add a clippy step to `run_all_tests.sh --profile pr` (one line, cheap on a small crate today) or document the manual gate explicitly in the issue's `Execution Status`. Either way, the `cargo clippy` claim in the Execution Status block needs to match what the validation harness actually exercises.

### C.2 — pass-1 #18 (schema sync script ergonomics) → ✅ Resolved

[scripts/check_diagnostic_schema_sync.py:24-30](scripts/check_diagnostic_schema_sync.py:24) now prints "schema sync: failed to invoke generator: cargo run -q -p sifr_diagnostics --bin gen-diagnostic-schema\n" when the cargo invocation fails, then forwards `generated.stderr`. That's exactly the improvement pass-1 sketched. The `--locked`/`--frozen` part wasn't taken; arguably fine for this milestone since `Cargo.lock` is already committed and the generator's transitive dep set is small (`schemars`, `serde_json`).

### C.3 — pass-1 #19 (schema generator silent failure) → ✅ Resolved

[crates/sifr_diagnostics/src/schema/mod.rs:11-12](crates/sifr_diagnostics/src/schema/mod.rs:11) replaces the `"{}"` fallback with `unwrap_or_else(|err| panic!("failed to serialize diagnostics JSON schema: {err}"))`. Since `expect_used` is workspace-warn, the explicit `panic!` form is the right choice. The schema-sync pipeline now fails loud on serialization breakage instead of silently writing `{}` and failing the byte-compare downstream.

The pass-1 follow-up about taking a file-path argument (so the regenerate command becomes `cargo run --bin gen-diagnostic-schema -- docs/schemas/diagnostics.schema.json`) wasn't taken; the regenerate command is still `> docs/schemas/diagnostics.schema.json`. Cosmetic, not a blocker.

### C.4 — pass-1 #20 (schema doesn't require `Option` fields) → **Not addressed (must-fix before merge)**

[docs/schemas/diagnostics.schema.json](docs/schemas/diagnostics.schema.json) `RenderedDiagnostic.required` ([schema lines 74-84](docs/schemas/diagnostics.schema.json:74)) still omits `help`. `DiagnosticSpan.required` ([schema lines 250-255](docs/schemas/diagnostics.schema.json:250)) still omits `file`, `line`, `column`, `end_line`, `end_column`, `label`. Yet the writer always emits these as `null` when None — verified by the new round-trip test asserting `"help":null` is in the JSON output ([render/mod.rs:488-489](crates/sifr_diagnostics/src/render/mod.rs:488)).

This means the writer's contract ("explicit null fields") and the schema's contract ("may be absent") disagree. A consumer that strictly validates against the schema will accept payloads missing these fields, then fail at the application layer when it expects `help: null` and gets nothing.

Two paths forward, pick one:

1. **Tighten the schema** (matches the writer): override `JsonSchema` to add the Option fields to `required`. The `schemars` v1 idiom is to implement `JsonSchema` manually for these structs and call `.add_required("help")` etc., or use an attribute helper if one exists. This is the option that keeps the documented "explicit null" contract.
2. **Loosen the writer** (matches the schema): add `#[serde(skip_serializing_if = "Option::is_none")]` to the Option fields in `RenderedDiagnostic` and `DiagnosticSpan`. Then strike "explicit `null` fields where applicable" from the milestone DoD ([issue line 721](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:721)) since it would no longer hold.

The pass-2 round-trip test would still pass under (2) because `Option::None` deserializes from missing-field input either way — but the `assert!(json.contains("\"help\":null"))` assertion in [render/mod.rs:488](crates/sifr_diagnostics/src/render/mod.rs:488) would then need to be deleted, exposing that the contract changed.

Recommend (1). It matches the milestone DoD as written, requires no test deletions, and gives external consumers a stable shape.

---

## §D. Documentation

### D.1 — pass-1 #22 (architecture wording) → **Not addressed**

[internal_docs/architecture.md:720](internal_docs/architecture.md:720) still reads:

> Canonical diagnostic object: parser, lowering, type checking, borrow checking, and codegen emit `SifrDiagnostic` values from `sifr_diagnostics`.

`milestone_diag_1`'s scope is to introduce the model. None of those crates emit `SifrDiagnostic` yet (the only consumer is `crates/sifr_hir/src/lowering_outcome.rs`, which holds the type but doesn't construct or emit). The wording overstates current state and will mislead a reader who lands on the architecture doc.

Two-character fix: change "emit" to "must emit" or "are required to emit … once migrated"; or prefix the bullet with "Target shape:". Pick one before merging, since the architecture doc is the canonical reference and this paragraph is the load-bearing one for the diagnostic taxonomy migration.

### D.2 — pass-1 #23 (review-blocked file disposition) → **Not addressed**

[reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md](reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md) is still in the tree, and the issue's `Execution Status` line 18 still references it as the only review artifact. After this pass-2 review lands, the directory will hold three review files for the same milestone (blocked, pass-1, pass-2), with the issue pointing at "blocked." A future reader will reasonably conclude the milestone never had a successful review.

Action before merge — pick one:

- Delete `…-review-blocked.md`; replace the Execution Status reference with bullet pointers to `…-review-pass-1.md` and this file (`…-review-pass-2.md`).
- Rename it to `…-review-blocked-attempt-2026-04-29.md` and re-classify it as an automation-failure artifact, not a review.

Either is fine; both keep the chain of evidence intact while making the canonical review obvious. Recommend (1) — the file currently records process state, not review content, and the Execution Status update can carry the same context in one line.

### D.3 — Issue Execution Status

The Execution Status checklist at [issue lines 17-19](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:17) still has:

```
- [ ] Claude review for milestone_diag_1 completed and all actionable findings addressed.
- [ ] milestone_diag_1 PR opened and merged.
```

After this pass-2 review and the §C.1 / §C.4 / §D.1 / §D.2 fixes land, the first checkbox can be ticked with a reference to `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-pass-2.md`. The validation evidence block at lines 22-28 should also be updated to say `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` (matching what the gate actually requires).

### D.4 — Roadmap row 31.7 → ✅ Internally consistent

[internal_docs/roadmap.md:50](internal_docs/roadmap.md:50) marks Phase 27 as `completed, amended` with the note about the ad-hoc phase, and row 57 marks 31.7 as `in_progress`. Pass-1 #22.2 flagged the freeform `completed, amended` token; it's still cosmetic and not worth fixing in this milestone.

---

## §E. New issues introduced in pass 2 (regressions / fresh bugs)

None observed. The pass-2 deltas are:

- `LoweringOutcome` (new file, 7 lines, no logic).
- `DiagnosticArg` tagged form (verified by round-trip test).
- Builder `severity` parameter restored with assertion (asserted at runtime; assertion message unambiguous).
- Drop-discipline counter + `should_panic` tests (no observable behavior change in non-test code; the `take_unemitted_diagnostic_drop_count` API is `pub` and exposed as a leaf hook).
- `SourceSpan::new_validated` (additive; no caller change).
- Three new render tests (additive).
- `PARSE_OPAQUE_ERROR` deletion (constant was unused; no callers to break — confirmed via grep).
- `\u{10ffff}` sentinel removal + `sort_by_cached_key` (locked by the new ordering test).
- `format_arg(Float)` switched to `serde_json::to_string` (round-trip and human-render now agree).
- Schema sync script error message + generator panic-on-error (additive).
- Cargo.toml comments (cosmetic).

The risk surfaces I checked for and didn't find:

- **Drop-counter race:** documented in §A.4 but currently latent (only two drop tests).
- **Tagged DiagnosticArg breaking ordering of pre-existing tests:** the existing `orders_by_path_then_span_then_args_then_insertion_order` test still passes; the `args` JSON shape changed but consistency holds.
- **`format_arg(Float)` panicking on non-finite leaks:** confirmed unreachable for `From`-constructed values; non-finite via direct enum construction is still a public-API trap, but no caller exists.
- **Schema-sync script accepting an empty `expected`:** no — `actual != expected` when `expected = ""` and `actual = "{...}"`, the script reports out-of-sync. OK.
- **Public surface drift in `lib.rs`:** `pub use` list at [crates/sifr_diagnostics/src/lib.rs:1-14](crates/sifr_diagnostics/src/lib.rs:1) is consistent with what the model and render modules expose; nothing leaked accidentally.

---

## §F. Risk spot-checks (re-verified)

- **No public diagnostic types defined outside `sifr_diagnostics`:** confirmed via `grep -rn "use sifr_diagnostics" crates/` — only `crates/sifr_hir/src/lowering_outcome.rs:2` (which imports `SifrDiagnostic`, not redefines it). ✓
- **`Severity::Help` cannot exist:** `Severity` enum unchanged from pass 1 (`Error | Warning | Note`); `ChildSeverity` provides `Note | Help`. ✓
- **`SourceDiagnostic` requires a span:** `primary_span: SourceSpan` (not `Option`). ✓
- **`ErrorEmitted` zero-size + unforgeable:** `assert_eq_size!(ErrorEmitted, ())` at [model/mod.rs:717](crates/sifr_diagnostics/src/model/mod.rs:717). ✓
- **Builder `#[must_use]` and `!Clone`:** `assert_not_impl_any!(DiagnosticBuilder: Clone)` at [model/mod.rs:716](crates/sifr_diagnostics/src/model/mod.rs:716); `#[must_use]` at [model/mod.rs:292](crates/sifr_diagnostics/src/model/mod.rs:292). ✓
- **JSON envelope `{ "version": 1, "diagnostics": [...] }`:** unchanged. ✓
- **`DiagnosticSink` insertion-order monotonicity:** locked by `sink_records_errors_with_insertion_order` and `orders_by_path_then_span_then_args_then_insertion_order`. ✓
- **Renderer is the only sort site:** confirmed; `render_sink` sorts via `sort_by_cached_key`. ✓
- **Round-trip identity:** locked by `rendered_diagnostic_json_round_trips_without_losing_arg_kinds_or_nulls`. ✓
- **deny_unknown_fields on consumed payloads:** confirmed on `DiagnosticEnvelope`, `RenderedDiagnostic`, `RenderedDiagnosticChild`, `RenderedDiagnosticSuggestion`, `RenderedSuggestionEdit`, `DiagnosticSpan`, `DiagnosticSpanLine` ([render/mod.rs:9, 16, 31, 38, 46, 53, 68](crates/sifr_diagnostics/src/render/mod.rs:9)). ✓

---

## §G. Migration-friendliness re-check

Items relevant for `milestone_diag_4a` planning:

- **`LoweringOutcome` shape is locked but minimal.** No `record(...)` or `extend(...)` helper. The first migration PR will need to define the helper API before any HIR pass can route through it. Cheap to add now; cheap to add then. Either is fine.
- **`SourceSpan::new` vs `new_validated` policy.** With `new` retained as `pub const fn`, HIR's first migration will have a choice: (a) feed AST ranges through `new_validated` and propagate `SourceMapError` upward, or (b) trust the AST and use `new`, with debug builds catching invariant violations only at the render boundary. Spec line 687 mandates debug-build construction validation; the cleanest answer is to make HIR adapters always call `new_validated` and translate the `Err` to `SIFR-INTERNAL-*`. Document the policy in `architecture.md` when `milestone_diag_4a` lands.
- **Render-boundary `SourceMapError → SIFR-INTERNAL-*` conversion.** Still unwired (correct for this milestone). The first migration that calls `render_sink` from a non-test caller will need to add the conversion. Worth a TODO comment near `render_sink`'s `Result<DiagnosticEnvelope, SourceMapError>` signature so the next implementer doesn't miss it. Optional for `milestone_diag_1`.
- **Public `DiagnosticArg::Float` variant accepts non-finite by direct construction.** Tightening to `pub(crate)` + forcing `From` is a one-line change but breaks the tagged-JSON consumer surface (consumers can no longer construct `DiagnosticArg` values for tests). Defer to `milestone_diag_2b` when registry helpers exist; track in the issue.
- **`model/mod.rs` decomposition.** Should land before `milestone_diag_2a` registry growth.

---

## §H. Required actions before merging `milestone_diag_1`

**Must-fix (block merge):**

1. **§C.1 — clippy lint regression.** Add `#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]` at [crates/sifr_diagnostics/src/lib.rs:1](crates/sifr_diagnostics/src/lib.rs:1). Re-run `cargo clippy -p sifr_diagnostics --all-targets -- -D warnings` and confirm clean. Update the Execution Status validation list to record `--all-targets`.
2. **§C.4 — schema/writer asymmetry.** Either tighten the schema to require Option fields (preferred) or loosen the writer to skip them. Update [reviews/](reviews/) tests accordingly.
3. **§D.1 — architecture wording.** Re-phrase [internal_docs/architecture.md:720](internal_docs/architecture.md:720) to reflect target shape rather than current state.
4. **§D.2 — review-blocked file.** Delete or rename `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md`; update [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:18](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:18) to point at the pass-1 + pass-2 review files.
5. **§D.3 — Execution Status.** Tick the review checkbox once §H.1-§H.4 land; update validation evidence block to match the actual gate command.

**Should-fix (track explicitly, not block merge):**

6. **§B.1** — decompose `model/mod.rs` per the pass-1 sketch before `milestone_diag_2a` adds registry code.
7. **§B.3** — drop or static-assert `severity_rank` to keep it from drifting from `Severity`'s declared `Ord`.
8. **§A.4 noted gap** — add a `#[cfg(not(debug_assertions))]` test that drops a builder and asserts the counter incremented.
9. **§B.5** — comment near `span_lines` warning the human renderer about embedded `\r` on CRLF sources.
10. **§A.5 follow-up** — TODO comment near `render_sink`'s `Result` return so the `SourceMapError → SIFR-INTERNAL-*` conversion isn't missed when `milestone_diag_4a` lands the first non-test caller.

**Optional polish:**

- Add a `cargo clippy --workspace --all-targets -- -D warnings` step to `scripts/run_all_tests.sh --profile pr` so the gate is automated.
- Add a `scripts/check_diagnostics_maintainability_guardrails.py` modeled on the HIR guardrails to lock the eventual decomposition in §B.1.
- Rename `dropping_*_panics_in_debug` tests to `…_in_debug_increments_counter_then_panics` to make the dual-effect explicit.
- One-line comment on `kind_rank` in `DiagnosticOrderingKey` explaining its overlap with `path_rank` (§B.2).

---

## §I. Summary

Pass-2 cleanly addresses every pass-1 must-fix at the code level. The blocking residue is two items that pass-1 already raised and pass-2 didn't touch:

- The clippy gate hasn't been run with `--all-targets` and currently fails (§C.1).
- The schema-vs-writer Option-field asymmetry is unchanged (§C.4).

Plus two doc/process items that need to land alongside the merge to keep the artifact trail honest (§D.1, §D.2). All four are mechanical, sub-30-minute fixes.

Pass-1 should-fix items #7 (model decomposition), #9 (severity_rank), #17 (RelatedKind ordering), and #22 (architecture wording) are still open as documented follow-ups; #22 needs to land for the merge per §D.1.

— Pass-2 review complete.
