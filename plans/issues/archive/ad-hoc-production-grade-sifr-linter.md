# Ad Hoc Phase: Production-Grade Sifr Linter

Status: completed on 2026-05-27

## Purpose

Build a production-grade Sifr linter by reusing as much of Ruff's linting architecture as is safe, while keeping Sifr lint semantics, rule IDs, diagnostics, suppressions, and editor behavior Sifr-owned.

The goal is not to port Ruff's Python lint rules. The goal is to avoid rebuilding proven infrastructure: config composition, file discovery, rule selection concepts, phase-gated lint orchestration, suppression mapping, fix application, test patterns, and LSP code-action patterns.

## Source Inputs

This phase is based on:

- Phase 36 tooling contracts in `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`
- Tooling analysis contract in `internal_docs/tooling_analysis.md`
- Tooling reuse strategy in `internal_docs/tooling_reuse_strategy.md`
- Current `sifr_lint` foundation in `crates/sifr_lint`
- Ruff linter docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/linter.md`
- Ruff configuration docs in `/Users/yaseralnajjar/work/sifr/ruff/docs/configuration.md`
- Ruff workspace/config crates in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_workspace`
- Ruff linter crate in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_linter`
- Ruff diagnostics crate in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_diagnostics`
- Ruff server LSP code-action and settings patterns in `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_server`
- Local review artifacts:
  - `reviews/sifr-linter-ruff-config-review.md`
  - `reviews/sifr-linter-ruff-registry-rules-review.md`
  - `reviews/sifr-linter-ruff-engine-review.md`
  - `reviews/sifr-linter-ruff-suppression-fixes-review.md`
  - `reviews/sifr-linter-ruff-file-discovery-review.md`
  - `reviews/sifr-linter-ruff-lsp-editor-review.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-1.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-2.md`
  - `reviews/sifr-linter-ruff-reuse-review-pass-3.md`

## Quality Contract

Entry criteria:

- Phase 36 is complete.
- `sifr_lint` exists as the Sifr-owned policy-rule crate.
- The Ruff linter reuse audit artifacts above are checked in and reviewed.
- This phase plan is reviewed and approved before implementation starts.

Exit criteria:

- `sifr lint` is production-grade for Sifr policy rules.
- Lint config, rule registry, suppressions, file discovery, fix applicability, LSP diagnostics, and code actions are implemented through Sifr-owned APIs.
- Language-neutral Ruff infrastructure is reused or adapted where practical.
- Python lint semantics, Python rule IDs, and Python project/module behavior do not become Sifr lint authority.
- Full local validation passes or any inherited unrelated gate failure is recorded with proof that this phase did not cause it.

Required quality controls:

- Hard compiler diagnostics remain unsuppressible and cannot be downgraded.
- Policy diagnostics are configurable, suppressible only by explicit Sifr rule IDs, and emitted through `sifr_diagnostics`.
- `sifr_lint` must not import `ruff_linter::rules::*`, `ruff_linter::registry` as production registry, `ruff_linter::linter` as production orchestration, `ruff_python_semantic`, Python project/module resolution, or Ruff Server diagnostic behavior as semantic authority.
- `sifr_lsp` must remain a protocol adapter. Lint diagnostics flow through `sifr_analysis` into `sifr_lint`.
- LSP suppression and fix code actions must be gated by a typed diagnostic class, not diagnostic-code string prefixes.
- Parser-aware suppression ranges must be implemented before adding syntax, HIR, or workspace lint rules.
- The parser-aware suppression gate must be mechanically enforced through Rust types. Syntax, HIR, and workspace lint-rule modules must depend on the parser-aware suppression API at compile time; if that API is absent or bypassed, those modules fail to compile or fail the linter reuse contract check.
- Fix-capable lint rules must define applicability, conflict handling, formatter interaction, idempotence, and safety tests before they are enabled.
- Any adapted Ruff code must be dependency-audited and Sifr-owned at the API boundary.

## Problem Statement

The current `sifr_lint` crate is a Phase 36 foundation. It provides:

- Sifr-owned rule metadata
- `RuleSeverity` and `RuleStatus`
- `# sifr: ignore[rule-id]`
- unknown, unused, and blanket suppression diagnostics
- a physical-line `trailing-whitespace` rule
- simple `.sifr` file collection

That is enough to prove the tooling boundary, but it is not a production linter. Production Sifr linting needs:

- real lint config in `sifr.toml`
- rule selection and severity resolution
- per-file ignores and discovery/exclusion parity
- parser-aware suppression mapping
- phase-gated lint orchestration
- syntax, HIR, workspace, and fix-capable rule families
- LSP/editor diagnostics and code actions
- regression checks that prevent Python lint dependencies from leaking into Sifr

Ruff already has battle-tested infrastructure for many of these concerns. Sifr should reuse that work where it is language-neutral, but not inherit Python lint semantics.

## Product Decision

Sifr will build a Sifr-owned linter with a Ruff-inspired architecture.

The architecture is:

```text
.sifr source
  -> sifr_syntax
  -> sifr_frontend / read-only HIR and workspace views
  -> sifr_lint
       - Sifr rule registry
       - Sifr config and rule selection
       - phase-gated lint runner
       - Sifr suppression/fix engine
  -> sifr_analysis
  -> sifr lint and sifr_lsp diagnostics/code actions
```

Ruff linter code is classified as one of:

- `reuse-direct`: use the same crate/API directly because it is language-neutral and already acceptable.
- `adapt`: copy or reimplement the pattern behind a Sifr-owned API after dependency audit.
- `reference-only`: use for design guidance; implementation is Sifr-native.
- `reject`: do not depend on it for production lint behavior.

## Ruff Linter Reuse Matrix

