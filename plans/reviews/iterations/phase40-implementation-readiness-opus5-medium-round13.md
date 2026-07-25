# Phase 40 Plan Review — Single Schema Epoch Cutover

**VERDICT: NOT SATISFIED**

The patch does achieve the headline goal cleanly: `schema_version: 2` is stated as the sole Phase 40 release-governance epoch, and all eleven listed contracts (`channels.json`, `stable-release-plan.json`, `stable-release-signoff.json`, install receipts, `sifr self version --format json`, self-update plan JSON, `qualification-artifact-index.json`, `stable-site-release-facts.json`, `stable-incident-request.json`, `stable-incident-signoff.json`, `release-profile-report.json`) are enumerated at v2 (lines 171–176). `rc` removal, no-fallback/no-migration language, and the "contracts start at version 2" rationale are all consistent. There is no lingering `schema_version: 1` obligation, negotiation, or dual-read/write path.

However, one ownership contradiction blocks a clean implementation of exactly the cutover this review is about.

## Material findings

**1. The v2 cutover of the pre-existing runtime producers (receipts, CLI JSON, self-update plan) is assigned to two milestones at once, and M40.0's exit gate is unsatisfiable if M40.2 owns it.**

- `milestone_40_0` DoD (lines 433–434) requires: *"repository search checks fail if a Phase 40 schema, fixture, **producer, or consumer** still names schema v1."* This means at M40.0 close, **no producer or consumer may emit/read v1** — including the receipt writer, `sifr self version` JSON, and the self-update-plan producer, which exist today at v1 from the preview substrate.
- `milestone_40_2` scope (lines 563–565) explicitly assigns that same work: *"Update every governed JSON producer and consumer to schema version 2 **in one cutover**, including CLI JSON, self-update plans, receipts…"*
- M40.0's own "Check in the … schemas" list (lines 363–368) covers only the new governance schemas + release-index; it does **not** list converting the existing receipt / `self version` / self-update-plan schemas, reinforcing that the runtime producer cutover is intended for M40.2.

These cannot both hold. Either M40.0 must pull the receipt/CLI/self-update producer conversion forward (making M40.2's headline "one cutover" bullet a no-op and contradicting its own scope), or M40.2 owns the cutover and M40.0 closes with v1-naming producers still present — failing M40.0's DoD search check. An implementer cannot tell which PR performs the atomic conversion of the shared producers, and one reading leaves M40.0 with an unmeetable gate. Two sequential PRs each claiming to be *the* single atomic cutover of the identical producer set is the ambiguous-ownership case the review asks to catch.

**Resolution:** assign the receipt / `sifr self version` / self-update-plan producer+consumer v2 conversion to exactly one milestone. Given M40.0's search-check gate, the cleanest fix is to state that M40.0 performs the full v1→v2 producer/consumer cutover (and add the receipt/self-version/self-update-plan schemas to its check-in list), and reduce M40.2's first bullet to "add stable channel/version support on the already-canonical v2 producers" rather than re-describing a cutover.

## Non-blocking refinements

- **`qualification-artifact-index.json` has no schema/validator check-in owner.** It is required to be v2 by the epoch section (line 174) and is produced by the M40.1 collector (line 497), but no milestone's "Check in … schema" list includes it — unlike every other governance contract, whose schema+validator is locked in M40.0. For consistency with the "lock all schemas before mutation-capable work" principle, add its schema/validator explicitly to M40.0 (or state that M40.1 checks it in).
- **"legacy facade" wording** (lines 392, 399, referring to `verification/runner/sifr_verify/profile_runner.py`) is a pre-existing component name, not a compatibility path — correct, but a one-line note that this is an existing runner (not a new compat shim) would prevent a future reader from flagging it against the no-legacy rule.

No other contradictions, omitted producers/consumers, accidental compatibility obligations, fallback behavior, or validation gaps were found across the channels/receipts/CLI/self-update/qualification/plan/sign-off/site-facts/incident/report surfaces.
