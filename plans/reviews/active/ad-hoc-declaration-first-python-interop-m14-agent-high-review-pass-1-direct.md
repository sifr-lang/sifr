# M14 Binding And Certification Authoring — Direct Full Review (PR #2994, `codex/m14-python-binding-certification`)

Single-agent review of the complete `main...HEAD` diff (53 files, +3,746/−330) plus the uncommitted profile remediation. All findings below were reconstructed independently from the diff and verified by code reading and live experiments against the branch binary in scratch packages under `/tmp` (removed afterward; no repository files were modified).

## Scope reviewed

- Plan/acceptance: `plans/issues/active/ad-hoc-declaration-first-python-interop.md:1945-2010` (M14 waves, tasks, acceptance, focused evidence).
- Implementation: `sifr_package` binding artifact + DLPack certification schema, `sifr_driver` scaffold rendering + protocol certification validation, `sifr` CLI (`python bind`, `python certify dlpack`, shared authoring context), codegen cross-module `PythonError` conversion, build-cache identity wiring.
- Evidence: `binding_authoring_tests.rs`, `arrow_certification_tests.rs`, `cache_tests.rs`, driver interop tests, `binding-authoring` verification suite, DLPack evidence fixtures, demo, public docs (`docs/python-interop.mdx`), durable architecture docs, area manifest/README, delivery profiles.

## Remediated blocker — verified, not just accepted

The prior named blocker (new blocking `binding-authoring` suite absent from delivery profiles) is correctly remediated by the uncommitted diff adding `"binding-authoring"` to the `python_interop` suite selections in `verification/profiles/create-pr.json:118`, `merge.json:110`, `nightly.json:123`, `release.json:122`. I verified:

- The suite name matches the manifest suite (`verification/areas/python_interop/manifest.json:67`) and the runner command registration (`runner.py:34`).
- All four modified profiles pass `sifr_verify.profiles.validate_selected_area_suites` (unknown-suite check) — ran locally, all OK.
- `profile_assignment_matrix.py` passes (`rows=17`) and `coverage_matrix_readiness` passes (`guarantees=13 surfaces=34 strict=yes`).
- I ran the suite end-to-end against the branch binary: `python interop binding authoring ok: sources=5 generated=4 untyped_failures=2 drift_checks=2 mutations=0` — matching the stated post-remediation evidence.

## What holds up well

- **Source precedence and fail-closed typing** are real and tested live: override → stub-package → py.typed → external-stub → introspection is enforced in `python_binding_probe.py:197-235`; `Any`/`object`/`Callable`/generics/missing annotations/overloads all raise (`python_binding_probe.py:55-84,132-139`), and the Rust renderer independently re-validates identifiers, parameter ordering, and a closed type allowlist (`crates/sifr_driver/src/python_binding.rs:165-235`). No path synthesizes a raw handle.
- **Determinism/read-only behavior**: `bind --check` uses a frozen lock mode and only re-probes in memory (`python_binding_cli.rs:217-283`); the suite's snapshot assertions confirm zero mutation on both success and failure paths, including `certify --check` (new snapshot guard in `example_packages.py:352-364`).
- **Artifact/fingerprint/cache integrity**: builds validate the artifact, generated-file digests, and consumed typing-source digests on every runtime load (`python_runtime_context.rs:139-173`), and binding identity now perturbs the package build-cache key (`project_codegen.rs:133-146`, with a unit test). Archive packaging includes the artifact, generated sources, and typing sources (`cargo/package.rs:157-159`).
- **DLPack certification exactness**: the fixtures instrument the actual managed-tensor deleter function pointer via ctypes and assert exactly one call, capsule consumption, pointer identity, and value equality *within one process run* (`fixtures/torch_dlpack/.../dlpack_evidence.py:56-99`); validation refuses `copy_performed`, non-1 deleter counts, and missing within-run assertions at every layer (fixture → CLI → artifact → build). `certify --check` re-executes fixtures and compares full evidence (`python_dlpack_certification_cli.rs:99-123`). Bridge targets are certified under the stable `bridge.*` logical identity with a unit test (`python_certification.rs:118-130,180-194`).
- **PythonError cross-module conversion** (`stmt_expr_method_and_question_mark.rs:314-364`) rebuilds the exact five contract fields plus the injected `__sifr_python_error` field, matching both the type-system contract (`python_interop.rs:3-33`) and the emitted struct; misclassification via the name-based `imported_project_functions` set is benign because both sides are contract-verified.
- Suites: all five source kinds, checked-in compilation, overload/`Any` rejection, ordinary/bind drift parity for typing-source drift, and non-mutation are genuinely exercised (`runner/binding_authoring.py`). Unit tests (61 passing, ran locally) cover artifact drift, escape, DLPack evidence rules, and the authoring digest's import-root independence.

