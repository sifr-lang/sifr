# Review Pass 1 — milestone_diag_7 Parser Diagnostic Classification

Scope under review: working-tree changes to single-file frontend, project import discovery, the new `frontend/parser_diagnostics.rs` module, the new `parser_*.sifr` e2e fail fixtures, and the new unit-test bucket added to `crates/sifr_driver/src/tests/single_file_frontend.rs`.

Read against the brief: replace broad parser diagnostics with category-specific active SIFR-PARSE-0002..0009 emissions, parse with the latest Ruff target version so Sifr-owned syntax is not version-rejected, and produce structured diagnostics whose `code` / `severity` / `message_template` / declared message arg / `parser_category` JSON arg all align with the registry.

## Verdict

**NOT MERGEABLE.** The slice's *intent* and *taxonomy mapping* are largely correct, but the patch:

1. Breaks three pre-existing `sifr_driver` unit tests because the `[label]` message-prefix contract was removed without updating callers (B1, B2, B3).
2. Fails `cargo clippy -p sifr_driver --no-deps -- -D warnings` due to a redundant closure introduced in this slice (B4). The workspace policy is `-D warnings`.
3. Renders a grammatically broken `"syntax error: expected Expected …"` whenever `ParseErrorType::OtherError(_)` is hit (M1) — and `OtherError` is the bucket Ruff reaches in our own representative fixture for `SIFR-PARSE-0002` (`def main(:`), so this is the *first* thing a user sees from the new active code.

The validation the patch author ran (`cargo test -p sifr_driver parse_source` + `cargo test -p sifr --test e2e test_e2e_fail`) does not cover any of B1–B3, and `cargo clippy` was not run. The required local validation in [AGENTS.md](../AGENTS.md) is `scripts/run_all_tests.sh --profile quick` — that gate would have caught all four blockers.

The classification module itself ([crates/sifr_driver/src/frontend/parser_diagnostics.rs](../crates/sifr_driver/src/frontend/parser_diagnostics.rs)) is well structured: exhaustive match on `ParseErrorType` (no `_` arm — good forward-pressure when Ruff is bumped), per-variant code/template/arg/category, and no fallback paths back to the broad PARSE-0002 bucket. The registry side ([crates/sifr_diagnostics/src/codes.rs:375-462](../crates/sifr_diagnostics/src/codes.rs:375)) is consistent with the emission templates and arg names.

Fix B1–B4 + M1 and a re-review pass should be short.

---

## Blockers

### B1 — `sifr_driver::tests::discovery_and_workspace::test_project_and_test_discovery_parity_reports_reachable_parse_errors` fails

[crates/sifr_driver/src/tests/discovery_and_workspace.rs:161-164](../crates/sifr_driver/src/tests/discovery_and_workspace.rs:161) still asserts:

```rust
assert!(project_errors.iter().any(|e| e.message.contains("[helper]")));
assert!(test_errors.iter().any(|e| e.message.contains("[helper]")));
```

Before this slice, [project/discovery.rs](../crates/sifr_driver/src/project/discovery.rs) prefixed parser errors with `format!("[{label}] {e}")`. After this slice, the label is moved to a `RenderedDiagnosticChild { severity: Note, message: "while parsing helper" }` and never appears in `e.message`. The assertion now flips to false. Confirmed by running the test directly:

```
thread 'tests::discovery_and_workspace::test_project_and_test_discovery_parity_reports_reachable_parse_errors' panicked at crates/sifr_driver/src/tests/discovery_and_workspace.rs:161:5:
assertion failed: project_errors.iter().any(|e| e.message.contains("[helper]"))
```

This is a contract change to the diagnostic surface that the slice's tests should cover. Either:

- Update both assertions to `e.children.iter().any(|c| c.message == "while parsing helper")`, or
- If module-prefixed parse messages are still desired (matching `FrontendDiagnosticStyle::ModulePrefixed` for HIR errors via [frontend/module_lowering.rs:46-48](../crates/sifr_driver/src/frontend/module_lowering.rs:46)), have `parse_module_with_diagnostics` accept a style argument and prefix the rendered `message` in `ModulePrefixed` mode.

