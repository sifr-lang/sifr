# Ad Hoc Phase: Latest Stable Release Convergence

Status: active on 2026-08-26. Items 0-28 are complete. Item 29 NumPy and Pandas
is next.

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
| HTTP stack | `http 1.4.1/1.4.2`, `http-body 1.0.1`, `http-body-util 0.1.3`, `h2 0.4.14`, `hyper 1.10.1` | `1.5.0`, `1.1.0`, `0.1.5`, `0.4.19`, `1.11.0` |
| TLS stack | `rustls =0.23.35`, `rcgen 0.14.8` | `=0.23.43`, `0.14.9` |
| ICU family | `icu_collator 2.2.0`, `icu_datetime 2.2.0`, `icu_decimal 2.2.0`, `icu_locale 2.2.0`, `icu_plurals 2.2.0` | `2.3.1`, `2.3.0`, `2.3.0`, `2.3.1`, `2.3.0` |
| SHA-2 consolidation | workspace/catalog 0.10.9 plus 0.11 alias | one canonical 0.11.0 dependency |
| Base64 | 0.22.1 | 0.23.1 with an explicit safe feature policy |
| Exact integer | `num-bigint 0.4.6` | 0.5.1 |
| Rust syntax renderer | `syn 2.0.117`, `prettyplease 0.2.37` | `syn 3.0.4`, `prettyplease 0.3.0` together |
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
| Minor train | `alembic 1.18.4`, `boto3 1.43.33`, `certifi 2026.6.17`, `polars 1.41.2`, `schwifty 2026.3.0`, `sqlalchemy 2.0.51`, `torch 2.12.1` | `1.19.1`, `1.43.80`, `2026.7.22`, `1.44.1`, `2026.7.3`, `2.0.52`, `2.13.0` |
| Boto3 service emulator | `localstack/localstack:2.0.1` | `localstack/localstack:4.14.0` at manifest digest `sha256:3ebc37595918b8accb852f8048fef2aff047d465167edd655528065b07bc364a` |
| Crypto ABI | `cffi 1.17.1`, `cryptography 45.0.7` | `cffi 2.1.1`, then `cryptography 50.0.1` |
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
| 6 | complete | GitHub Actions | All maintained third-party actions use reviewed latest-stable immutable SHAs and workflow contract tests pass. |
| 7 | complete | Ruff 0.16.4 fork | Sifr changes are replayed on the latest Ruff stable base; fork, gitlink, ownership, parser/formatter/linter evidence, and snapshots agree. |
| 8 | complete | Rust utility/foundation train | Each listed utility dependency is advanced sequentially; all direct declarations converge and generated/vendor evidence agrees. |
| 9 | complete | Rust async foundation | Bytes, Futures, and Tokio converge with runtime/concurrency validation. |
| 10 | complete | Rust HTTP stack | HTTP, body, H2, and Hyper converge with network/HTTP validation. |
| 11 | complete | Rust TLS stack | Rustls and rcgen converge with certificate, TLS, and provider validation. |
| 12 | complete | ICU family | All five ICU4X crates converge together and text/i18n behavior passes. |
| 13 | complete | SHA-2 consolidation | One SHA-2 0.11 dependency remains and all digest evidence passes. |
| 14 | complete | Base64 0.23 | Base64 is current without an unapproved unsafe default feature and parity/error tests pass. |
| 15 | complete | Num BigInt 0.5 | Exact integer behavior, serialization, limits, and generated code pass. |
| 16 | complete | Syn 3 and Prettyplease 0.3 | The single syntax-AST compatibility unit is current and code generation/SQLx scanning passes. |
| 17 | complete | LSP Server 0.10 | Response handling and editor protocol smoke tests pass. |
| 18 | complete | Reqwest 0.13 | Canonical features/provider selection and HTTP client loopback behavior pass. |
| 19 | complete | Rusqlite 0.40 | SQLite interop and exact catalog/fixture locks pass. |
| 20 | complete | SQLx 0.9 | Runtime/TLS features and checked/offline query contracts pass. |
| 21 | complete | Itertools 0.15 | Iterator compilation and parity pass before DataFusion consumes this line. |
| 22 | complete | Arrow 59 and DataFusion 55 | The coupled analytical stack, Rust/Python bridge fixtures, and locks pass. |
| 23 | complete | Polars 0.55 | Rust dataframe fixtures and exact catalog evidence pass. |
| 24 | complete | Rust Redis 1.6 and graph reconciliation | Redis passes and an official-registry check confirms every maintained Rust direct declaration is current. |
| 25 | complete | Python minor train | Listed non-coupled Python releases advance sequentially and both environment lanes resolve. |
| 26 | complete | CFFI 2 and Cryptography 50 | CFFI advances first; cryptography then advances; ABI, certificate, and error paths pass. |
| 27 | complete | FastAPI and Starlette | FastAPI advances first; Starlette then advances; web bridge fixtures pass. |
| 28 | complete | Python Redis services | Redis advances before its fake/client/container companions; compiled live-service certification passes or records only the pre-approved structured Docker skip. |
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

### Item 6 record

State: complete

