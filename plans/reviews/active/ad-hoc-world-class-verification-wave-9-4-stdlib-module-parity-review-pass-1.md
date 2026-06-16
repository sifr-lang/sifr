I've inspected all changed files plus the inventory fixtures, manifest wiring, profile selections, profile_runner code, and the underlying stdlib intrinsic surface in `crates/sifr_stdlib`. Findings below.

## Blockers

**None.** Wave 9.4 honestly converts stdlib parity into module-owned executable evidence for supported namespaces, with fail-closed inventory validation, correct profile routing, and clean known-gap accounting.

## Verifications passed during review

**Module-owned executable evidence for supported namespaces**
- `selected_entries` (check_stdlib_module_parity.py:179) filters by `support_status == "supported"`, then by `profile` (`merge` for merge scope, `merge|full` for full scope). Merge → `math + string`; full → `math + string + env + time`. Matches the summary and the phase target's merge/nightly split for stdlib parity.
- `run_entries` (check_stdlib_module_parity.py:186) returns a hard failure if the selected scope's filter is empty (line 188-189), so a future regression that removed all `merge`-profile supported rows would not silently turn merge green.
- Each entry executes `cargo run --locked -q -p sifr -- check <fixture>` with a 15s timeout (line 199), routed through the area runner so the per-case timing and failure tail are captured in the result JSON.

**Honest, fail-closed known-gap rows**
- `support_status == "known_gap"` requires a `known_gap` string reason (line 130), and `profile == "inventory-only"` excludes them from `selected_entries`. Self-test mutation pops the `known_gap` reason and confirms the validator fails.
- Known-gap rows still require non-empty `supported_apis` AND token coverage in the fixture (lines 138-145). So a legacy fixture that drifts away from its stale APIs will fail inventory rather than silently become unrepresented.
- The legacy fixtures (`02_json`, `04_re`, `05_collections`, `06_io`, `09_random`, `10_hash_encoding`) match the gap reasons given. I cross-checked: `05_collections.sifr` does import `Set` from `sifr.collections`; `06_io.sifr` still has `except str as e`; `09_random.sifr` imports `random_choice` which is not exported from `_sifr.crypto`'s random_module surface.

**API token coverage**
- `validate_api_coverage` (line 156) requires every declared API for every entry to be substring-matched in the fixture, with unique API names enforced per entry. Self-test mutates the first supported API's tokens to `["__missing_stdlib_token__"]` and asserts the validator fails.
- Spot-checked all 10 entries: every declared API token is present in the corresponding fixture. For instance the math entry's `"pi >"` and `"e >"` tokens are specific enough to require the `>` comparison rather than incidental substring matches.

**Profile wiring actually executes**
- `profile_runner.run_core_guardrails` (profile_runner.py:277-291) unconditionally runs `complexity-resource`, `module-inventory`, `module-merge-check`, then loops over remaining selected `stdlib_parity` suites, skipping the always-on four. The skip set matches the four guardrail suites, so additional profile selections (specifically `module-full-check` in nightly/release) are picked up by the loop.
- Selected-areas entries match the policy: create-pr/merge → `module-merge-check` (redundant with the always-on call, see suggestion 2); nightly/release → `module-full-check`. The selection is consistent with the phase target table for stdlib parity.

**Schema/report accounting**
- `total_variants` in `runner.py` sums `len(case_results)`, which equals the case count per suite (one variant per case). Manifest defines `module-inventory` with 2 cases (`module-inventory` and `module-inventory-self-test`), matching the reported `variants=2`. `module-merge-check` and `module-full-check` each have 1 case → `variants=1`. Area-level `variants=8` reconciles: 1 (complexity-resource) + 1 (namespace-demos-check) + 1 (namespace-leetcode-check) + 2 (module-inventory) + 1 (module-merge-check) + 1 (module-full-check) + 1 (audit-fixtures) = 8.
- `zip(cases, case_results, strict=True)` enforces the case/result list parity.

**Maintainability / file-size**
- `check_stdlib_module_parity.py`: 264 lines. `runner.py`: 181 lines. `profile_runner.py`: 581 lines. All well under the 900-line cap.

**Path / hermeticity**
- `resolve_repo_path` (line 245) rejects paths that escape `REPO_ROOT` after `.resolve()`.
- The checker only invokes `cargo run --locked -q -p sifr -- check <fixture>` — no network and no fixture writes outside the repo. Aligns with the hermetic merge/create-pr rule.

