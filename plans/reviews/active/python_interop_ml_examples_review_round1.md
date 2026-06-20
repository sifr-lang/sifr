Now I have a complete picture. Let me write up the review.

## Review: Python interop ML full examples (round 1)

### 1. Blocking findings
None. The implementation matches the production-grade verification intent described in the task:

- Both fixtures run real library code (torch tensor math through `mul`/`reshape`/`sum`; sklearn `DecisionTreeClassifier.fit`/`predict`/`classes_`) and convert results back to typed Sifr values via `to_float`, `copy_list_float`, and `copy_list_int`. The torch fixture additionally validates DLPack metadata (`dtype`, `dimensions`, `shape`) and releases the capsule before closing the source tensor (`fixtures/torch_dlpack/torch_full_example.sifr:43-51`).
- Stdout markers are static, deterministic strings and are wired into `ExampleCase.stdout_marker` (`runner/ml_examples.py:9-21`).
- Shared `example_packages.py` factor-out is faithful to the original `dataframe_examples.py` flow: source-presence checks, temporary package creation under `target/verification/areas/python_interop/<suite>_examples_package/<case_id>`, `.venv` symlink to area uv environment, `sifr run` via `cargo run -p sifr`, 600s timeout, `--locked` enforcement at the case-runner level (`runner.py:74-103, 221-244`).
- `lib/sifr/python.sifr` `own` annotations on `_buffer_from_raw`, `_arrow_from_raw`, `_dlpack_from_raw`, `_record_fields_from_handles` are correct: each carries non-copy fields (`String`, `list[int]`, `list[str]`) that the function moves into wrapper class instances, which is only legal when the tuple is owned. Callers (`zero_copy_dlpack_tensor` and friends) pass `raw` once and don't reuse it, so the move is sound.
- Manifest wires a separate `ml` suite (`manifest.json:151-163`), command added to `COMMAND_ARGS` and `AREA_PROJECT_COMMANDS` (`runner.py:74-78, 99-103`), `--ml-examples` arg + dispatch + self-test plumbed in `runner/run.py:18, 134, 146, 173-181`, and `REQUIRED_FIXTURES`/`REQUIRED_SOURCE_FIXTURES` include the new dirs (`run.py:54, 103-104`).

### 2. Non-blocking suggestions

1. **Loose wrapper signatures lose type-checker coverage.** `dataframe_examples.py:13-19` and `ml_examples.py:24-33` declare `example_runner: Any | None = None` while `example_packages.build_examples_report` keeps the precise `Callable[[RunnerPaths], list[dict[str, Any]]] | None`. Restoring the same `Callable[...]` type on the wrappers would keep self-test stubs type-checked at their entry points.

2. **`_objects_from_handles` is dead code and didn't need `own`.** `lib/sifr/python.sifr:182` is unreachable (no in-tree caller, confirmed via grep across `*.sifr`). It also operates only on `tuple[int, int]` (copy fields), so the `own` annotation is not required for correctness — unlike its siblings whose `own` is load-bearing. Consider deleting the function (per CLAUDE.md: "If you are certain that something is unused, you can delete it completely.") to match the rationale in `plans/issues/active/ad-hoc-embedded-python-interop.md:117`, which references *non-copy* fields.

3. **Schema additive change is harmless but worth flagging.** `_report` in `example_packages.py:289-305` adds `"suite": "<name>-examples"` to the JSON output, so the existing dataframe report now also carries that key. No external consumer was found, but if any downstream tooling later snapshot-compares these files, expect a one-time diff for the dataframe report.

4. **Resource cleanup on intermediate failures (pre-existing pattern).** If e.g. `call_attr(torch, "mul", ...)` raises after several `from_*` objects exist, those Objects are leaked because the `except` re-raises without closing them. Both new fixtures follow the same pattern as the merged dataframe examples, so this is not a regression — surface it on the roadmap only if strict no-leak negative paths become a goal.

5. **Sklearn marker assumes sorted `classes_`.** `fixtures/sklearn/sklearn_full_example.sifr:55` asserts `classes[0] == 0 and classes[1] == 1`. This relies on sklearn's documented contract that `classes_` is sorted unique labels — stable today; just a coupling worth keeping in mind if you ever switch estimators.

6. **`pyproject.toml` upper bounds are wider than the installed wheels** (`torch>=2.5,<3` ships 2.12.1; `scikit-learn>=1.5,<2` ships 1.9.0). The `uv.lock` is canonical, so this is fine — noting only that the manifest leaves substantial drift room before the lock gets refreshed.

### 3. Another review round required?
No. Findings #1–#6 are quality polish, not correctness gates. The implementation, validations, and plan/doc updates are consistent with the post-closeout dataframe examples precedent. Recommend proceeding to PR.