| Ruff area | Decision | Sifr requirement |
| --- | --- | --- |
| Ruff fork parser/AST/token/trivia/source ranges | reuse-direct through `sifr_syntax` | `sifr_lint` consumes syntax through Sifr wrappers, not raw linter entrypoints |
| `ruff_text_size`, source range utilities | reuse-direct where already in workspace | Keep source offsets/ranges compatible with Sifr diagnostics and LSP conversions |
| Ruff workspace config composition | adapt | Implement `sifr.toml` lint config with Ruff-style layering, extends, CLI/editor overrides, diagnostics for unknown keys, and cycle detection |
| Ruff `pyproject.toml` authority | reject | Sifr config authority is `sifr.toml`; Ruff lint config migration requires a separate reviewed migration phase |
| Python `target-version`, per-file target version | reject | No Sifr lint behavior depends on Python versions |
| Ruff plugin settings (`pyflakes`, `pycodestyle`, `isort`, `pydocstyle`, etc.) | reject | No Python plugin config leaks into Sifr |
| File resolver settings: include/exclude/extend-exclude/force-exclude/respect-gitignore | adapt | Use Ruff's product model with `.sifr` defaults and Sifr explicit-target semantics |
| `ignore` walker and `globset` matching | reuse-direct or adapt | Replace naive path matching with robust glob/gitignore discovery |
| Ruff package-root detection (`__init__.py`) | reject | Sifr uses Sifr workspace/package semantics, not Python package roots |
| Ruff cache key primitives | adapt later | Use Sifr cache namespace and keys covering source metadata, config, Sifr version, and rule registry revision |
| Ruff rule registry contents | reject | Sifr owns rule IDs, categories, status, docs URLs, and defaults |
| Ruff registry/code generation pattern | reference-only initially | Static registry is acceptable until rule count justifies macro generation |
| Ruff `RuleSelector` prefix/specificity model | adapt later | Add Sifr rule selectors once rule count/categories require it |
| Ruff rule redirects | adapt | Keep deprecated Sifr rule IDs working with explicit replacement metadata |
| `ruff_linter::rules::*` | reject | Python AST/Python semantic rule implementations are not Sifr lint rules |
| Ruff `SemanticModel` / Pyflakes binding model | reject | Sifr semantic lint rules use `sifr_frontend` and HIR views |
| Ruff linter `SourceKind` | reject | Sifr source kind is `.sifr`; notebooks are out of scope |
| Ruff linter phase ordering | adapt | Build a Sifr phase-gated lint runner: file, token, physical line, syntax, HIR, workspace, suppression, per-file ignores, fixes |
| Ruff AST checker implementation | reference-only | Implement Sifr syntax/HIR checkers natively |
| Ruff logical/physical line checker pattern | adapt | Use for line-based Sifr policy rules |
| Ruff `noqa` syntax | reject | Sifr suppression syntax is `# sifr: ignore[rule-id]` |
| Ruff `NoqaMapping`/directive lookup pattern | adapt | Implement Sifr parser-aware suppression mapping for multi-line statements/ranges |
| Ruff file-level blanket `noqa` | reject unless reviewed later | Blanket suppressions remain forbidden in this phase |
| Ruff diagnostic type | reference-only | Keep `sifr_diagnostics` as canonical diagnostic model |
| Ruff `Fix`, applicability, isolation, apply-fixes algorithm | adapt | Map to Sifr `SuggestionApplicability` and implement Sifr-owned conflict handling/source maps |
| Ruff code-action deferred resolution | adapt | Use for fix-all, rule suppression, and future organize/import actions through `sifr_lsp` |
| Ruff workspace edit tracker | adapt | Add version-aware edit tracking for lint fixes/code actions |
| Ruff fix-all | adapt with policy-only gate | Fix-all applies only safe policy fixes, never hard compiler diagnostics |
| Ruff organize imports/isort | reference-only or reject | Requires a separate Sifr import-organization lint/fix phase after Sifr import semantics are specified |
| Ruff server settings model | adapt | Add editor/global/workspace lint settings with config-preference behavior cleaned of Python options |
| Ruff LSP diagnostic data payload | adapt | Add typed diagnostic class and code-action metadata for hard vs policy diagnostics |

## Ruff Rule And Config Planning Decisions

This phase makes the rule and config decisions up front. Implementation PRs must follow this section and the Milestone 1 machine-readable manifest. No implementation PR may newly port a Ruff rule family, accept a Ruff config key, or reinterpret a rejected/deferred row without a reviewed update to this phase.

### Rule family audit

Scan source: `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_linter/src/rules`, Ruff 0.15.12 maintenance fork.

Disposition values:

- `sifr-native`: implement only if the same policy is meaningful in Sifr, using Sifr syntax/HIR/workspace APIs and Sifr rule IDs.
- `formatter-owned`: do not lint; formatter owns the behavior.
- `future-phase`: not part of this phase; requires a later reviewed product phase before implementation.
- `reject`: no Sifr rule in this phase.

