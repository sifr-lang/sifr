I've reviewed the working-tree diff against main for M5 Wave 1. Below is my assessment.

## What the diff does

- **PYCTX-0001 activation**: `reserved.rs` removes the reservation; `python_interop.rs` (registry) adds active entry with fixture; `registry.rs` adds to `ACTIVE_DIAGNOSTIC_CODES`; `docs/errors/SIFR-PYCTX-0001.md` and both diagnostic-code index docs updated.
- **Sysroot types**: `_sifr/python.sifr` defines `ExitCauseKind` (`Enum`), `ExitCause(NonSend)` with `__init__`, `ExitDecision` (`Enum`). Re-exported via `sifr/python_core.sifr` and `sifr/python.sifr`.
- **Opaque `cleanup=context`**: `python_interop.rs` accepts `context`; `async_close` and `async_context` still emit PYRES-0002 via `reserved_cleanup`.
- **Context enter/exit**: `python_interop/context.rs` parses `@python.context.enter(Self.__enter__)` / `.exit(Self.__exit__)`, enforces exact target segments, and enforces borrowed-`self` on enter vs. `own self` on exit; signature validator enforces `Result[..., PythonError]`, empty params on enter, single `ExitCause` param on exit, `ExitDecision` ok on exit, and `is_direct_type` on enter's ok.
- **Class-level context validation**: `validate_context_class_methods` requires exactly one enter and one exit, rejects context decorators outside `cleanup=context`, and rejects a distinct opaque entered value whose cleanup is not `Drop`.
- **Direct-use guards**: `core_and_calls.rs` rejects `ExitCause(...)` outside sysroot private lowering with PYCTX; `methods_lambdas_and_comprehensions.rs` rejects direct method calls to registered `python_context_exit_methods` with PYCTX.
- **Must-use split**: `must_use_obligations.rs` adds `MustUseObligationKind::{CloseLike, ContextOnly, AsyncContextOnly}` and `MustUseObligation { kind, label }`; `mod_context.rs::must_use_obligation_for_type` classifies opaques. `return_lowering.rs::transfer_return_ownership` transfers only `CloseLike` via `mark_moved_with_flow` and emits PYCTX for `ContextOnly`/`AsyncContextOnly` on returns and aggregate paths (list/set/tuple/dict/constructor/iterator/ok-wrap/question/errwrap/if-expr/list-comp/set-comp/dict-comp/generator).
- **Consuming-method gate**: `class_body_lowering.rs::has_unmatched_consuming_method` now recognizes `ContextExit` as a valid consuming method under `cleanup=context` and preserves the M4 `Self.close` rule under `cleanup=close`.
- **File-size / prepass move**: `mod_impl.rs` delegates the opaque prepass to `collect_python_opaque_classes` in `python_interop.rs`; `python_context_exit_methods` is populated alongside `python_consuming_methods`. `python_interop.rs` is 808/900; `python_interop/context.rs` is 208/900.

## Interim safety analysis

- **Preserves M4 close transfer**: `CloseLike` still mark-moves through `return_lowering` and aggregates; `has_unmatched_consuming_method` still requires exactly one `@python(Self.close)` under `cleanup=close` and rejects extras.
- **Ordinary class / native `with`**: `statement_dispatch.rs` sync-with is untouched. A `cleanup=context` opaque binding routed through native `with` does not discharge its `ContextOnly` obligation, so it hard-errors at function exit via `reject_live_must_use_bindings_at_function_exit` (OWN_USE_AFTER_MOVE with `ContextOnly` label).
- **Declaration order**: opaque prepass runs before signature and class-body lowering; `validate_context_class_methods` sees `python_opaque_classes` fully populated regardless of textual order.
- **Ownership escape**: return/aggregate paths — PYCTX. Reassignment — caught in `control_flow.rs`. Owned-arg call — moves binding, but the callee re-records the obligation in `annotations_and_function_lowering.rs::…for param in &params { … record_must_use_binding }`, so the obligation propagates until some function exit rejects it. Direct `ExitCause` construction and direct exit-method calls — PYCTX.
- **PYCTX consistency**: all new M5 validation emits `SIFR-PYCTX-0001`; only `async_close`/`async_context` cleanup atoms and top-level misapplication of `.context.enter/exit` (via `collect_python_interop_declarations`) still report `PYRES-0002`, which is consistent with the wave's still-reserved surfaces.

## Non-blocking observations (do not gate the wave)

- `validate_context_class_methods` only inspects `Type::Class` for the entered value. `is_direct_type` allows `list[T]`, `tuple[…]`, `Union[T, None]`, and `Dict[str, T]` of an opaque — so an enter declared as `Result[list[Session], PythonError]` (Session `cleanup=close`) passes the distinct-opaque check silently.
- `ExitCause`, `ExitDecision`, and `PythonError` are matched by class-name string only; a user-local `class ExitCause: pass` satisfies the parameter/return type check. Consistent with existing name-based sysroot conventions.
- The direct-construction guard is `func_name == "ExitCause"` at the source call name — aliasing (`X = ExitCause; X()`) bypasses it, and any user class also named `ExitCause` gets PYCTX at construction.
- `with tx:` on a `cleanup=context` opaque hard-errors, but via `OWN_USE_AFTER_MOVE` (must-use), not via a dedicated PYCTX telling the user sync-context execution is reserved for W3/W4.
- Top-level `@python.context.enter` / `.exit` (outside a class) emits `PYRES-0002` "belongs to a later phase" rather than a PYCTX explaining they are method-only.

None of these are correctness holes for a reserved, non-executing wave.

## Blockers

None.

SATISFIED
