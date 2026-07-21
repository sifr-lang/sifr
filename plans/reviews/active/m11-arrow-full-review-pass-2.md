All seven agents have reported and I've independently reproduced every blocker-class claim against the freshly built branch compiler (and disproved one agent claim end-to-end). Here is the consolidated pass-2 review.

---

# M11 Arrow C Data Interface — full milestone review, pass 2 (PR #2991, `main...HEAD` at e52c0699d)

## Verdict

**NOT SATISFIED.**

The remediation commit is real and substantial: B4, B5, B6 are verifiably fixed at the source level, B1's three reported shapes are now check-rejected, the certification chain now measures instead of asserts, and the M10 buffer surface is byte-for-byte unregressed. But the owned-consumer **method** surface — the exact shape pass-1 B2 demanded — went from "un-declarable" to "check-clean, then compiler panic," and method-call argument positions still never consume owned affine arguments at all, so check-clean programs fail in rustc. Both violate "if it compiles, it works" and the milestone's own acceptance items, and both are invisible to the passing validation gates because no fixture exercises a method-shaped Arrow consumer.

## Blocking findings

### NB-1 — `@python(Self.m)` with `own value: python.Arrow*` is check-clean and ICEs in codegen (reproduced personally, twice)

```python
@python.opaque(type=pyarrow.Array, cleanup=drop)
class Sink(NonSend):
    @python(Self.push)
    def push(self, own value: python.ArrowArray) -> Result[None, PythonError]: ...
```

`sifr check` → no errors. `sifr emit` → panic:

```
panicked at crates/sifr_codegen/src/class_method_emitter.rs:709:17:
class method IR lowering produced empty body for non-unit return: Sink::push
error[SIFR-INTERNAL-0001]: internal compiler panic during single-file code generation
```

The panic fires even if the method is never called, and reproduces for `ArrowSchema` and `ArrowStream` (mechanism is kind-independent). Root cause: the method-body builder's parameter loop uses `input_conversion(...)?`, which has no `Type::PythonArrow` arm (`crates/sifr_codegen/src/python_interop_direct.rs:636-644`; conversions file has zero Arrow hits), so the builder returns `None` and the empty-body panic at `class_method_emitter.rs:705-713` triggers. The free-function loop has the Arrow path (`python_interop_direct.rs:196-208` → `append_argument_preparation`); the method loop was never given one.

This is a regression created by e52c0699d itself: the new convention arm (`parameter_conventions.rs:12-15`) preserves `own` for Arrow method params, and the new lowering test `class_methods_preserve_owned_arrow_parameters` (`python_arrow_contract_tests.rs:221-229`) pins the shape as *accepted* — but no codegen path or codegen test was added (`python_arrow_codegen_tests.rs` covers only free-function consumers and `@python.arrow(Self)` receivers). Worse, the compiler steers users into the panic: writing the borrowed form produces `must transfer ownership with 'own'` (verified live), and following that advice ICEs. Buffers do not share this hole — `own view: python.Buffer[uint8]` on a method fails closed at check with SIFR-PYCONV-0001 (verified live) — so this is Arrow-specific and new in M11. Remediation: add the Arrow argument-preparation path to `python_interop_method_body_with_retained_errors` (mirroring lines 196-208) plus a codegen test, or fail closed at check.

### NB-2 — Method-call argument positions never consume owned affine arguments; check-clean programs fail at rustc E0382 (reproduced personally)

```python
class Holder:
    def consume(self, own a: python.ArrowArray) -> Result[None, PythonError]:
        ...  # body releases a — correctly checked

def main() -> ...:
    v: python.ArrowArray = make([1, 2, 3])
    r1: None = h.consume(v)
    r2: None = h.consume(v)   # check-clean!
```

