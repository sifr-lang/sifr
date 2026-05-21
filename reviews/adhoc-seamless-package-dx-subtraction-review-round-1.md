

No SIFR-PACKAGE-07xx codes exist in the codebase yet — they're pure specification. Let me now write the subtraction review.

---

## Subtraction Review: Adhoc Seamless Package DX

**1. Verdict: READY AFTER SUBTRACTION**

The design has solid foundations (src layout, `__init__.sifr` API, Cargo projection, PackageSession) but carries significant v1-inappropriate complexity that should be cut before implementation. The removed items can return in v2+ once the core is stable.

---

**2. Remove Now**

| Item | Rationale |
|---|---|
| **`[scripts]`** | Full npm-scripts equivalent adds: disambiguation with app target names, namespace collision rules, `--script`/`--bin` disambiguators, expansion visibility, and schema for `command` + `args`. For v1, this is a nice-to-have that competes for implementation bandwidth with the core package graph and CLI wiring. Users who need workflow aliases can use shell aliases or a `Justfile`. Remove `[scripts]` from v1. Keep `sifr run <name>` for app targets and `sifr run --bin <name>` for explicit targeting. |
| **Custom dependency groups (`[dependency-groups.<name>]`, `include = [...]` composition)** | The `include` field for group composition is explicitly modeled after uv's group composition. For v1, it is premature. Three dependency tiers (`runtime`, `test`, `dev`) plus the base `[dependencies]` table are sufficient. Custom groups can be added in v2 when real-world grouping needs are better understood. The `[dependency-groups.<name>.dependencies]` table shape should be deferred, not just the `include` composition. |
| **All 5 group CLI flags (`--group`, `--only-group`, `--no-group`, `--all-groups`, `--no-default-groups`)** | Five flags for v1 group selection is over-engineered. For v1, implement only `--group <name>` (include named group alongside defaults) and `--no-default-groups` (exclude default groups). `sifr test` defaults to `runtime + test + dev`. `sifr run`/`check`/`build` default to `runtime` only. This is the minimal surface. |
| **`[package].default-run`** | Only needed when multiple app targets exist AND the user doesn't want `src/main.sifr` as the default. This is a very narrow edge case. The resolution order already picks `src/main.sifr` first, then single discovered targets, then reports ambiguity. In v1, just require `--bin <name>` when multiple targets exist. Users who want a persistent default can put their entry point at `src/main.sifr`. This field can return in v2. |
| **`--message-format json`** | JSON output adds schema maintenance burden, versioning obligation, and test surface for the entire output contract. For v1, human-readable output is sufficient. JSON output is a v2 extension when the schema is stable and the demand is proven. |
| **`SIFR-PACKAGE-0701` (manifest `[exports].modules` used)** | This is a Phase 37 → adhoc migration diagnostic. Since Phase 37 layouts are explicitly treated as "internal implementation fixtures" (issue line 196), the migration path already handles this. `0701` as a production diagnostic for v1 packages using old schema is appropriate, but it doesn't need to be a separate high-priority milestone feature — it surfaces naturally when the migration command runs. Keep it, but deprioritize its milestone placement. |
| **`SIFR-PACKAGE-0707` (layout migration validation failed)** | Layout migration (`sifr package migrate-layout`) is milestone 7 territory. The migration validation check (API diff) is an internal safeguard. For v1, the migration is documented as manual or a simple script, not a CLI command with rollback. This diagnostic belongs to milestone 7, not milestone 1. Remove from v1 scope, add when migration is implemented. |
| **`SIFR-PACKAGE-0708` (Cargo alias metadata conflicts with sifr.toml)** | This diagnoses conflicts between Phase 37 `Cargo.toml [package.metadata.sifr.aliases]` and the new `sifr.toml [dependencies]` projection. Since Phase 37 alias metadata is explicitly "internal transitional data only" (issue line 489), and the issue already says Sifr "may delete or rewrite it when regenerating package projections," `0708` is an internal migration diagnostic, not a user-facing v1 feature. Remove from v1. The conflict is resolved by the projection regeneration itself. |
| **`src/bin/tools/migrate.sifr -> "tools/migrate"` nested naming** | Deeply nested `src/bin/` paths with dot-path naming ("tools/migrate") add parsing complexity and potential edge cases around path separators and name normalization. For v1, restrict `src/bin/*.sifr` to flat names only: `src/bin/admin.sifr` -> "admin". Nested directories can be added in v2 with explicit path mapping rules. |
| **`[package.metadata.sifr]` fallback scanning** | The "discovery-only compatibility path" (issue line 581) that falls back to scanning for `sifr.toml` if Cargo stops surfacing `package.metadata` is a Cargo API stability hedge. This is an internal adapter detail, not a user-facing feature. Document it in `DEPENDENCY_AUDIT.md` as a known risk, but remove it from the issue spec — it belongs in implementation notes, not user-facing design. |
| **`sifr --explain <diagnostic-code>`** | The explain command adds a full diagnostic documentation system, help text generation, and recovery hint integration for every diagnostic code. For v1, this can be a docs-only reference (`docs/errors/`) rather than a runtime command. A `sifr explain` command can be added in v2 when there are enough stable diagnostics to justify the surface. |

