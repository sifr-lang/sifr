## Rust Interop Certification 5 — Full-Diff Implementation Review

Scope: current working tree (`agent/rust-interop-certification-5`), excluding the `editor_integrations` submodule bump and `.cert5probe/` scratch files. Independently rebuilt the compiler from the working tree (`target/debug/sifr`) and exercised generated Rust through a throwaway out-of-tree driver over `sifr_python_parser` → `sifr_lowering` → `sifr_codegen::generate_rust_with_metadata`. No repository files were modified.

### What is genuinely fixed

**Round-5 finding 1 (unconditional receiver insertion) — argument side fixed.** `rust_interop_method_body` now takes `include_receiver` (`crates/sifr_codegen/src/rust_interop_direct.rs:75-92`), gated at the call site on the class carrying `@rust.opaque` (`crates/sifr_codegen/src/class_method_emitter.rs:701-707`). Verified by emission: a non-opaque class method bound to a free bridge function emits `bridge::things::consume()` with no injected receiver, and the guard test `non_opaque_rust_bound_method_does_not_inject_receiver_argument` (`crates/sifr_codegen/src/rust_interop_direct_tests.rs:81-110`) pins it.

**Round-5 finding 2 (name-based ownership inference) — declaration side fixed.** Receiver ownership is now real source metadata (`RustInteropDeclaration::consumes_receiver`, `crates/sifr_ir/src/rust_interop.rs:80`), set from `own self` at `crates/sifr_lowering/src/lower/classes/class_body_lowering.rs:499-503`, and the `close=` policy selects the sole consuming member (`sifr_ir::rust_opaque_close_method`, `crates/sifr_ir/src/rust_interop.rs:84-99`). `validate_rust_opaque_close_method` (`crates/sifr_lowering/src/lower/classes/rust_opaque_validation.rs:3-72`) rejects mismatched and borrowed close members before rustc, and the driver contract additionally requires `consumes_receiver` (`crates/sifr_driver/src/build/rust_interop/opaque_validation.rs:55`). Double close is now a lowering diagnostic via `rust_consuming_methods` (`crates/sifr_lowering/src/lower/rust_interop.rs:26-62`, consumed at `crates/sifr_lowering/src/lower/expressions/methods_lambdas_and_comprehensions.rs:226-228`). Reproduced directly:

- `close=async_close` + `def close(own self)` → `SIFR-RUST-CONFIG-0001` "reserved for the member selected by the class close policy".
- Two `await resource.aclose()` calls → `SIFR-OWN-0001` "use of moved value: 'resource'".

The bridge contract now carries the owning receiver (`crates/sifr_codegen/src/rust_interop_bridge_contract.rs:172-190`) with a matching plan test at `crates/sifr_codegen/src/rust_interop_plan_tests.rs:289-345`, and the emitted trait/impl routes `bridge::resources::aclose(self).await` on `Handle<T>`. The file decomposition (`class_semantics.rs`, `rust_opaque_validation.rs`, `rust_interop_opaque_contract_tests.rs`, `rust_interop_test_support.rs`) is pure relocation plus new code, responsibility-based, and every touched file is under the 900-line cap (largest: `_scenario_checks.py` 898, `rust_interop_tests.rs` 897).

`cargo test -p sifr_codegen -p sifr_lowering` passes (914 + 834).

### Blocking findings

**1. `cargo test -p sifr_driver --lib` fails on the current tree.**

```
build::sysroot_interop::tests::sysroot_private_opaque_interop_resolves_self_close_method
panicked at crates/sifr_driver/src/build/sysroot_interop.rs:377
  SIFR-RUST-HANDLE-0001: opaque Rust handle `_sifr.io.FileHandle` requires `close` cleanup method
test result: FAILED. 412 passed; 1 failed; 50 ignored
```

