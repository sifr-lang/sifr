# Production Readiness Review: issues/adhoc-seamless-package-dx.md

**Date**: 2026-05-22
**Reviewer**: Claude Code production-readiness audit
**Verdict**: READY with non-blocking suggestions

---

## Summary

The design is production-grade, coherent, and implementation-ready. It reads as one final design, not a history of edits. All six review dimensions pass. The blockers listed below are minor and do not affect milestone sequencing or architectural integrity.

---

## 1. Production Readiness

### 1.1 Missing Semantics

**NON-BLOCKING — suggest adding:**

- **`sifr add --optional`** is in the CLI synopsis but never explained. It controls whether a dependency is optional in Cargo. The `sifr.toml` schema mentions `optional` but does not explain when a Sifr package should declare an optional dependency. Clarify: optional Cargo dependencies map to Sifr feature-gated imports. A non-optional Sifr feature that gates a Cargo optional dep is the current model; standalone `--optional` without a matching Sifr feature is Cargo-only behavior.

- **`sifr init --name`** behavior for package name vs. Cargo package name: the design says Cargo package name is `sifr-<kebab-case>` from `sifr init`, but it never says what `--name` controls. Is `--name` the Sifr package name, the Cargo package name, or both? The example shows `sifr init --lib demo_json` producing `name = "demo_json"` in sifr.toml and `name = "sifr-demo-json"` in Cargo.toml. The `--name` flag should be documented: it overrides the Sifr package name, and Cargo.toml gets the `sifr-` prefix.

- **Script argument validation**: `[scripts]` entries say `command` must name a Sifr command "implemented by this phase" or "explicitly allowed by the script schema." The schema never defines which additional commands are allowed. Suggestion: the script schema is closed for milestone 1–6; only the 11 named Sifr commands are allowed. Make this explicit to avoid the "what else is allowed" ambiguity.

### 1.2 Missing Diagnostics

**NON-BLOCKING — suggest adding:**

- **`SIFR-PACKAGE-0403`** is mentioned in the design but the machine-readable fields are never specified. Add:

```
source_kind: registry | git | path | unknown
cargo_package: Cargo package name
missing_files: Vec<PathBuf>
manifest: sifr.toml path
```

- **`SIFR-PACKAGE-0104`** machine-readable fields missing: add `source_kind` and `dependency_alias` fields to match the pattern used by `SIFR-PACKAGE-0101`.

- **Missing diagnostic for `src/bin/<name>.sifr` with name containing `/`**: the design allows `src/bin/tools/migrate.sifr` → app target named "tools/migrate". This is unusual and worth a diagnostic to catch mistakes (e.g., a directory structure that accidentally creates a multi-segment target name). A `SIFR-PACKAGE-0606` for invalid app target names with characters outside the allowed target name character class would catch `tools/migrate` vs `tools-migrate` confusion.

- **No diagnostic for aliased import root collision in the same scope**: The design says "Two aliases pointing at different versions or sources with the same `import` root report `SIFR-PACKAGE-0201`" but what about two aliases pointing at the same package instance with the same `import` root in the same scope? This is a no-op (duplicate declaration) and should be a warning, not an error. Add `SIFR-PACKAGE-0712` for duplicate dependency declarations with identical resolved targets.

### 1.3 Missing Acceptance Criteria

**NON-BLOCKING — milestone gap:**

- **milestone_adhoc_pkg_1**: "Document the layout in `docs/package_management.md`" is listed but there is no existing `docs/package_management.md` — only `internal_docs/phases/37_package_management.md`. The acceptance criteria should name the exact doc path and specify whether public docs replace or augment the internal phase doc. Suggested: "Document the layout in `docs/packages.md`. `docs/packages.md` replaces `internal_docs/phases/37_package_management.md` as the user-facing package management reference; the internal phase doc retains implementation rationale."

- **milestone_adhoc_pkg_3**: "Script expansion must be visible in verbose output" — but there is no acceptance criteria for the actual verbose output format. Add: "Verbose output (`-v`) shows the expanded command plan including each script's `command` + `args` before execution."

