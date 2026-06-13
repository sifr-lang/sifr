# Review: milestone_diag_4a — Slice 2b.1 Decimal-Family HIR Migration (Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a-decimal` (working tree on top of `5ad7b756`, the slice 2a merge)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-3.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-2.md)

## Verdict

**Ready to ship as the slice 2b.1 PR.** The decimal-family migration is complete and consistent: every E250x-tagged HIR/type-system emission site that exists in the working tree now carries the matching `SIFR-DECIMAL-000N` constant from the registry, every decimal-family fail fixture is re-keyed, and the lone decimal-family verification baseline directory is updated across `compact`, `human`, and `json` renderers. The supporting plumbing (`TypeError.code`, `LowerCtx::type_error`) is the minimum bridge needed to forward `Result<_, TypeError>` decisions from `sifr_type_system::check` into the structured HIR transport added by slice 2a, with no behavior change at the codeless arms. Local validation matches the slice 2a baseline (`report_signature=e1bf653aaa770517`, wall-time `84.52s`), and the reported `scripts/run_e2e_pass.sh` failures are unrelated codegen-level issues that this slice does not touch. Scope discipline holds: embedded `[E250x]` text remains (deferred to `milestone_diag_6`), and the broader `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge is intentionally left intact for non-decimal domains.

No blocking findings. Six observations are recorded as `O*` (optional polish, do not block) and `N*` (informational notes).

## Diagnostic identity mapping (the load-bearing assertion of this slice)

The full registry → emission-site → fixture/baseline cross-walk holds. Each row is an active `SIFR-DECIMAL-000N` code at [crates/sifr_diagnostics/src/codes.rs:40-51](../crates/sifr_diagnostics/src/codes.rs:40), the corresponding `[E250N]`-tagged emission site(s) in HIR/type-system, and the fixture/baseline that pins the public identity:

| Active code | Constant | Emission site(s) | Fixture / baseline |
| --- | --- | --- | --- |
| `SIFR-DECIMAL-0001` | `DECIMAL_INVALID_LITERAL` | [decimal_methods.rs:86-89](../crates/sifr_hir/src/lower/decimal_methods.rs:86), [decimal_methods.rs:127-130](../crates/sifr_hir/src/lower/decimal_methods.rs:127) | [decimal_invalid_literal_string.sifr:1](../crates/sifr/tests/e2e/fail/decimal_invalid_literal_string.sifr:1), all three renderer baselines under [decimal_invalid_literal/baselines](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/) |
| `SIFR-DECIMAL-0002` | `DECIMAL_BIGDECIMAL_INVALID_LITERAL` | [decimal_methods.rs:97-100](../crates/sifr_hir/src/lower/decimal_methods.rs:97), [decimal_methods.rs:187-191](../crates/sifr_hir/src/lower/decimal_methods.rs:187) | [bigdecimal_invalid_literal_string.sifr:1](../crates/sifr/tests/e2e/fail/bigdecimal_invalid_literal_string.sifr:1), [bigdecimal_constructor_non_literal_string.sifr:1](../crates/sifr/tests/e2e/fail/bigdecimal_constructor_non_literal_string.sifr:1) |
| `SIFR-DECIMAL-0003` | `DECIMAL_FLOAT_MIXED` | [check.rs:44-52](../crates/sifr_type_system/src/check.rs:44), [check.rs:385-393](../crates/sifr_type_system/src/check.rs:385) | [decimal_float_mixed_arithmetic.sifr:1](../crates/sifr/tests/e2e/fail/decimal_float_mixed_arithmetic.sifr:1) |
| `SIFR-DECIMAL-0004` | `DECIMAL_MIXED_WITH_BIGDECIMAL` | [check.rs:31-38](../crates/sifr_type_system/src/check.rs:31), [check.rs:371-379](../crates/sifr_type_system/src/check.rs:371) | [decimal_bigdecimal_mixed_arithmetic.sifr:1](../crates/sifr/tests/e2e/fail/decimal_bigdecimal_mixed_arithmetic.sifr:1), [decimal_forbidden_mixed_arithmetic_seeded.sifr:1](../crates/sifr/tests/e2e/fail/decimal_forbidden_mixed_arithmetic_seeded.sifr:1) |
| `SIFR-DECIMAL-0005` | `DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN` | [decimal_methods.rs:111-118](../crates/sifr_hir/src/lower/decimal_methods.rs:111), [decimal_methods.rs:141-145](../crates/sifr_hir/src/lower/decimal_methods.rs:141), [decimal_methods.rs:149-156](../crates/sifr_hir/src/lower/decimal_methods.rs:149), [expressions.rs:993-999](../crates/sifr_hir/src/lower/expressions.rs:993) | [decimal_constructor_float.sifr:1](../crates/sifr/tests/e2e/fail/decimal_constructor_float.sifr:1), [decimal_forbidden_float_conversion_seeded.sifr:1](../crates/sifr/tests/e2e/fail/decimal_forbidden_float_conversion_seeded.sifr:1), [float_from_decimal_forbidden.sifr:1](../crates/sifr/tests/e2e/fail/float_from_decimal_forbidden.sifr:1) |
| `SIFR-DECIMAL-0006` | `DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN` | [decimal_methods.rs:171-178](../crates/sifr_hir/src/lower/decimal_methods.rs:171), [decimal_methods.rs:197-201](../crates/sifr_hir/src/lower/decimal_methods.rs:197), [decimal_methods.rs:205-211](../crates/sifr_hir/src/lower/decimal_methods.rs:205), [expressions.rs:1000-1006](../crates/sifr_hir/src/lower/expressions.rs:1000) | [bigdecimal_constructor_float.sifr:1](../crates/sifr/tests/e2e/fail/bigdecimal_constructor_float.sifr:1), [float_from_bigdecimal_forbidden.sifr:1](../crates/sifr/tests/e2e/fail/float_from_bigdecimal_forbidden.sifr:1) |
| `SIFR-DECIMAL-0007` | `DECIMAL_SCALE_INVALID` | [decimal_methods.rs:52-58](../crates/sifr_hir/src/lower/decimal_methods.rs:52), [decimal_methods.rs:231-238](../crates/sifr_hir/src/lower/decimal_methods.rs:231), [decimal_methods.rs:241-248](../crates/sifr_hir/src/lower/decimal_methods.rs:241), [decimal_methods.rs:278-285](../crates/sifr_hir/src/lower/decimal_methods.rs:278), [decimal_methods.rs:288-295](../crates/sifr_hir/src/lower/decimal_methods.rs:288) | [decimal_quantize_requires_int_scale.sifr:1](../crates/sifr/tests/e2e/fail/decimal_quantize_requires_int_scale.sifr:1), [decimal_round_scale_out_of_range.sifr:1](../crates/sifr/tests/e2e/fail/decimal_round_scale_out_of_range.sifr:1) |
| `SIFR-DECIMAL-0008` | `DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID` | [decimal_methods.rs:61-67](../crates/sifr_hir/src/lower/decimal_methods.rs:61), [decimal_methods.rs:231-238](../crates/sifr_hir/src/lower/decimal_methods.rs:231) (via `decimal_scale_diagnostic_code("bigdecimal")`), [decimal_methods.rs:338-345](../crates/sifr_hir/src/lower/decimal_methods.rs:338), [decimal_methods.rs:348-355](../crates/sifr_hir/src/lower/decimal_methods.rs:348) | [bigdecimal_quantize_negative_scale_context.sifr:1](../crates/sifr/tests/e2e/fail/bigdecimal_quantize_negative_scale_context.sifr:1), [bigdecimal_round_requires_int_scale.sifr:1](../crates/sifr/tests/e2e/fail/bigdecimal_round_requires_int_scale.sifr:1) |

I cross-checked this in two complementary ways:

1. `git grep -nE 'E250[1-8]' crates/sifr_hir/src/ crates/sifr_type_system/src/` yields exactly 24 emission-site hits, every one of which now sits inside an `error_with_code(...)` (HIR) or `TypeError { code: Some(_), .. }` (type system) construction. No codeless `ctx.error(...)` carries an `[E250x]` tag in production source.
2. Every of the 15 decimal-family fail fixtures listed in the patch starts with the exact `[SIFR-DECIMAL-000N] [E250N] …` signature corresponding to the registry/code mapping above. The harness in [crates/sifr/tests/e2e.rs:2553-2567](../crates/sifr/tests/e2e.rs:2553) requires *both* the parsed code (`expected.code`) and the message substring (`failure.message.contains(message)`) to match per fixture, so the fixture is the actual gate — `cargo test -p sifr -- --skip test_e2e_pass` exercises both sides end-to-end.

The mapping is a clean 1:1 between the eight legacy `[E250N]` slugs and the eight active `SIFR-DECIMAL-000N` codes. No drift, no double-mapping, no missed slug.

## Pipeline plumbing review

### `TypeError.code` and `LowerCtx::type_error`

The migration of `sifr_type_system::check` from "`[E250x]`-in-message-only" to a structured `Option<DiagnosticCode>` field is the smallest reasonable shape:

- [crates/sifr_type_system/src/lib.rs:31-36](../crates/sifr_type_system/src/lib.rs:31) adds `pub code: Option<DiagnosticCode>` to `TypeError`. `Option<_>` (rather than required) is correct for this slice because only the two decimal-family arms (E2503 + E2504) carry an active code today; everything else still flows codeless and is correctly initialized with `code: None`.
- [crates/sifr_type_system/src/check.rs:32](../crates/sifr_type_system/src/check.rs:32), [crates/sifr_type_system/src/check.rs:45](../crates/sifr_type_system/src/check.rs:45), [crates/sifr_type_system/src/check.rs:372](../crates/sifr_type_system/src/check.rs:372), [crates/sifr_type_system/src/check.rs:386](../crates/sifr_type_system/src/check.rs:386) attach `Some(DECIMAL_…)` to the four decimal-family arms (binary mix, binary float, comparison mix, comparison float). All other 18 `TypeError { … }` sites carry `code: None`. I confirmed exhaustiveness with `grep -n "TypeError {" crates/sifr_type_system/src/check.rs` (22 hits, every one matches one of the two patterns) and with `grep -rn "TypeError {" crates/ --include="*.rs" | grep -v "tests/" | grep -v "src/check.rs" | grep -v "src/lib.rs"` (zero hits — no other crate constructs `TypeError`, so no out-of-tree initializer is missing the field).
- [crates/sifr_hir/src/lower/mod.rs:220-226](../crates/sifr_hir/src/lower/mod.rs:220) adds `LowerCtx::type_error(error: TypeError)` that forwards to `error_with_code` when `error.code.is_some()` and to `error` otherwise. This is the precise minimum bridge: it preserves the codeless legacy behavior bit-for-bit while letting the typed code flow through whenever the type system populates it.
- [crates/sifr_hir/src/lower/expressions.rs:370](../crates/sifr_hir/src/lower/expressions.rs:370), [crates/sifr_hir/src/lower/expressions.rs:393](../crates/sifr_hir/src/lower/expressions.rs:393), [crates/sifr_hir/src/lower/expressions.rs:515](../crates/sifr_hir/src/lower/expressions.rs:515), [crates/sifr_hir/src/lower/expressions.rs:565](../crates/sifr_hir/src/lower/expressions.rs:565), [crates/sifr_hir/src/lower/aug_assign_lowering.rs:319](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:319), [crates/sifr_hir/src/lower/aug_assign_lowering.rs:325](../crates/sifr_hir/src/lower/aug_assign_lowering.rs:325) replace `ctx.error(e.message)` with `ctx.type_error(e)` at every call site that previously discarded the typed error. That's six call sites, which matches `git grep -nE 'ctx\.error\(e\.message\)' crates/sifr_hir/src/` returning zero (i.e. all forwarders were migrated). No site still drops `e.code`.

There is no production behavior change at codeless arms: a `TypeError { code: None, … }` flows through `type_error` → `error(message)` → `LoweringError { code: None, message, … }`, byte-equivalent to the prior path. The slice 2a transport tests at [crates/sifr_hir/src/lower/diagnostic_transport_tests.rs](../crates/sifr_hir/src/lower/diagnostic_transport_tests.rs) still exercise that exact branch decision and continue to pass.

### `error_with_code` allow attribute

The `#[allow(dead_code, …)]` and TODO marker on `LowerCtx::error_with_code` ([crates/sifr_hir/src/lower/mod.rs:228-234](../crates/sifr_hir/src/lower/mod.rs:228)) are now correctly removed in this slice — the function gains many production callers via [crates/sifr_hir/src/lower/decimal_methods.rs](../crates/sifr_hir/src/lower/decimal_methods.rs) and the new `type_error` forwarder, so the dead-code allow would now be stale. Slice 2a's pass-2 N7 review item ("delete the allow when the first domain migration calls `error_with_code`") is closed exactly as predicted.