## Non-blocking suggestions

1. **Empty-token coverage hole.** `validate_api_coverage` (line 172) accepts `tokens: [""]` because `"" in source` is always true. Add a `len(token) > 0` (or `token.strip()`) guard so an accidentally-blank token can't satisfy coverage. Tighten the self-test to cover this branch.

2. **Always-on `module-merge-check` plus selected-areas entry duplicates the run path.** `run_core_guardrails` already runs `module-merge-check` unconditionally; create-pr and merge also list it in `selected_areas`, but the loop skip set excludes it so the duplicate is suppressed. The dead `selected_areas` entry reads as the active source of truth — consider either (a) removing `module-merge-check` from `selected_areas` in create-pr/merge, or (b) dropping the unconditional call and letting `selected_areas` drive everything. Today's behavior is correct; the redundancy invites a future divergence.

3. **Nightly/release run `math + string` twice.** `module-full-check` covers the merge-profile entries plus env + time, but the always-on `module-merge-check` already ran them. Either narrow `module-full-check` to `profile == "full"` entries only, or scope `selected_entries(scope="full")` to exclude `profile == "merge"` rows. The double-execute is cheap because the fixtures are small `check` calls, but it inflates `cargo run` cost and adds confusion when investigating timings.

4. **Hash + encoding share one known-gap row.** `stdlib-module-hash-encoding` conflates `sifr.hash` (sha256/md5 are present per `crates/sifr_stdlib/src/crypto_regex_uuid.rs`) with `sifr.encoding` (base64 helpers also intrinsic-declared). The known_gap reason says hash APIs are present but the fixture is stale on encoding. Splitting into two rows — a supported hash row pointing at a new minimal sha256/md5 fixture, and an inventory-only encoding row keeping the legacy fixture — would let hash APIs graduate to executable evidence without rebuilding the encoding example. This is a follow-up promotion, not a blocker.

5. **`supported_apis` naming is overloaded.** For known-gap rows it lists APIs the *fixture* demonstrates, which can include unsupported ones (`random_choice`, `base64_encode`/`base64_decode`). Either rename to `example_apis` (with an optional `supported: bool` field per API) or split into `supported_apis` and `unsupported_apis`. Today this is purely a clarity issue — the validation logic is consistent.

6. **Self-test exercises only two failure paths.** `run_self_test` mutates "missing API token" and "missing known_gap reason." Many other validator failures are unexercised: bad `schema_version`/`area`, unsorted IDs, duplicate IDs, missing required string fields, empty `supported_apis`, fixture-doesn't-exist, `zero_example_inventory` with APIs declared, `command != "check"`, unknown `support_status`/`profile`. Adding mutations for at least the sort/uniqueness and zero-example invariants would harden the gate.

7. **Math fixture imports six APIs that aren't inventoried (and aren't called).** `01_math.sifr` imports `tan, abs_val, pow_val, min_val, max_val, round_val` but the bodies never call them, so the inventory rightly skips them. The inventory's silence is honest, but a follow-up that either (a) declares them with `tokens` that match the imports or (b) extends the fixture to actually invoke them would tighten the "every inventoried supported API has example coverage" requirement against future math additions.

8. **`SUPPORTED_PROFILES` overloads "profile".** Inside the checker, `profile` means lane (merge/full/inventory-only), while in verification jargon "profile" is create-pr/merge/nightly/release. Renaming the inventory field to `lane` or `merge_lane` would make the checker easier to read.

9. **`module-full-check` description in the manifest.** The suite name suggests "all modules" but actually means "modules whose lane is merge or full." A short description (or a `description` field on the manifest suite) would prevent reviewers from inferring it covers known-gap rows too.

10. **15-second `cargo run` timeout is tight on a cold cache.** `module-merge-check` runs in `run_core_guardrails`, which is sequenced before the `crate_tests` step that exercises `cargo` more broadly. In practice, the `audit-fixtures` invocation just above warms the binary, so 15s holds. If the audit-fixtures step ever moves later, this lane could flake on cold hosts. Consider raising the per-fixture timeout to ~30s or move the warm-up call explicitly upstream.

## Recommendation

**No additional review round is needed.** Wave 9.4 is ready for PR/merge. The non-blocking items can be threaded into a Wave 9.5+ follow-up or absorbed at Wave 10 closeout; none change correctness, fail-closed posture, or the merge/nightly boundary.
