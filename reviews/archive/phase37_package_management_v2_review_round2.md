

## Round 2 Review — Phase 37 Package Management v2

I've reviewed the full v2 document. **Three blockers remain** before this can serve as the planning contract. Optional polish items are listed after blockers.

---

### Blocker 1 — Frozen mode exception is ambiguous (line 710, item 6)

The frozen mode table says:

> generated Rust/Cargo artifact inputs are already valid **or can be created without violating frozen-mode rules**. If generated artifacts are considered disposable build outputs, this exception must be documented and deterministic.

Item 6 in the frozen mode conditions says essentially the same thing in different words: regeneration is allowed only when documented as an exception.

**Problem:** These two paragraphs form a circular definition. The exception requires the exception to be documented before it can be used, but there's no documented list of what those exceptions are. This leaves implementers without a concrete rule.

**Fix:** Add a concrete definition of what counts as an acceptable exception. For example, any generated artifact that is explicitly tracked in `.gitignore` or equivalent (e.g., `target/`, `.sifr-gen/`) is a disposable build output and does not need per-artifact documentation. All other generated state (e.g., modified manifest files, lock files, extracted distributions) is not a disposable build output. Alternatively, if there are no acceptable exceptions in Phase 37, remove the "or can be created" clause entirely and require all generated state to be pre-validated.

---

### Blocker 2 — sdist content hash basis is ambiguous (line 455)

The graph digest includes "SHA-256 of all `.sifr` source files under selected package `[source].roots`."

**Problem:** A selected distribution may come from an sdist archive whose contents differ from the current source tree (e.g., a version tag or release commit that is behind or ahead of HEAD). The sdist is the artifact uv selected and that Cargo will compile against — but the hash is computed from the *current* source tree, not the sdist contents. This means the graph digest won't reflect the actual artifact that will be built, and `sifr build --frozen` could silently use a different source than what `uv.lock` and the sdist describe.

**Fix:** For sdist distributions, compute the hash from the archive contents (extracted after archive path validation), not the current working tree. For wheel distributions, the same applies — hash from archive contents after extraction. Only path, editable, Git, and workspace dependencies use the working tree hash (already covered at line 507-508). Clarify: "SHA-256 of all `.sifr` source files under selected package `[source].roots`, computed from the artifact contents for wheel/sdist distributions and from the current source tree for path/editable/Git/workspace distributions."

---

### Blocker 3 — Missing diagnostic code for Sifr-capable index claim vs. artifact verification failure

The document defines `SIFR-PACKAGE-0103` ("selected distribution lacks required Sifr metadata") and `SIFR-PACKAGE-0104` ("cannot locate Sifr metadata inside selected distribution artifact"). The detection logic at lines 186-189 includes this case:

> a configured Sifr package index/name map marks the distribution as Sifr-capable, **after which Sifr must still verify package metadata in the artifact**.

If the index marks a distribution as Sifr-capable but the artifact verification fails (no `sifr.toml` or `[tool.sifr]` found), which code fires? Neither `0103` nor `0104` covers this scenario — `0103` covers the case before artifact inspection, and `0104` covers the case where the artifact was found but metadata was unlocatable. The index-claim-then-fail case is a distinct failure mode.

**Fix:** Add `SIFR-PACKAGE-0105` between `0104` and `0201`:

```
`SIFR-PACKAGE-0105` | Sifr package index marks distribution as Sifr-capable but artifact verification found no Sifr metadata
```

Or merge this into `0104` with a qualifier: "cannot locate Sifr metadata inside selected distribution artifact, including when a configured Sifr package index or name map expected Sifr capability." Either approach closes the gap — pick whichever is cleaner for the diagnostic reference.

---

### Optional Polish (non-blocking)

1. **`SifrPackageGraph` lacks package alias support.** The spec says `sifr.toml` can hold "optional package aliases for Sifr import ergonomics," but `SifrPackageGraph` and `SifrPackageMetadata` have no alias field. Either add `import_alias: Option<SifrPackageName>` to the struct, or explicitly scope this as a Phase 38+ feature.

2. **`backend.offline-lock-optional = true` has no example.** The option appears in the Cargo section (line 396) but no example shows the TOML syntax. Add a concrete example in the manifest models section.

3. **Conflict handling for `--locked --frozen --offline` is undocumented.** The command modes table has four entries (default, locked, offline, frozen) but no entry for combinations. If `sifr build --locked --frozen` is valid, which mode wins? If it's an error, say so.

4. **Dependency declaration and alias remapping is underspecified.** The document says "a Sifr package can be published to PyPI or a private Python package index as a wheel/sdist containing `.sifr` source plus Sifr metadata" and gives examples where `pyproject.toml` `name = "sifr-json"` maps to `sifr.toml` `name = "json"`. But when a consumer declares `sifr-json` in `pyproject.toml` and imports `json`, the remapping mechanism is not documented. Either the consumer must declare the Sifr package name (the one in `sifr.toml`) in their dependencies, or there must be a remapping rule. Document which approach Phase 37 takes.

5. **Multi-lockfile workspaces (e.g., uv managed virtual workspaces with nested `uv.lock` files) are not addressed.** If a project has multiple `uv.lock` files, which one does Sifr read? Document the resolution rule.

---

### Summary

| # | Blocker | Location | Fix |
|---|---|---|---|
| 1 | Frozen mode exception circular definition | line 710 | Define what counts as acceptable exception (disposable build output boundary) |
| 2 | sdist/wheel content hash uses wrong source | line 455 | Use archive contents for wheel/sdist; working tree only for mutable inputs |
| 3 | Missing diagnostic for index-claim vs. artifact-fail mismatch | lines 806-807 | Add `SIFR-PACKAGE-0105` or clarify `0104` scope |

**Fix these three and v2 is ready as the Phase 37 planning contract.** The optional polish items can be addressed in implementation or follow-up refinements without blocking acceptance.
