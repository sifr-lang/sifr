# Phase 37 Gap Analysis Review (Round 1)

**Reviewer**: Implementation-readiness audit
**Document**: `internal_docs/phases/37_package_management.md` (proposed v3 planning contract)
**Date**: 2026-05-19
**Result**: **ready-with-nits**

---

## Summary

The plan is structurally sound and implementable. The core decision (Cargo as primary substrate, sifr.toml as compiler semantics authority, sifr_package as the only Cargo-touching crate) is internally consistent. Milestone ordering is correct with no circular dependencies. Demos are specific and concrete. The main issues are nits: missing definitions for existing references, a few missing diagnostic codes, incomplete demo file content, and minor inconsistencies. None constitute blockers.

---

## Blockers

**None.** No material blocker prevents an engineer from beginning implementation.

The plan defers to future milestones for items not yet in the codebase (`cargo_metadata` not yet in Cargo.toml, `sifr_package` crate not yet created, Phase 36 completed). This is appropriate for a planning contract.

---

## Nits (Require Clarification or Fix)

### N-1: `SIFR-PACKAGE-0103` Is Referenced But Not Defined

The "Sifr Metadata In Cargo" section states:
> If unsupported Sifr compiler metadata appears in Cargo metadata instead of `sifr.toml`, Sifr reports `SIFR-PACKAGE-0003`.

The diagnostics table at line 822 correctly maps `SIFR-PACKAGE-0003` to "unsupported or misplaced Sifr compiler metadata in Cargo metadata." However, the table skips `SIFR-PACKAGE-0103` entirely — it goes 0101, 0102, 0104. `SIFR-PACKAGE-0103` is never mentioned in the document body, only as a reserved block comment (`SIFR-PACKAGE-0302` through `0309` are reserved for trust).

**Action**: Either remove the gap by defining `SIFR-PACKAGE-0103`, or explicitly mark it reserved in the table.

### N-2: `OperationPlan` Is Referenced But Never Defined

The guardrails section states:
> mutating CLI commands route through `OperationPlan`

The module map defines `ops/{plan,mutate,resolve,read,publish}.rs`, but `OperationPlan` the type is never described: its fields, its role, what it guards, or how it differs from `ops::mutate` or `ops::publish`. The module name `plan.rs` suggests an implementation detail, not the top-level public type.

**Action**: Define `OperationPlan` in the module map or rename to something that appears in the source. Clarify what it prevents (e.g., "ensures all mutating CLI ops go through one gate that validates lock-mode compliance").

### N-3: `sifr.toml` `edition` Field Has No Semantic

`sifr.toml` has `edition = "2026"` in examples, but the plan never defines:
- What a Sifr edition means (syntax version? ABI version?).
- What happens if a dependency declares `edition = "2025"` while the root has `edition = "2026"`.
- Whether Sifr edition maps to Cargo edition or is orthogonal.
- What breaks if `sifr.toml edition` and `Cargo.toml edition` differ for a Rust-backed Sifr package.

The existing `sifr_workspace_design.md` introduced `[package].edition` as a string field but also did not define its semantic. This is a pre-existing gap that Phase 37 inherits. Since Phase 37 is the first phase to use `edition` semantically (with compiler version requirements), this gap becomes more impactful.

**Action**: Add a definition for `sifr.toml [package].edition`. At minimum, define that it is orthogonal to `Cargo.toml edition` and has no cross-package enforcement in Phase 37. Recommend a deferred semantics section: "Sifr edition semantics (compatibility, migration, deprecation) are defined in a separate edition policy doc, not in Phase 37."

### N-4: `[features]` Cross-Package Mapping Validation Is Unspecified

The feature section shows:
```toml
tls = { cargo-package = "sifr-http", cargo-feature = "tls" }
json = { cargo-package = "reqwest", cargo-feature = "json" }
```

