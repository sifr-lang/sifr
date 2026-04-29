# Review: milestone_diag_4a — Renderer + Workspace-Inference Slice (Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a` (uncommitted working tree on top of `73b4e32c`)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Pre-implementation review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-preimplementation-review-pass-1.md)

Slice scope (as stated by the implementer):

1. Add canonical human/compact/JSON presentation helpers in `sifr_diagnostics` that render from `DiagnosticSink` through `render_sink`.
2. Compact grouping uses `(severity, code, message_template, primary display file)`.
3. Remove workspace message-prefix diagnostic-code inference from `CompileError`.
4. Workspace/project discovery/build/test-runner construction sites carry an explicit `DiagnosticCode` where touched.
5. HIR `LoweringError` replacement and `CompilePhase::TypeCheck` deletion are deferred to later `diag_4a` waves.

## Verdict

**Mostly aligned with the stated scope, with a handful of concrete miss-fires.** The presentation helpers are well-shaped and tested against the canonical 4-tuple compact key. The workspace prefix classifier is gone and replaced by explicit `with_code` calls at every site that previously fed it. However the slice introduces a `code: Option<DiagnosticCode>` field on `CompileError` while leaving the legacy fallback bridge mapping all four `CompilePhase` arms to **retired** codes (`SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`); several "where touched" sites are migrated only syntactically (`{ … }` → `::new`) and never receive an active code, so they are now wired into the retired-code path more visibly than before. The legacy CLI renderer (`apply_diagnostic_recovery_limits`, `Severity::Help`) is intentionally untouched, which is consistent with the stated scope. No HIR or `TypeCheck` deletions leaked in.

## Slice fidelity

| Scope item | State |
| --- | --- |
| Canonical `render_sink_human/compact/json` helpers | ✓ landed in [crates/sifr_diagnostics/src/render/presentation.rs](../crates/sifr_diagnostics/src/render/presentation.rs) and re-exported from [crates/sifr_diagnostics/src/lib.rs:15-19](../crates/sifr_diagnostics/src/lib.rs:15) |
| Compact key `(severity, code, message_template, primary_display_file)` | ✓ [presentation.rs:145-162](../crates/sifr_diagnostics/src/render/presentation.rs:145) — matches the issue's contract exactly |
| Workspace message-prefix classifier removed | ✓ `CompileError::workspace_diagnostic_code` deleted ([diagnostics.rs:97-118](../crates/sifr_driver/src/diagnostics.rs:97)); regression test added at [tests/diagnostics.rs:60-69](../crates/sifr_driver/src/tests/diagnostics.rs:60) |
| Workspace/project/build/test-runner sites carry `DiagnosticCode` | Mostly ✓; partial misses listed under R1–R4 below |
| HIR `LoweringError` left untouched | ✓ no changes under `crates/sifr_hir/`; [module_lowering.rs:23-44](../crates/sifr_driver/src/frontend/module_lowering.rs:23) still bridges via `CompilePhase::TypeCheck` |
| `CompilePhase::TypeCheck` deletion deferred | ✓ enum and Display arm preserved ([diagnostics.rs:34-39, 217-225](../crates/sifr_driver/src/diagnostics.rs:34)); `cmd_check` panic boundary still uses `CompilePhase::TypeCheck` |
| `Severity::Help`, `apply_diagnostic_recovery_limits`, legacy CLI renderer untouched | ✓ — explicitly within scope (the renderer wiring is the next wave) |

## Concrete findings (in priority order)

### R1 — Legacy fallback bridge emits *retired* codes for any unmigrated site (high)

[crates/sifr_driver/src/diagnostics.rs:120-133](../crates/sifr_driver/src/diagnostics.rs:120):

```rust
fn diagnostic_code(&self) -> &'static str {
    if let Some(code) = self.code {
        return code.code();
    }
    // Transitional legacy bridge for unmigrated non-workspace paths in diag_4a.
    match self.phase {
        CompilePhase::Parse => "SIFR-PARSE-0001",
        CompilePhase::TypeCheck => "SIFR-TYPE-0001",
        CompilePhase::Codegen => "SIFR-CODEGEN-0001",
        CompilePhase::Build => "SIFR-BUILD-0001",
    }
}
```