| Ruff family | Disposition | Locked Sifr decision |
| --- | --- | --- |
| `airflow` | reject | Airflow framework rules do not apply to Sifr. |
| `eradicate` | sifr-native | Commented-out-code detection may be implemented as a token/comment policy rule after parser-aware suppression; no Python syntax classifier is reused. |
| `flake8_2020` | reject | Python `sys.version` rules do not apply. |
| `flake8_annotations` | future-phase | Sifr already requires static typing through compiler diagnostics; annotation style policy needs a Sifr type-style phase. |
| `flake8_async` | future-phase | Async policy must wait for documented Sifr async semantics; Ruff's Python async patterns are not reused. |
| `flake8_bandit` | future-phase | Security linting requires Sifr stdlib/runtime API rules; no Python security rules are ported. |
| `flake8_blind_except` | reject | Sifr does not use Python exceptions. |
| `flake8_boolean_trap` | sifr-native | Boolean-argument readability may be implemented as a Sifr syntax/HIR policy if Sifr call/parameter semantics warrant it. |
| `flake8_bugbear` | sifr-native | Individual bug-prone-pattern ideas may become Sifr-native policy rules only when backed by Sifr semantics; no direct rule port. |
| `flake8_builtins` | future-phase | Reserved-name policy needs Sifr namespace and stdlib conventions; Python builtin lists are rejected. |
| `flake8_commas` | formatter-owned | Comma placement is formatter-owned. |
| `flake8_comprehensions` | sifr-native | Comprehension simplification rules may be added only for Sifr AST constructs with equivalent semantics. |
| `flake8_copyright` | sifr-native | Header/copyright checks are file/comment policy rules if Sifr wants them. |
| `flake8_datetimez` | future-phase | Requires Sifr datetime API policy; Python datetime semantics are rejected. |
| `flake8_debugger` | sifr-native | Debug/probe call detection may be Sifr-native once Sifr debug APIs are named. |
| `flake8_django` | reject | Django framework rules do not apply. |
| `flake8_errmsg` | reject | Python exception message rules do not apply. |
| `flake8_executable` | sifr-native | Executable/shebang policy may apply to CLI scripts and demos through file metadata rules. |
| `flake8_fixme` | sifr-native | FIXME/TODO comment policy can be Sifr-native with Sifr-owned tag configuration. |
| `flake8_future_annotations` | reject | Python future-annotation rules do not apply. |
| `flake8_gettext` | future-phase | Requires Sifr i18n API policy. |
| `flake8_implicit_str_concat` | formatter-owned | String literal concatenation layout belongs to parser/formatter policy unless a Sifr semantic rule is later approved. |
| `flake8_import_conventions` | future-phase | Import alias conventions require Sifr import/package semantics. |
| `flake8_logging` | future-phase | Requires Sifr logging API policy. |
| `flake8_logging_format` | future-phase | Requires Sifr logging API policy. |
| `flake8_no_pep420` | reject | Python namespace package rules do not apply. |
| `flake8_pie` | sifr-native | Individual cleanup rules may be Sifr-native only when the Sifr AST/HIR pattern is equivalent. |
| `flake8_print` | future-phase | Print/debug policy depends on Sifr CLI/runtime conventions; no Python print rule is ported. |
| `flake8_pyi` | reject | Python stub-file rules do not apply. |
| `flake8_pytest_style` | future-phase | Requires Sifr test framework conventions. |
| `flake8_quotes` | formatter-owned | Quote style is formatter-owned. |
| `flake8_raise` | reject | Python raise/exception rules do not apply. |
| `flake8_return` | sifr-native | Return-control-flow simplifications may be Sifr-native after HIR control-flow support. |
| `flake8_self` | reject | Python `self` conventions do not apply as Ruff defines them. |
| `flake8_simplify` | sifr-native | Simplification ideas may be Sifr-native when proven semantics-preserving for Sifr HIR. |
| `flake8_slots` | reject | Python `__slots__` rules do not apply. |
| `flake8_tidy_imports` | future-phase | Import restrictions require Sifr import/package semantics. |
| `flake8_todos` | sifr-native | TODO comment policy can be Sifr-native with Sifr-owned tag configuration. |
| `flake8_trio` | future-phase | Trio-specific async rules do not apply; Sifr async policy belongs in a later phase. |
| `flake8_type_checking` | reject | Python runtime/type-checking import rules do not apply. |
| `flake8_unused_arguments` | sifr-native | Unused-argument policy may be Sifr-native if it stays policy-only and does not duplicate hard compiler diagnostics. |
| `flake8_use_pathlib` | future-phase | Requires Sifr filesystem/path API policy. |
| `flynt` | reject | Python f-string conversion rules do not apply. |
| `isort` | future-phase | Import sorting/organization is out of scope until Sifr import-order semantics and LSP organize-import behavior are specified. |
| `mccabe` | sifr-native | Complexity metrics may be implemented against Sifr CFG/HIR with Sifr thresholds. |
| `numpy` | reject | NumPy framework rules do not apply. |
| `pandas_vet` | reject | Pandas framework rules do not apply. |
| `pep8_naming` | future-phase | Naming conventions need a Sifr naming-style phase; Python naming exceptions are rejected. |
| `perflint` | sifr-native | Performance policy may be Sifr-native when based on Sifr HIR/codegen semantics. |
| `pycodestyle` | formatter-owned | Formatting-style rules are formatter-owned; only non-format text policies may become Sifr-native through separate rows. |
| `pydocstyle` | future-phase | Requires Sifr documentation-comment conventions. |
| `pyflakes` | sifr-native | Only policy checks such as unused imports/variables may be Sifr-native; undefined names and correctness remain hard compiler diagnostics. |
| `pygrep_hooks` | sifr-native | Text/comment/file pattern checks may be Sifr-native with Sifr-owned rule IDs. |
| `pylint` | sifr-native | Only individual Sifr-equivalent policy ideas may be implemented; Python object-model, dunder, exception, and import rules are rejected. |
| `pyupgrade` | reject | Python-version modernization rules do not apply. |
| `refurb` | sifr-native | Modernization/cleanup ideas may be Sifr-native only after a Sifr language-edition policy exists. |
| `ruff` | sifr-native | Ruff's own meta/text rules may be used as reference only; each shipped rule must be Sifr-owned. |
| `tryceratops` | reject | Python exception rules do not apply. |

Milestone 1 must encode this table in `verification/tooling/linter_manifests/ruff_rule_config_audit.json`. `check_linter_reuse_rules.py` must fail if a Ruff rule-family directory exists in the fork but is missing from the manifest. The manifest is pinned to the Ruff fork state at phase planning time, and the check must compare the manifest against the actual filesystem directories under the pinned fork. Any filesystem directory not present in the manifest is a failure. Milestone 5 must fail if a new Sifr rule references a row whose disposition is `reject`, `formatter-owned`, or `future-phase` without a reviewed phase update.

### Config surface audit

Scan sources: Ruff docs `docs/linter.md`, `docs/configuration.md`, `crates/ruff_workspace/src/options.rs`, and `crates/ruff_workspace/src/configuration.rs`.

