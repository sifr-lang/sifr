I have enough grounding now to deliver the critique.

## **NOT SATISFIED — proceed only after resolving the blockers below**

The proposal is on the right track and the user is correct that the current verification stack has no signal for *long-session, large-codebase* LSP behavior — `lsp_protocol_smoke/stress.py` use ad-hoc temp dirs with 1–3 files, `lsp_workspace_perf` has 5 trivial files, and `editor_query_corpus/multi_file` is a 2-file marker fixture (M17). Existing perf RSS is sampled by `/usr/bin/time -l/-v` at *process exit* (`verification/performance/run_benchmarks.py:430-477`), which by construction cannot detect monotonic growth or leaks inside a long-lived LSP. So the gap is real and the architectural shape (submodule corpus + main-repo driver script + sampled RSS) is sound.

But several concrete blockers and gaps need to be settled before implementation.

---

### Blockers (must resolve)

1. **Phase framing is wrong.** `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:1-31` and `reviews/typescript-go-architecture-transfer-full-implementation-review-pass-1.md:9` already say the TS-Go transfer is **complete and SATISFIED on `main`** as of 2026-06-02. Calling this a "next phase of TypeScript-Go architecture transfer audit work" misrepresents the state. It is a **post-phase verification hardening task** (peer to Phase 29 hardening or a perf-budget follow-up), and should be tracked as such — either as an entry under `internal_docs/phases/29_verification_hardening.md` or a new ad-hoc issue. Otherwise the phase tracker reopens and the M0–M17 closeout narrative becomes incoherent.

2. **Reproducibility model is under-specified.** "Deterministic generator plus committed corpus" creates two sources of truth that can drift. Pick one of these and write it down before coding:
   - **Generator-authoritative**: commit only the generator + a content hash manifest; regenerate to a temp dir each run; *no* corpus files in the subrepo. Pro: zero drift. Con: every run pays generation cost; bisecting requires reproducing input.
   - **Corpus-authoritative + drift checker**: commit the corpus + the generator + a `check_corpus_matches_generator.py` contract (mirroring `check_ruff_fork_update_contract.py`). The submodule SHA pin is the real reproducibility anchor; the generator is documentation. Pro: cheap reruns. Con: drift checker must be wired into the gate.
   The proposal as written has both files and a generator with no enforcement of the relation — that will rot within two PRs.

3. **Submodule lifecycle is not wired.** The proposal adds the submodule but doesn't address: (a) `git submodule update --init --recursive` in `scripts/run_all_tests.sh` / `run_e2e_pass.sh` so a fresh clone works, (b) shallow-clone strategy (the existing `verification/package_management/demo_repositories/sifr-demo-*` submodules in `.gitmodules:13-32` pin to a branch — pin to a SHA instead for true determinism), (c) CI clone cost (1200–2000 small files is fine, but document expected size), (d) what happens when the submodule is absent in offline runs (skip with reason, not fail). Without this, the test will silently no-op on clean clones.

4. **`LspClient` cannot run a long session as-is.** The existing client (`verification/tooling/lsp_protocol.py:23-145`) has two structural limits that the proposal glosses over:
   - **Single-in-flight serialization**: `request()` waits on `_wait_for_response(request_id)` before any other request can be sent (line 47-54). The proposed "hundreds of didChange edits with concurrent requests" cannot exercise the M11 priority lanes (`LatencySensitive`, `Formatting`, `Workspace`, `Background`) — the scheduler will only ever see one queued request at a time. To meaningfully test the scheduler, either extend the client with a pipelined send + correlated receive loop, or accept upfront that this test only covers serialized request/notification interleaving and remove the scheduler-coverage claim.
   - **`_read_message` reads stdout one byte at a time** (line 131) for the Content-Length header. Fine for ~50 messages, painful for ~10k. Switch to a buffered reader, or budget for this.

5. **`--parent-pid` is required, not optional.** M13 (`internal_docs/typescript_go_architecture_transfer_m13_lsp_cancellation_progress_watchdog.md:29-33`) wires `sifr lsp --stdio --parent-pid` and `lsp_protocol_stress.py:26` uses it. A long-running session that doesn't pass `--parent-pid` will not exercise the watchdog and risks orphaning a `sifr lsp` if the harness crashes. Bake it in.

6. **RSS sampling via `ps` is fine on macOS+Linux, but the design must address three known gotchas:**
   - **Sample cadence vs. cost**: shelling out to `ps` every 100ms over a 60s session = 600 forks. Use a single long-lived poller thread; period ≥250ms; record (t, rss) tuples plus high-water mark.
   - **`ps -o rss=` unit**: KB on both macOS and Linux — document explicitly. (`/usr/bin/time -l` on macOS returns *bytes* since Mavericks; the existing parser at `run_benchmarks.py:430-442` assumes bytes on Mac and KB on Linux — replicate that convention or factor it out.)
   - **What "reasonable" means**: the proposal mentions `max RSS` and `max p95 latency` thresholds but with no baseline. Either pre-run on a reference machine and commit budgets to `verification/performance/budgets.json` under a new `perf.lsp.long_session.*` family (preferred — reuses `check_budgets.py` retry logic at `scripts/run_all_tests.sh:217-231`), or commit them in the driver script and accept that they aren't part of the unified perf gate. Don't invent a third budget system.

