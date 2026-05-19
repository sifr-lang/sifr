# Phase 37 v3 Review — Round 2

## Verdict: **Ready-with-nits** (4 blockers, 6 production gaps, 3 clarity issues)

The v3 plan is a strong architectural document. The Cargo-as-primary decision is correct, the scoping rule is coherent, the non-goals are well-scoped, and the maintainability architecture with `DEPENDENCY_AUDIT.md` / `TRACEABILITY.md` / `FEATURES.md` is the right approach. The plan correctly avoids a native resolver, avoids uv as the core substrate, and correctly keeps `sifr.toml` narrow.

The blockers below are the only things that would prevent a correct implementation. The production gaps and clarity issues are fixable in-milestone without redesign.

---

## Blockers

### B1: `src/lib.rs` marker has no enforcement mechanism

The plan states the marker "must not become a second source of truth for Sifr behavior" and says "generated `src/lib.rs` marker should contain no semantic implementation." Neither the plan nor the guardrails specifies how this is enforced. A package author can add real Rust code to the marker and affect Cargo build behavior (different dependencies, cfg flags, feature-gated behavior, etc.), which Sifr cannot detect or prevent.

**Fix:** Add `SIFR-PACKAGE-0501` ("rust marker source contains non-trivial implementation") and specify that `sifr package --dry-run` validates the marker is empty or only contains an empty `lib.rs` with a re-export comment. Alternatively, document that the marker is the package author's responsibility and Sifr makes no guarantees if it's modified.

### B2: `Cargo.toml` `include` attribute is ambiguous for Sifr source files

The example shows an explicit `include = ["Cargo.toml", "sifr.toml", "sifr/**/*.sifr", ...]` pattern, but the plan never specifies whether `include` is required, recommended, or optional. `cargo package` defaults to including files tracked by VCS or the manifest. If a package uses `exclude = [...]` that happens to drop `.sifr` files, or uses a sparse registry and has no VCS, the archive will be malformed and `sifr package --dry-run` must catch it — but the plan doesn't specify how.

**Fix (choose one):**

- **Required `include`:** Add `SIFR-PACKAGE-0403` ("cargo `include` does not enumerate `.sifr` source files") and require explicit `include` patterns for `sifr/` directories in `Cargo.toml`.
- **Inferred:** `sifr package --dry-run` walks the Cargo package root, finds all `sifr/` directories (respecting `[source].roots`), and verifies they appear in the `cargo package` dry-run archive. No `include` modification required.

### B3: Cross-instance type passing at package scope boundaries is undefined

The plan states `sifr-math@1.4::math.Vector != sifr-math@2.1::math.Vector` as a type identity rule, but does not define what happens when a value crosses a package scope boundary. Example:

```sifr
# in image-lib (sifr-math 1.4)
def make_vector() -> Vector: ...

# in app
import math_v1
result = math_v1.make_vector()  # type: sifr-math@1.4::math.Vector

def process(v: Vector) -> None: ...  # which Vector?
```

Is this a type error? Is it resolved by import alias? The plan doesn't say. Without this rule, the type identity guarantee is technically true but practically unenforceable at call sites that bridge scopes.

**Fix:** Define that cross-scope type identity requires the calling code to explicitly import from the same package version. If `app` uses `math_v1.Vector` from one scope and `math_v2.Vector` from another, they are distinct types. Calling `math_v1.make_vector()` and passing to a function expecting `math_v2.Vector` is a type error. Add `SIFR-PACKAGE-0204` with structured `expected_package_id` / `actual_package_id` fields.

### B4: `cargo metadata` determinism is not guaranteed

The plan's property test states: "Derived `SifrPackageGraph` is deterministic for same `cargo metadata`, lockfile, manifests, target, features, and selectors." But `cargo metadata --format-version 1` is not guaranteed deterministic across invocations — it can return packages in different orders depending on resolution tie-breaking, feature flags, and Cargo version. If the derived graph depends on iteration order of `cargo metadata`'s package list, it will be non-deterministic in ways that violate the property test.

**Fix:** In `crates/sifr_package/DEPENDENCY_AUDIT.md`, record that `cargo metadata` output must be normalized before graph derivation: sort packages by a stable key (e.g., `package_id` string), sort dependency edges, and sort `BTreeMap` / `BTreeSet` structures. Alternatively, use a canonical representation in the digest computation so that non-determinism is detected (if two runs produce different graphs, the digest differs).

---

## Production Gaps

### G1: No fetch lifecycle specification

The plan delegates to `cargo fetch` for source availability, but does not specify when `sifr` invokes it:

- On every `sifr build` / `sifr check` automatically?
- Only when `sifr fetch` is explicitly run?
- On first package-aware command after `cargo metadata` discovers a new package?

