# Architecture Review: adhoc-seamless-package-dx.md (Round 1)

**Reviewer:** agent architecture review
**Date:** 2026-05-20
**Document:** `issues/adhoc-seamless-package-dx.md`
**Implementation state:** Phase 37 complete, adhoc phase spec exists, implementation not started
**Review scope:** Round 1 — spec completeness, implementation readiness, blockers

---

## Executive Summary

**STATUS: NOT READY — 18 concrete blockers and 7 meaningful gaps requiring spec changes before implementation can proceed.**

The plan is well-structured and avoids Cargo complexity inheritantly, but it assumes implementation state that does not exist and makes schema decisions that conflict with Phase 37. Key blockers: (1) manifest schema change is underspecified, (2) `__init__.sifr` API derivation has no implementation path, (3) Cargo failure boundary conflates redaction with error classification, (4) credential code retirement plan is vague, (5) dependency schema is underspecified for the projection model, and (6) migration command overlaps with Phase 37 guardrails in ways that need explicit carve-outs.

---

## 1. User Experience and Command Semantics

### 1.1 Command Surface

The command surface is well-defined with clear lock/network mode semantics. `sifr run` without flags may fetch and update — this matches Rust's local development model and is the right UX.

**Blocker 1:** `sifr init --lib` / `sifr init --bin` semantics are underspecified for the case where the target directory already exists with content. Does it fail, prompt, or overwrite? Phase 37's `sifr init` behavior (if any) needs to be checked and either documented or spec'd.

**Blocker 2:** `sifr fix --package <name> --check` is described but `--check` vs. without `--check` behavior is ambiguous. Does `--check` error on drift without writing? Does `sifr fix` without `--check` require `--frozen` to reject mutation? The relationship between `sifr fix`, lock modes, and projection drift needs explicit spec.

**Blocker 3:** `sifr --explain <code>` for non-package diagnostics (e.g., `SIFR-TYPE-0001`) may reference package manager recovery commands that are not relevant. The spec should clarify that `--explain` only includes package-manager-specific recovery when the code is in `SIFR-PACKAGE-*`.

### 1.2 Lock/Network Mode Semantics

The table on page 9-10 is clear. One gap:

**Blocker 4:** Projection drift check timing is specified ("before Cargo command execution") but the behavior when drift is detected AND the command requires network is not spec'd. Does it fail before trying to run Cargo? Does it proceed and let Cargo fail? The spec should say "projection drift always fails the command before invoking Cargo."

---

## 2. Package Layout and `__init__.sifr` API Model

### 2.1 Target Layout

The `src/` layout is correct and simpler than Phase 37's `sifr/<package>/` model. One structural issue:

**Blocker 5:** The spec says `src/lib.rs` is "a Cargo target" but for pure Sifr packages the marker must be "pure" (comment-only). This is already in Phase 37 (`SIFR-PACKAGE-0501`). However, the spec does not address what happens when `sifr init --lib` creates a pure marker and the user later adds Rust code manually. Does the marker auto-become non-pure? Does Sifr re-validate on every package-aware command? The lifecycle of the pure marker after `sifr init` needs explicit spec.

**Gap 1:** The spec says `src/__init__.sifr` derives the public API from re-exports, but does not define what happens when `src/__init__.sifr` contains code that is NOT a re-export (e.g., function definitions, class definitions, type aliases). Are those also public? Or must everything be re-exported? The Phase 37 `[exports].modules` model exports everything under the module. The Python `__init__.sifr` model can export arbitrary names. The spec should clarify:

```sifr
# Is this valid public API?
# src/__init__.sifr
type MyAlias = dict[str, int]

class PublicClass:
    pass

def public_factory() -> PublicClass:
    ...
```

### 2.2 `__init__.sifr` Semantics

**Blocker 6:** The spec says `parse_init_sifr_reexports` extracts public names from `from .module import name` and `from .namespace import name` forms. But it does not define:
- What happens with `import module` inside `__init__.sifr` — is the module itself public?
- What happens with wildcard imports (e.g., `from .utils import *`) — are they rejected (Phase 37 says yes) or allowed for `__init__.sifr`?
- Can `__init__.sifr` define a function `foo` and export it via `foo = foo` or via re-export?

