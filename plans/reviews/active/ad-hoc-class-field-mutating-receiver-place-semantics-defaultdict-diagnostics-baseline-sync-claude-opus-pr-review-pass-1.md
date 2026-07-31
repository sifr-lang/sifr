## Review: diagnostics baseline sync (192c21778, `origin/main..HEAD`)

> **Correction recorded after terminal review pass 2:** The retained code is
> `SIFR-NAME-0002`, not `SIFR-NAME-0001`. The two removed downstream
> expectations were stale false positives from before `defaultdict` was
> type-modeled; the binding now has the modeled `__sifr_defaultdict_list` type
> and the following index/append expression is well-typed. The poison-binding
> and cascade-suppression explanation below is superseded by this correction.

**Diff scope** — exactly one file, one hunk, no production code:
`verification/areas/diagnostics/fixtures/diagnostics/e2e_bare_defaultdict_constructor_rejected/baselines/check-compact.stderr.txt:1-4` → count line `3 errors` → `1 error`, and deletion of the `SIFR-STDLIB-0001` (main.sifr:4:21) and `SIFR-TYPE-0002` (main.sifr:4:5) lines. `SIFR-NAME-0002 …:3:14 undefined function: 'defaultdict'` is retained unchanged.

### 1. Does the baseline match current main behavior? Yes — byte-exact.

Reproduced with the already-built `target/debug/sifr` (no build run), using the runner's exact argv shape (`--diagnostic-format` is a global flag placed before the subcommand, per `verification/runner/sifr_verify/area_adapter.py:528-529`):

```
1 error, 0 warnings, 0 notes
E SIFR-NAME-0002 …/main.sifr:3:14 undefined function: 'defaultdict'
EXIT=1
```

`diff` against the committed baseline reports no difference (byte-identical, trailing newline included). stdout is 0 bytes, matching `check-compact.stdout.txt`; exit 1 matches both `check-compact.exit-code.txt` and `manifest.json:446-452` (`expect_exit_code: 1`). Both companion baselines correctly stay untouched.

### 2. Is dropping the two cascades fail-closed, or does it hide a valid primary error? Fail-closed — verified by probe.

The two deleted diagnostics were both on `main.sifr:4` (`groups["alpha"].append("beta")`), i.e. derived entirely from `groups` being bound to the unresolved `defaultdict(list)` call on line 3. Direct probes (temp files outside the repo) show the suppression is scoped to the poisoned binding, not blanket masking:

- Genuine, non-cascade index/method errors still fire: `x = 5; x["alpha"].append(...)` → both `SIFR-STDLIB-0001` and `SIFR-TYPE-0002`.
- Real container typing still diagnosed: `groups: dict[str, list[str]] = {}; groups["alpha"].append(...)` → `SIFR-STDLIB-0001 type 'None | list[str]' has no method 'append'`.
- Multi-error recovery past the poison is intact: unresolved call followed by unrelated errors → all three reported (`SIFR-NAME-0002`, plus two independent `SIFR-TYPE-0002` on later lines).
- Two independent unresolved calls → two `SIFR-NAME-0002`, so recovery does not stop at the first name failure.

Exit code stays 1 and the primary error is unchanged, so nothing is silently accepted. The removed lines were classic cascade noise on an error-typed binding; removing them from the expectation reduces noise without weakening the fail-closed contract.

### 3. Is any related baseline / manifest / fixture update owed? No.

- `data/baseline_metadata.json:923-937` — `source_hash` hashes `main.sifr` (`checks/code_baseline_coverage.py:333`), and the recomputed digest is `sha256:1773f96a…3455352`, identical to the recorded value. `main.sifr` is untouched, so no re-stamp is owed. `bless_reason`/`bless_reference` (PR #2572) remain accurate for the fixture's purpose; house precedent confirms these are not re-stamped on content-only re-bless (`a7a5df414b` changed a `check-json.stderr.txt` baseline without touching `baseline_metadata.json`), and `manifest.json` sets `baseline_metadata_rules: {required: false, source: "not-applicable"}`.
- No code-coverage loss: `data/code_baseline_coverage.json:775-783` anchors `SIFR-NAME-0002` to this fixture (still satisfied), while `SIFR-STDLIB-0001` is anchored to `e2e_stdlib_defaultdict_keyword_constructor` and `SIFR-TYPE-0002` to `hir_mixed_semantic_recovery` — neither depends on this fixture.
- No recovery-surface loss: this fixture is not listed in `data/recovery_surface_coverage.json`, so the multi-error requirements in `checks/code_baseline_coverage.py:354-385` are unaffected.
- Baseline trio completeness and file-ownership checks (`code_baseline_coverage.py:294-313`) are unaffected — no files added or removed.
- The parallel e2e fixture `crates/sifr/tests/e2e/fail/bare_defaultdict_constructor_rejected.sifr` needs no change: it carries the single `# expect-error[col=14]: SIFR-NAME-0002` marker, and markers assert code existence rather than exhaustive coverage (`crates/sifr/tests/e2e_support/harness_model.rs:511-512`), so it passes under both old and new behavior. `checks/baseline_hygiene.py` constraints (no `[Edddd]`, no `SIFR-TYPE-0001`) are satisfied.

### 4. Is standalone prerequisite scope appropriate? Yes.

The diagnostics baselines suite compares stderr byte-exactly, so before this commit `--area diagnostics --suite baselines` was red on untouched `origin/main` — an illegal red-gate state per `verification/README.md:92-93`. Fixing it in a one-file, production-code-free commit, separately from the class-field receiver work, keeps the phase-closure PR's diff attributable and lets the gate go green independently. Commit message (`test: sync bare defaultdict diagnostic baseline`) matches the change class.

### Non-actionable observation

The originating compiler change is not identified in the commit (1128 commits separate this baseline's last edit at `b481dea901` from main head). Provenance would normally be worth citing in the PR body, but it is not a defect here: the behavior was verified directly against the built compiler, and the probes above establish that the new output is principled cascade suppression with precisely scoped poisoning rather than lost diagnostics. Nothing to change in the diff.

Zero blocking findings, zero non-blocking findings.

SATISFIED