| Ruff config/CLI surface | Disposition | Locked Sifr decision |
| --- | --- | --- |
| `select`, `extend-select`, `ignore` | adapt | Support in `[lint]` using Sifr rule IDs/categories, not Ruff prefixes. |
| `extend-ignore` | reject | Deprecated Ruff compatibility spelling; Sifr has no legacy Ruff config compatibility surface, so users must use `ignore`. |
| `fixable`, `extend-fixable`, `unfixable`, `extend-unfixable` | adapt | Support once fixes exist; validate against Sifr fix metadata. |
| `extend-safe-fixes`, `extend-unsafe-fixes` | adapt | Support as Sifr fix-safety overrides after M6; only policy diagnostics are affected. |
| `unsafe-fixes` / `--unsafe-fixes` / `--no-unsafe-fixes` | adapt | Use Sifr `disabled`/`hint`/`enabled` model; never applies hard diagnostics. |
| `per-file-ignores`, `extend-per-file-ignores` | adapt | Support with Sifr glob matching and Sifr rule IDs. |
| Inline `# noqa`, file-level `# ruff: noqa`, `--ignore-noqa` | reject | Sifr uses `# sifr: ignore[rule-id]`; blanket file suppression remains forbidden in this phase. |
| `preview`, `explicit-preview-rules` | adapt | Support preview/experimental Sifr policy rules through Sifr `RuleStatus`; no Ruff preview rule semantics. |
| `exclude`, `extend-exclude`, `include`, `extend-include`, `force-exclude`, `respect-gitignore` | adapt | Support with `.sifr` defaults and explicit-target semantics. |
| `extend` config inheritance | adapt | Support path-relative inheritance, ordered merge, cycle detection, and deterministic diagnostics. |
| `--config` path and inline config overrides | adapt | Support Sifr lint overrides after config loader exists; inline overrides use Sifr keys only. |
| `pyproject.toml`, `ruff.toml`, `.ruff.toml`, `[tool.ruff]` | reject | Sifr lint config authority is `sifr.toml`; migration mode requires a future reviewed phase. |
| `target-version`, `per-file-target-version` | reject | Python-version gates do not apply. |
| `extension`, Python file-type mapping, notebooks | reject | Sifr lint targets `.sifr`; notebook support is out of scope. |
| `src`, `namespace-packages`, `builtins`, `typing-modules` | reject | Python import/module behavior is not Sifr lint authority. |
| `dummy-variable-rgx` | future-phase | Only a Sifr unused-symbol policy can define ignored-name conventions. |
| `task-tags` | sifr-native | May support Sifr-owned TODO/FIXME tag policy if comment rules ship. |
| `allowed-confusables` | sifr-native | May support Sifr-owned Unicode/confusable policy. |
| `logger-objects` | future-phase | Requires Sifr logging API policy. |
| Plugin option blocks (`flake8-*`, `isort`, `mccabe`, `pep8-naming`, `pycodestyle`, `pydocstyle`, `pyflakes`, `pylint`, `pyupgrade`) | reject | Plugin config blocks are not accepted wholesale; Sifr-native equivalents need Sifr-owned config keys. |
| `line-length`, `indent-width` | formatter-owned | Formatter config owns formatting dimensions; lint may read only if a reviewed Sifr policy rule needs it. |
| `fix`, `fix-only`, `show-fixes`, `diff`, `exit-non-zero-on-fix` | adapt | CLI behavior follows Ruff's user model but acts through Sifr fix engine and Sifr diagnostics. |
| `output-format` | adapt | Start with human and JSON output; SARIF/GitHub/GitLab/JUnit require explicit Sifr schema tests before support. |
| `cache-dir`, `no-cache` | future-phase | Add only after lint caching has a Sifr cache key and invalidation contract. |
| `required-version` | future-phase | Consider later as a Sifr toolchain version guard. |
| `isolated` | adapt | Support no-config lint mode for CI/editor troubleshooting. |
| `watch`, daemon/server-specific settings | future-phase | Editor/LSP behavior is owned by `sifr_lsp`, not Ruff server. |

Milestone 1 must include these config decisions in the same `ruff_rule_config_audit.json` manifest. `check_linter_reuse_rules.py` must fail on accepted config keys that are absent from this audit, and on Ruff/Python config keys accepted without a `sifr-native` or `adapt` disposition.

The audit manifest schema is:

- `schema`: integer schema version.
- `ruff_fork_pin`: Ruff fork commit SHA or exact version tag used for the scan.
- `rule_family_source`: absolute or repository-relative path to the scanned Ruff `crates/ruff_linter/src/rules` directory.
- `rule_families`: array of objects with `name`, `directory`, `disposition`, `rationale`, and `sifr_requirement_note`.
- `config_sources`: array of scanned Ruff docs/source paths.
- `config_surfaces`: array of objects with `key`, `kind` (`config`, `cli`, `comment-directive`, or `plugin-block`), `disposition`, `rationale`, and `sifr_requirement_note`.
- `accepted_sifr_config_keys`: array of config keys accepted by Sifr lint in the current implementation.
- `rejected_ruff_config_keys`: array of Ruff/Python keys from this audit with `reject`, `formatter-owned`, or `future-phase` disposition that must fail if accepted as Sifr lint configuration.

Manifest validation must prove:

- every actual Ruff rule-family directory is represented exactly once;
- every accepted Sifr lint config key appears in `accepted_sifr_config_keys` and has an `adapt` or `sifr-native` disposition;
- every config surface with `reject`, `formatter-owned`, or `future-phase` disposition appears in `rejected_ruff_config_keys` until a reviewed phase update changes its disposition;
- no rejected, formatter-owned, or future-phase rule family/config surface is exposed as an enabled Sifr lint feature;
- the manifest fork pin and source paths match the checked Ruff fork.

## Linter CLI Parity Contract

Ruff's linter command is `ruff check`; Sifr's linter command is `sifr lint`. Sifr must not alias Ruff's lint behavior onto `sifr check`, because `sifr check` is already the hard compiler/type/ownership diagnostic command. The Sifr mapping is:

```bash
ruff check [OPTIONS] [FILES]...
sifr lint [OPTIONS] [FILES]...
```

Required command behavior:

- no positional files defaults to `.`;
- multiple file and directory targets are accepted;
- `-` and `--stdin-filename <path>` read from stdin and use the filename for config, per-file-ignore, discovery, and diagnostic path context;
- CLI options override `sifr.toml` and inherited config;
- global `--config <file-or-override>` and `--isolated` apply to lint as well as format after the lint config loader lands;
- lint-local `--output-format` takes precedence over global `--diagnostic-format` for `sifr lint` only;
- hard compiler diagnostics remain outside `sifr lint` rule selection, suppression, fix-all, and `--exit-zero` policy behavior;
- all CLI fixes operate only on policy diagnostics.

Exit status contract:

| Exit code | Condition |
| --- | --- |
| `0` | no lint violations, or all violations were fixed and `--exit-non-zero-on-fix` is not set |
| `1` | lint violations remain, `--diff` found fixable edits, or `--exit-non-zero-on-fix` observed applied fixes |
| `2` | invalid CLI arguments, invalid lint config, invalid rule selectors, invalid output format, or file discovery/config errors |
| `3` | internal compiler/linter failure caught by the panic boundary |