The answer affects offline/frozen behavior. If `sifr build --offline` fails because `cargo fetch` was never run, users need to know they must run `sifr fetch` first. If `sifr build` automatically runs `cargo fetch`, offline mode is hard to reason about.

**Fix:** Define that `sifr build` / `sifr check` / `sifr run` / `sifr test` invoke `cargo fetch` lazily when source is unavailable and the mode is not `--offline`. `--offline` fails immediately with `SIFR-PACKAGE-0104` ("package source unavailable in offline mode") if any selected Sifr source is not in the Cargo source cache.

### G2: Publishing failure after Sifr validation is undefined

The plan says `sifr publish` delegates to Cargo after Sifr validation succeeds. If `cargo publish` fails after validation (e.g., credential expiry, network error, registry-side conflict), the plan doesn't define what state `sifr publish` is left in. Specifically: does the `Cargo.lock` mutation from `cargo publish --dry-run` (which resolves versions) persist? Does the user need to revert?

**Fix:** Define that `sifr publish` runs `cargo publish --dry-run` first (no lock mutation), then `sifr package --dry-run`, then `cargo publish` with `--allow-dirty` or equivalent. If the final `cargo publish` fails, exit with `SIFR-PACKAGE-0402` and print the Cargo error redacted as structured cause. `Cargo.lock` is not mutated by the dry-run steps.

### G3: `sifr build --workspace` behavior is ambiguous

Does `--workspace` build all Cargo workspace members, all Sifr packages in the workspace, or the current package? For a mixed workspace (some Rust-only crates, some Sifr packages), building "all workspace members" would build Rust-only crates that Sifr doesn't need. For Sifr-only workspaces, there's no difference. The plan specifies `sifr test --workspace` as "Cargo workspace members that expose Sifr metadata" but doesn't specify the same for `build`.

**Fix:** Add a row to the mode semantics table:
| `sifr build --workspace` | Cargo workspace members | compile all Sifr-capable packages in the Cargo workspace; Rust-only packages are built only if reachable from a Sifr package |

### G4: Cargo optional dependencies and Sifr features are unlinked

Cargo optional dependencies (`features = [...]`) and Sifr feature flags are not yet connected. The `sifr.toml` has no `[features]` section. If a Sifr package declares Cargo optional dependencies, the question is whether those dependencies are always compiled, always excluded, or controllable via Sifr-level feature flags.

**Fix:** Add `[features]` to `sifr.toml` with a mapping to Cargo features:
```toml
[features]
json = { cargo-feature = "json", cargo-package = "reqwest" }
rustls = { cargo-feature = "rustls-tls", cargo-package = "reqwest" }
```
`sifr build --features json,rustls` activates those Cargo features. Without this, optional Cargo deps are always compiled when reachable, which may increase binary size or introduce unwanted behavior.

### G5: No spec for `cargo_metadata` version stability

The plan correctly says "Production code should avoid direct dependency on Cargo's internal `cargo` crate APIs" and prefers the `cargo_metadata` crate + CLI JSON. But `cargo_metadata` is a third-party crate with its own release cycle. The plan's `DEPENDENCY_AUDIT.md` requirement covers this, but the plan should specify a minimum pinned version and a policy for upgrading.

**Fix:** Add to the `Cargo Integration Strategy` section: "Pin `cargo_metadata` to an exact version in `Cargo.lock`. Do not auto-update it without a targeted audit of breaking API changes. Log the `cargo_metadata` version in `crates/sifr_package/DEPENDENCY_AUDIT.md` with the Cargo `format-version` it was validated against."

### G6: Credential management for private registries is deferred but critical

The plan defers to "Cargo registry behavior" for credentials, but Sifr needs to handle the case where Cargo credentials are absent for a private registry. Does `sifr build` fail? Does it prompt? Does it read `CARGO_REGISTRIES_*` env vars? The answer affects production use of Sifr packages behind authentication.

**Fix:** Add `SIFR-PACKAGE-0105` ("cargo registry credentials unavailable") and define that Sifr delegates credential acquisition to Cargo: if `cargo metadata` fails due to missing credentials, Sifr surfaces `SIFR-PACKAGE-0105` with a remediation pointing to `cargo login` or `CARGO_REGISTRIES_*` documentation. Document this in the CLI contract section.

---

## Clarity Issues

### C1: "Direct dependency" scope boundary is under-defined

The plan uses "direct dependency" as the scope boundary multiple times but never defines it formally. Is it "a package P such that Q appears in P's `[dependencies]`"? Or "a package reachable in exactly one Cargo dependency hop"? The difference matters for the scoped import rule.

