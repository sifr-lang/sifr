The two prior vocabulary review passes never ran — both files contain only a model-selection error line, so this is effectively pass 1 and I'll review from scratch.

# Verification Vocabulary Review — Decisive Findings

## Verdict

The conceptual architecture is right and should not be restructured. The vocabulary is 6/7 correct. **Rename `lanes` → `profiles`; keep everything else.** Patch the phase doc with the edits below before implementation — they are mechanical and cheap now, expensive after PR 6.

## Q1 — Term-by-term assessment

**`suites`, `cases`** — Universal. Every reference repo you sampled uses both. Keep.

**`runner`** — Keep. It is literal (the directory holds the engine that runs things), and CPython's `libregrtest` — your closest structural analogue, a stdlib-only Python runner with resource policy — is called "the test runner" throughout CPython's docs. No reader will stumble.

**`areas`** — Keep. This is less standard than suite/case but it is *not* invented vocabulary: rust-lang triages with `A-*` labels that literally mean "area," and "ownership area" reads naturally in review comments ("this fixture belongs to the diagnostics area"). The directory form `verification/areas/diagnostics/` is self-explanatory to a new contributor, which is the actual test.

**`schemas`** — Keep. Plain and accurate.

**`policy`** — Keep. Slightly unusual as a directory name, but the doc gives it a crisp contract (machine-facing operational rules) and no standard term does better. "config" would be worse — policy signals *rules*, not knobs.

**`lanes`** — **Rename to `profiles`.** Two independent reasons, either sufficient:

1. **You already shipped the word.** The single public validation facade is `scripts/run_all_tests.sh --profile create-pr` / `--profile merge`. The phase doc's own hygiene standard is one word per concept and no duplicate surfaces for the same gate — yet as written, a contributor types `--profile merge` and the runner reads `lanes/merge.json`. That is exactly the split-brain the phase exists to eliminate. Renaming the public flag instead is worse: the doc commits to the facade being the stable public contract.
2. **"Lane" has a reserved meaning in compiler engineering.** In Rust, LLVM, and SIMD documentation generally, a *lane* is a vector lane. Sifr compiles to Rust; the day Sifr documents anything about vectorization or portable SIMD, "lane" will mean two unrelated things in the same repo. "Profile" has a mild collision with Cargo build profiles, but that overlap is semantically *aligned* (a named bundle of execution policy), whereas lane-vs-lane is a true homonym.

## Q2/Q3 — Alternatives rejected

- **`domains`** (for areas) — Rejected. More abstract, carries DNS/DDD baggage, and reads academic. "Which domain owns this fixture?" is worse English than "which area owns this fixture?" No reference repo uses it.
- **`harness`** (for runner) — Rejected. Defensible (TypeScript uses it), but renaming buys nothing over `runner`, and "harness" vaguely connotes the *whole* apparatus including cases. `runner` names exactly what the directory contains: the engine.
- **`specs`** (for schemas) — **Rejected hard.** In a compiler repository, "spec" means the language specification. A serious compiler repo must keep that word free. This alone disqualifies the `domains/profiles/suites/cases/harness/specs/policy` slate as a package.
- **Rust-like `modes`** — Rejected as a top-level concept. In compiletest, a mode is an *execution strategy* (ui, run-make, incremental), orthogonal to ownership. Your areas are ownership domains. The doc already gets this right: "areas own suites by execution mode when useful" — mode is a suite-level attribute inside an area, not a taxonomy peer. Promoting it would create the competing-taxonomy problem the doc explicitly forbids.
- **CPython-like `resources`** — Rejected as a top-level concept for the same reason. CPython's `-u network,largefile` resources are cost classes, and the doc already places resource classes exactly where they belong: inside profile policy. Correct as designed.

## Q4 — Recommended final vocabulary and structure

Top-level concepts (five, as the doc already insists): **`runner`, `schemas`, `profiles`, `areas`, `policy`.** Subordinate, area-local concepts: **`suites`, `cases`** (with `fixtures`, `baselines`, `corpora`, `data` as on-disk material kinds). The 5+2 split in the doc is correct — do not promote suites or cases to top level.