The lint CLI parity manifest is locked. Milestone 1 must encode this table in `verification/tooling/linter_manifests/lint_cli_parity.json`. `check_linter_reuse_rules.py` or a dedicated `check_linter_cli_contract.py` must verify the manifest against the implemented `sifr lint` clap surface and must fail if a Ruff lint CLI surface is unclassified.

The lint CLI parity manifest schema is:

- `schema`: integer schema version.
- `ruff_check_sources`: scanned Ruff docs/source paths used to build the matrix.
- `sifr_cli_sources`: scanned Sifr CLI source paths.
- `surfaces`: array of objects with `ruff_surface`, `sifr_spelling`, `disposition`, `implementation_milestone`, `rationale`, `conflicts_with`, `fixture`, and `notes`.
- `output_formats`: array of objects with `name`, `disposition`, `schema_contract`, and `fixture`.
- `exit_codes`: array of objects with `code`, `condition`, and `fixture`.
- `rejected_surfaces`: array of Ruff surfaces that must fail if accepted by `sifr lint`.

Manifest validation must prove:

- every Ruff `check` option and hidden compatibility spelling scanned for this phase appears exactly once;
- every implemented `sifr lint` option appears in the manifest with an `adapt` or `sifr-native` disposition;
- every rejected or future-phase Ruff surface is absent from the implemented clap surface;
- every required conflict pair is enforced by the CLI parser or by deterministic usage diagnostics;
- every implemented output format and exit code has at least one fixture.

| Ruff `check` CLI surface | Sifr spelling | Disposition | Required implementation milestone |
| --- | --- | --- | --- |
| `ruff check [FILES]...` | `sifr lint [FILES]...` | adapt | M2 |
| no positional files defaults to `.` | same | adapt | M2 |
| multiple files/directories | same | adapt | M2 |
| `-` stdin target | `-` | adapt | M2 |
| `--stdin-filename <path>` | same | adapt | M2 |
| `--select <RULE>` | same | adapt | M2 |
| `--extend-select <RULE>` | same | adapt | M2 |
| `--ignore <RULE>` | same | adapt | M2 |
| hidden/deprecated `--extend-ignore <RULE>` | none | reject | M1 manifest classification only |
| `--per-file-ignores <mapping>` | same | adapt | M2 |
| `--extend-per-file-ignores <mapping>` | same | adapt | M2 |
| `--fixable <RULE>` | same | adapt | M6 |
| `--extend-fixable <RULE>` | same | adapt | M6 |
| `--unfixable <RULE>` | same | adapt | M6 |
| `--extend-unfixable <RULE>` | same | adapt | M6 |
| `--fix` | same | adapt | M6 |
| hidden `--no-fix` | none | reject | M1 manifest classification only |
| `--fix-only` | same | adapt | M6 |
| `--diff` | same | adapt | M6 |
| `--unsafe-fixes` | same | adapt | M6 |
| hidden `--no-unsafe-fixes` | `--no-unsafe-fixes` | adapt | M6 |
| `--show-fixes` | same | adapt | M6 |
| hidden `--no-show-fixes` | none | reject | M1 manifest classification only |
| `--exit-non-zero-on-fix` | same; conflicts with `--exit-zero` | adapt | M6 |
| `--exit-zero` | same; conflicts with `--exit-non-zero-on-fix` | adapt | M2 |
| `--output-format <format>` | same | adapt | M2 for `concise`, `full`, `json`; later rows require explicit schema tests |
| `--output-file <path>` | same | adapt | M2 |
| `--statistics` | same; conflicts with `--show-files`, `--show-settings`, `--diff`, and `--watch` | adapt | M5 |
| `--show-files` | same; conflicts with `--show-settings` and `--statistics` | adapt | M2 |
| `--show-settings` | same; conflicts with `--show-files` and `--statistics` | adapt | M2 |
| `--preview` | same | adapt | M2 |
| hidden Ruff `--no-preview` | `--no-preview` | adapt | M2 |
| `--exclude <pattern>` | same | adapt | M2 |
| `--extend-exclude <pattern>` | same | adapt | M2 |
| `--respect-gitignore` / `--no-respect-gitignore` | same | adapt | M2 |
| `--force-exclude` / `--no-force-exclude` | same | adapt | M2 |
| `--no-cache` | same | future-phase | Requires Sifr lint cache contract before exposure |
| `--cache-dir <path>` | same | future-phase | Requires Sifr lint cache contract before exposure |
| `--watch` | none in this phase | future-phase | LSP owns editor watch behavior; CLI watch requires a later watcher/incremental phase |
| `--ignore-noqa` | `--ignore-suppressions`; independent from `--ignore <RULE>` | adapt | M3 |
| `--add-noqa` | none in this phase | future-phase | Bulk suppression insertion requires a migration policy and parser-aware suppression fix engine |
| `--show-source` / `--no-show-source` | none | reject | Deprecated Ruff spelling; use `--output-format full` or `concise` |
| `--target-version <py>` | none | reject | Python versioning does not apply |
| `--extension <ext:language>` | none | reject | Sifr lint targets `.sifr`; notebook/Python source-kind mapping does not apply |
| global `--config <file-or-override>` | same global Sifr flag | adapt | M2 |
| global `--isolated` | same global Sifr flag | adapt | M2 |
| Ruff log flags `--verbose`, `--quiet`, `--silent` | none in this phase | future-phase | Sifr needs one cross-command logging contract before adding these |

Required output-format decisions:

- `concise`: compact one-line policy diagnostics for terminal and CI logs.
- `full`: human diagnostics with source excerpts when source ranges are available.
- `json`: stable `RenderedDiagnostic` array matching Sifr's diagnostic schema.
- `json-lines`, `junit`, `grouped`, `github`, `gitlab`, `pylint`, `rdjson`, `azure`, and `sarif` are future-phase unless this phase adds explicit schema fixtures and docs for each format before implementation.
- `--statistics` prints a rule-count summary instead of regular diagnostics. If later combined output is desired, a reviewed update must define exactly how statistics interact with every output format.
- `--show-settings` prints resolved lint config, rule selection, file discovery settings, per-file ignores, preview state, output settings, and CLI overrides for the target.

