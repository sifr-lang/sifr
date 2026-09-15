# Item 42: Ruff release replay preparation

Status: preparation only, registered 2026-09-08 before source edits.

## Ownership and dependencies

This session exclusively owns `/private/tmp/sifr-item42.qVw420`, root clone
`codebase` on `codex/latest-stable-item42-prepare`, and actual Ruff clone
`codebase/third_party/ruff` on `codex/latest-stable-item42-replay`. Both clones
use independent Git indexes and object stores created with `--no-local
--no-hardlinks`; no alternates, shared targets or predecessor writes are used.
Temporary files, cache and evidence are the sibling `tmp`, `cache`, and
`evidence` directories. Parent and predecessor repositories remain read-only.

Latest main observed separately: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
Explicit root dependency/fork point: reviewed but UNMERGED Item 41
`2abbeee2e13a0076a66558d17987fcd6a06e21e4`. Actual Ruff dependency/fork point:
`d260c041f2c6149444364494b30701ae94eb0e1f`. Item 37 is merged. Item 41 is
source input, not a claim of prerequisite delivery. Item 42 review compares
against those dependency commits; Item 41 changes remain separately attributed.

## Source boundary

Recheck official stable Ruff (baseline 0.16.6); replay every Sifr fork patch
over that upstream release, retaining Item 41 Rust 1.98.1 and uv 0.12.10 pins.
Audit exact official release SHAs for fork-local checkout, upload-artifact,
attest-build-provenance, CodSpeed, cargo-binstall, setup-buildx, install-action,
run-on-arch and setup-mold. Include corresponding generated-workflow source
where needed. Root changes are only the real Ruff gitlink, `.gitmodules`,
submodule ownership expectations and directly stale current Ruff metadata.
No compiler, SDK, SQL, Item 43, unrelated upgrade or external owner repair.
Upstream release source/lock changes remain upstream imports, not independently
selected root dependencies. Native compatibility failures must be attributed.

## Named validation boundary

Eventual native checks, all WITHHELD pending explicit Police capacity release:

- In Ruff: `cargo test -p ruff_python_parser -p ruff_python_ast -p ruff_python_formatter`.
- Root `area developer_tooling: static, formatter`; inspection shows these
  expand to Cargo tests in rule-suppression and completion quality and to
  compiler builds/runs in formatter rules and AST coverage. Suite labels do
  not imply that their complete workloads are lightweight.
- Root `area core_language: syntax_parser_lexer_matrix`, a Rust validation
  suite compiling the compiler and fixtures.

Permitted preparation checks: official metadata/source comparisons, patch
coverage and action-pin audit; pure source-only variants from those named
suites after inspecting their entrypoints; `python3
scripts/check_submodule_ownership.py --self-test`, ownership guard,
`git diff --check`, and `python3 scripts/check_file_size_guardrails.py`.
Any skipped variant remains explicitly WITHHELD, never an aggregate pass.

Inspected source-only variants selected from the named suites: normal and
self-test entrypoints of `check_tooling_rules_lock.py`,
`check_tooling_dependency_boundaries.py`, `check_lsp_split_brain.py`,
`check_linter_diagnostic_class.py`, `check_no_pre_v1_compatibility.py`,
`check_pre_v1_regression_closure.py`, and `check_formatter_rules_manifests.py`;
self-tests only of `check_completion_quality.py` and
`check_formatter_ast_coverage.py`. These are all under
`verification/areas/developer_tooling`. Create the owned root `target`
directory first because completion-quality's self-test stores a fixture there.
Both rule-suppression variants, completion-quality positive, both formatter
rules variants and formatter AST positive remain WITHHELD native workloads.
No tool installs, lock resolution, native Cargo work, broad Sifr gate, PR,
push, publication or merge under the current preparation authorization.

## Review and terminal boundary

