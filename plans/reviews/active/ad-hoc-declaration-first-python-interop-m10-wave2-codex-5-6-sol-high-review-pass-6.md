# M10 Wave 2 Codex 5.6 Sol High Review — Pass 6

Reviewed `main...3d79ab04a` in full: 116 files, 3,901 additions, 342 deletions. The known `third_party/ruff` modification was ignored.

## Findings

1. **High — list repetition still permits cloning affine and dynamically typed elements.** The type checker accepts every `list[T] * int` without checking clone capability (`crates/sifr_type_system/src/check.rs:206`). Codegen then clones the source and uses `.iter().cloned()` (`crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_binop.rs:203`), while singleton repetition uses `std::iter::repeat` (`stmt_expr_binop.rs:178`). `PythonBuffer` is intentionally non-`Clone` (`crates/sifr_stdlib/src/python/buffer.rs:243`). Frozen `sifr check` nevertheless reports “no errors found” for both:

   ```sifr
   def duplicate(values: list[python.Buffer[uint8]], count: int) -> list[python.Buffer[uint8]]:
       return values * count
   ```

   and `list[Any] * count`. `list *= count` is also accepted and emitted as Rust `*=`, which is invalid for `Vec`. This leaves both affine and Any/Unknown clone paths open.

2. **High — variadic `min`/`max` bypass affine ownership and ordering capability checks.** Only the one-iterable forms apply the new affine projection guard. The two-or-more-argument paths merely lower operands and test type assignability. Codegen emits `std::cmp::min/max`, requiring `Ord` and consuming both operands. Frozen checking accepts affine buffers, `Any`, and even use after the implicit move:

   ```sifr
   def minimum(own left: python.Buffer[uint8], own right: python.Buffer[uint8]) -> None:
       selected: python.Buffer[uint8] = min(left, right)
       print(left.length())
   ```

   This is both a generated-Rust compilation failure and an ownership-analysis hole. The prior min/max blocker is therefore only partially closed.

3. **High — affine-list self `+=` is accepted.** Lowering consumes the RHS but does not reject it aliasing the assignment target. Codegen produces `values.extend(values)`, which Rust rejects and whose Python semantics would duplicate affine owners. Frozen `sifr check` accepts:

   ```sifr
   values: list[python.Buffer[uint8]] = []
   values += values
   ```

   The focused test covers only distinct `left += right` followed by RHS reuse.

4. **Medium — permanent evidence and activation governance overstate closure.** The phase record claims complete dynamic capability validation, augmented-assignment moves, and permanent negative coverage. The activation ledger marks all buffer evidence passing, but the negative tests omit the accepted cases above. The declaration architecture’s status header also still describes later zero-copy sections as inactive while the same document declares buffers active.

The exact pass-5 borrowed-call/constructor/aggregate-return cases, iterable min/max, concat/sorted/sum, tuple/dict moves, and distinct-RHS list `+=` are implemented and covered. No additional blocker was found in shared-storage admission, iterators/generators/yield, TypeVar rejection, conditional moves, exact-once release, access/release linearization, producer retention, Self/import-root/bridge generation, PYZC diagnostics, or file responsibility limits.

Validation run: focused lowering 26/26, codegen 6/6, and HIR maintainability guardrails passed. The four adversarial checks above all returned “no errors found.”

**VERDICT: CHANGES REQUIRED**
