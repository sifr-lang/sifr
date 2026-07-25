**Configuration used:** `claude-opus-5` (model confirmed from the running environment). Reasoning effort was requested as *medium*; effort level is set by the harness and not introspectable to me, so I can confirm the model but not independently verify the effort tier. Review was read-only — no files modified.

---

# Phase 40 Gap Review — Stable Channel GA Promotion and Release Governance

Evidence base: `plans/phases/40_...md` (94 lines), Phase 33 (`33_...md`, 363 lines), `plans/roadmap.md`, `plans/issues/archive/ad-hoc-sifr-self-update.md` (526 lines), `.github/workflows/preview-release.yml` (309 lines), `scripts/distribution/*` (13 files), `crates/sifr/src/self_update_{metadata,receipt,runner,cli}.rs`, `verification/areas/distribution_release/` (41 cases), `verification/profiles/*.json`, `docs/installation.mdx`, `docs/self_update.md`, `internal_docs/distribution_pipeline.md`.

**Overall verdict:** Phase 40 is a coherent *governance intent* document but is not implementable as written. It contains one factual error about the shipped receipt schema, one hard compatibility break it does not acknowledge, one self-contradictory constraint that makes its own rollback requirement unbuildable, and several DoD lines that no reviewer can objectively fail. It also omits the plan-structure scaffolding (`status:`, milestone sequencing, per-milestone Positive/Negative/Demo blocks) that Phase 33 — the directly analogous phase — uses throughout.

---

## Blockers

### B1. Publishing a `stable` key into `channels.json` bricks self-update for every already-installed preview user

This is the largest omission and it is not mentioned anywhere in the plan.

There is exactly **one** global metadata document — `https://github.com/sifr-lang/sifr/releases/download/channels/channels.json` (`crates/sifr/src/self_update_metadata.rs:10-11`; published to the shared `channels` release tag at `.github/workflows/preview-release.yml:287-309`). Its parser is strict and **fails the whole document**, not just the unknown channel:

- `self_update_metadata.rs:170-177` — the top-level object must have **exactly 2 keys**.
- `self_update_metadata.rs:190-194` — a `stable` key returns `Err` for the entire document, before any other channel is parsed.

`0.1.0-alpha.1` and `0.1.0-beta.1` are already published to the public (`plans/issues/archive/phase-33-preview-distribution-execution.md:107-108`). The moment Phase 40 milestone_40_4 adds `"stable"` to that shared file, every installed preview binary's `sifr self update` — including `--channel beta`, which never touches stable — hard-errors with *"stable channel metadata is disabled…"*. Users cannot self-update **off** the broken state, because self-update is the broken thing; the only recovery is the curl one-liner.

Phase 40 L55 makes this worse by forbidding the obvious escape hatch: it pins `channels.json` at `schema_version: 1`, so a version-bump-based cutover is explicitly out of scope.

**Concrete plan language to add** (milestone_40_4 Scope + DoD):

> - Stable activation must not break self-update for binaries built before stable activation. Choose and document one of: (a) publish stable resolution in a separate metadata document at a distinct trusted constant URL, leaving `channels.json` alpha/beta-only and byte-identical for pre-stable binaries; or (b) bump `channels.json` to `schema_version: 2` at a distinct URL and keep the `schema_version: 1` document published unchanged for the deprecation window defined in milestone_40_1.
> - DoD: a validation case installs the last pre-stable preview release, publishes the post-stable metadata set, and proves `sifr self update` and `sifr self update --channel beta` still succeed on that binary. This case is mandatory before any stable metadata is published publicly.

This finding also invalidates L55 as currently written (see B3) and must be resolved before milestone_40_4 can be scoped.

### B2. L55 states a factually wrong schema version for the install receipt

Phase 40 L55: *"Keep the ad hoc self-update receipt schema, `self version` JSON schema, and `channels.json` schema at `schema_version: 1`."*

The shipped receipt is **`schema_version: 2`**, and anything else is hard-rejected:
- `crates/sifr/src/self_update_receipt.rs:313-315` — `if schema_version != 2 { … "unsupported" }`
- `scripts/distribution/generate_version_installer.sh:364` — installer writes `"schema_version": 2`
- `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:23` — `"schema_version": { "const": 2 }`