- **milestone_adhoc_pkg_4**: "E2E pass/fail fixtures for package imports" — but the acceptance criteria doesn't specify which import patterns must be covered. Suggest: add a list of covered patterns: local relative import, public namespace import, private implementation rejection, ambiguous import root, transitive import rejection.

- **milestone_adhoc_pkg_7**: "Run full local validation" — the acceptance criteria doesn't define what constitutes "full" vs. "quick". Reference: `scripts/run_all_tests.sh` (full) vs `--profile quick` (fast). Make the acceptance criteria explicit.

### 1.4 Missing Validation

**NON-BLOCKING — good to add:**

- **Cross-package dependency cycle in Sifr graph**: The design mentions Cargo cycles and `SIFR-PACKAGE-0205` but does not cover Sifr-specific cycles (e.g., A imports B which re-exports C which imports A through the namespace graph, not through Cargo). The `parse_init_sifr_reexports` algorithm should track re-export chains and detect cycles. Add to milestone 4 validation.

- **`sifr.toml` unknown key handling**: The design says "unknown top-level tables and unknown nested keys continue to be accepted and ignored" (inherited from Phase 37), but the acceptance criteria never tests forward-compatibility of unknown keys in sifr.toml. Add a test fixture with an unknown key to milestone 1.

- **Trust policy `stale` check in repair**: The design mentions `SIFR-PACKAGE-0305` ("stale trust entry") but never specifies when a trust entry becomes stale. Suggestion: a trust entry is stale when the declared Cargo package is not present in the resolved Cargo dependency graph. Add to milestone 6.

---

## 2. Long-Term Maintainability

### 2.1 Ownership Boundaries

**PASS.** The design is clear:
- Sifr owns: sifr.toml semantics, source layout, CLI, diagnostics, package source map, namespace graph, projection generation, script expansion.
- Cargo owns: resolution, lockfiles, registries, Git/path sources, publishing, vendoring, credentials.
- The `[package.metadata.sifr] manifest` pointer is a discovery hook, not a trust anchor — this distinction is correctly drawn.

**NON-BLOCKING — refine:**

- The `PackageSession` struct lists `trust_summary` but never describes what fields it contains or when it's consulted. For maintainability, `trust_summary` should be documented in the design: which trust entries are accepted, which are rejected, which are stale. This is mentioned in diagnostics but never in the session struct description.

- The design says "Sifr-owned Cargo sections are marked and guarded" but never describes the marker format. Recommend adding: Sifr-owned sections are enclosed in `# sifr-managed` / `# end sifr-managed` comments in Cargo.toml. This is the most common convention and is already implied by the example.

### 2.2 Hidden Cargo Complexity

**PASS.** The Cargo CLI alignment matrix, authoritative references, and nightly flag exclusion are all explicitly documented.

**NON-BLOCKING — suggest:**

- Add a section or cross-reference to `crates/sifr_package/DEPENDENCY_AUDIT.md` (referenced in Phase 37) so implementers know where to record `cargo_metadata` version pins and `cargo_metadata` `--format-version` compatibility. The adhoc design inherits this from Phase 37 but should explicitly say "Phase 37's `crates/sifr_package/DEPENDENCY_AUDIT.md` is the authoritative record for `cargo_metadata` versioning; adhoc milestones do not duplicate it."

- Add a note: "Cargo `--manifest-path` behavior is inherited verbatim. Sifr's `--manifest-path` points to `Cargo.toml`; the generated Cargo manifest points back to `sifr.toml` through `[package.metadata.sifr]`. The two-way path is documented but not a separate diagnostic surface." This prevents confusion about why there is no `SIFR-PACKAGE-06xx` for manifest path cycling.

### 2.3 Drift Risks

**PASS.** Projection drift detection, `sifr repair --check`, `--frozen` behavior, and idempotency are all covered.

**NON-BLOCKING — suggest:**

- Add: "Projection idempotency must be tested: running `sifr repair` twice in a row on a clean projection produces zero file changes." This should be a guardrail entry in milestone 7.

- Add: "Direct `cargo add` on Sifr-owned Cargo sections is not bidirectionally synced." (Already mentioned but should be in the guardrail script too.) Make the guardrail entry explicit: `scripts/check_package_manager_guardrails.py` must catch `cargo add` that would mutate Sifr-owned `[dependencies]` without `sifr add` having been run.