Required CLI fixtures:

- default `.` and multi-target discovery;
- stdin with `-`, stdin with `--stdin-filename`, and stdin plus ignored positional file behavior;
- selector precedence across config and CLI;
- per-file ignore override behavior;
- `--show-files` and `--show-settings`;
- `--show-files`, `--show-settings`, and `--statistics` mutual-exclusion diagnostics;
- output-format and output-file behavior;
- exit-zero and exit-code matrix;
- rejected Python/Ruff-only flags;
- fix, diff, fix-only, unsafe-fix, show-fixes, and exit-non-zero-on-fix behavior once M6 enables fixes.

## Sifr Lint Architecture Requirements

### Rule ownership

- Sifr rule IDs are Sifr-owned.
- Rule metadata includes ID, summary, docs URL, default severity, status, source location, category, fix availability, and suppression complexity.
- Deprecated rules retain their IDs for at least two minor releases and point to replacements when possible.
- Rule categories must be Sifr concepts, not Flake8 or Ruff plugin categories.

### Diagnostic classes

Every diagnostic surfaced through analysis/LSP must carry a class:

- `Hard`: parse, type, ownership, result/option, runtime-safety, and workspace correctness diagnostics. These are unsuppressible and not part of fix-all.
- `Policy`: `sifr_lint` diagnostics. These can be configured, suppressed by explicit rule ID, and used for policy code actions.

LSP code actions must use this typed class. String-prefix checks such as `SIFR-LINT-*` are not sufficient as the production gate.

### Suppression complexity

Every policy rule must declare one suppression complexity:

1. `physical-line`: diagnostic is tied to a single physical source line.
2. `single-node`: diagnostic is tied to one syntax node.
3. `statement-range`: diagnostic can span a multi-line statement, block arm, function, class, match/case, or ownership/type construct.
4. `symbol-workspace`: diagnostic is tied to a symbol, HIR item, import graph, or workspace result.

Current line-based suppression is valid only for `physical-line` rules. Before any other category ships, `sifr_lint` must attach `# sifr: ignore[rule-id]` comments through `sifr_syntax` statement/range mapping.

M2 may expose only the current physical-line suppression behavior. M3 changes multi-line suppression attachment from line-attached to parser-aware statement/range attachment; M2 implementation notes and docs must call out that transition until M3 lands.

### Config ownership

Canonical lint config lives in `sifr.toml`:

```toml
[lint]
preview = false
select = ["default"]
extend-select = []
ignore = []
fixable = []
unfixable = []
unsafe-fixes = "hint"
include = ["*.sifr"]
exclude = []
extend-exclude = []
respect-gitignore = true
force-exclude = false

[lint.rules]
trailing-whitespace = "warn"

[lint.per-file-ignores]
"demos/generated/*.sifr" = ["trailing-whitespace"]
```

Required config semantics:

- CLI/editor overrides take precedence over discovered config.
- `sifr.toml` is authoritative.
- Ruff lint config files are not read implicitly.
- Unknown Sifr lint keys are deterministic diagnostics.
- Python-only Ruff keys are deterministic unsupported-option diagnostics only if a future migration mode explicitly reads Ruff configs.
- Extends are path-relative, ordered, cycle-detected, and tested.
- Explicit file targets are linted even when they match excludes unless `force-exclude` is active.
- `unsafe-fixes = "hint"` means unsafe fixes are surfaced as unavailable/user-confirmation-required suggestions but are not applied automatically. Future accepted values are `disabled`, `hint`, and `enabled`; `enabled` still applies only to policy diagnostics and never to hard compiler diagnostics.

### Lint runner phases

The production runner must be phase-gated. It should skip phases that have no enabled rules.

Required phases:

- file/discovery rules
- token/trivia rules
- physical-line rules
- syntax-node rules
- statement-range rules
- HIR/frontend rules
- workspace/import rules
- suppression filtering
- per-file ignore filtering
- fix applicability filtering
- deterministic diagnostic sorting

### Import organization boundary

Import sorting and import organization are not part of this phase unless a milestone explicitly adds Sifr import-order semantics first. Ruff's isort behavior is Python-specific. Any future Sifr import organization rule must define:

- Sifr import/workspace semantics used by the rule
- whether the rule is diagnostic-only or fix-capable
- interaction with package resolution and generated materialization
- formatter interaction and idempotence expectations
- LSP organize-import behavior, if any

### Fixes

Fix-capable rules must not ship until the fix engine supports:

- Sifr `SuggestionApplicability`
- safe vs unsafe policy
- non-overlap and grouped edit isolation
- source-map/edit tracking for LSP
- formatter interaction
- idempotence tests
- fix-all limited to safe policy fixes
- no fixes for hard compiler diagnostics

## Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | Ruff linter reuse matrix is encoded in machine-readable or checkable form before implementation starts |
| AC-2 | `sifr_lint` has a Sifr-owned rule registry with metadata, status, docs URLs, categories, fix availability, and suppression complexity |
| AC-3 | `sifr.toml` lint config supports rule selection, severity overrides, per-file ignores, include/exclude, gitignore, extends, and deterministic diagnostics |
| AC-4 | File discovery uses robust glob/gitignore behavior and preserves explicit-target semantics |
| AC-5 | Parser-aware suppression mapping supports multi-line syntax/HIR diagnostics before any non-physical-line rules ship |
| AC-6 | Hard vs policy diagnostic class is present in analysis/LSP diagnostic data and code-action gating |
| AC-7 | `sifr lint`, `sifr_analysis`, and `sifr_lsp` share one lint engine and produce equivalent policy diagnostics for the same source/options |
| AC-8 | LSP suppression code actions are offered only for policy diagnostics and never for hard compiler diagnostics |
| AC-9 | Fix-capable lint rules have applicability, conflict, source-map, formatter, and idempotence coverage before enabling fix-all |
| AC-10 | Guardrails reject `ruff_linter::rules`, Python semantic/project/runtime dependencies, Ruff rule IDs, and extension/editor-owned linter behavior |
| AC-11 | Docs explain lint config, rules, suppressions, fix safety, editor behavior, and non-reused Ruff/Python behavior |
| AC-12 | Full local validation passes before phase closure |
| AC-13 | A mechanical gate prevents syntax, HIR, and workspace lint rules from shipping before parser-aware suppression support is enabled |
| AC-14 | Unsafe fixes are never applied automatically unless the rule is policy-only, the fix is explicitly enabled, and the edit applicability permits it |
| AC-15 | Every Ruff rule family and Ruff lint config surface scanned for this phase has a locked `sifr-native`, `adapt`, `formatter-owned`, `future-phase`, or `reject` decision before implementation starts |
| AC-16 | `sifr lint` implements the locked Ruff-compatible CLI contract or has a reviewed manifest row for every unsupported Ruff `check` surface |

