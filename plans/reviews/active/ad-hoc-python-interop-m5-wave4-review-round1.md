I have enough. Let me write up the findings.

Based on my review of the M5 wave 4 changes, I've identified several actionable issues. The wave delivers the core closure/outcome codegen and normative exit decision table correctly for the happy path exercised by the sqlite fixture, but has coverage gaps that will produce ill-typed Rust or incorrect exit-once semantics on plausible user programs.

## Findings

**1. `rewrite_context_control_flow` never recurses into `RustStmt::LetElse.else_body`.** (`crates/sifr_codegen/src/stmt_support_emitter/python_context.rs:491-558`)

Any Sifr narrowing pattern that lowers to `LetElse` — e.g. `if x is None: break`, `if x is None: return v`, `if not opt: continue` — emits an else body with `Break`/`Continue`/`Return(...)` that the context rewrite ignores. `if_condition_lowering.rs:354-386` produces `RustStmt::LetElse` for these patterns in the IR path (the one the Python-context body always takes because it lives inside a try closure). Inside the outcome closure, a raw `break;`/`continue;` is a Rust compile error ("cannot break outside of a loop"), and a `return Ok(v)` (already wrapped once by try-closure lowering) does not match the outcome closure's return type. Trigger: any `with connect() as c: if opt is None: break` pattern inside a Python context body.

**2. `Ok(Ok(Some(...)))` outcome arm emits an ill-typed `return` when the enclosing try does not capture returns.** (`python_context.rs:190-198`, `python_context.rs:308-327`, `loops_try_finally.rs:198-244`)

`context_return_expression_type` returns `"()"` when `try_closure_depth == 0`, but the entry-guard at `python_context.rs:87` only checks `try_closure_error_type.last()`, which is populated by `loops_try_finally.rs:227` *unconditionally* for every try/except (only the depth counter is gated on `capture_returns`). So a Python `with` inside a try/except that has no `return` anywhere in its body has `try_closure_depth == 0`, an outcome type of `Result<Result<Option<()>, bool>, PythonError>`, and an unconditionally-emitted `RustMatchArm { body: [normal_exit(), Return(Some(Ident("__sifr_context_return")))] }` whose `return ()` does not match the enclosing try-closure return type `Result<(), PythonError>`. Trigger: the very common shape

```sifr
def process() -> None:
    try:
        with connect() as conn:
            conn.execute("insert ...")
    except PythonError as e:
        log(e)
```

fails to compile. The sqlite fixture happens to avoid this because every function has a `return N` inside the try body.

**3. `classify_cause_kind` uses substring matching on rendered Sifr type names.** (`python_context.rs:568-578`)

Any Sifr error type whose rendered name contains `"Cancel"`, `"Timeout"`, `"Deadline"`, `"RuntimeFault"`, or `"WorkerRuntime"` is silently reclassified — e.g., a user-defined `CancelableTask` error → `Cancellation` cause kind, so ignored suppression is falsely recorded as cancellation evidence and the normative decision table row is picked from the wrong bucket. Conversely, canonical Sifr types that don't contain those substrings (say a hypothetical `TimedOut`) are silently downgraded to `OrdinaryError`. The classification should be driven by the actual `Type` reaching this stage (or an explicit metadata bit set during earlier lowering), not string search over the rendered name.

**4. Non-Python `PythonError` reaches `python_error_exit_body` and then silently fabricates `SifrBoundaryError` cleanup evidence.** (`python_context.rs:171-181`, `398-488`)

When `active_error_type == "PythonError"` but the error's `__sifr_python_error` is `None` (i.e., a synthetic `raise PythonError(...)` construction by user code, not a bridge-produced replay), the outer `IfLet` on `.as_ref()` falls through to `non_python_error_exit_body(..., force_ordinary=false)`, and `classify_cause_kind("PythonError")` returns `"OrdinaryError"`. That's arguably correct per spec (no live triple to replay), but `record_context_ignored_suppression("PythonError")` labels the primary cause as `"PythonError"` even though it is being handled as an ordinary Sifr error. Evidence readers cannot distinguish "originating Python error whose replay was consumed" from "synthetic PythonError treated as ordinary." Consider passing the classified cause label (e.g., `"ordinary-error"`) rather than the raw Sifr type name to `record_context_ignored_suppression`.

