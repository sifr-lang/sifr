# Review Pass 3: Ad Hoc Sifr Self Update

## Verdict: READY

The cleanup is exactly the right move. By treating pre-schema and partial receipts as unmanaged installs that must be re-installed through the standalone installer, the contract collapses the entire backward-compat surface that pass 1 worried about (silent alpha→beta fallback, missing-channel inference, missing-`modify_path` heuristics). What remains is a fail-closed, single-schema, single-source-of-truth design that is genuinely smaller than pass 1's proposal, not just bolted shut.

All seven pass-1 blockers remain resolved, and the pass-2 polish points P2 (concrete `#!` + minimum-size predicate at `:249`), P3 (default `<install_dir>/install.json` with the `~/.sifr/bin` Phase 33 carve-out at `:252`), and P7 (rule 3 explicitly downgraded to a diagnostic affordance at `:174-175`) are now in the contract. The Phase 39 plan now explicitly owns stable self-update activation (milestone_39_4) and re-states the preview safety model in its exit gate.

No new blocking gaps. Items below are non-blocking polish; none of them justify holding the contract open.

---

## Blocking findings

None.

### Backward-compat removal: clean

- `ad-hoc-sifr-self-update.md:136` — "Existing unstable preview installs are allowed to break; users can re-run the new installer to enter the self-update-managed install contract." Unambiguous and matches the stated product direction.
- Locked decisions #6 (`:75`) and #12 (`:81`) treat missing `channel` and missing `modify_path` as *invalid receipts*, not as parse-with-defaults. No silent migration, no derived-from-version fallback, no asymmetric `modify_path` rule. Pass 1's B2 is closed by construction now, not by detection.
- `:159` "Malformed, partial, or pre-schema receipts are treated as unmanaged installs and fail before network access" — consistent with milestone 1 DoD at `:382`. Pre-schema is one named state, not a continuum of partial-schema fallbacks.
- Unknown-field rejection at `:158` keeps the schema closed. Combined with the explicit "exactly `1` until a reviewed schema bump" rule at `:156`, this commits Sifr to coordinated CLI+installer schema bumps rather than drift-tolerant parsing.

### Stable gating: properly two-sided