Consider `app -> lib_a -> lib_b -> sifr-math`. `sifr-math` is not a direct dependency of `app`. But if `app` has both `lib_a` and `lib_b` as direct deps, and both transitively depend on different versions of `sifr-math`, is `app`'s scope ambiguous?

**Fix:** Define "direct dependency of package P" as "any package Q where Q appears in P's `Cargo.toml` `[dependencies]` or `[dev-dependencies]` section, or any package Q reachable through exactly one Cargo dependency edge from P's selected Cargo package." Transitive deps (distance > 1) are never in P's direct dependency scope.

### C2: Alias metadata TOML location is ambiguous

The aliasing example uses `[package.metadata.sifr.aliases]` in `Cargo.toml`. This is the right place. But the validation rules section only says `manifest` must be relative to the Cargo package root and must match `sifr.toml`. It doesn't say the same for `aliases`. Are aliases validated against `sifr.toml`? If `aliases.math_v1` points to `dependency = "math_v1"` but `sifr.toml` has no corresponding alias, is that an error or a warning?

**Fix:** Add to the validation rules: "Every key in `[package.metadata.sifr.aliases]` must correspond to a Cargo dependency name in `[dependencies]`. The `import` field is the Sifr import name and is not required to match anything in `sifr.toml`." Alternatively, if aliases must be mirrored in `sifr.toml`, document that requirement explicitly.

### C3: Generated Rust module namespace for dependency packages is unspecified

When `sifr_codegen` compiles a dependency package's `.sifr` source, the generated Rust goes into a generated Cargo crate. The plan doesn't specify:

- The crate name (e.g., `sifr_http_generated`)
- The module structure inside (flat? nested matching the Sifr source tree?)
- Whether the dependency's `src/lib.rs` marker is included in the same crate or a separate one
- How visibility between generated dependency code and the main crate is controlled

This matters because generated module paths must not collide with user-generated Rust, with other dependency generated Rust, or with the marker `lib.rs`.

**Fix:** Add a section to "Cargo Integration Strategy" defining the generated crate naming convention: `[sifr-]{cargo_package_name}`. Module structure mirrors the Sifr source tree. The generated crate exposes a `pub mod` per `sifr/` subdirectory. The marker `lib.rs` is compiled in the same crate and may not re-export generated modules in a way that bypasses Sifr import boundaries.

---

## Minor Nits (non-blocking)

- **Lockfile freshness gap:** v3 intentionally has no `sifr.lock`, but for incremental compilation caching (`sifr build` with unchanged inputs), the derived graph digest in `target/sifr/graph-digest.json` must be checked before recomputing. The plan implies this but doesn't state it explicitly. Recommend: add a note that "The graph digest file under `target/sifr/` is the incremental cache invalidation key for package-aware builds."

- **SIFR-PACKAGE-0302 is missing from the diagnostic table** — `SIFR-PACKAGE-0301` is "backend native trust violation." `0302` through `0309` are unassigned. Define or reserve them.

- **`sifr update` recursive behavior** is mentioned but not specified. Does `--recursive` update transitive dependencies? Cargo's `--recursive` flag was removed in Cargo's redesign. The plan should say "delegates to `cargo update` with the same semantics" and note that `cargo update` operates on direct deps only unless `--recursive` equivalent is available.

- **Cross-platform source roots:** `sifr.toml` `[source].roots = ["sifr"]` is a string array. There's no mention of platform-specific source roots. Cargo supports `target.'cfg(...)'.dependencies`. Should Sifr? At minimum, the plan should note that Phase 37 does not support platform-specific source roots and define what happens if `.sifr` files appear only under platform-specific Cargo target directories.

---

## What's Already Resolved (from Round 1 feedback)

The following concerns from the Round 1 review are addressed in v3:

- `src/lib.rs` behavior is partially addressed (the non-goals say "pure Sifr packages still include a minimal Rust target" and "generated `src/lib.rs` marker should contain no semantic implementation") — **but B1 above still needs an enforcement mechanism**
- Derived lockfile concern: v3 intentionally has no `sifr.lock`, using `Cargo.lock` as the sole lockfile and `target/sifr/graph-digest.json` for cache invalidation — this is acceptable
- Cargo internals risk: correctly avoided with `cargo_metadata` + CLI JSON approach
- uv/Python interop: correctly deferred to Phase 43

---

## Summary Table

| Category | Count | Blocking? |
|---|---|---|
| Blockers | 4 | Yes |
| Production gaps | 6 | No (fixable in-milestone) |
| Clarity issues | 3 | No (clarify and document) |
| Minor nits | 4 | No (editorial) |

**Recommended path:** Resolve the 4 blockers before milestone_37_1 begins. The production gaps and clarity issues can be addressed as each milestone lands — the milestone scope descriptions already provide the natural integration points for these additions.
