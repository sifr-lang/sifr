Now I have enough context to write the review.

## Code review: M13b final closure

### Blocking findings

None.

### Non-blocking notes

**N1. Empty review placeholder file is untracked but present in tree.**
`plans/reviews/active/m13b-final-closure-opus-round1.md` is 0 bytes and untracked (per `git status`). Either populate it with the actual round-1 review response before merging (matching prior milestones' pattern) or remove it — an empty file in `plans/reviews/active/` is confusing on `git log` grep and would look like a lost artifact.

**N2. Phase tracker not yet updated to reflect M13b landing.**
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md` still shows M13 as `in progress` with only the M13a evidence cell (PR #2916). This is expected pre-merge, but the closeout rules require the row to gain the M13b PR sha and the top-level `## Status` line to flip to the final wording once merged. Also, the summary line at the top of the doc (`In progress.`) should be flipped when this lands — that's the ordinary M13-closeout housekeeping.

**N3. `fallback_signature_modules` are observed but no longer gated against any manifest field.**
`scripts/check_stdlib_native_intrinsic_allowlist.py:93-97` collects `_sifr.*` retained fallback signature modules from `sifr_retained_intrinsics/src/lib.rs` (19 hits today), but `_validate` never compares that observation to any manifest field — `fallback_signature_modules` is absent from the `allowed` dict (line 116-122) and the comparison loop at line 192-195 skips keys that aren't in `allowed`. On `main` this check only fired for `closing` surfaces; since there are no more `closing` rows, the observation has become purely informational (the reported count in the PASS line).

Concretely, `crates/sifr_retained_intrinsics/src/lib.rs:60-88` still registers 19 `_sifr.*` intrinsic modules (`_sifr.io`, `_sifr.math`, `_sifr.uuid`, `_sifr.url`, `_sifr.regex`, `_sifr.compress`, `_sifr.toml`, `_sifr.datetime`, `_sifr.html`, `_sifr.calendar`, `_sifr.i18n`, `_sifr.encoding`, `_sifr.unicode`, `_sifr.bytes`, `_sifr.collections`, `_sifr.json`, `_sifr.test`, `_sifr.runtime`, `_sifr.task`). None of those module names appear as manifest surface ids, so if any newly reintroduces stdlib behavior via a fallback signature table, no guard will fail.

Whether this is a blocker depends on how strictly one reads the plan text "Every remaining compiler-native stdlib-adjacent surface appears exactly once in the retained-glue manifest as `retained-by-design`." A defensible read is that fallback signature tables for stdlib-lowering bootstrap are language-glue and outside the manifest's remit, in which case dropping the ignored observation from the observed dict (or adding an assertion that the observed set is a subset of an explicit ledger) would make the intent explicit either way.

**N4. Stale comments in `sifr_retained_intrinsics/src/lib.rs` still describe fallbacks as transitional.**
Lines 70–71, 79–80, 82–83 say "Retained as a stdlib-lowering bootstrap fallback while these leaves migrate to compiled private declarations." With final closure, these are either permanent bootstrap language glue (comments should reflect that) or leaves that should also be migrated (out of scope for this PR). The file isn't in the M13b diff, so this is a follow-up note rather than a request to touch it here.

**N5. `_validate_final_transitions` self-test lacks an explicit negative for a new `closing`/`pilot`/`retained` row.**
`scripts/check_stdlib_manifest_schema.py:361-375` only exercises the happy path (new row with `state=retained-by-design` accepted). The rejection path at line 149-155 (`new manifest rows must be retained-by-design`) is logically correct but is not directly self-tested. Non-blocking; a one-line addition would tighten the guard against future refactors.

**N6. Regex used to observe direct-dependency packages is coupled to the exact format of `retained_dependency_specs`.**
`scripts/check_stdlib_native_intrinsic_allowlist.py:37` uses `r'"([A-Za-z0-9_-]+)\s+='` against the non-test portion of `crates/sifr_stdlib_manifest/src/features/dependency_plan.rs`. It works today (10 packages captured, matches the manifest allowlist), but relies on the specific `"pkg = { ... }"` string shape produced by `retained_dependency_specs`. If someone reformats specs (e.g., `"pkg={ version = ... }"` with no space, or splits a spec across lines), the guard silently sees fewer packages. Consider anchoring the extraction to the `retained_dependency_specs` function body via a named marker (e.g., a `// GUARDRAIL: retained-direct-dependencies` comment span) or scan for the whole Rust literal pattern rather than a string-prefix. Minor hygiene.

**N7. `scripts/check_source_crate_dependency_direction.py:106-113` retains forbidden-pattern regexes for `GeneratedCargoDependency`, `StdlibFeatureSpec`, and `STDLIB_FEATURE_SPECS`.**
These types no longer exist even inside `sifr_stdlib_manifest`. The patterns still act as no-regression guards forbidding any other crate from reintroducing them, which is the intended function; noting so a future reader doesn't remove them thinking they're dead. Not an issue.

### Verdict

READY. The diff cleanly executes the M13b intent: manifest reduced to `retained-by-design` rows only; schema hardened to the final state with correct base-vs-current transition rules; migration-closure script cleanly folded (stale-arch phrases + deleted ownership-registry check retained in the allowlist guard) and its runner/policy wiring removed; direct third-party dep emission is now confined to the 10-package retained compiler-language/runtime glue set gated by explicit `required_features`; stdlib module selection alone emits only sysroot crates (verified by the reworked `stateless_sysroot_leaves_do_not_emit_direct_third_party_dependencies` and the shrunk network/TLS/IPC snapshots); driver TLS trust now correctly derives from sysroot-crate features rather than package-name scraping. Locally reported guard counts (surfaces=10, exact_intrinsics=27, registry_files=11, preamble_files=9, retained_direct_dependency_packages=10, direct_runtime_roots=2, fallback_signature_modules=19) are consistent with the manifest and observed surfaces. The 7 non-blocking notes above are hygiene, not correctness.