```text
verification/
  README.md
  policy/
  schemas/
    profile.schema.json
    area.schema.json
    suite.schema.json
    case.schema.json
    result.schema.json
  profiles/
    create-pr.json
    merge.json
    nightly.json
    release.json
  runner/
    sifr_verify/
      __main__.py
      profiles.py
      areas.py
      scheduler.py
      results.py
      schemas.py
  areas/
    core_language/
    ... (14 areas, unchanged)
```

The end-to-end story becomes one sentence with one vocabulary: *`run_all_tests.sh --profile create-pr` invokes `sifr_verify`, which loads `verification/profiles/create-pr.json`, which selects areas and suites, which own cases.* That sentence is the "serious compiler repo" test, and it now contains zero invented words.

**Keep all 14 areas.** Areas cost one manifest each, the "subdirectories created only on first use" rule kills empty ceremony, and merging `algorithmic_compatibility` into `ecosystem_compatibility` would blur two genuinely different contracts (scored corpus projection vs. pinned non-blocking OSS signal). The `regression`-vs-contract-ownership tension is real but the doc already resolves it explicitly (minimized fixed bug → `regression`), matching Bun and LLVM practice.

## Q5 — Precise doc edits

Patch `issues/ad-hoc-repository-architecture-and-verification-surface-cleanup.md`:

1. **Global rename**, lane → profile, in the verification sections only: `lanes/` → `profiles/`, `lane.schema.json` → `profile.schema.json`, `lanes.py` → `profiles.py`, "Lane files" → "Profile files", "Lane-local shape" → "Profile-local shape", "lane policy" → "profile policy", "thin lane dispatcher" → "thin profile dispatcher", "Verification Lane Normalization" (PR 7) → "Verification Profile Normalization", "lane-by-lane equivalence" → "profile-by-profile equivalence", and the Core Principles line "Lanes select verification areas; lanes do not own fixtures" → "Profiles select verification areas; profiles do not own fixtures."

2. **Replace** the concept-definition bullet:
   - Old: `` `lanes` answer when verification runs and with what resource budget. ``
   - New: `` `profiles` answer when verification runs and with what resource budget. The name deliberately matches the public `--profile` flag of `scripts/run_all_tests.sh`; the flag value resolves directly to `verification/profiles/<name>.json`. ``

3. **Add** under Verification Architecture rules (closes a real ambiguity — the doc currently uses "case" and "fixture" interchangeably in places):
   > A case is the unit of verification: a manifest or suite entry plus the fixture files and expected outputs it references. A fixture is on-disk input material only; fixtures are never executed except as part of a case.

4. **Add** a schema-strictness rule (without it, committed `.schema.json` files can silently drift beyond what the stdlib validator actually checks, and the schemas become false advertising):
   > The runner must reject, with an error, any committed schema that uses keywords outside the supported subset. Silent ignoring of unsupported schema features is forbidden.

5. **Add** an area-adapter boundary rule (prevents 14 areas growing 14 mini-frameworks — the one real over-engineering risk in this design):
   > An area `runner.py` implements only the schema-defined adapter interface (discover, execute, report results for its cases). Scheduling, parallelism, retries, resource classes, and report generation belong exclusively to `sifr_verify`; area adapters may not implement their own.

6. **Add** a naming-convention rule so the mixed casing reads as intentional rather than accidental:
   > Profile names are kebab-case because they are CLI-facing (`--profile create-pr`). Area and suite names are snake_case because they are identifier-facing (directories, manifest keys, Python modules).

**Tradeoff acknowledged for the rename:** "lane" arguably evokes parallel execution streams better than "profile," and Cargo also uses "profile" for build settings. Neither outweighs flag/vocabulary unification plus the SIMD-lane collision. If you reject the rename, the fallback is renaming the public flag to `--lane` — but that breaks the facade-stability commitment and every existing doc reference, so it's strictly worse.

One housekeeping note: both `reviews/ad-hoc-repository-architecture-and-verification-vocabulary-fable-review-pass-{1,2}.md` contain only a model-error line ("There's an issue with the selected model (fable-high)…") — they're dead artifacts and should be deleted or replaced with this pass rather than archived.
