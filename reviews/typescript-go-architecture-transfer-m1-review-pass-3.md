Review complete.

## Findings

Pass-2 blockers — both resolved:

1. **DIRECT_FS_PATTERN regex** — `verification/tooling/check_typescript_go_m1_guardrails.py:23-25` now uses `\.is_file\(\)` / `\.is_dir\(\)` (escaped backslash + escaped parens) inside the raw string. The new pattern correctly matches `lock_modes.rs:46`'s `if package_root.is_dir() {` (confirmed via direct probe and regex smoke test). Negative cases (`is_file()` without a leading dot, bare `read_to_string(...)`) still don't match.

2. **Inventory miss for `lock_modes.rs:46`** — `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:52` now records it under "Package offline availability" with documented effect ("Offline dependency validation probes whether package roots are available.") and M2 expectation. Cross-check: the script discovers 59 direct-fs sites, 0 missing from the doc.

Additional non-blocker touch-up also present: inventory heading at `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:30` now reads "Current Direct-Read, Probe, And Documented Effect Inventory".

Validation results (all green):
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → PASS (exit 0)
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` → PASS
- `python3 scripts/check_file_size_guardrails.py` → PASS (2014 files, 900-line cap)
- `python3 scripts/check_source_crate_dependency_direction.py` → exit 0
- `cargo fmt --check` → clean
- `git diff --check` → clean

M1 approved for PR
