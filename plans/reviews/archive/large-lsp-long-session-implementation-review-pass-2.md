# Large LSP Long-Session Verification — Implementation Review (Pass 2)

## **NOT SATISFIED**

The subrepo + submodule + verifier scaffolding is correctly assembled, but the
implementation has (a) a real regression in the LSP analysis path that the new
"fallback" code only partially fixes, (b) a gate wiring that silently no-ops on
a fresh clone, and (c) a long-session verifier whose configured workload doesn't
actually stress the architecture features the corpus exists to exercise.
Detailed blockers and concrete fixes below. Each blocker is reproducible from
the current branch.

---

### Blocker 1 — Standalone fallback is destroyed by the next `refresh_projects` (regression)

`LspAnalysisWorkspace::open_document` now falls back to inserting a standalone
`LspDocumentAnalysis` in `self.documents` when `project.open_document(...)`
fails (`crates/sifr_lsp/src/analysis_workspace.rs:45-65`). Good. But
`refresh_projects` is unchanged:

```rust
for document in documents.documents() {
    if let Some(root) = workspace_root_for(document.path()) {
        self.documents.remove(document.uri());        // <- clobbers fallback
        grouped.entry(root).or_default().push(document);
    }
}
```

`crates/sifr_lsp/src/analysis_workspace.rs:101-108`. Any subsequent
`refresh_projects` call removes the standalone fallback for every workspace
file and then re-creates the project. The re-created project's `files_by_uri`
still won't contain the orphan (still not in the import graph), so
`with_document` falls all the way through to "analysis is unavailable" and
every subsequent hover/completion/symbol query fails.

`refresh_projects` is unconditionally called from `Session::close_document`
(`crates/sifr_lsp/src/session.rs:130-136`) and from any new-root open, so this
is reachable from normal editor use. I verified it by adding the temp test
below (then reverted):

```rust
// open `main.sifr` and an `orphan.sifr` (not imported by main, same package),
// query orphan → OK (standalone fallback). Then close main → query orphan:
// LspError { code: InternalError,
//   message: "analysis is unavailable for .../src/orphan.sifr" }
```

Output of `cargo test -q -p sifr_lsp regression_secondary_fallback_lost_after_refresh_projects`:

```
panicked at crates/sifr_lsp/src/session.rs:596:14:
orphan should still have analysis after close-triggered refresh:
LspError { code: InternalError,
  message: "analysis is unavailable for /var/.../orphan.sifr" }
```

**Fix:** make `refresh_projects` honor the fallback. Two options:
- After `LspProjectAnalysis::open` returns, re-insert standalone entries in
  `self.documents` for any `open_uris` not present in `files_by_uri`.
- Or stop removing from `self.documents` in `refresh_projects` and let the
  project path's `files_by_uri.contains_key(uri)` arbitrate per call.

Also add the failing test above as a permanent regression test. The currently
landed test `project_secondary_open_falls_back_to_document_analysis_when_unmapped`
is **misnamed**: its `helper.sifr` is imported by `main.sifr` (`from helper
import helper`), so the file *is* in the project's source map and the fallback
path is never taken. The test passes whether the new fallback code exists or
not — it cannot detect either the fix or its regression. Either rename it to
reflect what it actually covers (project-managed secondary open) or rewrite it
with an actually-unmapped file plus the close-then-query sequence above.

---

### Blocker 2 — Smoke gate silently SKIPs on a fresh clone

`scripts/run_all_tests.sh:179-180` wires:

```sh
python3 .../lsp_large_session.py --self-test
python3 .../lsp_large_session.py --mode smoke
```

without `--require-submodule`. The verifier's `run_large_session` treats a
missing manifest as a *skip*, not a failure, unless `--require-submodule` is
set (`verification/tooling/lsp_large_session.py:151-161`):

```python
except LspProtocolError as error:
    if require_submodule:
        print(..., file=sys.stderr); return 1
    print(f"LSP large session: SKIP: {error}"); return 0
```

