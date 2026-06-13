---

## CHANGES_REQUESTED

The phase contract is well-structured: crate ownership is clear, non-goals are comprehensive, the dependency graph is acyclic, and no speculative crates are introduced. Four targeted fixes are needed before implementation.

---

### Finding 1 — Broken validation gate: `sifr_lint --depth 2` (BLOCKING)

**Where**: milestone_ir_boundary_1 and milestone_ir_boundary_2, validation step:
```
cargo tree -p sifr_lint --depth 2 shows no lowering dependency
```

**Problem**: `sifr_lint` currently depends on `sifr_frontend` (confirmed in `crates/sifr_lint/Cargo.toml`). After milestone_ir_boundary_2, `sifr_frontend` will depend on `sifr_lowering`. `cargo tree --depth 2` shows transitive deps at depth 2, so `sifr_lowering` will appear at depth 2 via `sifr_lint → sifr_frontend → sifr_lowering`. The check will fail even when the milestone intent (no *direct* lowering dep from sifr_lint) is fully satisfied.

**Fix options** — pick one and state it explicitly:
- Change the check to `--depth 1` if the rule is "no direct dep on sifr_lowering."
- Explicitly scope milestone_ir_boundary_1 to also drop `sifr_lint → sifr_frontend` (only if that removal is actually intended).
- Use `cargo metadata` with explicit direct-dep filtering and note that sifr_lowering appearing transitively via sifr_frontend is acceptable.

Locked rule #12 says "must not depend on sifr_lowering" without clarifying direct vs. transitive. The validation gate and the locked rule should agree on which it is.

---

### Finding 2 — Missing dependency-direction validation for `sifr_stdlib` itself (MEANINGFUL)

**Where**: milestone_stdlib_boundary_1, Validation section.

**Problem**: The Definition of Done asserts "sifr_stdlib has no dependency on lowering, frontend, codegen, driver, package, analysis, LSP, or CLI crates" but no validation command checks this. The milestone validates that intrinsic signatures resolve correctly but does not verify the dependency boundary was not accidentally violated during the move.

**Fix**: Add `cargo tree -p sifr_stdlib --depth 5` (or equivalent) to the validation list, checked against the forbidden set from locked rule #5.

---

### Finding 3 — "stdlib bootstrap plumbing" exception is undefined (MINOR)

**Where**: Locked rule #14.

> `sifr_driver` should obtain lowered modules through `sifr_frontend` except for explicitly documented stdlib bootstrap plumbing.

The exception exists but is not documented anywhere in the phase. An implementer who wants to add a direct driver → lowering call can cite this exception without any review gate.

**Fix**: Either (a) define concretely what "stdlib bootstrap plumbing" means (e.g., "driver calling `sifr_stdlib::StdlibSource` to obtain source bytes for the compilation bootstrap pass, not lowering machinery"), or (b) remove the exception and add it as a named non-goal.

---

### Finding 4 — Conditional binary-size check is unevaluable (MINOR)

**Where**: milestone_stdlib_boundary_2, Validation:
```
scripts/check_codegen_binary_size.sh if generated dependency changes can affect binary-size gates
```

The quality contract requires byte-for-byte equivalence of generated Cargo dependencies unless an intentional cleanup is recorded. Under that constraint, the condition is always true: any generated dependency change can affect binary size. The conditional makes the gate effectively optional with no trigger definition.

**Fix**: Either make it unconditional (matches the quality contract intent), or explicitly list the specific dependency changes that are exempt from the size check and drop the conditional form.

---

### Finding 5 — `Cargo.lock` omitted from sifr_hir reference sweep (MINOR)

**Where**: milestone_ir_boundary_2, Validation:
```
rg "sifr_hir|crates/sifr_hir" Cargo.toml crates internal_docs docs issues scripts verification
```

After the rename, `Cargo.lock` will still contain `sifr_hir` package entries until regenerated. A stale Cargo.lock with `sifr_hir` would pass this check. Either add `Cargo.lock` to the rg target list (and handle the expected-empty case) or add a note that Cargo.lock is intentionally excluded and explain why.

---

### What is already solid

- No dependency cycles in the target graph. The `sifr_stdlib → sifr_type_system / sifr_diagnostics / sifr_source` fan-in is clean and acyclic.
- Non-goals section is thorough: `sifr_core`, `sifr_utils`, `sifr_runtime_async`, `sifr_model`, `sifr_runtime_validation`, and the full speculative surface list are all explicitly deferred.
- Crate ownership contracts (`sifr_ir` owns data / `sifr_lowering` owns production / `sifr_stdlib` owns the host contract) are precise and non-overlapping.
- The `IntrinsicSignature` / `StdlibFeatureSpec` public concept sketch is a useful implementation anchor.
- Quality contract covers panic policy, fixture preservation, line-count guardrail, and the positive+negative validation requirement.
- Source of truth clause ("PRs must not widen the crate tree beyond the crates named here") is the right governance model.