### B2 — `sifr_driver::tests::project_build_check::test_check_project_reports_reachable_parse_errors_in_import_closure` fails

[crates/sifr_driver/src/tests/project_build_check.rs:273-280](../crates/sifr_driver/src/tests/project_build_check.rs:273) asserts:

```rust
assert!(
    errors.iter().any(|e| {
        e.message.contains("[helper]")
            && (e.message.contains("failed to parse")
                || e.message.contains("Expected a parameter"))
    }),
    "reachable parse errors must still fail check_project: {errors:?}"
);
```

Both halves of the substring conjunction are now false. The actual rendered diagnostic from this fixture (`def value(:\n` reachable from `main`) is:

```
RenderedDiagnostic {
  code: "SIFR-PARSE-0002",
  message: "syntax error: expected Expected a parameter or the end of the parameter list",
  message_template: "syntax error: expected {expected}",
  args: {"expected": String("Expected a parameter or the end of the parameter list"), "parser_category": String("parser_recovery")},
  children: [RenderedDiagnosticChild { severity: Note, message: "while parsing helper" }],
  …
}
```

`[helper]` is gone (now a child note), and `"failed to parse"` is gone (the slice deliberately removed that legacy prefix). The substring `"Expected a parameter"` *does* still appear, but only as a side effect of the `OtherError` rendering bug discussed in M1 — i.e., the test passes accidentally only when `OtherError` reuses Ruff's English sentence. Don't rely on that.

The test should match on `e.code == "SIFR-PARSE-0002"` (and/or any 0003-0009 code) plus a child-note check, not on the legacy free-form message.

### B3 — `sifr_driver::tests::test_runner::test_run_tests_reports_deterministic_parse_error_order` fails

[crates/sifr_driver/src/tests/test_runner.rs:308-313](../crates/sifr_driver/src/tests/test_runner.rs:308):

```rust
assert!(
    first_messages.first().is_some_and(|message| message.contains("test_a_bad.sifr")),
    "first parse error should be from lexicographically first fixture: {first_messages:?}"
);
```

`run_tests` flows through `parse_import_closure_modules` with `DiscoveryDiagnosticStyle::FilePath` ([test_runner/orchestrator.rs:49](../crates/sifr_driver/src/test_runner/orchestrator.rs:49)). Pre-slice, the file path landed in `e.message` via `[label]` and the assertion ordered fixtures by message. Post-slice, the file path lives in `e.children[0].message` (`while parsing /tmp/.../test_a_bad.sifr`) and `e.message` contains only the templated "syntax error: expected …" string with no path information. Test panics:

```
first parse error should be from lexicographically first fixture: ["syntax error: expected Expected a parameter or the end of the parameter list", "lexical error: unexpected EOF while parsing"]
```

Same fix family as B1/B2 — the test should assert on the child note, or on a path-bearing rendered field once one exists.

### B4 — Clippy violation on the new `parse_source` body

[crates/sifr_driver/src/frontend/api.rs:15](../crates/sifr_driver/src/frontend/api.rs:15):

```rust
parse_module_with_diagnostics(source, None).map(|parsed| parsed.into_suite())
```

`cargo clippy -p sifr_driver --no-deps -- -D warnings` fails:

```
error: redundant closure
  --> crates/sifr_driver/src/frontend/api.rs:15:53
   |
15 |     parse_module_with_diagnostics(source, None).map(|parsed| parsed.into_suite())
   |                                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: replace the closure with the method itself: `ruff_python_parser::Parsed::into_suite`
   = note: `-D clippy::redundant-closure-for-method-calls` implied by `-D warnings`
```

Verified by re-running clippy on `git stash` (pre-slice tree passes cleanly). Fix: `.map(Parsed::into_suite)`.

The pre-existing tree is clippy-clean for `-p sifr_driver --no-deps`, so this is a regression introduced by this slice.

---

## Major

