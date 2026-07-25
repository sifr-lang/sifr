I've read round 1, round 2, both current target plans, and the current read-only Phase 40 plan. No files were modified. Here is the round-3 delta confirmation.

---

# Round-3 Final Delta Confirmation — Rust Interop Follow-Up Plans

Re-inspected the two target plans and the read-only Phase 40 plan against the four NEW findings round 2 raised while returning SATISFIED. All concurrent/user-owned files (Phase 40, the empty `phase40-gap-review-opus5-medium.md`, `plans/reviews/iterations/`) left untouched.

## NEW-1 through NEW-4 disposition

| Finding | Disposition | Evidence in current plans |
|---|---|---|
| **NEW-1** — Phase 40 ↔ cert/hardening stable-gate ownership & reverse-dependency unreconciled | **CLOSED** | Reconciled from **both** target plans' side. Certification `certification_0` states "Phase 40 `milestone_40_1` is downstream of both `hardening_1` and this item" (`cert:137-139`) and "Treat `stable_support_claims.json` as the compatibility-derived input that Phase 40 digests into its canonical `stable-release-plan.json`; it is not a second release-plan authority. The governed Phase 40 release plan remains authoritative" (`cert:152-154`). Hardening's header adds: "Phase 40 `milestone_40_1` may not claim its required release-profile Rust-interop suite execution until `hardening_1` has merged. Its stable-claim gate additionally depends on … `certification_0`; `hardening_1` owns execution wiring, while `certification_0` owns claim derivation and stable-candidate validation" (`hard:12-16`). Consistent with read-only Phase 40, which already lists `hardening_1`–`hardening_4` + `certification_0` in Upstream Handoffs (`ph40:49-57`) and binds the `stable_support_claims.json` digest into `stable-release-plan.json` (`ph40:176-179`). No competing authority; digested-input framing matches on both sides. |
| **NEW-2** — `check_tiers.py --self-test` vacuous | **CLOSED** | `hardening_2` now "adds a real `check_tiers.py --self-test` entrypoint that exercises temporary tier data and fails on missing/duplicate assignments, matrix/TOML mismatch, invalid tier names, and empty fixture lists rather than silently running the ordinary checked-in-data path" (`hard:165-168`). The command is retained in both plans' Required Validation (`cert:315`, `hard:255`) and now has an owning deliverable. |
| **NEW-3** — `certification_14` sysroot-guard self-test not named | **CLOSED** | `certification_14` now: "Replace the completion-time backstop in `check_sysroot_stdlib_resource_certification_gate.py` … **and update the guard's `--self-test` completed-matrix assertion in the same PR**. Keep its supported stdlib-core invariants" (`cert:221-225`). |
| **NEW-4** — Redis/PostgreSQL loopback fidelity unspecified | **CLOSED** | The `certification_4`–`certification_8` section now bounds it: "Redis and PostgreSQL harnesses emulate only the handshake and request/response frames exercised by the certified operations, plus the malformed/early-close frames required by negative evidence. General Redis or PostgreSQL server compliance is out of scope and must not expand these PRs" (`cert:199-202`). Scope is bounded to certified frames + negative cases exactly as required. |

## Remaining material gaps

None. The four edits are precise, land in the correct owning items, and introduce no new contradictions:
- The dual cross-reference in NEW-1 is symmetric and agrees with the user-owned Phase 40 plan; no duplicate release-plan authority.
- The stable-claim dependency is captured at `milestone_40_0` (confirms hardening merged + validates `stable_support_claims.json`, `ph40:258-260`) and executed at `milestone_40_1` (`ph40:304-310`), consistent with `certification_0`.
- All ten round-1 blockers/mediums remain closed (unchanged since round 2); the round-3 deltas are additive precision only.

## Readiness verdicts

- **`rust-interop-runtime-ecosystem-certification.md`: READY.** Fully decomposed ordered one-PR items, exact evidence IDs, promotion vs. `unsupported-by-design` criteria, guard-migration closeout (now including the guard's own self-test), bounded loopback harness fidelity, acyclic Pydantic/bridge-v2 sequencing, and a Phase-40-reconciled stable-claim gate. No structural scope left for an implementer to invent.
- **`rust-interop-verification-matrix-hardening.md`: READY.** Concrete `(tier, execution_kind)` table, `diagnostic_crate_rationale`, executable-evidence provenance schema, structured rejected-syntax markers with migration + mutation self-tests, all-four-profile wiring via a `profile_runner` mechanism, and now a real `check_tiers.py --self-test` deliverable.

## Overall verdict

`SATISFIED`
