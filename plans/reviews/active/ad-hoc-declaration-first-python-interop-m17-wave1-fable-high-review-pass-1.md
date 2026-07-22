I have everything I need. All bypass vectors were verified empirically against the actual guard code. Here is the review.

---

# Review: M17 Wave 1 — PR #2997 (75198e226..1ca5d91b5)

## Scope verified

I read the complete one-commit diff (28 files: 11 migrated `.sifr` fixtures, 11 new bridges, 4 runner files, README, plan), the M17 contract in `plans/issues/active/ad-hoc-declaration-first-python-interop.md:2431-2510`, the full post-diff `example_packages.py`, the raw API surface in `stdlib/sifr/python.sifr` / `python_core.sifr`, the compiler trust gate in `sifr_lowering`, and the remaining raw fixtures. I ran the new guard function directly against candidate bypass inputs.

What holds up well: all eleven inventoried examples are genuinely migrated with faithful assertion parity (numpy/pandas/polars/pyarrow/torch/sklearn/fastapi/crypto/boto3/redis/sqlalchemy bridges reproduce every semantic check the old raw fixtures made, and each `.sifr` main adds a before/after `resource_diagnostics()` equality check); bridges are hermetic, deterministic (Stubber, fakeredis, in-memory SQLite, fixed `random_state`), copied file-by-file via `bridge_files` whitelists so co-resident bridges like `numpy_buffer/python_bridges/buffer_*.py` don't leak into example packages; source-policy failures do fail the whole suite closed (`build_examples_report` returns `examples-failed` with zero cases run, `example_packages.py:63-73`); failures propagate as `PythonError` through `Result` — no panic paths; the one intentional raw example (`demos/m16_raw_api`, with `fixtures/primitive_conversion/raw_typed_ergonomics.sifr` as its verification counterpart) lives outside the ordinary suites; and cases really execute compiled Sifr binaries via `cargo run -p sifr -- run`.

## Findings

### MAJOR 1 — The "fail-closed" source-policy guard is a bypassable denylist; the report and README certify a policy the guard does not enforce

`verification/areas/python_interop/runner/example_packages.py:15-39` (`RAW_API_SYMBOLS`) and `:280-289` (`imported_raw_api_symbols`).

The wave's headline deliverable is that "the runner fails closed if ordinary examples use the raw Python API or `@trust_python_dynamic`", and every report now stamps `source_policy: "ordinary-examples-forbid-raw-python-api"` (`example_packages.py:558`). I verified four independent bypass classes empirically — all return an empty match set from the guard:

1. **Unguarded raw symbols.** The denylist holds 23 names; `stdlib/sifr/python.sifr` exports ~70 raw functions/types. Missing: `from_value` (`python.sifr:103` — a universal `Object` constructor), `to_value`, `kwarg`, all `copy_list_*`/`copy_tuple_*`/`copy_dict_str_*`/`copy_record_fields`/`copy_as_bytes`/`copy_buffer_bytes`, all sized conversions `to_i8`…`to_usize`, all `zero_copy_*`/`export_arrow_*`/`release_*` protocol helpers, `enter_context`/`exit_context`/`with_context`, `run_coroutine_blocking`, `BufferView`, `ArrowCapsule`, `DlpackTensor`, and the callback constructors.
2. **The M16 method-style raw API needs no import at all.** `fixtures/primitive_conversion/raw_typed_ergonomics.sifr:22-38` demonstrates `.get_item`, `.get_attr`, `.call`, `.call_method` directly on `Object`, and Sifr allows unannotated locals (e.g. `demos/runtime_observability_boundary_demo.sifr:7`), so an ordinary example can write `x = from_value(...)`, then `x.call_method(...)` / `to_value(...)` — genuine raw-object plumbing — while importing only unguarded names. The `@trust_python_dynamic` substring check is no backstop: the compiler requires that decorator only for *dynamic* `import_module` targets (`crates/sifr_lowering/src/lower/expressions/regular_calls.rs:53-61`); every pre-migration raw example compiled without it.
3. **`from sifr.python_core import Object`** is not matched by the regex (`sifr\.python\s+import` doesn't match `sifr.python_core`), yet `python_core` publicly exports `Object`, `LocalCallback` (whose `callable: Object` field is another handle source), and the callback constructors.
4. **Legal-grammar textual evasions.** `from sifr . python import Object, import_module` (spaces around the dot) and backslash line continuation both parse as ordinary Python (the Ruff-fork grammar) and both slip through — the non-parenthesized branch captures only up to end-of-line.

Consequently `verification/areas/python_interop/README.md:69-74` overclaims: "the example runner rejects raw `Object` imports, raw call/conversion helpers, and `@trust_python_dynamic` before execution" — most conversion helpers and all import-free call helpers are not rejected. Today's eleven examples are honestly migrated (I read them all), so no current certification is false — but the mechanism that is supposed to keep the certification honest going forward is fail-open across the most ergonomic raw path the project itself just built in M16.

**Remediation:** invert to an allowlist. For ordinary example sources, flag any name imported from `sifr.python`/`sifr.python_core` outside `{PythonError, ResourceDiagnostics, resource_diagnostics, ExitCause, ExitCauseKind, ExitDecision}` (the set the migrated + biip-schwifty + sqlite-context fixtures actually need), with a whitespace-tolerant module pattern (`sifr\s*\.\s*python(_core)?`) that also spans continuations (scan with the newline-insensitive parenthesized branch or strip `\\\n` first). Additionally reject the import-free raw-method tokens (`.call_method(`, `.get_attr(`, `.get_item(`, `.call(` on non-declaration values is harder textually — at minimum reject `from_value`/`to_value` via the allowlist, which removes every `Object` source). Longer-term, the durable fix is a compiler/driver-enforced package policy (declaration-first mode) instead of a regex, consistent with how every other trust decision in this phase is compiler-owned.

### MINOR 2 — The new rejection path is never exercised end-to-end by self-tests

`example_packages.py:166-168` unit-tests `imported_raw_api_symbols` against one seed string, but no self-test drives a raw-importing source through `validate_source_presence`/`build_examples_report`, so the actual fail-closed branch (`example_packages.py:218-231`) and its `examples-failed` propagation are untested. A regression that, e.g., inverts the condition or drops the `continue` would pass all 19/19 today. **Remediation:** add a self-test that writes a synthetic fixture containing `from sifr.python import Object` into a temp fixtures root and asserts the report is `examples-failed` with the `ordinary example uses raw Python API` reason (and a sibling for `@trust_python_dynamic`).

### MINOR 3 — Two stdout markers now assert metadata the bridges no longer verify

- Torch: marker `...:dtype=float32` (`runner/ml_examples.py:14`), but `torch_dlpack/python_bridges/torch_example.py` never asserts `doubled.dtype == torch.float32` — the old fixture checked `DlpackTensor.dtype == "float32"` from typed protocol metadata.
- PyArrow: marker `...:kind=array:...` (`runner/library_examples.py:20`), but `pyarrow_capsule/python_bridges/pyarrow_example.py` infers "array" only from `len(capsules) == 2`; the old fixture asserted `ArrowCapsule.kind == "array"` and `copy_possible`. Checking the two PyCapsule names (`arrow_array`/`arrow_schema`) would restore exactness.

Both are one-line bridge assertions. Reports should not print protocol facts the run didn't check.

### INFO 4 — `execution_model` is asserted, not derived

`_run_case` hardcodes `"execution_model": "compiled-sifr-declaration"` (`example_packages.py:479`), so the report-level `invalid_execution_models` check (`:81-91`) is tautological for real runs. It is truthful — the case genuinely builds and runs a Sifr package through the sifr CLI — but the field is a label rather than evidence. Acceptable for Wave 1; worth keeping in mind when Wave 3 consolidates these rows into the certification ledger.

### Observations (not actionable)

- All eleven migrations chose the hermetic-bridge path (single `run() -> str`); none bind library symbols as direct typed declarations. The contract explicitly permits "declarations **or** hermetic bridges", and `biip-schwifty`/`sqlite-context` cover the direct-declaration shape, so this conforms — but the compiled-Sifr side of each ecosystem example now exercises only one declaration call plus diagnostics; the library work is Python-side. Wave 2/3 compiled-binary service evidence is what will carry the heavier compiled-Sifr claims.
- Raw usage remaining in `fixtures/` (live roundtrips, buffer/arrow/dlpack/callback mechanics, `primitive_roundtrip`) is sanctioned: live lanes are Wave 2 scope and the rest are certification inputs per the README's new paragraph.
- The Wave 1 checkbox in the plan is still `- [ ]` while the added prose describes completion — consistent with the repo's convention of flipping at closure; not a defect.
- AGENTS.md constraints hold: no compiler code touched, all files well under the size cap, lockfile untouched, no new panic paths, and the dirty `third_party/ruff` submodule is outside the diff as stated.

## Verdict

The migration itself is honest, complete over the inventoried eleven, deterministic, and leak-checked. But the wave's defining deliverable — a *fail-closed* source-policy guard — is demonstrably fail-open across four bypass classes (including the raw-method API this project shipped one milestone ago), and the README/report certify enforcement the code doesn't provide. That is an actionable major finding against the stated intended outcome.

VERDICT: NOT SATISFIED
