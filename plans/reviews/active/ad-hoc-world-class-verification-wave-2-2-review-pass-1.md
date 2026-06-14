# Wave 2.2 External Review — Findings

## Verdict
**No blockers. Wave 2.2 is APPROVED for PR/merge** with two low/medium follow-ups noted below.

## Coverage verification
All 16 Wave 2.2 rows (`proposed_pr_slice: 2.2`) are closed and accounted for. Cross-checked:

| Row id | Test | Touched in diff |
| --- | --- | --- |
| 0004 | `..::test_generate_rust_elides_unreachable_returns_after_always_exit_paths` | `async_control_codegen_tests.rs:75-126` |
| 0005 | `..::test_generate_rust_preserves_loop_else_recursion_and_try_except_returns` | `async_control_codegen_tests.rs:48-99` |
| 0006 | `..::test_async_generated_errors_convert_to_error_return_type` | `async_runtime_codegen_tests.rs:110` |
| 0013 | `..::test_await_task_handle_desugars_to_join_observation` | `async_task_runtime_codegen_tests.rs:277` |
| 0014 | `..::test_scope_spawn_fallible_coroutine_lowers_to_result_spawn_helper` | `async_task_runtime_codegen_tests.rs:65` |
| 0015 | `..::test_task_gather_fallible_tasks_keeps_error_parameter_unwrapped` | `async_task_runtime_codegen_tests.rs:102` |
| 0016 | `..::test_task_gather_lowers_to_private_gather_helper` | `async_task_runtime_codegen_tests.rs:37` |
| 0017 | `..::test_task_group_basic_lowers_to_scope_runtime_substrate` | `async_task_runtime_codegen_tests.rs:7` |
| 0018 | `..::test_task_handle_join_lowers_to_task_result_observation` | `async_task_runtime_codegen_tests.rs:251` |
| 0019 | `..::test_task_race_fallible_tasks_keeps_error_parameter_unwrapped` | `async_task_runtime_codegen_tests.rs:162` |
| 0020 | `..::test_task_race_lowers_to_private_race_helper` | `async_task_runtime_codegen_tests.rs:134` |
| 0021 | `..::test_task_timeout_handle_lowers_to_private_timeout_result` | `async_task_runtime_codegen_tests.rs:326` |
| 0023 | `..::test_fieldless_class_gets_default_constructor` | `classes_and_basics_codegen_tests.rs:69-77` |
| 0025 | `..::test_guarded_non_option_compare_does_not_emit_some_wrapping` | `classes_and_basics_codegen_tests.rs:184-198` |
| 0026 | `..::test_mut_on_local_nested_function_mutborrow_call_argument` | `classes_and_basics_codegen_tests.rs:51-61` |
| 0035 | `..::test_generate_rust_open_uses_canonical_filehandle_constructor` | `iterators_and_generators_codegen_tests.rs:224-237` |

Inventory consistency: `red_blocker.failure_count: 16`, `test_result: 691/16/707`, all 16 Wave 2.2 rows flipped to `closed`, remaining 16 open rows correctly bucketed into 2.3/2.4/2.5 — matches `cargo test -p sifr_codegen` reported output exactly. No row classification drift; no Wave 2.2 row left as `open`; no later-wave row prematurely marked `closed`.

Fix-class compliance: parser-invalid fixtures uniformly converted to `r#"..."#` raw strings; SIFR-ASYNC-0001 fixtures all gained `await task.sleep(0.0)` real suspensions (a genuine yield point per the policy text, not an escape hatch); `open()` fixture gained `encoding="utf-8"`. No silent compiler-bug masking — every closure either changes only the fixture text or refreshes assertions made newly reachable by a now-parseable fixture.

## Findings

### Medium — `classes_and_basics_codegen_tests.rs:184-198` (row 0025)
The fixture for `test_guarded_non_option_compare_does_not_emit_some_wrapping` was rewritten *substantively*, not just minimally repaired. The original tested:

```python
first = token[0]
if first is not None and first == "-":
    return -1
```

The replacement removes the `is not None` clause entirely and inserts an outer `if len(token) > 0:` guard before `token[0]`:

```python
if len(token) > 0:
    first = token[0]
    if first == "-":
        return -1
```

That changes what the test exercises: the original specifically asserts that a *compound `is not None and ==`* guard does not Some-wrap the equality; the new fixture only exercises a simple equality after a length guard. The compound-guard regression target is no longer covered.

Compounding this, the positive assertion was weakened from `first == "-".to_string()` to just `first == "-"` (line 197). The new substring would still match `first == "-".to_string()`, so a regression that re-introduces the `.to_string()` wrap on the RHS would silently pass.

If the original fixture genuinely no longer typechecks under current HIR (i.e. `is not None` on a non-Optional is now a hard error), that's worth stating in the triage row's `current_output` / commit message — currently the row only mentions a parser error, which a raw-string conversion alone should have resolved. Either tighten the assertion (anchor it so it fails on `.to_string()` reappearance) or add a separate fixture that still covers the compound guard pattern.

### Low — `async_control_codegen_tests.rs:124`
Assertion weakened from `assert!(unreachable_tail.contains("return 2 as i64;"))` to `assert!(unreachable_tail.contains("2_i64"))`. The tail-expression contract change justifies dropping the `return …;` prefix, but the substring is now broad enough to match any incidental `2_i64` token in the output. Tightening to e.g. `contains("2_i64\n}")` or asserting `!unreachable_tail.contains("return 2_i64")` would preserve the original "tail, no explicit return" intent of this test alongside the elision check.

### Low — phase doc evidence row
The Wave 2.2 Implementation Notes section (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:349-361`) omits the `cargo fmt --check: pass` and `python3 scripts/check_file_size_guardrails.py: pass` validation rows that the user reported and that Wave 2.0/2.1 notes both include. Add them for evidence-trail parity with the prior subwaves.

### Low — scope-overlap clarity
The Wave 2.2 doc describes "secondary normalized-literal / constructor assertions exposed after those fixtures started lowering" generically. Concretely, the parser-fix in `async_control_codegen_tests.rs` brought along `99 as i64` → `99_i64`, `(10 as i64)` → `(10_i64)`, `77 as i64` → `77_i64`, and dropped a `return 2 as i64;` form — all of which are Wave-2.1-class literal-spelling refreshes that only became reachable here. Naming this overlap explicitly in the implementation notes would prevent a future reader from misreading it as a Wave 2.1 row leaking out of scope.

## Answers to review questions

1. **Genuine closure without hiding compiler bugs?** Yes for 15 of 16. Row 0025 is the only one that warrants a second look — the fixture rewrite goes beyond a minimal raw-string repair and the assertion was loosened. Not a blocker, but worth a justification or assertion tightening.
2. **Policy-compliant and representative?** Yes — `await task.sleep(0.0)` is a real suspension (not the documented escape hatch), `encoding="utf-8"` matches SIFR-IO-0801's "explicit encoding" requirement, and the raw-string conversions preserve indentation faithfully.
3. **Inventory internally consistent?** Yes — `failure_count: 16`, `test_result: 691/16/707`, exactly the 16 Wave 2.2 ids flipped to `closed`, and the 16 remaining open rows partition cleanly into 2.3 (5), 2.4 (6), 2.5 (5).
4. **Docs/validation notes accurate enough?** Substantively yes; missing two validation rows (see Low item above).
5. **Blockers before Wave 2.2 PR?** None.

Recommend opening the PR; address the medium-severity row-0025 note either by tightening the assertion (`!rust_code.contains("first == \"-\".to_string()")`) or by adding a separate compound-guard fixture as a Wave-2.5 (or later) regression target, and add the two missing validation evidence rows to the phase doc.
