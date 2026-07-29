## Review — PR #3060 @ `2b2f613fd` (`sifr-lang/sifr`)

**Reviewed commit:** `2b2f613fd522184c65ce1cc4bce755406ac8b360` (local `HEAD` == remote `headRefOid` == PR #3060 head). Base `main`, `mergeable: MERGEABLE` / `mergeStateStatus: CLEAN`. `git merge-base HEAD origin/main` = `cad0e8aaf22296add58dea245c5f6f3465a71368` = `origin/main`, so the branch is based on current main. `git diff --numstat` on `plans/phases/index.md` and `plans/roadmap.md` is `1 0` / `1 0` — pure additions; #3058's corrected `../issues/active/…` paths appear only as unchanged context. No revert.

I re-executed the gates rather than trusting the recorded validation: `distribution_release` **125/125, 0 failures**; runner `--self-test` pass (schema registration 19); file-size guardrail **PASS** (`schema_contracts.py` 894, `release-publication.yml` 898). Working tree dirt is limited to the three unrelated submodules.

### All five pass-1 findings are closed

1. **Current base / no conflict — closed.** Verified above.

2. **Pinned canonical waiver, caller/source drift rejected — closed.** `SINGLE_MAINTAINER_APPROVAL_WAIVER_SHA256: b9630cc0…8008` is a workflow env constant (`release-publication.yml:150`), checked in-shell at `:243-248` and again authoritatively inside `resolve_publication_approvers` (`release_governance.py:648-657`) before any approval decision. GA passes `--expected-approval-waiver-sha256` through `run_stable_publication.sh:97-99,231-238`. I reproduced pass-1's exact attack — a canonical copy with `expires_at: 2099-01-01T00:00:00Z` — and it is now rejected: `single-maintainer approval waiver digest drifted`, rc=2. The literal is pinned in `schema_epoch_bootstrap_workflow_contract.sh:51`.

3. **Mode derived from the actual approval set, distinct precedence, mixed can't select waiver — closed.** `resolve_approval_decision` (`schema_bootstrap.py:105-156`) returns `distinct-reviewer` whenever any non-initiator approval exists, and `single-maintainer-waiver` only when the sole authorized approver is the initiator; the waiver is validated only in that branch, and the workflow reads `.approval_policy.mode` from that output (`:262-268`) instead of asserting it from the operation. So the pass-1 "extra legitimate approver aborts after `gh release create`" hazard is gone: owner+reviewer → `{"approvers":["release-reviewer"],"mode":"distinct-reviewer","waiver_sha256":"none"}`, which the materializer accepts. Direct probes all fail closed: empty history, wrong environment, `state: rejected`, and a non-owner self-approving (`$.owner_login: must equal the workflow initiator`).

4. **Sign-off binds initiator/approver/mode/digest — closed.** `initiator` + `approval_policy` are required in `stable_release_signoff.schema.json` and `validate_release_signoff` (`release_plan.py:324-353`) rejects `is_self != (mode == waiver)` per attempt; `oneOf` pins `waiver_sha256 == "none"` for distinct review and a real SHA-256 for waiver. Probed: self-approval under `distinct-reviewer`, distinct approver under waiver, and missing `initiator` are all rejected. `materialize_stable_publication.py signoff` now requires `--initiator/--approval-mode/--approval-waiver-sha256`, fed from the resolver's own decision.

5. **Real-waiver and boundary coverage — closed.** `approval_waiver_selftest.validate_repository_waiver` loads the actual `plans/releases/single-maintainer-approval-waiver.json`, asserts canonical bytes, JSON Schema conformance, owner/repo/environment, `expires_at == 2026-08-27T00:00:00Z`, the exact three-operation set, `require_unexpired=True`, and then drives the real CLI: all three allowed operations → waiver mode; owner+reviewer → distinct precedence; `normal`/`rollback`/`incident-roll-forward` → rc=2; no-waiver self-approval → rc=2; digest drift → rc=2; and `validate --kind single-maintainer-approval-waiver --require-canonical` (previously dead) is now exercised with `require_unexpired=True`. Contract cases pin the `ga-activation` guard, the `elif [[ -n "${approval_waiver}" ]]` rejection, and the incident runner's *absence* of `--single-maintainer-waiver` / `--expected-waiver-sha256`.

### Fail-closed / bypass assessment

A real GitHub `stable-release` environment approval remains mandatory in every path — the resolver reads `/actions/runs/{id}/approvals` and errors if no authorized approval is recorded, so an environment with no reviewers never produces one. Two independent gates keep the waiver out of `normal`/`rollback`/`incident-roll-forward`: the workflow only appends `--approval-waiver` for `ga-activation`, and `run_stable_publication.sh:97-101` `usage`-rejects the flag otherwise (verified by direct invocation). `run_incident_publication.sh:301-306` never receives it and still uses the bare-array output, so its `jq -er '.[0]'` and the demo are unbroken by `--include-policy` being opt-in.

Backward compatibility of the two newly-required evidence fields is safe: `gh api repos/sifr-lang/sifr/releases/tags/channels` lists **only `channels.json`** — no retained bootstrap-alpha evidence or stable sign-off exists, so nothing previously produced now fails `require_exact_keys`.

### Non-blocking observations (no correction required)

- **Bootstrap trust root.** For `bootstrap-*` the workspace, the waiver bytes, the governance scripts, *and* the pinned constant all come from the dispatched ref (local `workflow_call` resolves at the caller's SHA). The pin is therefore exactly as strong as `SITE_WORKFLOW_SHA256` / `LEGACY_INDEX_SHA256` — the established convention — but not stronger; it does not bind to protected `main`. This is pre-existing and systemic to the bootstrap path, not introduced here, and pass-1 explicitly accepted the pinned-constant option.
- `release-publication.yml:266-267` uses `jq -r` (not `-er`) for `.approval_policy.mode`/`.waiver_sha256`; a missing key would yield `null`, caught downstream by the materializer's `choices`. The orchestrator uses `jq -er` — worth matching.
- `run_stable_publication.sh` requires `--approval-waiver` for *every* `ga-activation`, even if a distinct reviewer approves (the file need only exist; expiry is checked only in waiver mode). Removal is already owned by `ad-hoc-distinct-release-reviewer-restoration.md`.
- The real-clock `require_unexpired=True` in the selftest makes `distribution_release` hard-fail on 2026-08-27. That is the intended forcing function and matches the `release_clippy.py:179` (`expiry < date.today()`) precedent — but it will block all PRs, not just releases.
- `plans/phases/index.md:55` uses the full relative path as link *text*, unlike every sibling row (filename only). Cosmetic.
- `resolve_distinct_approvers` is now a wrapper referenced only by tests.

**VERDICT: SATISFIED**
