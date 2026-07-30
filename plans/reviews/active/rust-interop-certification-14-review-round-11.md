All verification complete. Here is my review.

---

# Exact-Published-Head Review — Rust Interop Runtime/Ecosystem Certification 14 (PR #3083)

**Head reviewed:** `8452e8024eed4ae4126dd1b5960187e531554da0`
**Base:** `origin/main` = `f1c34cf9aaabadda546e670fca190decc580c935` — head is exactly current (`git rev-list --count 8452e8024..origin/main` = `0`; `origin/main` is an ancestor). PR state: OPEN, draft, MERGEABLE, `agent/rust-interop-certification-14` → `main`.

**Diff scope:** 26 files, +1068/−69. Exactly one Rust file changed — `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:104-111,171-178` — a `cfg(test)`-only change that binds `check_package_project(&pristine_entrypoint)` to a local and prints `{pristine_errors:#?}` on failure. Behaviour-identical. No compiler, frontend, diagnostic, LSP, codegen, or runtime path is touched.

## 1. Closeout inventory — independently recomputed

Recomputed directly from the canonical JSON, not from prose:

| Claim (issue lines 1748-1758) | Independent result |
| --- | --- |
| 36 fixture-matrix rows / 36 compatibility rows / 36 schema-v2 manifests | `fixtures=36`; `comp rows 36`; `find … -name fixture.json \| wc -l` = 36; `grep -l '"schema_version": 2'` = 36 ✅ |
| 72 passing, 0 planned | `{'passing': 72}` — no other status exists ✅ |
| 21 `supported`, 14 `supported-through-bridge`, 1 `unsupported-by-design` | `{'supported': 21, 'supported-through-bridge': 14, 'unsupported-by-design': 1}`; `categories=3` ✅ |
| 13 `cargo-probe`, 4 `compiler-diagnostic`, 10 `contract-only`, 9 `runtime-observed` | exact match ✅ |
| 44 crate aliases, 61 package examples, 18 scenario examples | `check_fixture_matrix.py` → `crates=44 package_examples=61 scenario_examples=18` ✅ |
| 36 structured stable claims | `stable support claims ok: claims=36` ✅ |
| no stale `future_owner` | `[r['id'] for r in rows if 'future_owner' in r]` → `[]`; claims → `[]` ✅ |

The `future-owned-by-separate-phase` category remains *declared* (matrix line 9) but has zero members — exactly as the closeout states.

## 2. Validators, self-tests, and backstops — all rerun at this head

```
check_fixture_matrix.py --self-test        → cases=233
check_compatibility_matrix.py --self-test  → cases=7   (rows=36 fixture_rows=36 categories=3)
check_tiers.py --self-test                 → cases=6   (tiers=5 fixtures=36)
check_stable_support_claims.py --self-test → cases=33  (claims=36)
check_stale_drafts.py --self-test          → cases=20
check_sysroot_stdlib_resource_certification_gate.py         → PASS (surfaces=1, future_runtime_rows=0)
check_sysroot_stdlib_resource_certification_gate.py --self-test → PASS
```

Full area: `sifr_verify areas run --area rust_interop` → **`variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`**, with `stable-candidate` registered as a real suite in `verification/areas/rust_interop/manifest.json` (5 suites × 2 cases). The 233 total confirms the +4 adversarial mutations (229 → 233) the closeout attributes to the new trust grants.

Guardrails: `check_file_size_guardrails.py` PASS (3019 files); `check_hir_maintainability_guardrails.py` PASS; `cargo fmt --check` clean; `git diff --check origin/main...8452e8024` clean.

## 3. Build-script trust grants — necessary, sufficient, and minimal

The round-1 over-declaration finding is **closed and independently proven minimal**. I resolved each direct dependency in the registry cache and checked for `build.rs`:

| Fixture direct dep | ships build.rs | granted |
| --- | --- | --- |
| `serde 1.0.228` / `serde_json 1.0.149` / `thiserror 2.0.18` | yes | yes |
| `bytes 1.11.1` / `indexmap 2.14.0` | no | no |
| `zerocopy 0.8.48` | yes | yes |
| `bytemuck 1.25.2` / `memmap2 0.9.11` | no | no |

