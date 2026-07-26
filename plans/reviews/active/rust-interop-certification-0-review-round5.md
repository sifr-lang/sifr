## Round-5 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only. I re-derived round-4 finding 1 from the tree, read the new `_prose_units` rules line by line, ran the checker and its self-test, and executed in-memory mutation probes (nothing written into the repo — the `main()` deletion probe ran against a throwaway temp tree). No file was edited, staged, or committed.

### Round-4 findings, re-checked

| # | Round-4 finding | Status now |
| --- | --- | --- |
| 1 | qualifier-borrowing bypass of the stable-claim prose gate | **Not resolved.** Only the one literal comma-`except` form was closed; five natural variants still bypass. See below. |
| 2 | justification linked an untracked parallel-agent file | **Resolved.** `rust-interop-runtime-ecosystem-certification.md:288-292` now states the evidence inline (412 variants, 20 algorithmic-corpus failures, no algorithmic/compiler file changed, owning issue lands separately). No dangling link remains. |
| 3 | promotion direction unguarded | **Resolved in principle.** `check_stable_support_claims.py:288-300` rejects a stable claim described with `future-owned`/`planned`/`pending`/`unadvertised`, with a real mutation at `:548-580`. Subject to the same unit-splitting defect as finding 1. |
| 4 | `main()`'s docs-wide wiring unguarded | **Resolved.** `:606-616` invokes `main([], repo_root=temp_root)` against a real temp repo. I deleted `_collect_public_documents` from `main()` in a copy: the self-test now fails with `main() did not enforce the docs-wide sweep` (exit 1). |
| 5 | self-test lost the guard on lane-step reporting | **Resolved for the seam.** `profiles.py:25` adds `cargo_cache_setup` to `PROFILE_STEP_NAMES`, and `selftest.py:325-352` asserts `events == [header, timed:cargo_cache_setup, cargo-cache-setup, offline, selected-areas]` — reverting `run()` to module-level `timed_step` drops `timed:cargo_cache_setup` and fails. Residual below. |
| 6 | cold prelude cost unrecorded | Open, optional. |
| 7 | `/tmp` hardcode in the new row | Open, optional (`zero_copy_runtime_matrix/examples/memmap2.sifr:13`). |
| 8 | maintainability headroom | Open, optional (`profile_runner.py` 869/900). |
| 9 | README did not disclose the aborts | **Resolved.** `README.md:147-149`: "…before both profiles later stopped on the same unrelated pre-existing algorithmic full-corpus failures." |

### Findings

**1. MEDIUM — BLOCKING (round-4 finding 1, not resolved). `_prose_units` only recognizes two unit boundaries, so qualifier borrowing still lets a contract-only row be advertised as runtime-supported, and a runtime deferral be advertised as shipped, with every gate green.**

`_prose_units` (`check_stable_support_claims.py:223-238`) collapses each `\n\n` block into one whitespace-joined string, then splits only on (a) `[.;]` + whitespace and (b) a **literal comma** followed by exactly `but|except|however`. Everything else in a paragraph stays one unit and can donate its qualifier.

All five probes below were appended to the real `docs/rust-interop.mdx` and validated against the real matrix and real claims file. Each returned **`_validate(...) == []`** — zero failures:

```
G  - these rows are contract-only and do not certify runtime behavior
   - `zero_copy_bytes` now provides runtime support

H  these rows are contract-only and do not certify runtime behavior except
   `zero_copy_bytes` which now provides runtime support        # one comma removed

I  These rows are contract-only and cannot certify execution, although
   `zero_copy_bytes` now provides runtime support              # 'although' not in the set

J  - the rows below are future-owned and planned
   - `zero_copy_runtime_matrix` now provides runtime support   # deferral direction

K  All these rows are future-owned and planned except
   `zero_copy_runtime_matrix` which now provides runtime support
```

`G`/`J` are the important ones: a markdown bullet list is the most natural way to document per-row status, and because no item ends in `.`, the entire list collapses to a single unit (verified: `_prose_units` returns exactly one element for `G`). `H` is the self-test's own new mutation string with one comma deleted. `J`/`K` show the defect is not confined to the contract-only rule — the `runtime_deferral_ids` rule at `:256-266` borrows `future-owned`/`planned` the same way.

