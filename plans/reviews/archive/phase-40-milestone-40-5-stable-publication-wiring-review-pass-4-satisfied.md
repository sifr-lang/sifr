## Review — Phase 40 / M40.5 stable publication production wiring, pass 4

I read the three archived reviews, the full 27-file diff, all 14 untracked files, the surrounding preview/bootstrap/drill steps, `release-publication-prepare.yml`, `preview-release.yml` (the only caller), `generation.py`, `release_plan.py`, and `verification/json_schema_202012.py`. I re-ran validation rather than trusting reported numbers.

**Independently reproduced:** distribution `full` **63/63**, whole area **120/120**, zero failures (governance 14/14, incident-recovery 9/9, stable-prepare 7/7, primitives 4/4, stable-publication 9/9). Both workflow contract cases pass standalone; both workflows parse as YAML; all `scripts/distribution/*.sh` pass `bash -n`; `file-size guardrails: PASS (2924 files, limit 900)` with `release-publication.yml` at 899 and the largest new file (`stable_publish_selftest.py`) at 892; `git diff --check` clean. Diff is Phase-40 distribution only — no Rust-interop implementation — and the contract test pins `workflow.count("\n  publish:\n") == 1`.

### Pass-3 remediation: closed, and non-vacuously

**Secret-process narrowing — closed.** `run_stable_publication.sh:92-98` requires `SITE_TOKEN`, captures `site_token`/`marketplace_pat` as plain (non-exported) shell variables, then `unset SITE_TOKEN VSCE_PAT`. After that line neither name exists in the orchestrator environment; each is reintroduced only as a command-scoped prefix:

- `GH_TOKEN="${site_token}"` on `verify_site_workflow_identity.sh` (`:227`, `:328` — that script hard-requires `GH_TOKEN` at `:30`), the `gh api …/dispatches` POST (`:368`), and `poll_site_release_run.sh` (`:372`).
- `GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="${marketplace_pat}"` on `publish_marketplace_extension.sh` (`:259`).
- `GH_TOKEN="" SITE_TOKEN="" VSCE_PAT=""` on `run_stable_public_smoke.sh` (`:382`).

I enumerated every remaining descendant: only first-party governance code and `gh`/`jq`/`cmp` inherit the job `GH_TOKEN`, which they need. `publish_marketplace_extension.sh:88` independently re-clears `GH_TOKEN`/`SITE_TOKEN` for `vsce`, and `run_stable_public_smoke.sh:132-141` clears all three for both the dispatcher `sh` and the freshly installed `sifr`. The contract test pins all five literals (`:138-145`).

**Schema parity tests — closed, and verified real.** `schema_contracts.py:149-167` adds the two `copy.deepcopy` + `validate_instance` rejections. I confirmed neither is vacuous by copying the schema tree to a temp dir and removing the constraint: dropping the prepare `allOf` makes the `publication_state: "activated"` + `mode: "initial"` fixture *accept*, and relaxing the sign-off `repository` `const` makes `example.invalid/site` *accept*. `verification/json_schema_202012.py:133-149` genuinely evaluates `allOf`/`if`/`then`/`const`. The fixture additions (`publication_state`, `next_generation`, `site_publication`) keep the positive path valid, and `release_plan.py:357-367` enforces the Python-side twin.

### Pass-1 and pass-2 remediations: still sound

Node 22 + `npm ci --ignore-scripts --prefix stable-source/editor_integrations/vscode` with `GH_TOKEN: ""` before any secret appears; `VSCE_BIN` refused unless executable in both the orchestrator (`:99`) and the adapter (`:46`); prepare and publish both execute `governance-source`/root checked out at `${{ github.sha }}`, which the orchestrator proves equals local `HEAD` **and** `origin/main`, with `--workflow-ref` regex-pinned to `refs/heads/main`; ancestry loop over the *plan's* `source.commit` and `evidence_commit` against the freshly fetched `refs/remotes/origin/main`, with `fetch-depth: 0` so it can't spuriously reject; per-attempt `stable-release-signoff-<v>-attempt-<run>-<attempt>.json`; paginated `--slurp …?per_page=100` in both `fetch_governance` and `upload_or_verify_governance` and in `publish_stable_release.py:159-185`; docs name the provisioning, the per-attempt asset, and now the secret narrowing.

### Whole-wave reassessment

Ordering, generation reservation, and resume all hold. I traced the crash matrix rather than trusting the contract test: snapshot-uploaded-but-clobber-failed leaves `{G, G+1}` retained with live still `G`, so `allocate_next_generation` returns `G+2` and the abandoned `G+1` stays burned; re-running the *same* summary is rejected at `revalidate` ("retained generations changed after prepare") before any mutation. Post-clobber resume detects `activated`, `_recover_realized_mutation` reproduces the exact predecessor from retained history and byte-compares against live, `next_generation > proposed_generation` is enforced in `validate_stable_prepare_summary`, and the generation-scoped site-facts asset is byte-identical across attempts, so `upload_or_verify_governance` converges. `allocate_next_generation` additionally requires live to equal its own retained snapshot, which the pending path preserves. `--clobber` appears exactly once and only on `channels.json`; `publish_stable_release.py` has none, pages assets, and byte-compares by immutable asset ID with `text=False`. The prepare summary is bound to `needs.prepare.outputs.summary_sha256` before revalidation. `poll_site_release_run.sh --result-out` emits `jq -cnS` — I checked byte-for-byte that this equals `canonical_json_bytes`, so `_site_publication`'s `require_canonical=True` read will not spuriously fail. Preview/bootstrap/drill are gated off by explicit `governance_mode` guards, `preview-release.yml` still type-checks against the now-optional inputs and omits the optional `VSCE_PAT`, top-level `contents: read` is re-granted only on `publish`, and registration is consistent across manifest, runner (with the `include_stable_publication` dedup flag), all three profiles, coverage matrix, `REQUIRED_SUITES`, and the gate inventory. Every ledger figure in `plans/issues/active/phase-40-stable-channel-ga-execution.md` matches what I measured, including the 899-line count.

On the safety question: I found no path that activates stable before exact release and Marketplace state.

### Non-blocking notes (all carried forward, none new to this wave)

- The secret narrowing is pinned by literal substring, not by execution. The fake `vsce` and fake stable dispatcher already exist in `stable_publish_selftest.py`; a `test -z "${GH_TOKEN:-}"` in each would turn the text pin into a behavioral one. The control is defense-in-depth over integrity-pinned targets, so this is hardening, not a gap.
- Preview/bootstrap asset inventory (`release-publication.yml:419,671`) is still unpaginated; pre-existing, fail-closed, and the paginated form now exists in the orchestrator.
- Marketplace server-side re-signing remains the one open first-publish risk by design — one live-Gallery validation before GA.
- `origin/main == workflow_commit` means main advancing during the approval window aborts the run. Fail-closed and re-dispatchable, but operationally worth knowing.
- The orchestrator resolves `repo_root` from `BASH_SOURCE` for the git identity checks but invokes adapters via CWD-relative `scripts/distribution/…`; identical in Actions and in the selftests, divergent only if invoked from elsewhere.
- Untested-but-shared: the ancestry loop's evidence-commit iteration, `publish_stable_release.py:145`'s `target_commitish` equality on resume, and `_recover_realized_mutation`'s `sha256_file` on a hypothetical directory matching `SNAPSHOT_RE`.
- `plans/reviews/active/phase-40-milestone-40-5-stable-publication-wiring-review-pass-4.md` is a 0-byte file. I wrote to no file, per your constraint.

SATISFIED