PR: [#3501](https://github.com/sifr-lang/sifr/pull/3501)

Base SHA: `e73fd79fae641b96fce0de32046cba2dabd1df04`

Candidate SHA: `c08ac5567d6acb64bbc76c731a6471806bc46b3a`

Merge SHA: `d0d57243c4c371aa1d17d57cb92380c2981a0933`

Nested merges: [sifr-vscode #14](https://github.com/sifr-lang/sifr-vscode/pull/14)
merged leaf candidate `cde0ef12602a19af1cd0805f79e119613f64de99` as
`732bcdc3ae2a494753025710dd138aa23a39b6e4`.
[editor-integrations #12](https://github.com/sifr-lang/editor-integrations/pull/12)
merged pointer candidate `2edb4f9746ea8d11f72380fc10411e694def9aa6` as
`d202b8c60240b6d2897c9deeda59be899bf47e24`. The root then advanced only the
resolved editor-integrations pointer.

Changed paths: all seven root workflow files that use external actions, the
extension CI workflow, both editor pointers, the immutable-action policy and
validator, the distribution case and current workflow contracts, the
submodule ownership guardrail, and the verification README.

Stable-source result: official GitHub releases and tag refs gave these exact
results:

- checkout v7.0.1: `3d3c42e5aac5ba805825da76410c181273ba90b1`.
- setup-node v7.0.0: `820762786026740c76f36085b0efc47a31fe5020`.
- upload-artifact v7.0.1: `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`.
- download-artifact v8.0.1: `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`.
- setup-uv v10.0.1: `20cfd1bf945f4377ade1205e4dbc17946fc9a30d`.
- Rust toolchain 1.98.0: `f8be11a05b1d4f3fcebe6410cc16743212b999b0`.

Forward-only result: all 56 maintained external action references use one of
the six reviewed exact commits and carry the matching release label. The
policy validator rejects mutable refs, stale or unknown actions, label drift,
and download steps without `digest-mismatch: error`. All ten
download-artifact v8 steps explicitly use fail-closed digest validation. No
mutable tag, version branch, stale immutable SHA, compatibility selection, or
parallel old action path remains.

The submodule guardrail now recognizes immutable checkout refs with release
comments. Its negative self-test prevents that comment form from bypassing
validation. Seven formerly skipped checkout steps now initialize recursive
submodules. Ruff 0.16.4 reformatted the touched guardrail source. The
verification README now gives one direct command for the action policy.

Focused validation: GitHub release and ref checks matched all policy entries.
The immutable-action policy and negative self-test passed with six actions and
56 references. The submodule ownership guardrail and negative self-test
passed. Every root and leaf workflow parsed as YAML. Exact download-artifact
metadata confirmed Node 24 and the digest-mismatch error input. Verification
runner self-tests, all profiles, all area manifests, Ruff check and format,
Bash syntax, diff checks, and the file-size guardrail passed. The final-state
distribution-release representative suite passed 58/58.

Review evidence: the one exact-SHA Claude Opus review returned `SATISFIED` for
the final candidate with no blocking findings. It independently confirmed all
56 refs, exact policy agreement, ten fail-closed download steps, complete
maintained-workflow inventory, automatic case discovery, contract literals,
and the sequential pointer chain. The response is retained in the
[#3501 review comment](https://github.com/sifr-lang/sifr/pull/3501#issuecomment-5389488025).
The candidate did not change after review, so no remediation review applied.

Gate evidence: only workflows, workflow verification, documentation, and
editor gitlinks changed. No compiler input changed, so the phase rules
prohibited the Sifr create-PR and merge gates.

Deferred follow-up: Opus confirmed a pre-existing step-splitting defect in the
submodule ownership guardrail. Item 35 owns replacing that parser with a
YAML-based step boundary and defining an explicit policy for evidence-only
checkouts. Item 7 owns the Ruff fork's internal workflow audit while it replays
the maintained fork. Neither finding is an Item 6 regression or omission.

Next action: implement Item 7 Ruff 0.16.4 fork convergence from the Item 6
record merge on `origin/main`.

### Item 7 record

State: complete

PR: [#3503](https://github.com/sifr-lang/sifr/pull/3503)

Base SHA: `99dcc86ac37a4676be2e130f21b9d0913a649429`

Candidate SHA: `dfcbcdef62ee7888483a14308b8b9a9d47bd17d4`

Merge SHA: `dd55520413fc792d1f59eea0a70b374bb9a8cd81`

Fork merge: [sifr-lang/ruff #4](https://github.com/sifr-lang/ruff/pull/4)
merged the `sifr/0.16.4-maintenance` candidate as
`f19957111640fdee8055bfe5b6aa854259344473`. The root gitlink advances to
that exact fork commit.

Changed paths: the Ruff gitlink, fork ownership and version policy, the
parser/AST/formatter/linter migration and fork snapshots, root parser and
formatter consumers, and five driver test/support modules whose parse-tree
boundaries now consume Ruff's `Suite` directly.

Stable-source result: the official Ruff v0.16.4 tag resolves to
`11c76bf48fdac06b2f240cba502eda96da4dce77`. The maintained Sifr patches were
replayed on that base and merged into the fork's single
`sifr/0.16.4-maintenance` branch. No v0.15 branch, compatibility parser path,
AST conversion shim, or fallback remains in maintained root consumers.

Forward-only result: Sifr now uses Ruff's current `Suite` parse root and
current string-annotation AST form. Dependency-graph construction accepts
borrowed statement slices from suites, and source-aware frontend paths avoid
cloning suites back into legacy `Vec<Stmt>` boundaries. All maintained driver
test helpers and maps use `Suite`; the migration does not preserve an old
shape behind an adapter.

Focused validation: the fork passed 251 parser unit tests and 554 parser
fixtures, 56 formatter unit tests with two expected ignores and 377 formatter
fixtures, plus all-target Clippy. The root passed all 19 Python-interop driver
tests, 556 active driver tests with 76 expected slow ignores, workspace
Clippy with warnings denied, formatting, HIR maintainability, diff checks, and
the 3,243-file first-party size guardrail.

Review evidence: the initial exact-SHA Opus review is retained in the
[#3503 review comment](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5389964080).
Its blocking findings were repaired together, and the permitted remediation
review returned `SATISFIED` in the
[#3503 remediation comment](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5390161266).
The first compiler gate then exposed stale `Vec<Stmt>` test-only boundaries
outside the reviewed diff. With the user's explicit exception authorization,
the repaired exact candidate received one final read-only Opus review; it
returned `SATISFIED` with no blocker in the
[#3503 final review comment](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5391225255).

Gate evidence: the first create-PR attempt on superseded candidate
`ca11e75a` failed compilation with 44 `E0308` mismatches and is recorded in
the [failure comment](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5390238802)
and [scope audit](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5390246625).
The user explicitly authorized a replacement budget for the complete
mechanism repair. On exact final candidate
`dfcbcdef62ee7888483a14308b8b9a9d47bd17d4`, every functional create-PR check
passed; its first post-clean performance observation timed out only because a
cold runtime-platform build took 208.645 seconds against the 120-second warm
budget. Repository policy excludes a first cold-cache run as performance
evidence, so the authorized warm observation passed: runtime platform 28/28,
Python interop 19/19, driver 556 active tests, and E2E 143/143. The evidence is
retained in the [cold](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5391306587)
and [warm](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5391457972)
comments.

The single merge gate on the same final candidate passed in 6,793.50 seconds
after the required private-target clean. It passed all areas and crate tests,
76/76 explicit driver generated builds, 1,140/1,140 codegen tests, and
698/698 merge E2E fixtures with report signature `127353b213e16688`.
Installed/source sysroot equivalence passed on Rust 1.98.0. The complete
metrics and report digest are retained in the
[#3503 merge-gate comment](https://github.com/sifr-lang/sifr/pull/3503#issuecomment-5392466081).

Deferred follow-up: Item 8 owns the dependency graph and can remove the
pre-existing suite clone in `graph_cache_and_queries.rs` if its touched
mechanism makes that appropriate. The fork's cosmetic inherited commit title
and the string-annotation arm comment do not affect behavior. Item 35 owns
final confirmation that no stale Ruff branch, version, workflow, or fork
surface remains.

Next action: implement Item 8 Rust utility/foundation train from the Item 7
record merge on `origin/main`.

### Item 8 record

State: complete

PR: [#3505](https://github.com/sifr-lang/sifr/pull/3505)

Base SHA: `e747a20f06584d184d95bac10ee842136eea391c`

Candidate SHA: `53320be497f298800c8f1d9001c22faa7a6acbe4`

Merge SHA: `a57da8f7208bede1dd67849606e21ba3612497e7`

Stable-source result: the train converged to `aho-corasick 1.1.5`,
`annotate-snippets 0.12.16`, `anyhow 1.0.104`, `bitflags 2.13.1`,
`blake3 1.8.7`, `bstr 1.13.1`, `cc 1.4.4`, `chrono 0.4.45`, `clap 4.6.6`,
`cookie 0.18.2`, `crc32fast 1.5.1`, `cxx 1.0.199`, `globset 0.4.20`,
`ignore 0.4.33`, `indexmap 2.14.0`, `insta 1.48.0`, `is-macro 0.3.8`,
`libc 0.2.189`, `md5 0.8.1`, `memchr 2.8.3`, `proc-macro2 1.0.107`,
`rand 0.10.2`, `regex 1.13.1`, `rust_decimal 1.42.1`, `rustc-hash 2.1.3`,
`schemars 1.2.2`, `serde`/`serde_derive 1.0.229`, `serde_json 1.0.151`,
`tempfile 3.27.0`, `thiserror 2.0.20`, `toml 1.1.4+spec-1.1.0`,
`uuid 1.25.0`, and `zerocopy 0.8.56`. `libc 1.0` prereleases were excluded.
Every active direct declaration, catalog literal, maintained fixture marker,
lock identity, and owned vendor package agrees with those stable releases.

Forward-only result: generated-project cache identity now incorporates the
SHA-256 of the live sysroot `Cargo.lock`. Source-tree development therefore
invalidates generated workspaces when the live lock changes even though the
development `sysroot.toml` intentionally contains zero release placeholders.
A missing or unreadable lock fails closed as `MissingAsset`; no compatibility
key, legacy cache path, or fallback was added. The train also adopts the
upstream `cc` trim-path behavior. The new Regex compile-time macro was audited,
but all maintained first-party patterns are dynamic and have no correct
compile-time conversion.

Changed paths: active Cargo manifests and catalogs, root and maintained
fixture lockfiles, generated dependency snapshots and version evidence,
package-owned sysroot vendor crates, stdlib feature trees, Rust interop
documentation/evidence, and the sysroot/driver cache-key implementation and
tests. Vendor replacement stayed selective; package-owned interop graphs were
not absorbed into the sysroot.

Focused validation: locked metadata and workspace checks passed, as did
workspace Clippy with warnings denied, formatting, HIR maintainability, the
3,243-file first-party size guardrail, first-party diff checks, offline direct
vendor and sysroot feature probes, 10/10 Rust interop variants, and the focused
compiler/stdlib/package suites. The full stdlib parity run passed all 411
LeetCode cases and every module/dependency-tree check. Its only demo failure,
`demos/m16_raw_api/src/main.sifr`, is the pre-existing canonical-requirement
defect already owned by Item 13 of the pre-v1 compatibility-removal phase.

Review evidence: the one exact-SHA Opus review returned `SATISFIED` with no
blocking finding in the
[#3505 review comment](https://github.com/sifr-lang/sifr/pull/3505#issuecomment-5393608119).
No remediation review was needed.

Gate evidence: the one create-PR invocation passed every functional check it
reached and exited `124` only when the first post-clean runtime-platform build
took 203.977 seconds against its 120-second warm budget; the cold
`binary_file_io_capability.sifr` build accounted for 159.454 seconds. The one
merge gate on the same exact candidate exited `0`. The same runtime-platform
case then took 3.656 seconds, and all verification areas, all crate tests,
76/76 explicit driver generated builds, and 698/698 merge E2E fixtures passed.
The create-PR and merge reports have SHA-256 digests
`2f3704905d1bdb1a6b2a4a7604cd01ee8912e2cb4c11afcc9c1a5fa4540abd0a`
and `d95e756ed094799314d05641c6bd88e21496216a7a7a3a63515596f17d5e2fb9`
respectively. Exact metrics and the no-rerun disposition are retained in the
[#3505 gate comment](https://github.com/sifr-lang/sifr/pull/3505#issuecomment-5394946937).

Deferred follow-up: the misleading `cxx` probe value marker is already owned
by the active Rust-interop drift-hardening review. Item 9 owns the pre-existing
Futures vendor/root skew when it upgrades the async foundation. Item 35 owns
the final vendor/root completeness audit, the pre-existing core-language
fixture-lock subset gap, and confirmation that unused workspace declarations
such as `annotate-snippets` and `cookie` either have a maintained owner or are
removed. These are pre-existing issues or cleanup suggestions, not Item 8
mechanism defects.

Next action: implement Item 9 Rust async foundation from the Item 8 record
merge on `origin/main`.

### Item 9 record

State: complete

PR: [#3507](https://github.com/sifr-lang/sifr/pull/3507)

Base SHA: `689a886181cc0e132b6d73c2ae99242ae5705826`

Candidate SHA: `1c46c2ddf641b9b407da6f4c78360414c6c0d1c3`

Merge SHA: `d17997a998bea31df41ae509df216b2055769a33`

Stable-source result: the official crates.io release feeds still reported
`bytes 1.12.1`, the complete `futures 0.3.34` family, and `tokio 1.53.1` as
the newest stable releases when Item 9 started. Root and maintained fixture
manifests and locks, generated dependency snapshots, exact-version tests, and
the package-owned sysroot vendor copies now agree with those versions. The
vendor replacement was selective: it updated `bytes`, `futures-channel`,
`futures-core`, `futures-sink`, and `tokio` without absorbing package-owned
interop graphs. All 644 files across the five replaced vendor packages match
their Cargo checksum manifests, and no Cargo extraction marker remains.

Forward-only result: the canonical runtime keeps its intentionally minimal
Tokio feature set. Tokio's new file-descriptor APIs require the unused `fs`
feature, its new runtime scheduling histogram is an unused observability
surface, and its Unix socket address additions have no maintained caller.
Bytes' new `BytesMut` APIs likewise have no first-party `BytesMut` call site.
No speculative feature, compatibility path, legacy pin, or fallback was
added. Existing runtime, task-scope, async code-generation, signal, process,
and HTTP-loopback paths consume the upstream correctness fixes directly.

Changed paths: active workspace, generated-runtime, Rust-interop, and direct
fixture manifests and locks; generated dependency/version evidence; stdlib
feature-tree and audit snapshots; exact Tokio source tests; and the five
package-owned sysroot vendor packages. No first-party compiler algorithm or
runtime API implementation changed.

Focused validation: locked/offline metadata and a generated task-scope project
compiled from the checked-in vendor. Rust interop passed 10/10 variants;
runtime passed 290/290 all-feature tests and 3/3 HTTP loopbacks; focused
process, signal, HTTP, Python, async-codegen, driver, and CLI suites passed.
Workspace Clippy with warnings denied, formatting, HIR maintainability, the
3,243-file first-party size guardrail, submodule ownership, first-party diff
checks, and all vendor checksum files passed. Full stdlib parity passed 411
LeetCode cases and every module, dependency, and audit check. Its only demo
failure was the pre-existing `m16_raw_api` Python-math canonical requirement
already recorded by Item 8 and owned outside this item.

Review evidence: the one exact-SHA Opus review returned `SATISFIED` with no
blocking finding in the
[#3507 review comment](https://github.com/sifr-lang/sifr/pull/3507#issuecomment-5395780334).
No remediation review was needed.

Gate evidence: the one create-PR invocation passed every functional step it
reached and stopped only when the cold runtime-platform area took 202.431
seconds against its 120-second blocking budget; the cold
`binary_file_io_capability.sifr` build took 157.560 seconds. The one merge gate
on the same exact candidate passed all guardrails and its first ten selected
areas, including the warmed runtime-platform area in 24.848 seconds, then
stopped fail-closed in performance control. Three instruction-CV samples for
`formatter-corpus-001-project-check` were 3.1346%, 3.4252%, and 3.0516%
against a 2% stability limit under recorded unrelated desktop CPU pressure.
Five other representative benchmarks and the frontend syntax guardrails
passed. Per the one-shot rule, neither gate was rerun.

The eight merge-profile areas not reached after that host-control stop were
run directly with their canonical locked/offline selections: 146 variants
passed with zero failures across distribution release, sysroot release,
project workspace, package management, stdlib parity, regression,
fuzz/property, and ecosystem compatibility. All 26 canonical full-mode crate
members passed, including both serialized generated-build suites. The
merge-profile E2E suite passed 698/698 fixtures with report signature
`127353b213e16688`. Create-PR, merge, full-crate, and E2E evidence digests and
the no-rerun disposition are retained in the
[#3507 gate comment](https://github.com/sifr-lang/sifr/pull/3507#issuecomment-5397724184).

Deferred follow-up: the new optional Tokio schedule-latency histogram remains
out of the deliberately minimal runtime feature set unless an observability
owner adopts it. `futures-macro 0.3.34` consumes Syn 3, while the existing
workspace still legitimately contains multiple Syn majors; Items 16, 24, and
35 own syntax-stack convergence and final graph reconciliation. Archived
historical network plans retain their contemporaneous Tokio text. None is an
Item 9 mechanism defect.

Next action: implement Item 10 Rust HTTP stack from the Item 9 record merge on
`origin/main`.

### Item 10 record

State: complete

PR: [#3509](https://github.com/sifr-lang/sifr/pull/3509)

Base SHA: `6f678957045af607ab382e9620f0e426f31bfcec`

Candidate SHA: `dc6dda859ef8b2cb2de0ada6346a6559500f40ac`

Merge SHA: `1ece067abd79262dfb5610e508610e76fc3b3670`

Stable-source result: the official crates.io release feeds reported
`http 1.5.0`, `http-body 1.1.0`, `http-body-util 0.1.5`, `h2 0.4.19`, and
`hyper 1.11.0` as the newest stable releases when Item 10 started. The H2
target advanced from the phase baseline's `0.4.18` to the same-day stable
`0.4.19` before implementation. Root and maintained Rust-interop manifests
and locks, generated dependency snapshots, exact-version evidence, and the
package-owned sysroot vendor copies now agree with the rechecked versions.
All 207 non-checksum files across the five replaced vendor packages match
their Cargo checksum manifests, package checksums match the root lock, and no
Cargo extraction marker remains.

Forward-only result: the canonical runtime uses Hyper's H2 automatic
data-frame budget and has a deterministic loopback test that exhausts a
one-frame server budget and observes `ENHANCE_YOUR_CALM` on both ends. The
HTTP/1 regression suite verifies Hyper 1.11's rule that Transfer-Encoding
overrides an earlier Content-Length, and the HTTP registry test consumes the
new `Method::QUERY` constant from HTTP 1.5. The new `http-body` and
`http-body-util` combinators and additive `SizeHint` APIs had no maintained
production caller that they could simplify, so no ceremonial wrapper,
compatibility path, legacy pin, or fallback was added.

Changed paths: the root workspace manifest and lock; the Rust-interop catalog
and four maintained fixture locks; generated HTTP dependency snapshots,
feature-tree evidence, and network traceability records; the canonical HTTP
runtime implementation and tests; and the package-owned `http`, `http-body`,
`http-body-util`, `h2`, and `hyper` vendor packages. Demo-repository gitlinks
and their independently owned locks did not change.

Focused validation: HTTP dependency snapshots passed 8/8; the all-feature
runtime passed 292/292 unit tests and 3/3 HTTP loopbacks; the all-feature
stdlib passed 53/53 tests; seven focused HTTP runtime tests passed; and the
stdlib parity audit passed all ten HTTP fixtures. Rust interop passed both
matrix variants and 237 self-tests. Generated Reqwest loopback, resource
lifecycle, Axum/SQLx backend, and Arrow/DataFusion/Polars scenarios passed
4/4. Workspace Clippy with warnings denied, formatting, HIR maintainability,
the 3,243-file first-party size guardrail, feature-tree equality, vendor
checksums, and first-party diff checks passed. A supplemental non-canonical
all-feature runtime Clippy probe exposed 515 pre-existing broad-feature
diagnostics in generated Unicode, Python ABI, TLS, and existing HTTP
signatures; canonical workspace Clippy passed, and Item 10 did not absorb that
separate backlog.

Review evidence: the initial exact-SHA Opus review returned `SATISFIED` with
no blocking finding in the
[#3509 review comment](https://github.com/sifr-lang/sifr/pull/3509#issuecomment-5398353457).
Its non-blocking suggestion asked the new H2 frame budget to be exercised
directly. The remediation candidate added the deterministic exhaustion test.
The one allowed remediation review found no new mechanism defect and requested
only a one-line traceability correction; the disposition and exact correction
are retained in the
[#3509 remediation comment](https://github.com/sifr-lang/sifr/pull/3509#issuecomment-5398353697).
The final Markdown-only correction did not trigger a forbidden third review.

Gate evidence: the one create-PR invocation passed every functional step it
reached and stopped only when the cold runtime-platform area took 205.464
seconds against its 120-second blocking budget; the cold
`binary_file_io_capability.sifr` build took 160.097 seconds. Before the long
gates, the mandatory private-target cleanup removed 40.7 GiB because the
unused target exceeded the 20 GiB limit. The one merge gate on the same exact
candidate completed with exit 0. Every functional area passed, including
performance 12/12, distribution/release 68/68, sysroot release 2/2, all
workspace crate members, 76/76 generated driver builds, and 698/698 E2E
fixtures with report signature `127353b213e16688`. Its only result was a
non-blocking warm-wall-time advisory after the intentional cold-cache cleanup.
Neither gate was rerun. Exact command results and SHA-256 evidence digests are
retained in the
[#3509 gate comment](https://github.com/sifr-lang/sifr/pull/3509#issuecomment-5399834340).

Deferred follow-up: the pre-existing non-canonical all-feature Clippy backlog
remains with its owning broad-feature surfaces. Item 11 owns the independently
audited Rustls and rcgen upgrade, including TLS provider and certificate
behavior. The second Item 10 review found no new mechanism defect, so no later
corrective item was added.

Next action: implement Item 11 Rust TLS stack from the Item 10 record merge on
`origin/main`.

### Item 11 record

Implementation [PR #3511](https://github.com/sifr-lang/sifr/pull/3511) started
from base `ebace873a977ff8e82bde5d55d06e3865e64b288`. The exact candidate was
`f432ab8d88d44b21bd193bca66d5deaa38ac1102`. It merged as
`8d433b6135c344d3bd0d77e6e825a1ea6ae00cbd`.

The official registry audit confirmed Rustls 0.23.43 and rcgen 0.14.9 as the
latest stable releases. Rustls 0.24 development releases were prereleases and
were not eligible. The root manifest and lock now select these versions.

The package-owned vendor copies are byte-identical to the official crate
archives. All 119 Rustls files and 22 rcgen files match their checksum
manifests. Their package checksums also match the root lock. No extraction
marker remains.

Four independent Rust-interop locks contained Rustls and now select 0.23.43.
Cargo also unified existing Windows target edges in three locks. One lock also
unified the existing tempfile `getrandom` edge. These changes added no package
node, and native scenarios for all four locks passed.

The canonical TLS runtime adopts the stable RFC 9149 ticket-request API. All
three client constructors request two new-session tickets and two resumption
tickets. Both server constructors emit two tickets and limit request-driven
responses to two. Sifr does not expose tickets or key material.

The rcgen 0.14.9 test proves canonical RFC 5280 BasicConstraints DER for
`ExplicitNoCa`. It rejects the old explicit-false encoding. No legacy
provider, fallback trust store, compatibility API, or parallel dependency path
was added.

Changed surfaces:

- The root Cargo manifest and lock.
- The Rustls and rcgen package-owned vendor copies.
- Four maintained Rust-interop locks.
- The TLS runtime policy and its tests.
- Exact dependency certification and the HTTP feature-tree snapshot.
- The network dependency audit and TLS traceability record.

Focused validation passed the sequential Rustls and rcgen tests and checks.
The all-feature runtime passed 295 unit tests and three HTTP loopbacks. The
all-feature stdlib passed 53 tests. The canonical Rust-interop area passed ten
variants. Four generated native lock scenarios also passed.

The public TLS loopback and configuration-error fixtures passed. Workspace
Clippy denied all warnings. Formatting, HIR maintainability, stdlib audit
fixtures, vendor checksums, diff hygiene, and the 3,243-file size guardrail
also passed. `crates/sifr_runtime/src/tls.rs` remains below the limit at 860
lines.

The exact-SHA Opus review returned `SATISFIED` with no blocking finding. The
full review is in the
[#3511 review comment](https://github.com/sifr-lang/sifr/pull/3511#issuecomment-5400449823).
Its seven notes were non-blocking coverage, documentation, and maintenance
suggestions. No remediation review or later corrective item was required.

The one create-PR gate ran after the required private-target cleanup. It passed
every functional step that it reached. Its runtime-platform area passed 28
variants but took 204.735 seconds against a 120-second cold budget. The gate
was not rerun. The exact evidence is in the
[#3511 create-PR comment](https://github.com/sifr-lang/sifr/pull/3511#issuecomment-5400708720).

The one merge gate ran on the same reviewed SHA and passed every blocking
lane. The warm runtime-platform area passed 30 variants in 24.656 seconds.
The gate also passed 1,140 codegen tests and 1,043 lowering tests. One lowering
test had its expected ignore.

All 76 driver generated-build integrations passed. The native E2E suite passed
698 of 698 fixtures with report signature `127353b213e16688`. The final report
contained only advisory wall-time and fixture-group-skew notes. The exact
evidence is in the
[#3511 merge comment](https://github.com/sifr-lang/sifr/pull/3511#issuecomment-5401929969).
Neither gate was rerun.

Next action: implement Item 12 ICU family from this record merge on
`origin/main`.

### Item 12 record

Implementation [PR #3513](https://github.com/sifr-lang/sifr/pull/3513) started
from base `483519ceec4250436dcd69c9d06d31c123930a91`. The final candidate was
`d95f5b665fb33945985b1931132f109091215d4b`. It merged as
`643018175fdbc82412b789cda6a171b793858749`.

The official registry audit confirmed these latest stable direct releases:
ICU Collator 2.3.1, ICU DateTime 2.3.0, ICU Decimal 2.3.0, ICU Locale 2.3.1,
and ICU Plurals 2.3.0. The root manifest, lock, generated dependency
snapshots, and exact-version certification now agree. The complete ICU graph
was regenerated from official archives with Cargo rather than edited by hand.
The 30 changed or added packages and all 1,176 package file checksums match the
official checksum manifests.

The upgrade adopts the ICU Locale 2.3 split: locale fallback code and data now
use their dedicated packages. The runtime tests also certify three changed
stable behaviors: `und` maximization remains `und`, a digit singleton starts a
private-use locale extension, and the Burmese collation regression no longer
panics. No old package path, legacy adapter, version fallback, or parallel ICU
graph remains.

Five maintained Rust-interop fixture locks shared ICU package identities with
the root graph. They now select the same ICU Locale, Provider, and Normalizer
versions. The complete targeted Rust-interop matrix passed 40 fixtures, 11
diagnostics, 44 crates, 61 package examples, 21 scenario examples, and 237
self-tests.

Changed surfaces:

- The root Cargo manifest and lock.
- The regenerated ICU package-owned vendor copies.
- Five maintained Rust-interop locks.
- The locale and collation runtime behavior tests.
- Exact dependency certification and generated dependency snapshots.
- The coverage classification, inventory, and dependency audit records.

Focused validation passed all five direct dependency checks in order. The
all-feature runtime passed 296 tests. The all-feature stdlib passed 53 tests.
The five targeted text and internationalization E2E fixtures passed with
signature `b3333f3bdb8a0884`.

Workspace Clippy denied all warnings. Formatting, HIR maintainability, vendor
archive identity, checksum verification, diff hygiene, and the 3,244-file size
guardrail passed. The broad stdlib parity runner passed 283 of 284 demos,
including `demos/text_i18n`. Its sole failure was the recorded pre-existing
`demos/m16_raw_api` dependency on canonical Python `math`; Item 12 did not
absorb that separately owned issue.

The initial Opus review of exact SHA
`cf22e08a5320999a80ff6b0c40a31549495eb589` returned `SATISFIED` with no
blocking finding. Its non-blocking maintenance notes produced one consolidated
refinement. The evidence is in the
[#3513 initial review comment](https://github.com/sifr-lang/sifr/pull/3513#issuecomment-5402371882).
The one allowed remediation review of exact SHA
`a9ac93764c66666291fe95da2cde9f867ef2d687` also returned `SATISFIED` and found
no new mechanism defect. Its evidence is in the
[#3513 remediation review comment](https://github.com/sifr-lang/sifr/pull/3513#issuecomment-5402416921).
No third review ran.

The one create-PR gate ran on the reviewed candidate. It passed every earlier
blocking guardrail and stopped in the Rust-interop matrix when five maintained
fixture locks exposed their old ICU shared package identities. After those
locks were regenerated, the full targeted matrix passed. The gate was not
rerun. Its exact evidence is in the
[#3513 create-PR comment](https://github.com/sifr-lang/sifr/pull/3513#issuecomment-5402493589).

The one merge gate ran after the fixture-lock correction. It passed every
earlier blocking guardrail and the complete Rust-interop area. It then stopped
because the new exact-version test target lacked coverage classification. The
final mechanical correction classified it as a merge-profile test fixture.
The complete targeted coverage-matrix readiness suite then passed all four
variants with no failure. The merge gate was not rerun. Its final lane report
had no advisory. The exact one-shot evidence and final focused result are in
the [#3513 final gate comment](https://github.com/sifr-lang/sifr/pull/3513#issuecomment-5402534557).

Next action: implement Item 13 SHA-2 consolidation from this record merge on
`origin/main`.

### Item 13 record

Implementation [PR #3515](https://github.com/sifr-lang/sifr/pull/3515) started
from base `524835c29cad0a22b69922bcef2fdfd038118463`. The final candidate was
`b992ace751235b69ade3acd6ff6038de32e0d2e6`. It merged as
`8eb8408d1f19c62ed1439a9172afed54b4d8c8a6`.

The official RustCrypto audit confirmed SHA-2 0.11.0 as the latest stable
release. Version 0.11.1 was still unreleased. The root workspace and generated
runtime catalog now declare one canonical SHA-2 0.11 dependency. The old
version-named alias and every maintained first-party SHA-2 0.10 edge are gone.

The migration adopts the SHA-2 0.11 digest output newtypes. First-party callers
encode digest bytes as explicit lowercase hexadecimal text. SHA-224, SHA-256,
SHA-384, and SHA-512 exact known-answer vectors passed. The shared digest owner
is in `sifr_sysroot`; package-private code retains its private owner.

The root lock and all maintained fixture locks now use the SHA-2 0.11 line for
first-party edges. A new regression check certifies direct declarations and
lock edges. The remaining SHA-2 0.10 lock node belongs only to external SQLx
0.8 and Polars 0.54.4 dependencies. Items 20 and 23 own those upgrades.

The official `hybrid-array` 0.4.14 archive replaced the older vendored copy.
Its archive digest and all 24 vendored file checksums passed. No compatibility
adapter, legacy path, fallback, or parallel first-party SHA-2 dependency was
added.

Changed surfaces:

- The root Cargo manifest and lock.
- The generated Rust-interop dependency catalog and maintained fixture locks.
- Digest runtime, sysroot, package, and compiler callers.
- SHA-2 behavior and dependency-graph certification tests.
- The official `hybrid-array` vendor copy and dependency snapshots.

Focused manifest, stdlib, sysroot, package, structural-identity, CLI,
fixture-lock, and Rust-interop tests passed. The complete targeted Rust-interop
matrix passed 40 fixtures, 11 diagnostics, 44 crates, 61 package examples, 21
scenario examples, and 237 self-tests. Coverage readiness passed all four
variants.

Workspace Clippy denied all warnings. Formatting, HIR maintainability, diff
hygiene, vendor verification, and the first-party file-size guardrail passed.

The initial Opus review of exact SHA
`7e5cb3609dac1efd74f7b2d82e2a028fba298ead` returned `SATISFIED`. Its evidence
is in the [#3515 initial review comment](https://github.com/sifr-lang/sifr/pull/3515#issuecomment-5402953717).
The one allowed remediation review of exact SHA
`24b6369835ec2cab7594370a36e7883acadf606c` also returned `SATISFIED` and found
no new mechanism defect. Its evidence is in the
[#3515 remediation review comment](https://github.com/sifr-lang/sifr/pull/3515#issuecomment-5402987650).
The final mechanical cleanup removed a phantom direct dependency and made the
lock assertion self-contained. Its disposition is in the
[#3515 review disposition comment](https://github.com/sifr-lang/sifr/pull/3515#issuecomment-5402996312).
No third review ran.

The one create-PR gate completed every functional step that it reached. The
cold runtime-platform area passed functionally but took 227.896 seconds against
its 120-second blocking budget. The gate stopped there and was not rerun.

The one merge gate ran on final SHA
`b992ace751235b69ade3acd6ff6038de32e0d2e6` and exited 0. All 35 lane steps
passed. The warmed runtime-platform area completed in 24.251 seconds, and its
slowest case completed in 3.651 seconds. The gate also passed all 76 ignored
generated-build tests, all 1,140 codegen tests, and all 698 E2E fixtures.

The merge gate took 6,459.31 seconds. It reported advisory wall-time and
fixture-group-skew observations. The exact one-shot results are in the
[#3515 final gate comment](https://github.com/sifr-lang/sifr/pull/3515#issuecomment-5404000545).
Neither gate was rerun.

Next action: implement Item 14 Base64 0.23 from this record merge on
`origin/main`.

### Item 14 record

Implementation [PR #3517](https://github.com/sifr-lang/sifr/pull/3517) started
from base `da4228e41d7a3c8c656a190d97b9e8548c013dc4`. The final candidate was
`96e6a72f18be675080fa2ed020d898712ef73a1c`. It merged as
`da9420e807bf6132ae326b5e2fb96a4078fbf022`.

The official audit confirmed Base64 0.23.1 as the latest stable release. The
workspace now selects `std` without the default `simd-unsafe` feature. The
stdlib uses the canonical 0.23 prelude engine constants.

Strict trailing-bit and padding error tests passed. The Base64 API tests also
passed all RFC vectors and URL-safe behavior. No compatibility adapter, legacy
path, fallback, or unsafe Base64 feature was added.

The official Base64 0.23.1 archive replaced the primary vendor package. The
official 0.22.1 package remains as `vendor/base64-0.22.1`. Vendored external
packages still require that release.

Both archive package hashes match the registry. Every recorded vendor file
checksum passed. A new certification derives the required Base64 versions from
first-party and vendored lock edges. It then makes sure that both packages
exist in the vendor tree.

Changed surfaces:

- The root Cargo manifest and lock.
- The Base64 stdlib implementation and behavior tests.
- The Base64 dependency and vendor-closure certification.
- The official Base64 0.23.1 and 0.22.1 vendor packages.
- The coverage-matrix classification for the new certification target.

The manifest suite passed all tests, including all three Base64 certification
tests. The focused generated E2E set passed all 14 fixtures. Coverage readiness
passed all four variants.

Workspace Clippy denied all warnings. The changed certification test also
passed targeted Clippy. Formatting, HIR maintainability, diff hygiene, vendor
verification, and the 3,247-file first-party size guard passed.

The initial Opus review of exact SHA
`e31728a4a91911b0ec9fe7d2609bf9f31151e097` found two blocking omissions. The
primary vendor replacement removed a required 0.22.1 package. The certification
also did not prove full Base64 vendor coverage. The evidence is in the
[#3517 initial review comment](https://github.com/sifr-lang/sifr/pull/3517#issuecomment-5404197232).

The one allowed remediation review of exact SHA
`96e6a72f18be675080fa2ed020d898712ef73a1c` returned `SATISFIED`. It confirmed
both corrections and found no new mechanism defect. The evidence is in the
[#3517 remediation review comment](https://github.com/sifr-lang/sifr/pull/3517#issuecomment-5404304751).
No third review ran.

The one create-PR gate completed every functional step that it reached. The
cold runtime-platform area passed all variants. It took 206.884 seconds against
its 120-second blocking budget, so the gate exited 124. It was not rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. The gate passed both Base64 versions, offline lock-mode builds, and
installed/source sysroot boundary equivalence. It also passed all 1,140 codegen
tests and all 698 E2E fixtures.

The merge gate took 6,522.67 seconds after the required target cleanup. It
reported advisory wall-time and fixture-group-skew observations. The exact
one-shot results are in the
[#3517 gate comment](https://github.com/sifr-lang/sifr/pull/3517#issuecomment-5405327400).
Neither gate was rerun.

Deferred follow-up: Item 35 owns the audit for a generic vendor-closure
guardrail. Item 14 supplies the dependency-specific Base64 closure check.

Next action: implement Item 15 Num BigInt 0.5 from this record merge on
`origin/main`.

### Item 15 record

Implementation [PR #3519](https://github.com/sifr-lang/sifr/pull/3519) started
from base `c0e2a66f4537bd70f36addbb17b0567715fc8ba4`. The final candidate was
`037b24c64c5ac5ac516efbac6d86b3e0c3154a0b`. It merged as
`6301c73867685e2cf9680ecc8aeba1df094478ae`.

The official audit confirmed Num BigInt 0.5.1 as the latest stable release.
The direct Sifr graph now uses that exact release with its explicit `std`
feature. BigDecimal still requires the Num BigInt 0.4 line, so its external
graph uses the latest compatible 0.4.8 release. This is upstream dependency
isolation, not a Sifr compatibility path.

The maintained Sifr implementation now uses the stable `BigInt::ZERO` and
`BigInt::ONE` constants. The idiomatic decimal demo names BigDecimal's integer
type explicitly. No conversion shim, legacy path, fallback, or parallel Sifr
API was added.

Both official registry archives replaced their vendor packages. Their package
hashes match the registry, and every recorded vendor file checksum passed. The
root lock and each tracked first-party fixture lock agree with the split graph.
A new certification checks the first-party Num BigInt selection and the
external BigDecimal edge.

Changed surfaces:

- The root Cargo manifest and lock.
- The generated dependency plan and tracked first-party fixture locks.
- Exact-integer compiler, runtime, stdlib, and demo callers.
- Num BigInt dependency and vendor certification.
- The official Num BigInt 0.5.1 and 0.4.8 vendor packages.
- Dependency feature snapshots and coverage classification.

Focused frontend, lowering, runtime, manifest, stdlib, demo, generated-build,
and dependency-plan tests passed. Num BigInt certification passed all three
tests. The standalone idiomatic decimal demo also compiled with the exact
dependency split.

Workspace Clippy denied all warnings. Formatting, HIR maintainability, diff
hygiene, snapshot equality, advanced lock metadata, coverage readiness, and
the first-party file-size guard passed.

The initial Opus review of exact SHA
`037b24c64c5ac5ac516efbac6d86b3e0c3154a0b` returned `SATISFIED`. Its evidence
is in the [#3519 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3519#issuecomment-5405668574).
No remediation review ran.

The one create-PR gate completed every functional step that it reached. The
cold runtime-platform area passed all variants. It took 204.572 seconds against
its 120-second blocking budget, so the gate stopped after that area. It was not
rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. The gate passed the split Num BigInt graph, full workspace tests, all
1,140 codegen tests, and all 698 E2E fixtures. Its E2E signature was
`127353b213e16688`.

The merge gate took 6,539.20 seconds. It reported advisory wall-time and
fixture-group-skew observations. The exact one-shot results are in the
[#3519 gate comment](https://github.com/sifr-lang/sifr/pull/3519#issuecomment-5406964129).
Neither gate was rerun.

Deferred follow-ups: Item 35 owns three audit improvements. Restrict generic
lock discovery to tracked maintained locks. Require the first-party Num BigInt
lock version to equal 0.5.1, not only to be unique. Add a maintained compile
gate for idiomatic Rust demos.

Next action: implement Item 16 Syn 3 and Prettyplease 0.3 from this record merge
on `origin/main`.

### Item 16 record

Implementation [PR #3521](https://github.com/sifr-lang/sifr/pull/3521) started
from base `fbf09e1b5cf703c5a43484be01bad4399da8cbf4`. The final candidate was
`797d5be4501821adaa52b40aa1fcd2dd4bd2177d`. It merged as
`5d3d474027efc8c66fda66058127756be0eb56f9`.

The official audit confirmed Syn 3.0.4 and Prettyplease 0.3.0 as the latest
stable releases. The target changed from Syn 3.0.3 after its 2026-08-24 stable
release. Maintained Sifr dependencies now use Syn 3.0.4 and Prettyplease 0.3.0.

The migration uses Syn 3 impl modifiers and file frontmatter directly. Impl
deduplication now distinguishes ordinary, default, and negative impls.
Canonical `From` relocation accepts only impls with no modifiers. The SQLx
scanner now certifies the Syn 3.0.4 safe-function syntax in an unsafe extern
block.

External packages still require Syn 2.0.117 and Prettyplease 0.2.37. These
packages use isolated vendor entries. They do not add a Sifr compatibility
path, adapter, fallback, or legacy API.

Changed surfaces:

- The root Cargo manifest and lock.
- The codegen dependency features and Syn 3 callers.
- The SQLx scanner regression tests.
- Eight tracked Rust-interop fixture locks.
- The JSON feature snapshot and coverage classification.
- Exact-version, lock-edge, and vendor certification.
- Official vendor packages for both required dependency lines.

Focused codegen, driver, SQLx, dependency-plan, and certification tests passed.
All eight affected fixture locks passed locked offline Cargo metadata. All four
vendor package hashes and every recorded vendor file checksum matched.

Workspace Clippy denied all warnings. Formatting, HIR maintainability, diff
hygiene, snapshot equality, coverage readiness, and the 3,249-file first-party
size guard passed.

The initial Opus review of exact SHA
`797d5be4501821adaa52b40aa1fcd2dd4bd2177d` returned `SATISFIED`. It found no
blocking finding. The evidence is in the
[#3521 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3521#issuecomment-5407626475).
No remediation review ran.

The one create-PR gate completed every functional step. The cold
runtime-platform area passed all 28 variants. It took 206.898 seconds against
its 120-second blocking budget, so the gate exited 124. It was not rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. The gate passed the split Syn and Prettyplease graph, all 1,142 codegen
tests, all 76 generated-build integrations, and all 698 E2E fixtures. Its E2E
signature was `127353b213e16688`.

The merge gate took 6,446.20 seconds. It reported advisory wall-time,
cache-footprint, and fixture-group-skew observations. The exact one-shot results
are in the
[#3521 gate comment](https://github.com/sifr-lang/sifr/pull/3521#issuecomment-5409218627).
Neither gate was rerun.

Deferred follow-ups: Item 35 owns four audit improvements. Include impl safety
in the deduplication key. Require exclusive first-party Syn and Prettyplease
lock edges. Reject stale versioned vendor directories. Simplify the maintained
lock scan to require a Syn edge in each lock.

Next action: implement Item 17 LSP Server 0.10 from this record merge on
`origin/main`.

### Item 17 record

Implementation [PR #3523](https://github.com/sifr-lang/sifr/pull/3523) started
from base `67f613e1b808f829292570afc563ab692a6ae3e6`. The final candidate was
`42f0dcf15048171376ccdd4124001b1b215dfa09`. It merged as
`2b6d5053becabaa9a05e390b7263c59aac542734`.

The official audit confirmed LSP Server 0.10.0 as the latest stable release.
The registry archive matched checksum
`3ee25a31f2e571e426eef2896179450cafc7e2f5be00d8a93b1c2d21c0ff7656`.
Its source metadata matched upstream release commit
`1b7da4272d8d27c78774f42a6e1ea66a4c1fe984`.

The migration uses the new typed `Result<Value, ResponseError>` response
model. One response constructor now owns success and error serialization.
Shutdown, queue rejection, cancellation, missing work, and normal completion
all use this canonical path. The old response constructors are absent.

Changed surfaces:

- The root Cargo manifest and lock.
- The LSP server response construction and protocol tests.
- Exact-version, lock-edge, archive, source, and vendor certification.
- The official LSP Server 0.10.0 vendor package.
- Coverage classification for the new certification target.

All 77 LSP tests passed. Three dependency certification tests passed. The LSP
protocol, transcript, marker, semantic-editor, and self-test variants passed.
All five coverage variants passed.

Workspace Clippy denied all warnings. Production LSP Clippy and certification
Clippy also passed. Formatting, HIR maintainability, diff hygiene, vendor
checksums, and the 3,250-file first-party size guard passed.

The initial Opus review of exact SHA
`42f0dcf15048171376ccdd4124001b1b215dfa09` returned `SATISFIED`. It found no
blocking finding. The evidence is in the
[#3523 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3523#issuecomment-5409418083).
No remediation review ran.

The one create-PR gate completed every functional step. The cold
runtime-platform area passed all 28 variants. It took 210.102 seconds against
its 120-second blocking budget, so the gate exited 124. It was not rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. The gate passed all 1,142 codegen tests, all 76 generated-build
integrations, and all 698 E2E fixtures. Its E2E signature was
`127353b213e16688`.

The merge gate took 6,662.74 seconds. It reported advisory wall-time,
cache-footprint, and fixture-group-skew observations. The exact one-shot results
are in the
[#3523 gate comment](https://github.com/sifr-lang/sifr/pull/3523#issuecomment-5410886213).
Neither gate was rerun.

Deferred follow-ups: Item 35 owns two protocol assertions. Certify strict
response handling before Sifr adds outbound LSP requests. Add a local assertion
for the request-cancelled protocol code `-32800`.

Next action: implement Item 18 Reqwest 0.13 from this record merge on
`origin/main`.

### Item 18 record

Implementation [PR #3525](https://github.com/sifr-lang/sifr/pull/3525) started
from base `3e66f509b94277a420e784ae4f4a2432c67aabd7`. The final candidate was
`f465070e6432e2c4210d335c95b3f6a4d49534d0`. It merged as
`ce11458ae2b1d76b9c2ff5f4cd691cde94619281`.

The HTTP demo [PR #3](https://github.com/sifr-lang/sifr-demo-http/pull/3)
started from `4a9128427fb4f87bf4fbc8f499a00e96eb24a021`. Its reviewed candidate was
`2aef5c21c843b86a5cf4094cef24e54a639ce8ab`. It merged as
`61b94722c1b2a66dd022522a0f373d88dbec3b8b`.

The application demo
[PR #3](https://github.com/sifr-lang/sifr-demo-app/pull/3) started from
`3a9e2ef3bb648125f9be7cc538722be537cbb7f0`. Its reviewed candidate was
`d611cba90be1ba20d043ce8f14aa5feb27641855`. It merged as
`a7c342a18f5166b4e3a433d302274685cfedc232`.

Both nested repositories used merge commits. Their main branches contain the
reviewed candidates. The root gitlinks still name those exact candidates.

The official audit confirmed Reqwest 0.13.4 as the latest stable release. The
registry archive matched checksum
`219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3`.
Its source metadata matched upstream release commit
`11489b34eda6d32b15ad4033e62beba2ee401350`.

Reqwest 0.13 removed the old `rustls-tls` feature. Maintained declarations now
use its canonical `rustls` and `json` features without defaults.

The regenerated graph uses AWS-LC RS 1.18.0 and AWS-LC Sys 0.44.0. Native-link
trust now names the exact `aws_lc_0_44_0_crypto` library.

Changed surfaces:

- Direct manifests, catalogs, locks, fixtures, and both maintained demos.
- Reqwest, AWS-LC, and pkg-config vendor packages with registry checksums.
- Generated dependency snapshots and package demo digests.
- Exact version, feature, provider, vendor, lock, and native-link certification.
- Network, HTTP, TLS, Rust-interop, coverage, and phase documentation.

Three exact dependency certification tests passed. All ten Rust-interop area
variants passed, and its matrix self-test passed 237 tests.

All four package-management variants passed. Three Reqwest runtime tests and
two opaque-runtime tests passed. Both demos passed locked checks.

The driver library passed 557 tests with 76 expected ignored tests. Formatting,
Clippy, HIR maintainability, links, checksums, and file-size checks passed.

The initial Opus review of exact SHA
`f465070e6432e2c4210d335c95b3f6a4d49534d0` returned `SATISFIED`. It found no
blocking finding. The evidence is in the
[#3525 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3525#issuecomment-5411848269).
No remediation review ran.

The one create-PR gate completed every functional step. The cold
runtime-platform area passed all 28 variants.

That area took 226.293 seconds against its 120-second blocking budget. The gate
exited nonzero and was not rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. All 698 E2E fixtures passed with signature `127353b213e16688`.

The merge gate took 6,929.77 seconds after the required target cleanup. It
reported advisory wall-time, cache-footprint, and fixture-group-skew observations.

The exact one-shot results are in the
[#3525 gate comment](https://github.com/sifr-lang/sifr/pull/3525#issuecomment-5413666709).
Neither gate was rerun.

Deferred follow-ups: Item 23 owns the Reqwest 0.12 edge required by Polars and
Object Store. Item 35 owns a fresh trust-policy audit for documentation examples.

Item 35 will also recheck AWS-LC environment autodetection and the standalone
Reqwest vendor anchor. No fallback or parallel provider path was added.

Next action: implement Item 19 Rusqlite 0.40 from this record merge on
`origin/main`.

### Item 19 record

Implementation [PR #3527](https://github.com/sifr-lang/sifr/pull/3527) started
from base `4c1aaf088d76723074f519ebec1d9fe62c7813d6`. The final candidate was
`dd1e9b55c88ae60d21fea4335f155fe08cde53b4`. It merged as
`fa1d792ccc5395b6451430fcae79aee1cddb4900`.

The official registry audit confirmed Rusqlite 0.40.2 as the latest stable
release. The registry archive matched checksum
`23f2a97da3e3873c73cb2a2e71b35c40ff95e0b1eefa8d72d8499a6928c3b5b3`.
Its source metadata matched upstream tag commit
`e88f112bef7899234a497baed5cc3c3d553deeb8`.

Maintained declarations now use exact Rusqlite 0.40.2 without default features.
They enable only `bundled`. This removes the default cache and WebAssembly VFS
graph from the native fixture.

Libsqlite3 Sys advanced to 0.38.2. Its registry archive matched checksum
`f1d20bef17f513b9b3004532233187769cd072d790971f4e4da0e346eb6401e8`.
The fixture graph no longer contains Hashlink 0.11.1, SQLite WASM RS 0.5.5, or
RS SQLite VFS 0.1.1.

The runtime fixture now creates and commits a savepoint with the name
`sifr; DROP TABLE evidence; --`. A later query proves that the table survives.
This certifies the Rusqlite 0.40 safe savepoint-name API against a real negative
control. Native trust still names only the exact `libsqlite3-sys` build script
and `sqlite3` link.

Changed surfaces:

- The root and fixture locks, catalog manifest, and opaque-runtime manifest.
- Exact Rusqlite version, feature, lock, runtime, and native-trust certification.
- The opaque-runtime source, README, fixture policy, and metadata.
- Rust-interop checks, compatibility notes, coverage metadata, architecture,
  and feature-phase documentation.

The three exact dependency certification tests passed. The full stdlib
manifest test suite passed. All ten Rust-interop variants passed, and its
matrix self-test passed 239 tests.

The generated lifecycle and alias-rejection runtimes passed. Package,
coverage, offline metadata, locked fixture, feature-tree, formatting, Clippy,
documentation-link, HIR, and file-size checks passed.

The initial Opus review of exact SHA
`dd1e9b55c88ae60d21fea4335f155fe08cde53b4` returned `SATISFIED`. It found no
blocking finding. The evidence is in the
[#3527 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3527#issuecomment-5414072098).
No remediation review ran.

The one create-PR gate completed every functional step through the
runtime-platform area. That area passed all 28 variants with one expected skip.
Its first cold-cache run took 232.147 seconds against the 120-second blocking
budget. The gate exited 124 and was not rerun.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. All 698 E2E fixtures passed with signature `127353b213e16688`.

The merge gate took 7,097.75 seconds after the required 49 GiB target cleanup.
It reported advisory wall-time, cache-footprint, and fixture-group-skew
observations. The exact one-shot results are in the
[#3527 gate comment](https://github.com/sifr-lang/sifr/pull/3527#issuecomment-5415741575).
Neither gate was rerun.

Deferred follow-ups: Item 35 owns a uniqueness assertion for the maintained
Rusqlite lock edge. It will also recheck whether the architecture overview must
name the savepoint behavior. The Item 19 record preserves the upstream tag
commit. Older archived rationale remains historical evidence and is not
rewritten.

Next action: implement Item 20 SQLx 0.9 from this record merge on `origin/main`.

### Item 20 record

State: complete

Implementation [PR #3529](https://github.com/sifr-lang/sifr/pull/3529) started
from base `4f0bba8fbb6938792188141cad81089a468264b0`. The final candidate was
`ff2c7c093214d80d5146497b1cf795d68c0f00f7`. It merged as
`e58082628f4439b50a972c5fcbcbe3838ca8ecfb`.

The official registry audit confirmed SQLx 0.9.0 as the latest stable release.
The SQLx archive matched checksum
`378620ccc25c62c89d8be1c819e76a88d59bdcc3304733330788948e619bfd71`.
The Core, Macros, Macros Core, and Postgres archives matched their recorded
registry checksums. The release tag resolved to upstream commit
`75bc0487eb661da811bb7a3c5d158f1bd463fef4`.

The canonical feature policy now uses the split `runtime-tokio` and
`tls-rustls-ring-webpki` features. It also enables only Postgres. It does not
keep the removed combined runtime/TLS feature or another compatibility alias.

SQLx 0.9 macros retain inactive MySQL and SQLite lock identities. Rusqlite 0.40
requires a different `libsqlite3-sys` release with the same native link name.
Cargo cannot place both releases in the root lock. The workspace catalog
therefore owns the production Postgres graph without macros. A separately
locked backend fixture owns compile-time query certification and its checked
metadata. The fixture feature tree proves that MySQL and SQLite are inactive.

The fixture lock is aligned with the root cache identities for every active
package. Its exact inactive-lock allowlist contains only Flume 0.12,
Libsqlite3 Sys 0.37, Spin 0.9.9, SQLx MySQL 0.9, and SQLx SQLite 0.9. The policy
test rejects both missing and stale allowlist entries.

The real backend fixture compiles a checked SQLx query from maintained offline
metadata. Runtime evidence records SQLx 0.9.0, Tokio, the ring WebPKI TLS
provider, and the Postgres backend. Negative tests reject missing or stale
query metadata before network access.

Changed surfaces:

- The root and backend fixture locks, workspace catalog, and fixture manifest.
- SQLx version, feature, lock, cache-identity, runtime, and query certification.
- Rust-interop matrices, fixture policy, metadata, and documentation.
- The exact dependency test and its coverage classification.

Focused SQLx, backend, scanner, manifest, policy, formatting, Clippy,
maintainability, and file-size checks passed. The broader Sifr test suite also
passed before the one-shot gates.

The initial Opus review of exact SHA
`fa88364ba1e39186d2af70a929d7c5df205c9144` found that the fixture allowed
active patch-version drift for Chacha20, Getrandom, and Whoami. The finding is
in the [initial review comment](https://github.com/sifr-lang/sifr/pull/3529#issuecomment-5416597745).

The remediation aligned every active cache identity and added negative policy
tests. The one allowed remediation review of exact SHA
`d727e73c3870851ac1a8dce4c797b995c3737d4d` returned `SATISFIED`. Its evidence
is in the [remediation review comment](https://github.com/sifr-lang/sifr/pull/3529#issuecomment-5416597701).

The one create-PR gate ran on the remediated SHA. All ten Rust-interop variants
passed. The coverage matrix then rejected the new dependency test because its
target classification was absent. The gate exited 1 after 174.45 seconds and
was not rerun.

The final commit added only the missing coverage classification. The
Rust-interop matrix and all 241 policy self-tests passed on the final SHA. The
two-review limit prohibited a third review.

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. All 698 E2E fixtures passed with signature `127353b213e16688` across
173 cold-cache groups.

The merge gate took 7,531.60 seconds. It reported advisory warm wall-time and
fixture-group-skew observations. The exact one-shot results are in the
[#3529 gate comment](https://github.com/sifr-lang/sifr/pull/3529#issuecomment-5418080254).
Neither gate was rerun.

Deferred follow-up: Item 35 owns a clean-`CARGO_HOME` sparse-index experiment
for the five inactive fixture-lock identities. This checks whether a root
`cargo fetch` also caches the inactive package summaries needed by the
separately locked fixture. This mechanism concern came from the second review,
so it was recorded instead of starting a third review round.

Next action: implement Item 21 Itertools 0.15 from this record merge on
`origin/main`.

### Item 21 record

State: complete

Implementation [PR #3531](https://github.com/sifr-lang/sifr/pull/3531) started
from base `01456f392d51ff611890d543e870ed3837fe4fb6`. The final candidate was
`946a9cfcce5767a09ff5d269f647d73f8e9b27e1`. It merged as
`5f4f6e3068a62d6e7230e97a2e23469d39f13903`.

The official registry audit confirmed Itertools 0.15.0 as the latest stable
release. The archive matched checksum
`8b4baf93f58d4425749ca49a51c50ebab072c5df6994d08fed93541c331481dc`.
The release tag resolved to upstream commit
`37bd72aa6d58e594711d127b52418ca5e58b6091`.

The workspace now declares Itertools 0.15.0 without default features and with
only `use_std`. The maintained Ruff fork already used this release. The old
root 0.14 declaration had no direct caller, so the change did not keep an old
first-party API or compatibility path.

The certification test compiles the new `array_windows`,
`array_combinations_with_replacement`, and `strip_prefix` APIs. It also checks
the new `Position` structure and the canonical `AllEqualValueError` type. The
test proves the exact first-party lock edge and the official vendor checksum.

DataFusion 54 still owns a transitive Itertools 0.14 edge. Item 22 owns its
removal as part of the coupled Arrow and DataFusion update. Bindgen owns an
external Itertools 0.13 edge. Neither edge is a maintained direct declaration
or a reason to add a fallback.

Changed surfaces:

- The root catalog, workspace lock, and stdlib manifest test dependency.
- Exact Itertools version, feature, lock, API, and checksum certification.
- The official Itertools 0.15 vendor tree and the versioned 0.14 tree that
  remains for the DataFusion 54 graph.
- Coverage classification and a narrow checksum-preserving whitespace rule for
  one upstream workflow file.

The stdlib manifest suite, formatter tests, and the broad non-pass Sifr test
suite passed. Ten iterator E2E fixtures passed with signature
`909ec704ebb3cab2`. Workspace Clippy, formatting, HIR maintainability,
file-size, and diff checks passed. An extra all-targets Clippy probe found only
pre-existing warnings in untouched test files; it was not an item-owned
failure.

The initial Opus review of exact SHA
`946a9cfcce5767a09ff5d269f647d73f8e9b27e1` returned `SATISFIED`. It found no
blocking finding. It independently confirmed both vendor trees, their
checksums, the feature graph, and the absence of a dangling first-party
consumer. The evidence is in the
[#3531 exact-SHA review comment](https://github.com/sifr-lang/sifr/pull/3531#issuecomment-5418474451).
No remediation review ran.

The one create-PR gate completed every functional step through the
runtime-platform area. That area passed all 28 variants with no failure. Its
first cold-cache run took 217.291 seconds against the 120-second blocking
budget. The gate exited 124 and was not rerun. The exact evidence is in the
[#3531 create-PR gate comment](https://github.com/sifr-lang/sifr/pull/3531#issuecomment-5418670731).

The one merge gate ran on the final SHA and exited 0. All functional steps
passed. All 698 E2E fixtures passed with signature `127353b213e16688` across
173 cold-cache groups. The gate took 6,840.87 seconds and reported advisory
wall-time, cache-footprint, and fixture-group-skew observations. The exact
one-shot result is in the
[#3531 merge gate comment](https://github.com/sifr-lang/sifr/pull/3531#issuecomment-5419518706).
Neither gate was rerun.

Deferred follow-ups: Item 35 owns a possible loosening of the exact
first-party edge-set assertion if a later maintained consumer legitimately
uses Itertools. It also owns the external Bindgen 0.13 audit and the
pre-existing all-targets Clippy warnings if they remain. These findings did not
block the Item 21 mechanism.

Next action: implement Item 22 Arrow 59 and DataFusion 55 from this record
merge on `origin/main`.

### Item 22 record

State: complete

Implementation [PR #3533](https://github.com/sifr-lang/sifr/pull/3533) started
from base `67a9061d03a2d1d7730287a9141dfacb8829053b`. The final candidate was
`61392c034d6c30216cc5a6bd2ceb67cfa01e925b`. It merged as
`92430c5e52042b82567e3798045d5a3414a42676`.

The official registry and upstream release audit confirmed Arrow 59.2.0 and
DataFusion 55.0.0 as their latest stable releases. The Arrow archive matched
checksum
`61d285d16bce7d0be61912f7928342b673067b6b7d7ef6cc179258ba7de1fecf`,
and its release tag resolved to upstream commit
`782e5a685501a9db6cc8e9a3b7cbff894940c47a`. The DataFusion archive matched
checksum
`96f76f0167ed0842b29a3d1e41be3c034c0a46409a3a703cc4cc84ee8c24abf4`,
and its release tag resolved to upstream commit
`d5552342012888b7d1a3ab88d92e3d292fc0cde0`. DataFusion 55 requires Rust
1.94, which is below Sifr's pinned Rust 1.98 compiler.

Arrow advanced first inside the compatibility unit. The intermediate root
check passed while DataFusion 54 temporarily retained Arrow 58. DataFusion
then advanced and collapsed both maintained locks onto one coherent family:
all 14 Arrow crates and Parquet use 59.2.0, and all 31 DataFusion crates use
55.0.0. DataFusion's direct Itertools edge now uses 0.15.0. Object Store still
owns an external transitive Itertools 0.14 edge; the certification names that
owner instead of preserving an old first-party compatibility lane.

The canonical advanced-data bridge adopts DataFusion 55's borrowed
`DataFrame::fill_nan(&ScalarValue, &[&str])` API. It registers the Arrow record
batch, builds and observes the NaN-fill logical plan, and propagates catalog
lookup failures as typed bridge errors. It no longer converts a catalog error
to a missing-table value. Runtime summaries, Sifr fixtures, driver assertions,
policy mutation coverage, exact family/checksum tests, and documentation agree
with that behavior.

Focused validation passed: the new Arrow/DataFusion certification passed all
3 tests; Itertools ownership certification passed all 4 tests; the locked
advanced-data Cargo workspace checked; the advanced-data Rust-interop matrix
passed both variants and all 242 mutation cases; its generated positive runtime
and negative mismatch tests passed; and the complete stdlib-manifest, coverage,
Rust-interop, and focused Python Arrow suites passed. The broad non-pass Sifr
test suite, workspace Clippy, formatting, HIR maintainability, file-size,
offline-fetch, and diff checks also passed.

The one exact-SHA Opus review returned `SATISFIED` with no blocking finding.
It independently confirmed the two lock families, checksums, new API, error
propagation, and absence of an Item 22 compatibility path. The evidence is in
the [#3533 review comment](https://github.com/sifr-lang/sifr/pull/3533#issuecomment-5419899447).
No remediation review ran.

The one create-PR gate passed every functional step. Its runtime-platform area
passed all 28 variants with one declared capability skip, but the clean-cache
area took 219.467 seconds against its 120-second blocking wall-time budget.
The gate therefore exited 124 after 1,152.22 seconds and was not rerun. The
exact result is in the
[#3533 create-PR gate comment](https://github.com/sifr-lang/sifr/pull/3533#issuecomment-5420042818).

The one merge gate ran on the same approved candidate and exited 0. All
functional steps passed, including all 76 generated-build integrations and the
exact advanced-data runtime fixture. All 698 E2E fixtures passed across 173
cold-cache groups with signature `127353b213e16688`. The gate took 6,809.08
seconds and reported only advisory wall-time, cache-footprint, and fixture-group
skew observations. Its exact result is in the
[#3533 merge gate comment](https://github.com/sifr-lang/sifr/pull/3533#issuecomment-5420864905).
Neither gate was rerun.

Deferred follow-up: Item 35 owns the review suggestions to anchor the catalog
error-propagation source assertion to the exact `table_exist` chain, add the
corresponding mutation case, and remove one redundant stored-plan `nanvl`
check if the final audit confirms that simplification preserves the evidence.
The external Itertools 0.13 edge remains owned by the Item 21 closure audit.
An untouched all-feature Python Arrow wrapper also emitted its pre-existing
dead-code warning; it did not affect Item 22 or workspace Clippy.

Next action: implement Item 23 Polars 0.55 from this record merge on
`origin/main`.

### Item 23 record

State: complete

Implementation [PR #3535](https://github.com/sifr-lang/sifr/pull/3535) started
from base `4f5432bdc7080811f0ab376d6be516def5cc784a`. The final candidate was
`4051c44442b607c2800dafb0975a3e1b6dd4abd3`. It merged as
`02a2429ae30fd2c48baa4165e32a1fb13e88ecae`.

The official crates.io and upstream release audit confirmed Polars 0.55.2 as
the latest stable release. The crate archive matched checksum
`d52d3ed4e6b3917427f6d3c43edbd2740babe228bb4ccfa3431eac105844045d`, and
the `rs-0.55.2` release tag resolved to upstream commit
`d7488c71ecfbc77790292ff5b365b991c08380ce`.

The exact generated-runtime catalog and advanced-data fixture now require
Polars 0.55.2. Both maintained locks were regenerated as a coherent 23-crate
Polars family. The advanced-data bridge adopts the stable
`DataFrameIsSorted::is_sorted` API from the Polars prelude and observes
sortedness on the live dataframe. It propagates Polars errors through the
typed bridge result and gates the runtime observation on that live result. No
stored compatibility value, legacy path, or fallback remains.

Exact dependency/checksum certification, capability classification, policy
mutation coverage, fixture output, and documentation were updated together.
Focused validation passed the three Polars certification tests, the complete
stdlib-manifest suite, the locked advanced-data Cargo check and Clippy, both
advanced-data Rust-interop variants and all 243 mutation/self-test cases, the
complete 10-variant Rust-interop area, the exact positive runtime and negative
mismatch cases, the five-variant coverage area, and the focused Python Arrow
and runtime tests. The broad non-pass Sifr suite, workspace Clippy, formatting,
HIR maintainability, file-size, offline-fetch, and diff checks also passed.

The one exact-SHA Opus review returned `SATISFIED` with no blocking finding.
It independently confirmed the exact release, coherent locks, live sortedness
observation, typed error propagation, and absence of compatibility behavior.
The evidence is in the
[#3535 review comment](https://github.com/sifr-lang/sifr/pull/3535#issuecomment-5421287214).
No remediation review ran.

The one create-PR gate passed every functional step. Its runtime-platform area
passed all 28 variants with one declared capability skip, but the clean-cache
area took 212.673 seconds against its 120-second blocking wall-time budget.
The gate therefore exited 124 after 1,181 seconds and was not rerun. The exact
result is in the
[#3535 create-PR gate comment](https://github.com/sifr-lang/sifr/pull/3535#issuecomment-5421459671).

The one merge gate ran on the same approved candidate and exited 0. Every
blocking area passed, including the fresh Polars/Arrow generated-project
runtime and mismatch paths. All 698 E2E fixtures passed across 173 groups with
signature `127353b213e16688`. The lane took 6,882.03 seconds, used
2,412,740,608 bytes peak RSS with zero swaps, and reported only advisory
wall-time and group-skew observations. Its exact result is in the
[#3535 merge gate comment](https://github.com/sifr-lang/sifr/pull/3535#issuecomment-5422596718).
Neither gate was rerun.

Deferred follow-up: Item 35 owns the review suggestion to replace the
file-wide textual `unwrap_or` absence assertion with a structural assertion
anchored to the exact sortedness observation if the final audit confirms that
it improves mechanism-level evidence.

Next action: implement Item 24 Rust Redis 1.6 and graph reconciliation from
this record merge on `origin/main`.

### Item 24 record

State: complete

Implementation [PR #3537](https://github.com/sifr-lang/sifr/pull/3537) started
from base `953df53fb312a47383978b9957584b7aa0f3b4c6`. The final candidate was
`814bf2559c0ca44e611984d07fd9ddefbf170079`. It merged as
`9e12539abe98a7fecd6f043bcf1e81b860fc68d9`.

The official crates.io audit confirmed Redis 1.6.0 as the latest stable
release, published on 2026-08-15 with checksum
`e37a4ca5c6ca42aa3e6df2fd32b987a65d32a4c2159a6f3fe0fd1df306a2658f`.
The upstream `redis-1.6.0` release tag resolved to commit
`20f68ee5a0e50a45c403ddb0d93dbe2838dd3aba`. A final live crates.io pass
checked all 109 maintained direct registry packages across 168 declarations
in 113 manifests and found zero stale packages.

All three maintained Redis declarations and locks now select Redis 1.6.0.
The canonical feature policy enables `connection-manager` and `tokio-comp`.
The opaque-resource bridge adopts Redis 1.6's `ConnectionManager`, bounds
reconnects to two attempts, retains connection and response timeouts, and
observes that bound in generated-runtime output. The separate malformed-RESP
probe continues to use a direct multiplexed connection because retries would
weaken that negative protocol test; it is not a fallback for the primary
connection path. Fixture metadata, locks, scenario mutations, matrix policy,
generated-runtime assertions, coverage classifications, architecture, and
public documentation were reconciled together. No legacy or compatibility
path was added.

The item also added a checked-in official-registry audit snapshot and exact-set
tests for the maintained Cargo manifest inventory, direct declaration count,
latest-stable requirements, and registry checksums. Focused dependency tests,
the complete stdlib-manifest suite, exact-test Clippy, both standalone Redis
fixture checks and Clippy, the complete 10-variant Rust-interop area, all 40
matrix fixtures and 243 mutation/self-test cases, four ignored generated
Redis runtime tests, the broad non-pass Sifr suite, workspace Clippy,
formatting, HIR maintainability, the 3,258-file size guard, JSON, offline-lock,
coverage-matrix, and diff checks passed.

The first exact-SHA Opus review found two in-scope omissions: the new test
targets lacked coverage-matrix classifications, and two canonical feature
policy lines still named the old Redis feature set. The remediation added both
classifications and updated both policy lines. The single remediation review
returned `SATISFIED`, confirmed both blockers fixed, and found no new mechanism
defect. The review evidence is in the
[#3537 first review](https://github.com/sifr-lang/sifr/pull/3537#issuecomment-5423317420)
and
[#3537 remediation review](https://github.com/sifr-lang/sifr/pull/3537#issuecomment-5423384023)
comments. No third review ran.

The one create-PR gate passed every executed functional step. Python interop
passed all 19 variants, runtime platform passed all 28 variants with one
declared skip, and Rust interop and diagnostics passed. The runtime-platform
area took 212.313 seconds against its 120-second cold-cache budget, primarily
because `binary_file_io_capability.sifr` took 166.065 seconds. The gate exited
124 after 1,100 seconds and was not rerun. The exact evidence is in the
[#3537 create-PR gate comment](https://github.com/sifr-lang/sifr/pull/3537#issuecomment-5423609456).

The one merge gate ran on the same approved candidate and exited 0. Every
blocking area passed. The generated-build driver batch passed 76 tests,
including the Redis callback and opaque-resource lifecycle paths. All 698 E2E
fixtures passed across 173 groups with signature `127353b213e16688`. The lane
took 6,936.72 seconds, used 2.3 GiB peak RSS with zero swaps, and reported only
advisory warm-wall-time and group-skew observations. Its exact evidence is in
the
[#3537 merge gate comment](https://github.com/sifr-lang/sifr/pull/3537#issuecomment-5424851098).
Neither gate was rerun.

Deferred follow-up: none. The remediation review found no new mechanism defect.

Next action: implement Item 25 Python minor train from this record merge on
`origin/main`.

### Item 25 record

State: complete

Implementation [PR #3539](https://github.com/sifr-lang/sifr/pull/3539) started
from base `bc9423835afe0af060b5c2bd03620cd1525bd774`. The final candidate was
`6c590146d31b18361594f7a50ed8ad896107abff`. It merged as
`154fcd7d20c8139f5ac833d5c5da38dfd8871c12`.

The official PyPI audit confirmed these latest stable releases:

- Alembic 1.19.1
- Boto3 and Botocore 1.43.80
- Certifi 2026.7.22
- Polars and Polars Runtime 1.44.1
- Schwifty 2026.7.3
- SQLAlchemy 2.0.52
- Torch 2.13.0

The audit checked all seven selected artifacts against their PyPI SHA-256
values. Both affected locks resolve with Python 3.14.7 and uv 0.12.5.

The Boto3 upgrade exposed a real protocol incompatibility in the maintained
LocalStack 2.0.1 emulator. The current Botocore SQS client no longer works
with that legacy service. The item replaced it with LocalStack Community
4.14.0, the latest stable open-source release. The image uses the immutable
multi-platform manifest digest
`sha256:3ebc37595918b8accb852f8048fef2aff047d465167edd655528065b07bc364a`.
No old protocol, compatibility path, or fallback remains.

The executable fixtures use current stable APIs. The Polars bridge uses
`struct.drop`. The Torch bridge uses `LinearCrossEntropyLoss`. The Alembic
bridge checks named CHECK-constraint autogeneration. The Certifi bridge loads
the CA bundle into an SSL trust store. A direct Schwifty suite generates and
validates 32 BBANs across eight national checksum algorithms.

The item added a machine-checked stable-release record. It verifies both lock
owners, exact versions, selected artifact hashes, and the LocalStack image
digest. Four negative mutations cover stale versions, missing artifacts,
missing declarations, and a stale service emulator. All four delivery profiles
now execute both new suites.

Focused validation passed after each sequential upgrade. Both lock checks,
the dependency audit, the feature suite, the DLPack demo, and the compiled
dataframe, ML, and library suites passed. The complete Python-interop area
passed every offline, compiled, and native case. Its first live pass identified
the old LocalStack protocol failure. The final focused live-service run passed
all six cases with the exact LocalStack 4.14.0 digest.

Ruff format and lint checks passed. JSON, runner and profile self-tests, profile
membership checks, HIR maintainability, file-size, and diff checks also passed.
No compiler input changed, so the phase rules prohibited the Sifr create-PR
and merge gates.

The first exact-SHA Opus review found one in-scope omission. The new stable
audit and feature suites were absent from all four delivery profiles. The
remediation added both suites to create-PR, merge, nightly, and release.
The one permitted remediation review returned `SATISFIED` with no blocking
finding. The evidence is in the
[#3539 first review](https://github.com/sifr-lang/sifr/pull/3539#issuecomment-5425832691)
and
[#3539 remediation review](https://github.com/sifr-lang/sifr/pull/3539#issuecomment-5425833075)
comments. No third review ran.

Deferred follow-up: Item 35 owns a reverse profile-coverage invariant. This
check must require each non-live manifest suite in at least one delivery
profile. Item 35 also owns reconciliation of the pre-existing release-profile
custody digests. The final audit can also derive the Schwifty output count and
mutation count, and can relax the cross-platform Torch tolerance if evidence
supports that change.

Next action: implement Item 26 CFFI 2 and Cryptography 50 from this record
merge on `origin/main`.

### Item 26 record

State: complete

Implementation [PR #3541](https://github.com/sifr-lang/sifr/pull/3541) started
from base `4065d05db13529ae280a1194148fb84340da4819`. Its exact candidate was
`0ac31643c4e7b2357694836980f5f15c97bc83d5`. It merged as
`daf6c293237a6b3e5d6e1e2b1759eccbe6133b75`.

The official source audit confirmed CFFI 2.1.1 and Cryptography 50.0.1 as the
latest stable releases. Cryptography 50.0.1 was newer than the phase baseline.
The audit also corrected the installed Cryptography baseline to 45.0.7.

The CFFI upgrade ran first. CFFI advanced from 1.17.1 to 2.1.1 while
Cryptography stayed at 45.0.7. The callback examples and full library suite
passed before the Cryptography constraint changed.

Cryptography then advanced to 50.0.1 while CFFI stayed at 2.1.1. The selected
wheels match their exact PyPI SHA-256 values. The installed Cryptography wheel
uses OpenSSL 4.0.2.

The CFFI feature suite now exercises the stable `cffi.gen_src` command. The
Cryptography bridge builds a root and leaf certificate chain. The stable X.509
verification API accepts the correct DNS name and rejects a wrong name.
Certifi SSL trust and Fernet encryption also pass. No compatibility path or
fallback was added.

The stable-release audit now owns nine Python packages. It checks exact lock
versions, selected artifact hashes, constraints, and both Python environment
owners. The new CFFI suite runs in all four delivery profiles.

Focused validation passed for the environment, dependency, ABI, tier-one,
callback, and compiled library suites. Direct feature probes also passed.
Ruff, JSON, lock, profile, HIR maintainability, file-size, and diff checks
passed. No compiler input changed, so no Sifr gate ran.

The one exact-SHA Opus review returned `SATISFIED` with no blocking finding.
The evidence is in the
[#3541 review comment](https://github.com/sifr-lang/sifr/pull/3541#issuecomment-5426226282).
No remediation review was needed.

Deferred follow-up: Item 35 owns two final audit improvements. It will compile
and load the source emitted by `cffi.gen_src`. It will also verify each recorded
package `requires_python` value against both owned Python lanes.

Next action: implement Item 27 FastAPI and Starlette from this record merge on
`origin/main`.

### Item 27 record

State: complete

Implementation [PR #3543](https://github.com/sifr-lang/sifr/pull/3543) started
from base `30f5a798ec9ccc20eda8b34eca45ab16c5f96721`. Its exact approved candidate
was `1b4d5d1425c2e1750e2baf77a63bd092055adbf8`. It merged as
`16dbb1b3d4656618d2a90abfc7c3ba949f9fdf13`.

The official source audit confirmed FastAPI 0.141.1 and Starlette 1.6.0 as the
latest stable releases. It also identified HTTPX2 2.12.0 and HTTPcore2 2.12.0
as the canonical stable client stack.

FastAPI advanced first while Starlette stayed at 0.52.1. The compiled library
suite passed before the Starlette constraint changed. Starlette then advanced
to 1.6.0, and the suite passed again.

Starlette TestClient reported its old HTTPX backend as deprecated. The item
removed the old `httpx` and `httpcore` distributions. It migrated maintained
callers to HTTPX2 and HTTPcore2 without a compatibility path or fallback.

The bridge now uses `app.frontend` for static frontend mounting. It verifies
that dependency headers and background tasks survive FastAPI response handling.
The Starlette bridge also enforces `max_body_size` and verifies the 413 response.
The HTTPX2 fixture uses `ASGITransport` and the new async client identity.

The stable-release audit now owns 12 Python packages and five mutations. It
checks exact versions, selected PyPI hashes, constraints, owner environments,
and rejection of the retired HTTPX distributions.

Focused validation passed for both sequential framework states. The final
library, feature, environment, dependency, tier-one, and HTTPX2 suites passed.
Ruff, JSON, documentation, HIR, coverage, taxonomy, file-size, and diff checks
also passed.

The first exact-SHA Opus review found three documentation omissions. The
remediation corrected the mutation count and two stale HTTPX client names.
The permitted remediation review returned `SATISFIED` with no blocking finding.
The evidence is in the
[#3543 first review](https://github.com/sifr-lang/sifr/pull/3543#issuecomment-5426675197)
and
[#3543 remediation review](https://github.com/sifr-lang/sifr/pull/3543#issuecomment-5426675612)
comments. No third review ran.

A compiler fixture changed, so both Sifr gates applied to the approved SHA.
The create-PR gate ran once after the required private-target cleanup. Every
reached check passed, but the cold runtime-platform area took 185.842 seconds.
This exceeded its 120-second warm budget and stopped the lane. The report hash
is `dcd81f6e8721509ddff159e5bc9e3648f44c469fe970a04adfe370568596ceb0`.

The merge gate ran once and exited successfully. Every validation area and
full crate suite passed. The E2E corpus passed 698 of 698 fixtures with report
signature `127353b213e16688`. The cold lane took 6,852.80 seconds, used 2.4 GiB
maximum RSS, and used no swap. Its report hash is
`e13aeda8e58d53217059ae1f1ef0fe4dc3c5b95c3bfb5d7153ac7cfe79688c68`.
Neither gate was rerun. The exact gate evidence is in the
[#3543 gate comment](https://github.com/sifr-lang/sifr/pull/3543#issuecomment-5428136428).

Deferred follow-up: Item 35 will decouple the retired-distribution guard from
the owner label. It will reconcile taxonomy with runtime topology paths. It
will also update current protocol documentation from HTTPX to HTTPX2. Frozen
historical evidence will remain unchanged. The final audit will classify other
HTTPX mentions, including the tier-two `pytest-httpx` package.

Next action: implement Item 28 Python Redis services from this record merge on
`origin/main`.

### Item 28 record

State: complete

Implementation [PR #3545](https://github.com/sifr-lang/sifr/pull/3545) started
from base `eaa3b18320e86a41d432c1dc8f390818f0b96e63`. Its exact approved candidate
was `5f368cb97c2dfca80a1ef544c806408e90548466`. It merged as
`96bda0982f1b53595d1f7da1aeb3ed643a6129bf`.

Official sources confirmed Redis 8.1.0, Fakeredis 2.37.1, Hiredis 3.4.1, and
Testcontainers 4.15.0 as the latest stable releases. The official Redis server
release was 8.10.1. Its Alpine OCI index digest is
`sha256:becdda6c7f4b3fb42e42fd7f120bbf5c54c4caaaf16f26da24e4563d2c1f0576`.

Redis advanced first from 6.4.0 to 8.1.0. All three companion packages stayed
fixed. The Redis 8.1 command-surface probe and compiled library suite passed.

Fakeredis then advanced from 2.36.2 to 2.37.1. The corrected RESP3 `ZPOPMIN`
shape and final-key cleanup passed. The compiled library suite also passed.

Hiredis then advanced from 3.4.0 to 3.4.1. The release contains bundled-parser
security fixes. RESP3 map parsing and the compiled library suite passed.

Testcontainers advanced last from 4.13.3 to 4.15.0. The item removed all old
service import paths. The runner now uses community imports and structured wait
strategies. No compatibility path, legacy import, or fallback remains.

The live Redis bridge now executes Redis 8 `SDIFFCARD` and `SUNIONCARD` against
the digest-pinned Redis 8.10.1 service. The offline feature suite covers the
four exact package versions without contacting Docker.

The stable audit now owns 16 Python packages and two service images. It checks
two lock owners, selected PyPI artifact hashes, exact image digests, and five
negative mutations. The new Redis feature suite runs in all four offline
delivery profiles.

Focused validation passed for every sequential state. The final lock, audit,
environment, package, tier-one, feature, policy, and compiled library checks
passed. The real Docker suite built and executed all six native binaries. It
reported zero skips and zero failures with deprecations treated as errors.

Runner self-tests, profile schema checks, Ruff, JSON, HIR, coverage, taxonomy,
file-size, and diff checks passed. Official PyPI hashes and the Redis image
digest matched the recorded values.

The first exact-SHA Opus review found one blocking mechanism. The offline
feature suite constructed a Docker client and therefore depended on the host.
The remediation changed it to inspect stable class APIs without construction.
The suite then passed with an intentionally unreachable Docker socket.

The permitted remediation review returned `SATISFIED`. It reproduced the old
failure, verified the fix, and found no new mechanism defect. The evidence is
in the
[#3545 first review](https://github.com/sifr-lang/sifr/pull/3545#issuecomment-5428693676)
and
[#3545 remediation review](https://github.com/sifr-lang/sifr/pull/3545#issuecomment-5428734463)
comments. No third review ran.

No compiler input changed. The phase rules therefore prohibited the Sifr
create-PR and merge gates.

Deferred follow-up: Item 35 will derive feature-suite versions from the stable
audit. It will audit the floating Postgres and Redpanda service images. It will
also scan every runner module for legacy Testcontainers APIs and add mutation
coverage for every forbidden import and wait helper. The final audit will
separate Redis release identity from its Alpine image variant. It will review
the runner bootstrap `E402` suppressions and derive Redis `numkeys` from the key
list.

Next action: implement Item 29 NumPy and Pandas from this record merge on
`origin/main`.

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

Current state: Items 0-28 are complete. Item 28 merged in PR #3545 as
`96bda0982f1b53595d1f7da1aeb3ed643a6129bf`. Its remediation Opus review
returned `SATISFIED` with no blocker.

Redis 8.1.0, Fakeredis 2.37.1, Hiredis 3.4.1, and Testcontainers 4.15.0 are
installed. All focused and live service checks pass. No compiler input changed,
so no Sifr gate ran.

Next action: merge this record-only update. Then start Item 29 NumPy and Pandas
from the resulting `origin/main`.
