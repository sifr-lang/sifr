

# Phase 37 v2 Package Management Review

## Overall Assessment

**Viable with concrete edits.** The design correctly leverages uv and Cargo as first-class tools rather than building a redundant Sifr lockfile. The ownership model, adapter strategy, and milestone decomposition are well-structured. Three concrete blockers require resolution before implementation proceeds; everything else is refinable during implementation.

---

## Critical Blockers

### 1. Source Discovery Mechanism Is Underspecified

The discovery flow (lines 420-437) assumes `sifr.toml` can be loaded from each dependency, but provides no concrete mechanism for how.

**What's stated:**
- "Ask uv adapter for the selected distribution graph" (line 427)
- "Filter selected distributions to those with Sifr metadata" (line 428)
- "Load each dependency's `sifr.toml`" (line 429)

**What's missing:**
- How to locate `sifr.toml` inside a wheel archive (extract? uv API?)
- How to locate `sifr.toml` inside an sdist (unpack? uv API?)
- How to locate `sifr.toml` for editable/path/Git dependencies (path reference? clone?)
- Fallback plan when `sifr.toml` is absent from a distribution

**Impact:** milestone_37_2 cannot be implemented without this. Frozen mode is completely broken since there is no defined mechanism to retrieve source from cached/distributed packages.

**Required doc change:** Add a "Source Discovery" section under Package Discovery Flow that specifies:
1. The uv adapter must expose a `get_distribution_artifact(package_id) -> DistributionArtifact` interface
2. `DistributionArtifact` variants: `Wheel { archive_reader }, Sdist { archive_reader }, Path { root }, Git { url, ref }, WorkspaceMember { root }`
3. For each variant, how `sifr.toml` is located relative to the artifact root
4. Error code `SIFR-PACKAGE-0104: cannot locate sifr.toml in distribution` with origin data

---

### 2. Reproducibility Without sifr.lock Is Unaddressed

The doc says "no committed `sifr.lock` is required" (line 995), but does not address reproducibility across machines or time.

**Problem:** The derived `SifrPackageGraph` depends on:
- `uv.lock` (committed)
- `Cargo.lock` (committed when backends exist)
- Dependency `sifr.toml` files inside distributions

If any cached distribution's `sifr.toml` changes (e.g., a new version is released), `sifr build --frozen` on machine A and machine B can produce different compilation results even with identical lockfiles. The doc explicitly says "package graph derivation must be deterministic" (line 980) but does not provide the mechanism.

**Required doc change:** Add to "Derived Sifr Package Graph" section:
```
The derived graph must be reproducible. Sifr computes a lock digest from:
- SHA-256 of `uv.lock`
- SHA-256 of each relevant `Cargo.lock`
- SHA-256 of each dependency's `sifr.toml` content (fetched from installed/cached distributions)

This digest is emitted to `target/sifr/graph-digest.json` and validated in frozen/locked modes.
```
Without this, `--frozen` reproducibility is only guaranteed for the specific cache state on the machine that produced the build.

---

### 3. uv Library API Feasibility Is Assumed

The adapter table (lines 326-336) lists uv crates (`uv-workspace`, `uv-lock`, `uv-client`, etc.) as reuse targets. The fallback rule (lines 288-293) says to write Sifr-native code when uv APIs aren't stable enough.

**Problem:** uv is primarily a CLI tool. The `uv` crates on crates.io (`uv-workspace`, `uv-lock`, etc.) exist but are internal implementation details that change frequently. The v0.5+ releases are not stable library APIs — they are used internally by the uv binary.

The doc says "Sifr may call uv through library APIs where available" (line 623), which correctly hedges, but does not commit to a concrete approach.

**Impact:** milestone_37_2 ("Implement uv-backed adapter for project/workspace discovery") is blocked on this decision. If uv library APIs are not usable, the entire adapter strategy defaults to shelling out to `uv` CLI, which the doc allows but does not plan for explicitly.

