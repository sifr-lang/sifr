# Sifr Compiler Architecture Gap Review

Date: 2026-02-14  
Reviewed file: `.cursor/plans/sifr_compiler_architecture_fa3c10ee.plan.md`

## Findings (ordered by severity)

### High 1: Contradictory indexing semantics

The plan defines conflicting out-of-bounds behavior:

- M3b says `s[i]` panics on out-of-bounds and `s.get(i)` is safe.
- M4 says out-of-bounds indexing returns `Result` or `Option`, and even states `list[i]` returns `Option[T]`.

This contradiction will cause divergence across parser/type checker/codegen/tests unless one model is chosen and enforced consistently for both strings and lists.

**Recommendation**

- Pick one global indexing contract and apply it everywhere:
  - either Python-like (`x[i]` may panic, `x.get(i)` is safe), or
  - total/safe indexing (`x[i]` returns `Option`/`Result`).
- Update milestone DoD and E2E expectations accordingly.

---

### High 2: Diagnostic mapping contract is missing

The pipeline compiles generated Rust with `rustc`, and tests assert diagnostic codes/columns, but there is no explicit contract for diagnostic attribution between generated Rust and original `.sifr` source.

Without a source mapping strategy, users will see unstable or confusing errors as features expand.

**Recommendation**

- Add a cross-cutting diagnostic contract covering:
  - stable Sifr diagnostic codes and ownership of each code,
  - span mapping from Rust output back to `.sifr`,
  - when to suppress/translate raw `rustc` diagnostics,
  - formatter for multi-file error rendering and related notes.

---

### Medium 1: `with` lifecycle contract is not reflected in milestone language scope

Early plan text removes `with` from initial AST scope. Later, cross-cutting destruction semantics make `with` a core cleanup primitive and assign M4/M8 responsibilities, but M4 language features do not explicitly list syntax/parser/type-checking work for `with`.

This creates a roadmap hole for reintroduction timing and ownership.

**Recommendation**

- Add explicit M4 scope entries for:
  - `with` syntax reintroduction,
  - typing rules for resource/context protocol,
  - lowering/codegen contract,
  - diagnostics for cleanup misuse.

---

### Medium 2: Incremental compilation contract is too coarse

The plan includes module caching and generated Rust caching, but does not define robust invalidation rules (content hash vs mtime, API fingerprinting, transitive invalidation, macro/decorator expansion effects).

This risks stale artifacts and hard-to-debug watch/LSP behavior.

**Recommendation**

- Define cache key and invalidation contracts:
  - content hash-based fingerprints,
  - public API signature hash per module,
  - dependency graph invalidation rules,
  - expansion fingerprinting for decorators/macros.

---

### Medium 3: FFI panic boundary guarantee is overbroad

The plan currently states panics from Rust/C libraries are converted to `Result::Err` via `catch_unwind`. That claim is too broad at FFI boundaries and can imply guarantees the runtime cannot always provide.

**Recommendation**

- Tighten wording to a policy contract:
  - define unwind behavior at Rust and C boundaries explicitly,
  - state which FFI paths are guaranteed recoverable vs aborting,
  - require explicit safe wrapper boundaries and document non-recoverable cases.

## Suggested Patch Areas in the Main Plan

1. **Cross-cutting Contracts**
   - Add a new diagnostics/source-mapping contract section.
   - Expand incremental compilation with concrete cache/invalidation rules.
   - Refine FFI panic/unwind wording for correctness.

2. **M4**
   - Add explicit parser/checker/codegen scope for `with`.

3. **M3b + M4**
   - Resolve and unify out-of-bounds indexing semantics.

## Final Note

The architecture is strong overall, but these gaps are contract-level issues. Resolving them now will reduce implementation drift across milestones and avoid regressions in compiler UX.
