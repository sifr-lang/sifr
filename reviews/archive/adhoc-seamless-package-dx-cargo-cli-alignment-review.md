# Cargo CLI Alignment Review: `adhoc-seamless-package-dx.md`

## Verdict: NOT READY

**Two concrete blockers and several non-blocking gaps prevent this phase from passing alignment review.**

---

## Blockers

### BLOCKER 1: `sifr remove --target target` — Cargo remove does not support `--target`

**Location:** Line 606

```
sifr remove <crate>... [--dev|--build|--target target] [--dry-run] ...
```

**Cargo docs:** `cargo remove` Section Options:
- `--dev` — Remove as a development dependency
- `--build` — Remove as a build dependency

There is no `--target` flag in `cargo remove`. Cargo add supports `--target` because it can add a dependency for a specific target platform, but `cargo remove` does not expose target-specific removal. The Sifr signature incorrectly inherits `cargo add`'s `--target` flag.

**Required edit:** Remove `--target target` from the `sifr remove` signature and dependency section.

---

### BLOCKER 2: `sifr publish` missing `--no-verify` flag

**Location:** Line 611

```
sifr publish [--dry-run] [--workspace] [-p|--package spec] ...
```

**Cargo docs:** `cargo publish` Publish Options include:
- `--dry-run` — Perform all checks without uploading.
- `--no-verify` — Don't verify the contents by building them.

Sifr ships `--dry-run` (correct) but omits `--no-verify`. Both are standard stable flags in current Cargo.

**Required edit:** Add `--no-verify` to the `sifr publish` signature.

---

## Non-Blocking Suggestions

### G1: Explicit audit requirement missing from workflow

The document lists authoritative Cargo references (lines 576–589) and the milestone 3 scope (line 1062) mentions a "CLI alignment matrix checked into docs or test fixtures," but the document does not **require** implementors to verify against Cargo docs/help before each milestone closeout — only to reference the matrix.

Cargo CLI behavior changes. Stable flags can become nightly-only. New flags ship. The audit step should be an explicit required action, not a reference artifact.

**Suggested edit:** Add to the "Review Requirements" section (line 1167):

> Before each milestone closeout, the implementor must audit the current stable Cargo docs and `--help` output for each delegated subcommand to confirm all documented flags are present and undocumented flags are absent. Flag additions or removals must be documented in the alignment matrix before the milestone is marked complete.

---

### G2: `-C/--config` and `-C/--directory` handling not addressed

Cargo exposes:
- `--config KEY=VALUE` — override configuration
- `-C PATH` — change working directory (nightly-only, requires `-Zunstable-options`)

Neither appears in the document's CLI signatures. Sifr should decide whether these are blocked, delegated, or documented as Sifr-incompatible.

**Suggested edit:** Add a decision paragraph in the CLI Commands section:

> Sifr does not expose `-C/--config` or `-C/--directory` in the package-management CLI. The `-C/--directory` option is nightly-only in Cargo and adds scope without user benefit when Sifr already requires a manifest path. `-C/--config` is reserved for future tooling integration; Sifr-owned config takes precedence over Cargo config for Sifr-owned behavior.

---

### G3: `sifr package --target-dir` missing

**Cargo docs:** `cargo package` Miscellaneous Options:
- `--target-dir` _directory_ — Directory for all generated artifacts. Defaults to `target` in the workspace root.

**Current Sifr:** `sifr package` has no `--target-dir` flag.

This is a common option in Cargo packaging workflows. If Sifr delegates to Cargo, it should either support `--target-dir` or document why it is blocked.

**Suggested edit:** Either add `--target-dir directory` to the `sifr package` signature, or add a decision note stating this flag is not exposed and why.

---

### G4: `-j/--jobs` and `--keep-going` missing from build/check/test/package

Cargo exposes these for parallel compilation and failure tolerance:
- `-j N / --jobs N` — parallel jobs
- `--keep-going` — build as many crates as possible without aborting on first failure

These are stable in Cargo's build pipeline. The document's signatures omit them.

**Suggested edit:** Add to the build/check/test/package signatures:
- Add `-j N / --jobs N` to `sifr build`, `sifr check`, `sifr test`, and `sifr package`.
- Add `--keep-going` to `sifr build`, `sifr test`, and `sifr package`.

---

### G5: `sifr package --message-format json` requires `-Zunstable-options`

**Cargo docs:** `--message-format` _fmt_ for `cargo package --list` requires `-Zunstable-options` in nightly Cargo.

The document lists `--message-format` in the package signature without noting the nightly requirement.

**Suggested edit:** Add a footnote or decision note:

> `--message-format` for `sifr package --list` requires Cargo nightly (`-Zunstable-options`). This flag is delegated but the nightly requirement should be surfaced in diagnostics if the user attempts to use it on stable Cargo.

---

### G6: `--ignore-rust-version` missing from init/add/update/fetch/package

Cargo exposes `--ignore-rust-version` as a manifest option for several commands. The document omits it.

**Suggested edit:** Add `--ignore-rust-version` to `sifr init`, `sifr add`, `sifr update`, `sifr fetch`, and `sifr package` if Sifr delegates these. If deliberately omitted, document the rationale.

---

## Confirmed Clean Areas

The following user-mentioned concepts were verified absent from the document (good):

| Concept | Status |
|---------|--------|
| `[scripts]` | Not present — line 56 explicitly excludes scripts |
| `--script` | Not present |
| `--alias` | Not present as a flag |
| `--filter` | Not present |
| `--sifr-only` | Not present |
| `[test-dependencies]` | Not present — document uses `[dev-dependencies]` correctly (line 406) |
| `sifr fix` | Not present — document uses `sifr repair` (line 625) and explicitly explains why it is not `sifr fix` |
| `sifr package --dry-run` | Not present — cargo package has no `--dry-run`, Sifr correctly omits it |
| Custom selectors in CLI | Not present — advanced Phase 37 selectors are deferred (line 754) |
| Dependency groups beyond Cargo | Not present — line 432 explicitly excludes uv-style groups |

The `sifr repair` design is sound: it is Sifr-owned, avoids `cargo fix`, handles projection drift, and the internal migration script (`scripts/migrate_sifr_src_layout.py`) is properly scoped as an internal tool.

---

## Summary

| Item | Status | Type |
|------|--------|------|
| `sifr remove --target target` | **BLOCKER** — cargo remove has no `--target` flag | Must fix |
| `sifr publish --no-verify` | **BLOCKER** — missing `--no-verify` from cargo publish | Must fix |
| Audit requirement | G1 | Suggestion |
| `-C/--config`/`-C/--directory` | G2 | Suggestion |
| `sifr package --target-dir` | G3 | Suggestion |
| `-j/--jobs`/`--keep-going` | G4 | Suggestion |
| `--message-format json` nightly | G5 | Suggestion |
| `--ignore-rust-version` | G6 | Suggestion |

Once BLOCKER 1 and BLOCKER 2 are addressed, the remaining items are suggestions. The document is structurally sound and the Cargo-alignment intent is clear — it needs these two concrete fixes and one explicit audit step to be implementation-ready.