---

**3. Defer**

| Item | Simplest v1 Replacement |
|---|---|
| **`[scripts]`** | Shell aliases or a `Justfile`. V2 can add `[scripts]` when real workflow alias patterns are identified from real usage. |
| **Custom dependency groups** | Two groups only: runtime + (test or dev). V2 can add `[dependency-groups.<name>]` when a concrete use case justifies the schema. |
| **`[package].default-run`** | Require `--bin <name>` when multiple targets exist. V2 can add `default-run` when the multi-target UX is proven. |
| **`--message-format json`** | Human-readable output only. V2 can add `--message-format json` once the stable JSON schema is defined from real v1 output patterns. |
| **`SIFR-PACKAGE-0707`** | Migration is manual/docs-only in v1. V2 adds the migration command and `0707`. |
| **Nested `src/bin/` paths** | Flat `src/bin/*.sifr` only. V2 adds `src/bin/<dir>/<name>.sifr -> "dir/name"` mapping. |

---

**4. Keep**

| Item | Rationale |
|---|---|
| **`src/` layout and `__init__.sifr` public API** | The core layout change from Phase 37's `sifr/<package>/` to `src/`. Essential. |
| **`src/main.sifr` and `src/bin/*.sifr` (flat only)** | Layout-discovered targets are the right model. Flat `src/bin/*.sifr` is the minimal surface. |
| **Manifest-less explicit file mode** | Explicitly called out as a first-class path (issue line 28, 261-268). No `sifr.toml` needed for `sifr run main.sifr`. Essential for the learning/scripting path. |
| **`[dependencies]`, `[test-dependencies]`, `[dev-dependencies]`** | Three-tier dependency model. `[test-dependencies]` and `[dev-dependencies]` both project to Cargo `[dev-dependencies]` (issue line 497) — consider whether two separate tables are worth it, but they're already specified and map cleanly to Cargo. |
| **`[package.metadata.sifr] manifest = "sifr.toml"` pointer** | Cargo→Sifr discovery hook. Required for `cargo metadata`-discovered packages. Keep. |
| **`PackageSession` and `OperationPlan`** | Single orchestration layer for all package-aware commands. Essential architecture. |
| **`SIFR-PACKAGE-0702` (projection drift)** | Essential for managed Cargo projection correctness. |
| **`SIFR-PACKAGE-0703` (missing/incorrect `package.metadata.sifr`)** | Essential for discovery validation. |
| **`SIFR-PACKAGE-0704` (Cargo include/exclude omits Sifr files)** | Essential for packaging/publishing preflight. |
| **`SIFR-PACKAGE-0705` (invalid source root)** | Basic validation. Essential. |
| **`SIFR-PACKAGE-0706` (virtual workspace root sifr.toml warning)** | Low-cost safety check. Keep. |
| **`SIFR-PACKAGE-0709` (pure marker missing, user-owned Rust prevents regeneration)** | Critical safety: prevents silently converting pure→Rust-backed. Keep. |
| **`SIFR-PACKAGE-0710` (explicit .sifr file outside source root)** | Enforces the manifest-less vs package-aware boundary. Keep. |
| **`SIFR-PACKAGE-0711` (production sifr.toml uses `[[bin]]` tables)** | Enforces the layout-discovery policy. Keep. |
| **`SIFR-PACKAGE-0101` Cargo wrapper** | Single stable wrapper for all Cargo failures. Essential. |
| **Redaction for credentials** | Essential for publish/auth diagnostics. |
| **`--locked`, `--offline`, `--frozen`** | Core lock mode semantics from Phase 37. Keep exactly as specified. |
| **`sifr add/remove/update`** | Essential dependency management. |
| **`sifr fix --check`** | Essential for projection drift detection. Keep. |
| **Trust policy and Rust-backed validation** | Core safety model. Keep. |
| **Type identity across package instances (`SIFR-PACKAGE-0204`)** | Core correctness guarantee. Keep. |
| **`sifr init --lib/--bin`** | Essential package creation. Keep. |

