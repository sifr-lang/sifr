

## Review: Milestone Plan ReadinessI reviewed both planning documents against all five criteria.

---

### Criterion 1: Milestone Sequencing — PASS

M1 (manifests/contracts) → M2 (Ruff fork AST) → M3 (sifr_format core) → M4 (CLI/config) → M5 (analysis/LSP) → M6 (guardrails/corpus) → M7 (docs/closeout) is correct. Later milestones depend on executable contracts from earlier ones, not merely documented ones.

---

### Criterion 2: Scope, Outputs, Validation, and Review Gates — PASS

Each milestone has sufficient specificity to implement without rediscovering requirements:

| Milestone | Scope completeness | Output artifacts | Validation | Review gate |
|---|---|---|---|---|
| M1: manifests | machine-readable capability/CLI/AST coverage manifests, no new undecided rows | manifest triple + guardrail stubs | self-tests, diff-check, smallest local lane | external confirms no deferred decisions |
| M2: Ruff fork | all Sifr AST extensions, `own mut` canonicalization, docstring hooks, fork snapshots | fork PR, fixture corpus, submodule update | `cargo test -p ruff_python_formatter --lib`, idempotence, roundtrip | fork prints Sifr directly, no wrapper post-processing |
| M3: sifr_format core | route through Ruff, `PyFormatOptions` conversion, stable Sifr diagnostics | Ruff-backed core, Sifr wrapper tests | `cargo test -p sifr_format`, `check_formatter_contract.py` dual pass | one formatter core, no fallback whitespace path |
| M4: CLI/config | full locked CLI surface (write/check/diff/stdin/range/exclude/gitignore/cache/config/isolated), config loader | production `sifr fmt`, config tests, 30+ CLI fixtures | `cargo test -p sifr && -p sifr_format`, CLI fixture suite | locked manifests followed, Ruff crates reused |
| M5: analysis/LSP | same formatter path, LSP protocol adapter only, parity snapshots | analysis/LSP parity, editor fixtures | `cargo test -p sifr_analysis && -p sifr_lsp`, tooling parity | no split-brain formatter path |
| M6: guardrails | AST coverage guardrail, corpus checks, performance budgets, M1 wire-in | guardrail, corpus evidence, wiring | guardrail self-test, perf budgets, quick validation | future syntax extensions cannot land without coverage |
| M7: docs/closeout | internal + public docs, full validation, every matrix row confirmed | docs, execution tracker, closeout artifact | quick + full validation, target docs checks | production-grade, gap-free, release-ready |

---

### Criterion 3: Missing Capabilities, CLI/Config, Ruff Reuse, Guardrails, Docs, Validation — PASS

- **CLI details**: All flags and exit behaviors are locked in the CLI parity manifest (30+ rows). Output summaries, diff stream destination, abnormal-error exit status are explicitly called out for Milestone 4 fixture coverage.
- **Config details**: Canonical `sifr.toml` schema, precedence (CLI → sifr.toml → Ruff migration → defaults), `extend` with cycle detection, explicit-target behavior, unknown-key/unsupported-option diagnostics are locked.
- **Ruff reuse**: Minimum reusable Ruff APIs listed, CLI subprocess prohibited, library-only path required, diff/stdin owned by Sifr wrapper.
- **Guardrails**: `check_formatter_ast_coverage.py` design is fully specified (6 requirements). `check_formatter_contract.py` dual-pass is already in the codebase and is referenced for M3 validation.
- **Docs**: Internal architecture and public formatter docs are explicitly scoped to M7.
- **Validation**: Phase closure validation list covers all relevant crates and tooling, with a fork-level command entry point if the fork adds formatter ecosystem tooling.

---

### Criterion 4: Remaining "Parts" or Deferred Decisions — PASS- No `Part 1`/`Part 2` language remains in either document.
- All "future phase" and "later phase" references are either governance language ("must not defer without a new reviewed planning update") or explicit non-applicability rationales with future phase hooks — confirmed clean by pass-6.
- One legitimate Phase 36 reference at execution.md:28 is a required implementation anchor: the plan explicitly states what Milestone 3 replaces and why, which is necessary context, not deferred work.

---

### Criterion 5: Overall Implementation Readiness — PASS

**The milestone plan is implementation-ready with no gaps.**

The phase doc:
- Uses milestones throughout, correctly sequenced.
- Locks every capability classification, CLI surface, config schema, Ruff integration point, guardrail requirement, and validation responsibility.
- Provides explicit entry/exit criteria, output artifacts, validation commands, and review gates for every milestone.
- Contains no deferred implementation decisions, no unclassified capabilities, and no references that could cause implementers to rediscover requirements.
