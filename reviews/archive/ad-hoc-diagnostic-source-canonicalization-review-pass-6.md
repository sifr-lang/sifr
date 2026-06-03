

Based on my review, here are my findings:

---

## Final Implementation-Readiness Review: Diagnostic Source Canonicalization (Pass 6)

**The phase is implementation-ready with three minor findings.**

---

### Finding 1: M4 cycle-trace scope is partially overstated (minor, non-blocker)

**What M4 claims:** "Preserve enough import-edge range data to render source-backed import cycle diagnostics."

**What `compile_order.rs` does today:** `compute_module_compile_order` (line 168) receives `parsed_modules: &HashMap<String, Vec<Stmt>>` — bare module names and AST statements, no import-range records. The `find_dependency_cycle_path` DFS at line 97 works entirely on module-name strings. The current cycle diagnostic at lines 213–219 renders module names as edges, not source spans.

**What achieving the M4 claim requires:** A new `compute_module_compile_order_with_ranges(...)` that accepts `HashMap<String, Vec<(Stmt, TextRange)>>` or an `ImportDependency`-carrying structure. The existing function signature must remain for backward compatibility with callers. The phase could alternatively add a parallel codepath that re-parses statements for range extraction at cycle-diagnostic time, with a test documenting why this is a lossy fallback.

**Severity:** Minor. The phase can still achieve meaningful cycle diagnostics (module-name edges as primary span) while deferring full source-range cycle tracing to a follow-up slice. The M4 behavioral requirements (primary span on one edge, related spans on others, cycle path in JSON) are achievable with the parallel codepath approach.

**What should change:** Add one sentence to M4 acknowledging the implementation approach for import-edge range extraction (fresh re-parse vs. parameter change vs. deferred pipeline extension) alongside the existing behavioral requirements.

---

### Finding 2: `check_diagnostic_source_canonicalization_contract.py` does not exist yet (minor, pre-flight)

**Status:** Only `check_diagnostic_presentation_contract.py` exists at `verification/tooling/`. The contract checker is required by:
- M1 line 424: "Wire the source-canonicalization contract checker and its negative self-tests into `scripts/run_all_tests.sh --profile quick`."
- Verification matrix row 12

**Severity:** Minor. This is M1 infrastructure — the contract must fail during M1 before any implementation. The phase author knows this. No implementation can begin until M1 adds the checker.

**What should change:** No doc change needed — the phase correctly tasks M1 with building this. Add a pre-flight note in the issue file header to state "M1 must be implemented first; all other milestones depend on the contract checker existing."

---

### Finding 3: Verification matrix row 6 needs an explicit list of "every flow" for ambiguous/collision states (minor)

**Status:** M3 line 457 requires parity in "every flow where those states can be constructed." The verification matrix (row 6) maps this to flow categories but does not enumerate which specific flows apply. From code audit:

- Ambiguous imports: occur in workspace discovery (`ResolutionFailureKind::Ambiguous`, `discovery.rs:211`), package graph derivation- Namespace collisions: occur in workspace discovery (`resolution_kind` dispatch, `discovery.rs:226`), package graph derivation
- Are there ambiguous/collision scenarios in single-file mode? (no — no discovery, no resolution)
- Are there scenarios in the CLI `check`/`build`/`run` paths? (yes — all go through project/package discovery)

**Severity:** Minor. The matrix categories are sufficient in practice. The gap is that a future implementer might miss a flow (e.g., the `sifr_driver::project::frontend` API path) and not add parity tests.

**What should change:** One sentence in M3 explicitly listing the flows: "single-file mode is exempt from ambiguous/collision parity (no discovery), but workspace/project mode and package mode in both CLI and driver API paths are in scope."

---

### No blockers remaining.

The prior five passes resolved:
1. `PackageImportOrigin` type correctness (pass2)
2. `PackageImportDependencyContext` with resolved-path fields (pass 1)
3. Missing `ImportDependency` and `PackageImportDependencyContext` structs (pass 2)
4. Sifr diagnostics SourceMap availability (pass 3)
5. Duplicate prevention requirements (pass2)
6. Contract guardrail wiring (pass4)
7. Edge cases for parser diagnostics (pass 4)
8. Ambiguous/collision flow parity scope (pass 4)
9. Registry activation gate — `SIFR-IMPORT-0005/0006/0007` active entries (pass 4)

All six milestones are correctly sequenced. The required codes are in the registry. The package diagnostic origin variants are correctly enumerated. M5 architecture (shared converter in `sifr_driver::diagnostics`) is consistent with existing code.

**The phase is ready for implementation.**
