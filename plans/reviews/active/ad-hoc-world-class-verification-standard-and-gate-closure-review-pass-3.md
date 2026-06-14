Confirmed: all crates referenced in focused validation (`sifr_analysis`, `sifr_lowering`, `sifr_diagnostics`, `sifr_runtime`, `sifr_lsp`, etc.) exist, all four profile JSONs exist, and the policy/area files referenced exist. Now writing the review.

---

# Findings

## P0 — Plan-blocking

**None.** All pass-2 P1 items are closed, and the external reviewer's 14 must-fix items are integrated coherently. Local-first validation is preserved emphatically in §Decisions, §Scope, §Acceptance Criteria, and §Non-Acceptable Closeout States. No text-level contradictions remain.

Specifically verified-closed against the external feedback:
- **Shipped guarantee registry** (`shipped_guarantees.json`) is now Wave 0 with guarantee_id, support status, public_doc, owner, surfaces, regression surface, unsupported-behavior policy.
- **Owner-cannot-be-unassigned** is explicit across all row types.
- **`red-blocker` status** is defined in §Decisions and applied to `sifr_codegen` in Wave 1 with `closes_in_wave: 2`.
- **Stale/unused baseline detection** is in Wave 4 (diagnostics) and Wave 5 (snapshots) with metadata requirements.
- **Hermetic/offline** is in §Decisions and Wave 0 with profile fields (`network: offline|live`, pinned revisions, reference host, etc.).
- **Cargo feature/target coverage** is the Wave 1 `cargo metadata` guardrail, and there is a dedicated row in the profile-assignment table.
- **Generated Rust gates** (emit → normalize → rustfmt → cargo check → clippy → run → panic/unwrap scan with allowlist policy) are in Wave 2.1..N.
- **Crash/ICE contract** is Wave 5.7 with sentinel rules and "if a known crash stops crashing, the sentinel fails."
- **Parser acceptance/rejection** is folded into Wave 3 with the matrix coverage and contradiction checks.
- **CPython exactness** (divergence catalogue with 9 enumerated excluded axes, generator linter, hand-seeded merge smoke before generation, shrinker required, stdout byte-equal, exit-code bucketing, no message comparison, release-binary reuse) is in Wave 6.0/6.1.
- **LSP transcript replay** is in Wave 9.1 alongside markers.
- **Perf memory/size trends** (peak RSS, emitted Rust lines/bytes, binary size, LSP latencies, package install time, stale-baseline check, benchmark-id stability) are in Wave 8.
- **Platform support manifest** (`supported_platforms.json`) is in Wave 9.5.

The plan grew substantially but did not become over-broad: each addition has an owner-row, a closing wave, and an acceptance criterion. Nothing is aspirational policy without a failing check.

## P1 — Worth tightening, not blocking

**1. Diagnostic code catalog file path is not pinned.** Wave 4 task: "Add a diagnostic code catalog with code, severity, stability, owner, docs link, renderer support, suggestion applicability, and machine-application applicability." The companion `code_baseline_coverage.json` path is named, but the catalog itself has no path. Implementers will guess. Suggest `verification/areas/diagnostics/data/code_catalog.json` or similar.

**2. `shipped_guarantees.json` schema validation is implicit.** Wave 0 says owner cannot be `unassigned`, but does not say the matrix check must also reject: unknown `status` enum values, missing/dead `public_doc` paths, surface fields that do not resolve to a row in `compiler_surface_matrix.json`, or guarantees that match no matrix row. Without these, the registry can drift into prose. Recommend a one-line addition to Wave 0 tasks: "Matrix check validates shipped-guarantee schema: status ∈ {stable, experimental, internal, unsupported}, surfaces resolve to matrix-row ids, and every stable guarantee has at least one matrix row."