`sifr check` → no errors. Generated Rust (verified in emit output): `fn consume(&self, a: ::sifr_stdlib::python::PythonArrowArray)` called twice as `h.consume(v)` — a by-value double move, guaranteed E0382. Same-call `h.consume2(v, v)`, `@staticmethod` `Tools.consume2(v, v)`, and `super()` calls are equally check-clean (probe-verified by the audit agent). Root cause: `lower_method_call` (`crates/sifr_lowering/src/lower/expressions/methods_lambdas_and_comprehensions.rs:27-334`) applies no `consume_owned_value` over method arguments; the only owned-argument consumption loops are in `regular_calls.rs:141-147` and `:458-472` (function calls). This gap is newly *reachable* at HEAD — before e52c0699d, methods demoted `own` to borrow so no by-value move was ever emitted — and it applies to all five Arrow kinds plus `python.Buffer`. It directly violates the M11 acceptance item "Ownership transfers once and remains moved," and check⇒compile. Remediation: run the owned-argument consumption pass over instance/static/super method arguments exactly as `regular_calls.rs:458-472` does, with tests for sequential, same-call, keyword, and Buffer variants.

Everything else from pass 1 is closed (details below). No other check⇒rustc divergence was confirmed: the agent-reported "missing hidden `__sifr_python_error` field" claim does **not** reproduce end-to-end — I re-ran the declaration-free Arrow-method module through the real pipeline and the field is emitted (the canonical `_sifr.python.PythonError` identity short-circuits at `class_field_emitter.rs:8-10`; the agent's synthetic probe module lacked that identity).

## B1–B7 disposition table

| Pass-1 finding | Disposition | Evidence |
|---|---|---|
| **B1** check-clean ICEs (omit / coroutine / `mut own` method) | **Verified fixed** for all three reported shapes — each now check-rejected with PYZC-0001 (reproduced live); enforcement in `direct_validation.rs:55-84`, reached for both functions and methods; test at `python_arrow_contract_tests.rs:205-219`. **But the family is not closed** — see NB-1 (new check-clean ICE at the same panic site). Containers/callables/context/lambdas/keyword-default shapes audited safe. |
| **B2** class-method `own` downgrade | **Partially fixed / regressed.** Convention preserved (`parameter_conventions.rs:11-26`), `borrowed_params` populated (`:41-57`, wired at `class_body_lowering.rs:547-573`), body-side release-then-use now check-caught (SIFR-OWN-0001, reproduced live), plain-method owned params emit by value. But the interop-method consumer shape now ICEs (NB-1), and the fix ships a silent behavior change to non-affine methods (major M-1 below). |
| **B3** same-call double consumption | **Partially fixed.** Function calls fully closed: `consume2(v, v)`, `consume2(v, take(v))`, reverse, keyword, list/tuple literals, Buffer — all OWN-0001 at check (I reproduced four of these live; enforcement `core_and_calls.rs:563-608`). **Still open for method-call argument positions** — see NB-2. |
| **B4** certification not proving/binding/enforcing | **Verified fixed** (all five sub-items). Fixture measures pyarrow `Buffer.address` vs exported `buffers[i]` via ctypes and instruments the release callback (`arrow_evidence.py:74-135`); `copy_performed` is derived, not literal. Artifact schema v2 adds `kind` + `identity_method` with `deny_unknown_fields` (`arrow_certification.rs:20-41`); fixture echoes target+kind and the CLI rejects mismatches (`python_cli.rs:363-379`). Runtime admission is per `(target, kind, producer_module, producer_type)` (`arrow_ops.rs:193-211`), cross-target and cross-kind rejection tested. Self methods are collected into the plan keyed by the opaque class target (`python_interop_plan.rs:201-227`) and enforced fail-closed by the driver (`python_interop.rs:82-135`, tests at :494-541). Containment validated before fixture execution (`python_cli.rs:175` → `:241-281`). |
| **B5** certify/build digest divergence | **Verified fixed.** Certify now derives declaration+bridge requirements through the same helpers as the build (`python_cli.rs:125-161` vs `check_and_package_commands.rs:158-231`); digest inputs are canonically sorted (`requirements.rs:38-78`). The arrow-examples lane no longer masks: `explicit_requirements=False` (`arrow_examples.py:26`), so certify→build parity is proven end-to-end on declaration-derived-only roots. Residuals are minor (below). |
| **B6** requested-schema borrow broken by consuming producers | **Verified fixed**, and the milestone text was updated to the owned one-shot contract. Type level: schema parameter must be required keyword-only plain `own` (reproduced live: borrowed, positional, and cross-call-reuse all rejected). Runtime: store entry removed *before* the producer runs (`arrow_ops.rs:281,318-339`), handle dead regardless of consumption; Sifr never installs a competing destructor, so consumed→no-op / unconsumed→exact-once with no double-free path. Real-producer coverage: `real_pyarrow_requested_schema_is_a_one_shot_transfer` mandatory in the exact-set CPython 3.11 lane, plus the compiled example exercises `schema=parameter` with a real `pyarrow.int64` schema. |
| **B7** evidence/ledger gaps | **Largely fixed.** Capability matrix arrow row is `active` with all evidence `passing` and real lanes behind it, blocking in all four profiles. Bridge/Self/import-root now have lowering+codegen tests (bridge went from zero to two). Compiled example covers owned transfer, rollback with zero-leak accounting, schema kind, `schema=parameter`, and all three producers (pyarrow/pandas/polars) with exact stdout marker. Evidence validator does deep structural cross-checks with four mutation self-tests (`run.py:809-834`). Codegen matrix covers all five kinds, multi-owned ordering, keyword-only owned args. Residuals: one stale public-docs sentence (M-2), no *compiled* bridge/Self fixtures, device kinds synthetic-only (honestly labeled). |

## Non-blocking findings

**Major**

- **M-1: silent language regression on plain class methods.** `def echo(self, s: str) -> str: return s` compiled and ran before this branch; at HEAD it is check-rejected with SIFR-OWN-0003 (reproduced live). This is a side effect of the pass-1-requested `borrowed_params` population (`parameter_conventions.rs:41-57`) and aligns methods with free functions — defensibly a consistency fix, and it fails closed with an actionable diagnostic — but it is an undocumented, untested breaking change to non-Arrow code shipped inside an M11 PR. It needs an explicit intentionality ruling, a pinning test, and a doc/changelog note before merge.
- **M-2: public docs contradict the compiler on the schema parameter.** `docs/python-interop.mdx:361` says `schema=parameter(name)` names a "required keyword-only **borrowed** `python.ArrowSchema` parameter"; the compiler rejects the borrowed form and requires plain `own` (verified live). The internal docs were fixed in e52c0699d; the public page was not.
- **M-3: consumption-state readers deref capsule payloads unchecked.** The new `checked_ref` alignment guard (`abi.rs:326-337`) covers acquisition, but the five `*_consumption` readers (`abi.rs:211-256`) re-fetch the pointer at `finish()` time and deref it raw — after the capsule has been handed to arbitrary consumer Python, which can `PyCapsule_SetPointer` to a misaligned address → Rust UB. Within the `[trust].python` envelope, but it is exactly the pattern pass-1 flagged; route them through `checked_ref`.
- **M-4: two fail-closed enforcement points have zero test coverage.** Build-time distribution re-verification (`package_python_certifications.rs:46-78`) and the kwonly-defaults fix (`class_body_lowering.rs:433-466`, `class_type_collection.rs:707-724` — verified working empirically) both landed untested.

**Minor**

- Requested-schema test gaps: no producer-cannot-satisfy mismatch fixture (only schema-*mode* mismatch exists), no producer-raises-with-schema-in-play test, no exact-once release counter for a non-consuming producer's schema.
- No compiled `.sifr` fixture for bridge-producer or Self-receiver Arrow acquisition (buffer/M10 shipped both); coverage is compiler-level tests only.
- B5 residuals: an entry file outside the package source roots can still produce a certify/build digest split (`Some(file)` vs `None` into `declaration_python_requirements`); certify hardcodes `CargoLockMode::Normal`; no unit test pins request parity between the two duplicated call sites.
- B4 residuals: one-certification-per-target means a class cannot certify two different Arrow kinds across its Self methods; stream no-copy identity compares a single buffer address; release count == 1 is largely by construction; a dishonest package fixture can still self-attest (inherent trust boundary, runner tripwires guard only the reference fixture).
- B3 test gaps: reverse-order, keyword-arg, and Buffer same-call double-move variants are enforced but untested.
- Sifr generator functions with owned Arrow params are check- and codegen-clean but rustc-unproven — add a fixture.
- Unchanged pass-1 nits: dead `foreign` flag and unreachable `acquire`/`acquire_with_schema` codegen arms (`python_arrow_codegen.rs:57-87`) with the corresponding stdlib methods still untested public API; `update_object_count` under the `ARROW_STORE` guard (I note the lock order was verified globally consistent — no deadlock); artifact write is plain `fs::write` (not temp+rename); no `--message-format json` on certify success paths; arrow-examples lane timeout budget; `unsafe impl Send` on ABI structs without safety comments; device-stream per-chunk `sync_event` is structurally unvalidatable at acquisition (worth a spec-boundary comment).

## What is verifiably solid at HEAD

The runtime capsule layer passed a fresh adversarial audit: exact-once release on every traced path (mid-pair acquisition failure, partial consumption with 2/4/6 counter tests, certification rollback, double-release token checks, GC-vs-store ordering), no data-dependent panics, clean GIL discipline with no Sifr lock held across Python, byte-exact capsule names, phase-unambiguous `release==NULL` semantics, alignment + device-metadata + device-type-allowlist validation at acquisition, `Drop` reconciliation on `PythonArrowArgument`, and test-store draining. The M10 buffer surface is verifiably unchanged (`buffer_ops` untouched; the `python.rs` shrink is a verbatim test-module move; diagnostics are wording-only with codes preserved). Certification is fail-closed end-to-end — measured no-copy evidence, target+kind binding at certify, build, and runtime, digest parity proven e2e by the un-masked example lane, and read-only `--check` re-execution.

## Acceptance-criteria assessment

- **"Arrow declarations have no copy switch and never certify uncertain copying"** — met. No copy parameter exists; evidence is measured (buffer-address identity, instrumented release), bound to target+kind+distribution+digest, and enforced fail-closed at certify, build, and acquisition.
- **"Ownership transfers once and remains moved even if a consumer later fails"** — met for free-function consumers (commit-before-call at `prepare_arrow_argument`, moved-remains-moved verified; failure reconciliation tested). **Violated for method consumers**: NB-1 makes the interop-method consumer un-emittable, NB-2 lets plain-method consumers double-move check-cleanly.
- **"Unconsumed resources release exactly once"** — met at the runtime layer across all traced exits.
- **Validation matrices** — acquisition/transfer, malformed-capsule, partial/double-consumption, use-after-move (function paths), requested-schema shapes, and pandas/PyArrow/Polars compiled pointer/release evidence are all present; the gaps are the method-shape fixtures (which is exactly where both blockers hide), the unsatisfiable-schema mismatch fixture, and compiled bridge/Self fixtures.

## Note on the passed validation gates

The green create-PR run (131/131 E2E, both Arrow lanes, exact-set CPython suites) is genuine but blind to both blockers: I confirmed by grep that no fixture anywhere passes an owned Arrow value to a class method, in either the interop (`@python(Self.m)`) or plain-method form. Closing NB-1/NB-2 requires adding precisely those fixtures.

**Bottom line:** M11 is not closure-ready. Two blockers remain, both in the method-consumer surface: add the Arrow argument path to the interop method-body builder (or fail closed at check) with a codegen test, and make method-call lowering consume owned affine arguments like function calls do, with sequential/same-call/Buffer tests. B4–B7 are closed; B1's reported shapes are closed; the remaining work is narrow and well-localized.
