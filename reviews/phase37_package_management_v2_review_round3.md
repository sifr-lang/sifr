

## Final Review — Phase 37 Package Management v2

### Blocker Assessment

All five Round 2 items (3 explicit blockers + 2 promoted from optional polish) are resolved in the current text:

| Item | Round 2 ask | Status in v2 |
|---|---|---|
| Blocker 1: Frozen mode exception | Define concrete acceptable exception boundary | Line 711: "disposable generated-output roots such as `target/`, `.sifr-gen/`, or another path explicitly configured as generated output and ignored by source control" + explicit enumeration of forbidden writes |
| Blocker 2: sdist/wheel hash source | Use archive contents for immutable distributions; working tree only for path/editable/Git/workspace | Line 456: "computed from wheel/sdist artifact contents after archive validation for immutable distributions and from the current source tree for path, editable, Git, and workspace distributions" |
| Blocker 3: Missing `SIFR-PACKAGE-0105` | Add diagnostic for index-claim-vs-artifact-fail mismatch | Line 814: present in diagnostic table; line 506: present in detection flow |
| Blocker 4: `import_alias` field | Add to `SifrPackageMetadata` | Line 430: `import_alias: Option<SifrPackageName>` present in struct |
| Blocker 5: Mode combination semantics | Document `--frozen --locked --offline` conflict handling | Lines 715-717: all combinations documented with behavior |

### Remaining Items (all optional, non-blocking)

- **`backend.offline-lock-optional` example** — appears in Cargo section (line 396) but TOML syntax not shown in manifest models. Non-blocking since the option is documented and implementable.
- **Dependency declaration and alias remapping** — when `pyproject.toml` declares `sifr-json` but `sifr.toml` names it `json`, the consumer's dependency declaration mechanism is underspecified. This is a Phase 38+ interface design question; Phase 37 can proceed with the detection and graph-derivation model as-is.
- **Multi-lockfile workspaces** — uv managed virtual workspaces with nested `uv.lock` files are not addressed. Sifr can default to the workspace-root `uv.lock` for Phase 37 and document that in implementation.

### Verdict

**v2 is ready** as the Phase 37 planning contract. No blockers remain.

The three optional polish items are design questions that do not affect Phase 37's core deliverables (manifest linking, uv lock integration, package-aware import resolution, Cargo backend coordination, workspaces/tooling, packaging/publishing). They can be addressed in implementation planning or Phase 38 without blocking acceptance.

**v2 is the Phase 37 planning contract.** Proceed to milestone scoping and implementation planning.