7. **Stale-rejection accounting.** M5 (snapshot+version stale rejection) and M13 (cancellation) mean that under hundreds of overlapping edits **some requests will legitimately be dropped/cancelled** — the LSP rejects them as stale by design (`crates/sifr_lsp/src/session.rs:602-640` per the review pass). The harness must classify those as expected, not as failures, and assert the *rate* is bounded. Otherwise the test will be flaky from the start. Define the success criterion as "every request either returns within budget or is cancelled with the documented code; no request hangs past the watchdog timeout."

---

### Important recommended changes

8. **Validation cost / profile placement.** Quick profile is currently ~263–330s (review pass 1, lines 24–93 of the issue tracker). Adding more to quick must be ≤ ~30s incremental. Concretely: wire a **smoke mode** (≈10–20s: open 5 files, ~50 edits, no workspace/diagnostic walk) into `--profile quick` and a **full mode** (1–3 min) into `--profile pr`/`nightly`. The full mode is where the 1200–2000-file corpus matters; the smoke mode can use a 20–30-file subset of the same corpus to keep the codepath identical.

9. **Corpus shape is under-constrained.** "Roughly 1200–2000 .sifr files" needs explicit per-shape budgets so the corpus is meaningful: (a) max import depth (e.g., 12 levels) — exercises M7 transitive reverse-dep closure; (b) fan-out factor (e.g., one "interface" module imported by 50 leaves) — exercises M10 one-module replacement and M7 signature invalidation; (c) package boundary count (≥3 separate `sifr.toml` package roots) — exercises M14 Workspace/Package/Stdlib bucket separation and M2 multi-project workspace symbol behavior already covered in `lsp_protocol_stress.py:185-223`. Without these constraints, the generator could produce 2000 disconnected files that don't stress what the architecture transfer actually built.

10. **Use `sifr.toml` conventions from the existing fixtures.** `verification/performance/query_projects/lsp_workspace/sifr.toml:1-4` uses `edition = "2026"`, `sifr-version = ">=0.3,<0.4"`, kebab-case package names. Generator must match, or `sifr_package` will reject the corpus.

11. **JSON evidence schema must match existing patterns** so it composes with the lane report at `scripts/run_all_tests.sh:72-76`. Look at `verification/performance/check_budgets.py` for the expected `metrics` shape (median_ms / p95_ms / mad_ms / coefficient_variation / peak_rss_bytes) and emit a superset, not a parallel shape. Otherwise the artifact will live outside the validation-lane report system.

12. **`SIFR_LSP_COMMAND` env var.** The existing client (`lsp_protocol.py:25-33`) lets you override the binary via env. The new script should honor it identically so reviewers can point at `target/release/sifr` for deliberate perf runs without editing code.

13. **Manifest schema for the subrepo.** "Expected symbols" is vague. Make it: `{ "version": 1, "generator_sha256": "…", "entrypoints": ["pkg_a/main.sifr"], "modules": [{"path": "...", "exports": ["..."], "imports": ["..."]}], "shape": {"depth": 12, "fanout_max": 50, "package_count": 3}}`. The driver asserts the LSP returns ≥ N of the declared exports from `workspace/symbol`; without that the test only proves the server didn't crash, not that it produced correct results.

14. **One concrete edit strategy.** "Hundreds of didChange edits" needs to specify the mix, because the M6 dirty-scope/event-compaction code paths differ:
    - leaf edits (private-body, reverse-dep closure stays empty — M5/M7 fast path)
    - intermediate edits that change a `def` signature (forces reverse-dep invalidation — M7 slow path)
    - storm bursts (10+ edits within compaction window — exercises `WatcherStorm` degradation per `crates/sifr_frontend/src/workspace_session.rs:339-358`)
    Tag each edit with its category; assert distributional balance in the evidence file.

15. **Don't claim to "verify memory is reasonable" without a leak test.** A long session test is mostly valuable if it can fail on **monotonic growth**. Add an explicit assertion: linear-regression slope of (sample_time → RSS) over the second half of the session must be ≤ N MB/min (tuned to noise floor), in addition to the peak-RSS budget. Peak alone won't catch a slow leak.

16. **Subrepo public visibility implication.** The corpus will be public on `github.com/sifr-lang/sifr-large-lsp-verification`. That's fine for generated synthetic Sifr code, but the README must say "this is synthetic perf scaffolding, not real user code" so external readers don't take it as idiomatic Sifr.

---

### Minor

- The empty file `reviews/typescript-go-large-lsp-long-session-design-review-pass-1.md` is already created in the working tree (1 line, no content). If this critique is meant to be that artifact, save it there.
- `verification/tooling/check_lsp_split_brain.py` is already part of the quick gate (`scripts/run_all_tests.sh:149-150`) — the new script's name should not clash; `lsp_large_session.py` works.
- macOS `ps` rounds RSS to whole pages and can lag a few hundred ms — record the sample timestamp from the harness's monotonic clock, not `ps`'s output.

---

### Recommendation

Address blockers 1, 2, 3, 5, 6, 7 before writing any code. Blocker 4 can be deferred only if you're willing to drop the scheduler-lane-coverage claim. Then proceed in three reviewable PRs: (a) create + populate the subrepo and submodule wire-up with a no-op driver, (b) implement the driver with smoke mode and JSON evidence (gated only by self-test), (c) commit budgets to `verification/performance/budgets.json` and flip on the gate.

Want me to draft the subrepo manifest schema and corpus-shape spec (item 9 + 13) next, or open the issue/phase entry to reframe this as a hardening task (blocker 1)?