The implementation path for `PackageSourceMap` needs the spec to define a concrete API extraction algorithm, not just "derived from `__init__.sifr`".

**Blocker 7:** `parse_init_sifr_reexports` is mentioned in the implementation requirements but never defined in the spec. What is the grammar? What import forms are supported? Only `from .X import Y`? What about `from . import X`? What about `import .X as Y`? This is critical for the public API model.

**Blocker 8:** Privacy checks use "derived namespace API graph, not filesystem presence alone." But the spec also says "a public namespace path is valid only when every namespace segment is represented by a directory with `__init__.sifr`." These two statements conflict when a public namespace has no `__init__.sifr` but contains implementation files. The spec should say: filesystem presence confirms directory shape; namespace API graph confirms what's exported. A directory without `__init__.sifr` is not a public namespace regardless of its contents.

---

## 3. Dependency Schema and Managed Cargo Projection

### 3.1 Dependency Declaration Schema

**Blocker 9:** The new `[dependencies]` schema in `sifr.toml` is underspecified in critical ways:

1. **Field compatibility with Phase 37 alias model:** Phase 37 uses `[package.metadata.sifr.aliases]` in `Cargo.toml` for the authoritative alias mapping. The spec says new packages use `[dependencies]` in `sifr.toml`, but it does not specify how `sifr add --alias name` interacts with this. Does it write to `sifr.toml [dependencies]` or `Cargo.toml [package.metadata.sifr.aliases]`? The spec says "writes the table key `name`" but does not say which file.

2. **Projection direction is underspecified:** The spec says "Sifr updates Sifr-facing dependency declarations first, then projects to Cargo dependencies." But for a newly created package, does `sifr.toml` contain all dependency declarations and `Cargo.toml` is fully derived? Or does `sifr.toml` contain only Sifr-specific policy and Cargo继续保持? The generated Cargo projection example shows the full Cargo package metadata including `include` patterns, which implies Sifr generates the full `Cargo.toml`. This conflicts with Phase 37's model where `Cargo.toml` is user-authored.

3. **User-owned Cargo sections are not spec'd:** "Sifr validates they do not conflict with the package graph." What does "do not conflict" mean precisely? If a user manually adds a Cargo dependency that is not in `sifr.toml [dependencies]`, does Sifr error, warn, or silently ignore it? The boundary between Sifr-managed and user-owned sections needs explicit rules.

4. **Bidirectional sync is not spec'd:** If a user runs `cargo add` directly (bypassing `sifr add`), does `sifr.toml [dependencies]` get updated? If not, drift accumulates. If yes, Sifr must parse Cargo.toml. The spec should either forbid direct Cargo manipulation (only `sifr add/remove/update`) or spec the sync direction.

### 3.2 Cargo Projection Model

**Blocker 10:** The drift diagnostic `SIFR-PACKAGE-0702` ("projected Cargo dependency or alias differs from `sifr.toml` dependency declaration") is ambiguous. "Differs" could mean:
- Cargo has an extra dependency not in `sifr.toml`
- `sifr.toml` has an extra dependency not in Cargo
- The same dependency has different version/git/path specs

The spec should distinguish these three cases with different diagnostics. A missing Cargo dependency is a failure. An extra user-owned Cargo section is acceptable (Phase 37 non-goal: "Sifr does not validate that all Cargo dependencies are declared in `sifr.toml`").

**Blocker 11:** "Sifr never shells into Cargo internals or links to Cargo private APIs" is already in Phase 37. But the adhoc spec's new projection model requires Sifr to generate `Cargo.toml` from `sifr.toml`. This means Sifr must write Cargo manifests, not just read them. The spec's drift recovery (`sifr fix --package <name>`) implies manifest mutation. This is a significant new capability that Phase 37 did not include. The spec should explicitly document: "Sifr generates and maintains its Sifr-owned Cargo manifest sections. User-owned Cargo sections are preserved but validated."

---

## 4. Compatibility with Phase 37 and Migration

### 4.1 Backward Compatibility

