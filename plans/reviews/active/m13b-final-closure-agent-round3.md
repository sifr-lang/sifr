## Code review: M13b final closure (round 3)

### Verdict

READY. No blockers. Confirming the round-2 verdict after two hygiene edits.

### Blocking findings

None.

### Post-round-2 edits verified

**Edit 1 — N1 addressed (crate-level doc reworded to retained-by-design glue).**
`crates/sifr_retained_intrinsics/src/lib.rs:1-4` previously read "Transitional
compiler-retained stdlib intrinsic signatures" / "This crate hosts fallback
signatures that still feed lowering and driver bootstrap while native stdlib
declarations continue replacing them." It now reads "Compiler-retained stdlib
bootstrap signature glue" / "This crate hosts retained-by-design fallback
signatures that feed lowering and driver bootstrap before sysroot source
declarations are available." That matches the frozen manifest state (the 19
modules under `retained-fallback-signature-glue` are `retained-by-design`) and
removes the mismatch flagged in round 2. Inline dispatcher comments at lines
70–71, 79–80, 82–83 remain on the round-1 wording ("Retained as compiler-owned
bootstrap signature glue for sysroot source lowering and reviewed by the final
retained-glue manifest"), so the crate is internally consistent.

**Edit 2 — N3 addressed (M13a row wording no longer implies all `_sifr.fs`
migrated).**
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:104` now reads
"M13a final sys boundary and remaining migratable `_sifr.fs` native leaves moved
through sysroot declarations, while builtin-`open` shadowing remains
retained-by-design language glue; agent review satisfied in round 2." This
correctly narrates that the migratable fs leaves closed while builtin-`open`
shadowing stays as language glue, matching the current manifest surfaces
(`internal_docs/stdlib_retained_compiler_intrinsics.toml:12-29` retains the
`builtin_open`/`builtin_open_text` and file-handle registry file entries as
`retained-by-design`). The row also flipped `planned` → `in progress` with the
M13a evidence cell populated; the M13b PR sha entry is (correctly) deferred
until the branch merges.

### Non-blocking notes carried forward

**N2 (from round 2) intentionally deferred: retained-dependency regex remains
line-shape-sensitive.**
`scripts/check_stdlib_native_intrinsic_allowlist.py:37`
`GENERATED_DEPENDENCY_PACKAGE_RE = re.compile(r'"([A-Za-z0-9_-]+)\s+=')` still
depends on the exact `"pkg = ..."` single-line shape in the non-test region of
`dependency_plan.rs`. Currently it captures exactly the 10 packages in the
manifest allowlist (`bigdecimal`, `metrics`, `num-bigint`, `num-traits`,
`rayon`, `rust_decimal`, `serde`, `serde_json`, `tokio`, `tracing`), and the
self-test at `check_stdlib_native_intrinsic_allowlist.py:436-448` plus the
manifest schema still catch drift on the allowlist side. The user explicitly
declined to redesign the marker/parser for M13b because the change would be
behavior-risking beyond closure scope; that is a reasonable call — the guard
degrades toward under-reporting the observed set, and any stale allowlist entry
that escaped detection would still be constrained by the manifest schema's
strict-fields rule. Consider revisiting under a follow-up hygiene item rather
than gating M13b on it.

**N4 (from round 2) partially advanced, remainder is post-merge bookkeeping.**
The M13 row now carries the M13a evidence entry and flipped to `in progress`.
The top-level `## Status` at
`plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:5` still says
`In progress.`, and the M13 row is missing the M13b PR sha / evidence line. As
called out in round 2, this is expected pre-merge state; the closeout edit that
flips `## Status` to final wording and appends the M13b PR sha to the M13 row
should land as part of the standard per-milestone closeout after the PR merges.

### Verification performed

- Diff review vs `origin/main` for
  `crates/sifr_retained_intrinsics/src/lib.rs`,
  `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`, and the 18
  other files in scope — the substantive changes (manifest reduction, schema
  strictness, retired-migration-guard fold, module-only dep leakage removal,
  fallback-signature-modules freeze, TLS trust derivation via crate features)
  match round-2 findings and remain confined to the closure scope.
- Confirmed `crates/sifr_retained_intrinsics/src/lib.rs:1-4` no longer contains
  the words "Transitional" or "while native stdlib declarations continue
  replacing them"; the new wording aligns with `retained-by-design`.
- Confirmed
  `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:104` now
  distinguishes migrated `_sifr.fs` leaves from the retained-by-design
  builtin-`open` glue.
- No new files touched outside those declared by the round-2 scope and the two
  hygiene edits; only prior review artifacts under
  `plans/reviews/active/m13b-final-closure-agent-round{1,2,3}.md` remain
  untracked.
- Round-2 non-blockers N2 and N4 verified as unchanged / partially advanced;
  neither reaches blocker severity for M13b.
- User-reported validation status (`git diff --check` PASS,
  `cargo fmt --check` PASS,
  `check_stdlib_manifest_schema.py` PASS with surfaces=11 /
  final_state=retained-by-design,
  `check_stdlib_native_intrinsic_allowlist.py` PASS with
  exact_intrinsics=27 / registry_files=11 / preamble_files=9 /
  fallback_signature_modules=19 / retained_direct_dependency_packages=10 /
  direct_runtime_roots=2,
  earlier full `run_all_tests.sh --profile create-pr` PASS with 129/129 e2e and
  warm wall-time advisory only) is consistent with a
  comments-and-docs-only follow-up on top of the round-2 baseline.

### Summary

Both hygiene items from round 2 (N1 wording drift, N3 M13a wording imprecision)
are cleanly addressed, no new blockers were introduced, and no code paths were
touched in this iteration. Branch is READY for PR. The remaining non-blocking
items (N2 regex fragility, N4 status/M13b evidence closeout) are appropriately
deferred to a follow-up hygiene pass and the standard post-merge closeout,
respectively.
