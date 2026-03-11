# Phase 31 LeetCode Corpus Policy

Phase 31 uses two version-controlled artifacts:

- `verification/leetcode/phase31_corpus_inventory.json`
  - raw inventory of every `audits/leetcode/*.sifr` fixture in the repository
  - tracks fixture paths, paired Python sources, oracle shape, and whether the fixture is part of the curated seed corpus
- `verification/leetcode/phase31_seed_corpus.json`
  - canonical Phase 31 seed corpus used for deterministic runner baselines
  - contains `50` curated problems with explicit topic, difficulty, scope, oracle, and timeout metadata

## Seed Corpus Contract

The seed corpus is the authoritative Phase 31 baseline for milestone `31_1`.

- Minimum size: `>= 50` problems
- Difficulty mix:
  - must include `easy`, `medium`, and `hard`
- Topic coverage:
  - `arrays`
  - `strings`
  - `hash_map`
  - `dp`
  - `graphs`
  - `trees`
  - `backtracking`
  - `math`
  - `heap_priority_queue`
  - `two_pointers_sliding_window`
- Scope classification:
  - `in_scope`
  - `blocked_feature`
  - `out_of_scope_external_dep`
- Oracle modes:
  - `embedded_asserts`: the fixture contains self-checking `assert` statements; a successful `sifr run` is a passing oracle
  - `no_oracle`: the fixture currently has no embedded sample assertions and is tracked for compile/runtime compatibility only

## Determinism Rules

- The runner always sorts cases by problem id before execution.
- The runner resolves a single release `sifr` binary (`target/release/sifr`) and reuses it for the whole corpus run.
- The runner always executes `sifr check` before `sifr run` so failure stages are stable.
- The runner emits structured JSON with a stable top-level shape:
  - `phase`
  - `manifest`
  - `output`
  - `command_prefix`
  - `summary`
  - `results`
- Each result records:
  - case identity and metadata
  - per-stage command, duration, timeout flag, return code, stdout, and stderr
  - stable status values: `PASS`, `NO_ORACLE`, `CHECK_ERROR`, `RUN_ERROR`, `TIMEOUT`

## Generation Commands

Regenerate the seed corpus, seed summary, and full inventory:

```bash
python3 scripts/build_phase31_leetcode_assets.py
```

Run the full seed corpus:

```bash
python3 scripts/run_phase31_leetcode.py \
  --manifest verification/leetcode/phase31_seed_corpus.json \
  --output verification/leetcode/phase31_seed_results.json
```

By default this command will build `target/release/sifr` once if it is missing, then reuse that binary for every case.

Run the milestone demo sample:

```bash
python3 scripts/run_phase31_leetcode.py \
  --manifest demos/m31_1_leetcode_runner_demo/corpus.json \
  --output demos/m31_1_leetcode_runner_demo/results.json
```

Validate the Phase 31 contract locally:

```bash
python3 scripts/test_phase31_leetcode.py
```

Generate the milestone `31_2` taxonomy artifacts:

```bash
python3 scripts/build_phase31_leetcode_taxonomy.py
```

Validate the taxonomy classifier and spot-audit contract:

```bash
python3 scripts/test_phase31_leetcode_taxonomy.py
```

Generate the milestone `31_3` remediation backlog artifacts:

```bash
python3 scripts/build_phase31_leetcode_remediation_backlog.py
```

Validate the remediation backlog contract:

```bash
python3 scripts/test_phase31_leetcode_remediation_backlog.py
```

## Current Baseline Notes

- The raw `audits/leetcode` fixture directory currently contains `411` Sifr fixtures.
- Oracle distribution in the raw inventory:
  - `embedded_asserts`: `208`
  - `no_oracle`: `203`
- The historical phase-31 artifacts in `audits/leetcode/*.md` are informative but not authoritative because they disagree on corpus size and pass-rate totals.
- The current taxonomy artifacts are generated from `verification/leetcode/phase31_seed_results.json`:
  - `verification/leetcode/phase31_failure_taxonomy.json`
  - `verification/leetcode/phase31_failure_repros.json`
  - `verification/leetcode/phase31_spot_audit.json`
  - `verification/leetcode/phase31_failure_report.md`
- The remediation planning artifacts are generated from the taxonomy outputs:
  - `verification/leetcode/phase31_remediation_backlog.json`
  - `verification/leetcode/phase31_remediation_backlog.md`
