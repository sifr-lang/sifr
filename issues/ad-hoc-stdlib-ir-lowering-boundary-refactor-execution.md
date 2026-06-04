# Ad Hoc Phase Execution Checklist: Stdlib, IR, and Lowering Boundary Refactor

Phase contract: [ad-hoc-stdlib-ir-lowering-boundary-refactor.md](./ad-hoc-stdlib-ir-lowering-boundary-refactor.md)

Status: planned

## Checklist

- [ ] `milestone_stdlib_boundary_1`: Create `sifr_stdlib` Contract Crate
- [ ] `milestone_stdlib_boundary_2`: Centralize Stdlib Feature And Dependency Manifest
- [ ] `milestone_ir_boundary_1`: Extract `sifr_ir` Data Crate
- [ ] `milestone_ir_boundary_2`: Rename Remainder To `sifr_lowering`
- [ ] `milestone_ir_boundary_3`: Dependency Direction Guardrails
- [ ] `milestone_ir_boundary_4`: Documentation And Phase Closeout

## Review Artifacts

Record planning and implementation reviews here.

- Initial planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-1.md` -> `CHANGES_REQUESTED`; addressed direct-vs-transitive lint dependency wording, added `sifr_stdlib` dependency validation, narrowed the driver stdlib-bootstrap exception, made binary-size validation unconditional, added `Cargo.lock` to stale-name sweeps, and added intrinsic signature/codegen/feature parity checks.
- Follow-up planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-2.md` -> `CHANGES_REQUESTED`; aligned `sifr_stdlib` locked dependency rules with the guardrail forbidden set and added a direct-lowering dependency guard for `sifr_analysis`.
- Final planning review: `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-3.md` -> `READY`; reviewer confirmed the contract is implementation-ready with precise crate ownership, acyclic dependency direction, enumerable exit gates, and milestone validation coverage.

## Validation Ledger

Record local validation for each milestone before opening the corresponding PR.

- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.
- M5: pending.
- M6: pending.

## Merged PRs

Record merged PR links here as each milestone lands.

- M1: pending.
- M2: pending.
- M3: pending.
- M4: pending.
- M5: pending.
- M6: pending.
