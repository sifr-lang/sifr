

Now I have a thorough understanding of both the plan and the existing Phase 37 implementation. Let me analyze the plan systematically.

---

## Review Summary: NOT READY — 12 Blockers Required Changes

The plan is architecturally sound in its goals but has critical gaps, contradictions, and underspecified areas that would cause serious implementation problems. Each blocker below requires a concrete change before implementation can proceed.

---

### BLOCKER 1: Layout Migration Destroys Package Identity

**Problem:** The migration command `sifr package migrate-layout --from sifr-rooted --to src-init` moves source from `sifr/` to `src/`, but the default `sifr.toml` `[source].root` is `"sifr"` (Phase 37 default), not `"src"`. Every migrated package would immediately have a broken source root after migration.

**Required change:** The `sifr.toml` migration must also rewrite `[source].root` from `"sifr"` to `"src"` or use the new default. Add this to the migration rules:

```
- Rewrite [source].root = "sifr" → "src" (or remove if src is the new default)
- Regenerate Cargo.toml include patterns from "sifr/**/*.sifr" to "src/**/*.sifr"
- Verify Cargo.lock source roots point to new paths
```

---

### BLOCKER 2: Two Competing API Models with No Hierarchy

**Problem:** The plan introduces `__init__.sifr`-derived public APIs but all Phase 37 packages still use `[exports] modules = [...]` from `sifr.toml`. The plan says manifest exports "becomes legacy compatibility, not the recommended public API model" but provides no decision rule for when both exist.

**Current state:**
- Phase 37 implementation (`manifest/sifr.rs:96-100`) derives exports from `[exports].modules` in `sifr.toml`
- The plan requires deriving exports from `__init__.sifr` re-exports for new packages
- Migration section says existing packages keep `[exports] modules = [...]`
- No conflict resolution rule exists

**Required change:** Add explicit precedence rules to the Public API section:

```
Conflict resolution:
- For packages with both __init__.sifr at source root AND [exports].modules:
  If __init__.sifr re-exports differ from [exports].modules → SIFR-PACKAGE-0701 diagnostic
  and require maintainer to resolve the conflict.
- [exports].modules is accepted for backward compatibility ONLY when __init__.sifr
  is absent from the source root.
- New packages created with sifr init must NOT generate [exports].modules.
```

---

### BLOCKER 3: Dependency Declaration Syntax — Two Models, No Migration Path

**Problem:** The plan describes two dependency declaration models with no bridge:

1. **Phase 37 model:** `Cargo.toml` + `[package.metadata.sifr.aliases]` for aliases
2. **Proposed model:** `sifr.toml` TOML with git/path/alias in `sifr.toml` directly

The plan shows the proposed syntax but never specifies:
- The `sifr.toml` schema for the new `[dependencies]` table
- Whether new syntax replaces `[package.metadata.sifr.aliases]` or coexists
- How `sifr add --alias name` maps to the new syntax
- What happens to existing `[package.metadata.sifr.aliases]` after migration

**Required change:** Add a dedicated section "Dependency Declaration Schema" specifying:

```
# sifr.toml new dependency schema
[dependencies]
<alias> = { package = "<cargo-name>", git = "...", tag = "...", version = "..." }
<alias> = { package = "<cargo-name>", path = "...", version = "..." }

[dev-dependencies]
<alias> = { package = "...", ... }

# Aliases map: alias name → Cargo dependency rename
# This REPLACES [package.metadata.sifr.aliases] for new packages
# Old [package.metadata.sifr.aliases] is parsed as: if sifr.toml [dependencies] absent,
# fall back to [package.metadata.sifr.aliases] for backward compat
```

---

### BLOCKER 4: `src/lib.rs` Marker — Creation Not Addressed

**Problem:** The plan references `src/lib.rs` as a "pure marker" but never specifies:
- When is `src/lib.rs` created? `sifr init --lib`?
- For the `src/` layout, should `sifr init --lib` create `src/lib.rs` automatically?
- What happens to `src/lib.rs` in the migration — does it need to be created from scratch?

The plan's generated Cargo projection shows a pure marker but the init command semantics don't include marker creation.

**Required change:** Add to the `sifr init` command semantics:

```
sifr init --lib <name>:
  - Creates Cargo.toml with Cargo package name sifr-<name>
  - Creates sifr.toml with [source].root = "src"
  - Creates src/__init__.sifr
  - Creates src/lib.rs with the canonical pure marker
  - Does NOT create [exports].modules
```

---

### BLOCKER 5: `sifr run` — Binary Target Resolution Underspecified

**Problem:** The plan says `sifr run` defaults to `src/main.sifr` when no `[[bin]]` target is configured, but:
- The existing Phase 37 demo uses `sifr/app/main.sifr` — how does this migrate?
- When Cargo builds a binary target, it looks for a `main.rs` or configured binary source
- The plan references `[[bin]]` configuration in `sifr.toml` but doesn't specify the schema

