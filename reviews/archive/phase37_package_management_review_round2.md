

Now I have everything I need for the review. Let me write it.

---

## Phase 37 Package Management — Hard Review (Round 2)

### 1. Verdict

**NOT SATISFIED.** The revised doc is a massive improvement over Round 1. The architecture is coherent, the manifest model is defined, the lockfile schema is present, the 7-milestone decomposition maps to the exit gate, and the Phase 27 non-regression contract is explicit. But there are four blocking issues and several non-blocking ones below.

---

### 2. Blocking Issues

**B-1: Wildcard re-export expansion is deferred, not defined.**

Line 193:
> Wildcard re-exports are rejected unless this phase defines a deterministic exported-name expansion with stable diagnostics.

The phase doc does not define the expansion. It defers it to "this phase" but provides no specification: no grammar for the re-export syntax, no determinism rules, no diagnostics. Every other design decision in this doc is concrete. This is the lone deferred spec inside a "complete model, not MVP" doc.

This blocks milestone_37_3 scope and the exit gate at line 434 ("re-exports...implemented"). Without a defined re-export model, the implementation team must make significant design choices in code — exactly the fragmentation this phase forbids.

**Required fix:** Define the wildcard re-export syntax and expansion semantics. Options: explicit-only re-exports (simplest, no deferred spec), `__all__` declaration in `__init__.sifr`, or `from X import *` with declared-name static expansion. Pick one and specify it. The current language ("unless this phase defines") passes the buck to implementation.

**B-2: Version specifier grammar is unspecified.**

The lockfile shows `requirement = "1.4"` but the grammar of version specifiers is never defined. The manifest examples use bare `"1.4"` and `{ version = "0.8" }` but don't show `^`, `~`, `>=`, `!=`, range combinators, or any of the standard semver constraint syntax. `milestone_37_1` scope says "version ranges" but the spec is silent on syntax.

This blocks manifest parsing (milestone_37_1) and lockfile determinism (milestone_37_2). Implementers must invent syntax from scratch, producing inconsistent constraint handling.

**Required fix:** Define the version specifier grammar inline. Example: `requirement = "^1.4"` (caret), `">=1.0,<2.0"` (intersection), `">=1.0"` (comparison). Align with Cargo's semver spec which is well-understood.

**B-3: Error codes are never specified.**

The doc references "package diagnostics," "deterministic diagnostics," "actionable diagnostics," and "stable diagnostics" throughout, but never defines a single `SIFR-PACKAGE-XXXX` error code. The validation matrix (lines 406-428) describes expected failure behavior ("ambiguous export root," "stale lock under --locked") but doesn't map to codes.

Per the Phase 27 non-regression contract (line 394), diagnostic stability is non-negotiable. Error codes must be defined before implementation so the compiler team can reserve the namespace. The current state leaves every error code as an implementation decision.

**Required fix:** Define the SIFR-PACKAGE error code namespace. At minimum, define codes for: unresolved import, undeclared dependency, ambiguous export, namespace collision, feature cycle, lockfile staleness, cache miss under offline, yanked version on new resolution, archive path traversal, and credential exposure attempt. The codes can be documented as placeholders with expected semantics if the full code list is too large for the planning doc.

**B-4: Registry protocol specifics are too vague for implementation.**

Line 283 says the registry uses "a sparse HTTP index" and line 298 specifies that "package tarballs are immutable once published." But the protocol mechanics are underspecified: no index format, no download endpoint contract, no yank propagation behavior, no alternate registry discovery, no publish authentication flow, no tarball format definition.

The `[trust]` table (lines 100-101, 299-300) is referenced but not defined: what happens when a transitive dependency isn't in `[trust]`? Is that a hard error? Does trust propagate? Can a package set its own trust level? What is the default (trust all vs. trust none)?

This blocks milestone_37_6 scope and the exit gate at line 437 ("registry publish/yank/owner/login flows are production-grade").

**Required fix:** Define at minimum: the sparse index response format, the package tarball structure, the publish authentication contract (token-based? OAuth? certificate?), and the `[trust]` semantics (default-deny, trust propagation rules, build-script gate behavior).

---

### 3. Non-Blocking Suggestions