The plan states that Cargo features are mapped, but it never specifies:
- What happens if `cargo-package` names a package that is not a direct Cargo dependency of the current package.
- Whether the mapped Cargo feature must exist at validation time or only at build time.
- Whether a Sifr feature can map to zero Cargo features (a no-op feature for conditional compilation).

**Action**: Add validation rule: "At `sifr build --features` time, if `cargo-package` does not appear in the resolved Cargo dependency graph, `SIFR-PACKAGE-0303` is reported." Reserve `SIFR-PACKAGE-0303`.

### N-5: Demo Source Files Are Missing

`sifr-demo-app/sifr/app/main.sifr` contains:
```sifr
from demo_http.client import get
from demo_json.parse import parse_json
```

But the plan never shows `sifr/demo_http/client.sifr` or `sifr/demo_json/parse.sifr`. The demo repo descriptions in the plan reference these files as concrete, but their contents are absent. This means:
- The imports in the demo app are not self-verifying.
- A reviewer cannot check whether the import syntax matches the actual exported module structure.
- The `get` function returning a `response.body` that has a dict with `.as_int()` is unverified.

**Action**: Either show the source files for `demo_http/client.sifr` and `demo_json/parse.sifr` in the plan, or add a note that "full demo source content lives in the GitHub repos; plan shows structural highlights only."

### N-6: `sifr tree` Command Has No Behavior Section

The CLI contract shows `sifr tree [--workspace|-p package] [--sifr-only|--all]`, but there is no "Behavior" subsection explaining what `sifr tree` outputs. Contrast this with `sifr add`, `sifr update`, `sifr fetch`, `sifr vendor`, `sifr package`, `sifr publish`, and `sifr check` — all of which have behavior descriptions.

**Action**: Add a one-paragraph behavior description for `sifr tree`. At minimum: "Displays the package dependency tree. `--sifr-only` shows only Sifr source packages. `--all` includes backend Rust crates. `--workspace` shows the Cargo workspace graph filtered by Sifr metadata."

### N-7: `sifr outdated` Semantics Are Unspecified

Milestone 37_5 mentions implementing `sifr outdated`. The plan never defines what "outdated" means in Sifr context:
- Compared to Cargo registry (matching `cargo outdated` behavior)?
- Compared to `Cargo.lock` (what changed since last lock)?
- Compared to `sifr.toml` declared version ranges?

**Action**: Add a one-line definition. Recommendation: "Delegates to `cargo outdated` where available; Sifr reports its own packages that have newer versions in their Cargo source."

### N-8: `[trust]` Policy Validation Lacks Depth

`trust.native = ["reqwest"]` in the example means reqwest is trusted for native linking. The plan does not specify:
- How to validate that reqwest actually uses native linking (vs. being a pure-Rust crate).
- What happens if a package trusts a crate that is not a direct dependency.
- Whether transitive trust propagation is supported (if A trusts B, and B uses reqwest, does A implicitly trust reqwest?).

**Action**: Add a one-sentence trust validation rule: "Trust is validated only against direct Cargo dependencies declared in the package's `Cargo.toml`. Transitive trust is not inherited; each Sifr package must declare trust explicitly."

### N-9: `sifr add <package> --features` Delegation Is Ambiguous

The behavior states: "delegates the Cargo feature mutation where practical."

"where practical" is not defined. Cases where it might not be practical:
- Sifr package that is not a direct Cargo dependency?
- Feature that has no Cargo equivalent?
- Package that uses a registry feature extension not in Cargo?

**Action**: Replace "where practical" with explicit rule: "Always delegates to `cargo add` for Cargo package name and version. For feature flags, delegates only when the named feature exists in the selected Cargo package's feature set. If the feature has no Cargo equivalent, adds it to `sifr.toml [features]` only."

### N-10: Diagnostics Table Excludes `SIFR-PACKAGE-0103` From the First Block