---

**5. Exact Edits Recommended for the Issue**

The following specific deletions and modifications will bring the design to a viable v1 scope:

**A. Remove from Non-Goals (lines ~50-58):**
- Delete bullet: "Do not add npm-compatible arbitrary shell scripts in this phase. Sifr may support named workflow aliases, but they must expand to Sifr command plans or structured executable argv, not unparsed shell strings."  
  *Replace with*: "Scripts and workflow aliases are deferred to v2. Shell aliases or project-specific tools like Justfile cover the common cases in the interim."

**B. Remove from `sifr.toml` Contract (lines ~152-198):**
- Delete lines ~183-187: `[package].default-run`, `[scripts]` entry and rules.
- Delete lines ~242-260: Script aliases section (script rules, namespace sharing with app targets, `--script`/`--bin` disambiguators).
- Delete `[scripts]` from the minimal app example (lines ~174-177).

**C. Remove from Dependency Model (lines ~414-473):**
- Delete lines ~456-461: Custom dependency groups and `include` composition.
- Replace lines ~451-455 (dependency groups) with a simpler v1 model:

> **Dependency groups (v1):**
> - `[dependencies]` is runtime, included by `run`, `check`, `build`, `test`, `package`, `publish`.
> - `[test-dependencies]` is test-only, included by `test` by default.
> - `[dev-dependencies]` is local development, included by `test` by default, not by `run`, `build`, `package`, `publish`.
> - Custom groups are deferred to v2.

**D. Remove from CLI Commands (lines ~611-636):**
- Delete `--script` from `sifr run` CLI signature (line ~616).
- Delete `[scripts]` from the run resolution order (line ~235).
- Replace `sifr test` CLI signature (line ~619): remove `--group name|--only-group name|--no-group name|--all-groups|--no-default-groups`. Replace with only: `--group name` (include named group) and `--no-default-groups` (exclude test/dev from test run).
- Add a note: "Additional group flags (`--only-group`, `--no-group`, `--all-groups`) are v2 extensions."

**E. Remove from Diagnostics (lines ~831-960):**
- Delete `SIFR-PACKAGE-0707` and its description (lines ~949-950, ~996-997).
- Delete `SIFR-PACKAGE-0708` and its description (lines ~951-952, ~997-998).
- Remove lines ~577-581 (fallback scanning description) — keep the fallback as an internal adapter note, not a user-facing feature.

**F. Remove from Internal Fixture Migration Plan (lines ~961-1010):**
- Delete or mark as v2: rollback descriptor system, `--apply-partial`, `.sifr/migrations/` directory structure. Replace with: "In v1, layout migration is documented as a manual procedure or simple scripted transformation. A dedicated `sifr package migrate-layout` command with rollback is a v2 feature."

**G. Remove from Milestones:**
- In `milestone_adhoc_pkg_1` (lines ~1030-1054): Delete "script expansion" from scope.
- In `milestone_adhoc_pkg_3` (lines ~1077-1102): Delete `--message-format json` from scope. Delete script expansion. Simplify group flags to the two supported ones.

**H. Update the implementation order (lines ~1187-1199):**
- Script aliases step can be removed entirely since `[scripts]` is deferred.

**I. Add a v2 backlog section:**

> ## Deferred to v2
>
> - `[scripts]` named workflow aliases
> - Custom dependency groups (`[dependency-groups.<name>]`) and group composition
> - `[package].default-run`
> - `--message-format json`
> - `sifr --explain <diagnostic-code>`
> - `sifr package migrate-layout` with rollback
> - Nested `src/bin/<dir>/<name>.sifr` paths
> - `--only-group`, `--no-group`, `--all-groups` group flags

---

**Summary of cuts:**

| Category | Removed |
|---|---|
| Schema fields | `default-run`, `[scripts]`, `[dependency-groups.*]`, `include` |
| CLI flags | `--script`, `--message-format json`, 3 of 5 group flags |
| Diagnostics | 0707, 0708 (2 of 12) |
| Target features | Nested bin paths |
| Commands | `sifr --explain` |
| Migration features | Rollback system |

**What remains is:** src layout, `__init__.sifr` API, flat `src/bin/*.sifr`, manifest-less mode, three dependency tiers, two group CLI flags, `PackageSession`, Cargo projection with drift detection, lock modes, trust policy, essential diagnostics, and the 10 remaining 07xx codes. This is a solid v1.