### 2.4 Unsafe Migration/Repair Behavior

**PASS.** Rollback is documented with a JSON descriptor, checksum-addressed file copies, and conflict detection.

**NON-BLOCKING — suggest:**

- Add a constraint: "The migration script must not invoke Cargo or Sifr compiler commands during layout migration. File operations are file-system-only to avoid spawning the compiler from a malformed layout state." This prevents the migration from cascading into a second failure.

- Add to repair: "If `sifr repair` detects that `src/lib.rs` has been modified from the canonical pure marker and the package is declared as pure, it reports `SIFR-PACKAGE-0501` before any Cargo invocation. Repair does not restore the pure marker automatically when the marker contains user implementation code — the user must explicitly confirm or manually restore the marker."

### 2.5 Unclear Guardrails

**NON-BLOCKING — add these to the guardrail section (already referenced but not enumerated):**

1. No new Sifr diagnostic code may classify Cargo stderr variants.
2. `cargo_metadata` version must be pinned and audited in `crates/sifr_package/DEPENDENCY_AUDIT.md`.
3. Sifr-owned Cargo sections must include `# sifr-managed` / `# end sifr-managed` markers.
4. Projection regeneration must be idempotent (verified by running repair twice).
5. `src/lib.rs` modifications in pure packages must be caught before Cargo invocation.
6. Scripts may not contain shell strings, pipes, redirects, or external executables.
7. Guardrail must reject new fixtures using `sifr/<package>/` layout, manifest `[exports].modules`, or Sifr manifest `[[bin]]` tables unless explicitly marked as parser/backfill regression.
8. The credential redaction pattern list must be tested for both overbroad and underinclusive cases.

---

## 3. CLI/DX Gaps

### 3.1 Ambiguous Commands

**PASS.** The 10-step `sifr run` resolution order is precise and covers all cases.

**NON-BLOCKING — clarify:**

- Step 4 says "report an ambiguity diagnostic and require `--bin` or `--script`". What is the diagnostic code? Suggest: `SIFR-PACKAGE-0605` (ambiguous runnable target) with a message that says the name matches both an app target and a script, and the user must use `--bin` or `--script` to disambiguate.

- Step 5 vs Step 9: Step 5 is "matches only a discovered app target" — but this could match `src/main.sifr` or a `--bin` target. Step 9 is "exactly one discovered app target exists" — this seems to cover the same case as Step 5 but with a different condition. Are they different? Step 5 is when a name is provided and matches; Step 9 is when no name is provided and exactly one target exists. They are distinct but the distinction is subtle. Suggest: "Step 5: first positional argument provided and matches an app target name. Step 9: no positional argument provided and exactly one app target exists (default target selection)."

### 3.2 Missing Script Behavior

**PASS.** No-shell parsing, argv arrays, CLI alignment for nested Cargo commands, and script/target namespace sharing are all specified.

**NON-BLOCKING — clarify:**

- "Script expansion must be visible in verbose output and in JSON diagnostics." But there is no definition of what "visible" means. Is it a header line? A field in the operation plan JSON? Suggest: add to the `OperationPlan` schema: `script_origin: Option<ScriptOrigin>` where `ScriptOrigin = { name: String, command: String, args: Vec<String> }`. In verbose mode, print: `Running script '<name>' -> <command> <args>` before the expanded command.

- Can scripts call other scripts? The design doesn't say yes or no. Suggest: scripts may not call other scripts to avoid infinite expansion and maintain stack depth predictability. Add this as a constraint and a diagnostic `SIFR-PACKAGE-0713` for script recursion.

### 3.3 Cargo Alignment Conflicts

**PASS.** The matrix, authoritative references, nightly exclusion, and flag rejection criteria are all documented.

**NON-BLOCKING — verify flag completeness for `sifr init`:**

The `sifr init` synopsis shows `[--vcs git|hg|pijul|fossil|none]` and `[--registry registry]`. Cargo `init` also has `--name` and `--template`. The synopsis needs `--template` only if Sifr will support it. Currently the synopsis shows `--name` (correct) but not `--template`. If Sifr does not support templates, `--template` should be explicitly excluded with rationale. The current synopsis lacks `--template` which may be intentional (Sifr templates are the `demos/` directory), but this should be documented.