At lines 815-834, the diagnostic table covers ranges 0001-0003, 0101-0105, 0201-0204, 0301, 0401-0403, 0501. `SIFR-PACKAGE-0103` is never mapped. N-1 above covers this.

---

## Observations (Not Blockers)

### O-1: Cargo Test Reuse Is Conceptual, Not Mechanical

The plan says to "port/adapt" Cargo resolver, lockfile, metadata, package/publish, vendor/fetch, workspace, and build script/proc macro/links tests. Cargo's tests are not distributed as standalone fixtures — they live in the Cargo source tree and test Cargo internals. Sifr cannot simply copy them. The plan uses "adapt" correctly rather than "copy," meaning engineers will reimplement analogous tests against the Sifr model. This is fine but should be explicitly noted.

**Recommendation**: The TRACEABILITY.md document (milestone 37_7 deliverable) should explicitly mark each Cargo test category as:
- `ported`: test logic derived from Cargo test, reimplemented for Sifr model
- `adapted`: Cargo test approach applied with Sifr-specific variations
- `skipped`: not applicable to Sifr model (with reason)
- `deferred`: future phase

### O-2: `edition = "2024"` in Cargo.toml Example Is Inconsistent With Workspace

The plan shows `edition = "2024"` in the Cargo.toml example for the sifr-http package. This is invalid — Cargo edition values are `2015`, `2018`, `2021`. `2024` is not a valid Cargo edition. This appears to be a typo (meant `2021` or `2024` was intended to be a Sifr-only value).

**Action**: Change `edition = "2024"` to `edition = "2021"` in the Cargo.toml example at line 49.

### O-3: sifr-demo-http Trust Demo Is Partially Incomplete

`sifr-demo-http` is marked as "Rust-backed Sifr package." Its `src/lib.rs` is not shown, and `sifr.toml` declares `trust.native = ["reqwest"]`. However, the package's actual Rust implementation is not shown in the plan. To validate trust, the demo needs either:
- `src/lib.rs` that actually uses reqwest (and gets built/trusted), or
- A stub that the trust validator must still check and flag as needing trust.

The plan mentions "Rust-backed trust validation accepts sifr-demo-http only with explicit reqwest trust" as a demo validation point. But without showing the actual reqwest usage, it's unclear whether the trust check would fire.

**Action**: Either show `src/lib.rs` using reqwest in the plan, or add a note that the demo includes a minimal reqwest FFI shim under `src/lib.rs` that exercises the trust policy.

### O-4: Phase 37 Adds Fields to an Existing sifr.toml Schema

`sifr.toml` currently exists in the codebase with fields from the workspace design phase (lines 38-46 of `sifr_workspace_design.md`): `name`, `version`, `edition`, `[workspace]`, `[source].roots`, `[[bin]]`, `[dependencies]` (reserved), `[profile.dev]` (reserved). Phase 37 adds `[exports]`, `[features]`, `[trust]`, `sifr-version`, and expands `edition` semantics. The plan never explicitly states that the sifr.toml schema is being extended rather than replaced, and does not define how unknown fields are handled (forward compatibility). The existing workspace design says "Unknown top-level tables and unknown nested keys are accepted and ignored for forward compatibility." Phase 37 should explicitly rely on this.

**Action**: Add a note: "Phase 37 extends the sifr.toml schema defined in `sifr_workspace_design.md`. All Phase 37 fields are additive. Unknown fields continue to be accepted per the forward-compatibility rule in the workspace design."

---

## What Is Good

### Structural Clarity

The plan correctly separates concerns:
- Cargo owns: resolution, sources, lockfile, registries, publishing, vendoring, backend Rust/native.
- Sifr owns: compiler semantics, import/export boundaries, type identity, scoped imports, package diagnostics, trust policy.

This boundary is consistently maintained throughout. The canonical files section is precise. The SifrMetadataInCargo section correctly specifies that only `[package.metadata.sifr]` lives in Cargo metadata, and all compiler semantics live in `sifr.toml`.