Additional directly necessary source paths registered before editing:
`verification/areas/developer_tooling/formatter_manifests/ruff_baseline.json`,
`README.md`, current Ruff selection lines in `internal_docs/architecture.md`,
and `crates/sifr_frontend/src/cache_keys.rs` solely the
`PARSER_OPTIONS_VERSION` release identity. The parser identity remains
gate-bearing for eventual delivery; no cache mechanism changes. Historical
performance/diagnostic provenance remains unchanged.

Generator/resolver workload proposed to Police but NOT released: private uv
0.12.10 and CPython 3.14.7, `uv lock --project scripts/benchmarks --python
<owned-python>` (no upgrades), then lock checks of all five fork projects.
Expected local benchmark version change only, 0.16.4 to 0.16.6. Private
cargo-dist 0.31.0 `dist generate --mode ci` after verifying its CLI/source,
expected `.github/workflows/release.yml` from the action commit map in
`dist-workspace.toml`. Current lock/generator inconsistencies are preserved
as explicit unfinished preparation obligations until authorized execution.
No generated lock or workflow is hand-edited and called regenerated.

### Actual resource release and expanded command registration

Police released the registered non-native batch after B49 naturally exited
and its process release check passed, with no Cargo/profiler remaining.
This supersedes the earlier proposed-only status for this batch. Native
compilation, broad gates and publication remain forbidden.

Prelaunch storage: 135 GiB available; owned codebase 929 MiB; owned target
4 KiB; owned cache empty. No cleanup is needed. All downloads, extracted
toolchains, Cargo/uv caches and temporary files stay under the owned item root.
The external `env.sh` records exact private paths. No shared/global install.

1. Hash-verify/extract official uv 0.12.10 arm64 macOS archive and cargo-dist
   0.31.0 arm64 macOS archive into `toolchains`. Use official Rust 1.98.1
   dated manifest to hash-verify the cargo, rustc and rust-std arm64 macOS
   archives, then their install scripts with `--prefix=<owned>/toolchains/rust
   --disable-ldconfig`. No source compilation. Private Cargo/Rust binaries,
   CARGO_HOME, RUSTUP_HOME and target paths are selected explicitly.
2. `uv python install 3.14.7` into the private Python/toolchain roots, then
   exact `uv --version`, `python --version`, `rustc --version`, `cargo --version`
   and `dist --version` identity checks. No project dependency installation.
3. From owned Ruff, `uv lock --project scripts/benchmarks --python 3.14.7`,
   then `uv lock --check --project PROJECT --python 3.14.7` for `.`,
   `python/py-fuzzer`, `python/ruff-ecosystem`, `scripts/benchmarks`, and
   `scripts/ty_benchmark`. No `--upgrade`; inspect every lock change and stop
   on unrelated drift. Expected benchmark local version 0.16.4 to 0.16.6.
4. `dist generate --help`, then `dist generate --mode ci` in owned Ruff.
   Source at cargo-dist v0.31.0 confirms this calls full guppy Cargo metadata
   with dependencies; Police explicitly authorizes the necessary declared
   dependency metadata/crate downloads into this private empty cache.
   No `--no-deps` substitute or native build. Expected generated output is
   `.github/workflows/release.yml`; preserve required existing customizations.
   Check generated output against exact tool/config, preserving frozen locks
   unless a registered upstream release input requires consistency.
5. Inspect final source, lock and workflow differences, record hashes and
   release the batch with actual remaining native obligations. Any different
   execution requirement must be reported before proceeding.

### First released batch result and newly concrete generator boundary

Private identities and all archive checksums matched: uv 0.12.10, managed
CPython 3.14.7, rustc/Cargo 1.98.1, cargo-dist 0.31.0. Benchmark `uv lock`
changed only local virtual package `scripts` 0.16.4 to 0.16.6; all five fork
lock checks passed. Existing benchmark Python-range/yanked-Astroid warnings
remain separately owned Item 41-F1, not suppressed or fixed here.

