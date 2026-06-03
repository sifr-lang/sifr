# Review Pass 4: Ad Hoc Sifr Self Update

## Verdict: READY

All eleven polish items from pass 3 landed cleanly, and the cleanup that pass 3 endorsed is now reinforced rather than diluted. The strengthened items — channel derivation pinned to the semver prerelease label, `modify_path` persisted from the request rather than hardcoded, pre-schema/partial receipts enumerated as their own diagnostic case, `self version --short --format json` explicitly rejected so the JSON shape stays singular, `self version` output `schema_version` distinguished from the receipt `schema_version`, rc rejection paired with a forward-leaning "or a reviewed channel contract adds rc" carve-out, manual-installer/self-update lock fused with atomic receipt writes, and synthetic fixtures called out for the dry-run-from-an-older-release test — close the few interpretive seams the previous contract left to the implementer. None of them widened the surface.

The contract still has the five elegance properties pass 3 measured:

1. Receipt is the trust anchor and is verified before network (`:104`, `:159`, `:177-186`).
2. Metadata is version-only (`:200-216`).
3. Installer URL bounded at compile time (`:209-218`, decision #14 at `:83`).
4. Installation is delegated to the immutable installer (`:240-258`, quality bar `:472-476`).
5. Receipt validity is binary (`:158-161`).

Property 5 is now further reinforced by the milestone 1 DoD requirement that the pre-schema rejection path is its own tested diagnostic (`:398`) rather than collapsing into "receipt missing".

No blocking gaps. The polish items below are non-blocking and can be folded into the milestones they naturally touch — none require holding the contract.

---

## Blocking findings

None.

### Pass-3 polish closure

| Pass-3 item | Resolved at | Notes |
|---|---|---|
| P1 channel derivation source | `:380`, `:333` | Pinned to semver prerelease label in M1 scope and distribution validation. |
| P2 `modify_path` persistence | `:381`, `:334` | Generated installer must record `modify_path` from request, not hardcoded `true`. |
| P3 pre-schema/partial diagnostic | `:268-269` | Now two enumerated diagnostic cases with their own remediation surface. |
| P4 `--short` × `--format json` | `:115`, `:309`, `:410` | Rejected; pinned in contract, unit tests, and M2 DoD. |
| P5 receipt vs `self version` schema_version | `:117` | Explicitly independent contracts. |
| P6 rc remediation | `:275`, `:416` | rc enumerated in diagnostics; M2 DoD allows either rejection or a reviewed channel contract update. |
| P7 manual installer vs self-update lock | `:324`, `:335`, `:388` | Same install lock, atomic receipt writes on both code paths, M1 DoD enforces. |
| P8 `SIFR_INSTALL_MANIFEST_DIR` trigger | `:254` | Canonicalized path comparison. |
| P9 dry-run-from-older fixture story | `:314` | Synthetic local fixture versions in M2 integration tests. |
| P10 pass-3 in execution checklist | `execution.md:19` | Recorded with `READY` verdict. |
| P11 roadmap row 37.1 | `roadmap.md:73` | Still `draft` — correct until M1 PR opens. |

### Trust boundaries: still closed by construction

- Installer URL: compile-time `INSTALL_BASE_URL` + resolved version string (`:213-214`); test override is described as "an explicit test-only path" and "Production runtime configuration must not accept arbitrary installer URLs from metadata or receipts" (`:218`). Together these are sufficient to reject CDN-poisoning and metadata-redirect attacks. (One residual ambiguity called out in polish below.)
- Metadata is strictly version strings, schema-versioned, and whole-document-rejected on first sight of stable/unknown/non-allowlisted entries (`:222-227`). The check is unconditional — it does not branch on "if channel == stable", so forward-dated stable metadata cannot reach a pre-Phase-39 CLI.
- Pre-execution installer checks are concrete: non-empty download, minimum size, `#!` first-line (`:252`). Atomic-rename-before-exec at `:251`. TLS bypass forbidden at `:248`.
- Receipt schema is closed: unknown fields rejected (`:160`), `schema_version` must be exactly `1` (`:158`), pre-schema/partial treated as unmanaged (`:161`). The single-source schema at `verification/distribution/self_update_install_receipt.schema.json` (`:165`) is bound to both generator output and Rust parser via M1 DoD (`:391-396`) and distribution validation (`:331`).
- Current-executable eligibility requires same-file metadata after canonicalization where the platform supports it, not path-string equality (`:183-186`). Discovery rule 3 explicitly downgraded to a diagnostic-quality affordance, not a trust anchor (`:177-179`).

### Clean unstable-contract design: preserved

- No backward-compat surface for pre-schema receipts. Decisions #6 (`:75`) and #12 (`:81`) keep missing `channel` and missing `modify_path` as invalid-receipt states. No silent migration, no derive-from-version fallback, no asymmetric defaulting. Pass-1 B2 stays closed by construction.
- Pre-schema is one named state — "unmanaged install, re-run the installer" — not a continuum of partial-schema fallbacks (`:138`, `:161`, `:268-269`, `:398`).
- This is the property that lets M2's eligibility code return `Result<Receipt, ReceiptError>` instead of a struct full of `Option<…>` fields. The simpler types fall out of the simpler contract.

### Stable gating: two-sided, with rc joining the same model

- Client-side rejection: `:76` (decision #7), `:102-103` (arg rejection), `:222-227` (metadata whole-document rejection), `:275` (rc enumeration in diagnostics).
- Server-side rejection: `:336` (no stable metadata until Phase 39), milestone_self_update_4 scope at `:446`.
- Phase 39 owns lifting both gates: `39_stable_channel_ga_promotion_and_release_governance.md:49-61` (milestone_39_4) plus the re-asserted preview safety model in the Phase 39 exit gate (`:88-92`).
- rc is now handled symmetrically with stable: rejected before Phase 39 with an explicit diagnostic, but explicitly allowed to be lifted by "a reviewed channel contract adds rc" (`:275`, `:416`). This is the right shape — pre-Phase-39 rejection is the contract, not an accident of the diagnostic table.

### PR-sized milestone sequencing: explicit

The mermaid at `:359-371` plus per-milestone scope/DoD blocks delimit five reviewable PRs. M1 grew slightly with the channel/`modify_path`/atomic-lock additions, but those changes touch the same `generate_version_installer.sh` and the same `verification/distribution/` schema file as the rest of M1's scope — they remain mechanically delimited.

### Testability: fixture story is complete

- Unit test list (`:297-309`) covers schema parsing, schema rejection (empty/invalid JSON/wrong types/unknown fields/unsupported schema_version), discovery order, same-file eligibility, channel/version parsing, stable rejection, rc rejection, update-needed comparison, dry-run output, and `--short --format json` rejection.
- Integration test list (`:311-324`) covers HTTP fixture, dry-run-from-older with synthetic fixtures, no-op, force-required cases, mismatched/missing receipt pre-network rejection, malformed-metadata rejection, env-passing, and the two concurrent-update tests (self-update × self-update, manual × self-update).
- Distribution validation (`:326-337`) covers metadata drift, version agreement, schema conformance, `binary_path` canonicalization, channel/`modify_path` derivation, atomic-rename + shared lock, and stable absence.

The previous pass-2/pass-3 worry that some tests would arrive without a fixture target (older release dry-run) is resolved by `:314` calling out synthetic local fixture versions explicitly.

---

## Non-blocking polish

- **P1. `INSTALL_BASE_URL` test override is described but not bounded by mechanism.** `:218` says "Integration tests may override `INSTALL_BASE_URL` through an explicit test-only path". A strict trust-boundary read wants this pinned to `cfg(test)` / build-time constant override, with runtime environment variables explicitly forbidden as the override channel. The current phrasing technically permits "an env var that production happens not to set", which is the wrong default. Recommend tightening to "compile-time / `cfg(test)` override only; runtime environment variables must not change the installer URL." Polish, because the production runtime line that follows already disallows arbitrary URLs from metadata or receipts — but the override mechanism itself should be named.
- **P2. Receipt required-field enumeration lives only in the schema file.** `:158` names `binary_path`, `channel`, and `modify_path` as required, plus `schema_version`. The other five fields (`name`, `version`, `target`, `install_dir`, `artifact`) are required by virtue of "unknown fields rejected" + schema file, but the contract text never says "all fields shown are required". This is fine because the schema file is authoritative per `:163-167` and M1 DoD requires generator/parser conformance — but a one-liner ("all fields above are required; the schema file at `verification/distribution/self_update_install_receipt.schema.json` is the authoritative enumeration") removes a future reading-comprehension trap.
- **P3. Install lock path is not named.** `:250` says "an exclusive update lock under the install directory" and `:335` says "the same install lock used by self-update", but the concrete path is left to M1. Implementor risk is low because both call sites are written in the same milestone, but pinning (e.g. `<install_dir>/.update.lock`) up front prevents drift between the bash side and Rust side. Polish.
- **P4. Channel allowlist not enumerated in the metadata section.** `:226` rejects "any channel outside the pre-Phase-39 allowlist" without naming the allowlist. Derivable from `:88` (`--channel alpha|beta`) and `:275` (rc gated), but a one-line "the pre-Phase-39 allowlist is exactly `{alpha, beta}`" at `:222` makes the rejection rule readable in isolation.
- **P5. rc-receipt remediation is not in the diagnostics remediation block.** `:275` enumerates the rc diagnostic; `:289-293` lists remediations for missing receipt, package-manager installs, and force-required cases — but not for a user with an rc-channel receipt. The implied remediation is "use `curl -LsSf https://sifr.sh/install | sh` until Phase 39 or a reviewed rc contract lands". One line.
- **P6. Diagnostic code family/range unassigned.** `:264` reserves "a small CLI diagnostic range" without naming it. Per Phase 31.7's `SIFR-<FAMILY>-dddd` taxonomy, the family should be named at M2 time. Not a contract gap because M2 will assign per the taxonomy; just worth noting in M2's scope.
- **P7. Dry-run lock semantics unstated.** Real updates acquire the install lock (`:250`). Dry-run is non-mutating and presumably should not block on the lock. The contract doesn't say. Two dry-runs racing a real update can only produce stale-display output, never a safety issue — so this is purely a UX paper-cut. One sentence in M2 or the `--dry-run` arg description suffices.
- **P8. `--force` pass-through assumes installer accepts `--force`.** `:256` requires the runner to pass `--force` only when requested. The bash installer must accept it. M3 scope covers this implicitly under "execute the installer with receipt-derived environment and requested `--force`" (`:425`), but the implementer should verify the installer's argument parser before relying on it — or add `--force` support if it's missing today. Not a contract gap, just a sequencing note for M3.

---

## Final implementation-readiness assessment

The contract is implementation-ready. The pass-3 cleanup gave the design a sharply smaller surface than pass-1 proposed, and pass 3's eleven polish items have now closed the residual interpretive seams without re-expanding the surface.

Concretely:

- **Trust boundaries:** sealed by construction at every layer (receipt fail-closed; metadata version-only; installer URL compile-time-bounded; immutable installer owns checksum/extraction; same-file eligibility; pre-execution installer-shape checks). The one remaining interpretive seam (P1, override mechanism for `INSTALL_BASE_URL`) is polish, not a blocker.
- **Unstable-contract cleanliness:** no backward-compat code paths anywhere. Receipt validity is binary; pre-schema is one named diagnostic state; unknown-field rejection keeps the schema closed; `schema_version` is exactly `1` until a coordinated bump.
- **Stable gating:** two-sided (client rejection + server refusal to generate), and Phase 39 explicitly owns the lifting through `milestone_39_4` with its own validation surface and a re-asserted preview safety model. rc is now handled the same way.
- **PR-sized milestones:** five PRs with explicit ordering, explicit DoD per milestone, and explicit `cli_model_and_entrypoint.rs` containment to protect the 900-line guardrail.
- **Testability:** every contract claim has a named test (unit, integration, or distribution-validation), and fixture targets exist for every test — including the dry-run-from-older case that previously had no fixture story.

**Recommendation:** merge the contract, open milestone 1, apply P1–P8 opportunistically inside the milestone PRs they naturally touch (P1, P2, P3 inside M1; P4 inside M1 or M2; P5 inside M2 or M5; P6 inside M2; P7 inside M2; P8 inside M3). Flip roadmap row 37.1 from `draft` to `in_progress` when the M1 PR opens (pass-3 P11, still correct).
