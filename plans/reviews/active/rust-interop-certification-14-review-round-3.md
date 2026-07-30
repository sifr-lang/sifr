## Independent Final Review — Round 3, Rust Interop `certification_14` / Track A closeout

**Read-only confirmed.** `git status` is byte-identical to session start. No file edited, no commit, no PR, no merge-scale suite. Only gitignored `target/` and `__pycache__` artifacts were produced.

### Round-2 findings: all four closed

| Round-2 finding | Status |
| --- | --- |
| **1 MEDIUM** — retrospective obligations neither performed nor re-homed | **Resolved and internally consistent.** Status table `certification_7` row (`:156`) now reads "merged; performance recalibration re-homed", naming the active follow-up. The old retrospective paragraph (`:996-1000`) records the `certification_14` audit and re-homing. The checklist item (`:1734-1737`) is `[x]` and adds "Do not bless shared-host samples into reference baselines." Closeout evidence (`:1786-1794`) names both the `certification_7` rerun and the `certification_8` repository-wide recalibration and states this closeout changes no baseline or threshold. The four surfaces agree. |
| **2 LOW** — Phase 40 lists an in-progress item as completed | **Resolved.** `40_stable_channel_ga_promotion_and_release_governance.md:54-58` now reads "merged Track A certifications 0 through 13, plus the in-progress `certification_14` closeout prerequisite", and the tense of the `certification_0` sentence is corrected to past. |
| **3 LOW** — round-10 artifact preamble | **Resolved.** The file now begins at its `# Published-Head Merge-Readiness Review …` title; no session preamble. |
| **4 LOW/informational** — transitive `ring`/`libsqlite3-sys` grants | **Adjudicated below: informational, not a blocker.** |

**Re-homing target verified.** `plans/issues/active/adhoc_performance_budget_host_variance.md` is tracked, unmodified by this diff, 7,666 bytes. Its DoD requires "a repeatable verdict across at least five consecutive controlled runs on a supported host" (`:132-133`), and `:12` states "This follow-up must not change performance baselines or add waivers merely to make one host pass." Both requirements the instruction names check out exactly.

**Cited artifacts.** Every `../../reviews/active/…` and `./…md` link in the certification issue resolves to a nonempty file. Only `rust-interop-certification-13-review-round-10.md` (161 lines, verdict `SATISFIED`, head `f8ab7080c…`) is untracked — it is a new file this PR adds, which is the intended resolution.

### Round-2 finding 4 — explicit adjudication: **informational, not a closeout blocker**

The transitive grants are `async_runtime_reqwest/.../sifr.toml:22` (`ring`), `opaque_resource_matrix/.../sifr.toml:26` (`libsqlite3-sys`, `ring`), and `ecosystem_backend_certification/.../sifr.toml:19` (`ring`). Reasons they do not block:

1. **The documented contract does not scope build-script trust to direct edges.** `internal_docs/rust_interop_architecture.md:1177`: "The sys crate that runs build code must be listed in `rust-build-scripts`; the native link it exposes must be listed in `native-links`." That is stated unconditionally. Each of those three manifests pairs its build-script entry with the matching `native-links` entries (`ring_core_0_17_14_`, `sqlite3`). The declarations follow the documented rule; the compiler's direct-only reach (`derive_backend_crates` → `validate_package_dependency_trust`, `trust_validation.rs:84-117`) is *under*-enforcement of a stated contract, not over-declaration against it.
2. **Transitive declaration is already the norm for the paired trust family.** `trust_validation.rs:121-146` records native links with evidence literally reading "manifest-declared **transitive** native link", and `certification_9` (`:1041-1043`) certified a post-build allowlist over "the exact direct and transitive native-link envelope". Removing the sys-crate entries would break the documented pairing.
3. **No `certification_14` claim is contradicted.** The only trust claim in the closeout is scoped: "The manifests now declare those four exact enforced graph entries" (`:1775-1776`) — about the four added grants, not about global minimality. This is materially unlike round-1 finding 1, where the narrative asserted a merge-lane regression that was not reproducible and the grants sat in the diff.

Correct home is a future trust-policy hardening topic (extend build-script trust to the transitive graph), not this closeout.

### Re-verified surface

**The four grants — exact, necessary, sufficient.** Diff adds exactly `rust-build-scripts = ["serde", "serde_json", "thiserror"]` (`bridge_type_roundtrip/sifr.toml:19`) and `rust-build-scripts = ["zerocopy"]` (`crate_backed_view_runtime/sifr.toml:18`). No `rust-proc-macros` key in either fixture. Static confirmation of exactness: of the bridge package's five direct deps (`bytes`, `indexmap`, `serde`, `serde_json`, `thiserror`) only the latter three ship `build.rs`; of the zero-copy package's five direct deps only `zerocopy` does. Both READMEs describe the grants accurately and scope them to direct dependencies.

