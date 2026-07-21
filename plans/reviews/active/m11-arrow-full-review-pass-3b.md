All verification is complete. Consolidating the pass-3 review.

---

# M11 Arrow C Data Interface — full milestone review, pass 3 (PR #2991, `main...HEAD` at 93d8932cb)

Every pass-2 actionable finding was independently re-verified against a freshly built branch compiler (all blocker-class claims reproduced or refuted live, not taken from agent reports), plus three parallel read-only audits over the lowering, runtime/certification, and docs/evidence surfaces.

## Verdict summary

**NOT SATISFIED.** The remediation commit genuinely closes everything pass 2 enumerated — both blockers, all four majors, docs, and evidence. But the fix commit itself introduces one new check-clean→rustc regression (protocol class-style calls), and the owned-transfer family it was closing is still open in two method-argument positions it didn't cover: explicit `__init__` constructors and `__call__` objects both accept an owned Arrow value twice check-cleanly and fail in rustc. By the standard both prior passes applied — a check-clean program must never fail downstream, and "ownership transfers once and remains moved" must hold on every consumer shape — these are blocking.

## Pass-2 disposition (all verified live or in code)

| Pass-2 finding | Disposition |
|---|---|
| **NB-1** interop-method owned Arrow ICE | **Closed.** `@python(Self.push)` with `own value: python.ArrowArray` is check-clean and emits a body that mirrors the free-function path — `prepare_argument` commits the move before the call, `finish()` + `reconcile_arrow_argument` after (`crates/sifr_codegen/src/python_interop_direct.rs:611-623`). Reproduced live for `ArrowArray`, `ArrowSchema`, `ArrowStream`; pinned by `arrow_owned_consumer_method_prepares_and_reconciles_argument`. `mut own`, `omit`, and coroutine shapes all still fail closed with PYZC-0001 (reproduced live). |
| **NB-2** method args never consumed | **Closed for every enumerated path.** Sequential (`h.consume(v); h.consume(v)`), same-call positional (`consume2(v, v)`), same-call keyword (`consume2(v, b=v)`), `@staticmethod`, `@classmethod`, `super()`, protocol-typed receivers, and inherited methods are all OWN-0001-rejected at check — all ten probes reproduced live. New `method_argument_ownership.rs` reuses the hardened `consume_owned_value`; index alignment verified (params exclude `self`, args returned in param order). Pinned by `method_calls_consume_owned_arrow_arguments_once` and `static_super_and_buffer_method_calls_consume_owned_arguments`. **But the family is not closed** — see B-2/B-3 below. |
| **M-1** silent plain-method regression | **Closed as an intentional, documented alignment.** Free functions reject the identical borrowed-escape shape (verified live), `docs/language/ownership.mdx:20-22` documents the method contract, `plain_method_move_parameters_follow_borrow_by_default` pins it, and the `own` escape hatch compiles and runs (verified live, including generic `Box[T]` and `own f: Callable`). The channel ripple is handled: stdlib `ChannelSender.send(own value)` is now a true move (`lib_runtime_needs.rs` threads the value back out via `Full(pending)` on backpressure), bootstrap test pins the owned convention, and I verified live: 6 values through a capacity-1 bounded channel delivered exactly once in order; `send(v); send(v)` on the same heap variable is OWN-0001-rejected; Copy values remain re-sendable; `Pool.map`'s `own items` convention still compiles and runs (parallel demo). |
| **M-2** public docs said "borrowed" schema | **Closed.** `docs/python-interop.mdx:361-362` now says required keyword-only `own`, "transfers"; no "borrowed schema" language remains anywhere in `docs/`/`internal_docs/`. |
| **M-3** raw consumption-state derefs | **Closed.** All five `*_consumption` readers return `Result` and route through `checked_ref` (`arrow_ops/abi.rs:209-266`); misalignment is an error, not treat-as-consumed, and the error path still releases exactly once (`PythonArrowArgument::finish`/`Drop` reconciliation). Pinned by `rejects_misaligned_payloads_and_reserved_device_types` (acquisition + finalization). |
| **M-4** untested enforcement points | **Closed.** Distribution re-verification got an extracted seam (`validate_distribution_versions`) with drift/probe-failure tests (`package_python_certifications.rs:79-119`); kwonly defaults pinned at lowering level and verified live at runtime (`c.add()` → 5, `c.add(extra=7)` → 7). |
| Compiled evidence gaps | **Closed.** `arrow_declaration_compiled.sifr` now has `@python.arrow(Self, schema=omitted)` acquisition on a `@python.opaque` class and a compiled owned method consumer (`ArrowSink.consume(own value)` called with a real array), with zero-leak accounting, the `method-transfer=len3` marker cross-checked by the fail-closed evidence validator, and the lane blocking in all four profiles. |

