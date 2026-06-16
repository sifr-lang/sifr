Read enough to assess. Findings below.

## Verdict

**No blockers.** The expansion is internally consistent and the validation already passes (variants=34, failures=0). Several non-blocking concerns worth addressing, but none require a fix before merge.

**Another review round is not required after merging this as-is.** A follow-up PR is the right vehicle for the items below.

## Findings

### Verified-correct (no action)

- **Source checksums match disk.** Re-computed all four `source_checksum_sha256` values from the working tree — exact match with manifests at `curated_manifest.json:8,32,56,80,104` and `ecosystem_broader_manifest.json:8,32,50,74`.
- **Squash-merge pin durability holds.** `git log --follow` on the existing `main.sifr` under each of the four project roots returns `f6ababa5dd…` for all of them. Because Wave 9.2 only adds new entrypoint files under *existing* project roots, every project root retains at least one tracked file whose `--follow` history includes `f6ababa5`. After a squash merge to `main`, that history is preserved verbatim on the existing files; only the new files lack pre-squash history (and the runner doesn't need them to — `project_revision_history` unions across all tracked paths). Pin contract is safe.
- **Source-checksum gate is correctly ordered before commands.** `oss_and_determinism.py:171-195` runs the checksum variant immediately after the pinned-revision variant and `continue`s on mismatch, satisfying the documented "fails as `source_checksum_mismatch` before commands execute" contract (`ecosystem_compatibility.md:30`).
- **No `failed_cases` double-count.** Each early-exit branch (metadata `:94-105`, pinned format `:112-122`, pinned unresolvable `:130-140`, pinned mismatch `:146-157`, checksum `:174-185`) increments `failed_cases` and `continue`s. `case_failed` (`:92`) is only set true inside the per-command loop, and the bottom-of-loop guard `:280-281` is only reached when none of the early exits fired. No path increments twice.
- **Variant arithmetic checks out.** 5 curated entries × (pin + checksum + check + run = 4) + broader (4+3+4+3) = 20 + 14 = 34. Matches the reported variants=34.
- **Suite/policy parity.** `suite_taxonomy.md:91-93` now lists source-checksum and SPDX-license as required for `oss-curated` and explicitly states `ecosystem-broader` shares the metadata contract — matching the runner's `required_missing` set in `oss_and_determinism.py:64-74`.
- **`is_sha256_hex` is appropriately strict.** Length 64 + lowercase hex (`:288`). Manifest values comply.

### Non-blocking concerns

1. **"Ecosystem breadth" framing is overclaimed.** (`curated_manifest.json`, `ecosystem_broader_manifest.json`)
   - Curated grows from 2→5 *entries* but project_roots are unchanged at 2 (`curated_cli_math`, `curated_data_flow`).
   - Broader grows from 2→4 entries but project_roots are unchanged at 2 (`broader_pass_signal`, `broader_known_failure_signal`).
   - The new fixtures are 2–10-line first-party language-feature exercises (e.g., `control_flow.sifr:1-12`, `option_scan.sifr:1-18`, `pipeline.sifr:1-7`, `type_mismatch.sifr:1-2`). They expand *entrypoint* coverage within the same four tiny first-party projects, not ecosystem coverage. The `ecosystem_limitations.md` doc honestly documents the absence of large public repos / external package graphs, so the policy itself is candid — but the wave's narrative should call this what it is (an entrypoint expansion, not an ecosystem one), or a follow-up should add at least one new project_root with genuinely third-party shape.

2. **Project-root reuse inflates variant count without adding signal.** `oss_and_determinism.py:171,159`
   - With 0001 and 0003 sharing `curated_cli_math`, and 0002/0004/0005 sharing `curated_data_flow`, the runner recomputes the same pinned-revision and source-checksum variant 3× for the second root. These are tautological — same root, same pin, same checksum — so they cannot fail independently. They pad `total_variants` without adding coverage.
   - Either cache the pinned+checksum verdicts per project_root (the pinned cache already exists at `:49`, just extend to checksum) and emit a single shared variant, or restructure manifests so each project_root appears once with a list of entrypoints.

3. **`project_source_checksum` hashes untracked files.** `oss_and_determinism.py:291-299`
   - `project_root.rglob("*")` plus `is_file()` walks the filesystem, not `git ls-files`. A developer with an untracked draft, editor backup, or OS-metadata file under a project_root computes a different digest than CI. `.gitignore` does not protect against this — the walker doesn't honor it.
   - With Wave 9.2 the checksum is now load-bearing in a merge-blocking gate, so the cost of a false-positive `source_checksum_mismatch` from environment noise is higher. Switch the walker to `git ls-files <project_root>` in a follow-up.

4. **License field is unvalidated beyond presence.** `oss_and_determinism.py:64-74` adds `license` to `required_missing` but never validates it's a real SPDX identifier. The policy at `ecosystem_compatibility.md:33-37` promises an SPDX contract; today `"license": "WhateverProprietary"` would pass. Either add an SPDX allowlist check or soften the policy wording.

5. **First-party fixtures declare MIT but carry no NOTICE/LICENSE artifact.** Every entry uses `"license": "MIT"` (e.g., `curated_manifest.json:9`) but no LICENSE/NOTICE file exists under the project_roots. The checksum can only lock what's there; a single string field is the entire license trail. A small `LICENSE` file per project_root would be locked by the checksum and gives the SPDX claim something concrete to point at.

6. **Variant labels in case results: `pinned-revision`/`source-checksum` vs. `<command>-<idx>`.** `oss_and_determinism.py:162,189,269` Mostly fine but the asymmetry (`source-checksum` is a single label, command variants are `check-1`/`run-2` numbered) shows up in case timing output (`runner.py:155-161`). Cosmetic, not a defect.

### Sanity items that look fine

- `pinned_revision_cache` correctly caches empty results (`oss_and_determinism.py:126-128`), so an unresolvable root doesn't get re-shelled per entry.
- New `.sifr` fixtures parse cleanly against current Sifr semantics (Option indexing on lists is shown in `option_scan.sifr:4` and `metrics.sifr:5` — existing pattern).
- `ecosystem_compatibility.md:59-61` correctly cross-links `ecosystem_limitations.md`.

## Recommendation

Land as-is. Open a follow-up captured under wave-9.x with: (1) at least one new project_root rather than another entrypoint on the existing four, (2) `git ls-files`-based checksum walker, (3) optional SPDX validator + LICENSE files inside each project_root, (4) shared per-root variant emission to deflate tautological counts.
