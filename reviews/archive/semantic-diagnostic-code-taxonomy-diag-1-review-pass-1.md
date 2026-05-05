# Review: Semantic Diagnostic Code Taxonomy — milestone_diag_1 (pass 1)

**Scope reviewed:** uncommitted working tree against the `milestone_diag_1` definition in `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`. This is a pre-production review; no compatibility/fallback layer is expected, and existing emission paths are intentionally not migrated in this wave.

**Files reviewed:**

- `crates/sifr_diagnostics/Cargo.toml`, `src/lib.rs`, `src/codes/mod.rs`, `src/model/mod.rs`, `src/source_map/mod.rs`, `src/render/mod.rs`, `src/schema/mod.rs`, `src/bin/gen-diagnostic-schema.rs`
- `docs/schemas/diagnostics.schema.json`
- `scripts/check_diagnostic_schema_sync.py`
- `scripts/run_all_tests.sh`
- Workspace + per-crate `Cargo.toml` deltas (`Cargo.toml`, `crates/sifr/Cargo.toml`, `crates/sifr_codegen/Cargo.toml`, `crates/sifr_driver/Cargo.toml`, `crates/sifr_hir/Cargo.toml`, `crates/sifr_type_system/Cargo.toml`)
- `internal_docs/architecture.md`, `internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md`, `internal_docs/roadmap.md`
- `issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md`
- `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md` (status only)

**Validation reported by author:** `cargo test -p sifr_diagnostics`, `python3 scripts/check_diagnostic_schema_sync.py`, `cargo fmt --check`, `cargo check --workspace`, `cargo clippy -p sifr_diagnostics -- -D warnings`, and `scripts/run_all_tests.sh --profile quick` all passing. Not re-executed by reviewer.

**Verdict:** changes are usable as a foundation, but several items materially diverge from the `milestone_diag_1` DoD or set traps for the later migration. The findings below are split by severity. Items marked **must-fix** are blockers per the milestone DoD or hard rules. **should-fix** items are clear deviations from the issue's documented API surface that will create rework if left until later milestones. **nit** items are local cleanups.

---

## Must-fix

### 1. `LoweringOutcome` is not added (milestone_diag_1 DoD scope item missing)

The DoD for `milestone_diag_1` ([issue lines 706](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:706)) lists:

> Add the canonical `LoweringOutcome` and `DiagnosticSink` types alongside the existing `LoweringError`. `LoweringError` becomes private transitional plumbing only, …

`DiagnosticSink` is in [crates/sifr_diagnostics/src/model/mod.rs](crates/sifr_diagnostics/src/model/mod.rs:437), but `LoweringOutcome` is nowhere in the tree (`grep -rn LoweringOutcome` over `crates/` returns nothing) and `crates/sifr_hir` does not yet import `sifr_diagnostics` despite the new dependency line in `Cargo.toml`. This is an explicitly in-scope DoD bullet, and the `LoweringOutcome { result, diagnostics: Vec<SifrDiagnostic> }` shape lives next to HIR (it references `LoweringResult`), so it cannot be defined later without also touching this milestone's surface area. Either add the type now (in `sifr_hir`) or amend the issue's milestone scope before closing.

The reviewer-supplied constraint says "existing compiler emission paths are intentionally not migrated in this first wave" — that does not contradict adding the new type alongside the old one; it only defers actually routing through it. A two-line `pub struct LoweringOutcome { … }` plus a re-export does not migrate any path.

### 2. `DiagnosticArg` JSON round-trip is lossy, and there is no round-trip test

DoD ([issue lines 721](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:721)):

> Lossless JSON means round-trip identity for diagnostics, explicit `null` fields where applicable, deny-unknown-fields deserialization for consumed payloads, and a schema-regeneration check.

`DiagnosticArg` is `#[serde(untagged)]` with variant order `String, Signed, Unsigned, Float, Bool` ([crates/sifr_diagnostics/src/model/mod.rs:25-33](crates/sifr_diagnostics/src/model/mod.rs:25)). A `DiagnosticArg::Unsigned(5)` serializes as `5`, which on deserialization matches the first numeric variant — `Signed(5)` — so `serde_json::from_str(&serde_json::to_string(&Unsigned(5)))` is **not** the identity. This breaks the documented round-trip property and silently changes the canonical args bytes used by `ordering_key`/`compare_diagnostics` and recovery dedupe (per spec line 615) for any code that ever stores an `Unsigned` arg.

