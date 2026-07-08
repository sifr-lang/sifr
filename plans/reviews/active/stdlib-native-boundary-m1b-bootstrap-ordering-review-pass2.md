## Review of M1b stdlib native-boundary bootstrap ordering

### Requirements coverage

- Deterministic public bootstrap order enforced via strict `>` index check in `validate_public_stdlib_bootstrap_order` (`sources.rs:497-529`).
- Forward and cyclic public imports rejected — a cycle necessarily contains at least one forward reference in the linear STDLIB_SOURCES order, so the index check catches both; the Python script additionally has an explicit `_first_cycle` graph walk (`scripts/check_stdlib_bootstrap_ordering.py:149-175`).
- Private declarations enforced dependency-free by `validate_loaded_private_stdlib_declarations_are_dependency_free` — rejects any `_sifr.*` or `sifr.*` import from a private declaration (`sources.rs:531-550`).
- Static (compile-time) STDLIB_SOURCES ordering is checked via `validate_static_public_stdlib_bootstrap_order` inside `validate_stdlib_source_inventory` (`sources.rs:403`).
- Standalone guard wired into `verification/policy/guardrails.json:67-73` and `profile_runner.py:360-362` with both live check and `--self-test`; both consistent with sibling guards.
- Pass-1 advisory (duplicate detection coverage) addressed in `check_stdlib_bootstrap_ordering.py:256-262` with a `["sifr.a", "sifr.a"]` self-test entry.

### Consistency between Rust and Python enforcement

Rust `stdlib_imports` (`sources.rs:552-575`) and Python `_imports_with_prefix` (`scripts/check_stdlib_bootstrap_ordering.py:132-146`) apply the same strategy: strip inline `#` comments, trim, then match `from X import ...` or `import a, b, c` with `starts_with` prefix. Both handle `import ... as ...` and comma-separated imports; both correctly skip bare `import sifr` (no dot) and `_sifr.*` when scanning `sifr.` prefix (and vice versa). The tests exercise both forward and unknown-import failures on the load path and the equivalent conditions in the Python self-test.

### Test coverage

- New Rust tests cover forward public import, unknown public import, cycle (as forward reference on the earlier arm), private-→private import, and private-→public import (`sources.rs:706-789`). All error paths assert both message and path.
- Python `_self_test` covers forward, comma-form, unknown, cycle, private-→private, private-→public, unsorted-private and duplicate-public (`scripts/check_stdlib_bootstrap_ordering.py:183-265`).

### Non-blocking observations

- Backslash line-continuation and single-line compound statements (`if TYPE_CHECKING: from sifr.x import y`) aren't recognized by either parser. Grep of the stdlib confirms no such usage exists, so this is a documented parser limitation, not a real gap.
- Self-imports (`imported_index == index`) would pass both parsers because comparison is strict `>`. No stdlib module does this and the case is degenerate; not worth adding a check.
- `validate_stdlib_source_inventory` runs twice in `load_stdlib_tooling_sources_from_sysroot` (direct call at `sources.rs:370`, then again through `load_stdlib_sources_from_sysroot` at `sources.rs:388`). Harmless duplication.
- `plans/reviews/active/stdlib-native-boundary-m1b-bootstrap-ordering-review-pass1.md` and `-pass2.md` are both empty files. They appear to be untracked review scaffolding — worth confirming intent (populate with reviewer notes or delete before commit); doesn't affect implementation correctness.

### Verdict

READY