## Findings

### Major 1 — `sifr python bind` relabels retained bindings to the new environment digest without revalidating them

- **File**: `crates/sifr/src/python_binding_cli.rs:150-151` (with `crates/sifr_package/src/python/binding_authoring.rs:303-326`).
- **Defect**: `load_python_bindings_for_update` deliberately ignores the recorded `environment_digest`, and `generate_binding` overwrites it with the freshly resolved digest while retaining all other modules' bindings unprobed. Since `digest_python_authoring_environment_probe` already excludes import roots (`digest_build_cache.rs:41-53`), the digest only ever differs when the underlying environment actually changed — exactly the case where retained bindings must not be silently revalidated by relabeling.
- **Proof (reproduced live)**: in a package whose resolved digest differed from the recorded one, `sifr check --frozen` correctly failed with "environment digest does not match"; running `sifr python bind math --symbols sqrt,ceil ...` succeeded, probing only `math`, and afterward `sifr check --frozen` reported "no errors found" with `_decimal`, `redis.client`, and `wrapt` bindings never re-verified. If the environment change had altered wrapt's stub signatures or version, builds would now pass against stale typed declarations (build-time shape probes catch arity drift but not annotation-type drift); only an explicit `bind --check` would detect it.
- **Remediation**: when the stored digest differs from the freshly resolved one, run the `check_bindings` re-probe/compare loop over every retained binding inside the same `bind` invocation and fail listing the drifted modules (or refuse and instruct per-module re-binding). Do not stamp the new digest onto unverified bindings.
- **Test required**: unit/suite fixture that authors module A, changes the recorded environment digest (or swaps a stub distribution version in the suite venv), re-binds module B, and asserts the command fails naming module A rather than emitting a relabeled artifact.

### Major 2 — `bind` writes the generated output before artifact validation; a failing bind clobbers another module's checked-in source

- **File**: `crates/sifr/src/python_binding_cli.rs:172` (write) vs `:198` (`write_python_bindings`, which performs duplicate-output/uniqueness validation).
- **Defect**: the generated `.sifr` file is written to disk before the updated artifact is validated. Any post-write validation failure (duplicate output path across modules being the direct one) leaves the filesystem mutated and inconsistent with the artifact. `safe_python_binding_output` also allows `--output` to silently overwrite any existing package file.
- **Proof (reproduced live)**: `sifr python bind wrapt --symbols ObjectProxy --output src/math_python.sifr` in a package where `src/math_python.sifr` is the recorded output of the `math` binding: the command exited 1 ("generated Python binding 'src/math_python.sifr' has drifted"), but `src/math_python.sifr` now contained the wrapt scaffold while the artifact still recorded the math binding — every subsequent `check`/`build` fails with drift until the file is manually restored.
- **Remediation**: assemble and validate the complete updated artifact (uniqueness, module ordering, all constraints not requiring the file on disk) before any filesystem write; additionally refuse to overwrite an existing file that is neither this module's recorded output nor marked with the generated header.
- **Test required**: suite case asserting that a bind whose output collides with another module's recorded output fails without modifying any package file (snapshot before/after).

### Minor 3 (actionable) — optional positional-only parameters generate declarations that `bind` accepts, ordinary `check` rejects, and `bind --check` still reports ok

- **File**: `crates/sifr_driver/src/python_binding.rs:267-297` (`render_parameters` drops the `/` marker and renders optional positional-only params as `name: T = python.omit`); probe records the kind faithfully (`python_binding_probe.py:96`).
- **Proof (reproduced live)**: override `def pow(x: float, y: float = 2.0, /) -> float: ...` → `sifr python bind math --symbols ...,pow` succeeds and checks in `def pow(x: float, y: float = python.omit) -> Result[float, PythonError]: ...`; `sifr check --frozen` then fails with `SIFR-PYCALL-0001: omittable positional parameter 'y' maps to a positional-only target parameter` (fail-closed — no runtime hole), while `sifr python bind --check` on the same snapshot reports `Python bindings: ok (4 module(s), 6 symbol(s))`. The authoring tool commits a package state that cannot compile, and its own read-only checker disagrees with ordinary check. Positional-only defaults are common in C-extension signatures, so this is reachable in normal use.
- **Remediation**: in `validate_declaration`, reject `PositionalOnly && optional` parameters with an explicit unresolved error (consistent with the "stop or emit an explicit unresolved marker" task), since the interop plan forwards omittable positionals by keyword and `sifr_ir::PythonParameterKind` has no positional-only variant.
- **Test required**: unit test in `python_binding.rs` for a positional-only optional parameter failing closed, plus a suite fixture mirroring the `pow` scenario asserting `bind` itself fails.