There is also no test that round-trips a constructed `DiagnosticEnvelope` through `serde_json::to_string` → `from_str` and asserts `==`. Add one and either:

- Drop `Unsigned` (almost everything we store is naturally `Signed` or `String`), or
- Switch the enum to a tagged form (e.g. `{ "type": "unsigned", "value": 5 }`) so the schema and the round-trip agree.

The same test should cover the explicit-`null` path (`help`, `label`, `line`/`column`, etc.) so a future serde override that flips to `skip_serializing_if = Option::is_none` is caught.

### 3. `DiagnosticBuilder::source` / `internal` signatures diverge from the documented API

The issue documents the target builder API ([issue lines 425-441](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:425)) as:

```rust
pub fn source(code: DiagnosticCode, severity: Severity, primary_span: SourceSpan) -> Self;
pub fn internal(code: DiagnosticCode, severity: Severity) -> Self;
```

The implementation drops the `severity` parameter and pulls it from `code.declared_severity()` ([crates/sifr_diagnostics/src/model/mod.rs:298-312](crates/sifr_diagnostics/src/model/mod.rs:298)). The DoD line "Add `DiagnosticBuilder` with the target surface described above, including `source`, `internal`, …" explicitly references that surface.

This is defensible — codes are intended to carry stable severity (issue line 705) and `Severity::Help` cannot exist top-level — but it should be an intentional, documented amendment rather than a silent deviation. Two ways forward:

1. Restore the `severity` parameter and assert at runtime that it equals `code.declared_severity()`. This is the smallest diff and matches the documented surface verbatim.
2. Update the issue's "Diagnostic Builder API" example to match the implemented surface (no severity parameter), and call out in the milestone notes that severity is now derived from the code constant.

Either is fine; what is not fine is shipping the milestone with the docs and the implementation disagreeing.

### 4. Drop discipline is debug-only — release path silently swallows misuse

The spec is explicit ([issue lines 549-552](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:549)):

> In debug builds this should panic. In release builds it should be surfaced at the nearest compiler error boundary as `SIFR-INTERNAL-*`, not silently ignored …

The current `Drop` impls on `DiagnosticBuilder`, `SourceDiagnostic`, and `InternalDiagnostic` use `debug_assert!` only ([crates/sifr_diagnostics/src/model/mod.rs:260-275, 408-415](crates/sifr_diagnostics/src/model/mod.rs:260)). In release, a dropped-without-emit/cancel diagnostic disappears with no signal at all. There is also no test asserting the debug panic actually fires (`#[should_panic]` on a dropped builder).

This milestone is the right place to wire up the release-mode escape: a thread-local "leaked diagnostic counter" or a process-level `SIFR-INTERNAL-*` constructor invoked from `Drop` is enough; the boundary plumbing can land later. As-shipped, the discipline is a guideline, not a contract — and a future `cargo test --release` regression will not catch it.

Add at minimum:

- A `#[should_panic(expected = "without …")]` test for builder-drop.
- A `#[should_panic]` test for `SifrDiagnostic`-drop.
- Either a release-mode reporting hook or an explicit DoD amendment narrowing the requirement to debug builds for `milestone_diag_1`.

### 5. `SourceSpan::new` does not validate against the registered source in debug builds

DoD ([issue line 687](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:687)):

> `SourceSpan::new(source_id, range)` should validate the range against the registered source in debug builds. Render/JSON lowering validates every `SourceSpan` against the source map before producing a `DiagnosticSpan` and returns `SIFR-INTERNAL-*` if a compiler bug produced an invalid span in release mode. Span validation tests in `milestone_diag_1` cover both construction and render-boundary validation behavior.

Current `SourceSpan::new` is a `const fn` taking `(SourceId, TextRange)` with no `SourceMap` parameter ([crates/sifr_diagnostics/src/source_map/mod.rs:19-25](crates/sifr_diagnostics/src/source_map/mod.rs:19)) — it physically cannot validate. Validation is only enforced inside `render_span` via `SourceMap::validate_span` ([crates/sifr_diagnostics/src/render/mod.rs:238](crates/sifr_diagnostics/src/render/mod.rs:238)), and that path returns `Err(SourceMapError)` to the caller rather than producing a `SIFR-INTERNAL-*` diagnostic.

Pick one and ship it:

- Add `SourceSpan::new_validated(map, source_id, range) -> Result<...>` and have HIR/parser construction call that one; keep `SourceSpan::new` for already-validated paths but document it as "internal compiler invariant — caller must guarantee."
- Add a debug-mode `validate_in(&self, map: &SourceMap)` helper plus a unit test that constructs an invalid span and exercises it.

