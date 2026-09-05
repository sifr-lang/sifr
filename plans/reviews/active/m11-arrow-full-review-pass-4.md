# M11 Arrow C Data Interface — full milestone review, pass 4 (PR #2991, `main...HEAD` at 554afe692)

Fresh full-milestone review: the complete `main...HEAD` diff (123 files, ~7,050 insertions over 4 commits), the M11 plan section and acceptance criteria, the project workflow, and all prior M11 review artifacts (passes 1, 2, 3b) were read. Three parallel read-only audits ran over (a) lowering/ownership call forms, (b) runtime/ABI/certification, and (c) docs/evidence/guardrails. Every blocker-class claim — prior or new — was independently reproduced or refuted live against a freshly built branch compiler, with a freshly built `main` baseline compiler used to classify pre-existing versus branch-introduced behavior. I ran the authoritative create-PR gate myself on this tree.

## Verdict summary

**NOT SATISFIED.** Every pass-3b finding is verifiably remediated for module-local code — all three blockers, both minors, and the durable-doc staleness — and the remediation commit introduced no false rejections I could find. But the owned-transfer family the last three passes have been closing is still open in three positions, each independently reproduced live: imported-class constructors silently drop `own` (cross-module double construction is check-clean and a guaranteed E0382 — and cross-module consumption is the milestone's primary delivery model), method overrides that add `own` are ignored at call sites (check-clean double move), and comprehension bodies never run the loop-move check (check-clean triple move, a pre-existing M10 Buffer hole inherited by all five Arrow kinds — the same standard pass 1 applied to B3). Each violates the acceptance item "Ownership transfers once and remains moved" and the check⇒compile contract that every prior pass enforced.

## Pass-3b disposition (every item re-verified live with the branch compiler)

| Pass-3b finding | Disposition |
|---|---|
| **B-1** `ProtocolName.method(...)` check-clean → rustc | **Closed.** `Duck.quack()` → SIFR-CLASS-0004 at check (reproduced live; same rejection as main). Enforcement: the `Type::Protocol` arm plus the `class_instance_methods` guard in `try_lower_class_method_call` (`crates/sifr_lowering/src/lower/expressions/method_argument_ownership.rs:121-138`); pinned by `class_style_calls_reject_protocol_and_instance_methods`. |
| **B-2** explicit `__init__` discards `own`; double construction check-clean | **Closed module-locally; reopened cross-module (NB-A below).** Live at HEAD: `Holder(v); Holder(v)` → OWN-0001; keyword form `Holder(value=v)` twice → OWN-0001; same-call `Pair(v, v)` → OWN-0001; implicit field-derived constructor `Wrap(v)` twice → OWN-0001; loop-body construction → OWN-0004. Constructor `FunctionType` now carries real conventions (`class_type_collection.rs:590-601`). Positive direction verified end-to-end: a package with explicit-`__init__`, `super().__init__`, pair, and implicit constructors plus a method consumer builds through rustc cleanly (single-use each). `super().__init__(value)` then reuse of `value` → OWN-0001 (live), and the emitter now evaluates `Parent::new(...)` in body order (`class_method_emitter.rs:278-312`). Pinned by `explicit_constructors_consume_owned_affine_arguments` and `super_constructor_consumes_owned_affine_arguments`. **But the identical shape through a module import is still open** — see NB-A. |
| **B-3** `__call__` objects never consume owned arguments | **Closed.** `sink(v); sink(v)` → OWN-0001; same-call `s(v, v)` → OWN-0001 (both live). Consumption wired at `regular_calls.rs:182`; single-use `__call__` builds through rustc. Adjacent callable shapes fail closed at check (verified live): attribute callables (`o.sink(v)` → SIFR-CALL-0005 "not callable"), subscript callables (`sinks[0](v)` → CALL-0005), method references (`f = h.consume` → CLASS-0004). Pinned by `callable_objects_consume_owned_affine_arguments`. |
| Minor: `super()` drops parent defaults | **Closed.** `super().greet()` with a defaulted parent parameter is check-clean (live; on main the same probe is rejected). Defaults keyed correctly per `__init__`/method (`method_argument_ownership.rs:64-81`); pinned by `super_calls_apply_defaults_and_reject_missing_methods`. |
| Minor: `super().nonexistent()` silently lowered | **Closed.** Now SIFR-CLASS-0004 "parent class 'Base' has no method 'pong'" (live). |
| Minor: `ClassName.instance_method(...)` class-style check-clean | **Partially closed.** Locally declared instance methods are rejected (live: `Holder.consume(v)` → CLASS-0004). Inherited and imported instance methods still slip through — see M-A. |
| Major: stale "Arrow … remain reserved" in the durable declaration doc | **Closed in that file** (`internal_docs/python_interop_declaration_architecture.md:345-346` now reserves only DLPack), **but the same staleness survives in a second public doc** — see M-D. |
| Minor: `method-transfer=len3` marker returns a constant | Unchanged (accepted in pass 3b; zero-leak accounting still covers cleanup). |
| Hygiene: dirty `third_party/ruff` submodule working tree | **Still present** (same formatting-only line-join in `parser/expression.rs`). The committed gitlink is unchanged in `main...HEAD` — nothing rides in the PR — but the local dirt must not be staged at merge. |

Earlier-pass closures (pass 1 B1–B7, pass 2 NB-1/NB-2/M-1–M-4) were re-verified at HEAD by a dedicated runtime/certification audit with file:line evidence: all six re-verification items hold — checked-alignment consumption readers with exact-once error-path release (`arrow_ops/abi.rs:209-278,344-355`, `arrow_ops.rs:66-101`), requested-schema one-shot removal-before-producer (`arrow_ops.rs:281-339`), measured (not asserted) certification evidence with `deny_unknown_fields`, target+kind binding, per-`(target, kind, producer_module, producer_type)` runtime admission, driver-enforced Self-method certification, and build-time distribution drift rejection (`arrow_evidence.py:74-135,251`, `arrow_certification.rs`, `python_interop.rs:82-136`, `package_python_certifications.rs:20-119`), certify/build digest parity through shared helpers (`python_cli.rs:125-161` vs `check_and_package_commands.rs:170-231`), lock/GIL discipline with no store lock across Python, and commit-before-call moves with prepare/finish/reconcile on both free-function and interop-method paths (`python_interop_direct.rs:196-210,611-622`).

## Blocking findings

### NB-A — Imported-class constructors silently drop `own`; cross-module double construction is check-clean and guaranteed E0382 (reproduced live)

```python
# holder.sifr
class Holder(NonSend):
    value: python.ArrowArray
    def __init__(self, own value: python.ArrowArray):
        self.value = value

# main.sifr
from holder import Holder

def run(own v: python.ArrowArray) -> None:
    a: Holder = Holder(v)
    b: Holder = Holder(v)   # check-clean!
```

`sifr check` → no errors; `sifr emit` → `Holder::new(v)` twice against `fn new(value: PythonArrowArray)` (by value) — guaranteed E0382. The implicit field-derived constructor has the identical hole (`class Wrap(NonSend): value: python.ArrowArray` imported and constructed twice — check-clean, emit shows two by-value `Wrap::new(v)`; both reproduced live). Root cause: all three import sites rebuild the constructor `FunctionType` by stripping conventions and calling `FunctionType::new` — `crates/sifr_lowering/src/lower/mod_impl.rs:693-705` (`.map(|(n, t, _)| (n.clone(), t.clone()))` then `FunctionType::new`, which assigns Borrow to every Move type, `sifr_type_system/src/types/definitions.rs:307-324`), likewise `mod_impl.rs:503-521` and `imports.rs:199-217`. So the call-site consumption loop sees Borrow and skips the argument, while codegen still emits the constructor parameter by value. This is precisely the pass-3b B-2 mechanism, fixed only in the defining module (`class_type_collection.rs:590-601`); exported *method* conventions do survive import (verified live: `s.consume(v); s.consume(v)` on an imported receiver → OWN-0001), so the gap is bounded to constructors. But "package authors declare once; consumers import and use" is the milestone's core objective — the single most idiomatic cross-module Arrow consumer shape double-moves check-cleanly. **Remediation:** preserve exported constructor conventions at all three import-rebuild sites (and force `own` for affine fields in the imported implicit-constructor fallback, mirroring `class_type_collection.rs:862-869`), with a cross-module double-construction test for Arrow and Buffer.

### NB-B — Method overrides that add `own` are ignored at call sites; check-clean double move (reproduced live)

```python
class Base(NonSend):
    def sink(self, a: python.ArrowArray) -> None: ...

class Child(Base):
    def sink(self, own a: python.ArrowArray) -> None: ...

def run(c: Child, own v: python.ArrowArray) -> None:
    c.sink(v)
    c.sink(v)   # check-clean!
```

`sifr check` → no errors; `sifr build` → E0382 `use of moved value: v` (reproduced live). Root cause: inherited methods are flattened into the child's method list *first* (`class_type_collection.rs:481-487`) and child overrides are appended without dedupe, while every signature lookup is first-match (`method_argument_ownership.rs:8-11`), so call sites use the parent's borrowed convention and never consume — but codegen emits the child's own method, which takes the parameter by value. Newly reachable in this PR: `own` on plain method parameters only became real via the pass-2 M-1 borrow-by-default alignment, so this is the NB-2 family ("method-call argument positions must consume owned affine arguments") in its override position. **Remediation:** make child overrides shadow inherited entries in the flattened method list (replace instead of append, or last-match lookup), with an override-adds-`own` double-move test.

### NB-C — Comprehension bodies skip the loop-move check; check-clean triple move (reproduced live; pre-existing for Buffer on main)

```python
def consume(own a: python.ArrowArray) -> int: ...

def run(own v: python.ArrowArray) -> None:
    xs: list[int] = [consume(v) for i in range(3)]   # check-clean!
```

`sifr check` → no errors; `sifr build` → E0382 (reproduced live at HEAD). `for`/`while` statements correctly reject the same shape with OWN-0004 (`control_flow.rs:663-685,803-876` — verified live), but comprehension and generator-expression lowering never runs the moved-across-loop snapshot check (`methods_lambdas_and_comprehensions.rs:529-842`, `generator_expression.rs:8-85`); the existing comprehension guards reject only affine *element/result* types, not a consumer call inside the body. The identical Buffer program is check-clean on the freshly built main baseline, so this is a pre-existing M10 hole inherited by all five Arrow kinds — exactly the situation pass 1 classified blocking for B3 ("pre-existing for `python.Buffer`, now inherited by all five Arrow kinds"), and it contradicts M10's claimed closure of iterator/generator affine paths. **Remediation:** run the loop-body moved-state check over comprehension/generator bodies (same snapshot mechanism as `for`), with Arrow and Buffer negative fixtures.

## Non-blocking findings

**Major**

- **M-A — The new class-style guard misses inherited methods and imported classes.** `SubSink.consume(v)` where `consume` is inherited is check-clean → rustc E0599 (reproduced live; the guard checks `"{Child}.{method}"` but only `"{Parent}.{method}"` was inserted into `class_instance_methods`, `class_type_collection.rs:636-641`). `ImportedClass.instance_method(v)` is likewise check-clean → receiverless `RemoteHolder::consume(v)` in emit (live; the set only records locally declared methods — admitted by the comment at `mod_context.rs:37-39`). Ownership is consumed exactly once and the failure is loud; the family is pre-existing on main (the Buffer variant is check-clean there too — verified live), which keeps this out of the blocker set by pass-3b's own precedent, but the guard shipped in this commit and should cover both shapes (insert inherited names during flattening; export/consult method kinds for imports, or reject class-style calls on imported classes conservatively).
- **M-B — Operator dunders never consume owned affine operands, and their emission is broken for Arrow operands.** `def __add__(self, own other: python.ArrowArray)` then `a + v` is check-clean and fails at rustc E0369 (`cannot add PythonArrowArray to &&Acc`) even on first use (reproduced live; the `int`-operand dunder builds and runs fine, so this is affine/by-value-specific). The dunder fallback in `expression_operators.rs:192-210` runs no consumption over either operand, so had the emission been coherent this would be a silent double move. Newly expressible: `own` dunder parameters became real via this PR's convention preservation (`parameter_conventions.rs:6-21`). Fail closed at check (reject `own` affine dunder params until the operator path consumes and emits correctly) or wire consumption plus correct receiver emission.
- **M-C — Value-position grandparent `super()` calls are newly check-clean and fail at rustc.** `return super().ping()` where `ping` is defined on the grandparent: on main this was (falsely) check-rejected because `SuperCall` was hard-typed `None`; the remediation gives it the real return type, so it now passes check — and codegen emits `B::ping(self)`, which only exists on `A` → E0599 (reproduced live on both compilers). The codegen defect is pre-existing (statement-position grandparent super fails identically on main — verified live), and the direct-parent case now works end-to-end (probe compiled and ran, printing the right value — a genuine improvement). But the commit widened the check-clean→rustc surface: resolve the defining ancestor when emitting `SuperCall` (or reject non-direct-parent super methods at check) with a pinning test.
- **M-D — Second stale "reserved for Arrow" doc line.** `docs/diagnostics/error-codes.mdx:133` still says PYZC "codes remain reserved for Arrow and DLPack declaration activation," contradicting the shipped active Arrow diagnostics and the two sibling docs corrected on this branch. Same class of staleness pass 3b flagged as major in the durable doc; one-line fix. (A full sweep of `docs/`, `internal_docs/`, and verification metadata found no other stale instance.)

**Minor**

- `@classmethod`/`@staticmethod` invoked through an instance (`t.show(1)`) is check-clean → rustc E0599 — identical on main (reproduced live on both); pre-existing, ownership still consumed once.
- Constructor emitter structural holes, all reproduced identically on main (pre-existing, unchanged by the rewrite): `super().__init__` inside an `if` → E0063 missing parent field; explicit `__init__` that never calls `super().__init__` → E0063; a derived class with *no* explicit `__init__` and a method-only parent gets no synthesized `new` at all → E0599 (`C()` on a 2- or 3-level chain fails at rustc while check-clean).
- Callable-type convention erasure is real at the type level (`annotations_and_function_lowering.rs:322-338` hard-codes Borrow; `type_rendering.rs:696-713` discards conventions on Function→Callable assignability; `contains_affine_resource` has no Callable arm) but currently has no silent path: Callable *local bindings* fail at rustc for any type (E0562 `impl Trait` binding, reproduced with `int` — pre-existing), and passing an `own`-param function into a Callable parameter and double-calling it fails loudly at rustc E0308 (reproduced live). Worth closing type-level (reject binding an `own`-affine-param function to a convention-less Callable) when Callables are next touched.
- `cls(...)` inside classmethods bypasses constructor arity/type validation (`core_and_calls.rs:364-383`); affine arguments are still consumed unconditionally, so the residual is a non-affine check→rustc divergence (static finding, pre-existing).
- `Type::Enum` carries no method signatures, so an enum method with an `own` affine parameter would skip consumption (`methods_lambdas_and_comprehensions.rs:82,163`) — contrived, statically identified.
- `python_interop_direct.rs:120,547`: `func.params.iter().find(|p| p.name == shape.name)?` silently aborts interop body emission on a name mismatch, falling through toward the empty-body panic instead of a diagnostic (defensive; not reachable with compiler-generated shapes today).
- The SuperCall type fix means statement-position `super().m()` returning `Result` now trips RESULT_UNUSED where it previously compiled — a defensible strictness change, worth a release note.
- Runtime hardening notes (audit-verified, none fail-open): producer admission identity is string-based (`__module__`/type-name spooagent within the already-trusted interpreter, `arrow_ops.rs:611-628`); `checked_ref` still returns an unconstrained lifetime (`abi.rs:344-355`, carried from pass 3b); surplus `unsafe impl Send` on the five ABI structs; certification-rollback release errors are swallowed (`stdlib arrow.rs:18` — error still propagates); a dead orphan window in `acquire_foreign_with_schema` if a second `PythonArrowSchema` construction path is ever added; two unreachable `continue`/skip arms inside the driver's fail-closed enforcement loop (`python_interop.rs:91-93,123-127`) that should be hard diagnostics; cert cache identity uses `unwrap_or_default()` (`python_runtime.rs:92-93`).
- The corrected durable-doc paragraph now reserves only DLPack but no longer affirmatively states Arrow is active (nit; the protocol-architecture doc carries the full active Arrow contract).
- File-size guardrail passes; several touched files remain 1–4 lines under the 900 cap (`core_and_calls.rs` 899, `lib_runtime_needs.rs` 898, `check_and_package_commands.rs` 898, `python_interop_direct.rs` 897, `class_body_lowering.rs` 896).

## What is verifiably solid at HEAD

The pass-3b remediation is real and complete for module-local code: all five blocker probes and both minor probes reproduce as fixed, the positive single-use direction builds through rustc (explicit/implicit/pair/keyword constructors, `super().__init__`, `__call__`, method consumers), and branch-exclusive moves are correctly accepted and compile. The negative matrix held under adversarial probing: sequential, same-call, keyword, loop, classmethod, `super()`-method, and imported-receiver double moves are all OWN-rejected at check; attribute/subscript callables and method references fail closed. The runtime/ABI/certification layer passed a fresh independent re-audit with zero blocker/major findings: exact-once release on every traced path, alignment-guarded consumption readers with sound error-path cleanup, one-shot requested-schema transfer with no double-free path, fully fail-closed measured certification (certify → artifact → build → runtime admission, with drift, mismatch, and containment rejection), digest parity, and clean GIL/lock discipline. Docs (except M-D), the capability matrix (arrow active with mutation-self-tested fail-closed validators), lane wiring (blocking in all four profiles), plan bookkeeping, and diff hygiene all check out.

## Validation performed

- Built the branch compiler (release) and a fresh `main` baseline compiler in a separate worktree; ran ~35 live probe programs through `check`/`emit`/`build`/`run` on both, including full rustc builds inside a trust-configured probe package against the pinned interop venv.
- Focused suites: `sifr_lowering` python_arrow 22/22, `sifr_codegen` python_arrow 9/9, `sifr_runtime --features python` arrow 13/13.
- Authoritative gate: `scripts/run_all_tests.sh --profile create-pr` → **pass** (exit 0), E2E 131/131 with signature `7c39b8c1dd4fec7c`, python_interop lane pass including `arrow-examples` (31.4s) and `arrow-cpython311`, 6 hardening variants with zero failures, all blocking budgets pass; sole advisory is the non-blocking warm wall-time notice (453.13s). As in every prior pass, the green gate is blind to the three blockers above: no fixture imports an Arrow-consuming class across modules, overrides a method to add `own`, or calls a consumer inside a comprehension.
- `python3 scripts/check_hir_maintainability_guardrails.py` → PASS; file-size guardrail clean over the touched set.
- Three parallel read-only audits (lowering call forms; runtime/ABI/certification; docs/evidence/guardrails); every blocker-class agent claim was independently reproduced or refuted live — notably, the agents' static claims that direct-parent `super()` method calls and local implicit constructors were broken were **refuted** by end-to-end runs, and their imported-constructor, override, comprehension, dunder, and class-style-guard claims were **confirmed**.

## Bottom line

The remediation genuinely closed everything pass 3b enumerated, and the runtime, certification, and evidence layers are in verifiably good shape. What keeps M11 open is one theme in three positions: parameter conventions — the carrier of the milestone's "ownership transfers once" guarantee — are still lost across the module-import boundary (NB-A), across override flattening (NB-B), and comprehension bodies never see the loop-move check (NB-C). All three admit narrow, well-localized fixes with the fixtures named above; closing them (plus the one-line M-D doc fix, and ideally M-A/M-B in the same sweep) should make the next pass a clean SATISFIED.

VERDICT: NOT SATISFIED
