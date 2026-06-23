# M7 Review — LSP and Tooling Sysroot Integration (pass 1)

Branch: `sifr-sysroot-stdlib-m7-lsp-tooling`
Reviewer: Opus 4.7
Scope: Audit current diff against `M7` tasks/acceptance/validation in
`plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md` lines 437–476.

## Verdict: FAIL

The PR delivers the easier half of M7 (source-origin enum, auxiliary sysroot
sources in the source map, stdlib symbol bucket, import-name → sysroot jump,
private exclusion from completion) but leaves several required tasks and
acceptance criteria unimplemented. The work as-presented is not ready to merge
under M7 — it should land as part of M7 only after the gaps below are closed
(or M7 scope should be explicitly split).

## Blockers

### B1. Sysroot mismatch / broken-sysroot diagnostics are not implemented

M7 explicitly lists three related tasks/acceptance items:

- Task: "Add tooling diagnostics when the editor process sees a broken or
  mismatched sysroot."
- Task: "Add CLI/LSP sysroot mismatch diagnostics that include the observed
  sysroot paths where available."
- Acceptance: "CLI and LSP report the same sysroot path for the same
  installation."

Grep for `sysroot_mismatch`, `SysrootMismatch`, `tooling diagnostic`,
`development sysroot`, and `dev.*sysroot` across `sifr_driver`,
`sifr_analysis`, and `sifr_lsp` returns no implementation. There is no
diagnostic code, no publish flow, no test asserting CLI and LSP observe the
same `ResolvedSysroot`, and no surfacing of the resolved sysroot path through
LSP notifications or `sifr/debugTrace`. The driver's `tooling_sources()`
(`crates/sifr_driver/src/stdlib/tooling.rs`) does call
`sifr_sysroot::resolve_sysroot(None)` and propagates failure as
`STDLIB_BOOTSTRAP_FAILURE` — but that's the bootstrap-failure path, not a
"mismatch with the CLI's view" diagnostic, and gives no observed-path detail.