All four codes are **retired** in the registry ([codes.rs:392-415](../crates/sifr_diagnostics/src/codes.rs:392)). This was the state pre-slice as well, so it is not a regression from the slice's own changes — but the pre-implementation review explicitly recommended re-pointing the bridge to *active* codes (`SIFR-BUILD-0002` etc.) when the workspace classifier was removed, precisely so the bridge would never emit retired strings into JSON/compact output and external `https://sifr.sh/docs/errors/SIFR-BUILD-0001` URLs.

The slice did not adopt that recommendation. Combined with R2–R4 below, the slice expands the surface that visibly relies on the retired-code fallback (sites like [build/entrypoint.rs:208-220](../crates/sifr_driver/src/build/entrypoint.rs:208) were previously *also* using struct literals without explicit codes — but since they were syntactically rewritten to `CompileError::new` here and several siblings were given codes, they now stand out as the only ones still falling through). At minimum, the bridge should map to active codes; ideally the slice would also migrate these few remaining touched-but-uncoded sites (see R3).

Additionally:

- [tests/diagnostics.rs:8-15](../crates/sifr_driver/src/tests/diagnostics.rs:8) and [tests/diagnostics.rs:17-29](../crates/sifr_driver/src/tests/diagnostics.rs:17) still positively assert `SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`. These tests are now codifying the retired-code fallback as the expected behavior. Either delete them or update them to assert active codes once the bridge is repointed.

### R2 — `materialize.rs` collapses cargo-build failures into `BUILD_MATERIALIZATION_FAILURE` (medium)

[crates/sifr_driver/src/build/materialize.rs:138-144](../crates/sifr_driver/src/build/materialize.rs:138) defines:

```rust
fn build_error(message: String) -> CompileError {
    CompileError::with_code(
        message,
        CompilePhase::Build,
        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
    )
}
```

…and every site in the file routes through it, including:

- [materialize.rs:116](../crates/sifr_driver/src/build/materialize.rs:116): `"failed to run cargo build: {error}"`
- [materialize.rs:120](../crates/sifr_driver/src/build/materialize.rs:120): `"cargo build failed:\n{stderr}"`

Both should be `BUILD_RUSTC_OR_CARGO_FAILURE` (`SIFR-BUILD-0005`). The parallel test-runner path already gets this distinction right at [test_runner/execution.rs:131-137](../crates/sifr_driver/src/test_runner/execution.rs:131) (cargo test failure → `BUILD_RUSTC_OR_CARGO_FAILURE`). The asymmetry is the smell — same operation, different code, no rationale. Splitting `build_error` into a filesystem helper (`BUILD_MATERIALIZATION_FAILURE`) and a cargo helper (`BUILD_RUSTC_OR_CARGO_FAILURE`) keeps both call patterns local.

### R3 — Touched sites that should carry a code but were left as `CompileError::new` (medium)

The slice scope says "where touched by this slice." The following sites were touched (struct literal → `::new` syntactic rewrites in the diff) but did not receive a `DiagnosticCode`. Each currently routes through R1's retired fallback.