The new `candidate.declaration.consumes_receiver` requirement (`crates/sifr_driver/src/build/rust_interop/opaque_validation.rs:55`) is not satisfied by the test's own `method_declaration` helper, which the diff left at `consumes_receiver: false` (`crates/sifr_driver/src/build/sysroot_interop.rs:632`), and the test's sysroot source declares `def close(self) -> None` (`sysroot_interop.rs:350-352`) — borrowed receiver, non-`Result` return — which the new policy also rejects. The reported validation set never ran this suite, so this went unnoticed. The tree is not green.

**2. Regression: a non-opaque class method with `own self` now emits a by-value receiver with no move tracking, producing raw rustc errors.**

`crates/sifr_codegen/src/class_method_emitter.rs:654-660` pushes `RustParam::SelfValue` whenever `method.rust_interop.first().consumes_receiver` — with **no** opaque-class gate, unlike the body side at line 701-707. `consumes_receiver` is set for *every* regular method with `own self` (`class_body_lowering.rs:499-503`), while `rust_consuming_methods` (the move-tracking set) is populated only for `@rust.opaque` classes with a `close=` policy, and `validate_rust_opaque_close_method` returns early for non-opaque classes (`rust_opaque_validation.rs:11-14`).

Observed, single call on a borrowed binding:

```python
class Plain:
    value: int
    @rust(bridge.things.consume, panic=trusted_no_panic)
    def consume(own self) -> Result[str, ResourceError]: ...

def use_it(p: Plain) -> Result[str, ResourceError]:
    try: return p.consume()
    except ResourceError as error: raise error
```

`sifr check` → *no errors found*. Generated Rust → `fn consume(self)` called as `p.consume()` with `p: &Plain` → rustc `E0507`. Two calls on a local → `E0382`. Before this diff the same source lowered to `fn consume(&self)` and compiled. Note the by-value receiver is also pointless here: `include_receiver` is false, so `self` is never forwarded to the bridge. There is no guard test for this path — `non_opaque_rust_bound_method_does_not_inject_receiver_argument` uses `consumes_receiver: false` and covers only argument injection. This is the same defect class round 5 flagged as blocking finding 1, relocated from the argument list to the parameter list. It also directly falsifies the tracker claim "preserves ordinary non-opaque bridge-method call shapes" (`plans/issues/active/rust-interop-runtime-ecosystem-certification.md`, certification_5 focused evidence).

**3. Call-site ownership for the owned close member is still not enforced before rustc.**

```python
async def close_it(resource: ResourceMatrix) -> Result[None, ResourceError]:
    try:
        _c: None = await resource.aclose()
        ...
```

`sifr check` → *no errors found*. Generated: `async fn close_it(resource: &ResourceMatrix)` calling `resource.aclose()` where `aclose(self)` consumes → rustc `E0507`. The same shape with a `Self.close` target reproduces it for `close=close`. `validate_rust_opaque_close_method` checks only the *declaration*; nothing requires the receiver expression at a close call site to be owned. The internal architecture text added by this diff (`internal_docs/rust_interop_architecture.md:208-216`) asserts "mismatched members, borrowed close receivers, duplicate close calls, and use after close are rejected during lowering or package-contract validation before rustc" — that holds for declarations and for double-close on a local, but not for closing through a borrowed binding, which is the ordinary library shape used by the already-certified `close_after_use` fixture (`verification/areas/rust_interop/fixtures/close_after_use/positive/closed_handle_error_surface.sifr:15-17`, which emits `fn verify_closed_handle_error_surface(resource: &NativeHandle)` calling a consuming `close`). Round-5 finding 2's core complaint — mismatched cleanup reaching a raw rustc ownership error — remains reachable.

### Additional findings