**Blocker 12:** The conflict resolution rule for `__init__.sifr` + `[exports].modules` uses `SIFR-PACKAGE-0701`. But Phase 37 existing packages use `[exports].modules` as the primary model. The migration path says "keep `source.roots = ["sifr"]` supported for existing packages until a later deprecation phase." This creates a three-way compatibility case not covered:
- Old packages: `source.roots = ["sifr"]` + `[exports].modules` (Phase 37 model)
- Migrating packages: `source.root = "src"` + `__init__.sifr` (new model)
- New packages: `source.root = "src"` (default) + `__init__.sifr` (new model, no explicit source config)

The spec should explicitly document the compatibility matrix, not just say "remains supported."

**Blocker 13:** The `sifr add --alias name` behavior for existing Phase 37 packages with `[package.metadata.sifr.aliases]` in `Cargo.toml` is unclear. If a user runs `sifr add pkg --alias foo` on a Phase 37 package, does Sifr write to `Cargo.toml [package.metadata.sifr.aliases]` (preserving Phase 37 model) or migrate to `sifr.toml [dependencies]` (new model)? The spec should choose one behavior and spec it.

### 4.2 Migration Command

**Blocker 14:** The migration command `sifr package migrate-layout --from sifr-rooted --to src-init` overlaps with Phase 37 guardrails. The Phase 37 guardrails (enforced by `scripts/check_package_manager_guardrails.py`) require new packages to use the new layout. The migration command must be exempt from the guardrails during the migration window, and the guardrails must be updated post-migration to require `src/` for all packages. This is mentioned in `milestone_adhoc_pkg_7` but needs explicit spec before implementation.

**Blocker 15:** Rollback using a `.tar` backup file has security and portability concerns: `.tar` format has no standard metadata for rollback metadata (what was the original state?), large packages produce large archives, and `tar` extraction has its own failure modes. The spec should either:
- Use a structured JSON migration descriptor + file checksums instead of `.tar`, OR
- Use `git stash`-style file-level backup instead of archive-level backup, OR
- Acknowledge the `.tar` approach and spec the rollback validation (how does it handle conflicts with subsequent modifications?)

---

## 5. Package Session Architecture

### 5.1 Operation Plan

**Blocker 16:** `PackageSession` is described with fields but the `OperationPlan` in the current `sifr_package` crate (`ops/plan.rs`) is minimal (just `PackageOperation`, `lock_mode`, `mutates_manifests`, `mutates_lockfile`). The spec's `OperationPlan` includes `CargoCommandPlan` with per-command arguments, which is not in the current implementation. The spec's operation plan schema should be validated against the current `OperationPlan` and any missing fields explicitly added to the spec.

**Gap 2:** The spec says "multi-package operations compute package topological order from `SifrPackageGraph`" but does not specify what happens when Sifr's topological order differs from Cargo's internal build order. If two packages have a circular path dependency (SIFR-PACKAGE-0205), does Sifr check before or after Cargo reports it? If Sifr detects it first, does it report before invoking Cargo? The spec should clarify the ordering: Sifr validates the Sifr package graph first, then lets Cargo handle its own ordering.

### 5.2 Credential Redaction

**Blocker 17:** The credential redaction patterns list includes `gh_` and `gho_`, `ghp_`, `ghs_`, `ghr_` (GitHub token prefixes) but the spec's `redact_word` function only checks `gho_`. `ghp_`, `ghs_`, and `ghr_` are not in the implementation (`cargo/errors.rs:70`). This is a concrete bug in the Phase 37 implementation that this spec should carry forward as a fix requirement: the redaction function must cover all GitHub token prefixes (`gho_`, `ghp_`, `ghs_`, `ghr_`, `ghu_`, `ghr_`).

---

## 6. Compiler/HIR/Codegen Integration

### 6.1 Type Identity

The spec's codegen namespace rule (`sifr_gen_<name>_<hash>`) is correct. The hash derivation from normalized Cargo package id + version + source + Sifr package name is the right approach. However:

**Blocker 18:** The spec says "HIR type identity includes the Sifr package instance id, not only the textual Sifr package name." But Phase 37's `TypeIdentityMismatch` in `graph/type_identity.rs` does not include the Sifr package instance id in its error structure — it uses `expected` and `actual` strings that include version but the implementation may not include the full instance id. The spec's machine-readable `SIFR-PACKAGE-0204` format should specify exactly which fields are required: `expected_package_instance_id`, `actual_package_instance_id`, `expected_cargo_package_id`, `actual_cargo_package_id`, `import_path`, `dependency_path`.

