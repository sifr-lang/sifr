Focusing on the three follow-ups plus a sanity check that nothing else regressed.

**Follow-up 1 - bootstrap.rs dead-code removal.** Safe. `local_callback`/`threadsafe_callback` now live in `sifr.python_core` and reach `sifr.python` only via the re-export path (bootstrap.rs:211+), so `result.module.functions` for `sifr.python` never contains them. The deleted branch was unreachable, and the intrinsic-name set for `sifr.python_core` is populated by the `_sifr.python` import walk at lines 200/205 - that side still stamps the underlying `py_*` names correctly. No downstream reader loses information.

**Follow-up 2 - `raw[4]` for `kind`.** Semantically equivalent and slightly more robust. `crates/sifr_runtime/src/python/callback_ops.rs:56-63` defines `label()` returning exactly `"local"` / `"threadsafe"`, and `callback_raw()` in `crates/sifr_stdlib/src/python.rs:621-629` forwards it verbatim as tuple element 4. Both `_local_callback_from_raw` and `_threadsafe_callback_from_raw` are only ever called from their matching kind's constructors and echo helpers, so `raw[4]` matches the previous hard-coded string one-for-one. Wiring the field also makes the runtime string no longer wasted work.

**Follow-up 3 - probe comment.** Comment accurately explains why the special case exists (closure-generic constructors + concrete `PythonError` can't be spelled by the generic probe template). The `is_python_raw_callback_probe` name + comment together read cleanly for a future contributor.

**No regressions in the surrounding diff.** Adapter scope (`is_python_callback_constructor_target` / `is_python_raw_callback_type`) and the manifest/guardrail closures are unchanged from round 1; `_call_object_callback` still marshals `Object -> handler -> (handle, token)` matching the previous compiler-lowered adapter.

**PR-ready.** No blockers introduced by the follow-ups and nothing missed in round 1.
