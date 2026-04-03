# Ad Hoc Optional/None Closure: Wave-R3d Reviewer Pass-1 (2026-03-30)

Reviewer workflow: `talk-to-claude` skill was invoked and wait loop was run for the requested output artifact path (`tmp/ad-hoc-optional-none-wave-r3d-review-pass1.md`), but no file was produced within the active implementation window.

Status: `ready-with-guardrails`  
Decision basis: direct root-cause replay + pre-apply reviewer prompts + post-implementation validation evidence.

## Reviewed approach

- Persist empty-list specialization into declared binding type for `append` / `insert` / `extend` so loop-body specialization does not collapse back to `list[Any]` at return boundaries.
- Enforce strict 2-arg `max`/`min` typing: reject optional operands and reject incompatible argument pairs.

## Guardrails

- This wave intentionally closes run-stage mismatches by promoting invalid Optional flows into explicit check-stage diagnostics.
- No demo rewrite was used; ownership stays in HIR typing/lowering.
- Keep future work focused on stronger proof/narrowing only if we can prove safety without hidden unwrap behavior.