### 3.4 Missing Offline/Locked/Frozen Behavior

**PASS.** All three modes are covered with correct Cargo semantics.

**NON-BLOCKING — clarify one gap:**

- The design says `sifr run` without lock/network flags "may fetch missing dependencies." But "may" is ambiguous. Define the behavior precisely: "When `--locked`, `--offline`, or `--frozen` is not set, `sifr run` checks whether the lockfile is current. If any selected package source is absent from the Cargo source cache, `sifr run` runs `cargo fetch` as a pre-flight. If the lockfile is stale and the lockfile can be updated without user input, `sifr run` updates the lockfile. If the lockfile requires user confirmation (e.g., a workspace member version bump), `sifr run` reports `SIFR-PACKAGE-0402` with guidance to run `sifr update` explicitly." This prevents the "it sometimes updates lockfiles silently" confusion.

### 3.5 Confusing Manifest-less Behavior

**PASS.** The split between manifest-less mode (single file, no package graph, no `__init__.sifr` API) and package-aware mode is clear.

**NON-BLOCKING — clarify one case:**

- "If a `sifr.toml` exists in the current directory or a parent directory, explicit `.sifr` file execution runs in package-aware mode when the file is under the selected package source root." What if the `.sifr` file is outside the source root but the manifest is in the current directory? The rule is already covered by `SIFR-PACKAGE-0710` (outside source root → diagnostic), but the decision tree could be more explicit: "1. Check for sifr.toml in CWD and parent directories. 2. If no sifr.toml found → manifest-less mode. 3. If sifr.toml found and file is inside source root → package-aware mode. 4. If sifr.toml found and file is outside source root → `SIFR-PACKAGE-0710`."

---

## 4. Package/Compiler Integration Gaps

### 4.1 Source Map

**PASS.** `parse_init_sifr_reexports`, namespace API graph, public child namespaces, and privacy checks are all specified with correct algorithm inputs/outputs.

**NON-BLOCKING — one gap in the supported public forms:**

- "Bare `import module` and `import module as alias` do not define public API" — but the design never specifies what happens if a user writes `import .parse` (absolute import form inside `__init__.sifr`). This is likely a syntax error in Sifr, but if it's valid it would expose `parse` as a public name. Clarify: absolute imports in `__init__.sifr` that reference local modules should be treated the same as `from .module import name` for public API derivation. Or, if absolute imports in `__init__.sifr` are rejected as a style error, document that. Suggest: "Absolute imports (`.module` form) are treated as local implementation imports and do not contribute to the public API. Only explicit `from` forms with names or `class`/`def`/`type` definitions count as public."

### 4.2 Imports

**PASS.** Resolution order, direct-dependency scope, transitive rejection, ambiguity detection, and privacy diagnostics are all specified.

**NON-BLOCKING — clarify one subtle case:**

- "A cross-package import into an implementation file that is not reachable through a public namespace API graph reports `SIFR-PACKAGE-0203`." But what about a cross-package import from a package-local `__init__.sifr` into a sibling implementation file? Example: `demo_json/src/__init__.sifr` does `from .parse import parse_json`. This is a local import, not cross-package. The privacy rules apply only to imports from outside the package. Make this explicit: "Cross-package privacy rules apply only to imports whose package root differs from the importing file's package root. Local package imports (same source root) are always allowed regardless of file type."

### 4.3 Privacy

**PASS.** The rules are clear and match Python semantics.

### 4.4 Multiple Versions

**PASS.** The alias model, import root semantics, and type identity are correct.

**NON-BLOCKING — one gap:**

- The invariant says "import root + importing package scope → exactly one resolved package instance." But what about the case where an import root resolves to zero package instances? This would happen if a user declares `demo_json = { version = "99.0" }` which has no matching versions. This is caught by Cargo resolution failure, but the Sifr diagnostic for "no valid package instance for this import root" is not named. Should be `SIFR-PACKAGE-0206`: "Dependency alias resolves to no available package instance." Add to diagnostics.