`dist generate --mode ci` completed full dependency Cargo metadata using
625 MiB private Cargo storage and changed no frozen Cargo lock. It then
exited 255: `'ci' is marked as allow-dirty in your dist config, refusing to
run`. cargo-dist v0.31.0 `run_generate` rejects that explicit contradiction.
Ruff intentionally has `allow-dirty = ["ci"]` because its release workflow
contains checksum-verified dist installation and other established release
customizations. Removing that setting and overwriting the workflow could
drop those controls. No workflow output was hand-edited or called regenerated.
The command failure is preserved and is not a validation pass.

All batch processes finished naturally; no owned Cargo/compiler/profiler or
provisioning process remained. Owned total storage was 3.1 GiB. Police was
notified of actual release and the newly required generator/customization
strategy. No native workload was started. Source review remains unused until
the source-preparation candidate is technically coherent.

### 42-G1 adjudicated changed-input reference generation

Police authorizes exactly one additional reference generation, with no native
build or permanent generation framework. Preserve `allow-dirty = ["ci"]` in
the delivered configuration and retain the customized release workflow.
Candidate fork `ce469c2a38ef19dfa41fa49715840e270a2cda04`, tree
`33e895a7c31d1b911fcffb9bd2da01f21d59ff2b`, is archived into the separately
owned `/private/tmp/sifr-item42.qVw420/reference` Git checkout. Its source
tree matches exactly before the only disposable configuration change:
`allow-dirty = ["ci"]` to `allow-dirty = []`.

Candidate configuration SHA-256:
`eb667cab05a88fb2e0bc176b7bad7d7dc81b72eb769847268109acc67b32a227`.
Reference configuration SHA-256:
`04722a38fad09b08505256d973af1b0454fcb9d361bb31507cfac3affe48e99e`.
Untouched customized workflow SHA-256 before action substitutions:
`943bb00690dd52f05b4ae2da64efe0cca3bfbaaee93124bab49f045fb1692072`.

Registered single invocation: source the owned `env.sh`, set
`CARGO_TARGET_DIR=/private/tmp/sifr-item42.qVw420/reference/target`, then run
the private cargo-dist 0.31.0 `dist generate --mode ci` from that reference.
Expected actual generated output: reference `.github/workflows/release.yml`.
Existing owned private tool/Cargo cache may be reused. Preserve original
exit-255 failure as a different-input attempt. Any new generation failure
stops this item; no retry or further scope absorption.

Use the actual generated reference to validate action identity/config mapping,
then update ONLY named in-scope action SHAs and labels in the maintained
customized workflow. Prove that all other bytes and YAML structures (verified
dist installer, release gate/environment/conditions/dependencies, secret and
permission boundaries, cache prohibition and publication order) remain equal.
The delivered output is CUSTOMIZED, never labeled generator-clean. Retain the
actual generated reference and exact mapping outside the reviewed Git trees.

The single changed-input reference generation passed (exit 0), output SHA-256
`ada12195e2daedb01fa846e892988afbe64aadea7455e01c8caedd82598ac5ff`.
Maintained customized workflow SHA-256 after exactly nine action substitutions:
`c1eab8be1ca43c490a4555c3bb7f5aaaffe19087ba9b90815b1ebca77139f98b`.
External `evidence/check_source_audit.py` and `action-audit-result.json` prove
all other bytes and YAML structure equal. All 130 live named-action callsites
match the official pins and published inputs: checkout 65, upload-artifact 37,
attest-build-provenance 4, CodSpeed 3, setup-buildx 4, run-on-arch 2, setup-mold
15. cargo-binstall/install-action have zero callsites after the upstream
development-tool migration; their official latest releases were still audited.
setup-mold has only the official `v1` tag, no GitHub Release; its exact commit
is pinned without inventing a release. Root ownership/diff/file-size checks
passed afterward (3,770 files). All reference/provisioning processes ended.

New source-observed delivery boundary: the unchanged root `Cargo.lock` still
records 14 Ruff path packages at 0.0.10 while their upstream 0.16.6 manifests
declare 0.0.12. This is separate from the coherent fork lock and benchmark uv
lock. No root resolver, hand-edit, native build or broad inventory fix was
performed. Police was notified before scope absorption; root locked native
qualification remains technically unready until this exact edge is assigned.