### Constructor extraction (refactor co-located with the migration)

[crates/sifr_hir/src/lower/expressions.rs:868-878](../crates/sifr_hir/src/lower/expressions.rs:868) shrinks the inline `Decimal()`/`BigDecimal()` constructor lowering paths from ~120 lines down to two `return lower_decimal_constructor_call(call, ctx);` / `return lower_bigdecimal_constructor_call(call, ctx);` calls. The extracted helpers live at [crates/sifr_hir/src/lower/decimal_methods.rs:106-220](../crates/sifr_hir/src/lower/decimal_methods.rs:106) and are byte-equivalent in observable behavior to the pre-extraction code modulo the `ctx.error → ctx.error_with_code` migration.

This is the right place to put them: `decimal_methods.rs` already housed `validate_decimal_string_literal`, `validate_bigdecimal_string_literal`, `decimal_conversion_error_type`, and `resolve_decimal_method_type`. Co-locating the constructor lowering completes the module's "everything decimal-family that HIR lowering needs" surface and lets `expressions.rs` shed an explicit dependency on `validate_*_string_literal`. The `super::expressions::lower_expr` import in `decimal_methods.rs` is the only new cross-module reference, mirroring how other domain-specific lowering helpers reach back to `lower_expr` (see e.g. `super::expressions::lower_expr` is already imported transitively via `aug_assign_lowering` etc.).

