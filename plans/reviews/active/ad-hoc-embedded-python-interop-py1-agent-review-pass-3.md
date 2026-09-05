Verified the fix at environment.rs:441–453:
- `json_array` now returns `Result<String, PackageDiagnostic>`, replacing the prior `unwrap_or_else(|_| "[]")` silent fallback.
- Both call sites (lines 157–158) propagate with `?`.
- Failure path emits `DiagnosticCode::PYENV_PROBE_FAILED` → SIFR-PYENV-0004 (confirmed in registry_entries/python_interop.rs:45–54), with a "report this as a Sifr probe bug" hint matching the AGENTS.md no-fallback rule.

This resolves the only non-blocking note carried from pass 2 that touched the no-fallback policy. Pass 1 blockers remained closed in pass 2; no new diff since then beyond this `json_array` change.

Reviewer satisfied; no blocking issues.