**Gap 3:** The spec says "Generated runtime code must not use data-dependent `unwrap` or `expect` for package dispatch." This is already a Phase 37 requirement but it applies to all generated code. The spec should reference the existing Phase 37 contract rather than restating it.

---

## 7. Same Package Multiple Versions and Aliases

### 7.1 Alias Schema

**Blocker 19:** The `import` field in the dependency declaration is described as "optional public import root for this dependency instance." But the spec does not define what happens when two aliases of the same package declare the same `import` root in the same scope — this would be a conflict (two packages exporting the same import root). The spec's invariant states "import root + importing package scope -> exactly one resolved package instance" but the resolution for conflicting aliases within one scope is not spec'd.

**Gap 4:** The spec says aliases are "for multiple versions" but doesn't address the case where the same version is aliased twice for different import roots (e.g., `from utils_v1 import foo` and `from utils_v2 import bar` where both resolve to the same Cargo package). Is this allowed? Is it different from a version conflict?

---

## 8. Cargo Workspaces, Virtual Roots, Path/Git/Registry Deps

**Gap 5:** The spec mentions `--filter` selectors but does not specify whether `--filter` expressions can be composed (AND, OR, negation across filter flags) for package selection. Phase 37 spec'd this. The adhoc spec references Phase 37 selectors but should either copy the selector spec or explicitly say "see Phase 37 selector semantics."

**Gap 6:** The virtual workspace root edge case (`SIFR-PACKAGE-0706`) is correctly handled but the spec does not address what happens when a virtual workspace root has a `sifr.toml` that is NOT the root manifest (e.g., `packages/app/sifr.toml`). Does Sifr report a warning for every workspace member's `sifr.toml`? Or only for the root one? The warning conditions need to be precise.

---

## 9. Lock/Offline/Frozen Behavior

**Gap 7:** The spec says `--offline` "rejects plans whose selected package sources are absent locally." But Phase 37's `validate_offline_source_availability` function checks Cargo source availability. The adhoc spec does not document whether offline validation uses the same Phase 37 logic or has additional offline-specific constraints. This is a gap for the implementation path: the adhoc spec should explicitly say "offline validation reuses Phase 37 `validate_offline_source_availability`."

---

## 10. Diagnostics, Machine-Readable Output, Redaction

### 10.1 Cargo Failure Boundary

**Blocker 20:** The spec says credential-related Cargo failures "are still wrapped in `SIFR-PACKAGE-0101`" and that `SIFR-PACKAGE-0105` "must be retired, documented as superseded, or mapped to `SIFR-PACKAGE-0101`." But the Phase 37 implementation in `cargo/errors.rs:58-65` has an active code path that returns `SIFR-PACKAGE-0105` via `credentials_unavailable` for credential errors. The spec must clarify:
- Is `SIFR-PACKAGE-0105` actively emitted by Phase 37? Yes, it is.
- Does this adhoc phase retire it? Yes, the spec says it must.
- What is the migration path for existing tests expecting `SIFR-PACKAGE-0105`?

The implementation currently has `SIFR-PACKAGE-0105` as an active code. Retiring it requires a test migration step that is not mentioned in the implementation order.

**Blocker 21:** The redaction tests requirement ("must include both overbroad and underinclusive cases") is correct but the current Phase 37 implementation's `redact_word` function (`cargo/errors.rs:67-78`) is trivially simple — it only redact_words tokens and URLs. It does not redact URL credentials in userinfo (`https://user:pass@host/path`). The spec's URL redaction example shows `https://user:token@private.example.com/pkg` becoming `https://[redacted host]/pkg`, which requires URL parsing, not just word-level redact_word. The current implementation cannot satisfy the spec's redaction requirements without significant extension.

### 10.2 Machine-Readable Output

**Gap 8:** The spec defines the `SIFR-PACKAGE-0101` machine-readable schema but does not define the machine-readable output format for other package diagnostics (e.g., `SIFR-PACKAGE-0203`, `SIFR-PACKAGE-0204`). The existing Phase 37 diagnostic model in `sifr_diagnostics` provides machine-readable output via the `json` renderer. The spec should either reference the existing renderer or define a package-specific JSON envelope.