### M1 — `OtherError` rendering produces ungrammatical "syntax error: expected Expected …" messages

[parser_diagnostics.rs:224](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:224):

```rust
ParseErrorType::OtherError(message) => expected_details(message.clone(), "parser_recovery"),
```

`expected_details` ([parser_diagnostics.rs:228-239](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:228)) hard-codes the template `"syntax error: expected {expected}"`. But Ruff's `OtherError` payload is a *full English sentence*, often beginning with "Expected …" itself, e.g. `"Expected a parameter or the end of the parameter list"` (literally what `def main(:` produces — see B2 dump above). The composed message becomes:

> syntax error: **expected Expected** a parameter or the end of the parameter list

This is the user-visible string for the slice's own `parser_expected_token.sifr` representative fixture and for the existing `helper.sifr` parse error in `test_check_project_reports_reachable_parse_errors_in_import_closure`. Two related concerns:

1. **Grammar.** Other `expected_details` callers (e.g. `ExpectedExpression → expected_details("expression", …)`) use a noun phrase, so the prefix reads correctly. `OtherError` is the only caller passing a full sentence.
2. **JSON arg semantics.** The `expected` arg ends up holding a sentence rather than the *thing expected*. Tooling consuming `args["expected"]` as structured data gets a meaningless string.

Fix options (lowest impact first):
- Give `OtherError` its own helper that uses a different template, e.g. `"syntax error during recovery: {message}"` with arg name `"message"`. The registry already declares `[arg!("expected"), json_arg!("parser_category")]` for SIFR-PARSE-0002, so changing the arg name would require a registry change. Alternatively, keep `expected` but pass a noun-phrase prefix and put the raw payload in the JSON arg.
- At minimum, drop the `"expected "` literal for the `OtherError` arm so the message is `"syntax error: <payload>"`.

Either way, the representative fixture for SIFR-PARSE-0002 should not exercise the `OtherError` path — it's strictly worse than `ExpectedToken { expected, found }`. If you can pick a source that triggers `ExpectedToken` instead (e.g. `def main):` or similar) the canonical message becomes the cleaner `"syntax error: expected <token>; found <token>"` form that `expected_details` was designed for.

---

## Minor

### m1 — Unit test asserts only the first emitted diagnostic is the expected one

[tests/single_file_frontend.rs:75-89](../crates/sifr_driver/src/tests/single_file_frontend.rs:75) uses `errors.first()` per case. Ruff's `parse_unchecked` now surfaces *all* parser errors (an intentional and good improvement over the old `parse_module().into_result()` which collapsed to the first error only — see m4). For a few of the chosen sources Ruff emits more than one diagnostic; e.g. `def value(:\n` produces SIFR-PARSE-0002 *and* SIFR-PARSE-0003 (`"unexpected EOF while parsing"`). The current cases happen to put the expected category first, but a future Ruff revision could reorder. Prefer `errors.iter().any(|d| d.code == expected_code.code())` over `.first()`.

### m2 — `EmptyDeleteTargets` is bucketed as `PARSE_INVALID_TARGET`, not `PARSE_MALFORMED_DECLARATION_LIST`

[parser_diagnostics.rs:131-133](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:131) groups `EmptyDeleteTargets` with `InvalidDeleteTarget` under PARSE-0005 with `target_kind = "delete target"`. By the *registry summaries* (PARSE-0005 = "Invalid assignment, delete, starred, or named-expression target syntax"; PARSE-0007 = "Empty or malformed declaration list syntax"), an *empty* delete target list is closer to PARSE-0007 — `Empty{Global,Nonlocal,Import}Names` and `EmptyTypeParams` already live there ([parser_diagnostics.rs:159-162](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:159)). Either bucket is defensible, but the asymmetry between "empty `del`" (PARSE-0005) and "empty `global`/`nonlocal`/`import`/`type-params`" (PARSE-0007) reads accidentally and will surprise anyone grepping by category. Recommend moving `EmptyDeleteTargets` to `declaration_list_details("delete statement")`.