`scripts/run_all_tests.sh` does **not** invoke `scripts/clone_subrepos.sh` nor
`git submodule update --init`, so on a clean checkout (the most common CI
shape) the gate prints SKIP and exits 0. The acceptance criterion says smoke
"runs in the quick validation lane" — a silent skip doesn't satisfy that. This
is the same gap the design review's blocker 3 flagged; it's still open.

**Fix:** pass `--require-submodule` in the gate invocation, *and* either have
`run_all_tests.sh` shell out to `git submodule update --init verification/sifr-large-lsp-verification`
before the LSP block, or document in the script's prologue that
`clone_subrepos.sh` is required first. Pick one; don't ship the current
"works on my machine" shape.

---

### Blocker 3 — Verifier doesn't stress the architecture features its corpus was sized for

The corpus has 1206 files, 3 packages, depth-380 chains and 20-way API fanout
(`verification/sifr-large-lsp-verification/manifest.json` `shape`). The
verifier configuration neutralizes most of what that costs:

- `diagnostics_mode="off"` for both `smoke` and `full`
  (`verification/tooling/lsp_large_session.py:48,57`). With push diagnostics
  disabled and no `workspace/diagnostic` polling, the session never schedules
  diagnostic jobs, so M5 stale-version rejection, M11 priority lanes, M13
  cancellation/watchdog, and M14 bucketed indexes are not meaningfully driven.
  The README says "diagnostic publication remains covered by existing LSP
  protocol smoke and stress checks" — but those use 1–3 file fixtures, which
  is precisely what motivated this corpus in the first place. With diagnostics
  off the verifier degenerates to "client-side request latency on top of an
  effectively-idle server".
- `edit_text` produces an identical-shape change for every category — it just
  appends a `# lsp-large-session {category} {round_index}` comment line. The
  `private-body` / `shared-api` / `storm` labels exist in the evidence file
  but the LSP sees the same trivial trailing-comment edit each time. So:
  - "shared-api" edits never change a `def` signature, never invalidate
    reverse-dep closure — M7 slow path uncovered.
  - "storm" edits don't burst within the M6 compaction window — they're spaced
    one per round behind a synchronous query mix and a synchronous response.
    `WatcherStorm` degradation isn't reached.
  - "private-body" edits are the same as the others, so even the "fast" path
    isn't distinguishable from the "slow" path in the report. The design
    review explicitly called this out (item 14); the implementation didn't
    address it.
- `LspClient` is serialized request/response (`verification/tooling/lsp_protocol.py:47-54`),
  so M11 priority lanes never see more than one queued request at a time.
  This was a known limitation the design review allowed *if* the scheduler
  coverage claim was dropped. The doc and issue text don't make that drop
  explicit; either drop the claim or pipeline the client.

**Fix (minimum):** turn diagnostics on for `--mode full` (either
`diagnosticsMode=push` or call `workspace/diagnostic` after each round), and
change `edit_text` so `shared-api` actually mutates an exported function
signature (e.g., rename the parameter or change the return type) and `storm`
emits 10+ `didChange` notifications in tight succession before the next
request. Otherwise the long-session verifier doesn't measure long-session
behavior; it measures cold project load + a trivial edit loop.

---

### Important issues (should be resolved before merge)

4. **Thresholds are 30–60× looser than observed and can't catch the leaks
   they exist to catch.** Recorded peak RSS 17 MiB vs 1024 MiB threshold,
   p95 8.3 ms vs 10000 ms, slope 2.5 MiB/min vs 96 MiB/min. The slope
   threshold means a real leak has to add 96 MiB *within a one-minute window*
   over the second half of a session whose total RSS is currently 17 MiB.
   That's not a leak detector; that's an OOM detector. Either tighten to
   ~3–5× observed with explicit headroom rationale in the comment, or wire
   peak/p95/slope into `verification/performance/budgets.json` under a
   `perf.lsp.long_session.*` family and let `check_budgets.py`'s retry logic
   handle drift (the existing path the design review recommended).

