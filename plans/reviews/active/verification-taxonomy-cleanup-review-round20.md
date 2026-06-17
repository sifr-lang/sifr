## NOT SATISFIED

The round-20 rewrites introduced a large, systemic regression: a bulk text substitution injected named-capability phrases into other tokens (filenames, table cells, headings, prose), breaking links and producing word-soup sentences. The targeted greps from round 19 pass, but the new taxonomy checks didn't catch this class — and several artifacts are demonstrably wrong (the linked report files don't exist on disk).

### Blocker 1 — Broken file references in `concurrency_runtime_readiness_traceability.md`

Lines 31–37 cite reports whose names embed prose; none exist on disk. Real filenames are in `verification/areas/stdlib_parity/reports/`.

| Line | Broken link (cited) | Real file |
| --- | --- | --- |
| 31 | `concurrency_runtime_legacy-subprocess rejection capability_legacy_surface_traceability.md` | `concurrency_runtime_legacy_surface_traceability.md` |
| 32 | `concurrency_runtime_structured-task capability_traceability.md` | `concurrency_runtime_structured_tasks_traceability.md` |
| 33 | `concurrency_runtime_synchronization capability_sync_traceability.md` | `concurrency_runtime_sync_primitives_traceability.md` |
| 34 | `concurrency_runtime_blocking/offload capability_offload_traceability.md` | `concurrency_runtime_offload_traceability.md` |
| 35 | `concurrency_runtime_process-supervision capability_process_traceability.md` | `concurrency_runtime_process_traceability.md` |
| 36 | `concurrency_runtime_concurrency/runtime readiness_shutdown_traceability.md` | `concurrency_runtime_shutdown_traceability.md` |
| 37 | `concurrency_runtime_typed-IPC capability_typed_ipc_design.md` | `concurrency_runtime_typed_ipc_design.md` |

Requested fix: replace each cited path with the actual filename listed above.

### Blocker 2 — Capability column mislabeled with network-area terms

In the same table (`concurrency_runtime_readiness_traceability.md:32–36`) the Capability column reads `TCP`, `TLS`, `URL/HTTP primitives`, `HTTP transport`, `handoff` — these are async-network/network-HTTP domain labels, not concurrency-runtime capabilities. The Notes column on each row still describes the correct concurrency surface (structured tasks, synchronization, blocking/offload, process supervision, shutdown/diagnostics), so the labels were clobbered by a cross-domain bulk replace.

Requested fix: restore the concurrency capability names on those rows — `Structured tasks`, `Synchronization`, `Blocking/offload`, `Process supervision`, `Shutdown/diagnostics` (and verify row 31 is `Legacy-surface rejection` and row 37 is `Typed IPC`, both currently rendered with `capability` suffixes that read awkwardly but are at least domain-correct).

### Blocker 3 — Index-style section header still present

`concurrency_runtime_readiness_traceability.md:27`: `## capabilities 0 through 6 Readiness Inputs`. This is an indexed-bucket header (0 through 6) — exactly the class of wave-like leakage round 19 called out, with `capability` swapped in but the indexing kept.

Requested fix: rename to a named heading, e.g. `## Readiness Inputs By Capability`.

### Blocker 4 — Phrase-injected document title

`concurrency_runtime_process_traceability.md:1`: `# Concurrency Runtime process-supervision capability Process Traceability` reads as a sed replacement pasted into the title.

Requested fix: change to `# Concurrency Runtime Process Supervision Traceability` (or equivalent — pick one of the two phrasings and drop the duplication).

### Blocker 5 — Cross-token prose injections (regression class the new taxonomy missed)

The bulk replace welded capability names into surrounding prose. Each occurrence below is a regression vs. round 19's accepted naming.

