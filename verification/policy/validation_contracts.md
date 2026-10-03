# Validation stage and evidence contracts

Version 1 is prospective policy for the active
[validation execution plan](../../plans/issues/active/ad-hoc-validation-contracts-and-resource-aware-execution.md).
The canonical [stage policy](validation_contracts.json) references existing
profiles, manifests, owners and coverage matrices. It does not replace them.

```bash
uv run --project verification --locked python -m sifr_verify contracts check
uv run --project verification --locked python -m sifr_verify contracts plan --stage merge
```

`check` runs the existing strict guarantee/surface readiness and assignment
checks. `plan` expands selected suite/case IDs, owners, network modes, deadlines,
resource classes and E2E selected/total IDs. Its state is `selected` and explicitly
`not-executed`; an adapter case can expand into a larger runtime corpus, whose
execution must be demonstrated by the area result. Full policy over a representative
corpus does not mean every corpus entry ran.

## Boundaries and outcomes

| Stage | Acceptance and enforcement |
|---|---|
| create-pr | Core/representative feedback and relevant specialist work; selected cases must genuinely pass. |
| merge | Complete canonical merge inventory on the actual candidate; trusted required aggregate and repository protection must agree. |
| nightly | Explicit commit and broader hardening; serious reproduced defects become owned regressions that block affected delivery. |
| release | Actual release-candidate source qualification; no automatic main-push release qualification. |
| artifact-qualification | Run hash-verified installed-package/editor consumers; promote the qualified bytes. Source compilation is insufficient. |
| python-interop-live | Explicit real-service work with declared service/dependency versions; offline fixtures cannot satisfy its claim. |
| cloud | The live merge correctness inventory under shared-VM execution policy; report performance independently and require it wherever the performance contract says so. |

The cloud contract's correctness selection is derived from `merge`. Adoption of
PR #4259's execution route, per-stage scheduling, CI aggregate and external branch
protection are separate implementation items; a plan does not prove enforcement.

Functional outcomes are PASS/FAIL. Performance outcomes are PASS/REGRESSION/
INCONCLUSIVE, with required inconclusive/unavailable evidence blocking acceptance.
Infrastructure failures carry timeout, cancellation, OOM, ENOSPC, admission or
unavailability classifications. Performance admission must not prevent functional
execution. Stage and resource changes are prospective, and never reclassify older
failed or incomplete runs.

## Evidence identity and correctness checkpoints

`execution_identity.py` binds the observed commit, the existing full relevant-input
inventory (including fixtures, locks, submodules, selectors, policies and tested
prose), exact commands/selection, interpreter/dependency bytes, Cargo configuration,
versioned tools, platform, hashed runtime settings, local environment presence,
declared service versions, consumed artifacts, producer and claim-dependent
resource identity. Unknown dependencies invalidate reuse conservatively.

`execution_evidence.py` distinguishes selected, validated, compiled, executed,
passed, failed, skipped, blocked and infrastructure failure. Each selected claim
has an explicit required execution kind. A compile or manifest check cannot satisfy
a runtime claim. A complete receipt requires every selected ID exactly once,
actual positive execution counts and passing outcomes without infrastructure
failure. Partial correctness checkpoints can retain completed work but cannot
satisfy final acceptance. Consumers independently recompute expected identity and
verify consumed/retained hashes before accepting evidence; do not pass a copied
old key as the current identity.

The producer must match the independently trusted expected producer. A self-claimed
producer or digest does not establish CI trust; protected workflow/application
provenance is part of the later aggregate/custody enforcement. Ordinary local
receipts cannot become trusted release evidence by changing a JSON label.

Receipts are published exclusively and never overwrite failures. Freshness starts
at completion, so a long capture does not spend the post-completion validity window.
This protocol qualifies correctness only. It cannot combine interrupted paired
performance captures, adapt pair counts, cherry-pick passing cases, or retry an
unchanged failed performance run until green. Shared-cloud v2 remains independently
owned by the existing performance implementation with its fixed case/count/budget
contract. Cross-commit correctness reuse needs an explicit relevant-input
equivalence proof; the initial key deliberately rejects different observed commits.

## Compatibility, environments and security ownership

These are the current authorities and compatibility directions; this policy adds
no undocumented compatibility promises. M5 audits the supported version ranges
and executes actual predecessor transitions rather than substituting synthetic
same-source packages.

| Boundary | Owner and version/range authority | Direction and executable coverage |
|---|---|---|
| Source/project manifests and locks | Frontend/package management; current accepted formats and exact Cargo/uv locks | Accept supported declared input formats; reject unknown schemas/locks. Frontend/project/package suites own execution. |
| Persistent compiler/generated caches | Driver/sysroot; schema and producer identity in the [DX architecture](../../internal_docs/compiler_dx_architecture.md) | Same compatible schema/inputs may reuse; schema/input drift invalidates and rebuilds. This is not a promise to migrate arbitrary old cache bytes. Driver cache/corruption/concurrency/restart/GC tests own coverage. |
| Generated artifacts and sysroot | Compiler/codegen and release/distribution; source/toolchain/target/schema identity | Exact compatible toolchain pairing and source/installed equivalence; native/sysroot/generated-code suites own it. |
| Installer/update state | Release/distribution; version/channel/schema rules in governed metadata and install receipts | Registered upgrade/reinstall/rollback directions only; synthetic transitions plus M5's actual published predecessor coverage. |
| Editor/compiler pairing | Developer tooling; `editor_integrations/vscode/package.json` compatibility range | Compiler versions inside the declared editor range; reject incompatible pairings. Editor qualification owns coverage. |
| Python boundary | Python interop; pinned interpreter and declaration capability manifests | Supported interpreter/dependency/ABI combinations; reject unsupported ownership/callback/conversion combinations. Compiled/runtime interop suites own execution. |
| Rust boundary | Rust interop; exact compiler/toolchain/feature/ABI manifests | Registered bridge/schema/ABI combinations only; opaque-resource, panic, callback, native build, lock and zero-copy tests own coverage. |
| OS/architecture support | Runtime/platform; [versioned supported matrix](../areas/runtime_platform/supported_platforms.json) | Preserve existing supported and host-limited distinctions and allowed structured reasons; component support and release-package support are separate. |

OS/libc minimums and native packaging promises must come from qualified artifact
and support policy, not an inferred full Cartesian product or the current VM.
M5 must reconcile these floors with the actual release target/toolchain matrix;
unknown floors cannot be advertised as qualified. Stage plans always embed the
versioned current support matrix, without silently demoting supported targets.

Existing security verification belongs to its current area: distribution
archive/installer integrity and traversal; driver/package/sysroot path/symlink
and storage ownership; process execution/host-tool confinement and resource
exhaustion; Python/Rust ABI and callback boundaries. Preserve these tests before
adding gaps. Untrusted PR code must not access release credentials or mint trusted
qualification evidence. Workflow privilege and provenance enforcement is part
of M3/M5, not established by this policy document alone.

Keep separate claims for report determinism, compiler semantic/output determinism,
cache equivalence and release reproducibility. Normalize only declared volatile
fields; never normalize away failures, selected IDs or coverage differences.