- Pre-Phase-39 client: `:76` (decision #7), `:102` (stable arg rejection), `:220-226` (whole-document rejection for any `stable` channel, stable-looking version, or non-allowlisted channel in metadata).
- Pre-Phase-39 server: `:325` (metadata absent for stable until Phase 39), milestone_4 scope `:428` (release tooling refuses to generate stable metadata).
- Phase 39 activation: milestone_39_4 (`39_stable_channel_ga_promotion_and_release_governance.md:49-61`) explicitly owns the lifting, with its own receipt/installer/drift validation and an exit gate that re-asserts the preview safety model.
- Forward-dated metadata regression (pass 1 B5) cannot reach a pre-39 CLI because the whole-document check at `:220-226` is unconditional, not behind an `if channel == stable` branch.

### Receipt schema ownership: single source

- `:163-166` locks the schema file at `verification/distribution/self_update_install_receipt.schema.json` and binds *both* the generator output and the Rust parser to it.
- Milestone 1 DoD requires generator-output snapshot, Rust round-trip, canonicalized `binary_path` from the generator side, and a pre-schema-failure test (`:377-382`). Distribution validation enforces conformance (`:323`).
- This closes pass 1 B3 and pass 2 P8 in one step.

### Trust boundaries: bounded by construction

- Installer URL derived from compile-time constant + resolved version (`:209-216`, decision #14 at `:83`). Metadata cannot redirect.
- Metadata is version-strings-only (`:200-208`); no URLs, no targets, no checksums. The CDN cannot substitute the artifact path.
- Runner pre-execution checks at `:249` are now concrete (`#!` shebang, minimum-size floor, non-empty download) — implementers cannot disagree on what counts as "non-script".
- TLS bypass forbidden at `:246`; integration-only override path for `INSTALL_BASE_URL` documented at `:216`.

### Implementation milestones are PR-sized

Each milestone is scoped to one reviewable PR:

- M1: schema file + generator extension + metadata generator + drift checks + `distribution_pipeline.md` update. The biggest milestone, but the work is mechanically delimited by the schema and the existing bash installer.
- M2: four Rust modules (`self_update_cli`, `self_update_receipt`, `self_update_metadata`, `self_update_runner`), dry-run, eligibility, gating rules. No installer execution yet.
- M3: installer delegation, locking, env-passing, diagnostic mapping.
- M4: drift guardrails wired into `scripts/run_distribution_validation.sh`.
- M5: docs + demo.

The dependency between M1 and M2 is explicit (`:347-360` mermaid flow), and `cli_model_and_entrypoint.rs` containment is restated at `:233` so the 900-line guardrail is protected.

---

## Non-blocking polish

- **P1. Channel derivation at installer-generation time is implicit.** Milestone 1 scope (`:368`) says "Extend generated installer receipts with `schema_version`, `channel`, ...", but doesn't pin where `channel` comes from. The only reasonable source is the version's prerelease label (alpha/beta/rc), which the existing `generate_version_installer.sh:189-224` already parses for ordering. One sentence in milestone 1 — "the installer derives `channel` from the version's semver prerelease label" — removes a degree of freedom from the implementer.
- **P2. `modify_path` re-write semantics during self-update are not stated.** Runner passes `SIFR_NO_MODIFY_PATH=1` when the receipt says `modify_path == false` (`:253`), but the contract does not say the installer must record `modify_path` in the *new* receipt reflecting the runtime choice. If the installer always writes `modify_path: true`, the user's no-modify-path preference is lost after the first self-update — and the next update would re-edit shell profiles. Add to milestone 1 DoD: "The installer records `modify_path` equal to the requested setting (`SIFR_NO_MODIFY_PATH` honored), not a hardcoded `true`."
- **P3. Diagnostics list does not enumerate the pre-schema/partial-receipt case.** `:266-280` lists "standalone receipt missing" but not "standalone receipt is pre-schema / partial". Milestone 1 DoD at `:382` already requires this behavior; the diagnostics section should enumerate it with its own ID so the test asserts against a stable code and the user sees a remediation more specific than "no receipt found" (e.g. "this install predates self-update support; re-run the installer to enable `sifr self update`").
- **P4. `--short` interaction with `--format json` is undefined.** `:90` accepts both flags but the schema-frozen JSON shape at `:117-129` shows only the long form. Either declare `--short` no-op under `--format json`, or pin a shorter schema-versioned shape. Snapshot tests must commit either way.
- **P5. Receipt JSON `schema_version` and `self version --format json` `schema_version` are independent.** Both fields are named the same (`:119` vs `:142`) but describe different contracts. Worth one line at `:115` or `:140` distinguishing them so a future schema bump doesn't touch the wrong one.
- **P6. RC channel remediation still absent from `:282-287`.** Pass 2 P6 flagged this and it is unchanged. Users with a `0.1.0-rc.N` receipt running self-update will hit "invalid channel" with no specific guidance. Either accept `rc` in the preview allowlist (matches `generate_version_installer.sh:214-216`), or add an explicit "rc preview unsupported until Phase 39" diagnostic. Today this is a paper concern (no `-rc` release exists yet), but the bash installer already accepts the version shape.
- **P7. Locking covers self-update vs self-update, but not self-update vs manual `curl | sh`.** `:250` puts the exclusive lock under the install directory; `:316` proves serialization between two `sifr self update` invocations. A user who launches `curl ... | sh` in one terminal while `sifr self update` runs in another can still interleave on `install.json` (the bash installer writes the manifest non-atomically at `generate_version_installer.sh:302-313`). For full B7 closure, milestone 1 could move the lock + atomic-rename of `install.json` into the installer itself. Acceptable as polish because the integration test that pass 2 cited matches the explicit acceptance criterion; just note the residual race.
- **P8. `SIFR_INSTALL_MANIFEST_DIR` pass-through trigger uses path comparison.** `:252` says "pass when the receipt was discovered outside the default manifest path for the install directory". With symlinks in play, whether the comparison is path-string equality or canonicalized equality matters. Pin to canonicalized equality (consistent with the same-file eligibility check at `:177-184`).
- **P9. Dry-run-from-an-older-release integration test (`:307`) has no fixture story.** At milestone 2 there is only one schema-versioned release. The test needs either synthetic fixture versions or to be deferred to milestone 4. Pick now so milestone 2 doesn't ship a placeholder.
- **P10. Execution checklist still does not reference pass 2 or pass 3.** Pass 2 P9 asked for a one-line entry; it landed (`ad-hoc-sifr-self-update-execution.md:18`). Add a parallel one-line entry for this review (`reviews/ad-hoc-sifr-self-update-review-pass-3.md`, verdict `READY`) so the audit trail stays closed when milestone 1 opens.
- **P11. Roadmap row 37.1 remains `draft`.** Correct for now. Flip to `in_progress` when milestone 1's PR opens.

---

## Architecture elegance: yes, and tighter than pass 2

The four properties pass 2 measured against still hold, with one strengthened:

1. **Receipt is the trust anchor and is verified before network.** `:104`, `:159`, `:177-184`. Strengthened by the pre-schema=invalid rule at `:159`: there is no longer a degraded-trust state where a partial receipt grants some operations and not others.
2. **Metadata is version-only.** `:200-216` unchanged.
3. **Installer URL bounded at compile time.** `:209-214`, decision #14 unchanged.
4. **Installation is delegated.** `:240-258` and quality bar at `:454-458` unchanged.

The new property the cleanup adds:

5. **Receipt validity is binary.** Either every required field parses against the locked schema, or the install is unmanaged. There is no third state where the CLI infers, defaults, or "best-effort" parses. This is the property that lets milestone 1 ship without a backward-compat test matrix, and that lets milestone 2's eligibility code branch on `Result<Receipt, ReceiptError>` instead of `Receipt { channel: Option<…>, modify_path: Option<…>, … }`. The simpler types fall out of the simpler contract.

The phase still does not have a path where a CDN-poisoning attacker can substitute an installer, does not have a path where a parser bug can replace checksum-verified installation logic, and now does not have a path where a partial/stale legacy receipt can silently change the user's update train. Pass 1's structural risks remain closed by construction; the cleanup added a third class — receipt-inference drift — to the closed-by-construction list rather than the closed-by-testing list.

---

## Assessment

Clean: yes. Elegant: yes — the cleanup removed code and surface area, not just deferred them. Implementation-ready: yes, with the eleven polish items above applied opportunistically inside the milestone PRs they naturally touch (P1, P2, P3, P8 inside M1; P4, P5 inside M2; P6 inside M2 or M5; P7 inside M3; P9 picked before M2; P10 alongside this review; P11 when M1 opens).

## Recommendation

Merge the contract. Open milestone 1.
