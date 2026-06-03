# Review Pass 2: Ad Hoc Sifr Self Update

## Verdict: READY

All seven blocking findings from pass 1 are addressed at the contract level. The architecture now matches the "least new Rust code" intent: CLI verifies the install receipt, fetches version-only metadata, derives the immutable installer URL from compile-time constants, and delegates installation to the existing installer that already owns checksum verification, extraction, and PATH editing.

The remaining items are non-blocking polish. They can be applied in milestone PRs without re-opening the contract.

---

## Blocking findings

None.

### Pass-1 blocker disposition

- **B1 metadata URL injection** — Resolved. `ad-hoc-sifr-self-update.md:83` (locked decision #14), `:188-216` (no URL fields in metadata, `INSTALL_BASE_URL` is a compile-time constant, integration tests are the only override path).
- **B2 silent alpha→beta fallback** — Resolved. `ad-hoc-sifr-self-update.md:75` (decision #6 derives channel from prerelease label and fails closed otherwise), `:158` (receipt backward-compat rule), `:397` (milestone_2 DoD explicitly forbids defaulting to beta).
- **B3 receipt schema ownership** — Resolved. `ad-hoc-sifr-self-update.md:164-166` (schema lives at `verification/distribution/self_update_install_receipt.schema.json`, generator output + Rust round-trip both validated), `:323` (distribution validation enforces conformance), `:378-380` (milestone_1 DoD).
- **B4 same-file eligibility** — Resolved. `ad-hoc-sifr-self-update.md:177-184` (install-time canonicalization of `binary_path`, run-time canonicalization of current_exe, Unix dev+inode equality, documented fallback for platforms without stable inode metadata).
- **B5 forward-dated stable metadata** — Resolved. `ad-hoc-sifr-self-update.md:220-226` (whole-document rejection when metadata contains `stable`, stable-looking versions, unknown channels, or non-exact version strings), `:314` (integration test).
- **B6 APP_VERSION drift** — Resolved. `ad-hoc-sifr-self-update.md:322` (validation cross-checks embedded `APP_VERSION`, metadata, dispatcher target, GitHub release tag), `:431` (milestone_4 DoD requires extracting `APP_VERSION` from each immutable installer).
- **B7 concurrent update locking** — Resolved. `ad-hoc-sifr-self-update.md:250` (exclusive install-dir lock), `:316` (integration test for serialization), `:416` (milestone_3 DoD).

---

## Non-blocking polish

- **P1. `default_channel` in `channels.json` (`ad-hoc-sifr-self-update.md:202`) has no defined consumer.** Self-update always derives the channel from the receipt; the field is dead weight in this phase's contract. Either drop it or state that the bash dispatcher (not self-update) reads it, so a future reader knows why it exists.
- **P2. "Obvious non-script responses" pre-execution check (`ad-hoc-sifr-self-update.md:249`) is vague.** A phase contract should commit to a concrete predicate (e.g. "first line must start with `#!`" plus a minimum size floor). Otherwise different implementers will disagree on what counts.
- **P3. `SIFR_INSTALL_MANIFEST_DIR` "default location" is implied but not stated (`ad-hoc-sifr-self-update.md:252`).** Pin the default explicitly to `<install_dir>/install.json` so the runner's "pass it only when discovery happened elsewhere" rule has a precise trigger.
- **P4. Unknown-field rejection at `schema_version` ≥ 1 (`ad-hoc-sifr-self-update.md:160`) creates a hard forward-compat wall.** Acceptable given the explicit "fail closed" stance, but worth calling out in `internal_docs/distribution_pipeline.md` so a future schema bump knows it must ship coordinated CLI support before the receipt change rolls out.
- **P5. TLS trust-store source is unpinned.** `:247` forbids insecure certificate bypass, which is the load-bearing rule. Pinning rustls+webpki-roots vs system roots is implementation-level; not required at contract scope, but a single sentence in `distribution_pipeline.md` would prevent drift between dev and release builds.
- **P6. RC (`-rc.N`) channel is silently disallowed.** Locked decision #6 only names alpha/beta. That is consistent with Phase 39 gating, but the rejection diagnostic for an installed `0.1.0-rc.N` receipt should be enumerated in `:262-280` so users hitting it before Phase 39 get a specific remediation.
- **P7. Receipt-discovery rule 3 (`ad-hoc-sifr-self-update.md:172`) is structurally redundant with rule 2 once same-file metadata is enforced.** Keeping it costs little, but the contract could simplify by stating that rule 3 is purely a diagnostic-quality affordance, not an additional trust anchor — readers may otherwise think it widens the eligibility surface.
- **P8. Install-time canonicalization of `binary_path` (`:178`) has no explicit generator test in the validation contract.** Milestone 1's snapshot test covers schema shape; add one assertion that the written `binary_path` equals `canonicalize(install_dir + "/sifr")` to lock the runtime equality contract from the generator side.
- **P9. Execution-checklist (`ad-hoc-sifr-self-update-execution.md`) does not reference pass 2.** Once this review lands, append a one-line entry under "Review Artifacts" pointing to `reviews/ad-hoc-sifr-self-update-review-pass-2.md` with the `READY` verdict so the audit trail is complete before milestone_1 opens.
- **P10. Roadmap row 37.1 status remains `draft`, which is correct for now.** Update to `in_progress` only when milestone_1's PR opens; no change needed in this review.

---

## Architecture elegance: yes

The four-property test passes:

1. **CLI verifies receipt before any network call.** `ad-hoc-sifr-self-update.md:104` (receipt failures reject before network), `:168-184` (discovery + eligibility), `:177-184` (canonicalization + dev/inode same-file check). Receipt is the trust anchor; nothing downstream runs without it.
2. **Metadata is version-only.** `:188-216` reduces `channels.json` to `{schema_version, default_channel, channels: {alpha|beta: "<exact-version>"}}`. No URLs, no checksums, no targets, no installer names — therefore nothing the CDN serves can redirect or substitute the install artifact.
3. **Installer URL is derived from constants.** `:209-214` plus locked decision #14 (`:83`) make `INSTALL_BASE_URL` a compile-time string overridable only by an explicit test-only build path. The set of URLs the runner can ever fetch is bounded at compile time.
4. **Installation is delegated.** `:240-258` describes only: lock, download, atomic rename, exec installer with receipt-derived env. The quality bar (`:454-456`) explicitly rejects any drift toward duplicating extraction, checksum verification, target mapping, or shell-profile edits inside Rust.

The phase no longer has a path where a CDN-poisoning attacker can substitute an installer, and no path where a parser bug in the Rust side could replace checksum-verified installation logic. Both were the structural risks pass 1 flagged and both are now closed by construction, not by detection.

---

## Recommendation

Merge the contract as-is. Apply P1–P8 inline during milestone_1 and milestone_2 PRs where the touched files naturally cover them; P9 is a one-line edit to the execution checklist that should land alongside this review.
