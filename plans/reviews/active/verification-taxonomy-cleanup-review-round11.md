I have enough evidence. Let me compile the review.

## Blocking Issues

**1. Gate has false negatives that let the cleanup's own substitutions through.** All of the strings below are produced by this PR and still ride sequentially-numbered delivery work, but the current `verification_taxonomy.py` patterns let them pass (verified by replaying the regex set against each string):

| sample (sourced from the diff) | passes gate? |
| --- | --- |
| `# Reference: m_adhoc_5` | yes (regex wants digits-after-`m`) |
| `# Reference: m_driver_4` | yes |
| `Sifr native task surface Demo: Core Language` | yes |
| `archived compiler/codebase contract record` | yes (hyphenated/spaced "contract" is unguarded; only `contract_*`, `contract\s+slice`, `Contract\s+\d+` are caught) |
| `Phase ad-hoc test strategy process/runtime surface` | yes (no digit after `Phase`) |
| `TypeScript-Go architecture transfer synchronization surface` / `... blocking/offload surface` / `... process/runtime surface` / `... task-context and shutdown surface` / `... typed IPC surface` | yes |
| `native task surface+` (used as ordinal) | yes |

**2. `internal_docs/architecture.md:379` — broken sentence, leftover from "milestone 4" stripping.** Pre-diff text was `Phase ad-hoc test strategy milestone 4 moved …`; current text is `Phase ad-hoc test strategy process/runtime surface moved …`. Reads as a delivery-phase header that lost its number, and "process/runtime surface" is the new disguise label (see #4).

**3. `internal_docs/architecture.md:300–317` — TypeScript-Go transfer milestones renamed to "X surface" but the sequencing is unchanged.** Each `TypeScript-Go architecture transfer <X> surface` line corresponds 1:1 to an `M\d+` in the archived ad-hoc plan (`plans/reviews/archive/typescript-go-m1-...`, `…-m2-source-provider-overlay-…`, `…-m5-lsp-persistent-session-…`, etc.). The bullets even retain "before X" phrasing that only makes sense as ordinal sequencing ("before `WorkspaceSession` owns overlay lifecycle in blocking/offload surface", "before persistent LSP sessions … land in task-context and shutdown surface", "before snapshot reuse introduces cache reuse"). This is the exact smell the user flagged — "X surface" is the new "wave". `internal_docs/architecture.md:942` reinforces it with `(native task surface+, builtin lowering in synchronization surface)` as a literal ordinal.

**4. `verification/areas/stdlib_parity/reports/concurrency_runtime_structured_tasks_traceability.md` — title and every "M1 evidence" / "M1 fixtures" / "for M5" was rewritten to `native task surface` / `task-context and shutdown surface`.** Diff shows `# Concurrency Runtime M1 Traceability` → `# Concurrency Runtime native task surface Traceability`, `Milestone: milestone_concurrency_runtime_1` → `Surface record: concurrency_runtime_structured_tasks`, `Reserves ctx shape for M5` → `Reserves ctx shape for task-context and shutdown surface`. Same pattern propagates into `text_i18n_*.md`, `network_http_*.md`, `stdlib_bytes_cpython_1_traceability.md`, `stdlib_iterator_cpython_1_traceability.md`, `stdlib_rng_cpython_1_traceability.md`, `hyper_util_necessity.md`, etc. — 82 occurrences of `native task surface` across `verification/` and `demos/` and they read as delivery-label replacements (e.g., `Classified waivers carried from native task surface`, `## Local fixture anchors (native task surface)`, `covered (legacy + native task surface delegation replay)`).

**5. 219 demos rewrite `# Source issue: <real-issue>-execution.md` to `# Source issue: archived compiler/codebase contract record`.** The user explicitly said "contract" as a replacement for "wave" has the same smell — this is the same swap done 219 times. The phrase carries no compiler/codebase content and only exists to occupy the slot a delivery reference used to live in.

**6. 17 demos still carry literal delivery identifiers in `m_<word>_<digit>` form** (the gate's regexes require `m\d+`, not `m_X_\d+`):
- `demos/rooted_entrypoint/{main,helper,shared}.sifr:1` — `# Reference: m_adhoc_1`
- `demos/compiler_api/main.sifr:1` — `# Reference: m_driver_1`
- `demos/cargo_manifest/{main,helper}.sifr:1` — `m_adhoc_3`
- `demos/project_build/{main,helper,formatter}.sifr:1` — `m_driver_4`
- `demos/dependency_manifest/{main,helper}.sifr:1` — `m_adhoc_5`
- `demos/stdlib_loading/main.sifr:1` — `m_driver_2`
- `demos/test_runner_imports/{test_imports,helper}.sifr:1` — `m_driver_5`
- `demos/project_graph/{main,provider,consumer}.sifr:1` — `m_driver_3`

**7. Demo runtime strings now contain disguised delivery labels (also reach `emitted.rs`/`idiomatic.rs`, so they ship in golden output):**
- `demos/core_language/main.sifr:4` — was `Sifr M1 Demo: Core Language`, now `Sifr native task surface Demo: Core Language`
- `demos/compiler_api/{main.sifr,idiomatic.rs,emitted.rs}` — was `driver milestone 1 api spine demo: 42`, now `driver native task surface api spine demo: 42`
- `demos/rooted_entrypoint/{shared.sifr,idiomatic.rs}` — was `adhoc milestone 1 rooted entrypoint demo: pass`, now `adhoc native task surface rooted entrypoint demo: pass`

**8. Mechanical substitution produced doubled-token strings — accidental demo-output damage.** Example: `demos/statistics/main.sifr` originally printed `"m30_1b statistics parity demo: pass"`; after `m30_1b` → `statistics` it now prints `"statistics statistics parity demo: pass"`. The same demo's header is also now triplet-redundant (`# Reference: statistics` / `# Source issue: archived compiler/codebase contract record` / `# statistics statistics parity demo`). Worth sweeping for other `<word> <word>` repeats from the same substitution pass.

**Net effect:** the cleanup moved the delivery taxonomy off `wave`/`milestone`/`Mn` onto a new vocabulary — `native task surface`, `<X> surface` per ex-milestone, `archived compiler/codebase contract record`, and ungated `m_<word>_<digit>` references — and the gate's pattern set is unaware of any of these substitutions. The user's stated rule ("contract as a replacement for wave has the same smell") applies symmetrically to all of these new labels.

Suggested gate additions (won't fix wording, but will stop the false negatives from recurring): `\bm_[a-z][a-z0-9_]*_\d+\b`, `\b(?:native task|process/runtime|task-context and shutdown|blocking/offload|synchronization|typed\s+IPC|cache-key identity|production runtime audit|flow-graph|trace and status|editor corpus and snapshot handle|bucketed index|project residency|LSP (?:scheduler|latency budget|cancellation[, ]+progress[, ]+and watchdog)|snapshot reuse)\s+surface\b`, and an unhyphenated/spaced "contract" guard like `\barchived\s+[a-z/]*\s*contract\s+record\b`.

Reviewer not satisfied — blockers above need resolution.