**4. An existing "passing" positive fixture is now rejected by the compiler.** `verification/areas/rust_interop/fixtures/async_runtime_core/positive/stdlib_async_resource_lifecycle.sifr:11-12` declares `async def aclose(self)` under `close=async_close`; the new rule requires `own self`, so `sifr check` on it now emits `SIFR-RUST-CONFIG-0001: close=async_close requires exactly one Rust-bound async def aclose(own self) -> Result[None, Error] method`. Its `fixture.json` still says `expected_result: pass`, `status: passing`. It escapes detection only because its binding is a synthesized-plan driver test that never lowers the `.sifr` (`async_runtime_core/fixture.json`, `package_rust_interop_opaque_current_thread_clears_async_method_send_probe`). The fixture already contradicted the documented rule, but shipping enforcement without repairing the fixture leaves a stale "passing" positive that the compiler rejects.

**5. Opaque classes' non-close Rust-bound methods are silently unemitted while the new contract now demands a receiver for them.** `emit_opaque_rust_method_trait` filters to the selected close member only (`crates/sifr_codegen/src/class_emitter.rs:777-786`), yet `signature_contract` synthesizes a `Borrow` receiver for *every* regular method of an opaque class (`rust_interop_bridge_contract.rs:172-190`). Reproduced: an opaque class with `@rust(bridge.resources.ping) def ping(self)` passes `sifr check`, emits no `ping` at all, and the call site emits `resource.ping()?` → rustc `E0599`. The certified `opaque_handle_tokenizer` row's `encode` member (`.../positive/declared_send_sync_copy_handle.sifr:12`) is exactly this shape, and its contract signature silently changed shape with this diff. The dropped-method half is pre-existing; the contract/codegen divergence is new.

**6. Cross-module use of the new extension trait looks unreachable.** The trait is emitted with `Visibility::Private` (`class_emitter.rs:823-828`) and `render_local_module_imports` only imports source-level names (`crates/sifr_codegen/src/lib_project_codegen.rs:8-30`), so a consumer module importing an opaque class would not have `__SifrOpaque<Class>Methods` in scope for `handle.aclose()`. The certification fixture is single-module, so nothing exercises this. Reported from code reading, not executed — it needs either a guard test or an explicit scope statement.

### Evidence, metadata, and docs

The runtime evidence itself reads honest and matches round 5's independent execution: `bridges/resources.rs:519-532` implements `aclose(mut resource: Handle<ResourceMatrix>)` as a genuine owned close routed from the generated member, `close_observation` reports `closed/already-closed`, `run(&Handle<…>)` is borrowed, and `invalid_aliasing(mut resource: Handle<…>)` operates all four resources before observing the bridge-local shared-alias rejection. Matrix promotion is coherent across `rust_interop_fixture_matrix.json`, `rust_interop_compatibility_matrix.json`, `stable_support_claims.json` (deferral removed), `fixture.json` bindings to the two ignored generated-build tests, and both `docs/rust-interop.mdx` and `internal_docs/rust_interop_architecture.md` disclose the restrictions (bridge-local alias identity, no Sifr clone policy, `CLIENT SETINFO` disabled, client-internal tasks excluded, no general server compliance). The new `_rust_test_outcomes.py` process-derived-state rule plus its three self-test controls (including the "state literal does not count" mutation) is a real strengthening, and the `contains_empty_pass_body` exemption is now narrowed with a nested-method and pass-plus-field mutation (`check_fixture_matrix.py:339-372`), addressing round 3. Round 4's two findings are visibly closed: the owned `close=async_close` member exists and is proven to route to the package bridge, and the README wording is narrowed to the handle.

The tracker's own checklist still has "Run focused and authoritative local gates … merge the PR" unchecked — correctly, given finding 1.

### Assessment

The two round-5 blockers are each *partially* fixed. Receiver injection is now opaque-gated on the argument side but not on the parameter side, and ownership is now policy-derived at the declaration but still unenforced at close call sites — both still terminate in raw rustc `E0507`/`E0382` after a clean `sifr check`, which is the exact guarantee certification 5 is supposed to establish. On top of that, the tree does not pass `cargo test -p sifr_driver --lib`, so it is not ready for full gates or a PR.

VERDICT: NOT SATISFIED
