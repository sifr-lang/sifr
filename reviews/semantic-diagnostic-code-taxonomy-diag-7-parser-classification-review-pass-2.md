# Review Pass 2 — milestone_diag_7 Parser Diagnostic Classification

Scope under review: working-tree changes since pass 1, including the renamed [`parser_diagnostics.rs`](../crates/sifr_driver/src/frontend/parser_diagnostics.rs) (still untracked — flagging because `git status` shows it as `??`, so it must land with the slice or the slice fails to compile), the test-contract updates in [`tests/discovery_and_workspace.rs`](../crates/sifr_driver/src/tests/discovery_and_workspace.rs), [`tests/project_build_check.rs`](../crates/sifr_driver/src/tests/project_build_check.rs), [`tests/test_runner.rs`](../crates/sifr_driver/src/tests/test_runner.rs), the new classification unit-test bucket in [`tests/single_file_frontend.rs`](../crates/sifr_driver/src/tests/single_file_frontend.rs), the child-note rendering in [`sifr/src/main.rs`](../crates/sifr/src/main.rs), the helper-call refactors in [`frontend/api.rs`](../crates/sifr_driver/src/frontend/api.rs) and [`project/discovery.rs`](../crates/sifr_driver/src/project/discovery.rs), and the validation-contract update in [`manifest.json`](../verification/validation_contracts/manifest.json).

## Verdict

**Conditionally mergeable — one user-visible quality regression remains.**

Pass 1's blockers (B1–B4) are all resolved cleanly, validated locally by the author with `scripts/run_all_tests.sh --profile quick`. The structural shape of the slice — exhaustive match on `ParseErrorType`, registry-driven severity, distinct codes per category, child-note carrying module/file context — is solid. Pass-1 minors m1, m2, m5, m7, and m9 are addressed; m3 and m6 remain by design or as out-of-scope follow-ups.

The one remaining quality concern is **M1 partial fix (now M1')**: the new `recovery_expected` helper only strips a leading `"Expected "`/`"expected "` prefix from `OtherError` payloads. A non-trivial set of Ruff `OtherError` payloads do *not* start with that prefix, so the rendered message still reads `"syntax error: expected <full sentence>"` ungrammatically for those paths. This is a smaller surface than pass 1 (the most common ones — `def main(:` style — now read correctly), but it is still wrong on real fixtures. Given the slice's headline contract is "category-specific, well-formed parser diagnostics", I'd want this finalized before merge rather than carried as follow-up.

If you elect to merge with M1' as a tracked follow-up, I would still expect a regression test pinning the *rendered message string* of the SIFR-PARSE-0002 representative fixture so the carry-over is observable.

---

## Blockers cleared

### B1 → fixed
[discovery_and_workspace.rs:161-168](../crates/sifr_driver/src/tests/discovery_and_workspace.rs:161) now asserts on `e.children.iter().any(|child| child.message == "while parsing helper")` for both `project_errors` and `test_errors`. Equality match (not `contains`) is the right call — it pins the exact contract of `parser_diagnostics::parse_diagnostic`'s child-note format.

### B2 → fixed
[project_build_check.rs:273-279](../crates/sifr_driver/src/tests/project_build_check.rs:273) now asserts on `e.code == "SIFR-PARSE-0002"` *and* the child-note message — two independent signals, neither relying on Ruff's English sentence content. This is more robust than the pre-slice substring match.

### B3 → fixed
[test_runner.rs:294-333](../crates/sifr_driver/src/tests/test_runner.rs:294) destructures each diagnostic into `(message, children)` and asserts the first error's children carry the `test_a_bad.sifr` path. Determinism is asserted by full-equality of the `(message, children)` vectors across two runs. The path lives in `child.message` because [`orchestrator.rs:49`](../crates/sifr_driver/src/test_runner/orchestrator.rs:49) passes `DiscoveryDiagnosticStyle::FilePath`, which routes through `discovery_label → path.display()`. Correct.

Behavioral note: with `parse_unchecked`, all errors from the lexicographically-first failing module (`test_a_bad.sifr`) now appear at once, and `parse_import_closure_modules` short-circuits before `test_z_bad.sifr` is touched. The "first error is from `test_a_bad`" invariant therefore holds; the "deterministic across runs" invariant still holds (both runs produce identical N-error sequences). The test name still reflects the contract.

### B4 → fixed
[api.rs:15](../crates/sifr_driver/src/frontend/api.rs:15) is now `parse_module_with_diagnostics(source, None).map(sifr_python_parser::Parsed::into_suite)`. Method-reference form, clippy-clean. Verified consistent with the `Parsed::into_suite` signature in `third_party/ruff/crates/ruff_python_parser/src/lib.rs:415`.

---

## Major

### M1' — `recovery_expected` only patches a subset of `OtherError` rendering

[parser_diagnostics.rs:229-241](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:229):

```rust
ParseErrorType::OtherError(message) => {
    expected_details(recovery_expected(message), "parser_recovery")
}

fn recovery_expected(message: &str) -> String {
    message
        .strip_prefix("Expected ")
        .or_else(|| message.strip_prefix("expected "))
        .unwrap_or(message)
        .to_string()
}
```

`expected_details` hard-codes the template `"syntax error: expected {expected}"`. The strip succeeds when Ruff's payload reads "Expected …" (the common case the slice's representative fixture happens to hit), but `OtherError` is used at ~30 call sites in the vendored Ruff parser, and many of those payloads are not prefixed with "Expected"/"expected". Audited from `third_party/ruff/crates/ruff_python_parser/src/parser/`:

| Payload (source) | Rendered with current code |
|---|---|
| `"Trailing comma not allowed"` (`parser/mod.rs:661`) | `syntax error: expected Trailing comma not allowed` |
| `"Invalid mapping pattern key"` (`parser/pattern.rs:229`) | `syntax error: expected Invalid mapping pattern key` |
| `"Invalid value for a class pattern"` (`parser/pattern.rs:670`) | `syntax error: expected Invalid value for a class pattern` |
| `"missing closing bracket `]`"` (`parser/expression.rs:2004`) | `syntax error: expected missing closing bracket `]`` |
| `"missing closing brace `}`"` (`parser/expression.rs:2066`) | `syntax error: expected missing closing brace `}`` |
| `"missing closing parenthesis `)`"` (`parser/expression.rs:2176`) | `syntax error: expected missing closing parenthesis `)`` |
| `"<expr> expression cannot be used here"` (`parser/expression.rs:450`) | `syntax error: expected <expr> expression cannot be used here` |
| `"duplicate `mut` parameter modifier"` (`parser/tests.rs:214` — Sifr fork-specific) | `syntax error: expected duplicate `mut` parameter modifier` |

These all reach SIFR-PARSE-0002 via `parser_recovery`, the most popular bucket among parser errors. Users will see ungrammatical strings on common inputs (unbalanced brackets, malformed pattern matches, trailing commas, etc.).

Two issues compound:

1. **Grammar.** As above.
2. **JSON arg semantics.** `args["expected"]` ends up holding a full English sentence rather than the noun phrase the registry's documentation implies. Anything consuming structured output sees a string that's only meaningful as prose, not as a "what was expected" datum.

Suggested fix (registry-compatible): change `recovery_expected` so payloads that don't begin with `"Expected "`/`"expected "` are wrapped to read correctly under the existing template, e.g.:

```rust
fn recovery_expected(message: &str) -> String {
    if let Some(stripped) = message.strip_prefix("Expected ").or_else(|| message.strip_prefix("expected ")) {
        return stripped.to_string();
    }
    // Fallback: emit a noun-phrase the template can prefix without grammar drift.
    format!("recovery: {message}")
}
```

That keeps the registry contract (`{expected}` arg) and avoids the doubled-verb issue. Alternatively, give `OtherError` its own `(code, template, arg_name)` triple — but that requires a registry change and is heavier.

If the alternative is preferred, the registry side at [crates/sifr_diagnostics/src/codes.rs:381-385](../crates/sifr_diagnostics/src/codes.rs:381) needs a parallel update.

The slice's pass-2 author noted explicitly: *"normalizing OtherError payloads by stripping leading Expected/expected"* — so they're aware the prefix-strip is the intended fix, but the prefix space turns out to be smaller than the OtherError surface. This is a follow-up of the same root cause.

---

## Minor

### m1 — No unit test asserting the rendered human message

[main.rs:417-426](../crates/sifr/src/main.rs:417) now writes child notes after the primary line:

```rust
let _ = writeln!(output, "{label}: {message}", message = diagnostic.message);
for child in &diagnostic.children {
    let child_label = match child.severity {
        ChildSeverity::Note => "note",
        ChildSeverity::Help => "help",
    };
    let _ = writeln!(output, "{child_label}: {}", child.message);
}
```