---

## 11. Publishing/Vendoring/Release Process

The preflight checks are correct. One gap:

**Gap 9:** The spec says credentials are "redacted in diagnostics" but does not specify what happens when a publish fails due to credentials during `cargo publish`. The spec's `SIFR-PACKAGE-0101` covers this case. But the `sifr publish` behavior when the upload partially succeeds (e.g., some files uploaded before auth failure) is not spec'd. Cargo's behavior is to leave the partial upload in place. Sifr should document whether this is accepted or whether Sifr should check for partial publishes.

---

## 12. Tests, Demos, Guardrails, Implementation Order

### 12.1 Implementation Order

**Blocker 22:** The implementation order starts with "Source layout and `__init__.sifr` source-map rules" but milestone 1 does not include updating `PackageSourceMap::build` to derive public APIs from `__init__.sifr` re-exports. The spec's milestone 1 scope mentions "Add parser/source-map tests for public root imports, public namespace imports, private implementation rejection" but the `PackageSourceMap` currently uses `exports.modules` from `sifr.toml`, not `__init__.sifr` re-exports. This is not a new feature — it's a replacement of the existing Phase 37 public API model. The spec should explicitly state that milestone 1 replaces `PackageSourceMap`'s public API derivation from `exports.modules` to `__init__.sifr` re-exports, and that `[exports].modules` becomes legacy.

### 12.2 Guardrails

**Blocker 23:** The spec mentions `scripts/check_package_manager_guardrails.py` for source layout and projection boundaries, but does not spec the specific guardrails for:
- Sifr-owned Cargo section marking (how does the guardrail detect which sections are Sifr-owned vs. user-owned?)
- Projection idempotency (how does the guardrail verify idempotency without a test oracle?)
- Pure marker regeneration prevention (when a user adds Rust code to a pure package, what is the guardrail trigger?)

### 12.3 Demo Repositories

**Blocker 24:** The spec says "Update demo repositories only after the package-aware compiler supports the new layout." This creates a chicken-and-egg problem: milestone 1 validates with "package-aware compiler supports the new layout," but the demo repos are updated in milestone 7. What do milestone tests use as fixtures? The spec should either:
- Create new fixture repos for milestone-level testing (separate from the published demo repos), OR
- Update the demo repos incrementally as each milestone lands, OR
- Use in-tree verification fixtures under `verification/package_management/`

---

## 13. Long-Term Maintainability

### 13.1 Schema Complexity

**Gap 10:** The new `sifr.toml` dependency schema extends the Phase 37 schema with `[dependencies]`, `import`, and multiple-version aliases. The Phase 37 schema already has `source.roots`, `exports.modules`, `features`, and `trust`. The adhoc spec adds `source.root` (singular, not plural), `[[bin]]` targets, and `[dependencies]`. After this phase, `sifr.toml` will have:
- Phase 37: `source.roots`, `exports.modules`, `features`, `trust`
- Adhoc: `source.root` (singular), `[[bin]]`, `[dependencies]`, `import` alias

The spec should acknowledge the schema consolidation that must happen post-phase: `source.roots` vs. `source.root`, `exports.modules` vs. `__init__.sifr` derivation. The migration plan addresses this for new packages, but there is no post-migration cleanup plan for the old fields.

**Gap 11:** "Sifr owns the user-facing package semantics, commands, diagnostics, package API, and source layout" is the right principle. But the spec does not address what happens when a Cargo command that Sifr delegates has behavior changes in a future Cargo version. The Phase 37 contract says Cargo source IDs are opaque and Cargo metadata is the authoritative input. The adhoc spec inherits this. There is no contingency for Cargo API changes.

### 13.2 Avoid Hidden Complexity

**Gap 12:** The spec says "one Cargo process-failure wrapper code, Sifr-specific codes only for Sifr-owned policy/compiler failures." But the `SIFR-PACKAGE-070*` range is already allocated for this phase. The spec does not address what happens when a future phase needs more diagnostic codes. Is there a process for retiring/renumbering codes? Is there a cap?

