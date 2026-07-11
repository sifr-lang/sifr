## Review

**Scope of change**
- One-line addition of `timeout=1200` to `run_checked(...)` at `verification/areas/sysroot_release/runner.py:332` for `emit migrated stdlib smoke`.
- No changes to assertions, artifacts checked, leakage validation, LSP smoke, doctor/self-update snapshots, or cheap sysroot metadata calls.

**1. Justified root correction, or masking a hang?**
Justified root correction. Confirmed:
- `run_checked` default `timeout=120` (runner.py:834) exactly matches the observed failure signature `timeout after 120s`.
- The immediately preceding boundary equivalence lane already executed the equivalent source-owned probe-cache population successfully under `timeout=1200` (runner.py:204). A source-tree emit + build that completes in ~538s under 1200s is not a regression; it is the new steady-state cost of populating the 403-adapter / 399-target bridge probe cache from cold.
- The smoke uses a fresh temp sysroot (runner.py:279–294) and, unlike the boundary equivalence lane, does not pre-warm a dedicated probe cache. First `emit` therefore inevitably pays the full cache-population cost.
- No behavior change: only the wall-clock budget is relaxed. If the underlying work truly hung, the 1200s cap would still fail visibly (exit=124), preserving detection.

**2. Bound and consistency with existing suite policy**
Yes. 1200s is the standing bound already in this file:
- `run_host_installed_stdlib_heavy` uses 1200s for installed check, emit, build, and cargo build offline frozen (runner.py:419, 428, 438, 492).
- Boundary equivalence uses 1200s for installed and source-tree builds (runner.py:204).
- The 900s used elsewhere (runner.py:245, 569) is for source-cargo/dist packaging, not installed emit — different work.

The new call site sits between two 1200s peers and directly extends the same precedent to the smoke's cache-cold installed emit. Cheap calls in the same block (sysroot JSON, self version/update dry-run, doctor, LSP smoke, leakage) correctly retain their smaller budgets — the scope discipline is right.

**3. Should it be applied elsewhere or documented differently?**
- The smoke's other installed calls (`--print sysroot --json`, `self version`, `self update --dry-run`, `doctor`, `doctor --json`, `lsp --stdio`) do not exercise the Rust bridge probe cache; leaving them at the default 120s / 60s is correct and preserves fast failure on real regressions.
- Rationale is already captured in `plans/reviews/active/ad-hoc-stdlib-compiler-boundary-phase-closeout-round5-sysroot-timeout.md`.
- Optional (non-blocking) future cleanup: consolidate the eleven repeated `1200` literals in this file into a named constant like `INSTALLED_HEAVY_TIMEOUT_S`. Out of scope for this corrective and not required.

**4. Blockers**
None. The change is minimal, targeted at the exact failing call site, consistent with existing bounded precedent, does not weaken any assertion, and does not risk hiding a real hang.

SATISFIED