**Required doc change:** Add to "uv Integration Strategy":
```
uv API consumption strategy (choose one per adapter):

Option A (library-first): Use uv crates directly where stable. Document each crate version pinned in DEPENDENCY_AUDIT.md. Accept API breakage risk with version-locked adaptation.

Option B (CLI-first): Shell out to `uv` binary behind adapter. Parse structured JSON output. This is the safest path for Phase 37 since uv CLI behavior is stable and output is well-defined.

Phase 37 defaults to Option B for all adapters initially, with Option A evaluated per-adapter in Phase 38 if uv releases stable library APIs.

Shelling out to uv CLI is acceptable behind the adapter boundary as long as:
- structured output (JSON) is captured
- errors are mapped to diagnostic codes (not raw stderr)
- subprocess is killable for cancellation
- timeout behavior is defined per command
```
This is not a redesign — it makes explicit what the doc already implies but doesn't state.

---

## Important Issues

### 4. python-interop Field Duplication

`[tool.sifr].python-interop = true` in `pyproject.toml` (line 168) and `[python].interop = true` in `sifr.toml` (line 587) both express the same concept. This violates the single-source-of-truth rule (lines 212-248).

**Required doc change:** Pick one. Recommend keeping `[python].interop` in `sifr.toml` since the python-interop flag affects Sifr semantics (import resolution, code generation), not Python packaging. Remove `[tool.sifr].python-interop` from all examples.

---

### 5. `[distribution].distribution-name` Is Redundant

`sifr.toml` has `[distribution].distribution-name = "sifr-http"` (line 101) while `pyproject.toml` has `name = "sifr-http"`. These must match by construction, making the `distribution-name` field redundant and a sync hazard.

**Required doc change:** Remove `distribution-name` from `sifr.toml`. Sifr derives the distribution name from the linked `pyproject.toml`. Add to "Sifr-owned fields": "distribution name is derived from linked pyproject.toml and need not be duplicated."

---

### 6. Cargo Offline Is Underdefined

The command modes table (lines 627-633) says `--frozen` uses "Cargo locked/offline behavior" and `--offline` uses "Cargo offline where configured." Neither specifies what "offline where configured" means or what happens when a backend `Cargo.toml` exists but `Cargo.lock` does not.

**Required doc change:** In "Cargo Integration Strategy":
```
`sifr build --frozen` with backend manifests:
- Requires `Cargo.lock` to exist for each backend manifest
- Invokes `cargo build --locked` (no network, no lock update)
- If `Cargo.lock` is absent: fail with SIFR-PACKAGE-0301

`sifr build --offline` with backend manifests:
- Uses `cargo build --offline` when Cargo.lock exists
- Uses `cargo build` (may fetch missing deps) when Cargo.lock is absent, only if the project explicitly opts in to offline-optional mode
```
---

### 7. Editable/Path Dependencies Source Mutation Is Unaddressed

For path and editable dependencies, `sifr.toml` is read directly from the filesystem. The derived graph can change between compilations if source files change, without any lockfile update.

**Required doc change:** Add to "Package Discovery Flow":
```
For path/Git/editable dependencies, sifr computes a content hash of all `.sifr` files under `[source].roots` at graph derivation time. This hash is included in the lock digest.
```

---

### 8. Trust Policy Defaults Are Anti-Ergonomic

The trust policy example (lines 107-111, 540-542) uses empty arrays:
```toml
[trust]
native = []
build-scripts = []
proc-macros = []
```

This means **no** native Rust, build scripts, or proc macros are trusted by default. A package with a Rust backend using this policy would fail to compile.

**Required doc change:** Either:
1. Document this as intentional: "Empty trust arrays mean no backend Rust is trusted by default; packages must explicitly opt in"
2. Or change the example to show realistic trust values

Recommend option 1 with a note: "Empty arrays are intentional safety defaults. Packages with backends must declare which crates are trusted."

---

## Minor Issues

### 9. Missing Dependency Discovery for Sifr Packages

The Single Source of Truth rules (lines 214-241) don't explicitly cover how Sifr discovers that a Python distribution contains Sifr source. The doc says "Filter selected distributions to those with Sifr metadata" (line 428), but `uv.lock` does not contain this information.

**Required doc change:** Add to "uv.lock" section:
```
Sifr packages are identified by the presence of `[tool.sifr]` in their pyproject.toml (for workspace/path/Git members) or by the presence of a `sifr.toml` alongside pyproject.toml in the distribution archive (for installed distributions). The uv adapter must inspect distribution metadata to detect Sifr packages.
```