### Milestone Ordering

The milestone sequence is correct. Each milestone builds on the previous without circular dependency:
- 37_1: metadata and manifest linking (foundation)
- 37_2: package graph and scoped imports (derives from 37_1)
- 37_3: source compilation and PackageSourceMap (derives from 37_2)
- 37_4: Cargo commands and lock modes (derives from 37_1-3)
- 37_5: workspaces and tooling (derives from 37_2-4)
- 37_6: packaging and publishing (derives from 37_1-5)
- 37_7: validation and guardrails (synthesis)

### Data Structure Completeness

`SifrPackageGraph` and `SifrPackageMetadata` are well-specified with all fields defined. `ModuleOrigin` is a clean abstraction that integrates with the existing module resolver. The graph digest composition is thorough.

### Cargo Reuse Is Realistic

Using `cargo metadata --format-version 1`, `cargo fetch`, `cargo update`, `cargo add`/`remove`, `cargo build --locked/--offline/--frozen`, `cargo package`/`publish`/`yank`, and `cargo vendor` is the correct set. These are all stable command-line surfaces that don't require linking Cargo internals. The plan correctly prefers CLI over internal crate APIs.

### Multiple Versions/Scoped Imports/Type Identity

The core rule ("ambiguity is an error only inside one package's own direct dependency scope") is well-defined. The type identity example (`sifr-math@1.4::math.Vector != sifr-math@2.1::math.Vector`) is clear. `SIFR-PACKAGE-0204` is specified with required fields. The plan correctly defines direct dependency as exactly one Cargo resolution edge.

### Demos Are Concrete

The demo repos (`sifr-demo-json`, `sifr-demo-http`, `sifr-demo-test-support`, `sifr-demo-app`) are specific and GitHub-hosted. The validation suite is well-specified. The multiple-version alias example with `demo_json_v1` and `demo_json_v2` is clear and implementable.

### Diagnostics Are Comprehensive

15 diagnostic codes covering manifest, metadata, source availability, import ambiguity, type identity, trust, and packaging are well-defined. Structured origin data requirements are thorough. Cargo stderr/stdout redaction requirement is correctly specified.

### Boundary Rules Are Enforced

The maintainability architecture correctly defines that only `sifr_package::cargo::*` touches Cargo, no Cargo metadata crate types cross the public facade, and diagnostics go through `sifr_diagnostics`. This is a clean boundary that prevents the "Cargo creeping into the compiler" problem.

---

## Exact Patch Recommendations

### Patch 1: Fix Cargo.toml edition typo

**File**: Phase 37 plan, line 49
**Change**: `edition = "2024"` → `edition = "2021"`

### Patch 2: Add `SIFR-PACKAGE-0103` definition

**File**: Phase 37 plan, after line 824
**Change**: Add row to diagnostics table:
```
| `SIFR-PACKAGE-0103` | Cargo metadata parsing or normalization error |
```

Or mark it explicitly reserved: `SIFR-PACKAGE-0103` is reserved for future Cargo metadata diagnostics.

### Patch 3: Define `OperationPlan` in module map

**File**: Phase 37 plan, module map section
**Change**: Under `ops/`, add:
```text
  ops/{plan,mutate,resolve,read,publish}.rs
    plan.rs          -- OperationPlan struct: gates all mutating CLI operations
                        against lock-mode semantics. Ensures no mutating operation
                        proceeds in --frozen mode. Fields: operation, package_id,
                        lock_mode, validated_graph.
```

### Patch 4: Add `sifr.toml` extension note

**File**: Phase 37 plan, "sifr.toml" canonical file section
**Change**: Add at end of section:
> Phase 37 extends the sifr.toml schema from `internal_docs/sifr_workspace_design.md`. All Phase 37 fields are additive. Unknown top-level tables and unknown nested keys are accepted and ignored per the forward-compatibility rule in the workspace design. `[package].edition` in Phase 37 carries no cross-package enforcement in this phase; Sifr edition semantics (compatibility, migration, deprecation) are deferred to a future edition policy document.

