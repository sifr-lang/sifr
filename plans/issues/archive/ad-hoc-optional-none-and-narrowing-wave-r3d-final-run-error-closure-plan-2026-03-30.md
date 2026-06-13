# Ad Hoc Optional/None Closure: Wave-R3d Final Run-Error Closure Plan (2026-03-30)

## Scope

Residual post-R3c run-stage fixtures:

- `0054_spiral_matrix`
- `0763_partition_labels`

## Root cause decomposition

1. `0054`:
- `res` initialized as `[]` remained `list[Any]` at check-time across loop boundaries.
- append element type evidence inside loop did not persist in declared binding type.
- codegen concretized element shape from pushes and surfaced a Rust mismatch (`Vec<Option<i64>>` vs `Vec<i64>`) at run-stage.

2. `0763`:
- `count[c]` is `int | None`, but 2-arg `max()` accepted the operand pair without Optional/type compatibility enforcement in HIR.
- run-stage Rust mismatch surfaced (`max(i64, Option<i64>)`).

## Planned compiler-only fixes

- Persist empty-list specialization into declared scope type on `append`/`insert`/`extend`.
- Add strict 2-arg `max`/`min` validation:
  - reject Optional operands (`None`-containing union),
  - reject incompatible operand pairs.

## Success criteria

- No remaining run-stage failures for this residual pair.
- Both fixtures fail (if still invalid) at check stage with explicit diagnostics.
- Quick validation lane passes.