**3. Wave 6.1 release-binary reuse needs an environment guard.** "Build the Sifr CLI once with `cargo build --release -p sifr`, then run generated programs with `target/release/sifr run`" is correct, but does not say the suite must fail if `target/release/sifr` is stale relative to the working tree. Otherwise local re-runs against an old binary produce false-green or false-divergence. Recommend: "Suite must rebuild the release binary at run start and refuse to run if the binary is older than any tracked Sifr source file."

**4. Wave 9.4 stdlib doctest scope is conditional and vague.** "Add stdlib doctest/example validation for supported APIs where examples are shipped." If no stdlib module ships examples today, this task closes trivially. Either inventory which modules currently ship examples (so the wave has measurable scope) or remove the bullet to avoid a no-op acceptance row.

## P2 — Polish

**5. The decision-log entry for "external reviewer feedback incorporated" is not yet written.** The §Decisions Log has three rows but no row for the pass-3 feedback batch (shipped guarantee registry, hermetic policy, red-blocker, etc.). A row stamped 2026-06-14 noting "External reviewer additions incorporated; phase scope expanded to enforce shipped guarantee registry, hermeticity, crash/ICE contract, perf-trend memory/size, platform manifest, LSP transcripts" would make the diff legible to future readers.

**6. Acceptance criteria has 39 bullets and is starting to read like a manifest.** Not a defect — each is precise and maps to a wave — but at closeout this becomes a tedious checklist. Consider grouping by surface (Gate / Coverage / Evidence / Policy / Closeout) at Wave 10 time without changing the content.

**7. "Verified during this planning pass" duplicates two sub-sections.** The §Existing Facts To Verify section has two "Verified" sub-blocks (lines 138–148 and 150–154) that could be merged into one. Minor readability nit.

**8. Wave 7 Miri/Loom phrasing is soft.** "Add a Miri lane for unsafe/runtime Rust code where feasible" and "Add Loom, Shuttle, or equivalent deterministic concurrency tests where async/thread behavior is shipped and the model can run locally." The conditional is intentional, but the acceptance text doesn't say what evidence proves the conditional has been honestly evaluated. Add: "If Miri or Loom-style coverage is skipped, record the determination (with reason and reproduction) in the platform/sanitizer manifest."

---

# Required Edits

None block implementation. The four P1 items are concrete one-line edits the implementer can apply during Wave 0/Wave 4/Wave 6/Wave 9 PRs without restructuring waves. If the user wants them landed in this iteration before kickoff, the minimal patch is:

- **§Wave 0 tasks**, after the shipped_guarantees.json bullet, add: `Matrix check validates shipped-guarantee schema: status ∈ {stable, experimental, internal, unsupported}, surfaces resolve to matrix-row ids, every stable guarantee has at least one matrix row.`
- **§Wave 4 tasks**, replace `Add a diagnostic code catalog with code, severity, ...` with: `Add a diagnostic code catalog at verification/areas/diagnostics/data/code_catalog.json with code, severity, ...`
- **§Wave 6.1 tasks**, after the release-binary build bullet, add: `Suite must rebuild the release binary at run start and refuse to run if target/release/sifr is older than any tracked Sifr source file.`
- **§Wave 9.4** (stdlib doctest), either inventory the current example-shipping modules or remove the bullet.
- **§Decisions Log**, append a 2026-06-14 row for the external reviewer additions.

---

# Verdict

**Ready after minor edits.**

The plan is now substantively elegant: every shipped guarantee has an owner, a blocking surface, a hardening surface, a regression path, and a reproducible artifact; every red suite has a `red-blocker` row with `closes_in_wave`; every gate-expanding wave carries warm/cold measurement; merge stays hermetic and local-authoritative; CI parity is enforced by emitted profile plans rather than CI behavior. The external feedback expanded scope correctly without introducing fallback paths or quarantine vectors. The Wave 2/5/6/9 sub-PR decomposition keeps it implementable.

The five P1/P2 edits above are quality-of-life text fixes — they tighten ambiguous file paths and a couple of conditional bullets. None requires a third review pass after they land. If Sifr applies the five-bullet patch, the plan is fully implementation-ready and Wave 0 can begin immediately.