So `rust-build-scripts = ["serde","serde_json","thiserror"]` (`bridge_type_roundtrip/sifr.toml:19`) and `= ["zerocopy"]` (`crate_backed_view_runtime/sifr.toml:18`) are exactly the four direct build-script-bearing dependencies — no transitive proc macros, nothing spare. Adversarial enforcement is present: four new drift mutations in `_scenario_checks.py:163-183` and `_scenario_zero_copy.py:178-184`, each asserting the exact missing-entry diagnostic, plus the `_require_trust_targets`/`_require_trust` requirements at `_scenario_checks.py:497-504` and `_scenario_zero_copy.py:104-111`. Both READMEs document the grants.

## 4. Stale `future-owned` prose — closed (rounds 5/6)

The three compatibility-matrix notes now delegate rather than defer, and each delegate genuinely certifies the named behaviour: `opaque_resource_core` → `opaque_resource_matrix` (cert 5), `async_runtime_core` → `async_runtime_reqwest` (cert 4, which does execute cancellation/drop per its checklist line 500-501), `callback_subscription_core` → `callback_subscription_ecosystem` (cert 6). `sifr_sysroot_and_stdlib_architecture.md:915-916` uses matching completed wording.

A repo-wide sweep (`git grep future-owned`, excluding reviews/archive) leaves only category *definitions* and forward-looking policy (`rust_interop_architecture.md:1519`, `README.md:15,171`, `39_rust_interop.md:57,288,296`, `40_…md:52,550,859`, `check_sysroot_…py`). `sifr_sysroot_and_stdlib_architecture.md:908-909` is conditional policy ("Surfaces *still marked* future-owned … must not be claimed"), vacuously satisfied. The public-doc mentions (`docs/releases/0.1.0.mdx:52`, `docs/releases/compatibility.mdx:38`, `plans/releases/candidates/0.1.0/release-notes.md:75`) under-claim rather than over-claim. No false present-tense deferral survives.

## 5. Stable-candidate claim scope

`check_stable_support_claims.py` binds `docs/rust-interop.mdx` (PUBLIC_PATH, line 17) and rejects stable-claim markers in other `.md`/`.mdx` under the docs root (self-test cases at lines 758-786). All 36 claims and 33 adversarial mutations pass. The narrow contract-only rows are preserved and not runtime-promoted:

```
async_runtime_core         | supported              | contract-only
callback_subscription_core | supported              | contract-only
zero_copy_bytes            | supported              | contract-only
zero_copy_view_matrix      | supported              | contract-only
arrow_record_batch         | supported-through-bridge | contract-only
tensor_dlpack_bridge       | supported-through-bridge | contract-only
advanced_data_matrix       | supported-through-bridge | contract-only
```

## 6. PERF-HOST exception — correctly supported

The governed exception is properly grounded. `adhoc_performance_budget_host_variance.md:1-16` establishes the follow-up and its prohibition ("must not change performance baselines or add waivers merely to make one host pass"); DoD lines 146-152 require five consecutive controlled runs, a seeded-regression rejection, and documented conditions — which matches the closeout's re-homing sentence at issue lines 1786-1794 and the `certification_7`/`certification_8` retrospective at lines 156 and 996-1000.

**Round 9/10's exact cross-branch mapping is verified against the live artifact.** `/tmp/sifr-class-field-item2-performance-pass14.log:31-36` reports five regressions across four cases:

```
check-project-004-project-graph                       measured=3313.437 threshold=1357.524
check-single-file-001-arithmetic                      measured=4612.439 threshold=1334.139
diagnostic-non-regression-002-json-diagnostic-schema  measured=4132.029 threshold=1335.954
lsp-query-003-diagnostics median_ms                   measured=17.918  threshold=5.91
lsp-query-003-diagnostics p95_ms                      measured=22.939  threshold=10.933
```

The ledger entry (`adhoc_performance_budget_host_variance.md:112-124`) now reads "3313.437 ms, 4612.439 ms, 4132.029 ms, and 17.918 ms median plus 22.939 ms p95" — a correct four-cases-plus-separate-p95 mapping, structurally parallel to the closeout sentence it sits beside. Round 9's finding is closed exactly.

Round 8's singular lock-wait correction is present (issue lines 1895-1896: "a Cargo package-cache lock wait"). No baseline, threshold, waiver, or profile selection is touched anywhere in the diff (`git diff --name-only` matches nothing under baseline/budget/threshold except the follow-up issue's prose).

## 7. Merge-gate coverage behind the fail-fast stop

