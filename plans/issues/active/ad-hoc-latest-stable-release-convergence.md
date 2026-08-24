# Ad Hoc Phase: Latest Stable Release Convergence

Status: active on 2026-08-24. Items 0-5 are complete. Item 6 GitHub Actions is
next.

## Objective

Move every maintained Sifr toolchain, direct dependency, fork, CI action, and
editor integration to its latest stable release. Complete each compatibility
unit in sequence, keep the repository buildable between items, and close only
after a fresh registry audit finds no stale maintained surface.

## Latest-Stable Policy

- Use the newest non-prerelease release published by the component's official
  toolchain channel, registry, or upstream release feed.
- Recheck the official source when an item starts. The versions below are the
  audited baseline from 2026-08-23, not permission to install a stale version
  if a newer stable release appears before that item starts.
- Do not preserve an older version, compatibility lane, legacy API, version
  fallback, or parallel old path. Migrate the canonical path and delete the
  superseded surface.
- Adopt relevant new stable features and APIs during each upgrade when they
  simplify or strengthen the canonical design. A version-only change is not
  sufficient when the new stable release makes obsolete code unnecessary.
- Breaking changes are allowed. Update all maintained callers, fixtures,
  generated artifacts, and documentation instead of adding shims.
- Do not hand-update transitive Cargo or Python packages. Regenerate them from
  the updated direct graph and review the lock/vendor diff.
- A compatibility unit may contain dependencies that cannot compile in
  isolation. Inside a unit, apply its packages one at a time and validate after
  each step. Merge only the coherent final unit.
- The final closure item repeats the complete official-source audit. Any new
  stable release discovered there becomes a new item before closure.

## Item Loop

Every item follows this sequence:

1. Start from the current `origin/main` merge SHA on a new `codex/` branch.
2. Recheck the item's official stable versions and record the result.
3. Change only the active item's owned surfaces.
4. Run focused validation and the file-size guardrail.
5. Open one draft implementation PR.
6. Request one read-only Claude Opus review of the exact base and candidate
   SHAs. The prompt includes changed paths, scope, acceptance criteria, and
   focused validation evidence.
7. Apply all valid blocking findings in one batch. At most one remediation
   review is allowed. A new mechanism defect found on review two is recorded as
   a later item; there is no third review.
8. If compiler inputs changed, run one create-PR gate and one merge gate on the
   same final, Opus-approved candidate SHA. Do not rerun either gate. If
   compiler inputs did not change, do not run either Sifr gate.
9. Merge the implementation PR only when its exact evidence is satisfied.
10. Merge a record-only PR that updates this document with the implementation
    PR, base/candidate/merge SHAs, validation, review comment, deferrals, and
    exact next action. Do not externally review or run Sifr gates for that
    record-only update.
11. Start the next item immediately from the resulting `origin/main`.

For this phase, compiler inputs are first-party Rust source, generated-runtime
or code-generation templates, stdlib implementation/declarations, compiler
fixtures/snapshots, workspace Cargo manifests, Cargo lockfiles, and `vendor/`.
Workflow-only, planning-only, Python-environment-only, documentation-only, and
editor-submodule-pointer-only changes do not trigger the Sifr create-PR or
merge gates unless they also change one of those compiler inputs.

Review evidence is posted outside the reviewed Git tree, keyed by candidate
SHA, normally as a PR comment. A failed or incomplete Opus request is retried
at most twice with fresh temporary directories and never counts as approval.

## Audited Baseline

### Toolchains and forks

| Surface | Repository baseline | 2026-08-23 stable target |
| --- | --- | --- |
| Rust compiler | `rust-version = "1.93"`; no pinned toolchain file | Rust 1.98.0 |
| Rust edition | 2021 | 2024 |
| Python primary lane | `>=3.11,<3.14` | Python 3.14.7 only |
| PyO3 | 0.29.0 | 0.29.2 |
| Ruff fork | `sifr/0.15.12-maintenance` | upstream Ruff 0.16.4 plus replayed Sifr changes |
| uv | minimum 0.9.28 | exact 0.12.5 |
| Node release workflows | 22 | 24.19.0 LTS |

Rust 1.93 is a declared floor, not a reproducible compiler selection. Item 1
must choose and enforce the latest-only policy with a pinned toolchain file and
make CI/local use the same compiler. The project does not retain an older MSRV
lane under this phase.

### Rust direct dependencies

The audit found 64 stale direct dependency surfaces across the root workspace,
compiler crates, generated-runtime catalog, and Rust-interop catalog:

