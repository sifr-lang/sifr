All inspection and independent verification is complete. Here is the review.

---

# M16 Frozen Whole-Diff Review — Raw API Ergonomics On Shared Ownership

**Reviewer:** Independent milestone-closure review (agent High)
**Range:** `3f974f33b` → `df2f72de3` (single commit `feat(python): add typed raw API ergonomics`), PR #2996
**Scope reviewed:** entire diff (49 files, +1133/−85), plus surrounding sources in `sifr_lowering`, `sifr_codegen`, `sifr_runtime`, `sifr_stdlib`, `sifr_type_system`, the e2e fail harness, verification profiles/runner, plan invariants (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`), both durable interop contracts, `internal_docs/architecture.md`, docs, and the demo.

## Independent verification performed (read-only + local tooling)

- Compiled `verification/areas/python_interop/fixtures/primitive_conversion/raw_typed_ergonomics.sifr` with `sifr check` — clean, no diagnostics.
- Compiled all three new fail fixtures — the annotated codes (`SIFR-PYCONV-0001`, `SIFR-ASYNC-0003` ×2) fire at the exact annotated lines (see Finding 1 for extra pollution).
- Ran `sifr_runtime --features python`: raw coroutine suite **4/4** (owned-loop identity, concurrency, new checked Python failure, shutdown cancellation/join), owned-loop async runtime **14/14**, object-ops **6/6** including the new dict-mapping-key regression — all pass locally.
- Ran the new driver frontend test `test_raw_python_generic_conversion_and_object_methods_share_runtime_bridge` — passes; generated Rust reuses `from_str`, `py_call_attr_keyed`, and `__sifr_declaration_object_result`.
- Probed compiler edge cases: unannotated `to_value` is rejected with `SIFR-PYCONV-0001` (never silently untyped); raw `Object` methods without the `PythonError` import are rejected with the new guidance message, and the double emission from `for_call`/`resolve_method_type` is correctly collapsed by the driver's span-keyed diagnostic dedup; `await task.sleep(0.0)` in an otherwise-clean async fixture yields exactly one `SIFR-ASYNC-0003`.

## Invariant assessment (all verified against source, not just the patch)

- **One conversion authority.** `from_value`/`to_value`/`kwarg` validate through the exact declaration predicate `is_direct_type` (`crates/sifr_lowering/src/lower/python_interop/direct_validation.rs:9`) and generate code exclusively through the existing declaration converters `input_conversion`/`output_value_expr` (`crates/sifr_codegen/src/python_raw_api_codegen.rs`). No parallel converter, no second ownership state. `set[int]` and other out-of-set types hard-error with `SIFR-PYCONV-0001`.
- **One sealed identity.** Method-style ops gate on `is_python_object_contract()` (identity `_sifr.python.Object`, sealed, `crates/sifr_type_system/src/types/python_interop.rs:37`); no handle/token fields are exposed; `kwarg` erases only to the existing `(str, Object)` shape; `py_call_keyed`/`py_call_attr_keyed` reuse `python::call_object`/`call_attr` over the same tracked store as every prior raw and declaration path.
- **Checked errors.** All new surfaces return `Result[…, PythonError]`; codegen maps runtime errors through `bridge_error_expr` into the validated five-field source contract; the missing-`PythonError`-import case is a compile error, not a degradation.
- **Ownership/cleanup equivalence.** Raw methods lower as all-borrow signatures, receivers are borrowed (`&` in generated Rust), ordinary automatic drop applies; the package tests and fixture assert `live_objects` equality across success and handled-error exits; the owned-loop test was converted from a manual close chain to automatic release and its assertion strengthened (`:released` marker + diagnostics equality).
- **Owned loop, no per-call loop.** `run_coroutine_blocking` routes through `async_runtime::ensure_started` and the shared submission path (`crates/sifr_runtime/src/python/coroutine_ops.rs:4-11`); success/failure/concurrency/cancellation/shutdown are all covered and pass locally.
- **Async blocking effects.** The stdlib intrinsics are `@blocking_io` (standard rejection applies); compiler-known `Object` methods get the dedicated `reject_async_direct_raw_python_method` rejection; both are fixture-covered.
- **Root-cause fixes, not shims.** The `record_field` change (dict mapping keys win over dict attributes such as `.values`) is a genuine correctness fix at the runtime root with a targeted regression test; the Result-ok type-var fallback and return-position contextual typing in `regular_calls.rs`/`return_lowering.rs` are general, guarded inference improvements rather than intrinsic-special-cased hacks.
- **Guardrails/metadata.** All touched hand-maintained files are under 900 lines (largest: `methods_lambdas_and_comprehensions.rs` at 864, reduced by this diff); the retained-intrinsics and native-adapter-reachability inventories cover the three intrinsics and two keyed adapters; the capability row upgrades raw-api evidence (including cancellation) with concrete named owners; `docs/python-interop.mdx` accurately describes the closed conversion set, automatic release, and owned-loop coroutines; the demo's asserted output matches its code path.

## Findings

### MINOR-1 — New async fail fixtures import a nonexistent symbol, polluting the negative evidence
**Files:** `crates/sifr/tests/e2e/fail/python_raw_typed_conversion_requires_offload.sifr:2` and `crates/sifr/tests/e2e/fail/python_raw_object_method_requires_offload.sifr:2`
**Evidence:** Both fixtures contain `from sifr.task import sleep` / `await sleep(0.0)`, but `sifr.task` exports no `sleep`. Compiling each fixture emits the expected `SIFR-ASYNC-0003` **plus** two unrelated diagnostics: `SIFR-NAME-0004` ("module 'sifr.task' has no member 'sleep'") and `SIFR-NAME-0002` ("undefined function: 'sleep'"). The fail harness (`crates/sifr/tests/e2e_support/e2e_entrypoints.rs:288-337`) only requires the expected code to be present, so the suite passes, and I confirmed the intended rejection genuinely fires at the annotated line through the new M16 paths.
**Rationale:** A negative fixture should be a minimal program whose sole defect is the behavior under test; these fixtures would still fail to compile even if the blocking-effect rejection regressed to a different (non-diagnosed) failure shape, and they deviate from the canonical pattern in `blocking_io_direct_call_in_async_rejected.sifr`, which uses `await task.sleep(0.0)` with no import. I verified that swapping to `await task.sleep(0.0)` (and dropping the import) produces exactly one diagnostic — the expected `SIFR-ASYNC-0003`.
**Remediation:** In both fixtures, delete the `from sifr.task import sleep` line and change `await sleep(0.0)` to `await task.sleep(0.0)`.

### MINOR-2 — `internal_docs/architecture.md` interop status not updated for M16 in the frozen merge unit
**File:** `internal_docs/architecture.md:54`
**Evidence:** The interop status paragraph enumerates milestone capabilities through M15 (LSP declaration authoring) but says nothing about M16's typed generic conversion, compiler-known `Object` method surface, typed kwargs, or raw automatic-release/owned-loop coroutine ergonomics. The diff touches `internal_docs/` only for the two guardrail TOML inventories. Every prior milestone (M13 `a13a6608c`, M14 `dfbf31532`, M15 `82fec1296`/`d4bcba9b6`) updated this status line within its own milestone merge unit, and `AGENTS.md` lists the architecture doc among the tracking files to update per completed item. Since this review covers the frozen PR head intended for closure and merge, the omission is part of the deliverable under review.
**Rationale:** Milestone-closure completeness — internal architecture status is part of the documented Wave 5 closure surface and the established per-milestone workflow; leaving it stale means the authoritative internal status no longer reflects the shipped raw-API contract.
**Remediation:** Extend the interop status paragraph in `internal_docs/architecture.md` with one or two sentences covering M16: typed `from_value`/`to_value`/`kwarg` over the single declaration conversion authority, compiler-known checked `Object.get_attr`/`get_item`/`call`/`call_method`, ordinary automatic raw-object release, and raw coroutines on the application-owned loop.

## Non-findings (observations, no action required)

- `raw_typed_ergonomics.sifr` is presence-checked by the interop scaffold rather than executed there; this matches the established pattern for every `REQUIRED_SOURCE_FIXTURES` entry (e.g. `primitive_roundtrip.sifr`), the executable positive owner is the driver package test, and I independently confirmed the fixture compiles cleanly today.
- Duplicate keyword names in raw `Object.call`/`call_method` kwargs lists resolve last-wins via dict construction (unlike the declaration path's explicit duplicate check in `call_object_owned`); this is pre-existing raw-path semantics (`py_call` behaves identically) and not introduced or worsened by this diff.
- The unbound-`T` message for unannotated `to_value` ("`T` is not in the declaration conversion set") is precise and correctly spanned, though slightly indirect; acceptable.
- The `sifr_runtime --features python` cargo suites are exercised by focused validation rather than a create-PR cargo lane; this run model predates M16 and I re-ran the suites locally to confirm they pass at this head.

## Verdict

Two minor, precisely-scoped defects remain: fixture hygiene in the two new async fail fixtures, and a missing internal architecture status update. Everything else — single conversion authority, sealed identity, checked errors, automatic-drop/cleanup equivalence, owned-loop raw coroutine behavior across all five conditions, unsupported-type diagnostics, guardrail metadata, public docs, demo, and validation evidence — holds up under independent inspection and local re-execution.

VERDICT: NOT SATISFIED