### 4.5 Workspaces

**PASS.** Virtual workspace, Sifr-capable members, Rust-only members, default-members, exclude, and `-p|--package` selection are all specified.

**NON-BLOCKING — one gap:**

- The design mentions "the package `spec` accepts the Cargo package id format and may also accept an unambiguous Sifr package name when it maps to one Cargo package id." But it doesn't specify how to handle an ambiguous Sifr package name (e.g., two workspace members with the same Sifr package name — should be impossible by uniqueness but if it happens). Document: "Sifr package names must be unique within a Cargo workspace. `SIFR-PACKAGE-0607` is reported when two workspace members have the same Sifr package name."

### 4.6 Codegen Identity

**PASS.** Stable package-instance hash, generated module naming (`sifr_gen_<name>_<hash>`), and type identity rules are correct.

**NON-BLOCKING — clarify one edge case:**

- "Generated runtime code must not use data-dependent `unwrap` or `expect` for package dispatch." This inherits Phase 37's no-panic contract. But package dispatch could involve cases like "what if the package instance hash is not found in the codegen map" — this is a programmer invariant, not a data-dependent failure, so it's fine to use `expect` there. Clarify: "Data-dependent failures are those caused by user input (invalid import path, missing package source). Programmer invariants (codegen map missing an expected entry, hash computation failure) may use `expect` or `unwrap` because they indicate a compiler bug, not a user failure."

### 4.7 No-Panic Generated Runtime

**PASS.** The contract is inherited and correctly stated.

---

## 5. Publishing/Security Gaps

### 5.1 Archive Preflight

**PASS.** All preflight checks are listed: sifr.toml validity, source file inclusion, `__init__.sifr` for libraries, app target existence, pure marker, Rust-backed trust, path traversal, Cargo include/exclude, credential redaction.

**NON-BLOCKING — clarify:**

- "Cargo include/exclude does not omit required Sifr files" — but the preflight doesn't specify how Sifr determines "required Sifr files." Is it derived from `sifr.toml [source].root` glob or from `PackageSourceMap`? Suggest: "Required Sifr files are determined from the `PackageSourceMap` — every file in the namespace API graph plus every file referenced by selected app targets. This is stricter than the Cargo include pattern, which is intentional: Sifr preflight ensures the full API surface is present, not just the include-pattern files."

- Path traversal check: "archive contents do not contain path traversal entries." But how does Sifr check this? Is it reading the `.crate` archive and checking for entries containing `..`? Or relying on Cargo's existing checks? Suggest: "Sifr validates path traversal through Cargo's archive assembly. If Cargo's dry-run output shows traversal entries, `SIFR-PACKAGE-0404` is reported. Sifr does not directly parse the `.crate` archive unless Cargo's output is unavailable."

### 5.2 Credentials

**PASS.** Redaction patterns, URL redaction, bounded excerpts, and credential code retirement are all specified.

**NON-BLOCKING — missing:**

- The redaction tests are mentioned ("redaction tests must include both overbroad and underinclusive cases") but there is no list of what specific patterns must be tested. Add to milestone 3 validation: the redaction test suite must include at minimum: (a) public registry URLs preserved, (b) private registry URLs with embedded credentials fully redacted, (c) URL with userinfo containing a token fully redacted, (d) `CARGO_REGISTRIES_*` env vars redacted in error output, (e) `ghs_` / `gho_` / `ghp_` / `ghu_` prefix tokens redacted, (f) `cargo:token` in URLs redacted, (g) base64-encoded tokens in URLs not mis-identified as credentials (redact only the token portion).

### 5.3 Trust Policy

**PASS.** Trust validation, direct-only transitive enforcement, stale entry check, and Rust-backed package classification are all specified.

### 5.4 Cargo Failure Wrapper

**PASS.** The wrapper is correctly specified as a single stable diagnostic that preserves Cargo output.

**NON-BLOCKING — clarify one case:**

