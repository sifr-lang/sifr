I've reviewed the full M39.13 change surface against the phase doc, architecture doc, fixture matrix, compatibility matrix, validators, fixture READMEs, and docs pages. The validation suites pass, but they pass against self-declarations — the underlying scope/DoD picture is more concerning. Findings below, ordered by severity.

## Findings

### CRITICAL — M39.13's stated scope is not satisfied; the categorization moves work out of Phase 39

The phase doc is explicit that Phase 39 is self-contained:

- `plans/phases/39_rust_interop.md:48` "Any compiler, runtime, package, or verification plumbing missing for Rust interop must be implemented inside Phase 39 rather than deferred to another interop phase."
- `plans/phases/39_rust_interop.md:50` "Phase 39 has internal implementation checkpoints, not separate release phases."

M39.13's scope (`:259-264`) explicitly *requires* fixture evidence for "build scripts, proc macros, native links, and locked/offline Cargo behavior," plus ecosystem certification for "direct binding, local bridge, shared bridge, opaque handle, zero-copy, async, callbacks…" Its DoD (`:264-267`) is that "Every design capability has a passing positive fixture and a deliberate negative fixture" and "Phase closeout leaves no undocumented Rust interop gaps and no fixture family without both positive and negative evidence."

The compatibility matrix marks 11 of 31 rows `future-owned-by-separate-phase` with `planned` evidence in one or both directions:

- `native_build_script` (cc/bindgen/cxx/zstd) — both planned
- `proc_macro_trust` (serde_derive/prost-build) — both planned
- `cargo_locked_offline` — both planned
- `opaque_resource_matrix` (reqwest/rusqlite/tokio-postgres/redis) — both planned
- `async_runtime_reqwest` (tokio/reqwest) — both planned
- `callbacks_call_scoped` — both planned
- `callback_subscription_matrix` (tokio-tungstenite/redis/notify) — both planned
- `ecosystem_backend_certification` (axum/tower-http/sqlx) — both planned
- `ecosystem_cli_certification` (clap/tracing/tracing-subscriber/anyhow) — both planned
- `panic_boundary_wrapper_emission` — both planned
- `bridge_type_matrix` — positive planned

That set covers every Tier 3 fixture, half of Tier 4, plus the runtime-observed Tier 2 work. Re-labeling the gap "future-owned" does not change the fact that Phase 39 scope items were not completed inside Phase 39, contradicting `:48` and `:50`. The exit gate's advanced gate (`:380`) requires "Verification tiers 3-4 and the compatibility matrix are backed by local validation evidence" — Tier 3 currently has no passing evidence at all.

### CRITICAL — `future-owned-by-separate-phase` is being used as an in-Phase parking lot, not as a real phase handoff

`check_compatibility_matrix.py:131` only enforces that `future_owner` is non-empty:

```python
if not row.get("future_owner"):
    failures.append(f"{row_id}: future-owned row must name future_owner")
```

The actual `future_owner` values are descriptive labels, not phase IDs: "runtime resource certification," "runtime panic wrapper certification," "runtime async ecosystem certification," "runtime callback certification," "backend package ecosystem certification," "CLI package ecosystem certification," "native-link ecosystem certification," "proc-macro ecosystem certification," "locked/offline Cargo certification," "runtime bridge roundtrip certification." None of these are named subsequent phases, and the phase doc (`:50`) explicitly rules out separate release phases for this work. The category name implies a downstream owner that does not exist on the roadmap.

If this work is genuinely going to land later, the validator should require `future_owner` to match a known phase or tracked issue ID, and the phase doc must explicitly enumerate which milestones are actually deferred. As written, "future-owned" is indistinguishable from "not done."

### CRITICAL — "Supported" rows are not mechanically enforced; the validator only inspects self-declarations

`check_compatibility_matrix.py:120-125`:

```python
positive_status = _evidence_status(fixture, "positive_evidence")
negative_status = _evidence_status(fixture, "negative_evidence")
if category in CLAIMED_SUPPORT_CATEGORIES and (positive_status, negative_status) != ("passing", "passing"):
    failures.append(...)
```

`status: passing` is a literal string in `rust_interop_fixture_matrix.json` — the checker never runs a Cargo probe, executes a fixture, builds a sample package, or invokes the compiler. Every fixture directory under `verification/areas/rust_interop/fixtures/` contains only a `README.md`; there are no `.sifr` files, `Cargo.toml`s, baselines, or runners. So the chain "compatibility matrix → fixture matrix → fixtures" terminates in a README that says the fixture passes. That is not "mechanically enforced support" — it is documentation that mechanically matches other documentation.