**S-1: `sifr vendor` command is under-specified.**
The CLI contract includes `sifr vendor <dir>` but there's no description of what it produces. Copy to a vendor directory? Update manifest path references? What happens to lockfile references? Recommend adding a brief spec.

**S-2: Package naming convention is ambiguous.**
The manifest examples use plain names (`"http"`, `"json"`) while my Round 1 review recommended scoped names (`"@org/pkg"`). The doc doesn't specify whether scoped names are supported, required, or forbidden. Recommend specifying: plain names for initial use, scoped names for registry publishing, or no restriction.

**S-3: Workspace `[target]` inheritance is unclear.**
The manifest shows `[target.'cfg(unix)'.dependencies]` (line 94) but doesn't specify whether workspace members inherit root-level target configurations or must declare them independently. Recommend clarifying inheritance semantics.

**S-4: Feature conflict resolution is unspecified.**
The doc says "feature activation uses union semantics" (line 112) but doesn't specify what happens when two dependencies request conflicting features of the same package (one enables `tls`, the other disables it). Recommend stating that this is a resolution error.

**S-5: Artifact cache key is described but the cache invalidation strategy is not.**
Lines 305-313 list cache key components but don't specify when entries are evicted, how cache size is bounded, or what happens on cache corruption. For a production package manager these are operational concerns. Not blocking but worth a sentence.

**S-6: `sifr outdated` is in the CLI contract but no behavior is described.**
`--workspace` and `-p package` flags are mentioned for `sifr outdated` but the semantics (comparison to registry latest? comparison to semver ranges? comparison to lockfile?) are unspecified.

---

### 4. Specific Required Edits

For the doc to be approved, the following must be added:

1. **Wildcard re-export definition** — either define the syntax or explicitly prohibit wildcard re-exports and require explicit `from X import Y as Y` syntax only. Delete the "unless this phase defines" hedge.

2. **Version specifier grammar** — define at minimum: bare semver, caret (`^`), tilde (`~`), comparison operators (`>=`, `<=`, `>`, `<`, `!=`), and range combinators. Show in both the manifest example and the lockfile example.

3. **Error code namespace** — define a list of `SIFR-PACKAGE-XXXX` codes with their semantic descriptions. Minimum 10 codes covering the documented failure modes. This is a documentation change, not implementation.

4. **Trust model semantics** — define: default behavior (trust-all vs. trust-none), propagation rules, build-script gate behavior, what happens when an untrusted transitive dependency is encountered.

5. **Registry protocol basics** — define: sparse index format (even as a placeholder), publish authentication mechanism, tarball immutability guarantee, yank semantics (excluded from new resolution, usable from existing lockfiles — already stated, just needs a brief formalization).

---

### 5. What Is Already Good

To be balanced: the doc gets the following right in ways that avoid the most common package-manager design mistakes:

- **Cargo/uv role separation is correct.** Cargo is the generated build backend, not the resolver. uv is excluded from resolution entirely. This is the right call.
- **Package-origin tracking without flattening** is correctly specified. The `ModuleOrigin` enum and resolution order (lines 199-220) are clean and prevent the shadowing ambiguity that plagues naive package managers.
- **Direct-dependency import enforcement** (line 19, 346) is the right model. Transitive dependencies are compiled but not importable unless re-exported or declared.
- **`--locked` / `--offline` / `--frozen` semantics** are correctly defined and consistent with Cargo/uv conventions.
- **Workspace lockfile ownership** (line 229: workspace root owns the single `sifr.lock`) is correct.
- **Staged milestone decomposition** from manifest → resolver → import resolution → backend → CLI → registry → validation is the right order.
- **Quality contract** (no fallback, no legacy compat, no lazy fixes, production-grade only) is appropriately strict.
- **Phase 27 non-regression** is explicitly woven into the exit gate.

The doc is close. The four blocking issues are fixable with additions, not rewrites.

---

### Summary

| Category | Count |
|----------|-------|
| Blocking issues | 4 |
| Non-blocking suggestions | 6 |
| Specific required edits | 5 discrete additions |

The doc needs five targeted additions. Once those are in, the implementation team has a complete contract to build from. The architecture is sound — the gaps are in specification completeness, not design correctness.
