I've verified the plan against the actual repo state. Discovery snapshot is accurate. Now writing pass 4 review.

# Findings

## P0 — Plan-blocking

**None.** Pass 3's P1 items are all closed. The discovery snapshot is accurate (14 areas, 17 crates, profile/policy/schema files all match), and the resolved-decisions section now binds all the major implementation knobs (v2 schemas, registry paths, owner enum, hermetic policy, reference host, CPython serializer/oracle, fuzz/sanitizer/platform policy, closeout policy).

Verified-closed against pass 3 feedback:
- Diagnostic code catalog path pinned (`verification/areas/diagnostics/data/code_catalog.json`). ✓
- `shipped_guarantees.json` schema validation enumerated in Wave 0. ✓
- Wave 6.1 stale-binary guard. ✓
- Stdlib doctest scope concretized with the zero-example inventory escape hatch. ✓
- Decisions Log row for external-reviewer additions. ✓
- Wave 7 Miri/Loom skip records reason and reproduction. ✓
- Wave 10 profile-assignment matrix check. ✓
- `closes_in_wave` enum for `expected-missing` and `red-blocker` rows. ✓

Also verified locally against repo state:
- `profile.schema.json` and `area.schema.json` are strict and v1-only — plan's v2 migration story is sound. ✓
- `suite.schema.json` is the minimal three-field shape plan claims. ✓
- `run_crate_tests` hard-codes the 10-item crate list at line 307; the six omitted crates the plan calls out (`sifr_codegen`, `sifr_type_system`, `sifr_format`, `sifr_lint`, `sifr_source`, `sifr_ir`) match exactly. ✓
- `merge.json` `legacy_facade.e2e.fixture_manifest` points where the plan says. ✓
- 14 area manifests, 17 workspace crates — discovery snapshot table is complete. ✓

Local-first is preserved emphatically: §Decisions, §Resolved Implementation Decisions ("Hermetic local-first rule", "Local/CI parity"), and §Acceptance Criteria all say merge stays offline and CI cannot be the source of truth. No contradiction with the user's preservation requirement.

## P1 — Should fix before treating as final

**1. The user asked for "all decisions made"; one decision is still explicitly open in Wave 0.**
Line 250–251: "Wave 0 must either extend [`suite.schema.json`] or keep suite-specific metadata in area-owned data files validated by area checks." This is the only `either/or` in the resolved-decisions surface. Since the user's instruction was to resolve everything, pick one — either extend `suite.schema.json` to v2 alongside profile/area, or commit to area-owned suite metadata files. Recommend the latter because it's cheaper and the existing schema is minimal-by-design.

**2. Wave 5 lists 8 snapshot layers in tasks but only 7 sub-PRs.**
Sub-PR list at lines 537–543 enumerates 5.1 HIR through 5.7 crash/ICE. The task body at line 548 introduces an additional "parsed-source shape at the Sifr-owned boundary: `sifr_syntax` / `sifr_frontend`" layer with no sub-PR slot. Either fold parser-shape into 5.1 explicitly ("5.1 Parsed-shape and HIR-lowering snapshots") or add 5.0/5.8 for it. Implementers will otherwise dispute scope of 5.1.

**3. Several `cargo metadata` targets are tagged "phase decision required" in discovery but never resolved in §Resolved Decisions.**
Lines 226–244 mark these as needing a phase decision but don't get one:
- `sifr_driver/diagnostic_rendering_harness` bin classification (shipped/internal/tooling)
- `sifr_diagnostics/gen-diagnostic-schema` and `gen-error-docs` bins (tool vs shipped)
- `sifr_frontend/frontend_query_bench` bin (perf vs tooling)
- `sifr_stdlib/__test_fixture` feature + fixture bin profile assignment

Wave 1's cargo-metadata guardrail catches these at implementation time, so it's not a hard blocker — but the user explicitly said "make all decisions in the phase." Either add a one-line "Secondary bins/features default classification" row in §Resolved Implementation Decisions (e.g., codegen tools and harness binaries are `internal`, benchmark bins are `performance`, test-fixture features are `merge-blocking with default`), or state explicitly that this classification is delegated to the Wave 1 cargo-metadata guardrail and add an acceptance criterion that every workspace bin/feature has a profile classification by the end of Wave 1.

**4. `legacy_facade` migration strategy is silent under the v2 profile schema.**
Current `merge.json` is dominated by `legacy_facade` (matrix_suites, hardening_suites, e2e, crate_tests, distribution, generated_code_quality, performance_budget, thermal/memory policies). Profile v2 adds crate membership, network policy, reference host, profile-plan emission — but it's not stated whether v2 keeps `legacy_facade`, deprecates it, or replaces it field-by-field. Implementers must guess whether to migrate fields out, leave them, or invent a parallel "new world" namespace. Add one bullet under "Profile schema migration" in §Resolved Implementation Decisions stating the chosen path (recommend: keep `legacy_facade` as the migration surface, add new v2 fields alongside, and target `legacy_facade` removal in a follow-on phase — not this one).

**5. CPython hand-seeded smoke is merge-blocking but the python3 prerequisite isn't declared.**
Wave 6.0 makes CPython differential merge-blocking. That means every merge run requires `python3` locally. Sifr's verification framework already uses `uv` (`uv run --project verification`) so python3 is a de facto local dev dep — but the plan never says "python3 is a required local prerequisite, not a host skip." If a contributor's machine doesn't have a working `python3`, what happens at merge? Either add to "Hermetic local-first rule" or "Platform policy" in §Resolved Decisions: "python3 (matching `verification/pyproject.toml`'s minimum) is a required local prerequisite for create-pr/merge; absence is a setup failure, not a host skip."

