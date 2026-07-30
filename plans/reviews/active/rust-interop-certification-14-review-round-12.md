## Final Exact-Published-Head Review — Rust Interop Runtime/Ecosystem Certification 14 (draft PR #3083)

**Head reviewed:** `2ba78e2d88aba022dac58048b75556e5df1e1100`
**Scope:** `git diff origin/main...2ba78e2d8` → 27 files, +1258/−69 (`git diff --stat origin/main...2ba78e2d8`). Exactly one Rust file: `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:104-111,171-178` — `cfg(test)`-only, binds `check_package_project(&pristine_entrypoint)` to a local and prints `{pristine_errors:#?}` on failure. No compiler, frontend, diagnostic, LSP, codegen, or runtime path is touched.
No files were modified, committed, pushed, or PR state changed by this review.

---

### 1. Round-11 note 1 — build-script trust exactness: **closed at the root**

- Exact-equality enforcement is now wired for the bridge fixture: `_scenario_checks.py:504-511` calls the new `_require_exact_trust_targets` (`_scenario_checks.py:826-839`, `targets != expected_targets`) with `["serde", "serde_json", "thiserror"]`.
- Manifest retains exactly those three entries: `fixtures/bridge_type_matrix/examples/bridge_type_roundtrip/sifr.toml:19` → `rust-build-scripts = ["serde", "serde_json", "thiserror"]`.
- The over-declaration mutation is **effective**, not decorative. Empirically re-derived at this head:

```
["serde","serde_json","thiserror","syn","quote"] -> [... [trust].rust-build-scripts must equal ['serde','serde_json','thiserror']]
["serde_json","serde","thiserror"]               -> [... must equal ...]
["serde","serde_json","thiserror"]               -> NO FAILURE
```

- The three necessity mutations are intact and unweakened (`_scenario_checks.py:163-183`: serde-, serde_json-, thiserror-missing), and the new fourth (`:184-190`) is a distinct token so the harness's `if before not in original: return … setup token is missing` guard (`:202-203`) proves each mutation actually applies; baseline is asserted clean first (`:123-132`) and each file is restored (`:212`).
- Unrelated trust checks are untouched: `unsafe-rust-bridges` still uses the subset helper `_require_trust_targets` (`:495-503`, `:812-823`) with its own mutation at `:156-162`; the zero-copy path is unchanged from round 11 (`_scenario_zero_copy.py:104-111`, mutation `:178-184`).
- Root cause confirmed real, not fixture-cosmetic: the compiler is fail-closed on *missing* grants only — `crates/sifr_driver/src/build/rust_interop.rs:521-534` requires a `rust_build_scripts` grant when `backend.has_build_script`, and never rejects extras. So the fixture-side exact assertion is the correct and only available guard against over-declaration regressions.
- Recomputed fixture self-test count: `check_fixture_matrix.py --self-test` → **`cases=234`**, matching the ledger's corrected "229 → 234 / five new mutations" (`plans/issues/.../rust-interop-runtime-ecosystem-certification.md:1764,1774-1779`). 3 bridge necessity + 1 bridge over-declaration + 1 zero-copy = 5. ✅

*Informational only:* `_require_exact_trust_targets` is order-sensitive (a legitimate reorder of the manifest list would fail). Harmless for a frozen fixture; no action required.

### 2. Round-11 note 2 — offline-merge-smoke coverage: substantively closed, evidence text inexact

- Closeout enumeration now names the step: issue `:1910` — "package-management offline merge smoke 2/2" inside the post-fail-fast merge-step bullet, so the "no functional merge step hidden behind the fail-fast comparison is omitted" claim (`:1918-1920`) is now complete against the merge plan.
- Durable artifact section added: `plans/reviews/active/rust-interop-certification-14-merge-continuation-evidence.md:41-61`.
- Independently reproduced at this head:

```
uv run --project verification --locked python -m sifr_verify areas run \
  --area package_management --suite offline-merge-smoke
offline package merge smoke ok
offline package merge smoke ok
package management verification ok: variants=2, failures=0, blocking_failures=0, non_blocking_failures=0
```

The **2/2 count and pass status are exact**. Two record-accuracy defects remain in the new artifact section — see findings L1/L2.

### 3. Inventory, stable, and resource claims — all recomputed exact

| Closeout claim (issue `:1745-1757`) | Independent result |
| --- | --- |
| 36 fixture-matrix rows / 36 compatibility rows / 36 schema-v2 manifests | `fm rows 36`; `rows 36`; `manifests 36`, `Counter({2: 36})` ✅ |
| 72 passing, 0 planned | `Counter({'passing': 72})` — no other status ✅ |
| 21 supported / 14 supported-through-bridge / 1 unsupported-by-design | exact ✅ |
| 13 cargo-probe / 4 compiler-diagnostic / 10 contract-only / 9 runtime-observed | exact ✅ |
| 44 exact-pinned crate aliases | union of `required_crates` = 44 ✅ |
| 61 package + 18 scenario examples | `ls -d fixtures/*/examples/*` = 79 = 61+18 ✅ |
| 36 stable claims, no stale `future_owner` | `claims 36`; `future_owner` in rows = `[]`, in claims = `[]` ✅ |
| `future-owned-by-separate-phase` declared but unused | declared in `categories`, zero members ✅ |