I derived the authoritative merge plan (`scripts/run_all_tests.sh --profile merge --emit-plan`). `performance` is area index 13 of 23; the nine areas after it are `distribution_release`, `sysroot_release`, `project_workspace`, `package_management`, `stdlib_parity`, `regression`×2, `fuzz_property`, `ecosystem_compatibility`, plus full-corpus E2E (`fixture_count: 678`, `full-corpus`). The continuation record (issue lines 1897-1914) accounts for all of them except `package_management/offline-merge-smoke` — see finding L2. `stdlib_parity/module-merge-check` is separately covered by the create-PR profile, which passed at `e05dd42e9`.

Durable continuation evidence (`rust-interop-certification-14-merge-continuation-evidence.md`) is checked in with the uniquely-named project result JSON + SHA-256, CLI `6/6` in 164.08 s, driver `65/65` in 1794.34 s, and E2E `678 passed, 0 failed` at signature `5e45a6a7b96f2688` / `cache_hits=178/178`. Round 9's unsupported `801.93` figure is gone. Round 8 finding 3 is closed.

## 8. Review-ledger traceability and tracking state

All ten rounds are checked in, non-stub, and each carries an explicit verdict — NOT SATISFIED ×2 (1,2), SATISFIED (3,4), NOT SATISFIED ×2 (5,6), SATISFIED (7), NOT SATISFIED ×2 (8,9), SATISFIED (10) — and every one is recorded in the ledger at lines 1795-1857 with its finding and closure. Round 6's traceability finding is closed by lines 1814-1829. The carried `certification-13-review-round-10.md` is a full 161-line artifact (`## Verdict: **SATISFIED**`), not a truncated tail — the failure mode cert-12 round 3 caught does not recur.

Tracking state is honest and consistent: `certification_14` row is `in progress` (line 163); the final checklist box (line 1738) is correctly **unchecked**; roadmap reads "Track A closeout in progress"; Phase 39/40 edits describe Track A in past tense with correct certification attributions; the PR body accurately describes draft status pending this review. The last two commits (`a344d1187`, `8452e8024`) are doc-only ledger additions; `8452e8024`'s self-referential bullet ("first published head was `a344d1187…`") is factually accurate.

## Non-blocking findings

**L1 — Over-declaration guard asymmetry (`verification/areas/rust_interop/checks/_scenario_checks.py:805-816`).** `_require_trust_targets` only enforces a *superset* (it reports `missing` entries), whereas the zero-copy path uses `_require_trust` (`_scenario_async_reqwest.py:211-223`), which enforces exact equality. Empirically confirmed:

```
bridge_type over-declared (+syn,+quote,+proc-macro2) -> failures: []
zero_copy  over-declared (+syn)                      -> failures: ["... must equal ['zerocopy']"]
```

So the precise defect class round 1 found — an over-declared trust grant — is mechanically prevented for `crate_backed_view_runtime` but not for `bridge_type_roundtrip`. The current state is minimal and correct, and the necessity mutations are sound; this is only a missing regression guard on a fixture manifest, not a product trust path. Worth an exact-equality assertion in a follow-up.

**L2 — One merge-profile step absent from the continuation enumeration.** `package_management/offline-merge-smoke` runs after `performance` in the merge plan and appears nowhere in the closeout ledger, which nonetheless asserts "proving that no functional merge step hidden behind the fail-fast comparison is omitted" (line 1908-1909). I closed this myself at the exact head:

```
uv run --project verification --locked python -m sifr_verify areas run \
  --area package_management --suite offline-merge-smoke
→ offline package merge smoke ok  (×2 incl. self-test)
→ package management verification ok: variants=2, failures=0, blocking_failures=0
```

The step passes, so the coverage claim is substantively true and only the enumeration is one item short. Adding the line would make the record match its own claim.

## Assessment

The closeout contract holds under independent derivation: every inventory number recomputes from the canonical data, every validator and self-test passes at this head, the resource-completion backstop accepts zero deferrals while retaining its supported stdlib-core invariant, the four build-script trust grants are provably the minimal necessary set, the stable-claim surface is bound to exact matrix execution scope, no stale future-owned deferral survives, the historical stdlib handoffs name rows that all exist, and the performance retrospective is re-homed without touching a single baseline, threshold, or waiver. All findings from rounds 1-10 are closed and reproduce as closed. The `PERF-HOST` exception is correctly grounded in the governing follow-up and decisively supported by a same-host unrelated-branch control that fails the identical four cases 2.4×-3.4× worse. No functional merge-gate coverage is missing.

The draft status is the only thing standing between this state and merge, and it is intentional. The two findings above are low-severity record/regression-guard improvements that do not affect correctness, safety, or any claim's truth; neither warrants blocking. **Ready to mark ready for review and merge.**

VERDICT: SATISFIED