The existing snapshot test [`test_diagnostic_formats_share_canonical_sorted_capped_stream`](../crates/sifr/src/main.rs:1436) only exercises diagnostics built with `test_diagnostic`, which initializes `children: Vec::new()`. It does not exercise the new branch. The validation-contract manifest exercises it end-to-end (`closure_neg_check` asserting `"while parsing helper"` in stderr — verified to pass) but there is no in-crate unit test pinning the format.

Recommend adding one case to `test_diagnostic_formats_share_canonical_sorted_capped_stream` (or a sibling test) that constructs a diagnostic with `children: vec![RenderedDiagnosticChild { severity: Note, message: "while parsing foo" }]` and asserts the rendered human output contains `"\nnote: while parsing foo\n"`. Trivial to add, prevents silent format drift.

### m2 — Compact format silently drops child notes

[main.rs:331-396](../crates/sifr/src/main.rs:331) groups by `(severity, code, is_summary_group, message)` and emits `at <span>` / `help` / `url` lines but never `children`. Effect: a user running with `--diagnostic-format=compact` against a project with reachable parse errors loses the `"while parsing <module>"` context.

Defensible — compact format is intentionally lossy — but worth either:
- Documenting the lossiness in a code comment, or
- Inlining the first child note after the `(xN)` line.

Non-blocking; flagging because the slice introduces children on the parser path for the first time and the asymmetry between human/JSON (which preserve them) and compact (which drops them) is now observable.

### m3 — `children` populated in JSON output for every parser diagnostic

The slice's behavior change is: every parser diagnostic now carries a `children: [{severity: "Note", message: "while parsing <label>"}]` entry when invoked through project discovery. Tools or downstream consumers that previously assumed parser diagnostics had empty `children` will see populated arrays. No in-tree consumer breaks (verified by `cargo test -p sifr diagnostic_format` per the author's run), but this is a JSON contract change worth calling out in the PR description.

### m4 — `parser_category` taxonomy has minor inconsistency for f-string/t-string errors

For lexer-emitted f-string/t-string errors:
- `ParseErrorType::FStringError(_)` / `TStringError(_)` → `parser_category = "f_string"` / `"t_string"` ([parser_diagnostics.rs:110-111](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:110))
- `ParseErrorType::Lexical(LexicalErrorType::FStringError(_))` / `LexicalErrorType::TStringError(_)` → `parser_category = "lexical_string"` ([parser_diagnostics.rs:268-269](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:268))

Same kind of error semantically; two different category strings depending on which Ruff path emits it. Anyone aggregating by `parser_category` will see two buckets where one would do. Could collapse both to `"interpolated_string"` (or split the lexical case into `lexical_f_string` / `lexical_t_string` to match) — pick one. Either way, document the convention.

Non-blocking.

### m5 — Pass-1 m3 / m6 unchanged

Both intentional:
- m3 ("redundant child note per N parser errors per module"): still produces N copies of `"while parsing <label>"` for a module with N parse errors. Acceptable; document if not already.
- m6 (stdlib bootstrap): [stdlib/bootstrap.rs:30-52](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) still funnels stdlib parse failures into `STDLIB_BOOTSTRAP_FAILURE`. The `parse_module_with_diagnostics` helper exists now and could absorb that site if the bootstrap-failure facade is to be removed. Out of scope for this slice.

If both are deferred, file a follow-up issue covering them and link from the slice PR.

### m6 — Pass-1 m9 partially addressed

[parser_diagnostics.rs:17-19](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:17) now has:

```rust
// Sifr owns the latest Python-derived syntax surface pre-1.0; do not
// reject syntax only because Ruff's default target is older.
ParseOptions::from(Mode::Module).with_target_version(PythonVersion::latest_ty()),
```

