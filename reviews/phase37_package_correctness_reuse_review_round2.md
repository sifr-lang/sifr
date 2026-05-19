

Let me now produce a careful strict review comparing Round 1 findings against the current doc.

---

# Phase 37 Correctness Reuse Plan — Round 2 Strict Review

## Overall Assessment

Round 1 identified 7 critical gaps and 7 missing items. All have been addressed in the current doc. The SAT oracle now has 15 explicit constraints. Optional dep activation is a dedicated subsection. Multiple-version divergence is explicit. Resolution mode × command matrix is a table. The mandatory property tests and integration categories are complete. Two small findings remain before the doc is implementation-ready.

---

## Status of Round 1 Critical Gaps

All 7 gaps are resolved:

1. **SAT Oracle Encoding** — Lines 97-114 enumerate 15 constraints. The constraint set covers source identity, semver compatibility, feature→dep implication, dep→package implication, optional dep activation, feature conflicts as pre-solver constraint, target pre-filter, trust, workspace members, and source-kind isolation. Complete.

2. **Optional Dep Activation** — Lines 116-121 are a dedicated subsection. `dep:name` explicit activation, `name/feature` non-weak semantics, `name?/feature` weak semantics, and feature fixed-point expansion are all defined. The diagnostic `SIFR-PACKAGE-0103` for unactivated deps is specified.

3. **Feature Conflict Pre-Solver** — Constraint 9 (line 109): "Feature conflicts are pre-solver constraints: two conflicting feature atoms cannot both be active in the same solve." This is correctly placed in the SAT oracle requirements, not as post-hoc validation.

4. **Trust Propagation** — Constraint 12 (line 111) encodes the trust model in the oracle: "Native-capable backend dependencies, build scripts, `links` crates, and proc macros require explicit root trust entries; every selected native-capable package without trust fails the oracle." The Registry/Publishing/Trust section (lines 624-633) further specifies non-propagation and diagnostic reporting.

5. **Multiple-Version Policy** — Lines 123-128 are an explicit subsection with 5 hard-error cases covering same source, different registries, path vs registry, workspace vs registry, and Git vs registry. The aliasing rule for intentional divergence is stated: "Every Cargo resolver case that Cargo accepts through multiple semver-incompatible versions must have a Sifr negative test showing the intentional rejection, plus a positive alias test."

6. **Workspace Catalog Inheritance** — Lines 130-136 enumerate workspace catalog semantics as a test category. Catalog-only entries, `{ workspace = true }` inheritance, non-inheritance without the flag, override behavior, target-specific catalogs, and selection tests are all specified.

7. **Resolution Mode × Command Matrix** — Lines 138-149 are a table covering all 8 commands across all 4 modes.

---

## Remaining Findings

### Finding 1: Optional Dependency Cycle Fixed-Point Is Underspecified in the Oracle

The doc states (line 121):
> "Cyclic optional dependency graphs are rejected unless expansion reaches a stable finite fixed point without introducing a package cycle."

The SAT oracle constraint 14 (line 113) says:
> "Structural workspace package cycles and non-dev package cycles are rejected before PubGrub and are not treated as valid SAT solutions."

The gap: constraint 14 covers structural package cycles, but the "stable finite fixed point" property of cyclic optional dependency graphs is not a SAT constraint. It is stated as a pre-condition check. The oracle does not verify that cyclic optional deps actually reach a fixed point — only that they don't introduce a package cycle.

The doc's description of optional dep activation (lines 116-121) and the SAT oracle (lines 97-114) are internally consistent on this: cycles through optional dep edges that don't create structural package cycles are allowed if feature expansion terminates. But this convergence property needs to be encoded in the test suite explicitly.

**Recommendation**: Add to mandatory property tests (line 151):
> "Optional dependency cycle convergence: for any optional dep graph, iterative feature expansion terminates in bounded iterations; the bound is deterministic from the graph's strongly connected components."

### Finding 2: uv `version_map.rs` / `candidate_selector.rs` / `error.rs` Are Implementation-Crate Files, Not Test Files

The "Correctness Test Suite Reuse Plan" section (lines 74-83) references uv internal files:
> "uv `crates/uv/tests/it/lock.rs`, `lock_conflict.rs`, and resolver internals such as `preferences.rs`, `yanks.rs`, `upgrade.rs`, `version_map.rs`, `candidate_selector.rs`, and `error.rs`"

The files `preferences.rs`, `yanks.rs`, `upgrade.rs`, `version_map.rs`, `candidate_selector.rs`, and `error.rs` in uv are **implementation source files** in the uv resolver library crates (`crates/uv-resolver/src/`), not test files. The plan's phrasing "resolver internals such as..." conflates implementation references with test-porting targets.

This is not a blocker — the intent to use uv resolver internals as reference implementations is correct and stated in the Reuse Strategy section (lines 61-66). But the "Correctness Test Suite Reuse Plan" section should distinguish:
- Files in `crates/uv/tests/it/*.rs` → actual integration tests to port
- Files in `crates/uv-resolver/src/*.rs` → implementation references for patterns, not test ports

**Recommendation**: Add one sentence to the "Do not reuse directly" section (line 85):
> "uv resolver library implementation files (`preferences.rs`, `version_map.rs`, `candidate_selector.rs`, `error.rs`) are reference patterns, not test ports; port behavior from uv's `tests/it/` integration tests, not from internal library modules."

---

## What Is Confirmed Correct

The following are confirmed correctly scoped and are not blockers:

- **SAT oracle constraint 11** (yanked versions): correctly scoped to existing lockfiles with matching checksum, aligned with the sparse index contract (line 602).
- **SAT oracle constraint 15** (direct-dependency import boundaries): correctly scopes transitive dep compilation vs import availability, consistent with the PackageSourceMap `dependency_scopes` field (lines 487-489, 498).
- **Target pre-filter constraint 10**: correctly states dependencies whose target predicate is false for the active target never enter solver input, consistent with the resolution pipeline's pre-expanded feature edges model (line 345).
- **Cargo resolver-tests porting list**: correctly excludes Cargo `crates-io` / `cargo-platform`, correctly identifies `resolver-tests` as oracle/reference not implementation.
- **Property test for shortest conflict paths** (line 163): "package-edge count, then lexical tie-breakers" is a concrete tie-breaking rule that enables deterministic testing.
- **Upstream update process** (lines 179-185): 6-step process with traceability matrix update is precise and implementable.
- **Trust non-propagation model** (line 628): "A trusted direct dependency does not automatically trust an untrusted transitive native dependency" is correct and prevents the transitive trust escalation attack.
- **`dep:name` activation for optional deps** (line 118): this is an explicit opt-in model that differs from Cargo's implicit activation — intentional divergence correctly documented.
- **`name/feature` non-weak semantics** (line 119): correctly requires the dep to be non-optional or already activated, with `SIFR-PACKAGE-0103` error — this is stricter than Cargo and intentionally so.
- **Lockfile never stores credentials** (line 437): explicit guarantee aligned with trust model and credential redaction diagnostics.

---

## Verdict

**No blockers.** All Round 1 critical gaps are resolved. The plan correctly reuses Cargo resolver-tests and uv integration tests as oracles and reference patterns, not as implementation dependencies. Sifr's import-root semantics divergence from Cargo's multiple-version behavior is explicit with negative-test + positive-alias-test requirements. The SAT oracle has 15 concrete constraints. The property test suite is complete. The upstream update process is precise.

Two small findings above: optional dep cycle convergence needs an explicit property test, and uv resolver library files should be marked as reference patterns not test ports. Both are polish items.

**The plan is implementation-ready pending the two small additions.**
