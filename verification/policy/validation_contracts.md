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
| create-pr | Registered 96-case feedback core (22 SQL cases), 143 E2E fixtures and mandatory guards/toolchain; selected cases must genuinely pass. Direct invocation does not qualify omitted specialists. |
| merge | Complete canonical merge inventory on the actual candidate; trusted required aggregate and repository protection must agree. |
| nightly | Explicit commit and broader hardening; serious reproduced defects become owned regressions that block affected delivery. |
| release | Actual release-candidate source qualification; no automatic main-push release qualification. |
| artifact-qualification | Run hash-verified installed-package/editor consumers; promote the qualified bytes. Source compilation is insufficient. |
| python-interop-live | Explicit real-service work with declared service/dependency versions; offline fixtures cannot satisfy its claim. |
| cloud | The live merge correctness inventory under shared-VM execution policy; report performance independently and require it wherever the performance contract says so. |

The SQL feedback core contains `compiler-components`, `common-sql`, `contracts`,
`dependency-baseline`, `host-tools`, `integrated-qualification` and `mutation`.
Complete `merge`, `nightly`, `release` and inherited cloud coverage retain every
offline SQL specialist suite, including actual clean-build qualification.
The commit-bound `changes run` route selects the whole canonical core only when
every changed path has a reviewed all-consumers-in-core disposition; shared,
specialist, unknown, unavailable or non-content changes select whole `merge`.
No arbitrary subset receipt or separate CI selection protocol is introduced.
The finite prose closure is `README.md`, the HIR/driver maintainability checklists,
and the exact active validation plan; blanket documentation prefixes are not safe.
Existing required native platform jobs remain unchanged. Historical compact
shared-VM costs (jobs=1, private sysroot graph retirement) motivate this prospective
selection change; they do not establish normal local/CI latency. The applicable
route still requires cold/warm preparation-inclusive measurements. Existing
budgets/host scope remain unchanged; no fast-feedback target is claimed yet.

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

### Declared versions and compatibility directions

These values reconcile existing authorities; they do not expand or reduce their
support promises. A declared range, selected dependency and observed passing
execution are separate facts. Exact candidate/source, target, toolchain and raw
result bindings remain required before reporting qualification.