This is the load-bearing M7 acceptance item ("CLI and LSP report the same
sysroot path") and it is unaddressed in code and tests.

### B2. `GeneratedSupport` and `CompilerSynthetic` origins are dead variants

`crates/sifr_frontend/src/source_maps.rs:81-82` defines both variants, but a
workspace-wide grep shows they are *never* assigned to any `SourceFileView` and
no production code path produces them. M7 acceptance requires:

> Source maps correctly distinguish public stdlib, private declarations,
> generated support, compiler synthetic, and user files.

A source map cannot "distinguish" an origin variant that no source is ever
tagged with. The M7 validation list also calls out "Source-map origin tests
for user files, public stdlib files, private declarations, generated support,
and compiler synthetic sources." The new test
`analysis_source_map_tracks_public_and_private_sysroot_origins`
(`stdlib_tests.rs:84-105`) covers only three of the five required origin kinds.

Either tag the generated-Cargo synthesized files and compiler-introduced
preamble/synthetic sources with the appropriate origin (and add tests), or
remove the variants and explicitly defer the requirement to a later milestone
via a plan-doc update. Today the diff does neither.

### B3. "Prefer public wrappers; expose private only in internal/developer contexts" is not implemented

Task: "Make go-to-definition prefer public wrappers for user code and expose
private declaration links only in internal/developer contexts."

The current implementation makes only a binary choice: private declarations
are loaded into the source map and source-text/path retrieval works for them,
but they are completely absent from the stdlib symbol bucket
(`stdlib_navigation.rs:46-49` filters to `SysrootPublicStdlib` only). There is
no concept of an "internal/developer context" anywhere in `sifr_analysis` or
`sifr_lsp`, no toggle, capability negotiation, or environment flag that opts
into private-symbol resolution, and no tests covering the developer path. The
task as written is not satisfied — it asks for two distinct behaviors gated on
context, not a single "always hide private from definition" rule.

If the intent is to defer the dev-context surface to M8 (Rust interop for
private declarations), say so explicitly in the issue plan and adjust M7
scope. As-is, the task is silently dropped.

## High-severity concerns

### H1. Hand-rolled, non-parser symbol extraction in `stdlib_navigation.rs`

`stdlib_navigation.rs:55-115` re-discovers stdlib symbols by scanning lines for
the literal prefixes `def `, `class `, and a `name [:|=]` heuristic. The
project already has a real parser (`parse_module`) and a real symbol-from-HIR
extractor (`symbols_from_hir`) that the user-code symbol index uses. Reasons
this is a problem:

- It is parser-divergent: anything the real frontend will accept that this
  scanner misclassifies becomes a definition/completion bug that only the
  stdlib bucket exhibits. Today it already misbehaves on patterns the real
  parser handles fine (multi-line `def` signatures with a trailing-paren
  newline, decorated definitions whose `def` is preceded by whitespace, and
  `async def`).
- `constant_name` matches the first `:`/`=` on a line and then asserts the
  prefix is a bare identifier. That works for `MY_CONST: int = 5` and
  `MY_CONST = 5` but also fires for lines like `_ = foo()` (rejected by
  `public_name`, OK) — and it can interact with multiline string content in
  unpredictable ways once stdlib `.sifr` modules grow docstrings (no string
  state tracking).
- The current architecture loads sysroot sources as "auxiliary sources" that
  are deliberately not part of the module graph and not lowered to HIR. The
  fix is to either lower them like first-class modules (so the existing
  symbol-from-HIR pipeline produces stdlib bucket entries with real ranges
  and kinds) or invoke `sifr_python_parser` on each loaded source to get a
  real AST. Both are far less fragile than re-implementing a Python-ish
  toplevel scanner.

This will erode quickly as stdlib content grows. Worth fixing now before more
features lean on the stdlib bucket.

### H2. Definition routes to sysroot only on the import-name token, not the use site

`stdlib_navigation::stdlib_import_target` only fires when the token's owning
line begins with `from sifr.* import …`. That is the *declaration* in user
code, not the *use* site. The new LSP test
(`session.rs:514-539`) clicks position `(line 0, character 24)` — i.e. the
`randint` token in the `from sifr.random import randint` line — so the test
passes, but a user clicking `randint(1, 2)` on line 4 still resolves through
`locations_for_name` and lands on the import name in their own file.

The M7 acceptance bullet says "go-to-definition for a `sifr.*` *import* lands
in installed sysroot source" so the letter of the spec is met, but the
behavior users actually exercise (jump-to-def on the call) is unchanged. This
is the place the editor experience differs most from rust-analyzer / pyright
and should at minimum get a test of the call-site behavior and a follow-up
note in the issue plan if it's deferred.

### H3. `host.files()` semantics quietly changed

`host.files()` used to return only user-mapped files (`file_to_module.keys()`).
After this change it returns every file in `source_map().files`, which now
includes the sysroot aux files. Callers I checked are safe:

- `workspace_diagnostics` and `discover_tests` use `file_to_module.keys()`
  directly, so they continue to skip aux files.
- `analysis_workspace::file_maps`/`uri_by_file`/`file_maps_for` consume the new
  superset deliberately.

But there is no test pinning the new contract, and `document_symbols(file)`,
`diagnostics(file)`, `definition`, `hover`, `completion`, etc. all call
`module_for_file(file)?` which still returns `UnknownFile` for an aux FileId.
If an editor follows a sysroot URI and asks for any of those, the LSP will
return an internal error rather than a clean "not supported here" response.
The aux files now appear in the LSP file map and have file:// URIs, so this is
reachable: click-through-to-definition lands on a real sysroot file, the editor
then asks for foldingRanges/documentSymbols, and the request fails.

At minimum, document the constraint and add a path for "aux file: skip module-
dependent queries cleanly." Better: keep `host.files()` user-only and add a
separate `host.all_files_with_origin()` for the LSP file-map use case so the
contract is explicit.

## Medium / quality

### M1. `auxiliary_source_states` does the overflow check twice

`graph_cache_and_queries/loaders.rs:11-22` runs `usize::checked_add` then
`u32::try_from` on the result, with two near-identical error returns. Collapse
to a single `u32::try_from(module_count.checked_add(offset)?)`.

### M2. `range_for_name` round-trips u32→usize→u32

`stdlib_navigation.rs:160-164` writes
`u32::try_from(usize::try_from(start).ok()?.checked_add(name.len())?).ok()?`
where `start` is already `u32`. The inner `usize::try_from(start).ok()?` is
infallible on 32+-bit platforms but reads as if it might fail. Simpler:
`u32::try_from(name.len()).ok().and_then(|len| start.checked_add(len))`.

### M3. Symbol index revision/refresh churn

`refresh_stdlib_symbol_bucket` runs on every `symbol_index()` rebuild and on
every `refresh_existing_symbol_index` call, even though the auxiliary sources
are immutable after construction. Not a correctness issue, but the stdlib
scan runs every time a dirty user module triggers a partial refresh. Worth
caching the stdlib bucket and only rebuilding when the session reloads with
new aux sources.

### M4. Test fixture asserts hover signature shape, not source

`hover_request_for_stdlib_call_reflects_installed_signature` (`session.rs:570`)
asserts on `randint`'s signature string. That signature comes from the
external-defs pipeline (`stdlib_external_defs`), which has shipped with stdlib
type info since well before M7. The test passes today even if the sysroot
auxiliary source for `sifr.random` is silently empty — it never reads the
loaded sysroot text. The acceptance bullet "Hover and completion reflect the
installed stdlib version" needs a test that ties hover output to the actual
on-disk sysroot source (e.g. patch the dev sysroot's `random.sifr` to add a
new public function, then assert the new function appears in hover/completion
without rebuild). Otherwise this acceptance is partially under-tested.

### M5. Issue-plan delta understates the gap

The diff to `plans/issues/active/ad-hoc-sifr-sysroot-stdlib-toolchain.md`
moves M7 to "in progress" with an evidence line listing only what the PR
*did* implement, without acknowledging the unimplemented tasks (B1, B3) and
the half-implemented acceptance bullets (B2 origin coverage, H2 call-site
navigation, M4 hover sourcing). Either implement the missing pieces or update
the plan to explicitly defer them (with the new milestone they belong to) so
M7 closes against a true scope.

## What the PR does well

- Source-origin enum is clean, defaults to `UserSource`, and threads through
  `SourceFileView`/`ModuleGraphNode` and the `WorkspaceAuxiliarySource`
  pipeline coherently.
- `WorkspaceAuxiliarySource` plumbing through every existing `WorkspaceSession`
  / `FrontendContext` constructor is mechanical but consistent — no
  accidentally-dropped overload, and the existing tests update along.
- Splitting `FrontendContext` loaders into `graph_cache_and_queries/loaders.rs`
  is the right kind of decomposition (keeps the main file under the 900-line
  cap and groups responsibility by lifecycle phase).
- Frontend `SourceMapView::files` now actually contains the auxiliary sources,
  so LSP-side file maps can build URIs for sysroot click-throughs without a
  separate side channel.
- The completion-negative test (`labels.iter().any(|l| l.starts_with("_sifr"))
  == false`) is the right shape for proving private exclusion.

## Suggested next steps

1. Decide whether B1/B2/B3 are part of M7 or get split into M7a/M7b with the
   plan doc updated to reflect the split before opening a PR.
2. Replace the hand-rolled scanner with parser-driven extraction (H1).
3. Add a use-site definition test for `randint(1, 2)` and route call-site jumps
   through the stdlib bucket if the spec intent is "user-actionable navigation"
   (H2); otherwise leave a code comment and a plan-doc note that this is
   deferred.
4. Pin `host.files()`'s new contract with a test and decide whether to split
   user-files vs all-files (H3).
5. Strengthen the hover/completion test so it actually reads from the on-disk
   sysroot source (M4).

## Validation observations

The PR description lists `cargo test -p sifr_frontend -p sifr_analysis -p
sifr_lsp` and `cargo fmt --check` as passing. That's the right minimum, but
the M7 validation section explicitly calls for "Editor corpus snapshots updated
for sysroot-backed stdlib locations" — I see no snapshot updates in the diff.
Either confirm the snapshots are stable (no diff expected) and document why,
or update the corpus and include the snapshot file changes.
