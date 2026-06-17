# Review Round 16: NOT SATISFIED

The active-surface checks pass (verification taxonomy script, no stale rename refs found outside `./plans` for any of the round-16 renames), but several mechanical-substitution leftovers in primary internal docs are grammatically broken or read as incoherent prose — a blocker per criterion #4 ("broken docs/prose from mechanical rewrites, especially internal_docs/typescript_go_architecture_transfer_*.md, internal_docs/architecture.md").

## Blocker findings

### 1. `internal_docs/architecture.md` — "future capability" substitution leaves ungrammatical / count-noun-broken prose

- `internal_docs/architecture.md:692` — `"This contract is split across three future capability:"` is grammatically broken (numeral `three` followed by singular `capability`). Original was almost certainly `"three work items"` / `"three milestones"` / `"three workstreams"`.
- `internal_docs/architecture.md:6` — `"future capability quality checks"` reads as a noun-stack rather than the intended "milestone/phase quality checks"; should be re-phrased ("per-workstream quality checks", "future-work quality checks", etc.).
- `internal_docs/architecture.md:48` — `"the implementation work and future capability breakdown for that semantic contract"` — `future capability breakdown` reads as broken; was likely `"milestone breakdown"`.
- `internal_docs/architecture.md:125` — `"Every future capability that implements built-in functions, data structure methods, or stdlib modules must include a **safety test layer**…"` — count-noun usage. Should be `"Every implementation of built-in functions…"` or `"Every workstream that ships built-in functions…"`.
- `internal_docs/architecture.md:144` — `"…each divergence, its rationale, and the future capability where it is introduced."` — count-noun. Suggest `"the workstream where it is introduced"` or `"the work item that introduced it"`.
- `internal_docs/architecture.md:296` — `"New crates added per future capability as needed:"` — count-noun. Suggest `"New crates added per workstream as needed"` or just `"New crates as the compiler grows:"`.
- `internal_docs/architecture.md:744` — `"…may be added in a later future capability for performance-critical paths."` — `"later future capability"` is redundant. Suggest `"may be added later for performance-critical paths"`.

Remediation: re-read the original `phase|milestone|wave` sense of each occurrence and replace with a grammatically appropriate noun (e.g. "workstream", "work item", "stage", "later release", or rewrite to drop the noun entirely). Do not leave `future capability` as a bare count noun.

### 2. `internal_docs/typescript_go_architecture_transfer_guardrails.md` — `source-provider guardrail` and `future capability` substitutions read as broken prose throughout

Representative breakages:
- `:3` `status: source-provider guardrail preflight gate` (was a milestone label, no longer parses as a status).
- `:19` `"source-provider guardrail locks the following terms before behavior migration starts:"` — `"source-provider guardrail"` reads as the subject of the sentence; previously something like `"M1"`.
- `:25, :27, :30` `"Not implemented in source-provider guardrail."` — `"in source-provider guardrail"` is incoherent (it's a doc, not a thing one implements features "in").
- `:34` `"…locked architecture terms for later future capability, not source-provider guardrail behavior."` — `"later future capability"` and the comma-after-noun structure are broken.
- `:39, :42, :65, :71, :75, :81, :92, :94, :115, :127` — every cell of the Permitted exceptions and the Disposition sections uses `source-provider guardrail` as a project-label noun, including `"## source-provider guardrail Disposition"`, `"source-provider guardrail introduced…"`, `"At the source-provider guardrail planning gate"`, and `"source-provider guardrail Disposition"`.
- `:81` `"…is source-provider guardrail .sifrbuildinfo/build-metadata territory, not source-provider guardrail source-provider correctness."` — duplicates the noun so the second clause is unintelligible.
- `:89, :110` `"…unless a later package-aware snapshot future capability promotes a specific read into package identity."` / `"…unless a later build-metadata future capability promotes…"` — `"snapshot future capability promotes"` and `"build-metadata future capability promotes"` are broken noun phrases.

Remediation: this document needs a focused rewrite pass to convert the `source-provider guardrail` label back into either (a) an explicit named workstream (e.g. `"the source-provider guardrail workstream"`) or (b) drop the label and refer to the artifact (`"this guardrail"`, `"this document"`). Replace every count-noun use of `future capability` (e.g. `"later capability work"`, `"a later workstream"`, or drop the noun).

### 3. `internal_docs/typescript_go_architecture_transfer_event_compaction_dirty_scope.md:32`

- `"The threshold is intentionally local to the LSP analysis workspace until later future capability adds first-class watcher registries and scheduler queues."` — `"later future capability"` is redundant; was likely `"a later milestone"`. Suggest `"…until a later workstream adds…"` or `"…until later work adds…"`.

### 4. `verification/areas/stdlib_parity/reports/text_i18n_dependency_decisions.md:13`

- `"`PluralRulesError`, future capability pass 4 `TranslationError`."` — every other reference in this report and in the surrounding stdlib_parity reports uses `"capability pass N"` directly; the inserted leading `"future "` here is a stray substitution. Suggest dropping `"future "` so it reads `"`PluralRulesError`, capability pass 4 `TranslationError`."` (consistent with `text_i18n_formatting_traceability.md`).

## Non-blocking nits (pre-existing technical debt, not introduced by this PR)

- `internal_docs/sifr_workspace_design.md:56` — `"Package directories are not implemented in this phase."` (blame: 2026-06-01, pre-cleanup). Same doc line 3: `"Source phase: …"`. Pre-dates this round; can be cleaned in a follow-up.
- `internal_docs/typescript_go_architecture_transfer_editor_corpus_snapshot_handles.md:29` — `"API shape without exposing a public compiler API in this phase."` Pre-existing prose.
- `lib/sifr/time.sifr:7` — `"# Stable timezone constants (no timezone mutation surface in this phase)."` Blame: 2026-03-20, pre-cleanup.
- The `verification/areas/stdlib_parity/reports/*` use of `"capability pass N"` everywhere is consistent and legible, but it's a stylistic substitution rather than restored vocabulary; up to you whether to revisit before merge.

## Checks confirmed clean

- No stale references to `platform_contract`, `binary_file_io_contract`, `validation_contracts`, `workspace_contracts`, `coverage_matrix_closeout`, `coverage_matrix_self_test`, `concurrency_runtime_inventory_closure`, `concurrency_runtime_closeout_traceability`, `world_class_verification_closeout`, `integer_dtype_contract`, or `validation_contract_support` anywhere outside `./plans`.
- `python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py` → `"verification taxonomy ok"`.
- `"contract"` mentions remaining in `verification/`, `crates/`, `docs/`, `internal_docs/` are legit API/language/diagnostic contracts (e.g. `check_formatter_rules.py`, `Contract record`, `diagnostic_rendering_harness.rs`, `vscode_extension_rules.json`). I did not find a remaining instance of `"contract"` used as a delivery bucket outside `./plans`.
- No new `phase\d+|milestone\d+|wave\d+|m\d+_` filename or label leaks outside `./plans`.

Recommend addressing findings 1, 2, 3, and 4 before considering this round satisfied.
