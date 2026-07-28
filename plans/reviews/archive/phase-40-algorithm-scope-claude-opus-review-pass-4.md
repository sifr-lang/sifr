## Review — Phase 40 algorithmic release-scope, pass 4

**Verdict: APPROVED.** No blocking findings. Everything in the verification list is closed and I confirmed each item empirically (mutations run only in `/tmp/p4` and `/tmp/p5` copies; no repo file modified).

### Pass‑3 findings — verified closed

**1 MEDIUM (profile-derived divergence detection) — closed.** `coverage_matrix.py:370-400` now derives coverage per surface row by intersecting the row's `nightly_release_suite` area-suite tokens with `nightly.json` and `release.json` `selected_areas`, and requires `release_suite` whenever the two differ. It is wired into `main()` at `:84`, independently of `profile_assignment_matrix.json`. Both pass‑3 exploits now fail:

| Mutation | PAM | coverage_matrix |
|---|---|---|
| Delete PAM row + 3 CSM divergence fields | `ok: rows=16` (rc=0) | **rc=1** — `algorithmic_compatibility_profile: profile-derived release coverage diverges from nightly without release_suite` |
| PAM `nightly` under-declared to release's list + delete CSM fields | rc=1 (`nightly omits required suite`) | **rc=1** — same error |
| PAM `nightly`/`release` both reduced to a pure subset (`taxonomy-smoke`) + delete CSM fields — the *only* variant PAM still passes | `ok: rows=17` (rc=0) | **rc=1** — same error |

Zero false positives: I re-ran the predicate over all 34 rows against both profiles — `algorithmic_compatibility_profile` is the only diverging row and it declares `release_suite`.

**2 LOW (missing negative tests) — closed.** Three new cases delegate to production code: `profile_derived_release_divergence` (injects synthetic nightly/release profiles via the new keyword-only params), `equal_release_surface_suite` (`release_suite == nightly_release_suite`), `orphaned_release_divergence_metadata` (divergence metadata without `release_suite`, driven through `validate_surfaces`). Suite reports `cases=24` (was 21).

**3 LOW (empty `surface_id`) — closed.** `profile_assignment_matrix.py:143-145` now `continue`s. Direct probe on a row with `surface_id: ""`: exactly one error (`surface rows[0]: missing or invalid surface_id`) and an empty `release_suites` map — no spurious second `release_suite has no profile assignment row`.

### Other requested verification

- **Explicit release-suite content**: set-equality against `profiles.release` (`sid: release_suite does not match release profile assignment: advertised=[…] assigned=[…]`, including the empty-assignment case); tokens run through `validate_expected_tokens` (`references unknown suite algorithmic_compatibility:nope`).
- **Record/expiry fail-closed**: missing record, missing expiry, malformed expiry (`31-10-2026`), past expiry all error; dangling index link (monkeypatched `REPO_ROOT` with the target rewritten to `GONE.md`) → `release divergence record target does not exist: ALG-CORPUS`; unreadable `index.md` is `OSError`-guarded.
- **Targeted validation**: readiness `variants=4 failures=0`; `coverage matrix ok: guarantees=13 surfaces=34 temporary_rows=0 strict=yes`; `profile assignment matrix ok: rows=17`; self-tests `cases=24`; algorithmic `representative-subset` 12 + `taxonomy-smoke` 1 = **13 variants, 0 failures**; `sifr_verify --self-test` all pass (incl. resource-class selection); release plan emits `{"area":"algorithmic_compatibility","resource_classes":["default-local"],"suites":["representative-subset","taxonomy-smoke"]}`; `git diff --check` clean; all touched JSON parses; `file-size guardrails: PASS` (largest touched file 732 lines).
- **Release lane still pins the corpus**: `run_representative_subset` → `load_profile_manifest` → `validate_profile_manifest` enforces `expected_fixture_count == len(glob("*.sifr"))` (`runner.py:269-272`) and every declared category present (`:263-265`).
- **Scope**: nightly `leetcode-full` + `taxonomy-smoke` untouched and blocking; release row `status: blocking`, no advisory/non-blocking flag (the only `enforcement: advisory` in `release.json` is the pre-existing `cargo_cache_setup` step budget); inventory is exactly the 20 slugs, byte-identical; restoration is both a closeout gate and an acceptance criterion; `matrix_referenced_areas` gains no new strict area; no demo, fixture, baseline, or unrelated change.
- **Pass‑1/pass‑2 findings** all remain closed (README attribution, `release.json` description qualifier, `default-local` resource class, rejection inventory without "ownerless", guarantee-layer authority note, `milestone_40_1` cross-reference, `performance_budget_checks` in `full` mode, id-convention note at `profile_policy.md:141-143`).

### Non-blocking observations (not findings; no action required for this diff)

1. The profile-derived check keys off the row's own `nightly_release_suite`. Reducing *that* field to `algorithmic_compatibility:taxonomy-smoke` alongside the deletions makes both intersections equal and passes both checks. This needs an affirmative misstatement in a field whose fidelity was never gated, before or after this diff, and the obvious tightening (`declared ⊆ nightly selection`) is not free: `lowering_layer_snapshots` → `core_language:lowering_layer_inventory` and `runtime_platform_golden` → `runtime_platform:platform-rules` already reference suites no profile selects. Worth a separate issue on `nightly_release_suite` fidelity rather than a change here.
2. `selected_area_suite_tokens` and `is_area_suite_token` are now duplicated in `coverage_matrix.py:403-423` and `profile_assignment_matrix.py:156,243-247` with slightly different signatures; `coverage_matrix.py`'s `{cargo, e2e, sifr, sifr_codegen}` denylist is inert there, since `selected_areas` only ever holds real area names.

Note: `plans/reviews/active/phase-40-algorithm-scope-claude-opus-review-pass-4.md` is a zero-byte placeholder — per your instruction I did not write to it.

**APPROVED**