**Four mutations.** `_scenario_checks.py:163-183` adds three bridge build-script drift cases; `_scenario_zero_copy.py:178-184` adds one. Fixture self-test totals 233 (229 + 4) — exact. `_scenario_zero_copy.py:104-111` and `_scenario_checks.py:497-504` require exactly the declared sets.

**Diagnostic-preserving assertions.** All five pristine assertions in `package_rust_interop_build_tests.rs` (lines 110, 177, 204, 266, 386) now interpolate `{pristine_errors:#?}`.

| Gate rerun this round | Result |
| --- | --- |
| Full `rust_interop` area | `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0` |
| Inventory | `fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18`; `rows=36 fixture_rows=36 categories=3`; `claims=36` |
| Self-tests | fixture 233, compatibility 7, tiers 6, stable claims 33, stale drafts 20 |
| Resource gate + self-test | `PASS (surfaces=1, future_runtime_rows=0)`; self-test PASS |
| Matrix recomputed independently | 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`; 72/72 `passing`, 0 planned; execution 13/4/10/9; zero `future_owner` in rows or claims; 36/36 fixtures `schema_version: 2` |
| `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size (3011 files/900), HIR + driver guardrails, docs error-code links | all PASS |

**Phase mappings and stable claims.** Phase 39 milestone→certification attributions all match the status table: 39_5→`certification_1`, 39_6→`certification_5`, 39_9→`certification_7`, 39_10→`certification_8`, 39_11→`certification_3`/`certification_6`; 39_13 records 72 passing / 0 planned. Advertising is bound to `stable_support_claims.json` and its validator in both Phase 39 and Phase 40, not to phase prose. Roadmap row 39 correctly reads "completed, audited; Track A closeout in progress". Track B is consistently described as dormant and not a Phase 40 blocker.

### Findings

**1. LOW — one conjunct of the re-homing evidence overstates the follow-up's requirements.**
`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1790-1792` states the follow-up "requires five controlled consecutive runs, host/load/thermal evidence, a deterministic stability-rule regression, and **approved reference-hardware capture**." The first three map exactly to `adhoc_performance_budget_host_variance.md:132-133`, `:120`, and `:128`/`:134`. The fourth has no counterpart — the words "approved", "reference-hardware", and "capture" do not appear anywhere in that file (`grep` for `reference|hardware|approv` returns only two unrelated Phase-40 approval-boundary sentences at `:72` and `:79`). Fix: drop the phrase or replace it with the actual fourth DoD item, "documented controlled measurement conditions" (`:136-137`).

**2. LOW — `certification_7`'s final checklist item is still unchecked while its status row says merged.**
`…certification.md:811` remains `- [ ] Run focused and authoritative local gates, Opus review rounds to satisfaction, merge the PR, and unblock only certification_8`, though `:156` records it merged via PR #3053. This diff checked the byte-identical `certification_13` item (HEAD `:1460` → now `[x]`), so the omission is now the only such inconsistency in the Track A obligation surface, inside the same paragraph this closeout edited. Fix: mark `:811` `[x]`.

**3. LOW — the round-3 review artifact is currently 0 bytes.**
`plans/reviews/active/rust-interop-certification-14-review-round-3.md` is untracked and empty — the same class as round-1 finding 3. It must be populated or removed before the PR.

### Not findings

- `cargo clippy --workspace --all-targets -- -D warnings` fails with 14 errors in `sifr_codegen` (lib test), e.g. `single_char_pattern`. This is **not** the documented gate (`AGENTS.md` and every project script use `cargo clippy --workspace -- -D warnings`, which passes), no gate anywhere in `scripts/` or `verification/` passes `--all-targets`, and the failures are in a crate this diff does not touch. Pre-existing and out of scope, recorded for awareness only.
- No user-path panic surface added; the diff's `assert!` changes are test-only. No file-size, maintainability, or lint regression. Excluded dirty paths were not weighed.

All four round-2 findings are closed, the re-homing is byte-checkable against the named issue, Phase 40 distinguishes merged 0–13 from in-progress 14, both cited artifacts are durable and nonempty, and every inventory, mutation count, matrix figure, and gate I reran reproduces exactly. The three remaining items are editorial cleanups that touch no certified technical claim; finding 1 and 2 should still be applied before the closeout commit.

VERDICT: SATISFIED
