

Now I have enough context. Let me write the review.

---

## 1. Verdict

**NOT READY** for these ergonomics to be added to the issue. The phase file has critical gaps for items 1, 2, and 3, and items 4–5 are partially addressed but need precision. The phase is well-structured for its current scope but the user-thought items introduce surface-area decisions that are not currently modeled. Treating them as "already handled" or "implicitly covered" would be a design failure.

---

## 2. Findings: Blockers and Gaps

### Item 1: `[[bin]]` vs npm-style `scripts` — **Gap: no mention at all**

The phase defines binary targets through `[[bin]]` in `sifr.toml` (lines 166–176, 211–224) but says nothing about named task/script aliases. npm's `scripts` table and Cargo's `[[bin]]` are fundamentally different:

- `[[bin]]` declares a Cargo target with a file path — it's a runtime artifact.
- npm `scripts` is a named alias that maps to any shell command string — it's not tied to a physical binary on disk.

The phase conflates these by treating `[[bin]]` as the Sifr script entry point. This is fine if Sifr intentionally rejects task aliases, but that choice must be explicit, not accidental. The current text leaves ambiguity: is `[[bin]]` a binary target, a task alias, or both? The resolution order (lines 218–223) reads like Cargo's `--bin` selection, not like a script runner.

**Concrete fix:** Add a "Scripts vs Binary Targets" section with explicit non-goal or goal statement. If task aliases are in scope, define the schema and interaction with `[[bin]]`. If they are intentionally out of scope, state it in Non-Goals.

### Item 2: `[test-dependencies]` and dependency groups — **Partial coverage: `dev-dependencies` exists, groups do not**

The phase has `[dev-dependencies]` (lines 383–385) but no dependency group model. uv-style groups (`[dependency-groups]`, `--group`, `--only-group`, `--no-group`, with `dev` included by default) are not modeled. The phase does not define:

- What happens when `sifr test` is run without `--only-group test` — does it include dev deps or not? (uv: yes, dev included by default.)
- How group membership interacts with `sifr run` vs `sifr test` — `sifr run` does not need test deps; should they be available?
- Whether `[dev-dependencies]` is permanent Sifr schema or transitional Cargo projection syntax.
- Whether future groups like `[lint-dependencies]` or `[build-dependencies]` are in scope.

**Concrete fix:** Add a "Dependency Groups" section. Define `dev` as the only group for phase 1. Specify that `sifr test` implicitly uses test-group deps and `sifr run` does not. Model `[test-dependencies]` as a future extension, not current scope. Decide whether `[dev-dependencies]` is Sifr-native or Cargo-projected syntax.

### Item 3: Manifest-less single-file execution — **Gap: completely absent**

This is a **critical gap**. The phase defines a package model (`sifr.toml`, `src/`, `__init__.sifr`) but never addresses what happens when the user runs `sifr run main.sifr` from a directory with no package metadata. The architecture.md states "`[source].roots` define workspace user-module search roots, defaulting to `[\".\"]`; malformed workspace config is a hard build diagnostic rather than a single-file fallback." But the phase does not resolve the contradiction: if manifest-less execution is a requirement, it cannot be a hard diagnostic.

The phase's `sifr.toml` contract section (lines 142–223) assumes a manifest is present. The CLI section (lines 510–549) assumes package-aware commands. There is no single-file bypass path documented.

**Concrete fix:** Add a "Manifest-Less Execution" section under Non-Goals or explicitly scope it. Define the resolution order: if `sifr.toml` exists, use package-aware mode; if `sifr.toml` does not exist but the target file exists, use single-file mode with a warning or a diagnostic code. The distinction matters because single-file mode bypasses the entire `PackageSession` and `PackageSourceMap`, which has cascading implications for compiler integration (lines 614–643).

### Item 4: `[package.metadata.sifr] manifest = "sifr.toml"` — **Partially addressed but imprecise**

The phase includes this in the generated Cargo projection (lines 482–484, 495) and in diagnostics (SIFR-PACKAGE-0703 at lines 495, 831). However:

- The phase never states *why* this lives in Cargo metadata instead of `sifr.toml` itself. (Answer: it's a discovery hook so `cargo metadata` can surface Sifr packages without reading `sifr.toml` — but this rationale is absent.)
- The phase does not specify whether `sifr.toml` should self-reference its own path or whether the Cargo metadata pointer is the only canonical reference.
- SIFR-PACKAGE-0703 (line 831) says "missing or incorrect `[package.metadata.sifr]`" — "incorrect" is underspecified. What makes it incorrect: wrong path, missing file, non-matching hash?

**Concrete fix:** Add a "Manifest Discovery" section explaining that the Cargo metadata pointer is a one-time discovery hook used by `cargo metadata` consumers, not a trust anchor. Clarify SIFR-PACKAGE-0703 to mean "manifest file referenced by metadata does not exist or fails to parse."

### Item 5: CLI compatibility with Cargo — **Partially addressed, missing edge cases**

The CLI section (lines 510–549) defines `sifr run`, `sifr check`, etc. with `--locked/--offline/--frozen` flags matching Cargo. However:

- **`--` argument separator:** Cargo passes arguments after `--` to the binary. Sifr's phase does not define this. Does `sifr run --locked -- --help` pass `--help` to the binary or to Sifr?
- **`package.default-run`:** Cargo supports `[package] default-run = "name"` to avoid `--bin` for single-binary packages. Sifr does not model this. Should `sifr run` infer a default binary target from a `default-run`-equivalent field?
- **`--message-format`:** Cargo supports `--message-format=json`. Does Sifr need this for machine-readable diagnostics? (Current diagnostics schema is defined in the phase but `--message-format` flag is not.)
- **`-p` vs `--package`:** Both exist in Cargo. Sifr uses `-p package` per the phase. Is `--package` an alias or is it absent?
- **`sifr run` with no package in scope:** Cargo errors: "manifest path `/foo/Cargo.toml` is a virtual manifest, but specifies `package`"; Sifr has no defined behavior for this.

**Concrete fix:** Add a "CLI Compatibility Surface" section. Define that `--` argument passing is implemented (state the semantics). Decide on `default-run` equivalence. Add `--message-format` to the CLI contract if JSON output is required for CI. Document `-p`/`--package` as the only package selection flag (not `--package` as a separate alias).

---

## 3. Recommended Model

### Scripts/Tasks

**Decision: Accept `[[bin]]` as the sole binary target model. Explicitly reject npm-style `scripts` table in v1.**

Rationale: Cargo's `[[bin]]` is composable with Sifr's package model. npm-style scripts are shell strings that hide complexity but also hide failures. Sifr's core guarantee ("if it compiles, it works") is incompatible with opaque shell strings. `[[bin]]` targets are compiler-verified entry points.

If task aliases are needed later, they should be a separate `[tasks]` table that maps to pre-defined shell commands (not arbitrary strings), but this is out of scope for phase 1.

**Schema addition:**

```toml
[[bin]]
name = "demo-app"
path = "src/main.sifr"
```

**Interaction:** `sifr run` uses the resolution order already defined (lines 218–223). No change needed, but the Non-Goals should explicitly state npm-style `scripts` are not in scope.

### Dependency Groups / Test Dependencies

**Decision: Adopt uv-style dependency groups for phase 1, with `dev` as the only group.**

The Cargo `[dev-dependencies]` is a Cargo concept. If Sifr projects to Cargo, Sifr's dependency groups must map to Cargo's `[dev-dependencies]` for the `dev` group, and be projected to Cargo groups (if/when stabilized) or kept as Sifr-only metadata for non-dev groups.

**Schema:**

```toml
[dependency-groups]
dev = []
test = []
lint = []
```

Sifr projects `dev` to Cargo's `[dev-dependencies]`. Non-dev groups are Sifr metadata only until Cargo has equivalent schema.

**CLI behavior:**
- `sifr test` implicitly uses the `test` group.
- `sifr run` does not use test/lint groups.
- `sifr add <pkg> --dev` writes to the `dev` group (maps to Cargo `[dev-dependencies]`).
- `sifr add <pkg> --group test` writes to the `test` group (Sifr-only until Cargo supports it).
- `sifr run --only-group test` makes test deps available (for integration testing scenarios).

**Remove** the current `[dev-dependencies]` table key (lines 383–385) and replace with `[dependency-groups].dev`.

### Manifest-Less Single-File Execution

**Decision: Support manifest-less execution as a separate code path that bypasses `PackageSession`.**

When `sifr run main.sifr` is invoked:
1. Sifr checks for `sifr.toml` in the current directory and parent directories (Cargo's discovery algorithm).
2. If `sifr.toml` is found: use full package-aware mode with `PackageSession`.
3. If `sifr.toml` is not found: use single-file mode, parsing and compiling only `main.sifr` without dependency resolution, without `__init__.sifr` namespace rules, and without package API validation.

**Key constraint:** Single-file mode must not silently become package-aware mode. If a `sifr.toml` exists in a parent directory, the command must either use it (with workspace implications) or report `SIFR-PACKAGE-0706` or a new diagnostic code with a clear message that single-file mode requires being outside a package directory.

**New diagnostic:** `SIFR-PACKAGE-0710`: "Single-file execution not available in package context. Found `sifr.toml` at `<path>`. Run from inside a package or remove `sifr.toml`."

**CLI:** `sifr run main.sifr` — file argument implies single-file mode when no package context exists.

### Cargo Metadata Pointer

**Decision: Keep `[package.metadata.sifr] manifest = "sifr.toml"` as a required discovery hook. Document the rationale explicitly.**

The pointer is needed because:
- `cargo metadata` JSON does not include file contents. Without the pointer, Sifr would need to scan every Cargo package root for `sifr.toml`, which is slow and fragile.
- The pointer is a one-time discovery mechanism, not a trust or integrity mechanism. The contents of `sifr.toml` are the source of truth; the pointer is just a routing key.

**Refined SIFR-PACKAGE-0703:** "Cargo metadata points to `sifr.toml` but the file does not exist, is unreadable, or fails to parse. Check that `[package.metadata.sifr].manifest` path is correct."

**Do not** add `package.default-run` to the metadata pointer. The phase already defines implicit `src/main.sifr` discovery (line 175). If a package has multiple binaries and the user wants a default, use a Sifr-specific field:

```toml
[package]
name = "demo_app"
default-run = "demo-app"  # Sifr-specific, not Cargo metadata
```

### CLI Compatibility

**Decision: Match Cargo CLI surface with targeted Sifr additions. Define `--` semantics explicitly.**

Additions beyond Cargo:
- `--filter` selector (already in phase, matches `--filter` in Phase 37).
- `--message-format json` for machine-readable output (add to CLI contract).
- `-p` (not `--package`) for package selection (Cargo also supports `-p`; match exactly).

Cargo-compatible behaviors to adopt explicitly:
- `sifr run -- arg1 arg2` passes `arg1 arg2` to the binary (not to Sifr).
- `sifr check --locked --offline` fails if any dependency source is unavailable locally.
- `sifr fetch --frozen` fails if `Cargo.lock` needs an update.

**Missing from current phase:**
```bash
sifr run [(--bin name) | (package)] [--] args...  # define --
sifr check --message-format json                   # add format flag
sifr --explain <code>                              # already present (line 541)
```

**Remove from consideration:** `package.default-run` in Cargo metadata. Use Sifr-native `default-run` in `[package]` if needed.

---

## 4. Long-Term Maintainability Risks

1. **`[[bin]]` vs `scripts` ambiguity creates future feature pressure.** If users request `scripts` repeatedly, the phase 1 decision to reject them will face rollback pressure. Mitigate by documenting the explicit non-goal and the reason (opaque shell strings conflict with "if it compiles, it works").

2. **Dependency groups projection to Cargo is undecided for non-dev groups.** If Cargo stabilizes `[dependency-groups]` and Sifr has been using Sifr-only metadata, a migration is required. If Cargo does not stabilize it, Sifr carries the full group semantics alone. This is a known trade-off; document it.

3. **Manifest-less execution bypasses `PackageSession` entirely, creating two compiler entry paths.** Every compiler change (type system, HIR, codegen) must be validated against both paths. Without explicit test coverage for single-file mode, regressions will surface only in user reports. Add `cargo test -p sifr_driver -- single_file_run` coverage in milestone 1.

4. **`[package.metadata.sifr]` is a Cargo-specific hook that can be removed or renamed by Cargo.** The phase documents Cargo stability assumptions (format-version 1, stable CLI subcommands) but does not specify a recovery path if the metadata field is removed. Add a fallback: if `[package.metadata.sifr]` is absent, Sifr searches for `sifr.toml` in the package root (accepting the performance cost for discovery-only scenarios). This fallback must be tested.

5. **CLI `--message-format` without a stable output schema creates compatibility debt.** If JSON output is added, its schema must be versioned. A `--message-format=json` added in phase 1 with an unstable schema becomes a compatibility anchor. Either commit to JSON schema stability now or defer the flag to a later phase.

6. **The `PackageSession` model assumes `sifr.toml` is always present for package-aware commands.** If manifest-less execution is added, the session must gracefully degrade when no manifest exists. This interaction is not modeled in the current `PackageSession` responsibilities (lines 550–612). The session layer must have a "no-manifest" path that short-circuits before Cargo metadata invocation.

---

**Summary of required phase changes:**

| User thought | Status | Required change |
|---|---|---|
| npm scripts vs `[[bin]]` | Not modeled | Explicit Non-Goal (reject scripts) + `[[bin]]` resolution documentation |
| Dependency groups/test deps | Partial (`dev-dependencies` only) | Replace with `[dependency-groups]`, add CLI group flags, define uv-compatible semantics |
| Manifest-less execution | Absent | Add new section with bypass path, new diagnostic SIFR-PACKAGE-0710, single-file test coverage |
| `[package.metadata.sifr]` pointer | Present but imprecise | Add rationale section, clarify SIFR-PACKAGE-0703, add fallback search behavior |
| CLI Cargo compatibility | Partial | Define `--` argument passing, `--message-format`, package selection flags |