**Required change:** Add binary target resolution rules:

```
sifr run resolution order:
1. If --bin <name> provided → use [[bin]] path from sifr.toml or Cargo.toml
2. If src/main.sifr exists → compile as implicit binary target
3. If exactly one [[bin]] exists → use that
4. Otherwise → SIFR-PACKAGE-0605 (no binary target found)

# sifr.toml binary schema
[[bin]]
name = "demo-app"
path = "src/main.sifr"  # relative to package root
```

---

### BLOCKER 6: PackageSourceMap Privacy Rules — No `__init__.sifr` Re-export Parsing

**Problem:** The plan says public names are defined by `__init__.sifr` re-exports, but:
- The current `PackageSourceMap::build` in the implementation doesn't parse `__init__.sifr`
- `is_private_dependency_module` (line 302-318 of `source_map.rs`) checks the manifest's `[exports]` list, not `__init__.sifr` content
- The privacy enforcement currently uses `package.manifest.exports` — this is the Phase 37 model

The plan claims milestone_adhoc_pkg_1 will implement `__init__.sifr`-derived public APIs, but the current code path doesn't support this at all.

**Required change:** Add to milestone_adhoc_pkg_1 scope:

```
- PackageSourceMap must parse __init__.sifr for each public namespace
- Re-exported names are extracted from __init__.sifr import/from statements
- Privacy check uses: is this module's file path reachable through any
  __init__.sifr re-export chain from the package root?
- Implement: parse_init_sifr_reexports(__init__.sifr) -> BTreeSet<SifrName>
- Privacy rejection: if module not in derived re-export set → SIFR-PACKAGE-0203
```

---

### BLOCKER 7: Drift Diagnostics — Underspecified Action Surface

**Problem:** The plan says "Sifr-managed Cargo projection" adds "drift diagnostics when Cargo projection does not match Sifr package metadata" but provides no spec for:
- What constitutes drift (which fields?)
- Severity levels
- Recovery actions
- Interaction with frozen/locked modes

**Required change:** Add drift diagnostic specification:

```
Drift categories:
- PACKAGE-0701: Cargo package name != sifr.toml name (severity: error)
- PACKAGE-0702: Cargo include patterns omit sifr.toml or src/**/*.sifr (severity: error)
- PACKAGE-0703: [package.metadata.sifr] absent or points to wrong manifest (severity: error)
- PACKAGE-0704: Cargo edition != Rust 2024 (severity: warning)
- PACKAGE-0705: sifr.toml [source].root points to non-existent directory (severity: error)

Recovery: sifr fix --package <name> attempts to correct drift automatically
```

---

### BLOCKER 8: Multiple-Version Type Identity — Codegen Bridge Missing

**Problem:** The plan describes type identity mismatches across package instances (`SIFR-PACKAGE-0204`) but provides no spec for:
- How the codegen generates distinct types for aliased packages
- The generated Rust namespace for aliased packages (e.g., `demo_json_v1` vs `demo_json_v2`)
- How HIR type unification handles cross-instance values

Without this, multiple-version packages would generate type collisions.

**Required change:** Add to milestone_adhoc_pkg_5 scope:

```
Codegen namespace rules:
- Each package instance gets generated module: sifr_gen_<cargo-name>_<stable-hash>
- demo_json_v1 → sifr_gen_demo_json_v1_<hash_v1>
- demo_json_v2 → sifr_gen_demo_json_v2_<hash_v2>
- Hash is derived from Cargo package id + version + source (stable across builds)
- Cross-instance type mismatch: compile-time error, not runtime panic
- Type identity includes package instance in its qualified name
```

---

### BLOCKER 9: Package Session — `cargo_command_plan` Not Specified

**Problem:** `PackageSession` is a major architectural component but its `cargo_command_plan` field is underspecified:
- How is the plan computed from lock mode + feature selection + package selection?
- What is the schema for planned Cargo commands?
- How does the session handle command ordering for multi-package workspaces?

**Required change:** Add `OperationPlan` schema to the Package Session section:

```
cargo_command_plan: Vec<CargoCommand>
  where CargoCommand = {
    command: CargoSubcommand,  // metadata | fetch | build | test | package | publish
    targets: Vec<String>,      // package names or binary targets
    flags: LockModeFlags,      // locked | offline | frozen
    features: Vec<String>,     // feature flags
    order: TopologicalOrder,   // for multi-package
  }

OperationPlan validation:
- frozen + any write operation → reject before execution
- locked + network operation → reject before execution
- offline + unfetched source → SIFR-PACKAGE-0104 before planning
```

---

### BLOCKER 10: Workspace Semantics — Virtual Workspace + Sifr Root Collision

