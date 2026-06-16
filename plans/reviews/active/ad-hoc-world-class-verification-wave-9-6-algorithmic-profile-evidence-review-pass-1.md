# Wave 9.6 — Algorithmic Compatibility Profile-Owned Evidence — Review Pass 1

Scope: review the Wave 9.6 task in `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1594-1598`, namely promoting `algorithmic_compatibility` from manifest/taxonomy signal to profile-owned evidence with merge running a representative subset, nightly/release running the full corpus + taxonomy delta, structured per-row metadata, and no live network for merge.

## Verdict

The four acceptance bullets are met. **No blockers; another review round is not required.** A small handful of correctness-adjacent and ergonomics concerns are listed below for follow-up — none gate merge.

## Acceptance check against Wave 9.6 bullets

1. **Merge runs a representative subset with taxonomy rows per included problem/category.**
   - `verification/profiles/merge.json:117-125` selects `algorithmic_compatibility` with suite `representative-subset`.
   - `verification/areas/algorithmic_compatibility/data/leetcode_profile_manifest.json:23-143` defines 12 rows that cover the 12 `required_categories` in `taxonomy.required_categories` (`leetcode_profile_manifest.json:6-19`).
   - `verification/areas/algorithmic_compatibility/runner.py:255-257` enforces that every required category is represented; missing-category failure is wired and tested by the variants=12 run reported in the prompt.

2. **Nightly/release run the full corpus and taxonomy delta reports.**
   - `verification/profiles/nightly.json:113-122` and `verification/profiles/release.json:113-122` both select `leetcode-full` and `taxonomy-smoke`.
   - `runner.py:363-401` runs all `*.sifr` under `corpora/leetcode/src` and then calls `build_taxonomy_artifacts`, which emits `result_artifact`, `taxonomy_artifact`, `taxonomy_markdown`, **and** `delta_markdown` against the checked-in `baseline_taxonomy` / `baseline_results`.
   - Manifest count guard: `runner.py:262-264` fails if `full_corpus.expected_fixture_count` (411) drifts from the on-disk fixture count — 411 fixtures confirmed on disk.

3. **Each problem/category has owner, expected classification, command, timeout, and result artifact.**
   - Per-row schema enforced by `validate_representative_row` (`runner.py:272-308`): `id`, `owner`, `category`, `expected_classification`, `path`, `command`, `timeout_seconds`, `result_artifact`. Unknown fields are rejected.
   - All 12 rows in `leetcode_profile_manifest.json:23-143` carry these fields and pass validation (variants=1 for profile-manifest, variants=12 for representative-subset).

4. **No live network required for merge.**
   - Merge profile is offline-only: `verification/profiles/merge.json:15-23` (`network_policy.mode = "offline"`, `cargo_policy.offline = true`).
   - The representative-subset suite reads only checked-in fixtures and invokes `target/debug/sifr check` via `runner.py:548-560`; no HTTP, no corpus fetch.
   - `profile_runner.py:128-130` exports `CARGO_NET_OFFLINE=true` to the child, so even the `cargo build` invoked by `ensure_sifr_bin` cannot reach the network.

## Non-blocking findings

1. **Rigid `command` string match for representative rows.** `runner.py:301-304` rejects any command not exactly equal to `target/debug/sifr check <path>`. This couples the manifest to the debug binary path; a release-built sifr or a custom binary location would fail validation even though the runner itself supports it via `DEFAULT_SIFR_BIN`. Consider validating only the trailing `check <path>` suffix.

2. **`expected_classification` locked to `"PASS"`.** `runner.py:296-297` rejects any value other than `"PASS"`. Wave 9.6 only requires that each row *carry* an expected classification, not that all must be PASS. This is fine today (12/12 PASS) but blocks future representative rows that test a known-failing category surface without a code change.

3. **Shared `result_artifact` path across 12 rows.** All 12 rows in `leetcode_profile_manifest.json:32, 42, 52, ...` set the same `result_artifact`. `runner.py:321` only consults `payload["representative_subset"][0]["result_artifact"]`. If a future row specifies a different path it is silently ignored. Either lift `result_artifact` to a top-level manifest field, or validate that all rows agree.

4. **`full_corpus.command` is documentation-only.** `leetcode_profile_manifest.json:147` declares `target/debug/sifr check <dir>` but `runner.py:373-392` ignores it and iterates fixtures individually. Either honor the manifest command literally, or drop the field — it currently misleads a reader into thinking `sifr check <dir>` is the executed command.

5. **`full_corpus.timeout_seconds` not enforced per fixture.** `runner.py:375` hardcodes `timeout_seconds=30` per fixture. The manifest's `full_corpus.timeout_seconds=1800` (`leetcode_profile_manifest.json:148`) appears to be a suite-level budget but is never read. Either document it as the suite budget and add a separate per-fixture timeout to the manifest, or read it.

6. **Hardcoded `--generated-on` dates.** `runner.py:458` (`2026-06-16`) and `runner.py:497` (`2026-06-13`). Determinism is good, but the date is now divorced from the baseline file it pairs with. Suggest moving the value into the manifest (e.g. `taxonomy.generated_on`) so it lives next to the baseline and review notices when it drifts.

7. **`cargo build -q -p sifr` lacks `--locked`.** `runner.py:541-545` builds without `--locked`. `CARGO_NET_OFFLINE=true` (set in `profile_runner.py:129-130`) mitigates the network risk, but merge's `cargo_policy.locked = true` is not enforced at this call site. Add `--locked` for consistency with the cargo contract.

8. **`leetcode-full` resource label `external-corpus`.** `manifest.json:7-11`, `nightly.json:119-122`, `release.json:119-122` classify this suite as `external-corpus`, but the corpus is checked-in under `corpora/leetcode/src`. The label reads as "external network" to a casual reader. If the intent is "third-party content not authored by Sifr," consider renaming or documenting it; otherwise drop to `default-local + long-running`.

9. **Baseline `results` array is empty.** `leetcode_full_baseline_results.json` ships `case_count: 411` with `results: []`. This is consistent with the empty failure baseline (0 fail), but the diagnostic seed maps built from the baseline (`build_full_corpus_failure_taxonomy.py:301-315`) will be empty — first taxonomy regression will fall back to heuristics. Worth a one-line note in the baseline file or alongside it so a future bless run knows to repopulate `results` when failures appear.

## Coverage of supporting evidence

- Profile emit-plan diff shows `algorithmic_compatibility` rows in all four profiles with the suites stated above — matches what's in the JSON.
- Local validation reported by the user is consistent with the runner code: profile-manifest variants=1; representative-subset variants=12 (one per category); leetcode-full variants=411; taxonomy-smoke variants=1; delta report `+0` against the all-PASS baseline.
- file-size, py_compile, and jq checks reported clean — runner is at ~590 lines, well under the 900-line guardrail.

## Recommendation

Ship this as Wave 9.6. None of findings 1–9 block merge; (1), (2), (6), and (7) are worth a follow-up cleanup but can be folded into a later sweep without re-opening the wave.