| Compatibility unit | Baseline | Stable target |
| --- | --- | --- |
| Utility/foundation train | `aho-corasick 1.1.4`, `annotate-snippets 0.12.15`, `anyhow 1.0.102`, `bitflags 2.11.1`, `blake3 1.8.5`, `bstr 1.12.1`, `cc 1.2.63`, `chrono 0.4.44`, `clap 4.6.1`, `cookie 0.18.1`, `crc32fast 1.5.0`, `cxx 1.0.198`, `globset 0.4.18`, `ignore 0.4.25`, `indexmap 2.12.1/2.14.0`, `insta 1.47.2`, `is-macro 0.3.7`, `libc 0.2.178`, `md5 0.8.0`, `memchr 2.8.0`, `proc-macro2 1.0.106`, `rand 0.10.1`, `regex 1.12.3`, `rust_decimal 1.41.0`, `rustc-hash 2.1.2`, `schemars 1.2.1`, `serde/serde_derive 1.0.228`, `serde_json 1.0.149`, `tempfile 3.23.0`, `thiserror 2.0.18`, `toml 1.1.2`, `uuid 1.23.1`, `zerocopy 0.8.48` | `1.1.5`, `0.12.16`, `1.0.104`, `2.13.1`, `1.8.7`, `1.13.1`, `1.4.4`, `0.4.45`, `4.6.6`, `0.18.2`, `1.5.1`, `1.0.199`, `0.4.20`, `0.4.33`, `2.14.0`, `1.48.0`, `0.3.8`, `0.2.189`, `0.8.1`, `2.8.3`, `1.0.107`, `0.10.2`, `1.13.1`, `1.42.1`, `2.1.3`, `1.2.2`, `1.0.229`, `1.0.151`, `3.27.0`, `2.0.20`, `1.1.4+spec-1.1.0`, `1.25.0`, `0.8.56` respectively |
| Async foundation | `bytes 1.11.1`, `futures 0.3.33`, `tokio 1.52.3` | `1.12.1`, `0.3.34`, `1.53.1` |
| HTTP stack | `http 1.4.1/1.4.2`, `http-body 1.0.1`, `http-body-util 0.1.3`, `h2 0.4.14`, `hyper 1.10.1` | `1.5.0`, `1.1.0`, `0.1.5`, `0.4.18`, `1.11.0` |
| TLS stack | `rustls =0.23.35`, `rcgen 0.14.8` | `=0.23.43`, `0.14.9` |
| ICU family | `icu_collator 2.2.0`, `icu_datetime 2.2.0`, `icu_decimal 2.2.0`, `icu_locale 2.2.0`, `icu_plurals 2.2.0` | `2.3.1`, `2.3.0`, `2.3.0`, `2.3.1`, `2.3.0` |
| SHA-2 consolidation | workspace/catalog 0.10.9 plus 0.11 alias | one canonical 0.11.0 dependency |
| Base64 | 0.22.1 | 0.23.1 with an explicit safe feature policy |
| Exact integer | `num-bigint 0.4.6` | 0.5.1 |
| Rust syntax renderer | `syn 2.0.117`, `prettyplease 0.2.37` | `syn 3.0.3`, `prettyplease 0.3.0` together |
| Language server transport | `lsp-server 0.7.8` | 0.10.0 |
| HTTP client | `reqwest 0.12.28` | 0.13.4 and canonical `rustls` feature |
| SQLite | `rusqlite 0.39.0` | 0.40.2 |
| SQLx | 0.8.6 | 0.9.0 with split runtime/TLS features |
| Iterator utilities | 0.14.0 | 0.15.0 |
| Analytical query stack | `arrow 58.3.0`, `datafusion 54.1.0` | `arrow 59.2.0`, `datafusion 55.0.0` together |
| Dataframes | `polars 0.54.4` | 0.55.2 |
| Rust Redis client | 1.4.1 | 1.6.0 |

The other 43 audited direct crates were current. Closure rechecks them; a new
release adds a new item rather than being silently absorbed into an unrelated
unit.

Every Rust dependency item owns all affected direct manifests, the root and
relevant fixture lockfiles, generated dependency snapshots, exact-version
certification fixtures, and regenerated `vendor/` content. A broad unconstrained
`cargo update` is forbidden.

### Python direct dependencies