- `concurrency_runtime_process_traceability.md:5` — `…No process-supervision capability subprocess lifecycle gaps remain…`
- `concurrency_runtime_process_traceability.md:9` — `| Surface | process-supervision capability evidence | Notes |`
- `concurrency_runtime_process_traceability.md:32` — `| CPython family | Sifr disposition | Representative process-supervision capability fixtures |`
- `concurrency_runtime_process_traceability.md:35` — `legacy-subprocess rejection capability legacy subprocess diagnostics; process-supervision capability exposes explicit output_shell_text…`
- `concurrency_runtime_process_traceability.md:51` — `…not pending process-supervision capability implementation gaps.`
- `concurrency_runtime_process_traceability.md:52` — `the process-supervision capability public text output path consumes text/i18n structured-task capability explicit encodings without locale defaults.` (note `text/i18n structured-task capability` — two unrelated domains welded together)
- `concurrency_runtime_offload_traceability.md:5` — `blocking/offload capability readiness merged in PR #2325 and capture-sendability readiness merged in PR #2329.`
- `concurrency_runtime_offload_traceability.md:9` — `| Surface | blocking/offload capability evidence | Notes |`
- `concurrency_runtime_offload_traceability.md:23` — `| CPython family | Sifr disposition | Representative blocking/offload capability fixtures |`
- `concurrency_runtime_offload_traceability.md:26` — `legacy-subprocess rejection capability legacy-surface diagnostics; production APIs are sifr.runtime, sifr.parallel, and JoinSet.`
- `concurrency_runtime_offload_traceability.md:42` — `…generated blocking/offload capability CPU/Rayon surfaces…`
- `concurrency_runtime_inventory_readiness.md:3` and :7 — `The capability validation-lane and inventory readiness audit…`
- `concurrency_runtime_inventory_readiness.md:52` — `…for the process-supervision capability text/process boundary.`
- `concurrency_runtime_inventory_readiness.md:54` — `blocking/offload capability-supported blocking/CPU offload rows…`
- `concurrency_runtime_typed_ipc_design.md:44` — `The first accepted transport is process-supervision capability-owned child process pipes.`
- `concurrency_runtime_typed_ipc_design.md:167` — `…process supervision returns typed process evidence from process-supervision capability.`

Requested fix: in each of the above, drop the welded `<surface> capability` modifier (it adds nothing — the surrounding sentence already names the surface). E.g. `process-supervision capability evidence` → `evidence`; `blocking/offload capability-supported blocking/CPU offload rows` → `supported blocking/CPU offload rows`; `text/i18n structured-task capability explicit encodings` → `text/i18n explicit encodings`. After the cleanup, re-read each touched paragraph end-to-end to confirm it parses as English.

### Blocker 6 — Taxonomy check gap

The new patterns from round 19 (`capability pass N`, `capability_<name>_N`, `Capability/Concrete/parity Backlog`) didn't catch any of blockers 1–5: phrase-laden filenames, capability-name-welded prose, indexed `capabilities 0 through 6` headings, or cross-domain label substitution. The taxonomy still considers the report tree clean.

Requested fix in `verification/areas/coverage_matrix/checks/verification_taxonomy.py` (plus a self-test fixture for each):
- A check that flags backticked filename tokens containing a space (e.g. `\`[a-z_][a-z0-9_/-]* [^`]*\.md\``) — this would catch all of blocker 1.
- A check that flags `<domain> capability <noun>` welds where `<noun>` is one of `evidence|fixtures|implementation|public|subprocess|readiness|supported|owned|text/process|legacy` — covers blocker 5.
- A check that flags `capabilities? \d+ (through|to) \d+` — covers blocker 3.
- A check that flags duplicate capability-name fragments in `#` headings (`process-supervision capability Process`, `Blocking/offload capability Offload`) — covers blocker 4.

### How to re-validate

After fixes, the following should also pass (in addition to the round-19 grep):

```
rg -n '`[a-z_][a-z0-9_/-]* [^`]+\.md`' verification --glob '!target/**'
rg -n 'capabilities [0-9]+ (through|to) [0-9]+' verification --glob '!target/**'
rg -n '[a-z]+(/[a-z]+)? capability (evidence|fixtures|readiness|implementation|public|subprocess|owned|supported|text)' verification --glob '!target/**'
```

and every backticked `.md` link inside `verification/areas/stdlib_parity/reports/concurrency_runtime_readiness_traceability.md` must resolve to a file in that directory.
