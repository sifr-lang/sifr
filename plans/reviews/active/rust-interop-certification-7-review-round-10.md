## Review — Rust interop certification 7 (round 10)

**Base:** `c9d611fb7c7c5d05421d784d53a2b78c1a7dcae9` (`origin/main`, = merge-base — main fully integrated)
**Exact head:** `64e573d121675167536e0ff256b75d71b7c0049d`
**Diff:** 52 files, +2037 / −205. No files modified by me (working tree identical to session start).

### Validations run at exact head (after cold rebuild)

`target/` was empty — no lane reports exist, consistent with the stated `cargo clean` after disk exhaustion. I required no stale evidence and rebuilt from scratch.

| Check | Result |
|---|---|
| `cargo test -p sifr_driver --lib zero_copy -- --ignored` | **pass** — 3/3 mandatory, 57.3s |
| `cargo test -p sifr_driver --lib rust_interop` | **pass** — 166 passed, 0 failed |
| `cargo test -p sifr_codegen --lib rust_interop` | **pass** — 63 passed, 0 failed |
| `areas run --area rust_interop` | **pass** — variants=10, failures=0; rows=36/36, tiers=5, claims=30 |
| `cargo clippy --workspace -- -D warnings` | **pass** |
| `cargo fmt --check` | **pass** |
| `check_file_size_guardrails.py` | **pass** (2955 files, limit 900) |
| `check_hir_maintainability_guardrails.py` | **pass** |
| `git diff --check origin/main...HEAD` | **pass** |
| `areas run --area documentation` | **fail** — pre-existing, see Low-1 |

Measured inventory matches the issue's post-item block exactly (36 rows / 36 fixture rows / 5 tiers / 30 claims).

### Findings by severity

**No blocking or high findings.**

**Low-1 — the `documentation` area is red at head, caused by the dirty `editor_integrations` submodule, not by this diff.** Both failures (`documentation-ga-release` and the `documentation-structure` self-test, which shells into the same script) reduce to one message: `VS Code package metadata drift: version must be 0.2.0`. The working-tree submodule is at `a980835e6` (`heads/codex/bump-vscode-beta-14-1-ga980835`) with `vscode/package.json` version `0.1.7`; HEAD's committed pointer is `d7577d49`. `editor_integrations` is absent from `origin/main...HEAD` (empty diff), and `origin/main` itself expects `0.2.0`, so this is the user's local submodule checkout, not a certification-7 regression. **Gate-readiness impact:** if agent runs the authoritative lanes against this working tree, the documentation area will fail for this unrelated reason. The submodule needs restoring to the committed pointer (or the lanes run from a clean checkout) before the lane result is meaningful. Round 9 also required this path be preserved — I did not touch it.

Positively, the rust-interop-specific portion of that same area passed: `check_ga_release_docs.py` invokes rust_interop's `check_stable_support_claims.py` and validates the `docs/rust-interop.mdx` claims table and its markers. This is a cross-area consumer of cert-7's docs+claims change that round 9 did not exercise, and it reported no rust-interop failure.

**Note — `view=` identity remains unenforced for generated-record returns** (`zero_copy_validation.rs:238`). Unchanged, intended round-3/4 scoping, documented in `internal_docs/rust_interop_architecture.md` and pinned by `package_rust_interop_preserves_generated_record_view_contract`. A recorded scope limit, not a defect.

### Round 9 findings

- **Finding 2 (import-order/blank-line regression) — CLOSED.** `dab3b5bd0` restores alphabetical order (`_scenario_opaque_resources` → `_scenario_source_checks` → `_scenario_zero_copy`) and reinstates the blank line before `REQUIRED_SCENARIO_EXAMPLES` at `_scenario_checks.py:25-36`.
- **Finding 1 (lane evidence) — superseded.** No lane reports exist at all now; this is missing *future* evidence, not an implementation defect.
- **Rounds 1–8 — all remain closed.** No Rust source changed since `a487ca004`: the delta touches zero files under `crates/`, and the only rust-interop file in it is the `_scenario_checks.py` import fix.

### Main integration (Phase 40) — no Rust-interop semantic impact

The merge `64e573d12` is clean (parents `dab3b5bd0` / `c9d611fb7`); its diff vs parent 1 is exactly main's 720 insertions with no extra conflict-resolution content. I checked the two genuine coupling points, since Phase 40 reaches into rust-interop-owned data:

- **`planner.py:39` / `qualification_fixture.py:142` digest `verification/areas/rust_interop/data/stable_support_claims.json`** — the file cert-7 modified (claims 29→30). No pinned digest literal exists anywhere in the governance code, and no checked-in plan spec carries `advertised_claim_ids`; `require_digest` compares a plan-authored digest against the live file at plan time. The governance self-tests use a synthetic `stable_claims(variant=...)` fixture written into a temp source root, fully decoupled from the real data file. Cert-7's claim addition therefore cannot invalidate Phase 40 governance.
- **`release_evidence.py` `canonicalize_custodied_results`** rewrites the rust_interop area result with canonical JSON bytes and asserts `area == "rust_interop"`. It is release-profile-only, runs only on `status == 0`, and is an identity-preserving reserialization; cert-7 renames neither the area nor its result path.

All other Phase 40 changes are distribution/governance docs, plans, and archived reviews. **No integration regression affects certification 7.**

### Independent substantive verification

I re-derived the core logic rather than relying on prior rounds. `validate_zero_copy_contracts` groups by canonical target path via `.or_default().push(...)`, so every `by_target` value is non-empty and `declarations[0]` cannot panic; because the group key *is* the canonical path, the `zero_copy_probe_obligations` key is correct. Ordering holds: obligations are populated before probe planning reads them. All contract parsing is fully fallible — every malformed key/value yields a `SIFR-RUST-ZC-0001` diagnostic, with `mutable` and `copy_fallback` rejected verbatim. `returned_ok_type` does a depth-aware top-level comma scan using `char_indices` boundaries and `saturating_add/sub`, and degrades to a diagnostic (not a panic) when no top-level comma exists. `signature_has_unsupported_type` honors both `kind == Unsupported` and `unsupported_reason`, preserving the round-2 diagnostic-ordering fix.

### Verdict

**SATISFIED**

The implementation, evidence bindings, hermeticity, provenance, safe-Rust runtime behavior, and documentation are sound at `64e573d12`. Round 9's only actionable finding is closed, rounds 1–8 remain closed, and every focused check I ran — mandatory zero-copy tests, the full Rust-interop area, the broader driver/codegen interop suites, clippy, fmt, both guardrails, and diff hygiene — is green after a clean rebuild. Phase 40 integration does not touch Rust-interop semantics.

This is satisfaction on *implementation*. Gate readiness still depends on agent producing fresh green `create-pr` and `merge` lanes at this exact head, which I did not run per your instruction. Two things to settle first: restore `editor_integrations` to the committed pointer (or run the lanes from a clean checkout), or the documentation area will fail on the unrelated VS Code version drift; and watch `performance_budget_checks`, whose round-9 failure I previously attributed to environment rather than this diff — that attribution is still a prediction, and only a green lane run makes it evidence.
