## Round-4 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only. I ran the area, the runner self-test, `cargo fmt --check`, `cargo clippy -p sifr_rust_interop_catalog --locked --offline`, the file-size guardrail, `git diff --check`, and read-only Python probes against the checkers. No file was edited, staged, or committed. I re-derived every round-3 finding from the tree and re-derived the inventory independently.

### Round-3 findings, re-checked

| # | Round-3 finding | Status now |
| --- | --- | --- |
| 1 | create-PR budget evidence not reproducible | **Resolved.** `create-pr.json:16` is now `10000` blocking; `create-pr.latest.json` records `rust_interop_checks` at **3,867 ms** and `cargo_cache_setup` at **829 ms**, exactly what `README.md:142-147` publishes, and 6,133 ms of headroom is arithmetically right. |
| 2 | merge/nightly/release never executed | **Resolved.** `merge.latest.json` has all 23 steps `pass` (Rust interop 3,953 ms, cache setup 815 ms); nightly 4,161/495 ms and release 3,880/812 ms. All four reports contain exactly one passing `cargo_cache_setup` and one passing `rust_interop_checks`. Nightly and release later abort at `algorithmic_compatibility_checks` (412 variants / 20 failures, all LeetCode-corpus cases — nothing in this diff touches that path); the item discloses this at `certification.md:213-218`. |
| 3 | self-test poisoned the lane-timing channel | **Resolved at the root.** `selftest.py:324-326` overrides `run_timed_step`, and `profile_runner.py:288-290` introduces that seam. `uv run … --self-test` emits no `[sifr-lane-step]` line, and each of the four reports now contains `cargo_cache_setup` exactly once — no 0 ms duplicate. |
| 4 | docs-wide discovery untested | **Resolved as prescribed.** `_collect_public_documents` extracted (`check_stable_support_claims.py:47-53`), driven against a real temp tree at `:513-520`. Residual below. |
| 5 | contract-only notes unbound | **Resolved.** `check_compatibility_matrix.py:182-186` enforces `execution_kind == contract-only ⇒ notes contain contract-only`, with a mutation case at `:366-391`. All five preserved rows' notes were rewritten to lead with `Contract-only`. |
| 6 | cold prelude cost unrecorded | Open, optional. |
| 7 | setup command validated ≠ executed | **Resolved.** `cargo_setup.py:24` returns `shlex.split(CANONICAL_SETUP_COMMAND)`. |
| 8 | dead second offline guard | **Resolved.** `enable_offline_cargo` (`profile_runner.py:293-296`) no longer re-tests the policy. |
| 9 | ten forced in-place pins undocumented | **Resolved.** `certification.md:220-226`. |
| 10 | "exact-pinned" is alias-level | **Resolved.** `certification.md:197-199` now says "exact-pinned at the catalog dependency boundary … transitive family crates remain selected by that exact locked graph." |
| 11 | two catalog branches unmutated | **Resolved.** `_crate_catalog.py:208-213` (non-table `[dependencies]`) and `:226-236` (non-dict dependency value) — 14 cases + control. |
| 12 | `/tmp` hardcode in the new row | Open, optional. |
| 13 | no promotion guard on deferral prose | Open, optional. |

### Findings

**1. MEDIUM — BLOCKING. The public stable-claim prose gate can be bypassed by one natural sentence in the exact paragraph it protects, letting a contract-only row be advertised as runtime-supported with every gate green.**

Two checks guard prose, and their granularities leave a gap between them:

- `check_stable_support_claims.py:287-288` skips an **entire physical line** if any `DEFERRAL_TERMS` token appears anywhere on it — and `"contract-only"` is one of those tokens (`:37`).
- `_validate_public_document_scope` (`:248-267`) evaluates the `contract-only` + negation qualifier per **paragraph**, so a qualifier belonging to a *different* row satisfies it.

`docs/rust-interop.mdx:77-87` is a single paragraph that contains both `contract-only` and `unadvertised`. Reproduced against the real tree:

```
docs/rust-interop.mdx:81-82  … and `advanced_data_matrix` are
contract-only, except `zero_copy_bytes`, which now provides runtime support.
```

`_validate(...)` returns `[]`. The line check is skipped because the line contains `contract-only`; the paragraph check is satisfied by the `unadvertised` and `contract-only` that describe the seven deferrals. The focused area, all four profiles, and `stable-candidate` all stay green while the canonical public document claims runtime support for a contract-only row. (A pure overclaim on its own line *is* caught — `:283-294` fires — so this is specifically the qualifier-borrowing case, which is exactly how a future promotion PR would edit that paragraph.)

This is the same class as round-3 finding 5, in the one gate this milestone exists to create, and the item's own exit gate is "stable claims preserve the narrower contract-only rows without overclaim" (`certification.md:271`).

Fix (≈10 lines, no new authority): evaluate both rules at sentence granularity and require co-occurrence. Split each paragraph on `.`/`;` into units, and in `_validate_public_document_scope` require that the unit mentioning a `compile_scope_ids` row with a `RUNTIME_CLAIM_TERMS` phrase itself contains `contract-only` and a negation; likewise apply the `DEFERRAL_TERMS` skip in `_validate_unstructured_advertisements` per sentence rather than per line. Add the probe above as a mutation case — the existing `"contract runtime prose overclaim"` case (`:479-485`) uses a separate paragraph and therefore does not cover it.

### Optional

**2. MEDIUM-LOW — the item's justification for two failing profiles points at a file outside its own commit.** `certification.md:215-217` links `[ad-hoc-algorithmic-full-corpus-preexisting-failures.md]`, which is untracked and owned by a parallel agent. Nothing gates plans links (`check_compatibility_matrix.py` only checks `future_owner`), so if this PR merges first the sole justification for the nightly/release aborts is a dangling reference. Cheapest fix: state the evidence inline — 412 variants, 20 blocking failures, all algorithmic-corpus cases, no algorithmic or compiler file in this diff — and note that the owning issue lands separately.

