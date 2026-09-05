## Verdict: READY_WITH_NITS

Pass 1 blockers all resolved:
- `check_fixture_matrix.py:43` — `opaque_resource_core` added to `REQUIRED_FIXTURES` ✓
- `fixtures/opaque_resource_core/{fixture.json,README.md,positive/…,negative/…}` present ✓
- `check_sysroot_stdlib_resource_certification_gate.py` — suffix-based acceptance replaced by explicit `SUPPORTED_STDLIB_CORE_ROWS = frozenset({"opaque_resource_core"})` with positive+negative evidence status checks ✓
- Self-test updated to exercise the new allow path, the reject path, and the future-owner path ✓
- Doc row count bumped (14 → 15) and scope note added distinguishing core vs. matrix ✓
- Manifest `certification_rows` for `fs.sifr` retargeted `opaque_resource_matrix` → `opaque_resource_core` ✓

Nits (non-blocking):

1. **Evidence-file / test-source mismatch (mild overclaim risk).** `fixtures/opaque_resource_core/README.md` says positive+negative evidence is provided by `cargo test -p sifr_runtime interop`, but the `.sifr` fixtures reference symbols (`sifr_stdlib.fs.FileHandle`, `sifr_stdlib.fs.file_close`, `sifr_stdlib.fs.map_panic`) that don't necessarily exist yet at M3a. The row is marked `execution_kind: runtime-observed` — worth confirming the fixture matrix runner treats the `.sifr` files as declarative surface (like other opaque_* rows) rather than executing them, otherwise `status: passing` is aspirational. If the runner only cross-checks headers, this is fine — but the README should probably state that explicitly.

2. **Self-test coverage gap.** `_self_test` verifies rejection when a *non-allowlisted* row flips to supported, but does not verify rejection when `opaque_resource_core` itself is marked supported *without* passing evidence (e.g. `positive_evidence.status = "failing"` or missing `negative_evidence`). Since `_is_supported_stdlib_core` is the new load-bearing predicate, one direct negative test on it would harden the gate against regressions.

3. **Failure-message wording.** The new message (`"supported stdlib resource rows must be explicitly allowed core rows with passing evidence"`) fires for *any* non-future-owned category on any surface row — including e.g. an accidental `unsupported`. The wording is accurate for the intended case but slightly misleading otherwise. Minor.

No new correctness issues; another full pass is not needed. A quick follow-up to address nit 1 (clarify what "passing" means for this row) and nit 2 (one extra self-test assertion) would be sufficient before merge.
