## M39.10 Review Round 2

All round-1 fixes are implemented and verified. No blockers or regressions found.

### Round-1 fix verification

| Fix | Status | Where |
|---|---|---|
| `IntegerList` HIR variant + serialization (`int-list:2,3`) | ✓ | `sifr_ir/src/rust_interop.rs:61`, `sifr_codegen/src/rust_interop_plan.rs:575-583` |
| List/Tuple lowering with negative literal support | ✓ | `sifr_lowering/src/lower/rust_interop.rs:237-238,299-341` |
| Tensor/DLPack require dtype/shape/layout/strides/device | ✓ | `advanced_data_validation.rs:288-298` |
| Shape non-negative; shape/strides length match; rank matches | ✓ | `advanced_data_validation.rs:318-334,379-394` |
| DLPack requires `ownership=transfer` + `protocol=` + owned owner | ✓ | `advanced_data_validation.rs:122-143,303-312` |
| bf16 dtype | ✓ | `advanced_data_validation.rs:374` |
| 24-test advanced data contract coverage | ✓ | `rust_interop_advanced_data_contract_tests.rs` |
| Fixture matrix `scope=contract-only`, READMEs/docs call out staged runtime | ✓ | Matrix JSON + READMEs + `rust_interop_architecture.md:670-679` |
| Inline plan tests moved out for 900-line guardrail | ✓ | `rust_interop_plan_tests.rs` |

### Code-flow soundness checks

- `validate_advanced_data_contracts` runs after `validate_zero_copy_contracts` and after `signature_contracts` is populated (`rust_interop.rs:125-142`), so basic view fields are present and signature lookups work for the DLPack owned-owner check.
- `is_advanced_view_key` is `pub(super)` and reachable from sibling `zero_copy_validation` via `super::advanced_data_validation::…` (`zero_copy_validation.rs:280`); confirmed by passing test suite.
- `collect_value_paths` now matches `IntegerList(_)` as a no-op — no risk of dangling target-path collection (`rust_interop.rs:712-721`).
- `parse_advanced_data_contract` only triggers validation when at least one advanced key is present (`saw_advanced_key`), preserving backward compatibility for plain views.
- `validate_contract_shape` cleanly rejects cross-domain keys (arrow + tensor metadata, tensor + schema, non-DLPack + protocol) and DLPack transfer/owner gaps.

### Non-blocker observations (future polish, not gating)

1. `shape_value`/`strides_value` error reads "must be a non-empty integer list" even when the value isn't a list at all; the "non-empty" framing is slightly misleading since lowering already rejects empty lists. Pure messaging nit.
2. `validate_signature_ownership` clones the entire `RustBridgeSignatureContract` to dodge the borrow checker; a small scope would suffice. Cosmetic.
3. View-only declarations (no paired `@rust(...)`) silently skip both shared-bridge-root and signature-ownership checks; this matches the existing zero-copy validator's contract and is consistent with the staged certification scope.
4. The validator only enforces owned-owner-on-transfer for `data=dlpack`; tensor + `ownership=transfer` with a borrowed owner is accepted silently. Matches the stated round-1 spec but worth confirming intent before runtime certification.

### Verdict

No blockers remain. Another review round is not required — proceed to PR.
