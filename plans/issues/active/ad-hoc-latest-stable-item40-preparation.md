# Item 40 preparation registration

Status: registered before implementation and tests on 2026-09-08.

This session is preparation only under the coordinator's current decision:
no PR publication, create-PR gate, merge-profile gate, merge, or next item.
The expected-dirty parent phase ledger and all predecessor trees are read-only.

## Ownership and base

- Independent clone: `/private/tmp/sifr-item40.iCyYG2/codebase`.
- Root branch: `codex/latest-stable-item40-prepare`; base
  `4b4cc339964baeeb6641e57dc669fef700a5fa24`; private `.git/index`.
- Actual editor owner checkout: `editor_integrations`, repository
  `sifr-lang/editor-integrations`, base `d202b8c60240b6d2897c9deeda59be899bf47e24`.
- Actual extension owner checkout: `editor_integrations/vscode`, repository
  `sifr-lang/sifr-vscode`, base `732bcdc3ae2a494753025710dd138aa23a39b6e4`.
- Both owner branches will be `codex/latest-stable-item40-prepare`, with their
  existing independent submodule Git directories/indexes under this clone.
- Owned evidence/download/runtime/TMP/cache roots are respectively
  `/private/tmp/sifr-item40.iCyYG2/{evidence,downloads,runtime,tmp,cache}`.
  `TMPDIR` points to the owned tmp directory; npm and uv caches live under cache.
  Any Python environment is `/private/tmp/sifr-item40.iCyYG2/venv`.
  No shared target, cache, environment, index, or global toolchain is mutated.

## Exact source boundary

Root-owned paths:

- `.github/workflows/release-qualification.yml`
- `.github/workflows/release-publication.yml`
- `scripts/check_node_toolchain.py` (new canonical-selector/runtime preflight)
- `scripts/distribution/provision_marketplace_toolchain.sh` (new; registered
  before extraction after the 900-line publication workflow guard failure)
- `verification/areas/distribution_release/cases/node_toolchain_contract.sh`
- `verification/areas/distribution_release/cases/stable_publication_workflow_contract.sh`
- `verification/areas/distribution_release/node_toolchain_selftest.py` (new)
- `verification/areas/developer_tooling/check_vscode_extension.py`
- `demos/editor_candidate_qualification/run.sh`
- `demos/editor_candidate_qualification/README.md`
- `internal_docs/distribution_pipeline.md` (registered after initial Opus,
  before its documentation-only correction: maintained publication-toolchain
  description must match the new Node/npm selection and provisioning order)
- This scoped record and the Item 40 section of the phase ledger, if necessary.

Extension-owner paths only:

- `.node-version`, `package.json`, generated `package-lock.json`
- `.github/workflows/ci.yml`
- `scripts/setup-npm.sh` (new private-prefix exact npm provisioner)
- `README.md`

No TypeScript, Node types, VS Code engine/application, languageclient, vsce,
compiler, native dependency, Ruff, Python lock, or Item 33 changes. Local owner
commits may be recorded into real parent gitlinks in owner order; these refer
only to actual commits produced in the actual owner repositories. Nothing is
pushed or represented as delivered upstream.

## Selection and invariant

Official metadata rechecked on 2026-09-08:
<https://nodejs.org/dist/index.json> selects Node 26.8.1 (bundled npm 11.19.0);
<https://registry.npmjs.org/npm/latest> selects npm 12.0.2 and accepts Node
`^22.22.2 || ^24.15.0 || >=26.0.0`. The independent npm pin therefore requires
explicit private-prefix provisioning. `.node-version` and `packageManager`
remain canonical selectors; package engines/devEngines, lock root, workflows,
demo and active command runtime must agree. Missing owner checkout or mismatched
runtime must fail with actionable diagnostics before installation/native work.

## Named checks and cost/prerequisites

1. `bash verification/areas/distribution_release/cases/node_toolchain_contract.sh`:
   Python/text/runtime checks, seconds, no Cargo/native build.
2. `python3 verification/areas/developer_tooling/runner.py --suite editor-release
   --result-json target/verification/item40-editor-release.json`: all six named
   adapter variants, including `npm ci`, lint, typecheck, unit/extension tests,
   package under privately installed Node/npm; expected minutes and modest
   dependency downloads, no Sifr or Cargo build found in source inspection.
3. `PYTHONPATH=. python3 verification/areas/distribution_release/runner.py --suite qualification
   --result-json target/verification/item40-qualification.json`: nine offline
   Python fixture/planner tests; temporary Git repositories and toolchain version
   probes. Coordinator confirms there is no precise known failure of this exact
   offline suite; source inspection found it technically runnable, so it will run.
   Broad compiler gate failures do not withhold this independent named check.
4. Additional directly necessary diagnostic cases registered before execution:
   `python3 -m unittest verification.areas.distribution_release.node_toolchain_selftest`
   with exact cases `test_missing_checkout`, `test_selector_drift`,
   `test_missing_runtime`, `test_mismatched_runtime`, `test_matching_runtime`.
   Temporary metadata and command doubles only, seconds; no native work.
5. `git diff --check` in all three owned repositories, and
   `python3 scripts/check_file_size_guardrails.py` in root.
6. Additional necessary focused check registered before extraction/test:
   `bash verification/areas/distribution_release/cases/stable_publication_workflow_contract.sh`.
   The unchanged base publication workflow was exactly 900 lines; move the
   Marketplace toolchain provisioning responsibility into its own helper and
   update this contract's required invocation. Text-only Python/shell, seconds.

Download/checksum verification, private runtime extraction, npm's generated
`package-lock.json` refresh (`npm install --package-lock-only --ignore-scripts
--no-audit --no-fund`) and exact runtime version checks are necessary setup.
No broad suites, E2E, create-PR or merge gate are authorized in this preparation.
Heavy Cargo/native work requires separate coordinator capacity clearance.

## External state and review

Authenticated readback: compiler PR #3717 OPEN at
`98480c78587d6cbd99a7079d10c71825360bd468`; policy PR #3785 OPEN at
`4c7068b36904e216b02778f5664d2b8fd1159a6a`. Its one failed merge gate is retained
in <https://github.com/sifr-lang/sifr/pull/3785#issuecomment-5577596125>:
Python bridge E0560, callback E0425, and async SIFR-RESULT-0003; later qualification
was unreached. These are external and must not be retried or repaired here.

Freeze exact source SHAs across all three owners, then one Opus review using
Read/Grep/Glob only, explicitly listing withheld validation and preparation-only
acceptance. At most one remediation review. Evidence stays outside the source
tree keyed by exact candidate SHA. Return candidate/review/evidence/unreached
checks to coordinator and retire; do not start another item.