5. **Corpus drift checker is not wired into the main repo gate.** The subrepo
   ships `tools/generate_corpus.py check`, but nothing in
   `scripts/run_all_tests.sh` invokes it. If the submodule's files diverge
   from the generator without the subrepo's own CI catching it (or if the
   manifest is hand-edited), the main repo silently runs against off-spec
   data. The verifier also reads `manifest["corpus_sha256"]` into the report
   but never recomputes it from the corpus on disk — trivial to add and would
   give the gate a local drift signal independent of the subrepo's CI.

6. **`paths_match` in `crates/sifr_analysis/src/host/overlay_updates.rs:80-88`
   is undocumented and weak.** The helper accepts any absolute `requested`
   whose path components end with the relative `candidate`. There's no test,
   no comment explaining when source map paths are relative vs. absolute, and
   no assertion that the source map can't hold two suffix-equivalent entries.
   The test added in this PR uses absolute temp-dir paths, so it doesn't
   exercise the new helper at all. At minimum: add a doc comment stating the
   invariant (one host per project ⇒ no duplicate suffix in source map),
   and add a direct unit test in `overlay_updates.rs` covering both branches.

7. **Full mode is never executed by any lane, schedule, or doc-mandated
   workflow.** The acceptance criterion says "Full mode is documented and
   writes JSON evidence" — that's literally satisfied, but the artifact is
   useless without a cadence. Either schedule it nightly (the natural home),
   or rename the doc section to "manual qualification" and explain that any
   regression detection is on the developer to run.

8. **`verification/sifr_large_lsp_verification.md` is documentation but is
   currently untracked in git.** Same for `issues/ad-hoc-large-lsp-long-session-verification.md`
   and `reviews/typescript-go-large-lsp-long-session-design-review-pass-1.md`.
   Presumably they'll be `git add`ed before commit; flagging in case the
   commit-bundling step is the actual delivery boundary.

---

### Minor

- The verifier's `RssSampler` exits via `_stop.set()` then `_thread.join(timeout=2)`.
  Between `_stop.set()` and the last `wait`, the sampler can fork `ps` once
  more; harmless, just noting that the last sample's `elapsed_ms` may overrun
  the session by up to one polling interval. The slope math truncates to the
  second half so it's not a correctness issue.
- macOS `ps -o rss=` rounds to whole pages and can lag a few hundred ms;
  recording the sampler's monotonic timestamp instead of `ps`'s is correct
  (already done at line 88).
- Naming: the file `verification/sifr_large_lsp_verification.md` collides
  visually with the submodule directory `verification/sifr-large-lsp-verification/`.
  Consider `verification/lsp_large_session.md` or
  `verification/sifr-large-lsp-verification.md` (matching the dir).

---

### Verdict

**NOT SATISFIED.** Blockers 1, 2, and 3 each independently keep this from
being mergeable:

- 1 is a behavioral regression with a reproducer in this review; the test
  bundled with the PR cannot detect it.
- 2 makes the new gate a no-op on a clean clone, which is the only state
  CI ever starts from.
- 3 means the 1200-file corpus and the long-session machinery don't actually
  exercise the LSP scheduler / invalidation / storm-compaction paths the
  corpus was sized to stress.

Recommended sequence to clear: (a) extend `refresh_projects` to preserve the
standalone fallback and add the failing test from blocker 1 as a permanent
regression; (b) flip the gate to `--require-submodule` and wire
`git submodule update --init` (or call out the prereq script); (c) make
`shared-api` and `storm` edits behaviorally distinct and turn diagnostics on
in `--mode full`; (d) tighten thresholds or migrate to `budgets.json`;
(e) wire the drift checker into the main-repo gate. With those, the next
review pass can be SATISFIED.