The comment justifies *why we override the default* but doesn't justify *why `latest_ty()` over `latest()`* (today both are `PY314`). Five-word add — `// latest_ty stays coupled to ty's defaults` — would close the loop. Non-blocking.

---

## Things checked and OK in pass 2

- **`lazy import value` exercises `unsupported_syntax_errors()`.** Traced through `parser/statement.rs:286-336`: on PY314 (`latest_ty()`), `lazy import …` calls `add_unsupported_syntax_error(LazyImportStatement)` and parses the import. Result: `has_invalid_syntax() == false`, `unsupported_syntax_errors().len() == 1`. The new `parse_module_with_diagnostics` correctly hits the second branch and emits SIFR-PARSE-0009 via `unsupported_syntax_diagnostic`. Pass-1 m7 ("dead branch") cleared. Confirmed by the e2e fixture and the new unit test case at [single_file_frontend.rs:69-74](../crates/sifr_driver/src/tests/single_file_frontend.rs:69).

- **`EmptyDeleteTargets` reclassification.** [parser_diagnostics.rs:138](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:138) now routes through `declaration_list_details("delete statement")` → SIFR-PARSE-0007. Symmetrical with `EmptyGlobalNames` / `EmptyNonlocalNames` / `EmptyImportNames` / `EmptyTypeParams`. Pass-1 m2 cleared.

- **Parser-error-precedence comment.** [parser_diagnostics.rs:25-26](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:25) now documents why `unsupported_syntax_errors` is short-circuited when `has_invalid_syntax()` — addresses pass-1 m5.

- **Unit test searches all errors.** [single_file_frontend.rs:79-87](../crates/sifr_driver/src/tests/single_file_frontend.rs:79) uses `errors.iter().find(|d| d.code == expected_code.code())` with a `unwrap_or_else(|| panic!(…))` for diagnostic context. Robust against Ruff reordering errors. Pass-1 m1 cleared. The test correctly validates: code, severity (vs `declared_severity()`), presence of the per-bucket arg, presence of `parser_category` as a non-empty string, and that `message_template` isn't the `"{message}"` legacy placeholder. Eight cases cover each of SIFR-PARSE-0002…0009.

- **Validation contract update is consistent.** [manifest.json:200](../verification/validation_contracts/manifest.json:200) now asserts `"while parsing helper"` in stderr for the reachable-parse-error contract, matching the new child-note format. The `helper.sifr` assertion on `closure_neg_test` continues to work because `DiscoveryDiagnosticStyle::FilePath` renders the path in the child note (via `discovery_label → path.display()`).

- **Child-note + JSON envelope contract.** `RenderedDiagnostic.children` is `Vec<RenderedDiagnosticChild>` ([crates/sifr_diagnostics/src/render/mod.rs:32,40-43](../crates/sifr_diagnostics/src/render/mod.rs:32)). Schema type is part of the public envelope (`#[derive(JsonSchema)]`), so additions don't break consumers; child-note is a non-breaking enrichment.

- **`Parsed::into_suite` referenced via fully-qualified path in `api.rs`.** Avoids the redundant-closure clippy lint and avoids needing an additional import. Clean.

- **Stdlib bootstrap unaffected.** [stdlib/bootstrap.rs](../crates/sifr_driver/src/stdlib/bootstrap.rs) still uses its own `parse_module` path and still emits `STDLIB_BOOTSTRAP_FAILURE`. Confirmed by `git status` — the file isn't modified, so no regression risk for stdlib bootstrap behavior. (The pass-1 m6 follow-up is still tracked.)

- **No new `unwrap`/`expect`/`panic!` on user data paths.** The only `unreachable!` is the `Parsed::try_into_module()` invariant under `Mode::Module` ([parser_diagnostics.rs:21-23](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:21)), which is a programmer-invariant assertion against Ruff's API contract — not a data-dependent panic. Matches AGENTS.md.

- **Author's local-validation gate.** The reported run includes `scripts/run_all_tests.sh --profile quick` (the AGENTS.md gate) plus targeted `cargo test`, `cargo clippy`, `cargo fmt --check`, `bash scripts/run_validation_contract_matrix.sh`. The clippy + project-test failures from pass 1 are no longer present.

---

## Suggested fix order

1. **M1'** — extend `recovery_expected` (or split `OtherError` into its own helper) so non-`Expected `-prefixed payloads still produce grammatical messages. Add a regression test pinning the rendered SIFR-PARSE-0002 message for at least one such payload (e.g., a fixture that triggers `"missing closing bracket"`).
2. **m1** — add a unit test in `crates/sifr/src/main.rs` covering `render_diagnostic_stream` with a non-empty `children` vec for the human format.
3. **m2/m3/m4** — pick: address now (small) or file follow-ups.
4. **m5/m6** — link pass-1 m3/m6/m9 deferrals from the slice PR description.

Not satisfied yet — I'd want M1' addressed before merge; everything else is non-blocking and could ride a follow-up.