**5. Entered-value `LetElse` binding is never marked `mut`.** (`python_context.rs:285-294`)

`format!("Some({})", item.target)` emits `let Some(target)` unconditionally. If the with body reassigns or mutates `target` (allowed for non-opaque entered values per the plan's "converted to ordinary owned Sifr values"), the generated code fails to compile. Native with-lowering (line 55) correctly consults `self.mutated_vars.contains(&item.target)`; the Python path should do the same. Concrete trigger: `with cm() as x: x = something_else`.

**6. `python_error_exit_body`'s inner `IfLet` on `.as_mut()` uses `else_body: None`, silently dropping cleanup evidence in an unreachable branch.** (`python_context.rs:361-382`)

The outer `IfLet` established that `.as_ref()` was `Some`; the inner `.as_mut()` therefore is also `Some`. The `else_body: None` branch is dead but reads as if it intentionally handles a case where evidence is dropped. Either delete the redundant `IfLet` (unwrap directly with a `let-else` unreachable, matching the style used at line 285), or route the fallthrough to `record_context_cleanup_evidence` for defence in depth against future refactors.

**7. Test coverage is thin and the sqlite fixture is not referenced by any test infrastructure.** (`verification/areas/python_interop/fixtures/sqlite_context/context_codegen_smoke.sifr`; `python_context.rs:622-755`)

Only three unit tests exist in `python_context.rs`: rendered-string spot checks for enter/exit ordering, nested outer/inner enter order, and a direct call to `rewrite_context_control_flow`. Nothing exercises the `Ok(Suppress)`, `Ok(Propagate)`, or `Err(cleanup)` exit-decision arms, nor `active_error_type != "PythonError"`, nor mixed native+Python items, nor conversion failure, nor break/continue outside a while-else loop, nor the `try_closure_depth == 0` shape called out above (which is why finding #2 slipped through). The `sqlite_context` fixture is a manual smoke: `grep -r sqlite_context verification/` returns nothing under `verification/`, so it will silently rot. Wire the fixture into the compile-and-run harness (or a snapshot test) and add per-arm codegen tests before closing the wave.

**8. Nit: `python_context_counter` never resets.** (`lib_emitter_state.rs:96-97,228`; `python_context.rs:98-99`)

Suffix uniqueness within a function only requires resetting per function scope; monotonic-across-emitter is harmless but produces noisy diffs when unrelated changes reorder emissions and gives less useful failure names when reading generated Rust. Reset it per function like the other per-function counters (or, since names only need to be unique within the same scope, use `python_context_counter += 1` in a saved/restored fashion).

## Assessment against the required properties

| Property | Status |
| --- | --- |
| left-to-right enter / right-to-left exit | ✓ (recursion structure in `wrap_context_items`) |
| manager & entered value lifetime through exit | ✓ for the sqlite/opaque cases; entered-mutability caveat in finding #5 |
| exactly-once exit | ✓ (each outcome arm exits once) |
| enter failure skips exit | ✓ (`?` on `enter_context` propagates before any exit setup) |
| normal/return/break/continue exit with None triple + preserve action | ✓ for the arms that generate; break/continue rewrite is scoped correctly to `loop_depth` |
| originating Python replay is suppressible | ✓ (`python_error_exit_body`) |
| ordinary / timeout / cancellation / runtime causes are not suppressible and truthy is recorded | Correct in structure, but see finding #3 (fragile classification) |
| exit failure primary/secondary behavior | ✓ (`attach_secondary_python_error` + context assignment) |
| nested replay | ✓ (Arc-backed `PythonExceptionReplay`, `.as_ref()` reuse) |
| mixed native/Python items | ✓ per code inspection; not covered by any test |
| no panic on user paths | Codegen panics only on internal invariants (`unreachable!`, `panic!` in the renderer); no data-dependent panic in generated user code |
| hidden replay survives generated `PythonError` conversion | ✓ (`rust_interop_error_mapping::python_error_expr` and class-emitter constructor paths add `__sifr_python_error`) |
| generated code typechecks for all arms | ✗ — findings #1 and #2 both produce Rust type errors on realistic inputs |

VERDICT: CHANGES_REQUESTED