## Blocking findings

### B-1 — `ProtocolName.method(...)` is now check-clean and fails at rustc (new regression from 93d8932cb; reproduced live)

```python
class Duck(Protocol):
    def quack(self) -> None: ...

def main() -> None:
    Duck.quack()
```
`sifr check` → no errors; `sifr build` → rustc error (`SIFR-BUILD-0005`), reproduced live. On main this was check-rejected with CLASS_MISSING_MEMBER, because the old lookup matched only `Some(Type::Class {...})`. The new `try_lower_class_method_call` uses `method_function_type` (`crates/sifr_lowering/src/lower/expressions/method_argument_ownership.rs:6-14`), which also matches `Type::Protocol`, so a protocol method lowers to `HirExpr::Call { func: "Duck::quack" }` with no corresponding Rust item. Not Arrow-specific — any protocol. **Remediation:** restrict the `try_lower_class_method_call` lookup to `Type::Class` (or explicitly reject protocol receivers with CLASS_MISSING_MEMBER), plus a pinning test.

### B-2 — Explicit `__init__` discards `own` on affine parameters; double-construction is check-clean and guaranteed E0382 (reproduced live)

```python
class Holder:
    value: python.ArrowArray
    def __init__(self, own value: python.ArrowArray):
        self.value = value

def main() -> ...:
    v: python.ArrowArray = make([1, 2, 3])
    a: Holder = Holder(v)
    b: Holder = Holder(v)   # check-clean!
```
`sifr check` → no errors; emit shows `fn new(value: PythonArrowArray)` (by value) called twice with `v` — a guaranteed rustc E0382. Root cause: the explicit-constructor `FunctionType` is built with `FunctionType::new` (`class_type_collection.rs:593`), which assigns Borrow to all move-typed params and discards the written `own` (`sifr_type_system/src/types/definitions.rs:307-324`), so the call-site consumption loop skips the argument — while `class_method_emitter.rs`'s `new` branch emits constructor params by value unconditionally. The lowering path is pre-existing, but storing an Arrow resource in a class via its constructor is a core M11 consumer shape, and it directly violates the acceptance item "Ownership transfers once and remains moved." The related `super().__init__(value)` shape is also check-clean with reuse of `value` afterwards (reproduced live; the emitted constructor additionally reorders `Base::new(value)` after the body statements). **Remediation:** give explicit-`__init__` parameter lists real conventions (same path as methods), run owned-argument consumption over constructor calls (`ClassName(...)` and `super().__init__(...)`), with sequential/double tests for Arrow and Buffer.

### B-3 — `__call__` objects never consume owned affine arguments (reproduced live)

```python
class Sink:
    def __call__(self, own value: python.ArrowArray) -> Result[None, PythonError]: ...
sink(v)
sink(v)   # check-clean!
```
`sifr check` → no errors; emit shows `sink.__call__(v)` twice against `fn __call__(&self, value: PythonArrowArray)` — E0382. The callable-object path in `regular_calls.rs:169-187` returns `HirExpr::MethodCall` immediately without the convention-based consumption loop that both plain calls (`regular_calls.rs:458-472`) and, since this commit, method calls run. Pre-existing gap, but it is a method-argument position — exactly NB-2's family — reachable with every Arrow kind M11 ships. **Remediation:** run `consume_owned_method_arguments` over the `__call__` signature in that path, with a test.