### m3 — Child note emitted per-diagnostic, redundant when there are N parser errors in one module

[parser_diagnostics.rs:78-84](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:78) attaches the same `"while parsing <label>"` child to every diagnostic. For a multi-error file (e.g. unterminated string + EOF + recovery cascade) the user sees the same child note repeated N times. Acceptable for now, but if you ever introduce de-duplication at the renderer layer this is a candidate. No action required for this slice.

### m4 — Behavioral change: now reports *all* parser errors instead of only the first

This is not a regression — it's an improvement — but it is an undocumented behavior change that's worth noting in the slice's PR description. The old [api.rs](../crates/sifr_driver/src/frontend/api.rs) used `parse_module()`, which internally calls `into_result()` and discards everything but the first error. The new path uses `parse_unchecked` and iterates `parsed.errors()`. The `test_check_project_reports_reachable_parse_errors_in_import_closure` test output in B2 shows two diagnostics for one source where there used to be one. Tests asserting on `errors.len()` or "exactly one parse error" elsewhere may break — none do today, but the contract change is real.

### m5 — When `has_invalid_syntax()` is true, `unsupported_syntax_errors()` is silently dropped

[parser_diagnostics.rs:22-35](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:22) reports parser errors *or* unsupported-syntax errors, never both. If a source has both real syntax errors and version-gated features, the user only sees the parser errors. This is a defensible tradeoff (the grammar errors are usually the proximate cause) and matches the prior behavior; document it in a comment so future readers don't think it's an oversight.

### m6 — Stdlib bootstrap parser path still has the slice-2 TODO

[crates/sifr_driver/src/stdlib/bootstrap.rs:30-52](../crates/sifr_driver/src/stdlib/bootstrap.rs:30) still funnels stdlib parse failures into `STDLIB_BOOTSTRAP_FAILURE` with the original `// TODO(diag_4a slice 2): classify Ruff parse failures into the precise active parse-code buckets.` comments. Out of stated scope for *this* slice (single-file frontend + project discovery), but flag it: the same `parse_module_with_diagnostics` helper now exists and could absorb this site too — though stdlib bootstrap *intentionally* collapses to a single bootstrap-failure code, not a per-classification SIFR-PARSE-* code. If stdlib bootstrap is intended to keep the bootstrap-failure facade, add a note in [parser_diagnostics.rs](../crates/sifr_driver/src/frontend/parser_diagnostics.rs) explaining why bootstrap doesn't use it; otherwise file a follow-up issue.

### m7 — `parser_unsupported_syntax.sifr` doesn't exercise `unsupported_syntax_errors()`

The fixture (`async value\n`) trips `ParseErrorType::UnexpectedTokenAfterAsync` ([parser_diagnostics.rs:186-189](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:186)), which is a *parser error*, not a Ruff `UnsupportedSyntaxError`. So `unsupported_syntax_diagnostic` ([parser_diagnostics.rs:43-57](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:43)) — the only path that consumes Ruff's version-gated error type — has zero coverage. With `target_version = PythonVersion::latest_ty()` ( = `PY314`), the fixture menu for triggering `UnsupportedSyntaxError` is narrow (PY315-only syntax, like `t""`-strings nested in particular ways). Consider adding one such fixture or a `#[cfg(test)]` unit case so the `unsupported_syntax_diagnostic` path is exercised; otherwise, that branch is currently dead and a future Ruff bump that changes its shape won't be caught.

### m8 — `unreachable!` in `parse_module_with_diagnostics`

[parser_diagnostics.rs:19-21](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:19):

```rust
let Some(parsed) = parsed.try_into_module() else {
    unreachable!("module parse mode must produce a module syntax tree");
};
```

This is fine — it's a programmer-invariant assertion on Ruff's `Parsed<Mod> → Parsed<ModModule>` contract for `Mode::Module`, not a data-dependent panic in a user path. Matches the [AGENTS.md](../AGENTS.md) "no panics in user paths" rule. No action needed; flagging only because the milestone has unusually high panic-discipline expectations.