### 42-G2 root lock consistency registration

Police assigns the direct selected-Ruff root lock consequence to Item 42 and
releases selective noncompiling lock resolution plus locked metadata. This
does not authorize Item 49's broad inventory/vendor upgrades or native builds.
Recount corrects fourteen Ruff-prefixed packages to fifteen reachable fork
path identities, all 0.0.10 to 0.0.12: `ruff_annotate_snippets`, `ruff_cache`,
`ruff_db`, `ruff_diagnostics`, `ruff_formatter`, `ruff_macros`,
`ruff_memory_usage`, `ruff_notebook`, `ruff_python_ast`,
`ruff_python_formatter`, `ruff_python_parser`, `ruff_python_trivia`,
`ruff_source_file`, `ruff_text_size`, and `ty_static`. Each path is
`third_party/ruff/crates/<name>`. `ty_static` is an existing fork dependency
already represented in the root lock, not a new tool or owner.

Compared with the Item 41 fork dependency, the corresponding package dependency
tables and features are unchanged. Inherited workspace edges update the same
local package versions; no external dependency selection is being requested.
Root `Cargo.toml` path declarations/aliases (parser, AST, formatter, trivia,
source, text, literal) remain unchanged. The original root lock remains
recoverable at dependency commit `2abbeee2e13a0076a66558d17987fcd6a06e21e4`.

Before execution, the least expansive selected root resolver command is
private Cargo 1.98.1 `cargo update -p ruff_python_formatter --precise 0.0.12`.
No recursive option, broad update, hand-edit, build or test. Use the same
owned Cargo/tool cache and set root `CARGO_TARGET_DIR` to the owned
`codebase/target`. Inspect every changed/new/removed lock package and dependency
edge against selected upstream manifests. Expected delta is the fifteen local
versions only; stop before absorbing unrelated drift or an external owner.
Then run `cargo metadata --locked --format-version 1` without native execution,
record its output externally, and run root/fork diff guards. Vendor/catalog
consistency is required only for concretely necessary new selected edges;
no such external edge has been found by source inspection.

The selective resolver exited 0 and changed the fifteen expected local
versions, but also rewired twelve unrelated existing registry edges from
`windows-sys 0.61.2` to 0.60.2 or 0.52.0. No package was added/removed and
all registry versions/checksums stayed fixed. The twelve owners are
`anstyle-query 1.1.5`, `anstyle-wincon 3.0.11`, `errno 0.3.14`,
`nu-ansi-term 0.50.3`, `quinn-udp 0.5.15`, `rustix 1.1.4`,
`rustls-platform-verifier 0.7.0`, `socket2 0.6.4`, `stacker 0.1.24`,
`tempfile 3.27.0`, `term 1.2.1`, and `winapi-util 0.1.11`.
Their declarations were not changed by this Ruff release.

Original lock blob `7877c2ee5494afc233dcde7653236fc3856d83dc`, SHA-256
`21e7d7ea0b397059753a8f0c1c6796d77a5eb2522aed26f36fd77b8b7522fd52`.
Unaccepted complete resolver output SHA-256
`4a58ca332a2f21d189df074f460b1898829946b5266d6ac9f5b5d1c37940370a`,
27 additions/27 deletions, is preserved with external
`evidence/root-lock-resolver-unaccepted.diff`. Per the registered stop rule,
no hand-splitting, root locked-metadata check, native command or review follows
until the extra edges are attributed. The command ended naturally and its
resource reservation was released to Police.

#### 42-G3 conditional scope expansion and source proof

Police explicitly permits accepting the complete authentic resolver output if
all twelve old/new edges satisfy canonical source ranges and no other package,
checksum, feature-declaration or hidden lock change exists. The external
`evidence/windows-edge-source-proof.json` proves these conditions: all 923 lock
packages retain ordering and identity except the fifteen expected local version
changes; all 871 registry identities, versions and checksums remain fixed.
Exactly twelve dependency references change, with no other lock fields changed.
Eight inherited vendor manifests pass their recorded file/package checksums and
match inherited HEAD bytes. Four missing manifests are read from canonical
checksum-verified `.crate` archives without installing them. All twelve manifests
agree with official sparse-index requirements and requested/default features.

