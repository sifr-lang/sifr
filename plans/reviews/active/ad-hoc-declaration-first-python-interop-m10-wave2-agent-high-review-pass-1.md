# M10 Wave 2 review — pass 1

- Date: 2026-07-15
- Pull request: [#2988](https://github.com/sifr-lang/sifr/pull/2988)
- Reviewer: agent
- Reasoning/service tier: `high` / `fast`
- Scope: complete `main...HEAD` diff
- Verdict: changes required

## Findings

1. **High — `Self` buffer declarations emit Rust that cannot type-check.**
   `python_buffer_codegen.rs` passes `&self.__sifr_python_object` to `PythonBuffer::acquire`. That field is a runtime `ObjectHandle`, aliasing `ForeignObject`, while `acquire` requires `&PythonObject`, which is `&Handle<ForeignObject>`. A valid opaque class containing `@python.buffer(Self, ...)` therefore reaches rustc and fails with a mismatched-type error. The current test only inspects text and Rust syntax. Add an acquisition helper accepting `&ForeignObject`, or explicitly construct the expected sealed handle without changing ownership semantics, then add a compiled receiver fixture.

2. **High — `python.Buffer[T]` is not integrated into clone/equality and aggregate capability analysis.**
   Same-type equality is accepted unconditionally. Thus comparing two buffers passes lowering but emits `left.clone() == right.clone()`, which cannot compile because `PythonBuffer` intentionally lacks `PartialEq`. Likewise, a class containing a buffer field receives `Clone, PartialEq` derives because the affine-field check only recognizes `NonSend` classes, while `PythonBuffer` is intentionally non-`Clone`. Define recursive clone/equality/affine capability queries, reject buffer equality, and suppress incompatible derives for aggregates containing buffers. Add rustc-backed tests for direct comparison and buffer-containing records/options/collections.

3. **Medium — the claimed permanent compiled smoke is not owned by any runner.**
   `buffer_declaration_codegen_smoke.sifr` is not referenced anywhere outside itself. It is absent from the source inventory and executable dataframe cases. Consequently, the reported manual smoke can regress while all authoritative gates remain green. Register it as an executable case asserting output and zero outstanding resources; add compiled `Self` and bridge acquisition cases as well.

4. **Medium — activation is not atomic across compiler, durable status, and capability governance.**
   Lowering publicly activates buffer declarations and the diagnostic is active, but the capability ledger still says `reserved` with all evidence planned. The durable declaration, protocol, and runtime documents also say buffers remain reserved. This contradicts Wave 2's atomic public activation claim. Either keep the decorator hard-gated until Wave 3, or activate the ledger, durable status, and required executable evidence in this merge unit.

## Resolution status

- [x] Corrected `Self` acquisition through the sealed `ForeignObject` path and
  added rustc-backed receiver coverage.
- [x] Integrated buffers into cycle-safe recursive clone, equality, and affine
  capability analysis, with direct-comparison rejection and nested aggregate
  code-generation coverage.
- [x] Registered executable top-level, `Self`, package-bridge, and affine
  aggregate fixtures with output and zero-live-resource assertions.
- [x] Made activation status atomic across the durable architecture documents,
  declaration capability ledger, and permanent verification profiles.
- [x] Authoritative `create-pr` validation passed on 2026-07-15: Python interop
  `11/11` including buffer examples `4/4`, crate tests within the enforced
  600-second budget, runtime-platform variants `28/28`, and E2E `131/131` with
  signature `7c39b8c1dd4fec7c`.
- [ ] Run a fresh full-diff review and record the satisfied verdict.