Either way, the spec's "construction" half of the validation contract is unimplemented. Also note the render-boundary half currently produces a `SourceMapError` (`UnknownSource` / `InvalidSpan`) — the conversion to a `SIFR-INTERNAL-*` diagnostic is not wired (which is fine for now since no caller exists, but worth a TODO/issue link so the conversion lands with the first migrating crate, not later).

### 6. DoD-listed source-map test coverage is partially missing

The `milestone_diag_1` DoD ([issue line 734](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:734)) calls out:

> Source-map unit tests cover multibyte UTF-8 columns, multiline spans, zero-length spans, EOF spans, invalid span rejection, and byte/line/column consistency.

Coverage today:

- Zero-length / EOF: covered ([source_map/mod.rs:230-237](crates/sifr_diagnostics/src/source_map/mod.rs:230)).
- Invalid span rejection: covered ([source_map/mod.rs:240-250](crates/sifr_diagnostics/src/source_map/mod.rs:240)).
- Metadata: covered.
- **Multibyte UTF-8 columns:** asserted only via the renderer test (`renders_multibyte_utf8_columns_as_char_offsets`, [render/mod.rs:347-367](crates/sifr_diagnostics/src/render/mod.rs:347)). The `source_map` module itself doesn't exercise its line-start/byte-offset machinery against multibyte content.
- **Multiline spans:** the renderer test only asserts `lines.len() == 2` ([render/mod.rs:369-385](crates/sifr_diagnostics/src/render/mod.rs:369)); it never inspects line text, `highlight_start`/`highlight_end`, or that intermediate lines render correctly when a span covers ≥3 lines.
- **Byte/line/column consistency:** no test inverts a (line, column) pair back to byte offsets, and no test asserts that `byte_end` is exclusive while `end_column` lands on the position immediately after the highlight (per [issue line 677](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:677)).

Recommend at least three more tests:

1. Multibyte 4-byte emoji at the *line boundary* (the `highlight_end_byte.saturating_sub(line_start)` math in `render_line` is suspicious for spans crossing line boundaries — see also nit #14).
2. A 3-line span where the middle line is fully covered, asserting `lines[1].highlight_start == 1` and `highlight_end == lines[1].text.chars().count() + 1`.
3. CRLF source — `line_starts` only tracks `\n` ([source_map/mod.rs:170-178](crates/sifr_diagnostics/src/source_map/mod.rs:170)); confirm whether CRLF is intentionally counted as a single line break (it is in the current code) and lock it with a test before HIR starts feeding real source through.

---

## Should-fix

### 7. `model/mod.rs` is 676 lines — already a soft monolith

`AGENTS.md`: *"All crates should be decomposed into small, focused files — monolithic files are banned."* The `model/mod.rs` file currently contains `Severity`, `ChildSeverity`, `DiagnosticArg`, `DiagnosticChild`, `RelatedSpan`/`RelatedKind`, `DiagnosticSuggestion`/`SuggestionEdit`/`SuggestionApplicability`, `SifrDiagnostic`/`SourceDiagnostic`/`InternalDiagnostic`, `DiagnosticBuilder`, `ErrorEmitted`, `DiagnosticSink`/`AdmittedDiagnostic`, the template renderer, and a substantial test module. That is roughly eight independent concerns wedged into one file.

There is no enforcement script for this crate (the existing `check_hir_maintainability_guardrails.py` is HIR-specific), but the no-monolith rule applies across the workspace per AGENTS.md, and waiting until the file doubles in size during `milestone_diag_2`/`milestone_diag_4a` is exactly the regrowth pattern the HIR guardrails were created to prevent. Suggested split (no API surface change):

```
model/severity.rs       Severity, ChildSeverity
model/arg.rs            DiagnosticArg + From impls + canonical bytes helper
model/related.rs        RelatedSpan, RelatedKind
model/suggestion.rs     DiagnosticSuggestion, SuggestionEdit, SuggestionApplicability
model/diagnostic.rs     SifrDiagnostic, SourceDiagnostic, InternalDiagnostic, Drop impls
model/builder.rs        DiagnosticBuilder, template + arg validation
model/sink.rs           DiagnosticSink, AdmittedDiagnostic, ErrorEmitted
model/mod.rs            re-exports only
```

Worth pairing this with a registry-skeleton-style guardrail script in `milestone_diag_2a` (`scripts/check_diagnostics_maintainability_guardrails.py`) so the same regrowth doesn't recur in `codes/` once the registry lands.

### 8. `\u{10ffff}` path sentinel for spanless diagnostics is fragile

`ordering_key` substitutes `"\u{10ffff}".to_string()` when no primary span exists ([crates/sifr_diagnostics/src/render/mod.rs:189-190](crates/sifr_diagnostics/src/render/mod.rs:189)). The intent is "sort spanless diagnostics after all real paths," but the spec's ordering policy already has an explicit `diagnostic_kind_rank` field for exactly this purpose ([issue lines 610-614](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:610)).

Two issues with the current approach:

1. The sentinel is a real, valid `char`. Any path containing `\u{10ffff}` (extremely unlikely, but valid) would tie or sort earlier than the sentinel.
2. It's a string allocation per comparison. With `sort_by` (not `sort_by_cached_key`), every `compare_diagnostics` call rebuilds full `DiagnosticOrderingKey` values including this sentinel string and the `Vec<u8>` from `serde_json::to_vec` — O(n log n) allocations on a hot path the spec calls out as required for every renderer.

Cleaner shape: change the key's path field to `Option<String>` (or split source vs. internal up front via a top-level enum tier in the key tuple), pre-compute keys once with `sort_by_cached_key` or `sort_by_key`, and let `kind_rank` carry the source-vs-internal distinction without resorting to Unicode tricks. Mention this as a "follow up before `milestone_diag_4a`" if not done now — `compare_diagnostics` is the hot path that every renderer will share.

### 9. Renderer key fields and severity rank duplicate enum order

`severity_rank` ([render/mod.rs:220-226](crates/sifr_diagnostics/src/render/mod.rs:220)) maps `Error → 0`, `Warning → 1`, `Note → 2` — exactly the auto-derived `Ord` on `Severity` (declaration order). The custom function isn't wrong; it's just redundant code that drifts independently if the enum is ever reordered. Either remove `severity_rank` and use `severity as u8` / the derived `Ord`, or add a static assertion that the function and enum agree. Same applies to `kind_rank: u8::from(primary.is_none())` — readable, but a `SourceVsInternal` enum would document intent and make tiered ordering self-explanatory.

### 10. `DiagnosticBuilder` and friends should have `static_assertions`-style negative impl checks

The `model::tests` module asserts `SifrDiagnostic: !Clone` ([model/mod.rs:672-675](crates/sifr_diagnostics/src/model/mod.rs:672)). Spec also requires:

- `DiagnosticBuilder` is `#[must_use]` and not `Clone` (issue lines 549-550).
- `ErrorEmitted` is constructible only by `DiagnosticSink::emit_error` (issue lines 397-411).

Add:

- `static_assertions::assert_not_impl_any!(DiagnosticBuilder: Clone);`
- `static_assertions::assert_impl_all!(DiagnosticBuilder: !Sync);` is unnecessary, but `assert_eq_size` for `ErrorEmitted` to `()` would lock its zero-size invariant.
- A negative test confirming `DiagnosticBuilder` truly aborts at compile time when callers attempt `.clone()`.

### 11. `DiagnosticBuilder::arg` silently overwrites duplicate keys; `build()` only one-way-validates the template

`arg` does `self.args.insert(name.to_string(), value.into())` — calling `.arg("name", "x").arg("name", "y")` produces a diagnostic whose `args` map silently contains `"y"`. Combined with `validate_template_args` only checking that every placeholder has an arg (not the reverse — see [issue lines 360-361](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:360)), several misuse classes pass:

- Duplicate `arg` calls.
- Args declared but unused in the template (acceptable per spec for "JSON-only metadata," but there is no marker today — milestone_diag_2a will need it).
- Empty `arg` name (`""`) — `assert_valid_placeholder` only runs on placeholders, not on `arg(name, …)`. `arg("", v)` produces a `BTreeMap` entry with key `""` that no template can reference.

The "extra args / JSON-only metadata" half is properly a registry concern (`milestone_diag_2a`), but the duplicate-key and empty-name cases are local to the builder. Add `assert!(self.args.insert(...).is_none(), "duplicate arg `{name}`")` and `assert_valid_placeholder(name)` inside `arg(...)`.

### 12. `format_arg` for `DiagnosticArg::Float` uses `f64::to_string` — different from the JSON canonical form

`format_arg` produces user-facing message text. `f64::to_string()` formats `1.0` as `"1"` on stable Rust (no trailing zero), while `serde_json::to_string(&1.0_f64)` yields `"1.0"`. That means the rendered `message` and the JSON `args.value` for the same `Float(1.0)` carry different representations of the same number, which silently breaks the spec's grouping/dedupe contract: `message_template` is the stable head, but the rendered `message` is what users see, and divergence between the two surfaces (and between renderer output and JSON args) becomes a source of "why does JSON say `1.0` and the human renderer say `1`?" tickets.

Pick one canonicalization and use it for both the message render and the JSON `args` value — the simplest is to normalize at construction (`From<f64>` already enforces finite; have it normalize to a standard `format!("{value:?}")` or a lightweight wrapper that prints with consistent precision).

### 13. `PARSE_OPAQUE_ERROR` constant exists for a reserved code

Per [issue line 115](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:115):

> Constants exist only for `Active` codes. `Reserved` and `Retired` codes remain in the registry/docs but have no active emission constant.

`SIFR-PARSE-0001` is documented as "Reserved meaning only: opaque parser error … guardrails must reject it as a default parser emission code." ([issue line 186](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:186)). The current `DiagnosticCode::PARSE_OPAQUE_ERROR` ([crates/sifr_diagnostics/src/codes/mod.rs:13-14](crates/sifr_diagnostics/src/codes/mod.rs:13)) is exactly the "active emission constant" the spec forbids. It is unused today; deleting it is mechanical and removes a future guardrail miss.

`INTERNAL_COMPILER_PANIC` (`SIFR-INTERNAL-0001`) is fine — it is registered as `Active` per `milestone_diag_2a` DoD. `TYPE_ASSIGNMENT_MISMATCH` is fine. `NAME_UNDEFINED_VARIABLE` is fine. `TEST_NOTE` is `cfg(test)` only, fine.

### 14. `render_line`'s highlight clamping is wrong for spans whose end is on a *later* line

```rust
let highlight_end_byte = usize::try_from(byte_end)
    .unwrap_or(line_start)
    .saturating_sub(line_start)
    .min(line_text.len());
```

For a span starting on line 1 and ending on line 3, when `render_line` runs for line 1, `byte_end > line_end_of_line_1`, so `highlight_end_byte = byte_end - line_start_of_line_1`, then clamped to `line_text.len()` — that's correct. For line 2 (the middle line, fully covered), `byte_start < line_start_of_line_2 < byte_end`, so `highlight_start_byte = byte_start.saturating_sub(line_start_of_line_2) = 0` (fine) and `highlight_end_byte = (byte_end - line_start_of_line_2).min(line_text.len())` (fine). For line 3, `highlight_start_byte = (byte_start - line_start_of_line_3) = saturated to 0` (fine) and `highlight_end_byte = byte_end - line_start_of_line_3` (correct).

The math works out, but the *test* doesn't cover any of these branches. With no assertion on `lines[i].highlight_start/end`, a regression that produces `highlight_end = 0` on every line would pass today's test. Re-stating finding #6.3.

Also: when `byte_end == 0` and `line_start > 0`, `byte_end.saturating_sub(line_start) = 0`, then `.min(line_text.len()) = 0`, then `char_column(text, 0) = 1`. So `highlight_end < highlight_start` is prevented by `highlight_end.max(highlight_start)` ([render/mod.rs:320](crates/sifr_diagnostics/src/render/mod.rs:320)) — which is right, but means a degenerate input silently collapses to a zero-width highlight at the start. That's probably the right behavior; lock it with a test.

### 15. Workspace dependencies added to crates that don't yet import the new crate

`crates/sifr/Cargo.toml`, `crates/sifr_codegen/Cargo.toml`, `crates/sifr_driver/Cargo.toml`, `crates/sifr_hir/Cargo.toml`, and `crates/sifr_type_system/Cargo.toml` all add `sifr_diagnostics = { workspace = true }`, but `grep -rn "use sifr_diagnostics"` in each crate's `src/` returns nothing.

This isn't wrong — the milestone establishes the dep so later migrations don't have to thread Cargo.toml edits through every PR. But:

- It compiles `sifr_diagnostics` (and its `schemars` / `serde_json` build) into every build of every dependent crate immediately. Cheap today, but the moment `schemars` gains another transitive dep, that cost lands everywhere.
- `cargo udeps` (or similar) will flag these as unused dependencies if it ever runs in CI. Worth pre-empting with a brief comment in each `Cargo.toml` (`# wired up in milestone_diag_4a`) or deferring the dep additions to the migrating PRs in `milestone_diag_4a`.

If the choice is made to keep the deps now, fine — call it out in the milestone status section of the issue file so a reviewer doesn't ask the same question.

### 16. `display_path_for_path` is unused dead code

`SourceMap::display_path_for_path` ([source_map/mod.rs:163-166](crates/sifr_diagnostics/src/source_map/mod.rs:163)) exists but is not called by any other code in the crate, and the new dependent crates don't import the source map yet. With `unreachable_pub = "warn"` in workspace lints, this is currently allowed because the function is `pub` on a `pub` struct, but it's dead surface: `to_string_lossy()` silently drops non-UTF-8 path bytes, and "lossy display path policy" is a real spec item ([issue lines 685-686](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:685)).

Either delete the helper until a caller materializes, or expand it into the actual "diagnostic path display must be policy-driven" implementation with a test that asserts non-UTF-8 input is handled deterministically (current `to_string_lossy` replaces invalid sequences with U+FFFD, which silently leaks "path corruption" into diagnostic output).

### 17. `ChildSeverity` and `Severity` derive `PartialOrd, Ord` but `RelatedKind`, `DiagnosticSuggestion`, etc. do not

The renderer doesn't sort by these, so this is fine functionally, but it's worth a one-line comment or an explicit decision: are `RelatedKind` / `RelatedSpan` ever expected to be a sort key? If yes (e.g., "primary first, then origin, then notes" in human renderer), the trait is needed; if no, the asymmetry should be commented. Likely a no-op for `milestone_diag_1`.

---

## Schema / sync check

### 18. `check_diagnostic_schema_sync.py` runs `cargo run` every invocation

`scripts/check_diagnostic_schema_sync.py` shells out to `cargo run -q -p sifr_diagnostics --bin gen-diagnostic-schema` ([line 17](scripts/check_diagnostic_schema_sync.py:17)). This means:

- Every `scripts/run_all_tests.sh --profile quick` invocation rebuilds `sifr_diagnostics` once for the schema check and again for the unit tests. Cheap today (the crate is small) but it'll grow.
- The script doesn't pass `--locked` or `--frozen`, so a transient `Cargo.lock` divergence on a developer's machine could silently regenerate dependencies.
- If the build fails, `generated.stderr` is dumped to stderr without a marker or the command line that failed. A user staring at a wall of `error[E0…]` from `sifr_diagnostics` won't know the schema-sync script invoked it.

Two cheap improvements:

```python
# Print the failed command so the user can re-run it standalone.
if generated.returncode != 0:
    sys.stderr.write(f"schema sync: failed to invoke generator: cargo run -q -p sifr_diagnostics --bin gen-diagnostic-schema\n")
    sys.stderr.write(generated.stderr)
    return generated.returncode
```

And: cache the generated string in a temp file and `diff` against `SCHEMA_PATH` (the `actual != expected` byte compare today is fine, but a `diff -u` on mismatch makes regenerating obvious).

### 19. `gen-diagnostic-schema.rs` swallows JSON serialization failure

`schema::diagnostic_schema_pretty_json` falls back to `"{}"` on serialization failure ([crates/sifr_diagnostics/src/schema/mod.rs:11](crates/sifr_diagnostics/src/schema/mod.rs:11)). Then the binary prints `{}\n` and exits 0. The sync script then reports "out of sync" against the checked-in schema, but the *real* failure is hidden.

`schema_for!` cannot fail in any realistic case, so this is theoretical, but it's a needless silent-failure path. Replace with `.expect("schemars failed to serialize DiagnosticEnvelope")` (or, since `expect_used = "warn"` is on, `let json = match serde_json::to_string_pretty(&schema) { Ok(s) => s, Err(e) => panic!("…: {e}") };`). Bonus: have `main` print the schema to a file argument when given one, so the regenerate command is `cargo run --bin gen-diagnostic-schema -- docs/schemas/diagnostics.schema.json` rather than a shell redirect.

### 20. The schema's `RenderedDiagnostic` does not require `help` (and other Option fields)

Per [issue line 721](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:721) "explicit `null` fields where applicable." With `Option<String>`, serde emits `"help": null` when None and accepts both `"help": null` and missing `help` field on input. The Rust **writer** is consistent (always null), but the **schema** marks `help` as not required, which contradicts the "explicit null" intent.

Same for `DiagnosticSpan.file/line/column/end_line/end_column/label`. If consumers must be able to rely on these fields always being present (just possibly `null`), add them to `required` and either:

- Use `#[serde(default)]` on the deserialization side so missing-field input deserializes to None (it already does for `Option`), but keep them in `required` on the writer schema — except `schemars` controls both sides from the same struct.
- Or, more correctly: derive `JsonSchema` with a custom hook that adds the fields to `required`. This is the common `schemars` pattern.

Decide whether the contract is "always present, possibly null" (requires schema fix) or "may be absent" (the current behavior, but then strike "explicit null fields where applicable" from the DoD). Today they disagree.

---

## Documentation

### 21. Roadmap row `31.7` is `in_progress` but the issue still has the review checkbox unchecked

`internal_docs/roadmap.md` row `31.7` is "in_progress" pointing at the issue. The issue's `Execution Status` correctly marks the Claude review as `[ ]` (blocked) and the PR as `[ ]`. That's consistent.

However, the issue claims `cargo clippy -p sifr_diagnostics -- -D warnings` passed (line 27). With workspace `unwrap_used = "warn"` and `expect_used = "warn"`, the `.unwrap()` calls inside `#[cfg(test)]` modules in `render/mod.rs` (lines 361, 383, 416, 441) and `source_map/mod.rs` (line 248 `.unwrap_err()`) should normally fire those lints unless they're allowed in tests. There is no `#[allow(clippy::unwrap_used)]` and the workspace doesn't carve out tests. Either:

- The `pedantic` group's "warn" priority is being shadowed by something I'm missing, or
- The author ran clippy without the test target and `-- -D warnings` only graded non-test code.

Worth re-running `cargo clippy -p sifr_diagnostics --all-targets --tests -- -D warnings` and pasting the result before claiming the lint gate passes. If it does fail, the fix is `#[cfg_attr(test, allow(clippy::unwrap_used))]` at the test module level.

### 22. Architecture/phase doc edits are tight; one observation

`internal_docs/architecture.md` correctly retires `E####/W####` for `SIFR-<FAMILY>-dddd` and adds the `sifr_diagnostics` line. `internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md` correctly notes the corrective amendment. `internal_docs/roadmap.md` flags Phase 27 as `completed, amended`.

Two minor inconsistencies:

- `architecture.md` line 706 (after edit) still references "parser, lowering, type checking, borrow checking, and codegen all emit `SifrDiagnostic` values from `sifr_diagnostics`." That overstates `milestone_diag_1`'s state — those emission paths haven't migrated yet. Recommend rewording as "are required to emit `SifrDiagnostic` values from `sifr_diagnostics` once migrated" or scoping the sentence with "Target shape:" prefix.
- The roadmap row's status uses the freeform string `completed, amended` — neighboring rows use single-token statuses (`completed`, `in_progress`, `closed`, `complete`). Worth deciding whether to introduce a hyphenated status or the equivalent `amended` qualifier in a separate column. Cosmetic.

### 23. `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md` should not be in the merged PR

That file's purpose is to record that the desktop/CLI review automation failed to run. Once this manual review (or any successful one) lands, the blocked-review file is misleading — a future reader would see two review files and wonder which one is authoritative. Either:

- Delete it before merging, with the issue's `Execution Status` line updated to point at this `pass-1` review only.
- Or rename to `…-review-blocked-attempt-2026-04-29.md` and link it as an automation-failure artifact, not a review.

Right now, its presence + the unchecked checkbox suggests the review never happened, which contradicts merging the PR after this review.

---

## Migration-friendliness check

> Existing compiler emission paths are intentionally not migrated in this first wave; flag only if the new crate makes later migration harder or violates the milestone boundary.

Items above that affect later migrations:

- **#1 (LoweringOutcome missing):** every later migrating crate will need this type to land. Adding it now is free; deferring forces `milestone_diag_4a` to do it as a side-effect of routing diagnostics, which makes that PR review harder.
- **#2 (round-trip lossy):** the moment a real diagnostic stores an `Unsigned`-typed arg (e.g., a count like `arg("count", 3_u32)` — already used in tests at `model/mod.rs:642` and `666`), JSON consumers like editor LSPs round-trip through `Signed`. The `--diagnostic-format json` baseline will then disagree with the registry-declared arg type. Locking the contract now avoids breaking baselines later.
- **#3 (builder severity parameter mismatch):** every helper written in `milestone_diag_2b`/`milestone_diag_7`/`milestone_diag_8` will hard-code the current shape. Changing the signature later means rewriting every helper. Resolve before helpers proliferate.
- **#4 (release-mode drop is silent):** untestable in release until a leak counter exists. Migrating crates in `milestone_diag_4a` will accidentally drop builders with no signal in `cargo build --release`-only failure modes.
- **#5 (`SourceSpan::new` doesn't validate):** the lack of a debug-validating constructor means HIR's first migration will be writing AST-range-to-SourceSpan converters with no safety net. The first time HIR emits a span past EOF, render_span returns `Err`, and the caller has nowhere structured to send it.
- **#6 (test gaps):** delegated to migrating PRs without these tests, the regressions they would catch get blamed on the migrating PR rather than on this foundation.

The remaining items are local to `sifr_diagnostics` and don't materially raise migration cost.

---

## Risk spot-checks (no findings, recorded for the trace)

- **No public diagnostic types defined outside `sifr_diagnostics`:** confirmed via `grep -rn "SifrDiagnostic\|DiagnosticBuilder\|DiagnosticCode" crates/ | grep -v sifr_diagnostics` — empty. ✓
- **No raw `DiagnosticCode::new(...)` at non-constant call sites:** the constructor is private (`const fn new`); only `impl DiagnosticCode`'s `Self::new(...)` constants invoke it. Hard rule "Do not construct diagnostic codes with `format!` or raw strings at emission sites" is structurally enforced. ✓
- **`Severity::Help` cannot exist:** `Severity` is `Error | Warning | Note`, `ChildSeverity` is `Note | Help`, `DiagnosticSink::emit` accepts `Warning | Note`, `emit_error` accepts `Error`. ✓
- **`SourceDiagnostic` requires a span:** `primary_span: SourceSpan` (not `Option`). ✓
- **`ErrorEmitted` proof unforgeable:** `ErrorEmitted(())` private payload, only constructed inside `emit_error`. ✓ (size also confirmed zero — useful for cheap propagation.)
- **`DiagnosticArg::Float` finite-only at the `From` boundary:** asserted in `From<f64>`. The enum constructor is still public, so non-finite values can be built directly; the spec language is "must be finite" which is a *contract* stated, not enforced. Not a blocker, since direct enum-variant construction is unusual once helpers exist; revisit during `milestone_diag_2b`.
- **JSON envelope `{ "version": 1, "diagnostics": [...] }`:** matches spec verbatim. ✓
- **`DiagnosticSink` records monotonic insertion order:** confirmed; tested. ✓
- **Renderer is the only sort site:** confirmed; `render_sink` sorts before iterating. ✓
- **`SuggestionEdit.replacement` lives in suggestions only, not duplicated as `Help` child text:** structurally enforced (different fields), but the `milestone_diag_11` guardrail will need to lint helpers that copy suggestion text into help children. ✓ for now.

---

## Summary of recommended actions before merging `milestone_diag_1`

Blocking (must-fix):

1. Add `LoweringOutcome` (issue lines 706, finding #1).
2. Resolve `DiagnosticArg` round-trip lossiness and add a round-trip identity test (finding #2).
3. Reconcile `DiagnosticBuilder::source/internal` signatures with the documented API or amend the issue to match the implementation (finding #3).
4. Either implement release-mode drop reporting or amend the DoD; add `#[should_panic]` tests for the debug-mode drop discipline (finding #4).
5. Wire debug-mode `SourceSpan::new` validation, or split the constructor and document the invariant; add tests (finding #5).
6. Fill in the missing source-map test cases for byte/line/column consistency, multibyte at line boundaries, multiline highlight bounds, and CRLF handling (finding #6).

Should-fix (resolve before more crates depend on this surface):

7. Decompose `model/mod.rs` (#7).
8. Replace the `\u{10ffff}` sentinel with a typed source-vs-internal tier and pre-compute keys in `compare_diagnostics` (#8).
9. Validate builder duplicate/empty arg names; align `format_arg` for `Float` with the JSON canonical form (#11, #12).
10. Delete the `PARSE_OPAQUE_ERROR` constant for the reserved code (#13).
11. Tighten the schema sync script's failure messaging and the schema generator's error path (#18, #19).
12. Re-run `cargo clippy -p sifr_diagnostics --all-targets --tests -- -D warnings` and confirm (#21).
13. Soften `architecture.md` line 706 to reflect target shape, not current state (#22).
14. Decide what to do with `reviews/semantic-diagnostic-code-taxonomy-diag-1-review-blocked.md` (#23).

The remaining nits are optional polish.

— Pass-1 review complete.
