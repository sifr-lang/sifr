# Ad Hoc Phase: Remove Manifest-Less Project Auto-Detection

Status: active bounded compatibility cleanup.

## Problem

Outside a `sifr.toml` workspace, the CLI reparses an explicit `main.sifr` and
inspects neighboring files to decide whether to compile it as a project. A
resolvable local `from ... import ...` changes the command from single-file
mode to project mode.

This behavior predates native workspace discovery. It was introduced by
[PR #818](https://github.com/sifr-lang/sifr/pull/818) and tightened by
[PR #819](https://github.com/sifr-lang/sifr/pull/819), when filename and import
heuristics were the only way to distinguish a multi-file project. Native
`sifr.toml` workspace detection was later added ahead of the heuristic without
removing it.

The resulting resolver has two authorities:

1. the nearest ancestor `sifr.toml` selects workspace/project mode;
2. in the absence of a workspace, `main.sifr` plus a resolvable sibling import
   can also select project mode.

The second rule is legacy compatibility behavior. It contradicts the current
manifest-less explicit-file model, makes `main.sifr` behave differently from
other filenames, parses source during mode selection, and silently chooses
single-file mode when that probe cannot read or parse the source. Sifr does not
retain backward-compatibility heuristics or fallback paths unless they are an
explicitly approved product requirement.

## Approved Command-Mode Invariant

Mode selection is structural and deterministic:

1. If the input is inside the nearest valid ancestor `sifr.toml` workspace,
   use project mode.
2. Otherwise, use single-file mode.
3. A discovered malformed workspace manifest remains a hard diagnostic. It
   must not fall back to single-file mode.

The resolver must not inspect the entrypoint filename, parse source imports, or
probe sibling modules to select a mode.

Manifest-less explicit files remain supported for learning and scripting.
Manifest-less local imports do not implicitly create a project; users must add
`sifr.toml` when they need a multi-file workspace.

## Scope

This phase is one bounded implementation item:

- Remove the manifest-less `main.sifr` import-sniffing branch from
  `resolve_compilation_mode`.
- Remove `has_local_project_imports` and imports or dependencies used only by
  that probe.
- Preserve nearest-ancestor `sifr.toml` discovery and hard diagnostics for
  malformed workspace manifests.
- Update `run`, `build`, `check`, `emit`, and `trace` coverage so each command
  follows the approved invariant.
- Convert tests that intend to exercise multi-file project behavior into
  explicit `sifr.toml` workspace fixtures.
- Replace tests that lock the legacy trigger matrix with regression tests that
  prove all manifest-less filenames and import forms remain single-file.
- Update `docs/cli_command_semantics.md` and `docs/package_management.md` so
  they describe one workspace boundary and no legacy project-entry category.
- Remove current non-archival `legacy project` terminology that refers to this
  mode. Preserve historical records under `plans/issues/archive/`.
- Audit checked-in demos and verification fixtures that depend on implicit
  manifest-less project detection. Add a fixture-local `sifr.toml` only where
  the fixture intentionally represents a multi-file workspace.

## Out of Scope

- Removing manifest-less single-file commands.
- Changing Sifr import syntax or module-resolution semantics inside a valid
  workspace.
- Changing Cargo-backed package graph, publishing, or dependency behavior.
- Adding `--project`, automatic manifest generation, warnings, compatibility
  aliases, deprecation periods, or fallback detection.
- Rewriting archived phase and review records.
- Expanding this item to separately owned package-session compatibility paths.
  Record any such path in its own issue if it remains reachable after this
  resolver cleanup.

## Acceptance Criteria

- [ ] Outside every `sifr.toml` workspace, `main.sifr`, non-main filenames, and
      files with or without local imports all resolve to single-file mode.
- [ ] A resolvable sibling module never activates project mode without
      `sifr.toml`.
- [ ] Inside a valid `sifr.toml` workspace, every valid entrypoint filename
      resolves to project mode independently of its imports.
- [ ] A malformed discovered `sifr.toml` produces the existing hard workspace
      diagnostic and never falls back to single-file mode.
- [ ] Mode resolution does not read or parse entrypoint source and
      `has_local_project_imports` no longer exists.
- [ ] Manifest-less local imports receive the normal single-file import
      diagnostic rather than triggering project compilation.
- [ ] `run`, `build`, `check`, `emit`, and `trace` agree on the same boundary.
- [ ] Multi-file tests and fixtures that require project behavior declare an
      explicit workspace.
- [ ] Current user documentation contains no legacy project-entry trigger
      matrix or import-based mode-selection rule.
- [ ] No compatibility flag, warning period, automatic manifest, or fallback
      path is introduced.
- [ ] Targeted CLI and driver tests, the create-PR gate, the file-size
      guardrail, and the final merge gate pass.
- [ ] The implementation receives the phase-closure review and is merged.

## Validation

During implementation:

```bash
cargo test -p sifr -- resolve_compilation_mode
cargo test -p sifr -- compile_entrypoint
cargo test -p sifr -- trace
python3 scripts/check_file_size_guardrails.py
scripts/run_all_tests.sh --profile create-pr
```

Run `scripts/run_all_tests.sh` once on the final reviewed implementation
candidate before merge.
