

Now I have comprehensive context from the source inspection. Let me produce the review.

---

# Phase 37 Correctness Reuse Plan — Strict Review

## Overall Assessment

The plan has strong structural thinking and correctly identifies the main categories. However, it is incomplete and imprecise in several areas that will matter for production-grade correctness. I am being strict because ambiguity here will produce wrong test coverage and silent semantic drift.

---

## What's Good

1. **Not depending on Cargo resolver-tests or uv test crates directly.** Correct judgment.
2. **SAT oracle concept** inspired by Cargo's sat.rs is the right approach. The Cargo oracle encodes at-most-one per source identity and semver key, feature activation implying package, links uniqueness, semver compatibility constraints, dep-feature clauses, defaults, and optional dep handling. This is a solid skeleton.
3. **Property tests via proptest** are correctly identified. Cargo's metamorphic invariants (minimal-versions agreement, removing deps can't break solve, deterministic resolution) map well to Sifr semantics.
4. **Cross-layer tests** (HIR compilation of resolved packages, PackageSourceMap boundaries, Cargo lock verification) are essential and correctly placed.
5. **Port Cargo edge cases by category, not verbatim** is the right discipline.
6. **Traceability matrix** is correctly scoped.

---

## Critical Gaps

### Gap 1: SAT Oracle Encoding Is Underspecified

The plan says "validates existence/non-existence and feature activation, target selection" but does not enumerate the exact constraints that must be encoded. Looking at Cargo's sat.rs, the following constraints are encoded:

- At-most-one per package source identity (`links` constraint)
- At-most-one per semver-compatible activation key (same package, overlapping version ranges)
- If dependency is used then the package that provides it is used
- If feature is used then the dependency that provides it is used
- Optional dep clause (non-optional deps imply package selection)
- Feature-clause activation (features that enable other features)
- DepFeature weak/non-weak semantics (Cargo distinguishes `DepFeature { weak: true/false }`)
- Default feature propagation
- Dev-dependency preconditions
- Root package is always selected

