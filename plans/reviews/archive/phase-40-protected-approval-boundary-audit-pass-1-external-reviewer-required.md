## VERDICT: EXTERNAL REVIEWER REQUIRED

No compliant, truthful path exists. Both the schema-v2 bootstrap and GA activation are gated on a GitHub-server-recorded `stable-release` approval by a login that is not the triggering actor, and `yaseralnajjar` is the only identity with repository access.

---

## Live facts (independently verified)

| Claim | Result |
|---|---|
| `stable-release` has `protection_rules []` | ✅ `{"protection_rules":[], "can_admins_bypass":true}` |
| Collaborators / org members = only `yaseralnajjar` | ✅ both return exactly `yaseralnajjar` |
| No pending invites, outside collaborators, or teams | ✅ `invitations []`, `collaborators?affiliation=outside []`, `teams []`, `orgs/sifr-lang/invitations []` |
| Public channels asset is schema v1, alpha.1 / beta.14 | ✅ `{"schema_version":1,"channels":{"alpha":"0.1.0-alpha.1","beta":"0.1.0-beta.14"}}` |
| Run `30416219284` artifacts unexpired, source `c9d611fb…` | ✅ all 7 artifacts `expired:false`, `expires_at 2026-08-28`; `head_sha c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9`, conclusion `success` |

Two additional facts worth recording: the only environments that exist are `stable-release`, `stable-release-drill`, and `staging - docs` — **`preview-release` does not exist**, and all three have empty `protection_rules`.

---

## The binding gate

The frozen policy (`plans/issues/active/phase-40-stable-channel-ga-execution.md:16-18`) is enforced in code, not just prose:

- `.github/workflows/release-publication.yml:243-251` — for `bootstrap-*` modes, the protected job fetches **GitHub's own** `actions/runs/${GITHUB_RUN_ID}/approvals` and pipes it through `resolve-publication-approvers --initiator "${GITHUB_TRIGGERING_ACTOR}" --environment stable-release`.
- `scripts/distribution/run_stable_publication.sh:218-225` — the GA path does the identical fetch, then `jq -er '.[0]'`, which fails on an empty list.
- `verification/areas/distribution_release/governance/schema_bootstrap.py:86-119` — `resolve_distinct_approvers` keeps only entries with `state == "approved"`, an `environments[].name == "stable-release"`, and `login.casefold() != initiator.casefold()`; if none remain it calls `fail(...)`. Exit 2, fail-closed.
- Same file, `_validate_approvers` (226-243) — the written evidence/sign-off must carry a non-empty approver list, each entry unique and `!= initiator`. Independent second checkpoint.

**Consequence of today's state:** because `protection_rules` is empty, the job never pauses, so `/approvals` returns `[]`, so the resolver fails and the run aborts before mutating anything. The current configuration is not a hole — it is a fail-closed stop.

---

## Why every no-second-human alternative fails

| Route | Why it fails |
|---|---|
| **Add `yaseralnajjar` as required reviewer, self-approve** | The approval record is created, but `login == initiator` → filtered out → empty list → `fail`. Explicitly pinned by the `"case-insensitive self approval"` failure test (`schema_bootstrap_selftest.py:144-153`). |
| **Admin bypass** (`can_admins_bypass: true`) | Bypass skips *waiting* for reviewers; it does not synthesize an `approved` entry by a non-initiator. The resolver reads the approval history, not the environment rule, so bypass yields `[]`. |
| **Leave `protection_rules` empty and just run it** | Same `[]` → hard fail at `release-publication.yml:246`. |
| **GitHub App / bot as reviewer** | GitHub environment required-reviewers accept only **users and teams**; Apps cannot be listed. There is no App installation on this repo (`/installations` → 404). |
| **Machine/second account you control, used as initiator or approver** | This is the one route that would pass the string comparison — and it is exactly the shortcut to reject. It manufactures an identity split for a single human, producing a sign-off that *asserts* a non-initiating `release/distribution` reviewer while none existed. That is a false evidence record, not compliance. It also still requires granting repo access to a new account, i.e. the same external action, minus the honesty. |
| **`schedule` trigger** | Would change `GITHUB_TRIGGERING_ACTOR`, but creates no approver at all, so the list is still `[]`. Both publication entrypoints are `workflow_dispatch` / `workflow_call` only; adding a schedule is a workflow rewrite. |
| **Route bootstrap through a laxer environment** | `release-publication.yml:117` selects `preview-release` only when `governance_mode == 'preview'`; `bootstrap-index` lands on `stable-release`, and the resolver hardcodes `--environment stable-release` regardless. An approval on any other environment is rejected (`"wrong approval environment"` test, selftest:161-176). |
| **Edit the workflow to drop or stub the check** | Rewrites the frozen requirement, and the repo detects it: `verification/areas/distribution_release/cases/schema_epoch_bootstrap_workflow_contract.sh:47,48,104` asserts the workflow literally contains the approvals fetch, the `--initiator "${GITHUB_TRIGGERING_ACTOR}"` flag, and the `"approval by someone other than"` message. The distribution area gate fails. |
| **Hand-write the approvers into sign-off** | `_validate_approvers` rejects empty and initiator-matching entries, and the evidence is materialized in-run from the live API response — a hand-edited value would not match the run's approval history. |

Milestone 40.5 exit criteria (`plans/issues/active/…:404-410`) require a *protected, truthful* publication "with fresh … approval". There is no reading under which a single human satisfies "non-initiating reviewer."

---

## Minimal exact external action required

One action, taken by a human other than `yaseralnajjar`:

1. **Grant a second genuinely distinct human account `write` access** to `sifr-lang/sifr` — either as a repo collaborator or, preferably, as a member of a `release/distribution` org team (`gh api -X PUT repos/sifr-lang/sifr/collaborators/<login> -f permission=push`). That person must accept the invitation.
2. **Configure `stable-release` required reviewers** to that account/team, and enable `prevent_self_review` for defense in depth:
   ```
   gh api -X PUT repos/sifr-lang/sifr/environments/stable-release \
     -F prevent_self_review=true \
     -f 'reviewers[][type]=User' -F 'reviewers[][id]=<user_id>'
   ```
   Consider also setting `can_admins_bypass=false` so admin bypass cannot skip the pause.
3. Then run `preview-release.yml` with `governance_mode=bootstrap-alpha`, then `bootstrap-index` (channel `beta`, with the fresh alpha version), and later the GA activation — **`yaseralnajjar` initiates, the second human approves each run**. Each attempt (initial and resume) is a distinct `run_id`, so each naturally requires its own fresh approval, satisfying the frozen resume clause without extra machinery.

Everything else is ready: the qualification artifacts for `c9d611fb…` are unexpired until 2026-08-28, so there is roughly a month of headroom before a re-qualification run would be needed. The single blocker is the second human.

No files were modified and no external state was changed during this audit.