- [build/entrypoint.rs:208-211](../crates/sifr_driver/src/build/entrypoint.rs:208) — `"internal error: rooted project entrypoint cannot be converted into a single-file frontend result"` → falls to `SIFR-BUILD-0001`. Per the pre-impl review R5, this is a compiler invariant and should be `INTERNAL_COMPILER_PANIC` (`SIFR-INTERNAL-0001`).
- [build/entrypoint.rs:215-219](../crates/sifr_driver/src/build/entrypoint.rs:215) — `"internal error: frontend lowering missing 'main' module"` → falls to `SIFR-TYPE-0001`. Same: `SIFR-INTERNAL-0001`.
- [project/discovery.rs:395-400](../crates/sifr_driver/src/project/discovery.rs:395) — `"failed to read '{}': {}"` → falls to `SIFR-BUILD-0001`. Discovery I/O failure; `BUILD_MATERIALIZATION_FAILURE` (or a fresh "discovery I/O" code in a follow-up) is the sensible home, and the file is unambiguously in scope ("workspace/project discovery").
- [project/frontend.rs:27-30](../crates/sifr_driver/src/project/frontend.rs:27) — `"[{module_name}] module was not parsed"` → falls to `SIFR-BUILD-0001`. Internal invariant; `INTERNAL_COMPILER_PANIC`.
- [test_runner/orchestrator.rs:88-95](../crates/sifr_driver/src/test_runner/orchestrator.rs:88) — `"missing parsed test module '{}' from '{}'"` → falls to `SIFR-BUILD-0001`. Internal invariant; `INTERNAL_COMPILER_PANIC`.
- [test_runner/orchestrator.rs:107-113](../crates/sifr_driver/src/test_runner/orchestrator.rs:107) — keeps the struct-literal form and forwards `error.code` from the HIR lowering bridge. `error.code` is always `None` today (since [module_lowering.rs:29-37](../crates/sifr_driver/src/frontend/module_lowering.rs:29) emits `CompileError::new`, no code), so the field is defensive but presently inert. That is acceptable scope-wise (HIR LoweringError replacement is a later wave), but the comment-free `code: error.code` reads like wired-up forwarding when it is not.

The first five fall squarely under "test-runner / build / project-discovery construction sites" called out in the slice scope and should ship with explicit codes in this slice or its immediate follow-up.

### R4 — `workspace/mod.rs` picks codes via message-text matching (low)

[crates/sifr_driver/src/workspace/mod.rs:177-190](../crates/sifr_driver/src/workspace/mod.rs:177):

```rust
fn source_root_error(source_root: &str, reason: &'static str) -> CompileError {
    let code = match reason {
        "escapes the workspace root via '..'" => DiagnosticCode::WORKSPACE_SOURCE_ROOT_ESCAPES,
        "is not a directory under the workspace root" => DiagnosticCode::WORKSPACE_SOURCE_ROOT_NOT_DIRECTORY,
        _ => DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
    };
    …
}
```

This is functionally equivalent to a typed dispatch (callers pass exact `&'static str` literals from a closed set), but it is *ironic* in a slice whose headline change is "Remove workspace message-prefix diagnostic code inference from `CompileError`." A future contributor changing the literal at one of three call sites will silently flip the code to `WORKSPACE_INVALID_SOURCE_ROOT`. Replace with an enum (or take the `DiagnosticCode` directly at the call site as the other migrated helpers do) so the intent is local and typed. Not a correctness blocker, but worth fixing while the surface is small.

### R5 — `discovery.rs` parse-error mapping is single-bucket (low)

[crates/sifr_driver/src/project/discovery.rs:402-426](../crates/sifr_driver/src/project/discovery.rs:402) routes every Ruff parse failure to `PARSE_EXPECTED_TOKEN_OR_RECOVERY` (`SIFR-PARSE-0002`). The same single-bucket choice exists in [frontend/api.rs:21-38](../crates/sifr_driver/src/frontend/api.rs:21) and [stdlib/bootstrap.rs:33-50](../crates/sifr_driver/src/stdlib/bootstrap.rs:33). The inventory and pre-impl review describe categorising Ruff `ParseErrorType` variants across `SIFR-PARSE-0002..0009`. This is a transitional shortcut and is acceptable for this slice as long as the next parser-transport wave splits the buckets; if the parser-transport is not the immediate next wave, leave a `// TODO(diag_4a): split by ParseErrorType` near these sites so reviewers don't accept the single bucket as final.

### R6 — Tests for the canonical helpers do not exercise internal (no-primary-span) diagnostics (low)

[crates/sifr_diagnostics/src/render/presentation.rs:229-292](../crates/sifr_diagnostics/src/render/presentation.rs:229) covers two cases:

1. Compact grouping by template + primary file across two source files.
2. Human/JSON pass-through equivalence between `render_sink_*` and `render_*_envelope`.

What is not tested:

- An `InternalDiagnostic` (no primary span) flowing through `render_sink_human` — the current implementation guards `if let Some(primary) = primary_span(diagnostic)` ([presentation.rs:63](../crates/sifr_diagnostics/src/render/presentation.rs:63)) so it should produce just the header + `url` line; assert it explicitly because internal diagnostics are how panic boundaries surface.
- Compact severity-summary line for a sink with mixed Error/Warning/Note diagnostics.
- A diagnostic with `help: Some(_)` or related spans rendered through the human helper.
- A grouping case where two diagnostics share `(severity, code, message_template)` but differ in primary display file (covered) *and* a case where they share *no* file because both are internal — these should stay in one group keyed on `primary_display_file: None`.

These are small additions in the existing `tests` module; they prevent regressions when the human renderer grows.

### R7 — `is_internal_compile_error` keeps the message-prefix check alive (low)

[crates/sifr/src/main.rs:260-265](../crates/sifr/src/main.rs:260):

```rust
fn is_internal_compile_error(error: &CompileError) -> bool {
    if error.code == Some(DiagnosticCode::INTERNAL_COMPILER_PANIC) {
        return true;
    }
    error.message.starts_with("internal compiler panic during ")
}
```

The `Some(INTERNAL_COMPILER_PANIC)` branch is now the structured path, but the `message.starts_with(...)` fallback is still required because the touched-but-uncoded invariants in R3 emit messages that begin with `"internal error: "` (note: not `"internal compiler panic during "`), which would *miss* the prefix check anyway and therefore be classified as user errors → `EXIT_USER_DIAGNOSTIC` (1) instead of `EXIT_INTERNAL_COMPILER_FAILURE` (3). Repro: an entrypoint with a path whose `file_stem()` is `None` (rare on Unix, plausible on Windows for malformed inputs) goes through [build/entrypoint.rs:165-171](../crates/sifr_driver/src/build/entrypoint.rs:165), but the `"invalid project entrypoint path"` site is at least coded as `BUILD_MATERIALIZATION_FAILURE` so it remains a user-facing build error. The two unmigrated invariants in entrypoint.rs (R3) hit the gap: their `"internal error: …"` text neither matches the prefix nor carries the panic code. This is a latent exit-code regression that the slice doesn't introduce but exposes by making the structured path the primary check.

Once R3 assigns `INTERNAL_COMPILER_PANIC` to those invariants, the prefix fallback can be deleted.

### R8 — `compile_order.rs` cycle phase change is harmless but undocumented (low)

