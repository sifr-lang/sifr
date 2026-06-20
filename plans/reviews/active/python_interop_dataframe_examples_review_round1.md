I've reviewed the diff. Read-only, no files modified.

## Blockers
None. The compiler patch is correctly scoped, the new tests cover the three signature shapes that were broken, the fixtures use real `sifr.python` exports, and the area-runner wiring matches the existing live-examples pattern.

## Notable findings (ranked)

**1. `find_function_body_insert` uses naive string matching (low risk, worth a guard).** `crates/sifr_driver/src/build/python_runtime.rs:162-169` searches for `"\nfn main("` / `"\nasync fn main("` and then for the first `{`. Three latent footguns:
- `"\nfn main("` will not match `"\nasync fn main("` (the leading `async ` blocks the prefix), so the fallback to `"async fn main("` is needed — good. But a future signature like `pub fn main(` or `#[unsafe] fn main(` would silently fail with the misleading "no main function" error.
- A `///` or `//!` doc-line whose continuation starts at column 0 with `fn main(` would be picked up — exceedingly unlikely from prettyplease, but a single-line comment at column 0 with `// fn main(` is safe only because of the `\n//` prefix.
- A `fn main(` appearing in a string literal at column 0 would falsely match. Not realistic in generated codegen output, but a one-line "only match outside comments/strings" comment in the function would save a future reader. Optional: add a test for `pub fn main(` (currently rejected) so the behavior is at least documented.

**2. Stdout/stderr is never inspected — examples are exit-code-only.** `dataframe_examples.py:198-227` captures `proc.stdout`/`stderr` only on failure. The "what was verified" is entirely the `if len(copied) != 4 ... raise PythonError` line in each `.sifr` fixture. If a fixture is ever weakened (e.g., constants flipped to match a regressed answer), the suite still reports `example-passed`. Since this gate is the only thing actually executing NumPy/pandas/Polars in Sifr, consider one of: (a) have the Sifr program `print` a canonical summary line and grep it from `stdout`, or (b) capture stdout into the report unconditionally so reviewers can eyeball it.

**3. `_case_operations` is decorative, not derived.** `dataframe_examples.py:230-237` returns a hand-typed list per case. It's not cross-checked against the `.sifr` source, so the report's `operations` field will drift the moment a fixture changes. Either generate it from a comment block in the fixture or drop the field — right now it reads like evidence but isn't.

**4. `@blocking_io` is missing from the new examples.** Compare `fixtures/numpy_buffer/numpy_full_example.sifr:15` (`def main() -> Result[None, PythonError]:`) to the established `fixtures/numpy_buffer/py_buffer_roundtrip.sifr:14-15` which uses `@blocking_io`. Per the milestone-5 contract "every public `sifr.python` boundary operation is `@blocking_io`," and the existing fixtures consistently annotate. Sync main probably skates by today, but the inconsistency with the area's own fixtures is worth either fixing or commenting on so it doesn't read as an oversight.

**5. Fixture cleanup is skipped on assertion failure.** `numpy_full_example.sifr:36-49`, `pandas_full_example.sifr:43-57`, `polars_full_example.sifr:42-53` all raise `PythonError` *before* running their `close()` chain. The success path closes everything in reverse order (good), but a future bug that makes the assertion false would leak every handle — exactly the scenario where resource diagnostics would be most useful. `py.with_context` (milestone_py_6) was added for this. Not a blocker for a passing test, but it weakens the "production-grade" framing.

**6. Trust is over-granted per case.** `dataframe_examples.py:156` writes `allow-imports = [numpy, pandas, polars, pyarrow]` for all three cases, including the numpy-only case. Trust policy is one of the things this area exists to demonstrate; the cleanest thing would be a per-case roots tuple so the numpy fixture only trusts `numpy`. Minor.

**7. No subprocess timeout on `cargo run`.** `dataframe_examples.py:198-214` has no `timeout=...`; if a fixture hangs (Python deadlock, blocked GIL, etc.) the runner hangs until the orchestrator's 900s area-level timeout. Adding `timeout=600` with an `example-failed` + `reason="timeout"` branch would localize the failure.

**8. `src/lib.rs` marker is load-bearing-by-coincidence.** `dataframe_examples.py:135-138` writes a comment-only `lib.rs` "so Cargo recognizes the package," but the package is never compiled by Cargo — sifr CLI runs it. If this is here only to satisfy `cargo run -p sifr -- run --manifest-path <repo>/Cargo.toml` from `cwd=package_root`, the comment should say so; otherwise a reader will assume it's interface code.

**9. Self-test covers shape but not the cargo path.** `run_dataframe_examples_self_tests` injects synthetic case results and verifies the report aggregation — good for shape regressions, but the only thing that exercises the *actual* cargo+venv pipeline is the gate itself. Worth noting in the README under "self-tests do not cover example execution."

## What's solid
- The compiler test matrix (`injects_runtime_init_as_first_main_statement`, `..._into_result_returning_main`, `..._into_async_main`, `rejects_missing_main_function`) covers the regression and the two new shapes. Bootstrap prelude is prepended at file start, `if let Err(...) { process::exit(1) }` is inserted as the first body statement — semantically correct for both `()` and `Result<...>` returning mains.
- `runner.py:215-227` correctly pops `VIRTUAL_ENV` before `uv run --project ... --locked` to avoid the host venv shadowing the area lock.
- The area-venv presence check in `prepare_dataframe_example_package` fails fast with an actionable message instead of producing a misleading sifr build error.
- Manifest `dataframes` suite now has two cases (matrix metadata + executable), preserving the distinction the README spells out at lines 84-89.
- `uv.lock` regeneration is consistent with the added pyproject deps.

Bottom line: ship-quality with no blockers. Findings 2 and 3 are the ones that matter for the "examples genuinely verify behavior" framing — everything else is polish.