## Milestone Breakdown

Milestones are sequential. Each milestone closes with validation evidence and review before the next starts.

### Milestone 1: `lint_reuse_contract_and_manifests`

Goal: lock the Ruff reuse decisions into enforceable contracts.

Scope:

- create a linter reuse manifest matching this document
- create `verification/tooling/linter_manifests/ruff_rule_config_audit.json` matching the Ruff rule-family and config-surface audit in this document
- create `verification/tooling/linter_manifests/lint_cli_parity.json` matching the linter CLI parity contract in this document
- create a lint rule metadata manifest
- create `verification/tooling/check_linter_reuse_rules.py`
- make `check_linter_reuse_rules.py` verify:
  - `crates/sifr_lint/Cargo.toml` does not depend on forbidden Ruff/Python lint crates
  - `cargo tree -p sifr_lint` does not contain `ruff_linter`, `ruff_python_semantic`, Python project/runtime crates, or Ruff Server semantic behavior
  - production Sifr crates do not import `ruff_linter::rules`, `ruff_linter::registry`, `ruff_linter::linter`, `ruff_linter::noqa`, Python `Rule` IDs, or `ruff_python_semantic`
  - every directory under the pinned Ruff fork's `crates/ruff_linter/src/rules` appears in `ruff_rule_config_audit.json`
  - every Sifr-accepted lint config key is represented in the config-surface audit with an allowed disposition
  - no implementation code references a rejected, formatter-owned, or future-phase Ruff family/config key as an enabled lint feature
  - every Ruff `check` CLI surface scanned for this phase appears in `lint_cli_parity.json`
  - every implemented `sifr lint` option appears in `lint_cli_parity.json` with an allowed disposition and milestone
  - seeded negative fixtures fail the check
- create a placeholder lint config schema manifest
- create `verification/tooling/linter_manifests/suppression_gate.json`
- define the suppression-gate manifest schema:
  - `schema`: integer schema version
  - `gate_state`: `physical_line_only` or `parser_aware`
  - `allowed_rule_families`: array of `physical-line`, `single-node`, `statement-range`, `symbol-workspace`
  - `parser_aware_api`: Rust path that non-physical-line rule modules must depend on
  - `updated_by_milestone`: string milestone identifier that last changed the gate, such as `"m1"` or `"m3"`
- initialize the suppression-gate manifest with `gate_state = "physical_line_only"` and `allowed_rule_families = ["physical-line"]`
- make `check_linter_reuse_rules.py` validate the suppression-gate manifest path, schema, and state
- make `check_linter_reuse_rules.py` verify that any Sifr rule module whose `suppression_complexity` is not `physical-line` imports or depends on the manifest's `parser_aware_api` path, initially `sifr_lint::suppression::ParserAwareSuppressions`
- update internal docs to link this phase and the reuse audit artifacts

Validation:

- manifest self-tests
- Ruff rule/config audit manifest self-test
- lint CLI parity manifest self-test
- forbidden dependency guardrail and self-test
- `python3 verification/tooling/check_linter_reuse_rules.py`
- `python3 verification/tooling/check_linter_reuse_rules.py --self-test`
- suppression-gate manifest schema validation
- `git diff --check`

Review gate:

- external review confirms there are no unclassified Ruff linter subsystems or hidden Python semantic dependencies

### Milestone 2: `lint_config_and_file_discovery`

Goal: make lint configuration and file discovery production-grade.

Scope:

- implement `[lint]`, `[lint.rules]`, and `[lint.per-file-ignores]` in `sifr.toml`
- implement Ruff-inspired config layering, extends, overrides, unknown-key diagnostics, and cycle detection
- implement the non-fix portions of `sifr lint [OPTIONS] [FILES]...`: default `.`, multi-targets, stdin, rule selection, per-file ignores, output-format/output-file, show-files/show-settings, discovery flags, preview flags, `--exit-zero`, global `--config`, and `--isolated`
- split lint CLI argument modeling and execution into a dedicated `lint_cli.rs` module before expanding `cli_model_and_entrypoint.rs` or `check_and_package_commands.rs` beyond the hand-maintained file-size guardrail
- replace naive path matching with robust glob/gitignore discovery
- support include, exclude, extend-exclude, force-exclude, respect-gitignore, and explicit-target behavior
- add negative fixtures for deep directory traversal, ignored directories, symlink loops or cycles where supported by the walker, and pathological file counts within the local validation budget

Validation:

- `cargo test -p sifr_lint`
- config precedence fixtures
- lint CLI parity fixtures for all M2 rows in `lint_cli_parity.json`
- file discovery fixtures and negative tests

Review gate:

- external review confirms config/discovery reuse is language-neutral and Sifr-owned

### Milestone 3: `parser_aware_suppression_engine`

Goal: make suppressions correct for all future rule families.

Scope:

- replace line-only suppression attachment with parser-aware statement/range mapping
- expose a typed parser-aware suppression API, tentatively `sifr_lint::suppression::ParserAwareSuppressions`, that non-physical-line rule modules must use to register suppressible diagnostics
- support physical-line, single-node, statement-range, and symbol/workspace suppression complexity
- keep blanket suppressions forbidden
- keep blanket suppression reporting as a policy diagnostic; any future blanket suppression support requires a reviewed planning update and explicit feature/config gate
- implement `--ignore-suppressions` as the Sifr equivalent of Ruff's `--ignore-noqa`; it disables only policy suppression comments for the current lint run and does not affect per-file ignores or hard diagnostics
- report unknown and unused suppressions deterministically
- add multi-line suppression fixtures for calls, functions, classes, match/case, ownership/type constructs, and HIR diagnostics
- update `verification/tooling/linter_manifests/suppression_gate.json` to `gate_state = "parser_aware"` and `allowed_rule_families = ["physical-line", "single-node", "statement-range", "symbol-workspace"]`
- update `check_linter_reuse_rules.py` so any syntax, HIR, or workspace rule module that bypasses `ParserAwareSuppressions` fails validation