---

## Required Changes to the Issue

### Must-Fix (spec changes required)

1. **Define `parse_init_sifr_reexports` algorithm** — list all supported import forms, rejection criteria for unsupported forms, and the extraction output format (what does "public names" look like as data?).

2. **Specify `__init__.sifr` code-as-public-API** — define whether non-re-export definitions in `__init__.sifr` are public, and whether `import` and `import as` are valid re-export forms.

3. **Clarify Cargo projection ownership** — explicitly state: Sifr generates and maintains Sifr-owned `Cargo.toml` sections. User-owned Cargo sections are preserved but validated. Drift = any mismatch in Sifr-owned sections.

4. **Define `sifr add --alias` write target** — specify whether it writes to `sifr.toml [dependencies]` (new model) or `Cargo.toml [package.metadata.sifr.aliases]` (Phase 37 model). Choose one.

5. **Specify bidirectional Cargo manifest sync** — define what happens when user runs `cargo add` directly. Does `sifr.toml` update? Does Sifr warn? Does Sifr error?

6. **Define three-way compatibility matrix** — explicitly document behavior for: (a) old packages with `source.roots = ["sifr"]` + `[exports].modules`, (b) migrating packages, (c) new packages. Include the conflict resolution for each case.

7. **Retire `SIFR-PACKAGE-0105` with test migration plan** — Phase 37 implementation actively emits this code. Specify the test migration path and timeline.

8. **Extend redaction to URL userinfo** — the current implementation only redact_words word-level tokens. The spec requires URL credential redaction (`user:pass@host`). Specify the algorithm.

9. **Fix `redact_word` GitHub token prefixes** — `ghp_`, `ghs_`, `ghr_`, `ghu_` are missing from the current implementation.

10. **Specify migration rollback format** — replace `.tar` with a concrete alternative (JSON descriptor + checksums, or git-stash-style) with explicit rollback validation steps.

11. **Add test fixture strategy for milestone-level testing** — specify what fixtures milestone tests use (in-tree vs. demo repos).

12. **Define `--offline` source availability validation** — reference Phase 37 `validate_offline_source_availability` explicitly or spec differences.

13. **Define projection drift check failure behavior** — specify that projection drift always fails the command before invoking Cargo.

14. **Define `sifr fix` + lock mode interaction** — specify whether `sifr fix` without `--check` respects `--frozen`.

15. **Define alias conflict within same scope** — specify resolution for two aliases exporting the same `import` root in one package scope.

16. **Define `sifr init` on existing directory** — specify fail/prompt/overwrite behavior.

17. **Define `SIFR-PACKAGE-0204` machine-readable fields** — include `expected_package_instance_id`, `actual_package_instance_id`, `expected_cargo_package_id`, `actual_cargo_package_id`, `import_path`, `dependency_path`.

18. **Specify guardrail for user-owned Cargo sections** — concrete rules for "do not conflict with the package graph."

### Should-Fix (clarifications that improve implementation)

A. Reference Phase 37 selector spec instead of restating it.
B. Document the post-phase schema consolidation plan (Phase 37 `source.roots` vs. adhoc `source.root`, `exports.modules` vs. `__init__.sifr`).
C. Clarify virtual workspace `sifr.toml` warning conditions.
D. Clarify publish partial-upload behavior.
E. Define guardrail for pure marker lifecycle after user modification.

---

## Conclusion

The plan is in good structural shape and the core design principles are sound. The Phase 37 foundation is solid. However, this adhoc phase introduces significant changes to the public API model (`__init__.sifr` re-exports vs. `exports.modules`) and the dependency management model (`sifr.toml [dependencies]` vs. `Cargo.toml [package.metadata.sifr.aliases]`) that are underspecified in ways that will cause implementation divergence.

The 18 blockers above are not cosmetic — they represent places where the spec makes implicit assumptions about implementation state or Phase 37 behavior that are incorrect. Fixing them before implementation starts will prevent the kind of cross-milestone reworking that has historically cost significant time.

After these blockers are addressed, this plan is implementation-ready. The 7 gaps are additive improvements, not blockers.

---

*This is round 1 of at least 2 review rounds. Round 2 will assess the revised spec after blockers are addressed.*