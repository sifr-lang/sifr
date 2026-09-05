# M10 Wave 2 agent 5.6 Sol High Review — Pass 7

Reviewed the complete frozen `main...36cd2ff25` diff. The known `third_party/ruff` modification was ignored.

## Blockers

1. **High — affine collection insertion does not recursively or borrow-safely transfer ownership.** `append`, `appendleft`, `insert`, and affine `dict.pop` defaults only recognize direct names and mark them moved without rejecting borrowed resources. Borrowed and conditional expressions reach invalid generated Rust.

2. **High — field and subscript stores neither consume affine RHS values nor consistently avoid cloning them.** Field, nested-field, list, dict, and nested-subscript assignments accept borrowed escapes or owned reuse; simple list/dict codegen clones non-copy buffer names.

3. **High — affine self-`+=` detection is limited to a direct AST name.** Conditional and walrus-wrapped references to the target reach `extend` and generate Rust E0505 failures.

4. **High — dynamic clone capability checking is not recursive.** `list[list[Any]] * count` is accepted and emits a clone of `Vec<Box<dyn Any>>`, which is not cloneable.

5. **High — variadic `min`/`max` accepts capability-incompatible generic and borrowed operands.** Generic `T` emits a `PartialOrd` bound while calling `std::cmp::min`, and borrowed `str` operands produce `&String` where an owned `String` is required.

6. **High — affine buffers cross the async-generator `Send` boundary.** An owned buffer captured by `AsyncGenerator::new_lazy` violates the runtime's `Send` closure bound.

7. **Medium — direct strided footprints reject physically disjoint writable views.** Bounding intervals incorrectly conflict for interleaved disjoint views such as `[::2]` and `[1::2]`.

8. **Medium — activation evidence overstates negative and sendability closure.** The ledger and phase record claim passing evidence without permanent cases for the accepted programs above.

## Cleared areas and validation

The exact pass-6 direct adversarial cases now fail during Sifr checking, and valid cloneable list repetition/self-concatenation emit assignment-based Rust. Focused lowering `26/26`, codegen `7/7`, type capability `1/1`, full codegen `811/811`, and full type system `98/98` passed. The full lowering binary passed `735` with one ignored; its Unix worker integration case could not start in the read-only sandbox. HIR maintainability, diff checks, and file-size constraints passed. No additional blocker was found in producer retention, exact-once release, access/release linearization, direct overlapping admission safety, declaration validation, `Self`/import-root/bridge acquisition, or diagnostic registration.

**VERDICT: CHANGES REQUIRED**