Why the gate looks green: the new mutation case `"contract qualifier borrowing"` (`:512-519`) uses the comma-`except` form, which is the *only* adversative shape the regex handles. And the four blocked variants I probed in the qualifier-borrowing shape (`A`–`D` in my run) fail only incidentally — they trip the *stale-claim* rule at `:288-300` because the borrowed qualifier happens to contain `unadvertised`. Substituting a qualifier that avoids `future-owned/planned/pending/unadvertised` (e.g. "contract-only and do not certify") removes that accidental catch, which is precisely cases `G`–`I`.

I confirmed `check_stable_support_claims.py` is the *sole* consumer of `docs/rust-interop.mdx` in `verification/` — no other check would catch these.

Fix (in `_prose_units`, ~6 lines, no new authority):
1. Before collapsing whitespace, split each `\n\n` block at newlines that start a markdown list item or table row — `^\s*(?:[-*+]|\d+[.)])\s` and `^\s*\|` — so each bullet/row is its own unit while ordinary wrapped prose still joins (necessary: the real disclaimer at `docs/rust-interop.mdx:80-87` wraps its row tokens and its `contract-only` qualifier onto different physical lines, so a blanket newline split would fail the real doc).
2. Drop the comma requirement and widen the connective set: `(?:,\s+|\s+)(?=(?:but|except|however|although|though|whereas|besides|aside from|apart from|other than|with the exception of)\b)`.
3. Add `G`, `H`, `I`, and `J` verbatim as mutation cases (both rule directions), not just the comma form.

Residual after that fix: a bare soft line break with no punctuation at all ("…do not certify runtime behavior\n`zero_copy_bytes` now provides runtime support") also returns `[]` today and survives (1)–(2). Closing it requires splitting on every newline, which in turn requires rewrapping `docs/rust-interop.mdx:80-87` so each disclaimer clause is line-contained. That is a judgment call, not a blocker — I'd take (1)–(3) and record the soft-break case as a known limit.

### Optional

**2. LOW — the lane-step *reporting* is still not asserted (round-4 #5 residual).** `RecordingProfileRunner.run_timed_step` (`selftest.py:337-340`) returns a synthetic `StepResult` without calling the real `timed_step`, so `cargo_cache_setup` being emitted as a `[sifr-lane-step]` line is inferred from `PROFILE_STEP_NAMES` membership (`profiles.py:25`) rather than observed. The ordering assertion now does catch the specific round-3 regression, so this is a residual only. A captured-stdout assertion over one real `timed_step` call would close it.

**3. LOW — the prose gate is keyword-bound in a second way.** `ADVERTISEMENT_TERMS` (`:34`) and `RUNTIME_CLAIM_TERMS` (`:21-25`) miss claims phrased without those exact tokens — e.g. "`zero_copy_bytes` now performs real zero-copy exchange at runtime and is production-ready" passes cleanly. Inherent to keyword gating and out of round-4's scope; worth noting in `certification.md` as a stated limit of the gate rather than expanding the term list indefinitely.

**4. LOW — cold prelude cost still unrecorded (round-4 #6).** `README.md:142-152` records four warm cache-setup figures (566/815/495/812 ms) and no cold `cargo fetch --locked` number for the ~324 added packages.

**5. LOW — `/tmp` hardcode (round-4 #7).** `fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:13`. Matches the pre-existing `zero_copy_view_matrix` convention and is inert while the row is `planned`; must be replaced when `certification_7` executes it.

**6. LOW — maintainability headroom (round-4 #8).** `profile_runner.py` 869/900, `check_fixture_matrix.py` 758, `check_stable_support_claims.py` 675 (up 106 from round 4), `selftest.py` 595. The finding-1 fix adds to `check_stable_support_claims.py`; splitting the prose rules into a `_prose_scope.py` sibling would be the natural next decomposition.

### What verifies clean

Baseline `_validate` against the real matrix, real claims, and the real docs-wide sweep returns `[]`. `check_stable_support_claims.py --self-test` → `cases=21`, exit 0; real run → `claims=23`, exit 0. The `main()` docs-wide wiring is genuinely regression-guarded (mutation caught). The stale-promotion rule exists with real coverage. No `.md`/`.mdx` file lives outside `docs/`, so the sweep root is complete for the current tree. The runner self-test's cache-setup ordering guard is real. Round-4 optional items 2 and 9 are both properly closed, and the issue no longer depends on the parallel agent's untracked file. Round-3-era inventory, catalog mutation coverage, feature-policy single authority, error propagation, and suite registration all remain as verified in round 4 — nothing in this round's changes touches them.

The blocking issue is confined to one function (`_prose_units`) and its mutation cases; everything else in the milestone is ready.

**Blocking: 1.** Optional: 2–6.

NOT SATISFIED
