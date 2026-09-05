I've reviewed the phase document and the full uncommitted diff against the two applied refinements and the core single-epoch requirement.

## Verification of refinement 1 (schema definition ≠ early activation)

`milestone_40_0` now carries lines 381–384: the v2 schemas define the final alpha/beta/stable enum during the cutover, but "schema acceptance does not activate stable behavior." This is consistent with the existing M40.0 DoD "No publication workflow can accept stable yet" (line 460), with M40.2's release-publication.yml "initially exposes only alpha and beta operations; no … stable-changing input exists until `milestone_40_5`" (line 596), and with the M40.5 GA-activation transition (lines 936–937). No ownership conflict is introduced — M40.0 still owns only schema/enum definition.

## Verification of refinement 2 (rc-removal split)

- M40.0 now solely owns JSON-surface rc/v1 removal: "replaces the existing install-receipt, `sifr self version --format json`, and self-update-plan schemas, producers, consumers, fixtures, and tests … Delete every v1 fixture and code path" (lines 374–380), reinforced by line 179 and the search-check DoD at lines 442–445.
- M40.2 is narrowed to "the remaining non-JSON runtime and workflow surfaces: installer `APP_CHANNEL` derivation, dispatcher exact-pin parsing, and `preview-release.yml` inputs, plus their tests and docs" and explicitly disclaims the JSON removals as already done in M40.0 (lines 578–584).

I checked for a coverage gap against the canonical rc-rejecting surface list (line 94–96: dispatchers, release metadata, receipts, workflows, generated installers, self-update). Every surface maps cleanly to exactly one milestone; the previously-M40.2 "Rust self-update fixtures" now fall under M40.0's self-update-plan consumer/fixture deletion. M40.2's full-coverage DoD (lines 649–651) fires only after M40.0 has merged (M40.2 depends on M40.0), so no surface is orphaned or double-owned. The core single-`schema_version: 2`-epoch requirement (lines 163–206) is intact — no v1, migration, negotiation, dual-format, or fallback survives.

**VERDICT: SATISFIED**

## Material findings

None.

## Non-blocking refinements

- Lines 382–383 list "Stable resolution, **planning**, and installation remain unavailable until `milestone_40_2`." Resolution and installation land in M40.2 (correct), but the stable release **planner** is implemented one milestone earlier — `milestone_40_1` "Implement one dry-run-first stable release planner" (line 484). This is a wording imprecision, not an ownership conflict (M40.1 unambiguously owns the planner; nothing in M40.0/M40.2 claims it). Consider dropping "planning" from the M40.2 deferral clause, or attributing it to `milestone_40_1`, so the sentence doesn't under-credit M40.1's stable planner. Non-blocking.