Validation:

- `cargo test -p sifr_lint`
- suppression contract checks and self-tests
- guardrail proving syntax/HIR/workspace rules fail validation if they bypass the parser-aware suppression API
- `python3 verification/tooling/check_linter_reuse_rules.py`
- suppression-gate manifest state transition check

Review gate:

- external review confirms non-trivial rules cannot ship with line-only suppression semantics

### Milestone 4: `phase_gated_lint_engine`

Goal: implement the Ruff-inspired Sifr lint runner.

Scope:

- add phase-gated orchestration for file, token, line, syntax, statement, HIR, workspace, suppression, per-file ignore, fix-filtering, and sorting phases
- preserve current rules through the new runner
- add phase-skip tests proving disabled rule families do not run
- add deterministic ordering and invalid-source behavior

Validation:

- `cargo test -p sifr_lint`
- lint engine phase fixtures
- performance smoke for large files/projects

Review gate:

- external review confirms orchestration reuses Ruff's structure without importing Python checker semantics

### Milestone 5: `sifr_policy_rule_families`

Goal: add production Sifr lint rule families beyond the foundation rule.

Scope:

- add representative token/trivia rules
- add representative syntax rules
- add representative HIR/frontend policy rules
- add workspace/import policy rules only where Sifr workspace/import semantics are already specified
- only add rules whose originating planning row is `sifr-native`; any rule inspired by a `future-phase`, `formatter-owned`, or `reject` Ruff family requires a reviewed planning update before implementation
- implement `--statistics` over Sifr rule IDs, including deterministic ordering and output fixtures
- classify every rule by category, suppression complexity, default severity, status, and fix availability
- keep the static `RULES` slice until the shipped policy-rule count exceeds 50. At that point, implementation must add a reviewed planning update before introducing a `RuleSelector` specificity system or macro-generated registry.
- keep hard correctness diagnostics out of `sifr_lint`
- explicitly exclude import ordering rules unless Sifr import-order semantics are specified in this or a later reviewed phase

Validation:

- targeted rule tests
- snapshot fixtures
- unknown/unused suppression fixtures
- full lint diagnostics parity across CLI and analysis

Review gate:

- external review confirms every rule is Sifr-semantic and no Python rule was ported mechanically

### Milestone 6: `lint_fixes_and_code_actions`

Goal: add safe lint fixes and editor actions.

Scope:

- M6a: implement Sifr-owned fix applicability and edit isolation using Ruff-inspired patterns
- M6a: implement fix conflict resolution, deterministic fix ordering, and synchronous code actions for safe policy fixes and explicit suppressions
- M6a: keep fix-all policy-only and safe-by-default
- M6a: implement fix-related lint CLI rows from `lint_cli_parity.json`: `--fix`, `--fix-only`, `--diff`, `--fixable`, `--extend-fixable`, `--unfixable`, `--extend-unfixable`, `--unsafe-fixes`, `--no-unsafe-fixes`, `--show-fixes`, and `--exit-non-zero-on-fix`
- M6b: implement source-map/workspace edit tracking
- M6b: add deferred code-action resolution for expensive edits and multi-file/workspace edits
- M6b: add version-aware edit conflict handling for LSP
- M6b: add `verification/tooling/check_linter_diagnostic_class.py` with a self-test, or extend `check_lsp_split_brain.py`, so validation fails if LSP code-action handlers offer suppression or fix actions for `Hard` class diagnostics

Validation:

- `cargo test -p sifr_lint`
- `cargo test -p sifr_analysis`
- `cargo test -p sifr_lsp`
- fix idempotence and conflict fixtures
- LSP code-action smoke/stress coverage
- linter diagnostic-class guardrail and self-test

Review gate:

- external review confirms hard diagnostics cannot be suppressed or auto-fixed

### Milestone 7: `lsp_editor_docs_and_closeout`

Goal: close the phase with editor parity, docs, and production evidence.

Scope:

- update `sifr lint` docs
- update LSP/editor integration docs for lint diagnostics, suppressions, fix actions, and settings
- update VS Code and non-VS Code editor contracts if lint settings or actions change
- update `internal_docs/tooling_analysis.md`, `internal_docs/lsp_server.md`, and verification docs
- run final local validation and production-readiness review

Validation:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh`
- editor asset checks
- VS Code extension contract/package checks
- final lint reuse/production-readiness review

Review gate:

- final review confirms the linter is production-grade, Sifr-owned, and maximally reuse-informed without Python semantic leakage

## Validation Plan

Every implementation PR must run the narrowest relevant local checks plus targeted lint tests. Phase closure requires:

```bash
cargo test -p sifr_lint
cargo test -p sifr_analysis
cargo test -p sifr_lsp
cargo test -p sifr
python3 verification/tooling/check_linter_reuse_rules.py
python3 verification/tooling/check_linter_reuse_rules.py --self-test
python3 verification/tooling/check_rule_suppression_rules.py
python3 verification/tooling/check_rule_suppression_rules.py --self-test
python3 verification/tooling/check_tooling_dependency_boundaries.py
python3 verification/tooling/check_lsp_split_brain.py
scripts/run_all_tests.sh --profile quick
scripts/run_all_tests.sh
```

## Review Requirements

- This planning phase must be externally reviewed before implementation starts.
- Each milestone must include review focused on Ruff reuse boundaries, Sifr semantic ownership, validation coverage, and docs.
- Phase closure requires final review confirming:
  - no `ruff_linter` Python rule engine dependency
  - no Python semantic/project/runtime authority
  - hard vs policy diagnostics are enforced
  - suppressions are parser-aware for non-line rules
  - LSP/editor behavior uses Sifr-owned lint diagnostics and actions

## Execution Log

- `2026-05-26`: phase drafted after local Ruff linter/config/server scan and Claude subsystem review passes.
