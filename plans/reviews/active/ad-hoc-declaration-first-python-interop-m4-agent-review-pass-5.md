I've completed a thorough independent audit. Based on my investigation, here is my final review.

## Review Summary

**IR & HIR contracts** — `PythonInteropDeclaration` gains `cleanup: Option<PythonCleanupPolicy>` and `consumes_receiver: bool`; `HirClassKind::PythonOpaque(...)` stores the declaration on the class. `HirClass::python_opaque_declaration()` gives codegen a single source of truth. Only `Drop` and `Close` policies parse; `async_close`, `context`, and `async_context` explicitly hard-error via `PYRES_UNIMPLEMENTED_DECLARATION` (crates/sifr_lowering/src/lower/python_interop.rs:270–301). No later-milestone syntax is activated.

**Lowering** — Opaque metadata is collected in a pre-pass in `mod_impl.rs` so signature validation sees it regardless of declaration order. `is_direct_type` recursively accepts nested lists, tuples, dicts, closed records, and opaque classes. Non-send is enforced via `parent_class: Some("NonSend")` and constructor removal in `class_type_collection.rs:695–708`, wiring into the existing `NonSend` marker check used by task-scope and IPC-payload lowering. Opaque classes with structural fields are rejected at `class_body_lowering.rs:670–676`.

**Must-use side table** — `ctx.live_must_use_bindings` is recorded on `ann_assign`, `assign` (including first-assignment specialization), reset on reassignment (`control_flow.rs:344–357`), rejected on `_` discard (`control_flow.rs:259–265`), and transferred on return through `Name`, `ListLiteral`/`SetLiteral`/`TupleLiteral`/`DictLiteral`, `ConstructorCall`, `IteratorCall`, comprehensions, `OkWrap`/`QuestionMark`/`ErrWrap`, and `IfExpr` (`return_lowering.rs:130–212`). Control-flow join validation lives in the extracted `must_use_obligations::validate_branch_join` (`must_use_obligations.rs:6-48`) and is invoked from `lower_if` after `saved_moved`/`branch_moved_states` are restored. Function-exit rejection sorts alphabetically before diagnosing, so ordering is stable. `python_consuming_methods` is filled from the pre-pass so `mark_moved_with_flow` fires at the call site (`methods_lambdas_and_comprehensions.rs:257-266`). Class-method must-use narrows `ast_convention_to_param` to must-use types only (pass 4 correction), preserving ordinary class ABI.

**Codegen** — `python_interop_direct.rs` recursively lowers inputs (`from_list_results`/`from_tuple_results`/`from_dict_results`/`from_record_results`) and outputs (`list_items`/`tuple_items`/`dict_str_items`/`record_field`). `expect_instance` gates opaque factory returns and releases the rejected identity on mismatch (`object_ops.rs:181-196`). `semantic_close` consumes the receiver, poisons on Python failure, and closes on success (`opaque_ops.rs:5–28`). `RustParam::SelfValue` is emitted only for `consumes_receiver` methods and the opaque class derives only `Debug`. `python_opaque_classes` is populated from `emit_module_body` and threaded into method-body lowering.

**Driver & plan** — `PythonTargetProbe.expects_type` propagates through the plan into `apply_python_interop_metadata`; the runtime probe reports `is_type` via `isinstance(value, type)` and rejects opaque targets that resolve to a non-type; the signature validation is scoped to `PythonInteropDecoratorKind::Function` so opaque callables aren't re-validated as callables. Cache key gains `:callable|type:` axis.

**Runtime** — `ForeignObject` now holds a `Mutex<ForeignObjectState>` with `Open`/`Poisoned`/`Closed`. `poison()` transitions Open→Poisoned; both `Poisoned` and `Open` drop paths release via `release_object`, so the poisoned identity still decrefs when the last `Arc` drops. Boundary paths are attached via `with_child_path` and propagated through `list_items`/`tuple_items`/`dict_str_items`/`record_field`. `expect_instance` releases the resolved factory and, on mismatch, the rejected value before returning `Err`, closing the leak seen in earlier passes.

**Non-send identity** — `parent_class = "NonSend"` for opaque classes trips the existing NonSend enforcement in `task_scope_calls`/`ipc_payload_calls`; no `send=` grammar exists.

**Demo & fixture** — `demos/m4_demo/run.sh` invokes the biip/schwifty case through the Python interop runner and checks the biip/schwifty stdout marker; `builtins` is added to `import_roots` so `builtins.tuple` is authorized.

**Guardrails** — `check_file_size_guardrails.py` PASSES (touched files: python_interop_direct.rs 899, control_flow.rs 892, class_body_lowering.rs 788, python_interop.rs 758 — all under 900). HIR maintainability guardrails pass.

**Phase tracker** — `plans/issues/active/ad-hoc-declaration-first-python-interop.md` now marks M4 checked and links PR #2935.

## Verdict

Every M4 acceptance clause (recursive conversion, boundary-path errors with exact-once release, opaque lifecycle with distinct drop/close/poison, non-send identity, ordinary class ABI unchanged, no later-milestone activation) has a corresponding implementation and evidence path. Pass 1's four correctness gaps (compound-return panic patterns, popped-scope must-use, reassignment abandonment, aggregate/comprehension transfer) are fixed and covered by unit tests. Pass 2's file-size regression on `control_flow.rs` is resolved by the responsibility-based extraction in `must_use_obligations.rs`. Pass 3 verified the extraction; pass 4 verified the narrow must-use convention restoration; the authoritative gate then passed. The current implementation adds no new regressions on top of that.

No actionable blocker.

SATISFIED