[crates/sifr_driver/src/project/compile_order.rs:191-197](../crates/sifr_driver/src/project/compile_order.rs:191) flips the cycle-detection error from `CompilePhase::TypeCheck` to `CompilePhase::Build` and assigns `WORKSPACE_IMPORT_CYCLE`. Both changes are right (the issue's R5 expects exactly this rehoming), and the legacy `Display` label `"build error"` is more accurate than `"type error"` for an import cycle. Worth a one-line note in the PR description so reviewers don't misread it as a TypeCheck-deletion drift; otherwise correct.

## Test-coverage gaps that map to this slice's contract

- **No driver-level test asserts the canonical helpers integrate with workspace error construction.** Adding one `cargo test -p sifr_driver` test that builds a `DiagnosticSink` from a `CompileError::with_code(WORKSPACE_UNRESOLVED_IMPORT)` (via a thin shim) and asserts `render_sink_compact` produces the canonical 4-tuple grouping would tie the two halves of the slice together.
- **No regression test for the now-removed prefix classifier short-circuiting an explicit code.** The new test at [tests/diagnostics.rs:60-69](../crates/sifr_driver/src/tests/diagnostics.rs:60) covers "explicit code wins over a workspace-shaped message"; it does *not* cover "non-`Build` phase with a workspace-shaped message no longer infers `WORKSPACE-*`". Add a case with `CompilePhase::Codegen` and a message starting with `"could not resolve import"` and assert the code is *not* `WORKSPACE-0101`. The classifier's old `if self.phase != Build { return None; }` early-out makes this case already handled by deletion, but a test prevents regression.
- **No test asserts schema-stability of the new `DiagnosticEnvelope` JSON shape against the existing baselines.** This wave does not wire the canonical renderer into the CLI, so existing `crates/sifr/tests/verification/project/*/baselines/check-json.stderr.txt` continue to come from `compile_errors_to_diagnostics`. That is fine for now; flag explicitly in the PR description that baseline regeneration lands with the renderer-wiring wave so reviewers don't conflate the two milestones.

## Out-of-scope drift check (negative findings — *good*)

- No HIR file under `crates/sifr_hir/` is modified.
- `CompilePhase::TypeCheck` enum variant and `Display` arm are retained.
- `apply_diagnostic_recovery_limits`, `compile_errors_to_diagnostics`, `Severity::Help`, and the legacy `render_compile_errors` dispatcher in [main.rs:288-418](../crates/sifr/src/main.rs:288) all remain in place. The CLI is still on the legacy path.
- `LoweringError` and `LoweringResult` are untouched; `LoweringOutcome` from prior milestones remains unused.
- 91 fail fixtures and the two decimal verification baselines are untouched. This matches the slice scope and avoids contaminating this slice with the fixture re-keying that the HIR transport wave will need.

## Sequencing recommendations for the next wave

1. Decide whether to keep the bridge mapping retired codes or reroute it (`Build → BUILD_MATERIALIZATION_FAILURE`, `Codegen → CODEGEN_BACKEND_FAILURE`, `Parse → PARSE_EXPECTED_TOKEN_OR_RECOVERY`, `TypeCheck → INTERNAL_COMPILER_PANIC` until HIR transport lands). Whichever is chosen, update [tests/diagnostics.rs:8-29](../crates/sifr_driver/src/tests/diagnostics.rs:8) so the assertions pin the *current* policy rather than codifying retired strings.
2. Migrate the five touched-but-uncoded sites in R3 — they are all in files this slice already modified, so adding the `DiagnosticCode` argument is a one-line change per site.
3. Either split `build_error` in `materialize.rs` or accept that materialization and cargo-build failures share `BUILD_MATERIALIZATION_FAILURE`. The asymmetry with `test_runner/execution.rs` is the only correctness-relevant smell.
4. Replace the message-string match in `source_root_error` (R4) with a typed dispatch.
5. Add the four small renderer tests in R6 before the renderer wiring wave — they will catch the wiring slip when it happens.

## Validation snapshot (as reported by implementer)

- `cargo fmt --check` ✓
- `cargo test -p sifr_diagnostics` ✓
- `cargo test -p sifr_driver diagnostics` ✓
- `cargo test -p sifr --no-run` ✓
- `cargo clippy -p sifr_diagnostics -p sifr_driver -p sifr -- -D warnings` ✓

These cover the unit-level scope of this slice. The full e2e suite (`scripts/run_e2e_pass.sh`, `scripts/run_all_tests.sh --profile quick`) was not part of the reported run; this is acceptable because the CLI rendering surface is unchanged in this slice, but worth running before merge to confirm no workspace verification baseline drifted.

## Summary

The presentation helpers and the workspace-classifier removal are the cleanest parts of the slice and match the issue's contract precisely. The construction-site migration is ~85% complete: every workspace and test-runner code path that previously fed the prefix classifier now carries an explicit `DiagnosticCode`, which is the load-bearing piece. The five sites in R3 and the `materialize.rs` cargo-failure smell in R2 are the only places where the slice's "where touched" promise is honored only syntactically. The retired-code fallback in R1 is a pre-existing condition that this slice exposes more starkly without addressing; it should be the first thing the next wave fixes. None of the deferred work (HIR LoweringError, `TypeCheck` deletion, CLI renderer wiring, fixture re-keying) leaked in.