| Compatibility unit | Baseline | Stable target |
| --- | --- | --- |
| Minor train | `alembic 1.18.4`, `boto3 1.43.33`, `certifi 2026.6.17`, `polars 1.41.2`, `schwifty 2026.3.0`, `sqlalchemy 2.0.51`, `torch 2.12.1` | `1.19.1`, `1.43.78`, `2026.7.22`, `1.43.2`, `2026.7.3`, `2.0.52`, `2.13.0` |
| Crypto ABI | `cffi 1.17.1`, `cryptography 46.0.0` | `cffi 2.1.1`, then `cryptography 50.0.0` |
| Web framework | `fastapi 0.138.0`, `starlette 0.52.1` | `fastapi 0.141.1`, then `starlette 1.6.0` |
| Redis services | `redis 6.4.0`, `fakeredis 2.36.2`, `hiredis 3.4.0`, `testcontainers 4.13.3` | `8.1.0`, `2.37.1`, `3.4.1`, `4.15.0` |
| Numeric/dataframe | `numpy 2.4.6/2.5.1`, `pandas 2.3.3` | NumPy 2.5.2 and Pandas 3.0.5 on Python 3.14 |
| Arrow Python | 22.0.0 | 25.0.1 |
| Kafka Python | 2.3.2 | 3.0.11 |
| Packaging/build | `packaging 25.0`, unbounded Hatchling | `packaging 26.3`, pinned Hatchling 1.32.0 |

The seven current direct packages are still rechecked at closure. Python items
own all maintained first-party `uv.lock` files affected by their marker ranges
and the Python 3.14 evidence. The repository currently has seven such locks;
the eighth lock found by a repository-wide search belongs to vendored PyO3 and
is upstream-owned third-party content.

### CI, actions, and editor

| Surface | Baseline | Stable target |
| --- | --- | --- |
| `actions/checkout` | v4/mixed SHAs | v7.0.1 exact SHA |
| `actions/setup-node` | v4 | v7.0.0 exact SHA |
| `actions/upload-artifact` | v4 | v7.0.1 exact SHA |
| `actions/download-artifact` | v4 | v8.0.1 exact SHA |
| `astral-sh/setup-uv` | v5 | v10.0.1 exact SHA |
| `dtolnay/rust-toolchain` | floating stable/mixed SHA | exact selection consistent with the pinned Rust toolchain |
| `@types/node` | 22.10 line | latest Node 24-compatible line |
| `@types/vscode` | 1.91 | 1.134.0 |
| TypeScript | 5.7.3 | 7.0.2 |
| VS Code engine | 1.91 | 1.134.0 |
| Mint CLI | floating `@latest` | exact latest-stable pin captured when the item starts |

The top-level submodule audit found every tracked remote branch current. Ruff
is the only fork migration item. The editor package update must merge in
`sifr-vscode`, then merge the pointer in `editor-integrations`, then merge the
root pointer and consumer evidence.

## Ordered Items

Only the first incomplete row may be active.