### 10. Git Dependency Changed-File Detection Is Gix-Dependent

The Turborepo `[base...head]` selector (line 698) reuses `gix` behind an adapter. This is fine as an implementation detail, but the doc doesn't acknowledge that `gix` is a specific implementation choice with its own maintenance surface.

**Required doc change:** In the selector table, add: "Changed-file detection may use `gix` or equivalent Git library behind the adapter. Fallback to `git` CLI when gix is unavailable."

### 11. `[distribution].manager = "uv"` In sifr.toml Is Confusing

`sifr.toml` has `[distribution].manager = "uv"` (line 99) and `[backend].manager = "cargo"` (line 104). The distribution manager field implies Sifr could use a different distribution manager, but the entire architecture is built on uv. Having this field suggests flexibility that doesn't exist.

**Required doc change:** Either remove `[distribution].manager` from `sifr.toml` (since distribution management is owned by pyproject.toml/uv), or document it as reserved for future use with explicit "only `uv` is supported in Phase 37."

---

## What Works Well

1. **Ownership model is correct.** The sifr.toml/pyproject.toml/Cargo.toml division is clean and avoids the dual-config synchronization problem that plagued v1.

2. **Adapter architecture is sound.** The rule "no uv types cross the public facade" (line 791) is the right boundary. The allowed/disallowed type lists (lines 297-322) are concrete and enforceable via guardrails.

3. **OperationPlan is well-designed.** Centralizing all mutating operations through a plan (lines 644-676) before execution is correct. The dry-run mechanism is clean.

4. **Module map is appropriately decomposed.** The 9-module structure under `sifr_package` (lines 769-782) is not monolithic and maps cleanly to milestones.

5. **Guardrails are enforceable.** The `check_package_manager_guardrails.py` checklist (lines 805-816) is implementable and will catch most boundary violations.

6. **Milestones are implementable in sequence.** Each milestone builds on the previous without requiring future design decisions. The dependency chain is linear.

7. **Diagnostic codes are appropriately scoped.** The code ranges (000x for linkage, 010x for uv, 020x for imports, 030x for Cargo, 040x for packaging) are well-organized and leave room for expansion.

---

## Required Doc Changes (Summary)

| # | Location | Change |
|---|----------|--------|
| 1 | Package Discovery Flow | Add concrete Source Discovery section with `DistributionArtifact` types |
| 2 | Derived Sifr Package Graph | Add lock digest computation for reproducibility |
| 3 | uv Integration Strategy | Add explicit uv API consumption strategy (CLI-first default) |
| 4 | All examples | Remove `[tool.sifr].python-interop`, keep only `[python].interop` in sifr.toml |
| 5 | sifr.toml examples | Remove `distribution-name` field, derive from pyproject.toml |
| 6 | Cargo Integration Strategy | Clarify `--frozen`/`--offline` Cargo behavior for missing Cargo.lock |
| 7 | Package Discovery Flow | Add content hash for path/Git/editable source at derivation time |
| 8 | sifr.toml trust example | Document empty arrays as intentional safety defaults |
| 9 | uv.lock section | Document how Sifr packages are detected from distributions |
| 10 | Workspaces section | Acknowledge `gix` as implementation detail with CLI fallback |

---

## Blockers Summary

| Blocker | Severity | Blocks |
|---------|----------|--------|
| Source discovery mechanism | Critical | milestone_37_2, frozen mode |
| Reproducibility guarantee | Critical | exit gate condition 2 |
| uv API feasibility | Critical | milestone_37_2 implementation approach |
| python-interop duplication | Important | single-source-of-truth rule |
| Cargo offline definition | Important | `--frozen` correctness |

---

## Recommendation

**Proceed to implementation after addressing blockers 1-3.** The architecture is sound and the milestones are implementable. Blockers 1 and 2 require additions to the doc. Blocker 3 requires choosing between library-first and CLI-first uv consumption — CLI-first is the safer, more conservative choice that the doc already implies. Blocks 4-8 are refinements that can be made in parallel or during implementation.

If uv releases stable library APIs before Phase 38, Option A can be evaluated at that time without architectural changes.