The old 0.61.2 and new 0.60.2/0.52.0 Windows selections are legal under their
unchanged declared ranges. `winapi-util`'s source `<=0.61.*` normalizes to index
`<=0.61`, including 0.61.2. These twelve changes are accepted as deliberate
resolver collateral under 42-G3, not required Ruff API changes, evidence of an
invalid inherited lock, or a Windows runtime/full-gate pass. Requested feature
declarations are unchanged; target feature unification is not runtime-qualified.
The original and initially unaccepted output/diff remain preserved under their
historical evidence names. No manual lock edit, split, forced pin or retry occurs.
Item49 may audit this separately but is not a new preparation dependency.

Before the conditionally authorized full locked-metadata check, read-only cache
inventory found 656 of the root's 871 registry package versions absent from the
owned Cargo cache. This is not the anticipated warm short run: substantial
declared-graph downloads may be needed. Exact proposed command is private Cargo
1.98.1 `cargo metadata --locked --format-version 1`, cwd the owned root, inherited
private environment with `CARGO_TARGET_DIR` overridden to the owned root target,
stdout/stderr captured under external `evidence/root-locked-metadata.*`.
Preflight: 130 GiB free, Cargo cache 661 MiB, root target 8 KiB. The expanded
download/resource edge was reported before launch; metadata remains WITHHELD
pending coordination. No update, native build, broad gate or review has run.

Police subsequently released the full locked-metadata window after B50's fixed
four diagnostic targets naturally completed (release_pass=true, remaining=[]).
The 656 absent declared package downloads are explicitly included, using the
same private Cargo 1.98.1/cache and registered root output/target paths. Recheck
capacity immediately before launch, preserve full metadata/stderr and prove both
root/fork lock hashes unchanged afterward. This supersedes only the metadata
hold; all native tests, broad gates and delivery remain separately withheld.

Full root locked metadata passed, exit 0, naturally in 5.48 seconds. The complete
2,708,770-byte JSON contains 555 dependency packages/resolve nodes and 37 workspace
members, with no no-dependencies or target-filter substitute. Actual root target
is the registered private root target. Root lock SHA-256 remains
`4a58ca332a2f21d189df074f460b1898829946b5266d6ac9f5b5d1c37940370a`;
fork lock remains
`09811229fe4b7af95a64ecd5e3f5dd6b5873deb2da1999687aef68d92cf91135`.
No source or lock mutation occurred. Root/fork diff guards pass, fork is clean,
Cargo cache is now 1.0 GiB and root target remains 8 KiB. Process inspection
found no owned Cargo, rustc or download process after natural completion.
Police received the single actual resource-release/readiness callback. The
candidate is source-coherent; the named native checks above remain WITHHELD.

### Focused native release and final preparation evidence

Police subsequently released the exact named focused native window before the
initial Opus review. Tests bind implementation root
`e2672511485b924f16feee0f3d52f980831cac16` and actual Ruff
`02140782a069963aebeff32bac02721eec8d5f3e`; this later section is a record-only
update, with no implementation or validation-input changes. Both independent
targets use private Cargo/rustc 1.98.1, Python 3.14.7, own cache/TMP and four
Cargo jobs. Sixteen previously passed source-only variants retain unchanged
inputs and are reused, not rerun.

- Fork locked parser/AST/formatter tests: PASS, 1,309 passed, three ignored,
  74.90 seconds.
- Root completion quality positive: PASS, 127.49 seconds.
- Root rule-suppression positive and self-test: PASS, 174.94 and 0.34 seconds.
- Root formatter-rules positive and self-test: PASS, 72.93 and 3.31 seconds.
- Root formatter AST coverage positive: PASS, 3.76 seconds.
- Root syntax/parser/lexer native matrix row and harness initially passed, but
  the area wrapper exited 1 afterward because the registered external result
  path violated its existing inside-repository reporting contract. The invocation
  error is this session's, not a source/native regression. Original complete
  `evidence/native-syntax-matrix.log` preserves the wrapper failure and raw passes.