| Item | State | Scope | Acceptance criteria |
| --- | --- | --- | --- |
| 0 | complete | Phase and inventory lock | This active record and roadmap entry merge after exact-SHA Opus satisfaction; all maintained surfaces, compatibility units, gate rules, and closure rules are owned. |
| 1 | complete | Rust 1.98 toolchain | Local and CI compiler selection is reproducibly 1.98; the latest-only policy replaces the 1.93 floor; edition remains 2021. |
| 2 | complete | Rust edition 2024 | Every maintained manifest/template uses edition 2024; `gen` and other reserved syntax are correctly emitted/escaped; generated Rust compiles. |
| 3 | complete | uv 0.12 | uv and setup policy are current; all affected locks are reproducible under the new resolver. |
| 4 | complete | Python 3.14 and PyO3 | Python 3.14.7 is the only maintained Python lane, PyO3 is current, and older-lane configuration and evidence are removed. |
| 5 | complete | Node 24 LTS | Release/tooling workflows use the latest Node 24 LTS and compatible npm behavior. |
| 6 | pending | GitHub Actions | All maintained third-party actions use reviewed latest-stable immutable SHAs and workflow contract tests pass. |
| 7 | pending | Ruff 0.16.4 fork | Sifr changes are replayed on the latest Ruff stable base; fork, gitlink, ownership, parser/formatter/linter evidence, and snapshots agree. |
| 8 | pending | Rust utility/foundation train | Each listed utility dependency is advanced sequentially; all direct declarations converge and generated/vendor evidence agrees. |
| 9 | pending | Rust async foundation | Bytes, Futures, and Tokio converge with runtime/concurrency validation. |
| 10 | pending | Rust HTTP stack | HTTP, body, H2, and Hyper converge with network/HTTP validation. |
| 11 | pending | Rust TLS stack | Rustls and rcgen converge with certificate, TLS, and provider validation. |
| 12 | pending | ICU family | All five ICU4X crates converge together and text/i18n behavior passes. |
| 13 | pending | SHA-2 consolidation | One SHA-2 0.11 dependency remains and all digest evidence passes. |
| 14 | pending | Base64 0.23 | Base64 is current without an unapproved unsafe default feature and parity/error tests pass. |
| 15 | pending | Num BigInt 0.5 | Exact integer behavior, serialization, limits, and generated code pass. |
| 16 | pending | Syn 3 and Prettyplease 0.3 | The single syntax-AST compatibility unit is current and code generation/SQLx scanning passes. |
| 17 | pending | LSP Server 0.10 | Response handling and editor protocol smoke tests pass. |
| 18 | pending | Reqwest 0.13 | Canonical features/provider selection and HTTP client loopback behavior pass. |
| 19 | pending | Rusqlite 0.40 | SQLite interop and exact catalog/fixture locks pass. |
| 20 | pending | SQLx 0.9 | Runtime/TLS features and checked/offline query contracts pass. |
| 21 | pending | Itertools 0.15 | Iterator compilation and parity pass before DataFusion consumes this line. |
| 22 | pending | Arrow 59 and DataFusion 55 | The coupled analytical stack, Rust/Python bridge fixtures, and locks pass. |
| 23 | pending | Polars 0.55 | Rust dataframe fixtures and exact catalog evidence pass. |
| 24 | pending | Rust Redis 1.6 and graph reconciliation | Redis passes and an official-registry check confirms every maintained Rust direct declaration is current. |
| 25 | pending | Python minor train | Listed non-coupled Python releases advance sequentially and both environment lanes resolve. |
| 26 | pending | CFFI 2 and Cryptography 50 | CFFI advances first; cryptography then advances; ABI, certificate, and error paths pass. |
| 27 | pending | FastAPI and Starlette | FastAPI advances first; Starlette then advances; web bridge fixtures pass. |
| 28 | pending | Python Redis services | Redis advances before its fake/client/container companions; compiled live-service certification passes or records only the pre-approved structured Docker skip. |
| 29 | pending | NumPy and Pandas | Marker-split NumPy remains current for both Python floors and Pandas 3 behavior passes. |
| 30 | pending | PyArrow 25 | Both Python lanes and affine Arrow transfer/certification pass. |
| 31 | pending | Kafka Python 3 | Kafka bridge and compiled service-client evidence pass. |
| 32 | pending | Packaging and Hatchling | Packaging is current, Hatchling is explicitly pinned, and builds/locks are reproducible. |
| 33 | pending | VS Code extension toolchain | Node types, VS Code types/engine, TypeScript, package locks, VSIX qualification, and the three-repository pointer chain merge in order. |
| 34 | pending | Mint exact pin | Documentation tooling uses a tested exact latest-stable Mint release and documentation checks pass. |
| 35 | pending | Final registry audit and phase closure | Official-source audit finds no stale maintained surface; all item records are complete; one exact-SHA whole-phase Opus review is satisfied; closure docs merge and the record is archived. |

### Item 0 record

State: complete