### Patch 5: Add `sifr tree` behavior section

**File**: Phase 37 plan, CLI Contract section
**Change**: Add after the behavior paragraphs (after line 744):
> **`sifr tree`** displays the package dependency tree. `--sifr-only` shows only Sifr source packages. `--all` includes backend Rust crates from Cargo metadata. `--workspace` shows the Cargo workspace graph filtered by Sifr metadata. `--depth N` limits display depth. Output format is tree-style with package names, versions, and sources. Cycles are indicated with a marker rather than infinite recursion.

### Patch 6: Define `sifr outdated`

**File**: Phase 37 plan, milestone 37_5 scope
**Change**: Add to scope:
> - Implement `sifr outdated`: delegates to `cargo search` or registry index query for Sifr-owned packages, reports version delta against `Cargo.lock`. Reports only packages that are Sifr source packages (have `sifr.toml`). Reports semver diff and current Cargo.lock version.

### Patch 7: Clarify `sifr add --features` delegation

**File**: Phase 37 plan, CLI behavior section, `sifr add` bullet
**Change**: Replace:
> `sifr add <package> [--dev] [--features f1,f2] [--package member]` delegates the Cargo feature mutation where practical

With:
> `sifr add <package> [--dev] [--features f1,f2] [--package member]` always delegates package name and version to `cargo add`. For feature flags, delegates only when the named feature exists in the selected Cargo package's feature set. Features without a Cargo equivalent are written to `sifr.toml [features]` only.

### Patch 8: Add trust validation rule

**File**: Phase 37 plan, "Rust Backend And Trust" section
**Change**: After "Packages with native/backend behavior must explicitly declare trust" (line 807):
> Trust is validated only against direct Cargo dependencies declared in the package's `Cargo.toml`. Transitive trust is not inherited; each Sifr package must declare trust for its own direct backend dependencies. If a package declares trust for a crate that is not a direct dependency, `SIFR-PACKAGE-0304` is reported. If a backend crate is present but not declared in `[trust]`, `SIFR-PACKAGE-0301` applies.
> (Reserve `SIFR-PACKAGE-0303` for cross-package feature mapping failures, `SIFR-PACKAGE-0304` for trust-on-nonexistent dependency.)

### Patch 9: Add demo source file contents

**File**: Phase 37 plan, Organization Demo Repositories section
**Change**: Add minimal source file contents for `sifr/demo_http/client.sifr` and `sifr/demo_json/parse.sifr`. Example for parse.sifr:

```sifr
# https://github.com/sifr-lang/sifr-demo-json/blob/main/sifr/demo_json/parse.sifr

class JsonError(Error):
    message: str
    line: int
    column: int

def parse_json(text: str) -> Result[dict, JsonError]:
    # Implementation using serde_json backend
    ...
```

Or add a note: "Full module source content lives in the GitHub repos; this plan shows structural highlights and import paths."

### Patch 10: Clarify feature validation

**File**: Phase 37 plan, "Sifr Features And Cargo Features" section
**Change**: After "Backend-only optional Cargo features may remain Cargo-only" (line 284):
> At `sifr build --features` time, if `cargo-package` in a Sifr feature mapping does not appear in the resolved Cargo dependency graph, `SIFR-PACKAGE-0303` is reported. If the mapped `cargo-feature` does not exist in that package's feature set, `SIFR-PACKAGE-0304` is reported.

---

## Verdict

**ready-with-nits**

The plan is implementable as-is. The 10 nits above are clarification and completeness improvements, not structural fixes. An engineer reading this plan could start with milestone 37_1 immediately and work through all milestones without hitting a blocking gap. The nits should be fixed before the first implementation PR lands, not before starting work.