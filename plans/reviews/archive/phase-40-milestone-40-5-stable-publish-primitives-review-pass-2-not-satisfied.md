## Review — Phase 40 / milestone 40.5 stable publish-primitives, pass 2

Scope: 17 modified + 8 untracked files, `HEAD = da7c38fb15dbebe11b1e9be943f4d080b8e7bafc`. No files modified.

### Pass-1 closures verified

**1. Docs truthfulness — closed.** `internal_docs/distribution_pipeline.md:607-617` now says "The production workflow does not yet invoke this command; it must supply those fresh inputs before stable mutation is enabled," and `plans/releases/README.md:20-27` says "Wiring that command into the protected publish job is the next publication wave." Both match reality: `revalidate_stable_publication.py` has no caller outside its selftest, performs no download, and creates no checkouts.

**2. Test load-bearingness — partially closed** (see finding 1). I mutated every enumerated gate in-memory and re-ran the suite. Caught: summary byte-inequality (`revalidate:91`), burned-generation (`:71-78`), live-snapshot equality (`generation.py:53`), name/payload disagreement (`:43`), invalid snapshot name (`:36`), transported-hash (`fetch:122`), artifact expiry (`:93`), repository identity (`:46`), run conclusion (`:64`), run attempt (`:62`), pre-existing output (`:50`).

**3. Preview alpha/beta guard — closed.** `release-publication-prepare.yml:120-126`; precedence of `A && B || { … }` is correct and the arm is exercised by the contract case. The sole caller (`release-publication.yml:16-19,67`) never reaches the stable arm, so relaxing `channel` to `required: false` is not a live bypass.

**4. Governed unreadable-summary diagnostic + single read/hash/parse — closed.** `revalidate:46-58` reads once, `sha256_bytes(summary_bytes)`, then `load_json_bytes_strict` on the same bytes. Covered at `stable_publication_primitives_selftest.py:313-328`.

**Hardening claims verified.** `generation.py:32` hoists the live digest out of the loop. All three new scripts import `verification.areas.distribution_release.governance.*`, and `runner.py:331` qualifies the selftest module to the same package — I confirmed there is now a single `GovernanceError` identity, so `_expect_governance_error` is genuinely load-bearing. Pagination: I verified against the real API that `gh api --paginate --slurp` emits an array-of-arrays, so `jq '.[][] | .name'` (`:363-365`) is correct; if enumeration fails the loop yields nothing and `allocate_next_generation` fails closed. Streaming download bounds writes, kills on overrun, and unlinks on every failure path (`fetch:152-184`). Path traversal via `workflow_artifact_name` is closed upstream (`artifact_index.py:167-171,227`). Live-snapshot equality is production-satisfiable and `release-publication.yml:602-623` is the only `channels.json` mutator, uploading the snapshot before the `--clobber`. Workflow keeps `actions: read`/`contents: read`, no `secrets:`, `persist-credentials: false`, and derives both `candidate_version` and `source_commit` from the digest-verified plan (`:287-301`). The `.source.commit // .source_commit` output binding (`:424`) is not a dead fallback — it correctly spans the stable and preview summary shapes.

**Validation reproduced.** stable-publish-primitives 3/3; stable-prepare 6/6; governance 14/14; prepare workflow contract exit 0; `full` 61 variants and `full + stable-publish-primitives` also 61 (dedup works); coverage_matrix 5/5; file-size guardrail PASS (limit 900, largest new file 485 lines); HIR guardrails PASS. No `crates/`, demo, or Rust-interop changes.

---

### Findings

**1. LOW — the protected-input gate is still unexercised, and the ledger claims otherwise.**
`plans/issues/active/phase-40-stable-channel-ga-execution.md:593-595` states remediation "pins summary byte inequality, **protected-input drift**, …". It does not. I replaced the entire `if` at `scripts/distribution/revalidate_stable_publication.py:59-69` with `if False:` and the suite stayed green 3/3. The `mode="resume"` case at `stable_publication_primitives_selftest.py:276-291` is actually caught by the byte-equality gate at `:91`, because `mode` is fed to `materialize_stable_prepare` and reappears in the recomputed summary.

This is not a correctness hole — every field the gate compares (`operation`, `mode`, `evidence_commit`, `candidate_path`, `expected_plan_sha256`) flows into `materialize_stable_prepare`, so the gate is unfalsifiable by any black-box test and the path fails closed either way. But pass-1's finding-2 sub-item is not closed and the durable ledger now asserts a coverage property that does not hold. Fix: correct the ledger sentence to say the gate is an early defense-in-depth check subsumed by byte equality (or drop the gate).

**2. LOW — the wave's headline download gate, `written != expected_bytes`, has zero coverage.**
`scripts/distribution/fetch_qualification_artifacts.py:180-184` (and the overrun branch at `:164-168`) is the "must equal the authoritative API `size_in_bytes`" requirement. Both mutations survive: `if written != expected_bytes:` → `if False:` and `if written > expected_bytes:` → `if False:` leave the suite green. The fake API at `stable_publication_primitives_selftest.py:409` sets `size_in_bytes` from `archive_path.stat().st_size`, so declared and delivered sizes can never disagree. Also uncovered in the same helper: `metadata.get("expired") is not False` (`:92`), `workflow_run.get("id") != run_id` (`:97`), the output-parent check (`:52`), and `len(uploads) != 6` (`:71`). A one-line fixture tweak (write `size_in_bytes: st_size + 1` into a copy of the metadata, as the test already does for `expires_at` at `:207-210`) makes the equality gate load-bearing.

**3. LOW — `_gh_to_file` streams stdout while leaving `stderr=PIPE` unread, which can deadlock.**
`scripts/distribution/fetch_qualification_artifacts.py:152-170` reads `process.stdout` in a loop but only drains stderr via `communicate()` after stdout hits EOF. If `gh` ever writes more than the ~64 KiB pipe buffer to stderr mid-transfer, it blocks on the stderr write, stops writing stdout, and `process.stdout.read(1024 * 1024)` blocks forever — a hang in a protected release path bounded only by the 60-minute job timeout. `_gh_bytes` (`:137-149`) is safe because `subprocess.run` drains both. Practically unlikely (`gh api` is quiet on stderr in CI), but the fix is mechanical: redirect stderr to a `tempfile.TemporaryFile` and read it after the transfer.

### Nonblocking

- `plans/reviews/active/phase-40-milestone-40-5-stable-publish-primitives-review-pass-2.md` is 0 bytes in the working tree — the same class pass-1 flagged for its own active file. It should not land empty.
- `plans/issues/active/…:600-601` says the new scripts "use one canonical governance package identity." True among the three new scripts, but the other nine `scripts/distribution/*.py` still prepend `AREA_ROOT` and import top-level `governance`, so a single process that loads both families still gets two `GovernanceError` classes. Worth stating as the remaining convention split rather than as resolved.
- `fetch:71-72` (`len(uploads) != 6`) remains unreachable given `artifact_index.py:263`; keep or drop deliberately.
- The CLI surface is still split three ways (`prepare-stable-publication` as a `release_governance.py` subcommand, allocation and revalidation as standalone scripts).
- I could not independently confirm the claim that a real GitHub artifact's `size_in_bytes` exactly equals the downloaded ZIP length; taking that as verified per the brief. It is the one gate in this wave that hard-fails prepare if GitHub's accounting ever diverges.

VERDICT: NOT SATISFIED
