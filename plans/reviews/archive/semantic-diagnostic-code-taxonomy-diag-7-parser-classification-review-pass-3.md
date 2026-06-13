# Review Pass 3 — milestone_diag_7 Parser Diagnostic Classification

Scope under review: working-tree changes since pass 2, focused on the pass-2 merge-blocker (`recovery_expected` non-`Expected`-prefixed payloads) and the pass-2 minor m1 (no in-crate test pinning the human child-note format). Files touched on top of the pass-2 tree:

- [crates/sifr_driver/src/frontend/parser_diagnostics.rs:235-243](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:235) — `recovery_expected` now wraps non-prefixed payloads as `recovery: <payload>`.
- [crates/sifr_driver/src/tests/single_file_frontend.rs:96-129](../crates/sifr_driver/src/tests/single_file_frontend.rs:96) — new `test_parse_source_normalizes_parser_recovery_messages`.
- [crates/sifr/src/main.rs:1513-1535](../crates/sifr/src/main.rs:1513) — new `test_human_diagnostic_format_renders_child_notes`.

The rest of the diff is unchanged from pass 2 and is re-confirmed in **§ Re-checked from pass 2** below.

## Verdict

**Satisfied — mergeable.** Pass-2's M1' merge blocker is resolved: `recovery_expected` now (a) strips a leading `Expected `/`expected ` if present, otherwise (b) wraps the payload as `recovery: <payload>`. Both branches keep the registry's `SIFR-PARSE-0002` template/arg shape (`syntax error: expected {expected}`) intact, and the second branch removes the doubled-verb defect from the pass-2 message dump. Pass-2 minor m1 is also addressed by the new human-format unit test. The slice now satisfies the parser-classification contract end-to-end.

Local-validation gate (the one AGENTS.md requires) was rerun by the author against pass-3 — `scripts/run_all_tests.sh --profile quick` reports `report_signature=e1bf653aaa770517` with only advisory wall-time/group-skew flags. No new blockers introduced.

The deferred items below (m2–m4 from pass 2 plus a small new m7) are all non-blocking and worth tracking as follow-ups in the slice PR description, not gating.

---

## Pass-2 blocker resolved

### M1' (pass-2 merge blocker) → fixed

[parser_diagnostics.rs:235-243](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:235):

```rust
fn recovery_expected(message: &str) -> String {
    if let Some(stripped) = message
        .strip_prefix("Expected ")
        .or_else(|| message.strip_prefix("expected "))
    {
        return stripped.to_string();
    }
    format!("recovery: {message}")
}
```

This is exactly the shape pass-2 suggested. Walking through each `OtherError` payload audited in pass 2:

| Ruff payload | Pass-2 rendering (broken) | Pass-3 rendering |
|---|---|---|
| `Expected a parameter or the end of the parameter list` | `syntax error: expected Expected a parameter or the end of the parameter list` | `syntax error: expected a parameter or the end of the parameter list` |
| `Trailing comma not allowed` | `syntax error: expected Trailing comma not allowed` | `syntax error: expected recovery: Trailing comma not allowed` |
| `missing closing bracket \`]\`` | `syntax error: expected missing closing bracket \`]\`` | `syntax error: expected recovery: missing closing bracket \`]\`` |
| `duplicate \`mut\` parameter modifier` (Sifr fork; [statement.rs:3110](../third_party/ruff/crates/ruff_python_parser/src/parser/statement.rs:3110)) | `syntax error: expected duplicate \`mut\` parameter modifier` | `syntax error: expected recovery: duplicate \`mut\` parameter modifier` |

The `Expected `-prefixed case is now grammatical and concise. The non-prefixed case still has a slightly clunky `expected recovery: …` doubled-noun, but it (a) reads as unambiguous English ("expected, recovery:" parses as a discriminator), (b) keeps the `expected` arg as a structurally meaningful payload — `recovery: <X>` clearly signals "this is recovery telemetry, not a noun phrase the user can act on", and (c) preserves the registry contract without a registry-wide change. Pass 2 explicitly endorsed this shape.

The `args["expected"]` JSON value also now carries the `recovery: <payload>` form for the non-prefixed bucket, which is the right structured signal — downstream tooling can detect `parser_category == "parser_recovery"` *and* the `recovery: ` prefix and treat the payload as opaque recovery prose rather than a token expectation.