### Minor 4 (actionable) — binding validation accepts symlinked/out-of-package typing sources, unlike the certification fixtures hardened in this same PR

- **File**: `crates/sifr_package/src/python/binding_authoring.rs:201-208` (`path.is_file()` follows symlinks; no `symlink_metadata`/canonical-containment check), vs the new `validate_fixture` (`arrow_certification.rs`, symlink rejection + `canonicalize().starts_with(root)`).
- **Proof (reproduced live)**: replacing a consumed `typing/math_override.pyi` with a symlink to `/tmp/outside_override.pyi` passes both ordinary `sifr check --frozen` ("no errors found") and `sifr python bind --check` ("ok"). Content outside the package thus participates in build identity and satisfies drift validation, and `required_python_binding_archive_entries` references a path whose real bytes live outside the package. `bind`-time generation rejects symlinks (`python_binding_cli.rs:338-349`), so only the read/validate path is inconsistent.
- **Remediation**: in `validate_binding`, apply the `validate_fixture` discipline (reject symlinks, enforce canonical containment) to the binding output, overrides, and external stubs.
- **Test required**: unix-gated symlink test alongside `bridge_inventory_symlink_tests`.

### Minor 5 (actionable) — the new cross-module PythonError codegen path has no automated gate coverage

- **File**: `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs:314-364`, `module_prescan.rs:16-23`.
- **Gap**: the binding-authoring suite only type-checks (`sifr check`), the demo (`demos/m14_python_authoring`) exercises the path but is wired into no profile, and the E2E count is unchanged (131) — no fixture emits and runs this conversion. A regression would ship undetected by every gate.
- **Remediation/test required**: add an E2E pass fixture where `main` imports a project-module function returning `Result[_, PythonError]` and propagates the error under `try`/except, asserting runtime output; or register the demo's `bind --check` + compiled execution as a suite case.

### Minor 6 (actionable, low) — schema v2 certification artifacts fail with a serde "missing field" error instead of the schema-version diagnostic

- **File**: `crates/sifr_package/src/python/arrow_certification.rs` (`PythonCertificationArtifact.dlpack` is a required field; schema bumped 2→3).
- **Scenario**: an existing checked-in v2 `sifr.python-certifications.json` fails deserialization with `invalid '…': missing field 'dlpack'` before the intended "unsupported schema version 2; expected 3" message can render.
- **Remediation**: parse the header (`schema_version`) first and report the version mismatch, then fully deserialize. **Test**: v2-shaped artifact fixture asserting the message names the schema version.

## Prior suspected areas I could not reproduce

- **DLPack declarations bypassing certification when `certification_target` is `None`** (`python_certification.rs:78`): unreachable — every `PythonInteropPlanDeclaration` construction site sets `certification_target: Some(...)` (`python_interop_plan.rs:138,189,220`; `python_interop.rs:661`).
- **Byte-stream ambiguity in `python_binding_source_fingerprint`** (`binding_authoring.rs:72-103`): not exploitable — the fingerprint is recomputed from the same validated fields (symbol/source pairing, counts, and digest shape are independently enforced), so no two valid artifacts collide meaningfully.
- **Silent path drops in `relative_strings`** (`python_binding_cli.rs:376-390`): unreachable after `package_typing_sources` canonicalization; defensive only.
- **`DlpackCertifiedDevice::Any` vs specific device strictness** (`python_certification.rs:157-174`): exact-match is intentional exactness and strictly fail-closed.
- The demo's darwin-pinned SOABI/digest in `demos/m14_python_authoring/sifr.python-bindings.json` is not referenced by any gate (consistent with m12/m13 demos) and the README documents re-binding first; not actionable.

## Validation state

Known-good evidence accepted: original create-PR gate (Python interop 17/17, E2E 131/131, runtime platform 28/28, hardening 6/6). Independently re-verified in this session: `sifr_package` python unit tests (61/61), profile validators, coverage-matrix and readiness checks, and a live run of the remediated blocking `binding-authoring` suite (`sources=5 generated=4 untyped_failures=2 drift_checks=2 mutations=0`). Ignored as instructed: dirty `third_party/ruff`, prior invalid review artifacts. The two Major findings and Minors 3–4 were each confirmed with concrete reproductions against the branch binary.

The milestone's core machinery is sound and well-evidenced, but two confirmed Major integrity defects in the authoring flow (environment relabeling without revalidation; clobber-before-validate) plus three reproducible actionable Minors remain unresolved.

VERDICT: NEEDS CHANGES