PR: [#3489](https://github.com/sifr-lang/sifr/pull/3489)

Base SHA: `3b6a5a2d64a443860ac0166d8a78bee5ac99f209`

Candidate SHA: `213df45177d610ad6f4e6e40974d922d0ae90d08`

Merge SHA: `873ddd3534e73ac533d6de1241ae4313b112d621`

Changed paths: this active phase record and its roadmap registration.

Validation: `git diff --check` passed, the active record path resolved, and
`python3 scripts/check_file_size_guardrails.py` passed across 3,232 files with
the 900-line first-party source limit. Only Markdown planning files changed,
so the user-authorized Sifr create-PR and merge gates did not apply.

Review evidence: the one exact-candidate Claude Opus review returned
`SATISFIED` with no blocking findings. The response is retained in the
[#3489 review comment](https://github.com/sifr-lang/sifr/pull/3489#issuecomment-5385394818).

Deferred follow-up: Opus suggested making the Rust surface count mechanically
derivable and naming the `dtolnay/rust-toolchain` action owner more explicitly.
The already-locked Item 24 and Item 35 graph audits own count reconciliation;
Item 6 owns maintained third-party action SHAs. Neither suggestion identified a
new mechanism defect or changed Item 0 acceptance.

Next action: implement Item 1 Rust 1.98 toolchain convergence from the Item 0
record merge on `origin/main`.

### Item 1 record

State: complete

PR: [#3491](https://github.com/sifr-lang/sifr/pull/3491)

Base SHA: `4aac2860681a04eb66a1cbba64d901e893bc71b8`

Candidate SHA: `cf9702fabbf9230a13daaeff3e88609aacb0f73d`

Merge SHA: `445e03a6a1458d675055bc198b61da11f66f3321`

Changed paths: the root toolchain and workspace manifest, three release and
validation workflows, the supported-platform policy, architecture and
dependency-audit records, and Rust 1.98 Clippy/API migrations in runtime,
type-system, lowering, code-generation, lint, and package sources. The edition
remained 2021 and `Cargo.lock` did not change.

Stable-source result: the official Rust channel reported Rust 1.98.0, released
on 2026-08-20. `rust-toolchain.toml`, the workspace `rust-version`, the six
supported target declarations, and the six `dtolnay/rust-toolchain` selections
now resolve exactly Rust 1.98.0 with Clippy and rustfmt.

Focused validation: exact Rust, Cargo, Clippy, and rustfmt probes passed;
workspace metadata, check, format, Clippy, six affected-package test suites,
the runtime-platform support matrix, distribution-release checks,
maintainability checks, and the file-size guardrail passed. Added UTF-16
trailing-byte coverage passed with the Rust 1.98 `as_chunks` migration.

Gate evidence: the sole create-PR gate attempt identified that generated
projects outside the repository inherited the host rustup default 1.94.0. The
candidate did not change. After the host stable default advanced to 1.98.0,
all ten affected Python-interop variants passed directly. The sole merge gate
then passed on the unchanged candidate SHA, including 698/698 E2E fixtures.
The cold run took 6,508.96 seconds and exceeded only the advisory warm-cache
time budget; it exited zero with no test failure. Full evidence is retained in
the [#3491 final validation comment](https://github.com/sifr-lang/sifr/pull/3491#issuecomment-5386064589).

Review evidence: the one exact-candidate Claude Opus review returned
`SATISFIED` with no blocking findings. The response is retained in the
[#3491 review comment](https://github.com/sifr-lang/sifr/pull/3491#issuecomment-5385475087).
The candidate did not change after review, so no remediation review applied.

Deferred follow-up: Item 6 owns replacing the temporary mutable
`dtolnay/rust-toolchain@1.98.0` selections with reviewed immutable action SHAs.
Opus also noted a cosmetic `sort_by_key` allocation and the pre-existing
nightly sanitizer lane; neither is an Item 1 mechanism defect.

Next action: implement Item 2 Rust edition 2024 convergence from the Item 1
record merge on `origin/main`.

### Item 2 record

State: complete

PR: [#3493](https://github.com/sifr-lang/sifr/pull/3493)

Base SHA: `8fe5328f0c6e19d31a9029fccc1d3596e7b70d2d`

Candidate SHA: `d0fab13a90dede0c2cbc2290203f24fe91326513`

Merge SHA: `1110f85684fc561b44b63c8d927a9a2316d7699c`

Changed paths: every maintained Cargo manifest, generated-project template,
Rust probe, and Rust fixture moved to Edition 2024 and resolver 3. Rust source
was migrated for the new edition, including escaped `gen` identifiers and
edition-sensitive semantics. The LeetCode nested repository migrated first and
the root gitlink then advanced. `Cargo.lock` did not change.

Forward-only result: sysroot validation now requires resolver 3 exactly. The
Edition 2024 process-environment safety change exposed an unsound mutable API,
so public `sifr.env.setenv` and `unsetenv`, private `env_set` and `env_unset`,
their compiler/runtime paths, fixtures, demos, inventories, and documentation
were deleted. Environment access is read-only; no unsafe wrapper, legacy shim,
compatibility implementation, or fallback was added.

Focused validation: workspace all-target/all-feature checking, Edition and
resolver inventories, generated-manifest probes, codegen and driver ownership
tests, read-only environment tests, four environment E2E fixtures, three native
demos, parity and no-pre-v1 audits, formatting, maintainability checks, and the
file-size guardrail passed. The create-PR gate exposed two package test modules
over their stricter 420-line limit; they were split by archive/command-planning
and inventory/dynamic-import responsibility without changing production code.
The resulting package suites passed.

Gate evidence: the sole final create-PR gate passed on the candidate SHA,
including 143/143 create-PR E2E fixtures. The sole merge gate passed on the
same SHA, including every blocking verification area, all crate suites, and
698/698 merge E2E fixtures. The deliberately cold merge run followed the
required private-target cleanup, took 6,797.57 seconds, and reported only the
non-blocking warm-time and group-skew advisories. Evidence is retained in the
[#3493 final validation comment](https://github.com/sifr-lang/sifr/pull/3493#issuecomment-5387964099).

Review evidence: the final forward-only exact-SHA Claude Opus review returned
`SATISFIED` with no blocking findings for candidate
`833124c367f712814471a57cf6f1e623632f71c9`; see the
[#3493 review comment](https://github.com/sifr-lang/sifr/pull/3493#issuecomment-5387069404).
The only later change split oversized test modules. The one permitted
remediation review returned `SATISFIED` for the final candidate; see the
[#3493 remediation comment](https://github.com/sifr-lang/sifr/pull/3493#issuecomment-5387215942).

Deferred follow-up: Item 35 owns a negative resolver-2/absent-resolver sysroot
test, reconciliation of stale historical environment-mutation descriptions,
explicit ambient-missing environment fixture setup, and replacement of one
vacuous parity assertion. Opus classified these as non-blocking and found no
new mechanism defect on the remediation review. A crate-wide test-module
layout unification remains optional cosmetic work outside this phase.

Next action: implement Item 3 uv 0.12 convergence from the Item 2 record merge
on `origin/main`.

### Item 3 record

State: complete

PR: [#3495](https://github.com/sifr-lang/sifr/pull/3495)

Base SHA: `bcf23bd476297661529ec24f4e81cba8c9042a24`

Candidate SHA: `f25cc8a5190904a28637de75ef81e0211807592c`

Merge SHA: `e1888408e1ab343c28e62f3547d804236d914f74`

Changed paths: all seven maintained first-party uv project manifests, the
local validation gate, the local-first workflow, verification documentation,
and the distribution-release current-policy fixture. Every project now
requires uv 0.12.5 exactly. All three workflow uses select that version from
the canonical verification manifest, pin setup-uv v10.0.1 by immutable SHA,
and verify the official Linux x86_64 release checksum.

Stable-source result: the official uv release feed reported 0.12.5 as the
latest stable release and the setup-uv release feed reported v10.0.1 at
`20cfd1bf945f4377ade1205e4dbc17946fc9a30d`. The downloaded uv archive matched
the official SHA-256 independently. A repository-wide audit found seven
maintained project/lock pairs; `vendor/pyo3/uv.lock` is the only additional
lock and remained untouched as upstream-owned content.

Focused validation: all seven locks were regenerated sequentially with uv
0.12.5 without dependency upgrades and remained byte-for-byte unchanged. All
seven passed `uv lock --check --offline`. Exact project/action/checksum counts,
Bash syntax, local gate plan emission, runner self-tests, profile and area
checks, the uv-aware doctor, Python-interop environment/self-tests, diff
checks, maintainability checks, and the file-size guardrail passed. The
distribution-release representative suite passed 56/56 after remediation.

Review evidence: the initial exact-SHA review returned `SATISFIED` with no
blocking finding. The one permitted remediation added explicit uv artifact
checksums, tightened exact-pin parsing, and updated the maintained current
fixture. The final exact-SHA remediation review also returned `SATISFIED` with
no blocking finding. Both results are retained in the
[#3495 review comment](https://github.com/sifr-lang/sifr/pull/3495#issuecomment-5388085894).

Gate evidence: no compiler input changed, so the phase rules prohibited the
Sifr create-PR and merge gates for this item.

Deferred follow-up: the second review identified a new non-blocking mechanism
gap: the seven project pins, three `version-file` settings, and three
platform-specific checksums are not yet governed by one automated invariant.
Item 35 owns adding and exercising that invariant, including a clear failure
when a future runner platform has no matching checksum. Historical uv version
evidence remains intentionally unchanged.

Next action: implement Item 4 Python 3.14-only and PyO3 convergence from the
Item 3 record merge on `origin/main`.

### Item 4 record

State: complete

PR: [#3497](https://github.com/sifr-lang/sifr/pull/3497)

Base SHA: `5cbf633e2a54b710669d2a8ee05660a619d89f2b`

Candidate SHA: `a91bc52b5beea93c28c331ae6b6506265d83e77e`

Merge SHA: `a4723c2f6e0d10bbb516e8b2b01a817c20f69bfe`

Changed paths: all six maintained first-party Python project/lock pairs,
Python-interop policy and runtime suites, CPython differential policy, PyO3
compiler/runtime/package paths, the root Cargo graph, the three checked-in
PyO3 vendor crates, generated binding evidence, workflows, fixtures, and
Python-interop documentation. A fixture-inventory module keeps the maintained
runner below the 900-line source limit.

Stable-source result: the official Python release feed reported CPython 3.14.7
and the official crates.io graph reported PyO3 0.29.2. Every maintained Python
project and lock now selects exactly CPython 3.14.7. The old `cpython311`
environment and named runtime lanes were deleted in favor of canonical
buffer, Arrow, and DLPack runtime suites. PyO3, `pyo3-ffi`, and
`pyo3-build-config` are vendored exactly at 0.29.2 with independently verified
crate archives and checksums.

Forward-only result: the environment and differential checks reject any
interpreter other than GIL-enabled CPython 3.14.7. TensorFlow 2.21.0 publishes
no stable CPython 3.14 wheel, so its maintained dependency, bridge,
certification, fixtures, and documentation were deleted. No older Python lane,
free-threaded interpreter, TensorFlow fallback, compatibility shim, or legacy
runtime name remains. Touched Python sources also adopted Ruff 0.16.4 fixes;
the fork-wide Ruff migration remains owned by Item 7.

Focused validation: all six locks passed exact CPython 3.14.7 offline checks;
the Python-interop runtime and example suites, CPython differential checks,
PyO3 runtime/package/driver/CLI tests, distribution-release representative
suite, workspace Clippy and formatting, verification self-tests, profile and
area checks, doctor, maintainability checks, diff checks, and the file-size
guardrail passed.

Review evidence: the initial exact-SHA Claude Opus review returned `SATISFIED`
for candidate `2fd344776aae2364da6b0f2730e7d5bc74b3e3a1` with no blocking finding and
three valid follow-ups were applied. The one permitted remediation review of
`0c2ccec2a92924d0a2089ebacf6f4444d9139da2` found only two stale documentation
claims about example execution, not a new mechanism defect. Those claims were
corrected in the final candidate. The phase's two-review cap prohibited a
third review for that documentation-only correction. Both review results and
the final correction are retained in the
[#3497 review comment](https://github.com/sifr-lang/sifr/pull/3497#issuecomment-5389268108).

Gate evidence: required private-target cleanup made the first create-PR run a
cold-cache run; it timed out only because `runtime-platform` took 207.29
seconds against its 120-second warm budget while all completed correctness
checks passed. The repository policy prohibits using that run as
host-sensitive performance evidence, so the user explicitly authorized one
warm create-PR rerun. That run passed on the final candidate with
`runtime-platform` at 24.4 seconds, 19/19 Python-interop cases, and 143/143 E2E
fixtures. The sole merge gate then passed on the same SHA, including 25/25
Python-interop cases, 76/76 generated builds, 1,140/1,140 codegen tests, and
698/698 E2E fixtures. Both long runs exceeded only advisory warm-time budgets;
full reports are retained in the
[#3497 final validation comment](https://github.com/sifr-lang/sifr/pull/3497#issuecomment-5389268108).

Deferred follow-up: historical performance negative seeds retain their actual
Python 3.13.1 evidence and are not rewritten as if they ran on 3.14.7. Item 35
owns the final historical-evidence reconciliation. The remaining maintained
Ruff fork and its full parser/formatter/linter migration remain owned by Item
7; Item 4 introduced no parallel old path for either surface.

Next action: implement Item 5 Node 24 LTS convergence from the Item 4 record
merge on `origin/main`.

### Item 5 record

State: complete

PR: [#3499](https://github.com/sifr-lang/sifr/pull/3499)

Base SHA: `f4da1f0dc308a4b243b7aa6ae52f826431ccc2d8`

Candidate SHA: `e9225ac561fc27a65c8cd8d5f4619b8e8ab3df06`

Merge SHA: `c68af27e89d269e6d1a7aef7b1a1dad78f3c6936`

Nested merges: [sifr-vscode #13](https://github.com/sifr-lang/sifr-vscode/pull/13)
merged leaf candidate `09ff33c69928f4d7be3ccf108576354981c1b1d1` as
`7bf12ce026dcbe13679211a06caa76a61e84760a`;
[editor-integrations #11](https://github.com/sifr-lang/editor-integrations/pull/11)
merged pointer candidate `a174b94f40dff84348229bc54d0c9e8d2ddf1dce` as
`b42360d0bbc99c45f625db67ff9a1cb4afdfeaa1`. The root then advanced only the
resolved editor-integrations pointer.

Changed paths: the extension's exact Node selector, package and lock metadata,
CI workflow, and developer documentation; both editor pointers; the root
qualification and publication workflows; current distribution/editor
documentation and demo; the extension validation command; and executable
Node, qualification, and publication workflow contracts.

Stable-source result: the official Node distribution index reported
v24.19.0, released on 2026-08-03 as Krypton LTS, with bundled npm 11.17.0. The
official Darwin arm64 archive SHA-256 was independently verified before the
exact runtime was used. The extension `.node-version` is now the one canonical
selector consumed by extension CI, stable qualification, and protected stable
publication.

Forward-only result: npm 11 `devEngines`, exact `engines`, and the exact
`packageManager` selection reject any other Node or npm development toolchain.
All maintained installs use `npm ci --ignore-scripts --include=dev`, so release
tooling consumes the complete locked development graph without dependency
lifecycle scripts. No Node 22 selector, mutable Node major, compatibility lane,
fallback, or parallel local selector remains in maintained operational paths.

Focused validation: the official Node archive checksum and exact Node/npm
probes passed. Exact npm lock regeneration and clean installation changed no
dependency version. Extension lint, typecheck, unit tests, extension smoke,
and VSIX packaging passed. A negative probe proved ambient Node 24.16.0 and
npm 11.13.0 fail with `EBADDEVENGINES`. The Node toolchain, release
qualification, protected publication, YAML, developer-tooling, Bash, diff,
and file-size checks passed. The complete distribution-release representative
suite passed 57/57.

Review evidence: the one exact-SHA Claude Opus review returned `SATISFIED` for
the final candidate with no blocking findings. It independently confirmed all
three workflow consumers, publication checkout ordering, automatic contract
discovery, exact metadata agreement, the sequential pointer chain, and the
absence of maintained Node 22 documentation. The response is retained in the
[#3499 review comment](https://github.com/sifr-lang/sifr/pull/3499#issuecomment-5389377400).
The candidate did not change after review, so no remediation review applied.

Gate evidence: only workflows, verification tooling, documentation, a demo,
package metadata, and editor gitlinks changed. No compiler input changed, so
the phase rules prohibited the Sifr create-PR and merge gates.

Deferred follow-up: Item 6 owns immutable current `actions/setup-node`
selections. Item 33 owns the Node 24 type declarations and the extension's
three existing high-severity dependency advisories. Item 35 owns final
reconciliation of the explicit Node/npm bundled-major invariant and clearer
diagnostics when the demo uses a mismatched ambient toolchain or the nested
extension checkout is absent. Opus classified each as non-blocking; none is an
Item 5 mechanism defect.

Next action: implement Item 6 GitHub Actions convergence from the Item 5 record
merge on `origin/main`.

## Validation Ownership

- Planning, record, and documentation-only items: `git diff --check`, link/path
  checks applicable to changed records, and the first-party file-size guard.
- Toolchain/edition and Rust dependency items: focused crate/area tests, format,
  Clippy, maintainability/file-size guardrails, then the exact final-SHA
  create-PR and merge gates.
- Ruff: fork-native parser/AST/formatter tests plus Sifr syntax, formatter,
  linter, ownership, and developer-tooling areas before the Sifr gates.
- Python/uv: lock checks, environment probes, Python interop static suites, the
  Python 3.14 lane, and live suites owned by the package item. They do not
  trigger Sifr gates unless compiler inputs also change.
- Workflow/action/Node items: workflow structure, distribution-release,
  publication, artifact recovery, and release qualification contracts.
- Editor: nested compile/lint/unit/package/VSIX tests, editor release
  qualification, then coordinated pointer validation in each owner repository.
- Before any long Cargo gate, check free disk and private target size. Clean
  only this worktree's unused private target if it exceeds 20 GiB.

## Closure Contract

The phase closes only when:

- every ordered item is merged and recorded;
- the final official registry/channel/submodule/action audit finds no stale
  maintained direct surface;
- all Cargo and uv locks are generated by the selected current tools;
- vendored content and dependency/capability snapshots match the final graph;
- no historical evidence was rewritten as if it had used a newer version;
- no compatibility fallback, unowned failure, or hidden deferred mechanism
  remains;
- the last compiler-changing candidate has exactly one passing create-PR gate
  and one passing merge gate on its exact approved SHA; and
- the Item 35 whole-phase exact-SHA Opus review returns `SATISFIED` with no
  blocking finding.

## Current Handoff

Current state: Items 0-5 are complete. Item 5 merged in PR #3499 as
`c68af27e89d269e6d1a7aef7b1a1dad78f3c6936` after the leaf and intermediate
editor pointer PRs merged in order. Node 24.19.0 and bundled npm 11.17.0 are
exactly selected and enforced across extension CI, stable qualification, and
protected publication. The exact final candidate has Opus satisfaction and
changed no compiler input, so no Sifr gate applied.

Next action: merge this record-only update, then start Item 6 GitHub Actions
convergence from the resulting `origin/main`.