Police 42-G4 released exactly one changed-output-location invocation of the same
adapter/selected row, with result path
`target/verification/areas/item42-core-language-results.json` inside this owned
ignored root. No source, expectations, validator or selection changed. This
invocation passed in 0.96 seconds, actual wrapper variants=1/failures=0 and row
PASS (187 ms); `evidence/native-syntax-matrix-corrected.log` preserves output.
The original failure is not relabeled. No other native tests were repeated.

All named developer_tooling static/formatter variants are now accounted for:
sixteen reused source-only passes plus six newly passed native variants. The
named core_language syntax_parser_lexer_matrix wrapper and row pass. These
are scoped suite results, not an aggregate broad-gate or Windows-runtime pass.
Both trees and both lock hashes stayed unchanged during native validation.
Final target sizes were root 13 GiB/fork 1.0 GiB with 110 GiB free, no cleanup.
After each released batch, no owned Cargo/rustc/build/run process remained;
Police received actual resource release promptly for the next owner's window.

Full logs, official provenance, patch/action audits, generator reference, lock
proofs and the SHA-bound review packet live outside the reviewed trees at
`/private/tmp/sifr-item42.qVw420/evidence`. This final record alone supersedes
earlier chronological WITHHELD native statuses. Source Opus is still unused
before the record-only freeze. Broad gates, Windows runtime, PR, push,
publication and merge remain explicitly withheld. Item 41 remains UNMERGED.

### Initial exact-source review and one remediation

Initial Opus review of root `f3afa622393c2de4816f183bd49408ce79b5246f` and fork
`02140782a069963aebeff32bac02721eec8d5f3e` found one in-scope omission: four
upstream-new workflow setup-uv steps still selected 0.12.9, conflicting with
the retained five project contracts requiring exactly 0.12.10. The review
otherwise verified replay, ownership, customized release mapping, complete
lock delta and canonical Windows edge proof. Full response remains external
at `tmp/sifr-claude.ijhNKt/response.md`; it is not a pass.

One bounded remediation changes only three `version` scalars in fork
`.github/workflows/ci.yaml` and one in `.github/workflows/sync_typeshed.yaml`
to the already selected 0.12.10. Actual fork remediation commit:
`0f7e9ce63515fb45859f884452e5741eb19741c1`. The root updates only its real
gitlink and this record. External `check_uv_remediation.py` proves the exact
four substitutions with every other byte unchanged, all 28 explicit setup-uv
versions at 0.12.10, all five project contracts at ==0.12.10 and workspace Rust
at 1.98.1. Two steps without an explicit version override match the inherited
Item41 workflow steps exactly; no new selection exception is introduced.
The affected named-action/customized-release audit passes again at 130 calls.

No native source, fixture, lock, selected test input, release generator config
or customized release workflow changed. Reuse the completed focused native,
sixteen static variants, five uv lock checks, full locked metadata, generator
reference and source-preservation evidence; no native/generator/resolver rerun.
Only one exact-SHA remediation review remains. A newly discovered mechanism
defect at that second review becomes later-owned work and stops this item.

Nonblocking review follow-ups are recorded, not implemented: the root uv guard
does not cover fork-local explicit versions (tooling infrastructure owner), and
upstream's new Hawk-specific job explicitly uses Rust 1.98.0 independently of
the selected 1.98.1 workspace toolchain (fork CI owner). Neither is absorbed as
a new mechanism or unrelated upgrade in this preparation.

One source-only exact root/fork candidate Opus review, at most one remediation,
using Read/Grep/Glob only and strict empty MCP. Include unmerged dependencies
and every withheld native/release obligation. Publish evidence only outside
the reviewed trees. A new second-review mechanism defect is later owned work
and ends this item. Return exact root/fork SHAs, evidence, reviews and remaining
dependencies, then retire. No subsequent item starts here.
