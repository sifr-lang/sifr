# TypeScript-Go Architecture Transfer: Module Signatures And Dependency Invalidation

module-signature invalidation model makes frontend invalidation dependency-sensitive before structural cache reuse
exists. `FrontendContext` still owns process-local parse/lower/diagnostic caches,
but source updates now classify the change by import and export signature before
clearing cached query results.

## Signatures

`sifr_frontend::module_signatures` computes three deterministic signatures:

- `ImportSignature` from local `from module import name` statements;
- `ExportSignature` from public top-level functions, classes, constants, and
  public class members;
- `ModuleSignature`, which stores both import and export signatures for each
  frontend module.

Export signatures intentionally exclude function bodies so private
implementation edits can stay local when parameter, return, decorator, class,
and public field/member shapes are unchanged.

## Reverse Dependencies

`FrontendContext::rebuild_edges` now also builds reverse dependency edges from
the module graph. When a module's public export signature or import graph changes,
`update_module_source` invalidates the changed module plus the transitive reverse
dependency closure. Private body edits with unchanged import/export signatures
clear only the changed module's parse/lower/diagnostic/analysis caches.

module-signature invalidation model's reverse-dependency closure is module-graph based. Package/config reads and
failed lookup dependency records remain available on workspace snapshots and
continue to degrade through graph/workspace dirty scopes until later package and
watcher registry work can map those records to narrower module owners.

The global external-definition map remains process-local. When a signature or
graph change requires broader invalidation, the frontend rebuilds external
definitions from still-valid lowered modules before later queries re-lower the
invalidated closure.

## Dirty Scope Reports

`InvalidationReport` now carries the selected `WorkspaceDirtyScopeReport`:

- version-only updates report `None` with `DocumentVersionOnly`;
- private body edits report `OneModule` with `SourceTextChanged`;
- public export changes report `ReverseDependencies` with
  `ExportSignatureChanged`;
- import graph or parse-uncertain changes report `GraphStructure` with the
  relevant reason set.

`AnalysisHost::update_document` records that frontend-selected dirty scope in
the owning `WorkspaceSession`, so analysis snapshots preserve the same
dependency-sensitive invalidation report.

Class member export signatures include public method declaration shape and
public field shape, not method bodies. A private implementation edit inside a
method should therefore stay local when the class surface is unchanged.
