# Item 41: Rust and uv selection preparation

State: source preparation authorized on 2026-09-08. The coordinator explicitly
released B48 after recorder 97134, DTServiceHub 97167 and target 97262 were absent
and the direct recorder was reaped. Private provisioning, lock consistency and
the registered non-native checks may now proceed.
No publication, push, PR, broad gate, native Cargo compilation or merge is
authorized in this preparation session. Item 42/43 work is excluded.

## Ownership registered before source edits or tests

- Root: `/private/tmp/sifr-item41.Nx7JrP/codebase`, branch
  `codex/latest-stable-item41-prepare`, base
  `4b4cc339964baeeb6641e57dc669fef700a5fa24` (official latest main).
- Actual independent Ruff Git repository: `third_party/ruff` under that root,
  branch `codex/latest-stable-item41-toolchains`, base
  `f19957111640fdee8055bfe5b6aa854259344473` (root's existing gitlink).
- All owned temporary files, archives, environments, caches, toolchains and
  evidence stay under `/private/tmp/sifr-item41.Nx7JrP/`: `tmp`, `downloads`,
  `envs`, `cache`, `toolchains`, `evidence`. Cargo/Rustup and uv environment
  paths are private. No global toolchain configuration changes.
- Parent ledger/worktree/index/branch and every predecessor remain read-only.
  This clone does not stack the held Item 40 or SDK PR #3821 candidates.
- Item 38 prerequisite is merged in PR #3763, reviewed candidate
  `293d62528005032aa4d67f8d43dbe037899a2457`, merge
  `619422a7a385ce9b124c9a78d0229e3213237933`.

## Source boundary

Root selection sources:

- `rust-toolchain.toml`, `Cargo.toml` workspace compiler requirement;
- `.github/workflows/local-first-validation.yml`, `preview-release.yml`,
  `release-qualification.yml` under the same workflow directory;
- `verification/policy/github_actions.json`;
- `verification/areas/runtime_platform/supported_platforms.json`;
- `scripts/check_uv_toolchain.py` qualified checksum/version table;
- `verification/README.md` and the current toolchain paragraph in
  `internal_docs/architecture.md` (selection documentation, no architecture
  redesign); this item record.

The six root maintained project/lock pairs are `verification`,
`verification/areas/python_interop`, `demos/python_binding_authoring`,
`demos/python_environment_checks`, `demos/python_raw_api`, and
`demos/python_dlpack`: each owns `pyproject.toml` and `uv.lock`. Pin uv in each
manifest; use the selected uv to check each actual existing lock. Regenerate
only if required for the new tool version, without package upgrades.
Vendored PyO3 and fixture inputs are excluded. No new distribution-runtime
project exists on this base; held PR #3821's runtime remains its own item.

The Ruff owner has five additional maintained project/lock pairs, separate
from those six: `.`, `python/py-fuzzer`, `python/ruff-ecosystem`,
`scripts/benchmarks`, `scripts/ty_benchmark`. Add exact uv required-version to
their existing tool.uv sections. Update Ruff's `rust-toolchain.toml` and the
existing uv selections in `.github/workflows/ci.yaml`, `daily_fuzz.yaml`,
`memory_report.yaml`, `publish-docs.yml`, `publish-pypi.yml`,
`sync_typeshed.yaml`, `ty-ecosystem-analyzer.yaml`, `ty-ecosystem-report.yaml`.
Fork dependency constraints, lock resolution, action-version audit, source,
fixtures, upstream Python support range and Cargo minimum support remain with
their existing owners; no Ruff 0.16.6 replay. Root `third_party/ruff` must name
an actual commit produced in this independently owned repository.

Install Item 38's existing invariant as an explicit CI check. Its existing
unsupported-runner and missing-platform-checksum rejection remains required.
No expanded composite/direct-install discovery (later issue #3764).

## Official source readiness

Read-only source observations on 2026-09-08:

- Rust stable manifest: https://static.rust-lang.org/dist/channel-rust-stable.toml
  dated 2026-09-03, rustc `1.98.1 (48a229cea 2026-09-01)`.
- Immutable dtolnay/rust-toolchain branch target for 1.98.1:
  `ce678459e9fc7500d337468f904b95f1b5c10b5e`; its action.yml selects 1.98.1.
- https://api.github.com/repos/astral-sh/uv/releases/latest selects 0.12.10,
  published 2026-09-04T23:15:57Z.
- uv 0.12.10 x86_64 GNU/Linux archive SHA-256:
  `173d95a0c32d18c896c46ba6fafbf3cf9c14ab74b033f81b76c883ef492a976b`,
  matching the upstream checksum asset and GitHub release-asset digest.
- uv 0.12.10 local aarch64 macOS archive SHA-256:
  `51c6170e8e3a01cef9f33b94f582b7b81ac65046f55d40afb35f9cff5a68c179`.

## Registered named validation after actual resource release

Use privately provisioned exact tool paths, owned caches and GIL CPython
3.14.7. No native compilation. Execute only:

1. `rustc --version`; `uv --version`.
2. `python3 scripts/check_uv_toolchain.py --self-test` and
   `python3 scripts/check_uv_toolchain.py`.
3. `uv lock --check --project PROJECT` for each of the six root projects
   listed above. No `--upgrade`; preserve unchanged lock bytes if accepted.
4. `uv run --project verification --locked python -m sifr_verify areas run
   --area python_interop --suite env`. Source inspection establishes this
   suite uses the verification Python only, with no Sifr/Cargo build or full
   interop package installation.
5. `git diff --check` in root and Ruff owner; root
   `python3 scripts/check_file_size_guardrails.py`; relevant local doc links.

The coordinator explicitly authorized the five real Ruff project owners and
their required lock consistency after source inventory. Additional registered
commands from the Ruff repository root are `uv lock --check --project .`,
`uv lock --check --project python/py-fuzzer`,
`uv lock --check --project python/ruff-ecosystem`,
`uv lock --check --project scripts/benchmarks`, and
`uv lock --check --project scripts/ty_benchmark`. Use an owned interpreter/cache;
do not install project dependencies to check locks. If regeneration is required,
the corresponding `uv lock --project PROJECT` is authorized only for consistency
with these same selectors and existing locked packages; inspect any graph diff
before accepting it, and stop on unrelated dependency drift. No hook pass is
claimed by these lock checks.
The fork AGENTS hook recommendation is superseded by the explicit item-only
test scope and native-work prohibition. Source-only review will disclose this.

Native demo/release qualification and the single merge-profile gate remain
WITHHELD delivery obligations, with known compiler/policy prerequisite
histories preserved. Re-read prerequisite evidence before execution; do not
reproduce their unchanged failures. One exact-source Opus review and at most
one remediation; review tools limited to Read/Grep/Glob, strict empty MCP,
no modifications or broad validation. Review evidence stays outside Git.

## Preparation result before candidate freeze

All registered non-native checks finished and the resource slot was released
to the coordinator for B49. No owned provisioning/resolution/test/build process
remained at release. Rust 1.98.1, uv 0.12.10 and GIL CPython 3.14.7 were installed
only under the registered private roots. The new CI job runs Item 38's existing
invariant, including its explicit unsupported-runner rejection.

- Rust/uv version identity: PASS; upstream macOS uv archive digest matched.
- uv invariant self-test: PASS, 46 checks; repository invariant: PASS, six root
  exact pins and three setup-uv steps with the qualified Linux checksum.
- Six Sifr locks: PASS with the selected uv/Python; all six remain unchanged.
- Ruff locks: root, py-fuzzer and ruff-ecosystem passed in the initial grouped
  check; benchmark failed, so the loop stopped before ty_benchmark. The latter
  passed in its separately recorded check. The initial failed log is preserved.
- Ruff benchmark had an existing local identity mismatch: pyproject `scripts`
  0.16.4 versus locked virtual package 0.16.3. The registered uv regeneration
  changed only that local package version to 0.16.4, with zero external package
  changes. Its affected final lock check then passed. All five Ruff owners
  therefore have individually attributable passing evidence.
- Python interop `env`: PASS, one variant, zero failures; CPython 3.14.7.
- Root/Ruff diff checks: PASS. Root file-size guard: PASS, 3,770 files,
  900-line limit. Changed current-document references retain existing paths.

No Sifr gate, native compilation, cross-platform/release qualification, fork
hook run, PR publication, push or merge occurred. The source has eight root
Rust workflow selections and 24 Ruff uv selections across eight fork workflow
files; provisional earlier counts are superseded by this complete recount.

Fresh authenticated delivery-prerequisite readback on 2026-09-08:
compiler PR #3717 OPEN at `98480c78587d6cbd99a7079d10c71825360bd468`;
solo-policy PR #3785 OPEN at `4c7068b36904e216b02778f5664d2b8fd1159a6a`.
Neither is merged qualification. Existing failed gate histories are not reset.
Rust dependency audit drift remains Item 49's owner; no runtime graph changed
in this selection preparation. Publication, owner integration and the required
merge gate remain explicitly withheld until separately authorized delivery.

Later Ruff-tooling followup 41-F1: `scripts/benchmarks/pyproject.toml` contains
legacy `python = ">=3.10,<3.12"` but no standard `requires-python`, so uv reports
its implicit Python floor; the maintained lock currently records `>=3.14`.
The unchanged `astroid==3.3.7` is yanked for a Cryptography/Pylint crash.
These warnings concern existing benchmark dependency/support policy and belong
to a separately scoped Ruff-owned item. Neither warning was suppressed or
resolved through an unrelated dependency upgrade here.

## Single remediation registration

Initial exact-source review of root `9f8d6599ef33b6575e1b2ae7cf7ab0f537752594`
and Ruff `d3e02fa84dfa8814e3a3ed6c3892f2d5a48f7c72` returned NOT SATISFIED.
External response SHA-256:
`524231af1c0a8cb633a735bfc609e7872e00e6790c7256eb21f298e00f9cf142`.
That frozen candidate and initial evidence are preserved, with no gate/review
counter reset. One coherent correction batch is registered before its edits:

1. Root `.github/workflows/local-first-validation.yml`: add the required
   recursive-submodule checkout input to the new invariant job. Source reading
   found existing Item 37 ownership validation rejects its omitted input.
2. Real Ruff owner `.pre-commit-config.yaml`: advance ONLY the
   `astral-sh/uv-pre-commit` immutable revision and matching frozen-version label
   from 0.12.5 to the officially resolved 0.12.10 revision. The initial review
   identified this missed uv selection: the old hook executes uv0.12.5 against
   the newly exact0.12.10 manifest and therefore fails. This is selection
   consistency, not an unrelated pre-commit hook/package upgrade.

The coordinator authorized the following lightweight checks alongside B49:
`python3 scripts/check_submodule_ownership.py --self-test` and
`python3 scripts/check_submodule_ownership.py` (existing check directly covering
the omitted checkout rule), `python3 scripts/check_uv_toolchain.py --self-test`,
`python3 scripts/check_uv_toolchain.py`, root/Ruff `git diff --check`, and
`python3 scripts/check_file_size_guardrails.py`. These commands are registered
before execution. No install, resolver, hook environment, native build or broad
gate workload is included. Earlier locks/env/version evidence retains unchanged
inputs; do not repeat it. Official uv-pre-commit tag/source metadata is read-only
evidence for the corrected hook pin. One final-source remediation review only.

Other initial-review observations remain later owner records:

- 41-F2, Ruff tooling: the two previously unpinned setup-uv steps in
  `ci.yaml` (ruff-lsp) and `publish-versions.yml` need separate inventory work;
  they were not explicit old pins and currently select the same latest uv.
- Item 49/package owner: historical toolchain audit paragraph in
  `crates/sifr_package/DEPENDENCY_AUDIT.md` is not current toolchain evidence.
- SQL/distribution owners: recorded SQL Rust1.98 and selftest Rust1.90/uv0.12.5
  strings have no binding to selected workspace toolchains; no fixture rewrite.
- Editor owner checkout is absent in this intentionally scoped clone; the
  broader action-pin validator was not executed or claimed. Source inspection
  established coherence of this item's eight root Rust action selections.

None of those followups is silently implemented or treated as new scope.

The official `astral-sh/uv-pre-commit` tag 0.12.10 resolves to commit
`7720020191168f707a43279c65b6bf9cdf7ed04e`. Read-only inspection of that exact
revision's `pyproject.toml` confirms `uv==0.12.10`; `.pre-commit-hooks.yaml`
defines the unchanged `uv-lock` entry as `uv lock`. The corrected fork pin
therefore selects the same exact uv as its maintained manifests. No hook
environment was installed and no hook execution pass is claimed.

Remediation validation: ownership self-test and repository guard PASS; uv
self-test 46 and repository six-pin/three-setup invariant PASS; root/Ruff diff
guards PASS; file-size PASS, 3,770 files. These are the only repeated/new
focused checks. No install, resolution, native or broad validation workload
ran. Existing version, six-root/five-fork lock, env and archive evidence is
reused because its relevant inputs are unchanged. The final review must bind
the final root and actual Ruff owner commit; there is no third review allowance.