The README at `verification/areas/rust_interop/README.md:10` calls this out: "The fixture matrix is contract-first." But the prose framing then equates "contract-first" with "self-declared." The phase exit gate (`:386-388`) says "Every Rust interop path… must lower through structured metadata into generated Rust" and "the verification area contains positive and negative fixtures for every supported capability" — that requires real fixtures, not READMEs that assert pass.

### HIGH — Public docs overstate evidence

`docs/rust-interop-compatibility.mdx:24` claims: "Direct compatible functions, including `crc32fast`, `blake3`, `sha2`, `uuid`, and compatible `regex` shapes | Passing cargo-probe positive and negative fixtures." But the corresponding fixture directories contain only READMEs (no cargo-probe artifacts). The same pattern repeats for opaque handles, panic policy, blocking diagnostics, zero-copy. The public statement "Passing cargo-probe positive and negative fixtures" is not supported by anything an external reader could verify.

This is the docs/verification alignment failure the matrix was supposed to fix.

### HIGH — Tier 4 Arrow/tensor/advanced-data rows claim "supported-through-bridge" while the M39.10 status note explicitly says runtime certification is staged

The matrix marks `arrow_record_batch`, `tensor_dlpack_bridge`, and `advanced_data_matrix` as `supported-through-bridge` (positive/negative both "passing" with scope `"contract-only"`). But the M39.10 status text on `plans/phases/39_rust_interop.md:215` says: "Runtime-observed crate-backed certification for `arrow`, `datafusion`, `polars`, `ndarray`, and `candle` remains staged for ecosystem closeout." So either:

- the rows should be `future-owned-by-separate-phase` (consistent with the M39.10 staging note and M39.13 being closeout), or
- the M39.10 staging note is now stale and should be removed.

As-is, the same crates are described as both staged-for-closeout (phase doc) and supported (matrix). The architecture doc (`internal_docs/rust_interop_architecture.md:678-680`) carries the same staging language, which compounds the contradiction.

### HIGH — Status note carry-forwards from earlier milestones have now become indefinite

The M39.7 status (`:165`) defers "Full runtime cancellation/shutdown fixtures and borrowed-input wrapper-future ownership" to `async_runtime_reqwest`. The M39.8 status (`:184`) defers "Full generated wrapper emission for package-local bridge calls" to `panic_boundary_wrapper_emission`. The M39.11 status (`:228`) defers "call-scoped storage rejection… callback invocation panic mapping" to `callbacks_call_scoped` and `callback_subscription_matrix`. The M39.6 status (`:150`) defers full opaque resource wrappers to `opaque_resource_matrix`.

Every one of those targets is now `future-owned-by-separate-phase` in M39.13. So the work the earlier milestones declared "tracked by M39.13" is now retracked out of Phase 39 entirely. This is the smell of a closeout that cannot close.

### MEDIUM — `bridge_type_matrix` "passing" negative without a passing positive contradicts the M39.4 DoD

M39.4 DoD (`:130`): "Supported type mappings roundtrip through Rust bridge calls." That requires *positive* roundtrip evidence. The fixture matrix records `supported_type_roundtrips` as `planned` — i.e., M39.4 was declared done without the positive evidence its own DoD required. M39.13 then categorizes the row `future-owned-by-separate-phase` instead of fixing the missing positive evidence.

If runtime roundtrip is genuinely out of scope, M39.4's DoD wording is wrong and should be amended; otherwise this is a debt M39.13 should not be allowed to defer.

### MEDIUM — `same_workspace_crate` is `tier=1` "supported" but `execution_kind: contract-only`

A tier-1 fixture is supposed to be "direct crate and local bridge build" (rust_interop_tiers.toml:11-12, phase doc `:276`). Marking the row `execution_kind: contract-only` makes it look like a tier-0 contract check. Either the tier is wrong or the execution_kind is.

The same shape applies to `shared_bridge_crate` (`tier=1`, `contract-only`). The compatibility matrix validator doesn't catch the inconsistency because it doesn't cross-check tier against execution_kind.

### MEDIUM — `blocking_diagnostics` requires runtime crates for a `compiler-diagnostic` fixture

