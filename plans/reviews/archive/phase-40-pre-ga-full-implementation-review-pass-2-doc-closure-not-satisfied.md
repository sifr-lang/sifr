## Archived verdict: NOT SATISFIED

One actionable finding. Observations 1, 3, and 4 are closed and independently verified; observation 2's replacement wording substitutes a new inaccuracy for the old one.

### What I verified independently (not taken on report)

**Observation 1 — effective GA prepare deadline: closed, correct.**
`MINIMUM_PUBLICATION_WINDOW = timedelta(days=7)` (`verification/areas/distribution_release/governance/stable_prepare.py:66`) is enforced at `:203` via `_require_publication_window` (`:835-846`) against `qualification["workflow"]["expires_at"]`. The committed candidate index pins `workflow.expires_at = 2026-08-28T02:17:30Z` (`plans/releases/candidates/0.1.0/qualification-artifact-index.json`), so `expiry − 7d = 2026-08-21T02:17:30Z` — exactly the value now recorded in all four documents, and correctly kept distinct from the `2026-08-27T00:00:00Z` waiver expiry (which the diff now scopes to *recovery* completion, not GA). Two non-blocking precision notes, neither an inaccuracy worth a fix: the gate compares strictly (`expiry - now < 7d` fails), so a prepare starting exactly *at* `02:17:30Z` still passes — "before" is conservative, not wrong; and the phase-doc phrasing at `:890-893` says "protected GA publication must begin before", while the enforcement point is prepare (`internal_docs/distribution_pipeline.md:775-781` says "prepare" precisely). Publication necessarily follows prepare, so the stricter phrasing is safe.

**Observation 3 — recovery-before-activation ordering: closed.** Stated explicitly and consistently in three places (`internal_docs/distribution_pipeline.md:775-781`, `plans/issues/active/ad-hoc-sifr-site-production-credentials.md:63-67`, `plans/issues/active/phase-40-stable-channel-ga-execution.md:798-808`, `plans/phases/…:1038-1045`), including the correct causal explanation that advancing the live index to generation 2 *intentionally* invalidates the generation-1 precondition (the recovery reconstructs from the retained `channels-generation-1.json` snapshot, `scripts/distribution/prepare_schema_bootstrap_recovery.sh:138-142`) and that it cannot be repaired by a second attempt. No safety property weakened — this is documentation of a fail-closed design, not a relaxation.

**Observation 4 — drill evidence: closed, and I verified it live.** All four runs exist at exact source `1a90170dbe878b60cf644c63d28d3076f31e6320`:

| run | mode (from evidence `scenarios[].name`) | conclusion | created |
|---|---|---|---|
| 30496849280 | `publication` | success | 22:38:35Z |
| 30496850896 | (queued rollback) | **cancelled** | 22:38:36Z |
| 30496852409 | `first-ga` | success | 22:38:38Z |
| 30496911507 | `rollback` | success | 22:39:40Z |

I downloaded the three retained `protected-drill.json` artifacts and recomputed SHA-256: `3450ca33…9c9f`, `2e3d6f52…d03ae`, `be8b24b4…91822` — all three match the ledger literals byte-for-byte, and each carries `status: pass`, `environment: stable-release-drill`, `external_network: blocked`, `production_credentials: absent`, exactly as recorded. The cancellation narrative is corroborated by the timestamps: the rollback was queued at 22:38:36Z, displaced from the single pending concurrency slot by the 22:38:38Z first-GA dispatch, and redispatched successfully at 22:39:40Z.

**Archive/link truthfulness: confirmed.** All three archived files exist with the cited names and verdicts. `phase-40-candidate-evidence-closeout-pr-3072-review-pass-1-satisfied.md` ends `SATISFIED` with three observations (one low, two informational); pass 2 ends `SATISFIED` with "Two non-blocking observations, no actionable defect" and contains an explicit **"Pass-1 observation closed ✔"** section verifying the "Canonical report" wording narrowing at head `a9db40804…` — so the new ledger sentence "the second confirmed the pass-1 wording observation was closed and returned no actionable finding" is accurate. `b5f4d0673e8c77ae9fcebe47f377f9d45ae3c842` is indeed the `#3072` merge commit. The pass-1 pre-GA audit summary at `plans/issues/active/phase-40-stable-channel-ga-execution.md:1719-1727` matches that archive's contents and verdict.

---

### Actionable finding (observation 2 remains inaccurate)

**F1 — The revised proof-composition wording attributes fresh install and self-update execution to the four-target qualification, which does not perform either.**
`plans/phases/40_stable_channel_ga_promotion_and_release_governance.md:1102-1106` and `:1231-1234`.

What the four-host matrix actually executes (`.github/workflows/release-qualification.yml:86-141` → `scripts/distribution/qualify_stable_target.py:88-195`), per target on its native runner:
1. `build_release_artifacts.sh` build + package;
2. archive checksum comparison and `verify_archive`;
3. `tarfile.extractall` of the candidate archive into a clean temp root — **not** an installer/dispatcher-driven install;
4. `sifr --version` and a one-line `sifr check` compile smoke;
5. a receipt **synthesized by the script itself** (`receipt_dir/install.json`, written by `qualify_stable_target.py:145-155`, not produced by the installer) fed to `sifr self version --format json` — i.e. `self version`, **not** `self update`.

`grep -rn "self update" .github/workflows/ scripts/distribution/` returns no hit in `release-qualification.yml`. The only `self update` executions anywhere in the release path are `run_stable_public_smoke.sh:167` (ubuntu-24.04 runner, post-publication, `--dry-run`) and `run_schema_bootstrap_public_smoke.sh:97`; installed-sysroot self-update qualification is the single-host local release-profile certification (`verification/areas/sysroot_release/self_update_certification.py`, described at `plans/phases/…:971-975`). Likewise, the only installer-driven *fresh install* is `sh "${out}/stable-dispatcher"` at `run_stable_public_smoke.sh:161-166`, on that same one runner.

So the diff replaced a false claim ("post-publication fresh-install and self-update smoke pass on every supported target") with a differently false one ("pre-publication qualification executes fresh install and self-update on all four supported targets"). The dependent exit-gate phrase "byte-identical to the **fresh-install and self-update-qualified** artifacts" inherits the same defect — the byte-identity half is true and verified (staged assets derive from the digest-verified qualification artifacts via `materialize_stable_publication.py stage`, published by `publish_stable_release.py` into `published-assets.json`, then re-downloaded and digest-compared for every key at `run_stable_public_smoke.sh:145-158`), but the qualification is mischaracterized.

Suggested exact replacements (both preserve the design and weaken no safety property):

`:1102-1106` →
```
- Pre-publication qualification builds, packages, and verifies each of the four
  supported targets on its native runner: exact-archive checksum and archive
  verification, extraction into a clean root, `sifr --version`, a compile
  smoke, and `sifr self version` receipt validation. Installed-sysroot
  self-update is certified separately against an isolated schema-v2 fixture in
  the authoritative local release profile. Post-publication verification
  downloads and digest-checks every published target asset against those
  qualified bytes, while live installer fresh-install and
  `sifr self update --dry-run` execute on the protected workflow runner's
  matching target.
```

`:1231-1234` →
```
- public stable assets for all four supported targets are byte-identical to the
  per-target qualified artifacts, and live public installer fresh-install and
  self-update smoke pass on the protected workflow runner's matching target;
```

No other issue found; `:1098` ("published assets are byte-identical to the qualified assets") is accurate and unaffected. I modified no files, and I treated `editor_integrations`, `third_party/ruff`, and the leetcode corpus as unrelated user state.