**3. MEDIUM-LOW — the promotion direction is still unguarded (round-3 #13).** `_validate_public_document_scope:222-247` only constrains rows still in the derived deferral set, and `DEFERRAL_TERMS` makes `:287` skip any line saying "planned". When `certification_1`+ promotes a row, `docs/rust-interop.mdx:82-87` can keep calling it future-owned and planned with every gate green. A `runtime_deferral_ids`-complement rule (a claimed row's id must not appear in a paragraph with `future-owned`/`planned`) closes it; the same sentence-splitting from finding 1 makes it cheap.

**4. LOW — `main()`'s docs-wide wiring is still not regression-guarded.** The self-test calls `_collect_public_documents` directly (`:520`), not through `main()`. Deleting `:553` and the `public_documents` argument at `:558` leaves all 18 cases green while collapsing the gate to the single canonical file. This is what round 3 prescribed, so it's a residual, not a reopening.

**5. LOW — the fix to round-3 #3 removed the guard on the thing it fixed.** `RecordingProfileRunner.run_timed_step` (`selftest.py:324-326`) never calls the real `timed_step`, so nothing in the self-test asserts that `cargo_cache_setup` is *reported* as a lane step. If `run()` reverted to calling module-level `timed_step` directly, the ordering assertion would still pass and the fake-record regression would return. Asserting `"cargo_cache_setup" in legacy_facade_step_names()`-style coverage, or checking `PROFILE_STEP_NAMES` membership plus a captured-stdout assertion, restores it.

**6. LOW — cold prelude cost still unrecorded (round-3 #6).** `README.md:144-147` records four warm measurements (829/815/495/812 ms) but no cold `cargo fetch --locked` figure for the ~324 added packages, and the create-PR run consumed 717 s of its 15-minute cold envelope. One recorded cold number would make the envelope auditable. Note that merge/nightly/release carry no `step_budgets` at all, so the prelude being unbudgeted there is not specific to this change.

**7. LOW — `/tmp` hardcode carried into the new row (round-3 #12).** `fixtures/zero_copy_runtime_matrix/examples/memmap2.sifr:13` uses `/tmp/sifr-rust-interop.bin`. It matches the pre-existing convention at `fixtures/zero_copy_view_matrix/examples/memmap2.sifr:17` and is inert while the row is `planned`, but it violates the temp-dir rule at `certification.md:262-270` and must be replaced when `certification_7` executes it.

**8. LOW — maintainability headroom.** `profile_runner.py` is 869 lines against the 900 cap and gained 32 here; `check_fixture_matrix.py` 758, `check_stable_support_claims.py` 569, `selftest.py` 593. The `_matrix_inventory.py` extraction was the right call; the next `profile_runner.py` change should split rather than append.

**9. LOW — README timing prose does not disclose that two profiles aborted.** `README.md:147` says "Exact-state nightly and release runs measured 4,161 ms and 3,880 ms respectively," which is true of the step but reads as whole-profile success. The item plan discloses the aborts; one clause in the README would keep the two documents consistent.

### What verifies clean

Inventory reproduces exactly and independently: 36 compatibility rows / 36 fixture rows, 47 `passing` + 25 `planned`, categories 17 `supported` / 5 `supported-through-bridge` / 1 `unsupported-by-design` / 13 `future-owned-by-separate-phase`, execution kinds 13 `cargo-probe` / 4 `compiler-diagnostic` / 10 `contract-only` / 9 `runtime-observed`, 44 crate aliases (matching the 44 `[dependencies]` entries in `crates/sifr_rust_interop_catalog/Cargo.toml`), 23 claims, 7 runtime deferrals, 5 suites / 10 cases — every number in `certification.md:189-202` and `README.md:128-130`.

Both new rows are honest: `future-owned-by-separate-phase` with a real `future_owner`, both evidence directions `planned`, legal tier/execution pairs (2 and 4 with `runtime-observed`), and no `contract-only` note obligation. The five preserved rows changed `notes` only — `category`, `execution_kind`, `capability`, and evidence are byte-identical. `EXPECTED_FEATURE_POLICIES` (`_matrix_inventory.py:123-137`) remains the single authority for catalog, matrix, and fixture feature policy; the `candle` `default_features: false` addition propagated to matrix, `advanced_data_matrix/fixture.json`, and `Cargo.toml` consistently. Catalog mutation coverage is now 14 cases: non-table dependencies, non-dict dependency, missing crate, extra crate, non-exact pin, absent-from-lock, wrong features, both `default-features` directions, `optional: false`, wrong `package` alias, a `[features]` table, metadata missing, and metadata drift. `lib.rs` is a doc comment with no synthetic constant.

Suite registration is identical across create-pr/merge/nightly/release, `stable-candidate` is derived from the manifest, and the round-3 `_public_table` / role / schema / source / deferral-set mutation coverage all survives. `cargo_setup_command` is now single-authority and asserted for all five profiles (`selftest.py:99-104`, `:144-145`). Error propagation is intact: `ValueError → ProfileRunnerError → timed_step` status 2 → `run()` aborts before any validation step; `setup_env` pops `CARGO_NET_OFFLINE` so only the prelude can reach the registry, which `profile_policy.md:17-27` now states explicitly.

Executed here: area run 10 variants / 0 failures; fixture-matrix self-test 83 cases; compatibility 5; tiers 6; stale-drafts 20; stable claims 23 and self-test 18; runner self-test all eight sections pass; `cargo clippy -p sifr_rust_interop_catalog --locked --offline -- -D warnings` clean; `cargo fmt --check`, file-size guardrail (2833 files), and `git diff --check` pass. No scope creep: the `Cargo.toml` member, `cargo_metadata_classification.json` entry, and Phase 40 ownership edits are each required by this item, and no unrelated user change was touched.

**Blocking: 1.** Optional: 2–9.

NOT SATISFIED