`blocking_diagnostics` is `tier=0`, `execution_kind: compiler-diagnostic` (no Cargo build), but `required_crates: ["rusqlite", "rayon", "flate2"]` with pinned feature policies. A diagnostic-only fixture cannot meaningfully exercise those crates. Either the crates are not actually required (drop them) or the execution kind/tier is wrong.

### MEDIUM — Stale-drafts scan: rejection-context detector can be spoofed by trailing comments

`check_stale_drafts.py:69-87` accepts a stale pattern as "rejection context" if any of `rejected/reject/no/not/does not use/...` appears in the line *prefix* (before the match). That's adequate for the current corpus but fragile: a stray "no" anywhere earlier on the line legitimizes the pattern. Not a blocker, but worth tightening by requiring an inline comment marker (e.g., `# rejected:`) or fenced-block context.

### LOW — `docs/rust-interop.mdx:130-150` zero-copy example uses `device=cpu` symbolically

Architecture doc lists allowed Tensor metadata keys but not the symbolic device value spelling. Worth confirming the decorator grammar accepts bare `cpu` (vs. e.g., `device.cpu`) — but I don't see grammar prose that contradicts it. Flagging for cross-checking against the actual parser tests, not a review blocker on its own.

### LOW — Fixture READMEs are internally consistent but inverted in evidence model

Each new README states "Closeout category: future-owned-by-separate-phase. … is not claimed in Phase 39." That is honest documentation, but it amounts to saying Phase 39 ships without proving the contract for the very capabilities Phase 39 was supposed to certify. Worth a note that the README is the *only* artifact backing the row.

---

## Verification of explicit checklist items

**Support claims truthful and mechanically enforced** — No. Status strings are self-declared in JSON and the validator only checks that the strings say what the matrix says. No fixture directory contains executable test material; the chain bottoms out in READMEs.

**Every fixture family has positive/negative evidence and compatibility categorization** — Categorized, yes (31 fixtures all appear in both matrices, the row-count math reconciles). But "evidence" is `planned` in both directions for 10 families and in one direction for 1 family. The check correctly forces those into `future-owned-by-separate-phase`, but at the cost of leaving M39.13 scope unmet.

**No stale Rust interop draft syntax appears as accepted syntax** — The check correctly scans for `extern rust`, `from rust import`, `dlopen`, `crate=`/`path=`, legacy `native = [`, and `python`-fenced Sifr interop snippets; the rejection-context detector is weak but not currently abused (rust-interop.mdx uses explicit `# rejected:` markers). This part is in good shape.

**Docs, phase architecture, and verification area aligned** — Mostly self-consistent within the change, but inconsistent with prior milestone status notes (M39.4/6/7/8/10/11). The arrow/tensor/advanced-data "supported" categorization contradicts the M39.10 status note still present in the phase doc.

## Verdict

**M39.13 cannot be considered satisfied as written.** The DoD (`:264-267`) and Phase 39 self-containment rule (`:48`) are directly contradicted by:

1. Tier 3 has zero passing evidence; the exit gate's advanced gate (`:380-381`) cannot be claimed.
2. Eleven fixture families lack positive evidence, contrary to "no fixture family without both positive and negative evidence."
3. `future-owned-by-separate-phase` is being used as an in-Phase parking lot — labels are not named phases, and the phase doc rules out separate release phases for Rust interop.
4. The "supported" rows are not mechanically certified — every fixture directory is empty except for a README and the validator does not run any compiler/Cargo work.

**A final phase-level review should not proceed yet.** Before it can proceed, one of two things must happen:

- **Option A (close out properly)**: implement actual fixtures for the 11 future-owned rows under Phase 39, with real `.sifr` files, generated probes, baselines, and a runner the suite invokes — so the matrix's "passing" claim is verifiable.
- **Option B (amend the phase contract)**: amend `plans/phases/39_rust_interop.md` to (i) declare which milestones are formally deferred and to which named follow-on phase, (ii) drop the "self-contained" clause for those items, (iii) reduce M39.13 DoD to match what is actually being delivered, and (iv) make the validator require `future_owner` to reference a real phase or issue ID, not a free-form label.

Either path is defensible; the current state, which keeps both the DoD and the deferral, is not. My recommendation is Option B for the runtime-observed rows (they are genuinely large bodies of work) plus actually wiring up the contract-passing fixtures so the supported rows are no longer README-only — i.e., a partial Option A.