The `1` figure is inherited verbatim from `plans/issues/archive/ad-hoc-sifr-self-update.md:186,200`, which predates the sysroot-era receipt (the shipped receipt also carries `sysroot_schema_version`, `sysroot_sifr_version`, `sysroot_target_triple`, `sysroot_content_sha256` — fields absent from that archived doc's example at lines 185-195). As written, L55 instructs the implementer to hold a constant at a value the codebase already rejects. Fix: `receipt schema_version: 2`, `self version` JSON `schema_version: 1` (`self_update_cli.rs:293`), `channels.json` `schema_version: 1` — and reconcile with B1.

### B3. "field shapes do not change" (L55) is false, and contradicts the plan's own rollback requirement

L55 asserts stable activation *"changes the governed allowlist and accepted version classes, not field shapes."* Two counterexamples in the shipped artifacts:

1. **Version patterns are baked into the authoritative schema, not just an allowlist.** `verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json:40` pins `sysroot_sifr_version` to `^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta|rc)\.[0-9]+$`. A stable version has no prerelease label and cannot match. Similarly `self_update_metadata.rs:13-17` declares `enum PreviewChannel { Alpha, Beta }` — stable is not representable in the type, so this is a type change, not an allowlist edit.
2. **Rollback needs a field that L55 forbids.** L36 defines rollback as "stable metadata points to the approved rollback version, `sifr self update` refuses to install that older stable version without `--force`." That means the remediation for a bad stable release requires **every affected user to individually discover the incident and manually pass `--force`** — the default `sifr self update` is a no-op-or-refuse for exactly the population that needs remediating. Auto-remediation would require a withdrawal/yank signal in metadata (e.g. a `yanked` set), which is precisely the field-shape change L55 prohibits.

**Concrete plan language:**

> - milestone_40_2 Scope: define whether a withdrawn stable version is expressed as (a) metadata pointer revert only, requiring `--force` for affected users, or (b) an explicit withdrawal signal in stable metadata that makes `sifr self update` treat the installed version as ineligible and move to the approved target **without** `--force`. If (b), record the metadata schema change here and update milestone_40_4's schema-stability clause accordingly.
> - DoD: a validation case starts from a receipt pinned to the withdrawn version and proves the chosen behavior — including the exact exit code and diagnostic code emitted — without `--force`.

### B4. L64 requires rejecting "unsigned" stable versions, but no signing or provenance surface exists or is scoped

Phase 40 L64: *"Pre-GA stable metadata and **unsigned** or unapproved stable versions remain rejected."* L44 requires "artifact provenance checks." L93 requires "no stable update without sign-off."

A repo-wide sweep for `cosign|sigstore|slsa|provenance|attest|gpg|minisign|notarize|codesign` across `.github/`, `scripts/`, `crates/sifr/src/`, and `internal_docs/distribution_pipeline.md` returns no signing infrastructure. Integrity today is SHA-256-only (`.github/workflows/preview-release.yml:145-170`; checksums embedded in the immutable installer per `plans/phases/33_...md:85`). `preview-release.yml:24-25` grants only `contents: write` — no `id-token: write` or `attestations: write`. Phase 33 explicitly deferred signing authority (`33_...md:52`: "Long-term signing authority rotation policy" out of scope) and declined a custom signature layer (`33_...md:85`).

So "unsigned … remain rejected" is unverifiable: no artifact is signed, so either the clause is vacuous or it silently imports an unscoped workstream (key custody, rotation, verification in the installer, macOS notarization). No milestone owns it, and it does not appear in any DoD or exit-gate line.

**Concrete plan language:** either add a milestone, or narrow the words.

> - milestone_40_3 Scope: define the stable artifact integrity model explicitly. If GA requires more than the Phase 33 SHA-256 model, add: signing mechanism and verification point (installer vs CLI), key custody and rotation owner, offline verifiability, and macOS notarization/Gatekeeper posture for the two `*-apple-darwin` targets. If GA ships on the SHA-256 model, delete "unsigned" from L64 and state "checksum-verified and release-approved" instead.
> - DoD must name the check that fails a stable release lacking the chosen provenance evidence, and the verification case under `verification/areas/distribution_release/cases/` that exercises it.

### B5. Sign-off and promotion gates are self-referential and cannot be objectively failed

Four DoD/exit lines are unfalsifiable as written:

- L30 *"Promotion checklist is documented and mandatory."* — any checklist satisfies "documented"; "mandatory" names no enforcing mechanism.
- L31 *"Stable self-update cannot be enabled without passing the stable promotion checklist."* — circular: milestone_40_1 defines the checklist and the same milestone's DoD is that the checklist gates the thing. No content is specified, so the milestone is trivially satisfiable by an empty checklist.
- L48 *"Stable releases require auditable approvals and pass governance gates."* — "governance gates" is defined nowhere in the file.
- L39 *"Rollback path is tested and documented."* — no named test, area, or profile.

Compare Phase 33, which grounds every equivalent claim in an executable artifact: `33_...md:231-234` names the exact dispatcher resolutions, `:270-273` names SHA-256-before-install, `:308-312` names dry-run/real-run behavior, and `:236-249, 275-287, 316-327` split every milestone into explicit **Positive validation** / **Negative validation** case lists plus a **Demo** line. Phase 40 has none of these — only prose "validation planning goals" at L80-88.

The repo has the enforcement machinery Phase 40 should be naming: `verification/areas/distribution_release/` with 41 case scripts, a `manifest.json` with `representative`/`full` suites, and `tools/validate_self_update_metadata.sh` (which itself hard-rejects a `stable` key at `:75-76` — a gate Phase 40 must explicitly relicense). Stable gating today is enforced at 14 distinct code sites (dispatcher `generate_dispatchers.sh:85-86, 97-99`; CLI `self_update_metadata.rs:45-49, 190-194, 240-245`; receipt `self_update_receipt.rs:140-145`; plus workflow, artifact-builder, and metadata-generator sites). Phase 40 L27 says "Define the exact criteria for lifting stable gating in installer dispatchers, release metadata, and `sifr self update`" but never enumerates the sites, so "lifted everywhere" is not checkable.

**Concrete plan language:**

> - milestone_40_1 Scope: enumerate every stable-gating site that must be lifted, by file, with its current gate. Any site not listed must remain gated, and a validation case must prove it.
> - milestone_40_1 DoD: the promotion checklist is expressed as an executable check under `verification/areas/distribution_release/` that fails when any required stable-promotion precondition is unmet; a negative case proves stable activation is refused with that check failing.
> - milestone_40_3 DoD: replace "pass governance gates" with the named gate set — the `release` profile, the `distribution_release` `full` suite, the Phase 34 emitted-code gates, and the Phase 30/35 evidence required by D1 — each cited by profile name.

### B6. Phase 40 does not say whether the `merge`/`release` profiles gate stable activation, and `create-pr` does not run distribution checks

L62: *"Stable metadata, dispatcher, immutable installer, GitHub release, and docs drift checks are part of local validation."* "Local validation" is ambiguous across five profiles (`scripts/run_all_tests.sh:14-20`: `create-pr`, `merge`, `nightly`, `release`, `python-interop-live`).

This matters concretely: `verification/profiles/create-pr.json` does **not** select `distribution_release` at all (it sets `"distribution": "none"` at `:237`); `merge.json:222-241` runs the `representative` suite; `release.json:200-220` runs `full`. AGENTS.md tells contributors `--profile create-pr` is the "fast signal — use for PRs," and CI runs only `create-pr` on pull requests (`.github/workflows/local-first-validation.yml:35`). So a PR that changes stable dispatcher or metadata code gets zero distribution coverage unless Phase 40 says otherwise.

**Concrete plan language:** milestone_40_4 DoD should read *"Stable drift checks run in the `distribution_release` `full` suite selected by the `release` profile, and the stable-gating negative cases additionally run in the `representative` suite selected by `merge`. State explicitly whether any stable case is added to `create-pr`."*

### B7. Entry criteria are currently unsatisfiable, and the plan set has already violated its own ordering rule

Phase 40 L67 requires *"Phase 38 is completed."* Phase 38 is `draft` (`plans/roadmap.md:81`) and its own file carries `38_docs_and_documentation.md:3`: *"Needs more planning before execution (doc tooling, doc structure, scope boundaries, ownership model, and acceptance gates are still draft-level)."* Phase 40 therefore cannot start.

Compounding this: Phase 39 declares `Depends on: Phase 38` (`39_rust_interop.md:45-47`) yet is already `completed, audited` (roadmap L82) — so the sequential rule at `roadmap.md:22` ("Sequential execution only") has already been broken at this exact edge. Phase 40 should not silently inherit an ordering assumption that the repo has demonstrably not honored.

**Concrete plan language:** replace the blanket "Phase 38 is completed" with the specific artifacts Phase 40 consumes — e.g. *"the versioned public docs surface and docs quality gate from milestone_38_3 exist and gate `docs/installation.mdx` and `docs/self_update.md`; if Phase 38 remains draft at Phase 40 start, Phase 40 must land those two pages' stable content under an explicit docs gate of its own and record the deviation here."*

---

## Non-blocking improvements

### N1. Missing dependency on Phases 30 and 35 despite the objective naming their evidence
L4 gates GA on *"reliability/parity/performance evidence"*, but `Depends on` (L16-20) lists only 39, 38, 34, and the self-update substrate. Phase 30 (reliability/stdlib parity, completed 2026-03-09, `30_...md:6-9`) and Phase 35 (performance budgets, `status: completed`, `35_...md:3`) are the phases that *produce* that evidence. Note the asymmetry: Phase 34 has a reciprocal `Feeds Into` line (`34_...md:23`) and an explicit Phase 40 gate (L19); 30 and 35 have neither. Both are complete, so this is a contract gap rather than a schedule blocker. Add: *"Phase 30 parity evidence and Phase 35 performance budgets (`check_budgets.py`, `baselines.json`, `waivers.json`) must be green at promotion time, with no open waiver covering a stable-advertised path."*

### N2. `rc` is unaddressed, though the pipeline half-supports it
`-rc.N` is accepted by `preview-release.yml:59`, `build_preview_artifacts.sh:82-85`, and `generate_version_installer.sh:70-73`, and `generate_dispatchers.sh:83` returns `"rc"` — but `normalize_channel` has no `rc` arm, `generate_channel_metadata.sh:51-65` rejects it, and the CLI hard-rejects it at `self_update_metadata.rs:40-44, 195-199, 243-245`. So an rc is buildable but not resolvable or self-updatable. `ad-hoc-sifr-self-update.md:114` deferred rc to "Phase 39" (meaning this phase). Phase 40 mentions rc **zero times**. Decide: does GA require an rc soak gate before stable? If yes it is a milestone; if no, state that rc stays rejected at GA and that the half-support in the build scripts is removed or documented as dead.

### N3. Platform and artifact coverage for GA is undefined
The supported set is four targets, hardcoded in three places (`self_update_receipt.rs:7-12`; `generate_version_installer.sh:407-432`; `build_preview_artifacts.sh:5-10`) and advertised at `docs/installation.mdx:13-20` with Windows explicitly unsupported. Phase 33 deferred package managers (`33_...md:48`: brew/apt/npm/pip/cargo install/Windows). Phase 40 never states whether GA ships the same four targets. A "stable GA" that is macOS+Linux only with no Windows and no package-manager path is a defensible decision, but it must be a *recorded* decision. Add to milestone_40_1: *"GA platform matrix is exactly the Phase 33 target set; Windows and package-manager distribution remain out of scope and `docs/installation.mdx` states this at GA."*

### N4. No versioning/compatibility policy for what "stable" promises
Phase 33's classifier treats `0.X.Y` without a prerelease label as stable-looking (`33_...md:100`), so the first stable could be `0.1.0` or `1.0.0` — the plan never says which, nor what language/CLI compatibility a stable version guarantees going forward. This is directly load-bearing on the Phase 27 contract Phase 40 inherits (L69: stable diagnostic codes, exit codes `0/1/2/3`). Add a milestone_40_1 line fixing the first GA version and the post-GA compatibility promise for CLI flags, exit codes, diagnostic codes, and the three JSON schemas.

### N5. Plan-structure conformance
Relative to Phase 33, Phase 40 is missing: a `status:` field (roadmap L85 says `planned`, `plans/phases/index.md:50` says `unspecified` — drift); milestone sequencing (`33_...md:200-212` has a mermaid chain and `Depends on: milestone_33_N` per milestone); per-milestone `Goal:` / `Positive validation:` / `Negative validation:` / `Demo:` blocks; and a named execution-checklist issue. L78 says *"Validation evidence must be recorded in the phase execution checklist issue"* but no such issue exists — `plans/issues/active/` has five files, none release-related, and there is no `phase-40-*-execution.md` analogous to `plans/issues/archive/phase-33-preview-distribution-execution.md`.

### N6. Support, incident, and deprecation responsibilities are named but not assigned
L34 requires "owner responsibilities, and communication protocol"; the DoD (L38-40) only requires the rollback path be "tested and documented" and never requires the owner or protocol to be recorded. `verification/owners.json` exists and `verification/areas/distribution_release/manifest.json` already declares owner `release/distribution` — so there is a real place to anchor this. Add: *"milestone_40_2 DoD: the incident owner, escalation contact, disclosure channel, and target acknowledgement window are recorded in the phase doc, and the `distribution_release` area owner is updated to match."* Also unaddressed: how long a superseded stable version's immutable installer and release assets remain fetchable (relevant because pinned installs at `sifr.sh/install/versions/<v>` must keep resolving) — no retention policy exists anywhere in the repo.

### N7. Sign-off has no enforcement mechanism in the only publishing workflow
`preview-release.yml` is `workflow_dispatch`-only (`:5`) with **no `environment:` key**, so there are no required reviewers and no approval record; the sole gate is `concurrency` (`:27-29`). "Auditable approvals" (L48) has nothing to bind to. Concrete: *"milestone_40_3 DoD: the stable publish job runs under a protected GitHub environment with required reviewers, and the approval record is part of the sign-off artifact set."* Also note `preview-release.yml:245` hardcodes release-note text asserting stable is disabled — a stable-activation drift site the plan does not list.

### N8. Stale phase numbering in the inherited substrate doc
`plans/issues/archive/ad-hoc-sifr-self-update.md` refers to stable-channel governance as "**Phase 39**" at lines 3, 55, 64, 68, 84, 113, 114, 319, 336, 352, 465 — but Phase 39 is now Rust Interop and stable GA is Phase 40. Since Phase 40 L20 makes that document a normative dependency, an implementer reading it will find its gates pointing at a completed, unrelated phase. Phase 33's file already uses the corrected wording (`33_...md:9, 50, 81, 101`). Worth a one-line note in Phase 40 L20 rather than editing the archive.

---

## Considered and rejected as already covered

- **"No receipt/binary mismatch handling on rollback"** — covered by L37 and the shipped device/inode identity check (`self_update_receipt.rs:146-158`, `ad-hoc-sifr-self-update.md:226`).
- **"Installer URLs could come from metadata"** — explicitly forbidden at L56 and L93, and already enforced by construction (`self_update_metadata.rs:8-11, 88-93`; runner uses only the derived URL at `self_update_runner.rs:75`).
- **"No TLS/download hardening for stable"** — inherited via L56/L93 and already implemented: `--proto =https --proto-redir =https` (`self_update_runner.rs:76-92`), ≥1024-byte and shebang checks (`:180-200`), atomic rename (`:102-108`).
- **"Checksum verification missing for stable artifacts"** — inherited from the immutable installer (`generate_version_installer.sh:668-671`), re-verified in the workflow (`preview-release.yml:145-170`), and named in L46. (Distinct from B4, which is about *signing/provenance*, not checksums.)
- **"Metadata/dispatcher/installer drift"** — genuinely covered by L46, L62, and eight existing drift cases in `verification/areas/distribution_release/cases/`.
- **"No panic-freedom or diagnostic-stability requirement"** — thoroughly covered at L68-70 and L94.
- **"Concurrent/racing release publication"** — handled by `preview-release.yml:27-29` and the install lock (`self_update_runner.rs:48-52`, `generate_version_installer.sh:384-405`); no evidence stable changes this.
- **Generic release-management advice** (release calendars, changelog policy, SemVer tutorials, canary percentages, telemetry) — deliberately omitted; no repository evidence makes them relevant, and the project has no telemetry or staged-rollout surface to hook into.

---

**Recommended gating order before implementation starts:** resolve **B1** (it determines the metadata architecture and therefore the scope of milestone_40_4), then **B2/B3** (schema facts and rollback semantics), then **B4** (signing in or out), then **B5/B6** (make the gates executable). **B7** is a scheduling decision for the roadmap owner, not a rewrite of this file.
