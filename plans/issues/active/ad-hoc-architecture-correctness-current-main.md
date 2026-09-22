# Architecture correctness and agent workflow: current-main supersession

Status: active; Preparation/F33 pending record delivery
Baseline: main 3db97f2f7e354145224a582018cd2a978593a356 (2026-09-23)
Historical branch: codex/architecture-audit-closure-m12k at 46618f73f1d98ffdb77403202f645c8da8250ff6

## Authority

This is the canonical current-main record for the residual architecture audit. It supersedes the execution order and completion claims in the [unmerged historical phase](https://github.com/sifr-lang/sifr/blob/46618f73f1d98ffdb77403202f645c8da8250ff6/plans/issues/active/ad-hoc-architecture-correctness-and-agent-workflow-closure.md). Preserve its branch, PRs, review receipts, failed gates and raw logs as history. None are current-main integration evidence.

The crosswalk uses F01-F34 in the supplied final recommendations (/home/yaser5/projects/sifr/architecture-closure-inputs-20260923/final_recommendations.md, SHA-256 bb2370e7685700456aba35f8b56b11c3ef2433ab6ccba2a1f7254fb8d70a894f). That report contains pinned source links and evidence limitations. Reported Rust reproductions were not rerun for this record. Each row below has one acceptance owner. Existing-owner rows are handoffs, not duplicate implementation authority. A completion claim needs the owner's merge SHA, exact validation and review evidence. A repaired prerequisite is not a passed dependent qualification.

## Preparation/F33 crosswalk and bounded delivery

| ID | Owner / findings / status | Required acceptance evidence |
| --- | --- | --- |
| P00 | This record; F33; record-only preparation. | One owner, evidence class, bounded acceptance and order for F01-F34 and M1-M13. Documentation structure check. No named Sifr executable case. |
| X01 | Generated Rust owner in [Emitted Rust Excellence](ad-hoc-emitted-rust-excellence.md); F01; urgent, open. | Legal unwrap/expect names in check, emit, build, native, project and test flows; compiler-owned extraction still rejected. Reported reproduction and IR inspection are prior evidence, not current pass. |
| X02 | Existing Emitted Rust owner, including retained Item 12; F02-F04 and codegen part of F32; open after X01. | Contextual Result failures at every public emission boundary, no partial output, origin-aware exhaustive IR/final-source validation, trusted macro/bridge policy, diagnostic and fixture checks. |
| V01 | Compiler/performance qualification owner; F05; open prerequisite. | Compatible reference selected before long work, controlled-clock freshness boundary tests, host/toolchain admission and actual selected reference check. Old Mac capture is expired; Linux reference is host-specific. |
| V02 | Verification runner owner; F06; open. | One Python-interop area ID across selection, receipt, export and required cache paths; mutated suite/cache inputs alter identity and cold classification. |
| V03 | Verification guard owner; F24-F25; open. | Site-level filesystem effects and semantic parser exceptions. Negative tests insert a read in a listed file, an alias, a byte read, a new crate/bin and an aliased parse in mixed test/production source. |
| V04 | Verification process owner; F26-F27; open. | Worker-thread setup, selector and callback errors leave no child; absolute deadlines cover nested work and lock waits; distinguish native exit 124, timeout and cancellation. Preserve useful descendant/terminal behavior. |
| N01 | Driver test orchestration, with existing [DX9-F3](ad-hoc-native-cargo-reuse-followups.md); F15-F16; open. | Supported normal/locked/offline/frozen build/test parity, alternating dependencies in one owned root, same-key concurrency and immutable final output. No implicit Python-test expansion. |
| N02 | Package graph and driver identity owners; F07-F08; open. | Versioned framed persisted identities bind every semantic field; serialization failure cannot publish reusable success. Test absent/empty/unreadable/error distinctions at consumers. |
| N03 | Sysroot/native-context identity owner; F09; open. | Actual build context distinguishes declared variable absent, empty and literal <unset> with a versioned encoding. Prior Python model is not proof of a stale binary. |
| N04 | Driver interop/probe-input owner; F10-F11; open. | Required/optional roots, symlinks, errors and consumed bytes follow one snapshot policy; edit within one process invalidates digest. Direct probe still runs Cargo; DX9-F1 owns unused receipt disposition. |
| N05 | CLI formatter-cache owner; F12; open. | Marker binds implementation, options, source and path, has schema/ownership checks and atomic publication; revision-change and failure tests. |
| N06 | DX.3 storage owner and existing DX9-F5; F13-F14; open. | Complete owner inventory, lease-aware scoped pruning and bounded waits; wedged sibling, abandoned staging and live-child tests. Do not delete another session's target. |
| C01 | Frontend product owner; F17-F18; open. | Deterministic SCC/cycle diagnostics and one product for single/project/package/test retaining SQL, specialization, adapter, flow, exports and spans; same-snapshot CLI/analysis equivalence and measured memory. |
| C02 | Compiler-service boundary owner; F19; open after C01. | Lower-level capabilities and explicit context; negative manifest/import tests include aliases and feature targets. |
| E01 | LSP/package input owner; F20-F21; open before fast-hit reorder. | Registered watcher or explicit unsupported-client revalidation, external generations across manifest/lock/config/bridge/certification/environment, and create/delete/rename/reconnect/storm/multi-root/stale-publication tests. |
| E02 | Analysis/lint owner; F22; open after C01. | Current-revision diagnostics/actions reuse canonical parsed/HIR input; changed fix text gets new snapshot; policy/suppression equivalence. |
| E03 | LSP performance owner; F23; open after E01-E02. | Actual cache deltas in at least 25 modules: cold, unchanged, private/API edit, external change and cancellation. |
| E04 | Separate editor correctness owner; F34; open. | Real type symbols and nonempty hierarchy edges, lowercase names, uppercase non-types, imported bases and edits; otherwise reconcile advertised capability/docs explicitly. |
| H01 | Fuzz/property owner; F28; open. | Real guided execution plus semantic normalization, narrowing, ownership, incremental/full and deterministic-codegen properties; separate build/tool/timeout/findings and minimized seeds. |
| H02 | Typed lowering and unsafe bridge owners, split by subsystem; F29; open. | Semantic dispatch classification, strict decline and ABI/lifetime/alias/ownership/callback runtime contracts. Regex counts are not acceptance. |
| H03 | Maintainability/flow owner; F30; open. | Current normalized ratchets and API/fan-out evidence; flow equivalence and resource measurements before removal. |
| D01 | Documentation/registry owners; F31 and non-codegen F32; open, updated with each structural delivery. | Alias/target-aware maps, current API/path/link checks, negative registry and diagnostic-reference tests; historical receipts preserved. |
| Q01 | Final integration qualifier; all retained criteria; open after required owner merges. | Exact-candidate full merge profile, companion freshness, compatible performance reference and owner handoffs; whole-phase review and separate docs-only closure. |

Rows crossing packages specify a contract handoff; delivery splits at package ownership boundaries. Do not use this record to absorb another active issue.

### Resolved prerequisites and boundaries

- DX9-F7 lock drift: [PR #3903](https://github.com/sifr-lang/sifr/pull/3903), merge 439b5ebd5c6917872f58288453917a2427509192. Same-root alternation and resolution tests passed for that item. Retained Emitted Rust Item 12 remains unqualified.
- Taxonomy [issue #3898](https://github.com/sifr-lang/sifr/issues/3898) closed completed 2026-09-22 and is present on baseline main. The Emitted Rust header's blocker wording is stale; its owner updates that handoff before qualification.
- Solo-maintainer release approval: [PR #3827](https://github.com/sifr-lang/sifr/pull/3827), merge 33639f4ee3b7079ec4da889834cae0d55d763786. The [active-named record](ad-hoc-distinct-release-reviewer-restoration.md) describes permanent supersession. Publication still needs approval of the exact protected run.
- Windows native storage/process behavior stays with [Windows driver portability](ad-hoc-windows-driver-portability.md), not V04/N06.

## Historical milestone and draft disposition

Historical M1-M12 drafts #3553-#3564 remain open on the unmerged stack at this review. Their candidate SHAs, validation and reviews remain historical evidence. Evaluate small deltas against current owners before any selective port; do not merge the stack or claim old tests pass on current main.

| Historical item / PR | Disposition |
| --- | --- |
| M1 #3553 | Cache authority/serialization retained in N02-N04 under current Cargo-before-final-cache design. |
| M2 #3554 | Supported build/test parity in N01; mutable leased family and immutable final snapshot remain distinct. |
| M3 #3555 | Effects/parser guards V03 and lifecycle V04; performance budgets are separate from kill deadlines. |
| M4 #3556 | Current docs/maps D01; Cargo aliases and targets defeat directory-only crate checks. |
| M5 #3557 | Urgent legal-name X01 then structural safety X02 under generated Rust owner. |
| M6 #3558 | Result propagation, no recovery output and executable diagnostics X02. |
| M7 #3559 | Frontend product/cycles C01 with current metadata and memory evidence. |
| M8 #3560 | Lower services C02, editor correctness E01-E02 and measured scale E03. |
| M9 #3561 | Typed method and unsafe contracts H02; no count-only closure. |
| M10 #3562 | Existing framed SHA/storage foundation retained; substantial N02-N06 residuals. No competing primitive/cache CLI. |
| M11 #3563 | Guided/semantic goals H01; deterministic mutation smoke remains useful. |
| M12 #3564 | Ratchets/flow H03, companions and registries D01/Q01. |
| M12A/C/E #3565/#3567/#3569 | Merged into historical M12 branch only. Keep descendant/terminal/cancellation scenarios in V04, not obsolete machinery. |
| M12B #3566 | List fallback regression useful in H02; old failure is not presumed live. |
| M12D #3568 | Old registry mismatch is noncurrent; D01 owns new consistency tests. |
| M12F #3570 | Regenerate current companions in Q01 after current changes; no old generated diff transplant. |
| M12G #3572 | Conditional on current C01/H02 lowering exports. |
| M12H #3573 | Old heading absent on main; retain taxonomy check, retire old failure. |
| M12I #3576 | Live Python-interop prefix defect V02; budgets measured independently. |
| M12J #3577 | Old 120-to-300-second budget is not portable; V04/Q01 qualify current selection/cache state. |
| M12K (no PR) | Controlled-host reference attempt blocked; V01 is early prerequisite. |
| M13 #3571 | Historical branch review is not current-main closure; Q01 owns final integration and whole-phase review. |

Old draft status is historical candidate pending selective port or owner-approved closure, not approved for merge. Closing drafts is a separate repository action. Preserve failed/partial logs without relabeling.

## Order, gates and evidence

1. P00, then urgent X01.
2. Validation prerequisites V01-V04 and D01 guard/registry pieces; runner negative tests precede reliance on later green evidence.
3. Native/cache N01-N06; N01 depends on merged DX9-F7 behavior, N06 uses DX9-F5 ownership, and N02 establishes identities before warm-consumer claims.
4. X02 with generated Rust owner after X01, updating diagnostics/fixtures/materialization together.
5. C01, then C02.
6. E01 before Python fast-hit reorder, E02 after C01, E03 after E01-E02. E04 independently completes before editor closure.
7. H01-H03 and D01 current documentation/registry completion.
8. Q01 exact-candidate integration, owner handoff audit, whole-phase review and closure. Release qualification only on an actual request.

The 2026-09-23 assignment prospectively approves intermediate items with named acceptance tests, focused regressions and scoped Opus review, without per-item create-PR/full merge gates. Q01 runs one full merge profile on final merged work; repair the first in-scope cause, rerun failed/affected checks and the full gate until it passes. The whole-phase closer edits docs only and returns needs-implementation for defects. Other owners retain their own recorded rules.

Use exact crate/suite/case selection, fail fast by default and reuse compatible compiler/metadata/fixtures/Cargo target while running selected assertions. Evidence reuse needs unchanged implementation and validation inputs, not merely the same SHA: record tree, lock/submodule/config, compiler identity, command/selection, host/cache state where material, outcome and raw digest. Preserve failed/blocked/timeouts. Record-only updates need documentation checks, not broad gates or an extra external review after implementation review.

Scoped review names exact base/candidate SHAs, changed paths, scope, criteria and validation; it separates in-scope blockers from follow-ups. A second review with a new mechanism-level defect requires rescope. Merge one item, record PR/SHA/evidence and stop before the next batch. One session owns its worktree, branch, index and temporary paths.