Sifr must additionally encode:
- One version per import-root/source identity (stricter than Cargo's semver key)
- Target pre-filtering as hard constraint, not soft
- Backend Cargo trust constraints
- Source-kind constraints (registry vs path vs git vs url are distinct solver packages)
- Workspace member fixed-version constraints
- Feature conflict pre-conditions (mutually exclusive features cannot both be active)

**The phase doc must enumerate the complete constraint set, not just the high-level categories.**

### Gap 2: Optional Dep Activation Semantics Are Underspecified

Cargo's sat.rs encodes the optional dep activation logic in `process_pkg_features` with a `Weak` variant. Specifically:

```rust
// weak: feature "X/Y" of dep X is activated if dep X has been activated via another feature
// non-weak: feature "X/Y" of dep X activates X, activates feature Y of X, activates feature X (the dep feature name) of X
```

The plan mentions "weak deps" but does not specify Sifr's semantics for:
1. How `dep:foo` feature activation differs from `foo?/bar` weak feature activation
2. Whether optional dep activation is explicit (requires a feature to activate) or implicit (the dep itself is optional but once selected its features are available)
3. How cyclic optional deps resolve (Cargo's test_05_cyclic_optional_dependencies shows A optional-dep B, B dep-feature A; Sifr must define the resolution)

**Add a dedicated section on optional dependency activation semantics before the oracle section.**

### Gap 3: Feature Conflict Detection Must Be Pre-Solver, Not Post-Solver

The plan correctly says "mutually exclusive features are checked as a Sifr diagnostic" but does not define how feature conflicts interact with the SAT oracle. Specifically:

- Feature conflicts must be encoded as pre-solver SAT constraints (both features cannot be selected simultaneously)
- The oracle must validate that feature conflict declarations are consistent (no A conflicts A, transitive conflicts are surfaced)
- Conflict detection must happen before PubGrub runs, not as a post-resolution sanity check

**The oracle section must explicitly say feature conflicts are SAT-constrained, not post-hoc validated.**

### Gap 4: Trust Propagation Model Is Vague

The plan says "trust does not propagate" and "untrusted direct dependency does not automatically trust transitive." This needs a formal model in the oracle:

- Sifr needs a trust-constraint variable per package (trusted/untrusted)
- `links` crates and build scripts add a native-capability constraint
- Trust edges are constrained: if package A is trusted and A depends on B, B's native capability is constrained by A's trust declaration
- The oracle must encode: for every native-capable package, the trust path from the workspace root to that package must have a trust edge for each native-capable edge

Without this, the trust model is not testable.

**Add "Trust propagation constraints" as a bullet in the oracle requirements.**

### Gap 5: Multiple Semver-Incompatible Versions Policy Is Unclear

Cargo allows multiple semver-incompatible versions of the same crate when they come from different sources. The plan says "Sifr rejects unless export roots are aliased" but this needs a formal test case classification:

1. **Same source, semver-incompatible**: hard error (Cargo rejects this too via semver key)
2. **Different registry sources, semver-incompatible**: hard error (Sifr import-root semantics require one version per source identity)
3. **Path dep vs registry dep, same name**: hard error unless explicit `{ package = "real" }` alias
4. **Workspace member vs registry dep**: hard error unless explicit alias
5. **Git dep vs registry dep, different versions**: hard error

The plan's port of "Cargo cases where multiple semver-incompatible versions are allowed" must be explicit: Sifr must document which Cargo behaviors it intentionally rejects, not just which it preserves.

**Add a subsection: "Sifr vs Cargo Multiple-Version Policy — Intentionally Divergent Cases" with explicit entries.**

### Gap 6: Workspace Catalog Inheritance Semantics Are Missing

The plan mentions workspace catalogs but does not specify the inheritance semantics that must be tested. From the manifest model:

- `[workspace.dependencies]` is a central catalog. Members opt in with `{ workspace = true }`.
- Target-specific dependency tables at workspace level are catalogs. Members opt in.

This means the SAT oracle must handle:
- A workspace catalog entry that is not a direct dependency of any member (catalog-only)
- A member that inherits a constraint from workspace but overrides version/feature
- A member that declares a dep with the same name as a workspace catalog entry but a different version

These are subtle edge cases that Cargo's resolver-tests don't cover (Cargo has workspace inheritance but Cargo's test model doesn't isolate these scenarios).

**Add workspace catalog inheritance as a test category.**

---

## What's Missing

### Missing 1: Cargo Registry Model Semantics

Cargo's registry model (IndexSummary, QueryKind, describe_source, is_replaced) is not referenced in the plan. Sifr needs a registry model that handles:
- Sparse index with conditional requests
- Yanked version pre-filtering
- Registry authentication and token attachment
- Alternate registry priority

The uv tests cover some of this (auth tests, cache tests) but the plan doesn't identify which registry behavioral tests to port.

**Add: "Registry behavior tests (Cargo: registry model; uv: auth, cache, yank)" as a category.**

### Missing 2: gix Git Interaction Patterns

The plan references gix but doesn't identify specific Git interaction correctness tests. From workspace/monorepo behavior, Sifr needs to test:
- Git dependency with locked revision remains at that revision regardless of branch updates
- Git dependency with branch/target resolves to locked commit on fetch
- Git dependency with workspace member (same repo) uses local path, not remote
- Git workspace with submodule checkout behavior
- Git changed-package detection for `[base...HEAD]` filter

uv's git tests (in uv tests, not referenced in plan) and Cargo's git tests (in cargo tests) provide patterns.

**Add: "Git dependency resolution and locked revision verification" as a test category.**

### Missing 3: Lockfile Round-Trip Correctness

The plan doesn't explicitly test lockfile serialization/deserialization correctness. From uv's lock.rs and Cargo's lockfile behavior:
- `sifr.lock` read/write round-trip preserves all fields
- Lockfile version migrations are tested
- Stale lockfile (old schema) fails with SIFR-PACKAGE-0201
- Lockfile checksum validation detects manifest changes
- Lockfile preserves requirement strings exactly (not normalized)

**Add: "Lockfile round-trip, schema migration, and staleness detection" as a test category.**

### Missing 4: Deterministic Output Ordering

Cargo's sat.rs uses `BTreeMap` for deterministic iteration. Sifr must test:
- Manifest writes produce stable TOML ordering (same tables, same order)
- Lockfile writes produce stable ordering
- Diagnostic output is deterministic (no hash-based ordering, no non-determinism from parallel fetches)
- Tree output is deterministic
- Proptest must validate: same input produces same output across multiple runs

**Add: "Deterministic output ordering validation" as a required property test.**

### Missing 5: Resolution Mode Interaction Matrix

The plan defines ResolutionMode but doesn't specify the complete interaction matrix of modes and commands. From Cargo and uv behavior, the matrix is:

| Command | Online | Offline | Locked | Frozen |
|---------|--------|---------|--------|--------|
| `sifr sync` | resolves + writes lock | cache miss error | lock mismatch error | lock mismatch + offline |
| `sifr fetch` | downloads sources | cache miss error | (no-op or warn) | (no-op or warn) |
| `sifr build` | resolves + builds | cache miss error | lock mismatch error | lock mismatch + offline |
| `sifr check` | resolves + typechecks | cache miss error | lock mismatch error | lock mismatch + offline |

Each cell is a test case. The plan doesn't enumerate this.

**Add: "Resolution mode × command interaction matrix" as mandatory test coverage.**

### Missing 6: Package Graph Cycle Detection

Workspace member cycles and solver-level cycles are mentioned but not tested concretely. From Cargo's tests:
- Direct package cycle (A depends on B, B depends on A): rejected before solve
- Optional dep cycle (A optional-dep B, B optional-dep A): handled at feature resolution
- Workspace member cycle (A member depends on B member, B member depends on A): structural cycle, rejected
- Dev-dependency cycle (A dev-dep B, B dev-dep A): handled differently from normal deps

The SAT oracle in Cargo explicitly notes "cyclic dependencies are not checked in the SAT resolver" — Sifr must encode this differently since Sifr doesn't allow arbitrary cycles at all.

**Add: "Package graph cycle detection" as a test category with subcategories for each cycle type.**

### Missing 7: Workspace Member Selection Correctness

The plan mentions selectors but doesn't specify the complete selection semantics:
- `sifr build --workspace` selects all default-members, not all members
- `sifr build --exclude pkg` excludes from selection
- `--filter pkg...` selects pkg plus dependency closure
- Empty selection is an error unless command explicitly allows it
- Global files (root lockfile, root manifest catalogs) affect all selected members

**Add: "Workspace member selection and filter semantics" as a test category.**

---

## What Should Not Be Reused

### Don't Reuse: Cargo resolver-tests API and internals

Even though the plan says not to depend on it, the plan suggests "Port the cases" without specifying that the helper DSL in `helpers.rs` (ToDep, dep, dep_req, dep_req_kind, pkg, pkg_dep, pkg_dep_with, pkg_id_source, registry) must be replaced with Sifr-owned equivalents. These helpers use Cargo's internal types (Dependency, DepKind, Summary, PackageId). Sifr must own a parallel DSL with Sifr types (PackageName, Version, SourceSpec, FeatureSpec, DependencyKind).

### Don't Reuse: uv's snapshot test infrastructure

uv uses `uv_snapshot!` macro with `assert_fs` temp projects and insta snapshots. While the pattern is good, the infrastructure is Python-specific (venv creation, Python interpreter detection, pip compatibility). Sifr needs:
- Temp registry server (not Python venv)
- Temp Git repos (gix, not git2-based uv fixtures)
- Temp workspace discovery tests (not Python package discovery)
- Normalized snapshots for Sifr manifest/lockfile/module graph output

The snapshot content is not reusable; only the pattern (temp infrastructure + insta snapshots) is reusable.

### Don't Reuse: Cargo's multiple-version behavior for same source

Cargo allows multiple semver-incompatible versions of the same package from the same source. Sifr intentionally prohibits this via its one-version-per-source-identity model. The plan correctly identifies this as divergence, but doesn't mandate that the test suite explicitly verifies the rejection.

**Every intentional divergence must have both a negative test (Sifr rejects what Cargo allows) and a positive test (Sifr accepts what Cargo and Sifr both allow).**

### Don't Reuse: Cargo's optional dep implicit activation

Cargo's optional deps are activated implicitly by feature references. Sifr may choose a different model (explicit feature activation only). The plan should explicitly decide Sifr's model and test it, not adapt Cargo's model uncritically.

---

## What Needs to Change for Production-Grade Model

### Change 1: Enumerate All SAT Constraints Explicitly

The oracle section must be a complete list, not a sketch. Replace:
```
It validates existence/non-existence and feature activation, target selection, one version per import root/source identity, direct-dependency boundaries, no workspace cycles.
```

With explicit constraint list:
1. At-most-one per (source_kind, registry_name, package_name) — source identity constraint
2. At-most-one per semver-compatible group within same package name — version compatibility constraint
3. If feature F of package P is selected → dependency D providing F is selected — feature implies dep
4. If dependency D is selected → package providing D is selected — dep implies package
5. Non-optional deps → corresponding package must be selected
6. Feature conflicts: if F1 conflicts F2 → not(F1 AND F2) — pre-solver constraint
7. Target pre-filtering: packages whose target predicate is false for active target are excluded from solve
8. Backend Cargo trust: if package P has native capability (links, build-script, proc-macro) → trust edge required from workspace root
9. Workspace member fixed-version: workspace members resolve at their declared version, not latest compatible

### Change 2: Specify Optional Dep Activation Semantics

Add a subsection:

**Optional Dep Activation (Sifr Model)**
- Optional deps are declared with `{ package = "X", optional = true }`
- Optional dep is activated only when a feature explicitly references it via `dep:X` syntax
- Weak feature references (`X?/Y`) are activated only if X is already activated by another feature
- Cyclic optional deps are resolved by iterative feature expansion until fixed point
- This model is intentionally different from Cargo's implicit activation on feature reference

### Change 3: Add Workspace Catalog Inheritance Tests

Add a test category that explicitly covers:
- Catalog entry with no direct consumers (dead catalog entry)
- Member overrides catalog entry's version/feature
- Member inherits catalog entry without explicit opt-in (negative test — should error)
- Target-specific catalog entries

### Change 4: Define the Resolution Mode × Command Interaction Matrix

Add explicit matrix as a test planning artifact.

### Change 5: Specify the Update Process More Precisely

The plan's update process says "run resolver oracle suite + port newly relevant Cargo/uv tests" but is vague. Change to:

**Update Process (Precise)**
1. When upgrading `astral-pubgrub`, `semver`, `gix`, or registry/HTTP dependencies:
   a. Run full SAT oracle suite (all constraint categories)
   b. Run property tests with seed preservation for deterministic regression detection
   c. Check if Cargo resolver-tests added new categories (semver handling, feature edge cases, etc.)
   d. Check if uv added new lock/workspace/tree behavior
   e. Port new Cargo/uv categories as new Sifr test files, not as wholesale vendor
   f. Update traceability matrix with new test file entries
   g. Run full local validation (`scripts/run_all_tests.sh`)
2. Do not vendor Cargo resolver-tests or uv test crates as dependencies.
3. Document the upstream version pin and upgrade reason in architecture docs.

---

## Recommendations for Phase Doc Changes

1. **Expand the oracle requirements** with the explicit constraint list above.
2. **Add optional dep activation semantics** as a dedicated subsection.
3. **Add the resolution mode × command matrix** as a test planning artifact.
4. **Add a "Sifr vs Cargo Multiple-Version Policy" section** with explicit entries for each divergence case.
5. **Add workspace catalog inheritance** as a test category.
6. **Add package graph cycle detection** as a test category with subcategories.
7. **Clarify trust model** with explicit SAT constraint encoding.
8. **Specify the update process** more precisely.
9. **Add the complete test category list** to the phase doc (not just the high-level ones).

---

## Summary

The plan is structurally sound but under-specified in critical correctness areas. The SAT oracle needs a complete constraint enumeration. Feature conflict detection must be pre-solver. Optional dep activation must be specified. Trust propagation must be formal. Multiple-version policy must be explicit. The resolution mode interaction matrix must be enumerated. Workspace catalog inheritance and workspace member selection must be covered.

With these changes, the plan becomes production-grade. Without them, the test suite will miss correctness boundaries that will surface as bugs in user-facing package workflows.