## Non-blocking findings

**Major**
- **Stale durable doc:** `internal_docs/python_interop_declaration_architecture.md:345-346` still says "Arrow and DLPack decorator forms remain reserved" while the capability matrix, sibling docs, and the shipped code all declare Arrow active. This file is listed as a durable contract by the plan and was never touched on the branch; only DLPack should remain reserved.

**Minor**
- `super().greet()` no longer fills parent defaults: `try_lower_super_method_call` passes `defaults: None` (`method_argument_ownership.rs:66-74`) instead of `ctx.function_defaults["{Parent}.{method}"]`, so omitting a defaulted parameter is falsely rejected with SIFR-CALL-0004 (reproduced live) while the identical instance call fills it. No previously-working program breaks (the old path silently dropped kwargs and failed at rustc), but it's an inconsistent false rejection — pass the parent defaults key.
- `ClassName.instance_method(...)` called class-style is check-clean when arity coincides and emits a receiverless `Holder::consume(v)` → rustc E0061 (reproduced live, also with a plain `int` method). Pre-existing on main (old code accepted the same shape); ownership is at least consumed once now. Worth rejecting instance methods in `try_lower_class_method_call` while fixing B-1.
- `super().nonexistent()` still lowers silently to a `SuperCall` that fails at rustc (pre-existing missing diagnostic).
- Compiled evidence: `consume_method_length` returns the constant `3` after `sink.consume(value)`, so the `method-transfer=len3` marker proves the call succeeded but doesn't measure the transferred array (the zero-leak accounting still covers cleanup).
- The misaligned-finalization test pins only the schema reader; the other four rely on sharing `checked_ref`.
- `consumption_state` error path skips `drain_pending_releases` (deferred, not leaked); Arrow prep in methods skips the owner-failure-evidence wrapper (diagnostic quality only); `checked_ref` returns an unconstrained lifetime; docs don't state verbatim that the requested schema stays consumed on producer failure (internal doc does).
- Several touched files sit at 897–899 lines, one edit under the 900-line guardrail (`core_and_calls.rs`, `lib_runtime_needs.rs`, `check_and_package_commands.rs`, `python_interop_direct.rs`).
- Working-tree hygiene: `third_party/ruff` has an uncommitted formatting-only local change (`parser/expression.rs` line join) — must not ride along into the PR.

## What is verifiably solid at HEAD

The entire pass-2 closure list held up under independent reproduction: the interop-method Arrow path emits the same hardened prepare/finish/reconcile protocol as free functions; owned-argument consumption is enforced across instance/static/classmethod/super/protocol/inherited/keyword/same-call shapes; the method borrow-by-default alignment is intentional, documented, and pinned; the channel runtime is move-correct under backpressure (live exact-once test); all five consumption-state readers are alignment-guarded with sound cleanup on the error path; certification distribution drift fails closed with real tests; kwonly method defaults work end-to-end; the compiled fixture covers Self acquisition and a real owned method consumer with fail-closed evidence validation in all four profiles; and the milestone text matches the shipped owned one-shot schema contract. The green create-PR gate (14 interop variants, 13 Arrow CPython tests, 131/131 E2E) is consistent with everything I reproduced — and, as in both prior passes, blind to the three findings above because no fixture exercises a protocol class-style call, an owned-affine constructor, or an owned-affine `__call__`.

## Bottom line

Pass 2's enumerated remediation is fully and verifiably done. What keeps M11 open is narrow and precisely localized: one new protocol-call regression inside `try_lower_class_method_call` (B-1, a one-arm fix), and the last two uncovered owned-transfer positions — constructors and callable objects (B-2, B-3) — which allow check-clean double-moves of the very resources this milestone introduces. Closing those three, with the fixtures named above, should make the next pass a clean SATISFIED.

VERDICT: NOT SATISFIED