Validators/self-tests at this head: `check_compatibility_matrix.py --self-test` → `cases=7`; `check_tiers.py --self-test` → `cases=6`; `check_stable_support_claims.py --self-test` → `cases=33` / `claims=36`; `check_stale_drafts.py` → ok; `scripts/check_sysroot_stdlib_resource_certification_gate.py [--self-test]` → PASS. Full area: `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`. Guardrails: `file-size guardrails: PASS (3019 files, limit 900)`; `lowering maintainability guardrails: PASS`; `cargo fmt --check` clean; `git diff --check origin/main...2ba78e2d8` clean.

Prose de-deferral (rounds 5/6) holds: the three matrix notes now delegate to `opaque_resource_matrix`, `async_runtime_reqwest`, `callback_subscription_ecosystem` (all certified rows, cert 5/4/6), and `internal_docs/sifr_sysroot_and_stdlib_architecture.md:912-915` matches. Phase 39/40 and roadmap edits are past-tense with correct certification attributions; `certification_14` is still `in progress` (issue `:163`) with the final merge/review checkbox correctly **unchecked** (`:1738`).

### 4. PERF-HOST governance — unchanged

`plans/issues/active/adhoc_performance_budget_host_variance.md:112-124` is the only performance-file change: a ledger incident paragraph recording 1358.717/1357.524, 1366.015/1334.139, 1354.814/1335.954, 5.962/5.91 median + 11.664/10.933 p95, plus the same-host unrelated-branch control at 3313.437 / 4612.439 / 4132.029 / 17.918 median + 22.939 p95. No baseline, threshold, waiver, budget, or profile-selection file appears anywhere in `git diff --name-only origin/main...2ba78e2d8`. Round 9's four-case + separate-p95 mapping is intact.

### 5. Round-11 artifact and ledger summary — accurate

`rust-interop-certification-14-review-round-11.md` (135 lines, `VERDICT: SATISFIED` at line 135) reports head `8452e8024`, `cases=233`, and its two L-findings; both are exactly the notes the head commit closes, and the delta 233 → 234 is precisely the one added mutation. The ledger bullet at `:1859-1866` describes it accurately. Rounds 1-11 are all checked in, non-stub, with explicit verdicts (`NOT SATISFIED` ×2, `SATISFIED` ×2, `NOT SATISFIED` ×2, `SATISFIED`, `NOT SATISFIED` ×2, `SATISFIED`, `SATISFIED`) and all findings recorded closed. The carried `rust-interop-certification-13-review-round-10.md` is a complete 161-line artifact ending in `**SATISFIED**`. Continuation-evidence figures I could verify statically all reproduce: project suites are exactly 7 rows (`frontend_mode_parity` 2 + `project_graph_isolation` 5, per `data/validation_suites/manifest.json`), and I reran both suites — `project workspace verification ok: variants=2, failures=0, …` with literal `project graph isolation regression matrix: PASS`. I did not rerun the merge profile, the 1794 s driver ignored suite, or the 678-fixture E2E, per instruction.

---

## Findings (both low severity, evidence-record only)

**L1 — Non-reproducible console line inside the durable merge-continuation evidence.**
`plans/reviews/active/rust-interop-certification-14-merge-continuation-evidence.md:53-58` presents a fenced `text` block labelled "Result:" whose second line is `offline package merge smoke self-test: PASS`. That string is not producible: `verification/areas/package_management/tools/check_offline_package_merge_smoke.py:73` prints only `offline package merge smoke ok`, and both manifest cases (`offline-registry-lock-graph`, `offline-registry-lock-graph-self-test`, `manifest.json:27-44`) route through it, so the real transcript is that line twice. `grep -rn "offline package merge smoke self-test"` matches nothing in the repo outside this artifact — and round 11's own record (`review-round-11.md:123`) correctly wrote "offline package merge smoke ok (×2 incl. self-test)". The pass/2-of-2 substance is correct and independently reproduced; only the quoted output is paraphrased where the artifact asserts captured output. This is the same class of defect rounds 8 and 9 blocked on (pluralized lock wait; omitted 4132.029 ms sample). Fix: replace line 55 with the second literal `offline package merge smoke ok`, or annotate the block as a summary rather than captured output.

**L2 — Artifact preamble now understates its own contents.**
Same file, `:9-12`: "This artifact preserves compact reruns for the **three** results whose original console output had no unique durable report." The head commit added a fourth documented result, and the offline-merge-smoke rerun does not fit that description at all — its output was present in the original console; what was missing was its enumeration in the ledger. Fix: update the count and separate the "recovered lost output" sections from the "explicitly preserved coverage step" section, or move the offline-smoke rerun under its own framing.

No other actionable issue: no regression, no inventory/stable/resource inexactness, no performance-policy drift, no missing merge-gate coverage, no tracking inconsistency. Draft status is intentional and not a blocker.

VERDICT: NOT SATISFIED