**Problem:** The plan says a virtual Cargo workspace root "has no Sifr package identity" and Sifr must not require `[package.metadata.sifr]` at a virtual workspace root. But:
- What if a user intentionally places `sifr.toml` at a virtual workspace root? (Edge case but possible)
- The plan doesn't specify whether a root `sifr.toml` in a virtual workspace is silently ignored or causes a diagnostic

**Required change:** Add to Workspace Semantics section:

```
Virtual workspace root + sifr.toml behavior:
- If root Cargo.toml is [workspace] with no [package] AND sifr.toml exists at root:
  → SIFR-PACKAGE-0706 warning: "sifr.toml at virtual workspace root has no effect;
     move it to a package member or convert the workspace root to a package."
- Root sifr.toml with [package.metadata.sifr].manifest pointing to itself → ignored
- No error for explicit user intent, just advisory warning
```

---

### BLOCKER 11: Migration Validation — No Verification Criteria

**Problem:** The migration section describes the command but provides no validation criteria:
- How to verify the migration was correct (no broken imports, no lost exports)
- What test to run to confirm the migrated package behaves identically to the original
- Rollback strategy if migration produces incorrect output

**Required change:** Add to migration plan:

```
Migration validation:
1. Before migration: snapshot public API (derived from __init__.sifr or [exports].modules)
2. After migration: verify same public API is re-derived from new location
3. Compile all imports within the package to confirm no broken local paths
4. If any check fails → report SIFR-PACKAGE-0707 with diff showing what changed

Rollback:
- Migration command writes backup: <package>.sifr-migration-backup.tar
- sifr package migrate-layout --rollback <backup.tar> restores original
```

---

### BLOCKER 12: Non-Goal Overlap with Phase 37

**Problem:** The plan's Non-Goals overlap with Phase 37's committed behavior:
- Phase 37 already has `sifr add`, `sifr remove`, `sifr update`, `sifr tree`, `sifr package`, `sifr publish` (from Phase 37 CLI contract)
- The plan re-specifies these commands with new semantics but doesn't call out what's CHANGED
- The plan's "Sifr-managed Cargo projection" implies `sifr add` behavior will change, but this isn't explicit

**Required change:** Add a "Changes from Phase 37" section that explicitly lists which commands change behavior:

```
Commands with changed behavior:
- sifr add: now projects to sifr.toml [dependencies] (Phase 37: projected to Cargo.toml)
- sifr init: now creates src/ layout (Phase 37: created sifr/ layout)
- sifr run: now defaults to src/main.sifr (Phase 37: required explicit [[bin]] or file arg)

Commands with unchanged behavior:
- sifr fetch, sifr tree, sifr package, sifr publish: Cargo delegation unchanged
```

---

### Additional Minor Issues (non-blocking but should be addressed):

1. **`sifr --explain` reference:** Plan references `sifr --explain <diagnostic-code>` but doesn't document the command's behavior. Should add minimal spec.

2. **Diagnostic code ranges:** The plan uses `SIFR-PACKAGE-0701` through `SIFR-PACKAGE-0707` for new diagnostics (blockers 7, 10, 11) but doesn't reserve these ranges explicitly. Should add to Diagnostics section: "Range 07xx reserved for adhoc package phase."

3. **Guardrail script updates:** `check_package_manager_guardrails.py` references `source.roots` layout and Phase 37 fixture matrix. After migration, these will need updates to support the new `src/` layout. Should note in milestone_adhoc_pkg_7 that guardrail script must be extended for new layout model.

4. **Demo repository update:** All existing demo repos use `sifr/` layout. The plan says "update demo repositories only after the package-aware compiler supports the new layout." The milestone_adhoc_pkg_7 acceptance criteria includes updating demos, but no test criteria specify the transition validation.

---

### What IS Well-Specified:

- Privacy rules (`__init__.sifr` visibility, namespace rules)
- Package layout targets (src/ layout for lib/app/Rust-backed)
- Lock mode semantics (--locked, --offline, --frozen)
- Diagnostic code architecture and redaction requirements
- Milestone order (layout → projection → session → compiler → run → workspaces → publish)
- Phase 37 backward compatibility strategy (explicit opt-in via `source.roots = ["sifr"]`)

---

### Verdict

**NOT READY.** The plan has a strong architectural direction but 12 critical blockers prevent implementation. The most severe is **BLOCKER 2** (two competing API models) — without resolved precedence rules, any package with both `__init__.sifr` and `[exports].modules` will produce nondeterministic behavior. **BLOCKER 3** (dependency syntax) is equally severe since the entire Sifr-managed Cargo projection depends on a spec that doesn't exist.

Each blocker requires a concrete section addition with schemas, decision rules, or command semantics before the plan can move to implementation.