## P2 — Tightening, not blocking

**6. Duplicate "Verified during this planning pass" sub-headers.**
Pass 3 flagged this. Lines 162 and 174 both have the same heading. Merge into one block.

**7. Wave 0 is the largest unsplit wave.**
Pass 1 advocated for sub-PRing large waves; Waves 2/5/6/9 are sub-PR'd, but Wave 0 lands schemas v2 + coverage matrix infra + shipped guarantees + advisory check + promotion switch + offline/hermetic fields + plan-equivalence skeleton + policy doc updates in one PR. Implementable but heavy. Optional split: 0.1 schemas v2, 0.2 coverage matrix area + shipped guarantees, 0.3 hermetic/offline + plan-equivalence + policy docs.

**8. Wave 4 and Wave 7 are also large.**
Wave 4 covers code catalog + baseline coverage + recovery fixtures + suggestion validation + stale-baseline metadata + profile renderer split. Wave 7 covers fuzz targets across 5 entrypoints + sanitizer lanes + Miri + Loom/Shuttle + promotion docs. Both implementable but heavy; consider whether to pre-empt a "sub-PR breakdown" note like Waves 5/9.

**9. The §Required Tracking Updates list of gate-expanding waves omits Wave 9.3.**
Wave 9.3 (package management) promotes to merge after 20 consecutive nightly green runs. The promotion PR is gate-expanding (adds package-management integration to merge crate or area list). Add Wave 9.3 to the gate-expanding wave list so its promotion PR records warm/cold wall time.

---

# Required Edits

If you want a quick clean-up before kickoff, here is the minimal patch.

**§Resolved Implementation Decisions — replace the "Profile schema migration" bullet and "Area schema migration" bullet with also-explicit suite-schema and `legacy_facade` decisions:**

```
- **Profile schema migration:** introduce `verification/schemas/profile.schema.json` schema version `2` for explicit crate membership, hermetic/network policy, profile-plan emission, reference host, and feature/target assignments. `legacy_facade` remains the migration surface inside v2 profiles; new v2 fields land alongside it and `legacy_facade` field removal is out of scope for this phase. Keep schema version `1` readable only for migration tests until Wave 1 closes; no profile may remain v1 at phase close.
- **Area schema migration:** introduce schema version `2` for owner, network mode, pinned corpus checksum/revision, skip policy, baseline metadata contract, and suite artifact declarations. Existing v1 area manifests migrate incrementally by wave, but Wave 10 blocks any remaining v1 manifest for a shipped stable surface.
- **Suite schema strategy:** keep `verification/schemas/suite.schema.json` at v1 minimal shape. Suite-level metadata (owners, normalizers, artifacts, baseline metadata, stale-baseline rules) lives in area-owned data files validated by per-area checks; no v2 suite schema is introduced.
- **Secondary bins/features:** default profile classification for non-test bins and features is `internal` (compiler harness/tools) or `performance` (benchmark bins). Each workspace bin and feature must carry a profile classification by Wave 1 close; the Wave 1 cargo-metadata guardrail fails if any first-party bin or feature lacks an explicit assignment.
- **Local python3 prerequisite:** `python3` matching `verification/pyproject.toml` minimum is a required local dev prerequisite for create-pr and merge profiles. Absence is a contributor setup failure, not a host skip. The CPython differential hand-seeded smoke is therefore merge-blocking without weakening local-first.
```

**§Wave 5 — replace the sub-PR list with 8 sub-PRs to absorb the Sifr-syntax parsed-shape layer:**

```
- 5.1 Parsed-source shape snapshots at the Sifr-owned boundary (`sifr_syntax` / `sifr_frontend` above the parser fork)
- 5.2 HIR-lowering snapshots
- 5.3 Name-resolution snapshots
- 5.4 Type and ownership fact snapshots
- 5.5 CFG and flow fact snapshots
- 5.6 Codegen IR or structured input snapshots
- 5.7 Emitted-Rust snapshots for stable constructs
- 5.8 Compiler crash/ICE contract and sentinels
```

**§Existing Facts To Verify — merge the two "Verified during this planning pass" blocks into a single block.**

**§Required Tracking Updates Per Wave — extend the gate-expanding wave enumeration:**

```
Gate-expanding waves include at least Wave 1, Wave 2.final, Wave 3, Wave 4, Wave 7, and Wave 9.3 (package-management merge promotion). Wave 2.0 and Wave 2.1..N do not require wall-time measurement.
```

**Optional polish (P2):** add a single sentence to Wave 0 noting that it may ship as `0.1 schemas v2`, `0.2 coverage matrix area + shipped guarantees`, `0.3 hermetic/offline + plan-equivalence skeleton + policy doc updates` if size becomes prohibitive — matches the Wave 2/5/9 sub-PR pattern.

---

# Verdict

**Ready after minor edits.**

The plan is substantively elegant and discovery-complete. All decisions material to Wave 0 implementation are in place except for the suite-schema strategy and the `legacy_facade` migration question — both one-line decisions, neither blocking implementation in spirit but explicitly open in letter. The remaining P1 items (Wave 5 sub-PR off-by-one for parsed-shape, secondary bin/feature default classification, python3 prerequisite, Wave 9.3 wall-time tracking) are text-level tightenings of the kind a single editing pass clears.

After the five-block patch above lands, this is implementation-ready and Wave 0 can begin without another review pass. No structural changes are needed; the fallback-paths/silent-quarantine impossibility at closeout is well-enforced by the §Closeout policy decision and §Non-Acceptable Closeout States list. Local-first authority is preserved cleanly.