| Boundary and owner | Existing version/range authority | Direction and qualifying evidence |
| --- | --- | --- |
| Stable standalone Darwin artifacts; release/distribution | [Stable compatibility](../../docs/releases/compatibility.mdx) declares stable Sifr **0.1.0** on `aarch64-apple-darwin` and `x86_64-apple-darwin`, **macOS 15.0 minimum**; distribution architecture repeats this public floor | Preserve that stable support promise. Existing stable/native package qualification owns matching-target installation and runtime testing. The current beta1300 nonpublishing predecessor rehearsal executes the registered four-target contract. A newer hosted macOS execution is evidence for its observed environment; it is not relabelled as new exact-15.0 qualification. Exact-15.0 execution is not an additional acceptance requirement established for this plan. |
| Stable standalone GNU/Linux artifacts; release/distribution | Same stable0.1.0 authority declares `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`, **glibc 2.39 minimum** | Preserve the stable floor and matching-target/native transition tests. Record actual available environment evidence without inferring a broader Linux distribution/kernel/musl promise. This reconciliation does not newly certify all versions within the support range and does not introduce a separate minimum-libc runtime gate for beta1300. |
| Runtime/component hosts; runtime/platform | [Supported matrix](../areas/runtime_platform/supported_platforms.json), schema 1, Rust `1.98.1`; four native supported host/target pairs, Windows x64 MSVC/GNU `host-limited` | Supported rows still require execution on matching hosts; only existing Windows structured reasons may skip. Component status does not promise a Windows standalone installer. Missing floor evidence never changes a supported row to host-limited. |
| Compiler/sysroot/generated packages; compiler and release/distribution | [Pinned Rust toolchain](../../rust-toolchain.toml) `1.98.1`; sysroot manifest schema 1; metadata physical format 4 in [DX architecture](../../internal_docs/compiler_dx_architecture.md) | Exact compiler/source/toolchain/target and Cargo lock identity must agree; source/installed equivalence and exact artifact integrity are required. Drift invalidates/rebuilds private cache/metadata; no arbitrary old-cache migration promise. Manifest schema and private metadata format are distinct versions. |
| Installation/update; release/distribution | Installer receipt/channel metadata schema 2. [Published predecessor](../areas/sysroot_release/published_predecessors.json) is actual `0.1.0-beta.16`, source `11581e0630407d397079c032d0cd30fb87f4795e`; current nonpublishing qualification package is `0.1.0-beta.1300` from `package_build.py` | Actual published flat install → exact candidate migration, persisted program before/after, candidate reinstall, failed migration → original published state, failed candidate transaction → previous candidate generation. All eight `published_transition.CASES` required. Stable fixture upgrade/forced downgrade qualification is a separate exact-source protocol, not proof of arbitrary published predecessor compatibility. |
| Editor/compiler; developer tooling | [VS Code manifest](../../editor_integrations/vscode/package.json): extension `sifr.sifr-vscode` **0.2.0**, compiler **`>=0.1.0,<0.2.0`**, VS Code **`^1.137.0`** (stable semver `>=1.137.0,<2.0.0`) | Candidate and eligible stable rollback target must fit compiler range; qualification checks both. Thin client starts `sifr lsp --stdio`. Current build tools are exact Node `26.8.2`, npm `12.0.2`; those build pins are not additional compiler version promises. Do not treat a prerelease native rehearsal as stable editor qualification. |
| Python interpreter/ABI; Python interop and differential owners | [Verification project](../pyproject.toml) and [interop project](../areas/python_interop/pyproject.toml): exact **GIL-enabled CPython 3.14.7**, uv **0.12.10** | Sifr consumes the root application's selected uv environment. No older-interpreter/freethreaded/host-global fallback lane. Differential oracle equality and embedded-Python package/ABI qualification are distinct. |
| Python dependency and capability surface; Python interop | Interop pyproject ranges; exact selected versions, hashes and platform markers in its `uv.lock` and [dependency audit](../areas/python_interop/data/latest_stable_python.json); [declaration capabilities](../areas/python_interop/declaration_capabilities.json) schema 2 | Range admission is not testing of every version. Only selected locked artifacts and active capabilities with required positive/negative/cleanup/live evidence support observed claims. Linux x64 torch is exactly `2.14.0+cpu` from the declared CPU wheel source; other platform selections retain their existing PyPI policy. CPU tensors and synthetic GPU controls do not prove GPU execution. Current HTTP clients are `httpx2`/`httpcore2`; tier metadata is not a live import assertion. |
| Rust interop; Rust interop owner | Rust `1.98.1`; [catalog](../../crates/sifr_rust_interop_catalog/Cargo.toml) has 44 exact optional alias/version/feature pins; [compatibility matrix](../areas/rust_interop/data/rust_interop_compatibility_matrix.json) schema 2 and [stable claims](../areas/rust_interop/data/stable_support_claims.json) schema 1 | Supported and bridge-supported rows require their exact positive and negative evidence. `contract-only`, `cargo-probe`, `compiler-diagnostic` and `runtime-observed` remain distinct; a claim cannot be promoted by its tier or crate availability. Current bridge contract is unversioned; removed `bridge-version` inputs reject, without conversion/fallback. No compatibility across arbitrary Rust toolchains/features/ABI is promised. |

These floor values belong to the existing stable artifact/support authority.
This reconciliation preserves those promises and identifies the associated
tests and historical evidence. Current nonpublishing native results retain
their actual source, target, host and predecessor-transition scope; they do
not become a new stable release or exact-minimum-version qualification claim.
No fresh minimum-host execution requirement is added by this table. Linux
kernel and Windows OS minimums are not specified by the cited authorities;
leave them unspecified rather than inventing values. Preserve all existing
supported/host-limited and standalone/component distinctions.

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
