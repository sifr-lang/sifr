

Now I have full context: round 1 identified 10 gaps, the current phase doc (after round 1) has addressed most of them. Let me do a focused review against the current document state.

**Round 1 → Current gap analysis:**

| Round 1 recommendation | Status |
|---|---|
| Define `OperationPlan` explicitly | ✓ Done (lines 249-271) |
| `sifr_frontend` integration point | ✓ Done (lines 275-279) |
| Clarify diagnostics ownership | ✓ Done (line 238) |
| `check_package_manager_guardrails.py` | ✓ Referenced (lines 299-306) |
| Split `operations` into sub-modules | ✓ Done (lines 240-245) |
| `CargoLockDiff` categorization | ⚠ Partial — mentioned, not defined |
| Dependency audit file | ✓ Done (lines 292-295) |
| Feature expansion termination bound | ✓ Done (lines 287-290) |
| `gix` async design issue | ✓ Done (lines 296-297) |
| `FEATURES.md` document | ✗ Missing from phase doc |

---

## Review Verdict: 2 concrete blockers remain

The architecture is sound. The anti-corruption pattern, one-crate strategy, pure/IO split, `OperationPlan`, `PackageSourceMap` integration, and `sifr_diagnostics` integration are all correctly specified. These two items are actionable blockers before implementation:

### Blocker 1: `CargoLockDiff` is referenced but not defined

The phase doc says `backend::cargo_lock_verify` produces a `CargoLockDiff` and that it "categorizes differences" (line 283), but the struct itself is not defined anywhere. Without a concrete definition, implementers will invent it — likely inconsistently. This must be in the phase doc, not improvised.

**Required addition after line 285:**

```rust
struct CargoLockDiff {
    critical: Vec<CriticalDiff>,
    normalized: Vec<NormalizedDiff>,
}

enum CriticalDiff {
    PackageNameMismatch { expected: PackageName, actual: PackageName },
    VersionMismatch { expected: Version, actual: Version },
    SourceMismatch { expected: PackageSource, actual: PackageSource },
    ChecksumMismatch { expected: String, actual: String },
    FeaturesMismatch { package: PackageId, expected: BTreeSet<FeatureName>, actual: BTreeSet<FeatureName> },
    LinksMismatch { expected: Option<String>, actual: Option<String> },
    NativeCapabilityMismatch { package: PackageId, expected: NativeCapability, actual: NativeCapability },
    DependencyEdgeAdded { from: PackageId, to: PackageId },
    DependencyEdgeRemoved { from: PackageId, to: PackageId },
}

enum NormalizedDiff {
    OrderingOnly { reason: &'static str },
    TimestampOnly,
    OptionalFieldChanged { field: &'static str },
}
```

This is a blocker because `milestone_37_4` definition-of-done says "generated Cargo manifests and locks are deterministic and verified against `sifr.lock`" — without the struct, there's no contract for what "verified" means.

### Blocker 2: `crates/sifr_package/FEATURES.md` is documented in the maintainability section but the requirement is not in any milestone scope

Lines 292-295 document `FEATURES.md` as a required artifact, but `milestone_37_1` scope does not list it, and `milestone_37_7` scope (which covers documentation) does not mention it either. This creates a risk it gets dropped.

**Required addition to `milestone_37_1` scope:**

Add to the existing scope bullet list:
- Add `crates/sifr_package/FEATURES.md` with initial entries mapping each enabled external crate feature flag to its corresponding Sifr feature.

**Required addition to `milestone_37_7` scope:**
- Complete `crates/sifr_package/FEATURES.md` with the full feature flag map, rationale for each enabled flag, and documentation of any disabled flags that were considered.

### Minor optional polish (not blockers):

1. **`trust/` vs `backend/trust_bridge` relationship**: `Module map` shows both `trust/{model,validate}.rs` and `backend/{cargo_plan,cargo_lock_verify,trust_bridge}.rs`. The relationship between `trust::validate` and `backend::trust_bridge` is not explained. If `trust_bridge` is just the passthrough from `CargoBackendPlan` into trust validation, say so. If they have distinct responsibilities, document the boundary. This is minor because the code will clarify it, but explicit documentation prevents the wrong abstraction forming.

2. **`TRACEABILITY.md` format template**: The phase doc says the file exists and maps Cargo/uv categories to Sifr tests, but does not specify the table schema (columns: borrowed_category, sifr_test_file, milestone, diagnostic_code, intentional_divergence, non_port_decision). A template prevents inconsistent entries across implementers.

3. **`CargoBackendPlan` struct definition**: Referenced at line 829 as "produced by `backend::cargo_plan`" but not defined with field-by-field clarity. This is minor — the struct is clearly described in context, and implementers can derive it from the narrative.

---

**Summary**: The architecture is ready for implementation after adding the `CargoLockDiff` struct definition and placing `FEATURES.md` in a milestone scope. These are small, concrete additions that prevent downstream inconsistency. No architectural gaps remain.