### m9 — `with_target_version(PythonVersion::latest_ty())` is the right call; `latest()` would also work

`PythonVersion::latest_ty() == PY314 == PythonVersion::latest()` today ([third_party/ruff/crates/ruff_python_ast/src/python_version.rs:63-77](../third_party/ruff/crates/ruff_python_ast/src/python_version.rs:63)). `latest_ty()` carries a comment about staying coupled to ty's environment defaults; `latest()` is the unconditional newest stable. Either is defensible. `latest_ty()` is slightly more conservative against ty-driven roadmap shifts, but Sifr doesn't ship ty's environment options. A 5-word comment in the call site explaining *why* `latest_ty` over `latest` would help future readers; otherwise it reads as cargo-culted. Non-blocking.

---

## Things checked and OK

- **Registry ↔ emission alignment.** Every active SIFR-PARSE-0002..0009 entry's `message_template` ([codes.rs:381,392,403,414,425,436,447,458](../crates/sifr_diagnostics/src/codes.rs:381)) matches the per-bucket helper's `template` field in [parser_diagnostics.rs](../crates/sifr_driver/src/frontend/parser_diagnostics.rs). Every `declared_args` list contains the named placeholder (`MessageAndJson`) plus `parser_category` (`JsonOnly`). The registry's `assert_template_placeholders_are_declared` test ([codes.rs:1620-1641](../crates/sifr_diagnostics/src/codes.rs:1620)) enforces this contract and will keep enforcing it.
- **Severity contract.** `parse_diagnostic` uses `code.declared_severity()` ([parser_diagnostics.rs:87](../crates/sifr_driver/src/frontend/parser_diagnostics.rs:87)) so the registry's declared severity is the source of truth — no per-emission-site override. This matches the slice's stated expectation and the registry consistency test at [codes.rs:1522-1530](../crates/sifr_diagnostics/src/codes.rs:1522).
- **`parser_category` JSON arg.** Always emitted as `DiagnosticArg::String(non_empty_str)` (verified for every variant of `parse_error_details`). The unit test at [tests/single_file_frontend.rs:84-87](../crates/sifr_driver/src/tests/single_file_frontend.rs:84) asserts non-empty.
- **Exhaustive match, no fallback.** `parse_error_details` has no `_ =>` arm — every Ruff `ParseErrorType` variant is handled. If Ruff (vendored at [third_party/ruff](../third_party/ruff/)) gains a variant, the build fails and forces re-classification. This is the right shape for a pre-production compiler; matches [AGENTS.md](../AGENTS.md): "Do NOT create fallback paths or solutions unless explicitly requested."
- **Representative-fixture file paths.** All eight `representative_fixture_path` values in the registry ([codes.rs:380,391,402,413,424,435,446,457](../crates/sifr_diagnostics/src/codes.rs:380)) point at existing files in the working tree, and each fixture's `# expect-error:` line matches the registry-declared code. `test_e2e_fail` passes (per the patch author and verified above).
- **No new `unwrap`/`expect` on user data.** The only `unreachable!` (m8) is on Ruff's `Mode::Module → ModModule` invariant; no `.unwrap()` on `parsed.errors()` or arg conversion.
- **No CI-only behavior added.** The slice doesn't introduce conditional code, env-gated branches, or feature-flagged shims. Matches "no fallback compatibility" + "pre-production" rules.

---

## Suggested fix order

1. B4 (one-line clippy fix; keeps tree green).
2. M1 (rendering bug — user-visible quality regression for the slice's headline code).
3. B1 / B2 / B3 (test contract update — straightforward `e.message` → `e.code` + `e.children[0].message` swap, *or* re-add `[label]` prefix path through `parse_module_with_diagnostics`'s new context arg if you want to preserve message-level prefixing for the project-discovery flow).
4. m2, m7 (categorization + dead-branch coverage).
5. m1, m5, m6, m9 (test brittleness + comments + follow-up note).

After fixes, re-run `scripts/run_all_tests.sh --profile quick` (the gate the brief asks for) and re-submit for pass-2.