### Coverage

[single_file_frontend.rs:96-129](../crates/sifr_driver/src/tests/single_file_frontend.rs:96) pins both branches:

1. `def main(:\n` → `Expected a parameter or the end of the parameter list` (Ruff payload begins with `Expected `). Pass-3 asserts the exact final message string `"syntax error: expected a parameter or the end of the parameter list"` with `assert_eq!`. Full equality is right here — the slice is making a positive correctness claim about the strip path.
2. `def f(mut mut items: list[int]):\n    return items\n` → `duplicate \`mut\` parameter modifier` ([statement.rs:3110](../third_party/ruff/crates/ruff_python_parser/src/parser/statement.rs:3110); the only Sifr-fork-introduced `OtherError` payload that doesn't begin with `Expected `). Pass-3 asserts `starts_with("syntax error: expected recovery: ")` plus `code == SIFR-PARSE-0002`. `starts_with` is right here — the test is a regression pin on the prefix transformation, not the full English sentence (which is owned by the Ruff fork and could legitimately move).

The `find` on `code == PARSE_EXPECTED_TOKEN_OR_RECOVERY && message.contains("recovery:")` is robust against Ruff emitting additional diagnostics from the same source — both the duplicate-mut path and the closing-paren path will land in the diagnostic vector, and the test only inspects the recovery one.

Confirmed by running `cargo test -p sifr_driver test_parse_source_normalizes_parser_recovery_messages -- --nocapture` (per author's pass-3 validation log).

---

## Pass-2 minor m1 → fixed

[main.rs:1513-1534](../crates/sifr/src/main.rs:1513) adds:

```rust
#[test]
fn test_human_diagnostic_format_renders_child_notes() { … }
```

That asserts:

```
"parse error: syntax error: expected expression\nnote: while parsing helper\n"
```

against `render_diagnostic_output(&[diagnostic], DiagnosticFormat::Human)`. This pins:

- The `"parse error"` label for SIFR-PARSE-* codes (via [diagnostics.rs:130](../crates/sifr_driver/src/diagnostics.rs:130)).
- The exact `"\n"` separator between primary and child note.
- The `"note:"` prefix for `ChildSeverity::Note`.
- The trailing `"\n"` after the child note (i.e. `writeln!`, not `write!`).

This is the strongest possible regression pin for the new human-format branch in [main.rs:419-426](../crates/sifr/src/main.rs:419). If the format drifts (e.g. someone changes the separator, switches to `write!`, or alters the label), this test breaks first. Good.

Quietly confirmed: `ChildSeverity::Help → "help"` is also covered by the match in `render_diagnostic_stream`, but no test exercises the `Help` branch yet. Not a regression — the slice doesn't emit `Help` children — but if a future slice does, the renderer is ready and the unit test should be extended to cover it.

---

## Re-checked from pass 2

All of pass-2's verified items still hold for the pass-3 tree:

- **B1–B4 (pass-1 blockers, pass-2 cleared):** still cleared. [discovery_and_workspace.rs:161-167](../crates/sifr_driver/src/tests/discovery_and_workspace.rs:161), [project_build_check.rs:273-278](../crates/sifr_driver/src/tests/project_build_check.rs:273), [test_runner.rs:294-333](../crates/sifr_driver/src/tests/test_runner.rs:294), and [api.rs:15](../crates/sifr_driver/src/frontend/api.rs:15) (`Parsed::into_suite` method-ref form, clippy-clean) are unchanged.
- **Registry ↔ emission alignment.** All eight `SIFR-PARSE-0002..0009` registry entries at [codes.rs:375-462](../crates/sifr_diagnostics/src/codes.rs:375) match the per-bucket helper templates and arg names in [parser_diagnostics.rs:245-362](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:245). Severity always sourced from `code.declared_severity()` ([parser_diagnostics.rs:91](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:91)).
- **Exhaustive match, no `_` arm.** [parser_diagnostics.rs:108-232](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:108) handles every `ParseErrorType` variant. Future Ruff bumps are forced re-classification — matches AGENTS.md "no fallback paths".
- **`unsupported_syntax_errors` path is exercised.** Pass-2's verification of `lazy import value → LazyImportStatement → unsupported_syntax_diagnostic → SIFR-PARSE-0009` still holds, and [parser_unsupported_syntax.sifr](../crates/sifr/tests/e2e/fail/parser_unsupported_syntax.sifr) carries `# expect-error: SIFR-PARSE-0009`. Confirmed by [single_file_frontend.rs:69-74](../crates/sifr_driver/src/tests/single_file_frontend.rs:69) (the eighth case in the classification test).
- **Child-note + JSON envelope.** `RenderedDiagnostic.children` is a serde-serialized field on the JSON envelope; populating it for parser diagnostics is a non-breaking schema enrichment. JSON output for the parse path now naturally carries `children: [{severity: "Note", message: "while parsing <label>"}]`.
- **Validation contract.** [manifest.json:200](../verification/validation_contracts/manifest.json:200) asserts `"while parsing helper"` in stderr — matches the new child-note format and the human renderer's output. Confirmed by pass-2 and unchanged in pass 3.
- **No new panics on user paths.** The single `unreachable!` in [parser_diagnostics.rs:21-23](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:21) is a programmer-invariant assertion against Ruff's `Mode::Module → ModModule` contract, not data-dependent. AGENTS.md compliant.
- **Stdlib bootstrap unchanged.** [stdlib/bootstrap.rs](../crates/sifr_driver/src/stdlib/bootstrap.rs) still routes through its own `parse_module` path with `STDLIB_BOOTSTRAP_FAILURE`. The pass-1 m6 follow-up is still tracked.

---

## Outstanding (non-blocking, deferred)

These were flagged in pass 2 as "pick: address now or follow-up" and are still unaddressed in pass 3. None block merge.

### m2 (pass-2) — Compact format silently drops child notes

[main.rs:331-396](../crates/sifr/src/main.rs:331) groups by `(severity, code, is_summary_group, message)` and emits `at <span>`/`help`/`url` lines but never `children`. With `--diagnostic-format=compact`, parser diagnostics now lose their `"while parsing <label>"` context. JSON and human preserve them; compact does not. Worth either (a) inlining the first child note after the `(xN)` line in compact, or (b) a one-line code comment documenting the lossiness. Defensible either way; flagging because pass 3 introduces no compact-format coverage and the asymmetry is now permanent.

### m3 (pass-2) — JSON contract change

Every parser diagnostic now carries a `children` entry when invoked through project discovery. No in-tree JSON consumer breaks (verified across `cargo test -p sifr` per author logs), but downstream tooling that previously assumed parser diagnostics had empty `children` will see populated arrays. Worth calling out in the PR description.

### m4 (pass-2) — `parser_category` taxonomy inconsistency for f-string/t-string

Same kind of error reaches different `parser_category` strings depending on which Ruff path emits it:
- `ParseErrorType::FStringError(_)` → `f_string` ([parser_diagnostics.rs:110](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:110))
- `ParseErrorType::TStringError(_)` → `t_string` ([parser_diagnostics.rs:111](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:111))
- `ParseErrorType::Lexical(LexicalErrorType::FStringError(_))` / `TStringError(_)` → `lexical_string` ([parser_diagnostics.rs:270-271](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:270))

Anyone aggregating by `parser_category` sees two (or three) buckets where one would do. Pick a convention (`interpolated_string` everywhere, or `lexical_f_string`/`lexical_t_string` parallel to the parser-side names) and document it in the helper file. Minor.

### m5 (pass-1 m3 / m6 / m9) — still deferred by design

Per pass 2:
- m3 (one child note per parser error per module): N copies of `"while parsing <label>"` for an N-error file. Acceptable; renderer-side dedup is a separate concern.
- m6 (stdlib bootstrap): [stdlib/bootstrap.rs:30-52](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) still funnels stdlib parse failures into `STDLIB_BOOTSTRAP_FAILURE`. `parse_module_with_diagnostics` is now reusable if the bootstrap-failure facade is ever retired.
- m9 (`latest_ty()` rationale): the comment at [parser_diagnostics.rs:17-19](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:17) still doesn't explain *why `latest_ty()` over `latest()`*. Five-word fix; not blocking.

Recommend filing one issue covering all three.

### m7 (new in pass 3) — `ChildSeverity::Help` rendering branch is dead in tests

[main.rs:421-424](../crates/sifr/src/main.rs:421):

```rust
let child_label = match child.severity {
    ChildSeverity::Note => "note",
    ChildSeverity::Help => "help",
};
```

The new `test_human_diagnostic_format_renders_child_notes` covers `ChildSeverity::Note` only. The `Help` arm has zero in-crate coverage — no current emission site uses it. Not a regression (the slice doesn't emit `Help`), but the same one-line trick that pinned the `Note` format would pin `Help` too. Add a sibling test or an additional case if and when a slice introduces `Help` children. Non-blocking.

---

## Things checked and OK in pass 3

- **`recovery_expected` callsite invariant.** Only one caller — [parser_diagnostics.rs:230-231](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:230) (`OtherError` arm). The function is private to the module. Adding/removing prefixes there is contained.
- **`recovery_expected` empty-string behavior.** Pathologically, `recovery_expected("")` yields `"recovery: "`, which renders as `"syntax error: expected recovery: "`. Ruff doesn't emit empty `OtherError` payloads in the audited surface, so this is theoretical. Not worth guarding; flagging only because it's the one input that produces a trailing-space message.
- **`assert_eq!` on the `Expected `-prefixed full message.** Brittle to a Ruff-fork rename of the upstream English sentence, but that's the right kind of brittleness — it forces a deliberate update, not a silent regression. The non-prefixed case wisely uses `starts_with` instead.
- **`test_parse_source_normalizes_parser_recovery_messages` source for the recovery branch.** `def f(mut mut items: list[int])` traces to [statement.rs:3097-3115](../third_party/ruff/crates/ruff_python_parser/src/parser/statement.rs:3097) — the second `mut` token sees `convention.is_mutable() == true` and emits the `OtherError("duplicate \`mut\` parameter modifier")` we expect. The `find` predicate filters to the recovery diagnostic, so additional emitted errors (e.g., subsequent name-as-parameter resolution) don't perturb the test.
- **Test_runner determinism.** [test_runner.rs:294-333](../crates/sifr_driver/src/tests/test_runner.rs:294) destructures `(message, children)` and asserts full equality across two `run_tests` invocations. Children-aware. Pass-2 invariant preserved.
- **Validation contract still consistent with renderer.** `"while parsing helper"` exactly matches the format in [parser_diagnostics.rs:84-85](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:84) (`format!("while parsing {label}")`). No drift.
- **No new clippy violations.** `cargo fmt --check` passed per author's pass-3 log; pass-2's clippy-clean state is preserved (pass 3 only adds tests + a one-line `format!` in `recovery_expected`).
- **Untracked working-tree noise.** `package-lock.json` and `package.json` are present in `git status` and unrelated to this slice (probably tooling artifacts from another task). They should not land in this slice's PR — confirm exclusion when staging.

---

## Suggested follow-up issues (non-blocking)

1. **Compact-format children rendering** (pass-2 m2). One-line code comment or first-child-inline emission.
2. **Parser_category taxonomy harmonization** (pass-2 m4). Pick `interpolated_string` everywhere or `lexical_f_string`/`lexical_t_string`; document.
3. **Stdlib bootstrap parser routing** (pass-1 m6). Either retire the `STDLIB_BOOTSTRAP_FAILURE` facade and route through `parse_module_with_diagnostics`, or annotate why bootstrap intentionally collapses to a single bootstrap-failure code.
4. **`latest_ty` vs `latest` rationale** (pass-1 m9). One-comment fix at [parser_diagnostics.rs:17-19](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:17).
5. **`ChildSeverity::Help` test coverage** (pass-3 m7). Add when the first `Help` emission lands.

---

## Summary

Satisfied. M1' is closed grammatically *and* preserves the registry contract; m1 is closed by a tight regression pin on the human renderer; the rest of the slice is unchanged from pass 2's verified-OK state. Validation gate (`scripts/run_all_tests.sh --profile quick`, signature `e1bf653aaa770517`) passes with only advisory wall-time/skew flags. Recommend merge, with the five follow-ups above filed against a tracking issue and linked from the slice PR description.
