I've verified both fixes and swept the rest of the M3 surface for other actionable gaps.

**Finding 1 (positional `python.omit` misbinding) — FIXED**
- `crates/sifr_codegen/src/python_interop_direct.rs:43-86` — the wrapper body now tracks `forward_positional_by_name`; once any positional shape is `omit_when_absent`, every subsequent positional (omittable or not) is emitted as a `("name", handle)` push into `__sifr_python_kwargs`, and omittable variadic shapes short-circuit to `None`. So `f(1, None, Some(3))` produces `args=[1]`, `kwargs=[("c",3)]` → Python `f(1, c=3)`.
- `crates/sifr_lowering/src/lower/python_interop.rs:156-167` — hard-errors `PYCALL_INVALID_SHAPE` when `PositionalVariadic` follows any `omit_when_absent` positional; covered by `positional_variadics_after_omission_are_rejected`.
- `crates/sifr_driver/src/build/python_interop.rs:164-253` — target-probe validator excludes omittable positionals from the strict positional count (`take_while(!omit_when_absent)`), and every positional in the `forward_positional_by_name` window must exist by name in the target and must not be `POSITIONAL_ONLY`; covered by `omittable_positional_parameters_require_keyword_capable_target_parameters`.
- `verification/areas/python_interop/fixtures/declaration_sync/complete_call_shapes.sifr:23-52` — adds `dumps_with_positional_omits(7, ensure_ascii=False)` end-to-end, which the create-pr suite exercises.

**Finding 2 (bridge target reserved with SIFR-PYRES-0002) — FIXED**
- `crates/sifr_lowering/src/lower/python_interop.rs:232-240` — `parse_sync_function` checks `target.root() == Some("bridge")` and emits `PYRES_UNIMPLEMENTED_DECLARATION` (SIFR-PYRES-0002) anchored at the target span before any declaration is registered. Because lowering emits a hard error and skips the declaration, the empty-body `Ok(())` fallback in `function_emitter/generator_bodies.rs:362-374` is unreachable for bridge targets. Codegen still keeps defense-in-depth (`python_interop_direct.rs:18-20`). Covered by `bridge_target_is_a_hard_error_while_reserved`.

**Other M3 surface swept — no new actionable gaps**
- Caller-side lowering (`mod_impl.rs:170-213`, `expressions/regular_calls.rs:266-289`) wraps supplied arguments for omittable params as `__sifr_python_present_argument(v)` → `Some(v)` and unsupplied ones as `__sifr_python_omitted_argument()` → `None`; wrapper params for omittable slots are `Option<T>` (`generator_bodies.rs:308-327`).
- Cross-module propagation of python call shapes and omit defaults is wired through `imported_defaults.rs`/`imports.rs`.
- Non-Function decorator kinds (opaque, coroutine, callback, buffer, arrow, dlpack, context.*) remain hard-errored with SIFR-PYRES-0002 in `collect_python_interop_declarations`.
- `blocking_io` effect enforcement (`sync_python_declaration_is_blocking_in_async_context`) and `PythonError` mapping are unchanged and intact.
- RAII cleanup on wrapper failure paths is preserved: converted handles are stored in the args/kwargs vectors and cascade-drop on early `?` return.

Minor non-actionable observations (cosmetic, correctness unaffected):
- The build-time validator's "omittable positional parameter '{name}'…" message also fires for non-omittable positionals that inherit the by-name path from an earlier omittable sibling — the reject is correct but the wording is slightly imprecise.
- `parse_sync_function`'s `.filter(|root| *root != "Self")` on `required_import_root` is dead code (parse_target_path already rejects `Self` targets earlier).

Neither warrants a code change for M3.

SATISFIED.