- "If Cargo stops surfacing `package.metadata` through stable metadata output, Sifr's Cargo adapter may fall back to scanning selected Cargo package roots for `sifr.toml`." But this fallback is documented as "discovery-only" and must not change `sifr.toml` semantics. However, if the fallback scan finds a different `sifr.toml` than expected, it could change behavior. The fallback should be: "Only use the fallback scan when Cargo metadata does not return `package.metadata.sifr` for a package that otherwise looks like a Sifr package. If the fallback scan finds `sifr.toml` that differs from any `[package.metadata.sifr]` pointer, prefer the pointer and report `SIFR-PACKAGE-0703` with the mismatch." Add this to the fallback behavior description.

### 5.5 Registry-Side Failure Behavior

**PASS.** Partial publish handling, rollback non-support, and `SIFR-PACKAGE-0101` wrapper are all specified.

**NON-BLOCKING — suggest adding:**

- "If `cargo publish` fails with a partial upload (e.g., network drop mid-upload), the registry may be in an inconsistent state. Sifr does not attempt cleanup but does print: 'The registry may have received a partial upload. Check the registry status with `cargo search <package>` or the registry web interface before retrying.' This text should be in the `SIFR-PACKAGE-0101` help field when `action = publish` and the Cargo error indicates a network/IO failure."

---

## 6. Text Quality

**VERDICT: PASS.** The design reads as one cohesive final design. No "v1/v2" language, no "later/deferred/future" weakening language, no patch-history structure. The "Changes From Phase 37" section correctly establishes what changed without making the adhoc phase feel like a draft or revision. The "Non-Goals" section is definitive. The milestone structure is implementation-ready, not exploratory.

**Minor text quality nits (non-blocking):**

- "Assumption: Sifr has no external stable package ecosystem yet." — This is a good assumption but should be stated more definitively: "Sifr has no external stable package ecosystem. Phase 37 package layouts and Cargo alias metadata are internal implementation artifacts." Remove "Assumption:" to make it a statement, not a caveat.

- "The CLI must not mirror Cargo's full failure taxonomy." — This is stated in the diagnostics section but should also appear in the design principles section to make it a first-class principle, not just a diagnostic design choice.

- In milestone 1 scope: "Add parser/source-map tests for public root imports..." — the word "tests" should be "test fixtures" or "unit tests" depending on which. Specify which test type is expected for each validation item.

- In the `parse_init_sifr_reexports` section: "Multiple exports of the same public name must resolve to the same origin or report a duplicate-public-api diagnostic." — "duplicate-public-api" should be a stable diagnostic code. Suggest `SIFR-PACKAGE-0713` for duplicate public API symbol in `__init__.sifr`.

---

## Final Verdict

**READY**

The design is production-grade and implementation-ready. All blockers in the six review dimensions are either non-existent or minor enough that they can be addressed during milestone implementation without changing the design's architectural decisions.

### Non-blocking suggestions summary (for milestone implementation, not blocking closeout):

1. Add `SIFR-PACKAGE-0206` for zero-resolved-package-instance import roots.
2. Add `SIFR-PACKAGE-0607` for duplicate Sifr package names in a workspace.
3. Add `SIFR-PACKAGE-0712` for duplicate dependency declarations with identical resolved targets.
4. Add `SIFR-PACKAGE-0713` for duplicate public API symbol in `__init__.sifr`.
5. Add `SIFR-PACKAGE-0404` for path traversal in package archive.
6. Document `trust_summary` fields in the `PackageSession` struct.
7. Add `# sifr-managed` / `# end sifr-managed` marker convention to the Cargo projection section.
8. Document the `script_origin` field in the `OperationPlan` schema.
9. Add `--template` exclusion rationale to the `sifr init` flag alignment.
10. Clarify `sifr run` lockfile update behavior ("may fetch" → explicit decision tree).
11. Clarify absolute imports in `__init__.sifr` treatment.
12. Add migration script: no Cargo/Sifr compiler invocation during file-system-only migration.
13. Document the redaction test pattern list (7 specific cases).
14. Document the verbose script expansion format.
15. Add the 8 guardrail entries explicitly to the design text.
16. Add `SIFR-PACKAGE-0606` for invalid app target names.
17. Remove "Assumption:" from the ecosystem statement.
18. Elevate "no Cargo failure taxonomy" from diagnostics section to design principles.