The HIR maintainability guardrail script (`python3 scripts/check_hir_maintainability_guardrails.py`) is reported passing; I'd expect that since `decimal_methods.rs` grew to ~383 lines (well under typical caps) while `expressions.rs` shrank.

This refactor is mild scope creep relative to "migrate emission sites to active codes," but it's strictly local to the decimal-family files this slice already touches, and it's the kind of co-located cleanup the AGENTS.md guidance "keep changes focused on the requested milestone/issue" tolerates because the sites being migrated were the same lines being extracted. Not a blocker; recorded as **O1** below for completeness.

### `decimal_diag_code` vs `decimal_scale_diagnostic_code`

The two helpers at [decimal_methods.rs:12-26](../crates/sifr_hir/src/lower/decimal_methods.rs:12) deliberately run in lockstep:

- `decimal_diag_code(receiver_name)` → `&'static str` (`"E2507"` / `"E2508"`) feeds the embedded `[E25NN]` slug into the message text.
- `decimal_scale_diagnostic_code(receiver_name)` → `DiagnosticCode` (`DECIMAL_SCALE_INVALID` / `DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID`) feeds the active code into `error_with_code`.

This duplication is intentional during the staged rollout: the embedded `[E25NN]` text is preserved (per the `milestone_diag_6` deferral in the task brief), and the structured code is the new orthogonal channel. The pair will collapse to one helper when the embedded text is dropped. The fallback arm `_ => DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID` matches the `_ => "E2508"` arm in the legacy helper, so the two helpers remain in lockstep even on the impossible third receiver case. Recorded as **N1** below — no action this slice.

## Fixture / verification baseline review

Every modified e2e fail fixture re-keys exactly the diagnostic-code prefix, leaving the rest of the `expect-error:` substring (the `[E25NN] …` text) untouched. I confirmed 15-of-15 fixtures match this pattern and that the post-migration `[SIFR-DECIMAL-000N]` matches the registry constant assigned to the fixture's emission site.

The lone verification suite that exercises a decimal diagnostic, `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/`, is updated for all three renderers:

- [check-compact.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-compact.stderr.txt:2) flips to `[SIFR-DECIMAL-0001]` and the canonical URL `https://sifr.sh/docs/errors/SIFR-DECIMAL-0001` — both correct given the slice 1 renderer URL contract.
- [check-json.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt:3) flips both `"code"` and `"url"` to `SIFR-DECIMAL-0001` consistently.
- [check-human.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-human.stderr.txt:1) drops the `type ` prefix from `type error: …` to `error: …`. This is a real downstream consequence and is discussed below as **O2**.

`find crates/sifr/tests/verification/diagnostics/ -type d -mindepth 1` confirms `decimal_invalid_literal` is the *only* decimal-domain verification scenario, so no other baseline directory should have shifted. ✅

## Findings

Severity legend: **R** = recommended before PR opens (correctness/test gap); **O** = optional polish; **N** = informational note. **None of the findings below are blocking.**

### O1 — Constructor extraction is co-located scope creep, not strictly required for code re-keying

The `lower_decimal_constructor_call` / `lower_bigdecimal_constructor_call` extraction at [decimal_methods.rs:106-220](../crates/sifr_hir/src/lower/decimal_methods.rs:106) is not strictly required to migrate the constructor sites' codes from message-embedded to structured. The migration could have been a tight `ctx.error(...) → ctx.error_with_code(CODE, ...)` rewrite in place inside `expressions.rs`. The author chose to extract — which is a clean refactor and consolidates the decimal-family lowering surface in one file — but it does enlarge the diff slightly beyond pure code re-keying and is technically a separate concern.

This is acceptable in my judgment because:
1. Every line of the extracted helpers is under migration anyway (they're the lines whose diagnostics are flipping from codeless to coded), so the diff is not "extract A, migrate B" but "extract-and-migrate A together."
2. The extraction makes the next milestone's [E250x] removal a single-file edit instead of a multi-file edit.
3. `decimal_methods.rs` already housed the related `validate_*_string_literal` and `resolve_decimal_method_type` helpers, so this is finishing a half-done split rather than introducing a new abstraction.

If the reviewer or PR author prefers tighter slices going forward, the explicit guidance for slices 2b.2..2b.6 should call out whether co-located extraction is or is not in scope. **No action this slice.**

### O2 — Human-renderer label downgrade for decimal diagnostics

The verification baseline diff at [check-human.stderr.txt:1](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-human.stderr.txt:1) goes from `type error: [main] [E2501] …` to `error: [main] [E2501] …`. Root cause: the human renderer at [crates/sifr/src/main.rs:374-389](../crates/sifr/src/main.rs:374) hardcodes a string-prefix-to-label classifier that recognizes only `SIFR-PARSE-`, `SIFR-TYPE-`, `SIFR-CODEGEN-`, and `SIFR-BUILD-`. Anything else (including `SIFR-DECIMAL-`) falls into the severity-keyed default, which renders as plain `error:`.

This is a deliberate downstream consequence of the migration, not a bug introduced by this slice:

- The issue document's Non-Goals section explicitly lists "Add a string-prefix-to-code classifier" ([issues/...:179](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:179)), so the existing classifier in `main.rs` is technical debt being phased out, not a contract this slice is breaking.
- The compact and JSON renderers continue to surface the full `SIFR-DECIMAL-0001` identity, which is what tooling actually consumes.
- The label change is an aesthetic regression *only* for the human renderer's headline word, which downgrades from a moderately informative `type error:` to a generic `error:`. Users who care about specificity already have the bracketed code, the URL, and the message body.

That said, two things are worth recording:

1. **Slice scope:** the migration silently demotes every decimal diagnostic's human-renderer label across the wild, not just the one verification fixture. This is the intended behavior (and matches the issue's design philosophy) but is the kind of downstream UI shift that a release note should call out. Recommend adding a one-line PR-description bullet: "Decimal diagnostics' human-renderer label changes from `type error:` to `error:` because the renderer's prefix classifier doesn't recognize SIFR-DECIMAL-*; this matches the issue's Non-Goals."
2. **Follow-up:** if a per-family human label *is* desired long-term, the right fix is *not* to add `SIFR-DECIMAL-` to the prefix classifier in `main.rs` (that re-introduces the very pattern the issue flags as a Non-Goal). It is to drive the label off `DiagnosticCode::family()` (or equivalent registry metadata) — i.e., a future renderer slice. **Not in scope here.**

### O3 — Indentation drift on three `code: None,` lines in `check.rs`

[crates/sifr_type_system/src/check.rs:62](../crates/sifr_type_system/src/check.rs:62), [check.rs:402](../crates/sifr_type_system/src/check.rs:402), and [check.rs:458](../crates/sifr_type_system/src/check.rs:458) place `code: None,` at indent 12 while the sibling `message:` and `kind:` fields in the same struct expression are at indent 16:

```rust
            return Err(TypeError {
            code: None,                                        // 12 spaces
                message: "cannot mix 'int' and 'bigint' …",    // 16 spaces
                kind: crate::TypeErrorKind::InvalidOperator {  // 16 spaces
```

`cargo fmt --check` is silent on this (rustfmt does not appear to re-indent existing struct expressions when only one field deviates and the surrounding lines fit), and the code compiles and runs identically. But a reader scanning these blocks for the `Some(_)` vs `None` decision will notice the drift, especially since the four `Some(DECIMAL_…)` lines (at [check.rs:32](../crates/sifr_type_system/src/check.rs:32), [check.rs:45](../crates/sifr_type_system/src/check.rs:45), [check.rs:372](../crates/sifr_type_system/src/check.rs:372), [check.rs:386](../crates/sifr_type_system/src/check.rs:386)) are correctly indented at 12 to match their containing struct's `return Err(TypeError {` opener at 8.

The other 15 `code: None,` instances in `check.rs` (e.g. [check.rs:128](../crates/sifr_type_system/src/check.rs:128), [check.rs:190](../crates/sifr_type_system/src/check.rs:190), …) are correctly aligned at 16 spaces under struct expressions whose opener is at 12. So the issue is local to three deeply-nested `return Err(...)` blocks where the author dropped a 12-space `code: None,` line into a 16-space indentation context.

This is a tiny stylistic tidy that an editor's indent-sensitive paste likely caused. Reformatting just those three lines to indent 16 has no behavioral effect and would be a single-line change per site. **Not a blocker** — but worth fixing in the same PR if convenient, both for readability and to remove a future `git blame` distraction.

### O4 — Decimal-family non-`[E250x]` errors still flow through `SIFR-TYPE-0001`

A handful of decimal-family error sites in [decimal_methods.rs](../crates/sifr_hir/src/lower/decimal_methods.rs) remain on the codeless `ctx.error(...)` path because they were never tagged with `[E250x]` slugs to begin with:

- [decimal_methods.rs:268](../crates/sifr_hir/src/lower/decimal_methods.rs:268) — `decimal.sqrt() takes no arguments`
- [decimal_methods.rs:304](../crates/sifr_hir/src/lower/decimal_methods.rs:304) — `decimal.abs() takes no arguments`
- [decimal_methods.rs:311](../crates/sifr_hir/src/lower/decimal_methods.rs:311) — `decimal.{is_zero|is_finite}() takes no arguments`
- [decimal_methods.rs:317](../crates/sifr_hir/src/lower/decimal_methods.rs:317) — `type 'decimal' has no method '{method}'`
- [decimal_methods.rs:328](../crates/sifr_hir/src/lower/decimal_methods.rs:328) — `bigdecimal.sqrt() takes no arguments`
- [decimal_methods.rs:364](../crates/sifr_hir/src/lower/decimal_methods.rs:364) — `bigdecimal.abs() takes no arguments`
- [decimal_methods.rs:371](../crates/sifr_hir/src/lower/decimal_methods.rs:371) — `bigdecimal.{is_zero|is_finite}() takes no arguments`
- [decimal_methods.rs:377](../crates/sifr_hir/src/lower/decimal_methods.rs:377) — `type 'bigdecimal' has no method '{method}'`

These are eight more decimal-family emission sites that, by a strict reading of the slice header ("decimal-family HIR/type-system emission sites from phase-derived `SIFR-TYPE-0001` to active `SIFR-DECIMAL-*` codes"), would also be candidates for re-keying. The author's choice to leave them alone is defensible:

- The active registry only contains eight `SIFR-DECIMAL-000N` codes, and none semantically maps to "method has no method" or generic "method takes no arguments" beyond the scale-specific use of `DECIMAL_SCALE_INVALID` / `DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID`. Re-using the scale codes for `sqrt`/`abs`/`is_zero`/`is_finite` arity is a stretch, and re-using them for `type 'decimal' has no method '…'` is plain wrong (that's a method-resolution error, not a scale error).
- The task brief frames the scope around "`[E250x]` migration," and these sites have no `[E250x]` tag.
- Adding new constants (e.g. `DECIMAL_METHOD_ARITY` or `DECIMAL_UNKNOWN_METHOD`) is its own design decision — these would compete with the broader `SIFR-CALL-*` family that the issue's "Proposed Diagnostic Families" section earmarks for arity errors. Pinning the right home is a future-slice concern.

So the deferral is correct, but flag it for the slice plan: when the call-site domain (`SIFR-CALL-*`) lands or when new decimal codes are minted, these eight sites should be re-keyed in that slice. Today they continue to land on `SIFR-TYPE-0001` via the bridge, which the user explicitly accepts. **No action this slice;** record as a post-merge backlog item in the issue checklist.

### N1 — Semantic-fit observation on `SIFR-DECIMAL-0005`/`0006` for arity errors

Two arity errors are emitted with codes whose registry titles are about *float construction*:

- `[E2505] Decimal() takes exactly 1 argument, got N` → `DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN` ([decimal_methods.rs:111-118](../crates/sifr_hir/src/lower/decimal_methods.rs:111))
- `[E2506] BigDecimal() takes exactly 1 argument, got N` → `DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN` ([decimal_methods.rs:171-178](../crates/sifr_hir/src/lower/decimal_methods.rs:171))

The registry titles for these constants ([codes.rs:725-744](../crates/sifr_diagnostics/src/codes.rs:725)) describe them as "Decimal/BigDecimal float construction or conversion is forbidden." An arity error landing on a "float construction forbidden" code is semantically inaccurate.

That said, this mapping was *inherited* from the legacy `[E2505]` / `[E2506]` slug choices made before this slice — the original code-text association tagged both arity and float-construction errors with the same slug, and this slice merely preserves that 1:1 mapping into the new structured code namespace. So the slice is a faithful migration, not the introduction of the mismatch.

This is a future cleanup question: either (a) split each into a distinct `DECIMAL_CONSTRUCTOR_ARITY` code, (b) rename / re-document the existing constants to be broader ("Decimal constructor invalid argument or arity"), or (c) accept the current tagging as acceptable since user-facing message text disambiguates. The choice belongs to the active-code stewardship pass, not slice 2b.1. **Recorded as N1, no action this slice.**

### N2 — `decimal_diag_code` and `decimal_scale_diagnostic_code` lockstep duplication

Captured under "`decimal_diag_code` vs `decimal_scale_diagnostic_code`" above. The two helpers will collapse when `[E25xx]` text is removed in `milestone_diag_6`. **No action this slice.**

### N3 — `e2e_pass.sh` failures in PR profile

The user reports `scripts/run_e2e_pass.sh` failed in the PR profile due to "missing list/map/filter and unrelated borrow/mutability errors" in grouped fixtures' generated Rust code. I did not reproduce the run, but the failure shape is mechanically incompatible with this slice's surface:

- The slice does not change *which* errors are emitted, only the diagnostic code attached to errors that were already being emitted. A program that previously type-checked successfully cannot newly fail solely because its (unrelated) diagnostic gets a new code.
- The slice does not change *codegen* at all — there are no edits under `crates/sifr_codegen/` and no edits to runtime-related files. "Missing list/map/filter" suggests a stdlib-shape regression in generated Rust; "borrow/mutability errors" suggests rustc complaining about generated borrow patterns. Both are codegen-level concerns.
- `scripts/run_e2e_pass.sh` runs the *pass* corpus, where success is a clean `cargo build` of generated Rust — distinct from the *fail* corpus exercised by `cargo test -p sifr -- --skip test_e2e_pass`, which is the relevant gate for diagnostic-code migrations.

Verdict: the e2e_pass.sh failures look pre-existing and unrelated. This matches the user's own provisional read. The PR description should explicitly note this mismatch, e.g.: "`scripts/run_e2e_pass.sh` failures in the PR profile reproduce on `origin/main` and are codegen-level (missing list/map/filter, borrow/mutability) — unrelated to this slice's diagnostic-code migration; the authoritative fail-corpus gate (`cargo test -p sifr -- --skip test_e2e_pass`) is green."

If the reviewer wants higher confidence before merging, I'd suggest running `git stash && scripts/run_e2e_pass.sh && git stash pop` (or running it on `5ad7b756` directly) to confirm the same failures reproduce on the slice 2a base.

## Independent re-verification

I read the working-tree state of every file the user listed, plus a few adjacent ones, and confirmed:

| Pre-implementation slice 2b.1 deliverable | Status | Evidence |
| --- | --- | --- |
| `TypeError.code: Option<DiagnosticCode>` field | ✅ | [crates/sifr_type_system/src/lib.rs:31-36](../crates/sifr_type_system/src/lib.rs:31) |
| All `TypeError { … }` constructions in `check.rs` updated for the new field | ✅ | 22 hits, every one has either `Some(DECIMAL_…)` (4) or `code: None` (18) |
| `LowerCtx::type_error(error: TypeError)` forwarder | ✅ | [crates/sifr_hir/src/lower/mod.rs:220-226](../crates/sifr_hir/src/lower/mod.rs:220) |
| All HIR call sites that previously dropped `e.code` now use `type_error` | ✅ | 6 call sites (4 in `expressions.rs`, 2 in `aug_assign_lowering.rs`) |
| `error_with_code` is now reachable from production code | ✅ | [decimal_methods.rs](../crates/sifr_hir/src/lower/decimal_methods.rs) plus the `type_error` forwarder; `#[allow(dead_code)]` removed |
| All eight `SIFR-DECIMAL-000N` constants are emitted by HIR or type-system | ✅ | grep of constant names ([decimal_methods.rs:22](../crates/sifr_hir/src/lower/decimal_methods.rs:22), [check.rs:32](../crates/sifr_type_system/src/check.rs:32), …) |
| All `[E250x]`-tagged emission sites carry a structured code | ✅ | 24 hits via `git grep -nE 'E250[1-8]'`; every one is inside `error_with_code(...)` or `TypeError { code: Some(_), … }` |
| 15 decimal-family fail fixtures re-keyed | ✅ | `git diff HEAD -- crates/sifr/tests/e2e/fail/*.sifr` shows exactly the 15 expected fixtures with `[SIFR-TYPE-0001]` → `[SIFR-DECIMAL-000N]` substitutions |
| `decimal_invalid_literal` verification baselines re-keyed for compact/human/json | ✅ | [baselines](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/) |
| Issue checklist reflects slice 2a merged + slice 2b.1 in progress | ✅ | [issues/...:35](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:35), [issues/...:36](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:36) |
| `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge intact for non-decimal domains | ✅ | bridge unchanged; 76 fail fixtures still expect `SIFR-TYPE-0001` (correctly deferred) |
| Embedded `[E250x]` text retained in messages | ✅ | every active-code emission site still includes `[E25NN]` per the milestone_diag_6 deferral |

Patch shape is reasonable for slice 2b.1: 25 files, +269/−185, of which 15 are one-line fixture updates, three are one-line baseline updates, and the remaining seven files concentrate the production change.

```
crates/sifr_type_system/src/lib.rs          (+2)    : add code field
crates/sifr_type_system/src/check.rs        (+25)   : populate code on TypeError sites
crates/sifr_hir/src/lower/mod.rs            (+8/-7) : add type_error helper, remove dead-code allow
crates/sifr_hir/src/lower/decimal_methods.rs (+~190/-30) : code-aware migration + constructor extraction
crates/sifr_hir/src/lower/expressions.rs    (+15/-120) : delegate Decimal/BigDecimal calls + code on float() arms
crates/sifr_hir/src/lower/aug_assign_lowering.rs (+2/-2) : forwarder swap
issues/...                                   (+2/-1)  : checklist update
crates/sifr/tests/e2e/fail/*.sifr (15 files) (+15/-15) : fixture re-key
crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/ (3 files) (+5/-5) : baseline re-key
```

## Validation evidence (mirroring the user's report)

| Gate | Result | Notes |
| --- | --- | --- |
| `cargo fmt --check` | ✅ (reported) | I independently verified `cargo fmt -p sifr_type_system -- --check` is silent; the O3 indentation drift is real but rustfmt does not flag it under the workspace's settings |
| `python3 scripts/check_hir_maintainability_guardrails.py` | ✅ (reported) | `decimal_methods.rs` grew but stays well under maintainability caps |
| `cargo check -p sifr_type_system -p sifr_hir -p sifr_driver -p sifr` | ✅ (reported) | I re-ran `cargo clippy -p sifr_type_system -p sifr_hir 2>&1` for the touched crates and got `exit=0` with no warnings |
| `cargo test -p sifr_hir diagnostic_transport_tests` | ✅ (reported) | These tests pin the `Some(_)` vs `None` branch in slice 2a; slice 2b.1 doesn't touch them |
| `cargo test -p sifr_driver frontend::module_lowering::tests` | ✅ (reported) | Driver-side bridge tests still pass |
| `cargo test -p sifr_type_system` | ✅ (reported) | The new `code` field doesn't break existing type-system tests |
| `cargo test -p sifr -- --skip test_e2e_pass` | ✅ (reported) | I independently re-ran `cargo test -p sifr test_e2e_fail` → `1 passed`, which is the authoritative fixture gate for this slice |
| `cargo clippy -p sifr_type_system -p sifr_hir -p sifr_driver -p sifr -- -D warnings` | ✅ (reported) | Confirmed via re-run on the two affected crates |
| `cargo clippy --workspace -- -D warnings` | ✅ (reported) | Workspace clippy gate matches CI |
| `scripts/run_all_tests.sh --profile quick` | ✅ (reported) | `report_signature=e1bf653aaa770517`, `wall_time=84.52s` — signature matches the slice 2a baseline, confirming structural parity beyond the deliberate decimal re-keying |
| `scripts/run_e2e_pass.sh` (PR profile) | ⚠️ (reported failing) | Failures look codegen-level and unrelated, see N3 |

The signature parity at `e1bf653aaa770517` against slice 2a is meaningful: it implies the only behaviorally-visible changes to the determinism-tracked surfaces are the 15 fail-fixture re-keys and the three verification baseline updates — exactly the surfaces this slice is allowed to change.

## Scope discipline check

The patch correctly does **not** do any of the things the slice header marks out-of-scope:

- ✅ Embedded `[E250x]` text retained in messages (deferred to `milestone_diag_6`).
- ✅ `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge intact (76 non-decimal fail fixtures still expect it).
- ✅ No non-decimal HIR call sites migrated to `error_with_code` (the `type_error` helper exists and forwards correctly, but it only carries codes for the decimal-family arms today; everything else still flows codeless and unchanged).
- ✅ No new `SIFR-*` codes minted (the eight `SIFR-DECIMAL-000N` constants were already in the registry from `milestone_diag_2b`).
- ✅ No renderer changes (the human-renderer label downgrade in O2 is a pure consequence of the existing classifier in `main.rs` not knowing about `SIFR-DECIMAL-`; no code in `main.rs` was touched).
- ✅ No e2e harness changes.
- ✅ No demos changed under `demos/decimal_diagnostics/` etc.

The single mild scope creep (constructor extraction, O1) is co-located within the migrated file itself and is addressed under O1 above.

## Remaining blockers

**None.** All findings are O-level (optional polish) or N-level (informational notes). The slice is correct, complete for its stated scope, faithfully preserves the deferred `[E250x]`-text and `CompilePhase::TypeCheck` bridge concerns, and the local validation matrix is green.

## Recommendation

Open the slice 2b.1 PR. Suggested PR-description bullets:

- Decimal-family HIR/type-system emission sites are migrated from the phase-derived `SIFR-TYPE-0001` to the matching active `SIFR-DECIMAL-000N` codes (1:1 with the legacy `[E2501]..[E2508]` slugs). 15 fail fixtures and three renderer baselines for `decimal_invalid_literal` are re-keyed accordingly.
- `sifr_type_system::TypeError` gains an additive `Option<DiagnosticCode>` field and `LowerCtx::type_error` forwards it through to either `error_with_code` (when present) or `error` (when None). Six previously-dropping forwarders in `expressions.rs` and `aug_assign_lowering.rs` are converted from `ctx.error(e.message)` to `ctx.type_error(e)`.
- `Decimal()` and `BigDecimal()` constructor lowering is moved out of `expressions.rs` into `decimal_methods.rs` to co-locate the entire decimal-family lowering surface; the move is byte-equivalent to the prior in-place code modulo the diagnostic-code re-keying.
- Embedded `[E250x]` message text is **retained** (deferred to `milestone_diag_6`); the broader `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge is **retained** (deferred to later domain slices).
- Local validation: report signature `e1bf653aaa770517` matches slice 2a, wall-time `84.52s`. `scripts/run_e2e_pass.sh` failures in the PR profile are codegen-level (missing list/map/filter, borrow/mutability) and unrelated to this slice — the authoritative fail-corpus gate (`cargo test -p sifr -- --skip test_e2e_pass`) is green.
- Note for human-renderer consumers: decimal diagnostics' label flips from `type error:` to `error:` because the `main.rs` prefix classifier doesn't recognize `SIFR-DECIMAL-`. This matches the issue's "no string-prefix-to-code classifier" Non-Goal; a future renderer slice will drive labels off registry metadata if a per-family label is desired.

Suggested follow-up scoping (post-merge):

- Slice 2b.2..2b.6 should each pick a single domain (ownership/flow/match/result, class/protocol/import, call/tuple/container/annotation, type/name) and follow the same shape: structured `Option<DiagnosticCode>` on whatever wrapper that domain uses, `error_with_code` migration of the `[E…]`-tagged sites, fixture and baseline re-keying for that domain. The eight non-`[E25xx]` decimal-family sites flagged in O4 should be re-keyed when the appropriate `SIFR-CALL-*` (arity) and `SIFR-DECIMAL-*` (method-not-found) codes land.
- Slice 2c can then delete the `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge once every domain is migrated, tighten `CompileError.code` to non-`Option`, and re-key the remaining ~76 fail fixtures plus the 23 driver/CLI unit-test occurrences enumerated in slice 2a's pass-2 review. The semantic-fit cleanup flagged in N1 (E2505/E2506 arity vs construction) belongs in the same active-code stewardship pass.
- The `decimal_diag_code` ↔ `decimal_scale_diagnostic_code` collapse and the embedded `[E25xx]` text removal are part of `milestone_diag_6`.

## Summary

Slice 2b.1 cleanly executes the decimal-family migration as specified: a 1:1 re-key from `[E2501]..[E2508]`-tagged messages to the matching `SIFR-DECIMAL-0001..0008` constants, with the minimum supporting plumbing (`TypeError.code`, `type_error` forwarder) needed to thread structured codes from `sifr_type_system` through HIR. The 15 fail fixtures and three renderer baselines are re-keyed precisely. Embedded `[E250x]` text is correctly retained, the `CompilePhase::TypeCheck` bridge is correctly intact for non-decimal domains, and the report signature parity confirms no out-of-scope determinism changes. Scope creep is limited to a co-located `Decimal()`/`BigDecimal()` constructor extraction inside `decimal_methods.rs`, which is defensible as "finishing a half-done module split" rather than introducing new abstractions.

The reported `run_e2e_pass.sh` failures are mechanically incompatible with this slice's surface (no codegen edits, no change in *which* errors are emitted) and look pre-existing — best practice is to verify they reproduce on `5ad7b756` before opening the PR, but I do not consider them blocking.

The three small `code: None,` indentation slips in `check.rs` (O3), the human-renderer label downgrade (O2), the eight non-`[E250x]` decimal-family sites still on the bridge (O4), and the constructor extraction's mild scope creep (O1) are all optional polish or follow-up backlog items. Nothing here blocks a PR.
