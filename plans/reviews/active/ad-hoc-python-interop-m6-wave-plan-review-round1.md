I have enough context. Let me finalize the review.

## M6 Wave Plan Review — Round 1

The five-wave breakdown captures the major architectural surfaces, but I found several ordering, boundary, and completeness gaps that should be resolved before implementation begins.

---

### High severity

**F1 — Wave 2 activates lowering before Wave 3 ships the loader; violates the milestone delivery rule.**
Wave 2 lists "declaration target activation" (i.e. lifts `SIFR-PYRES-0002` for `bridge.*` and lowers real call sites), but Wave 3 is the wave that installs the runtime `MetaPathFinder`, source table, and collision guard. If waves land as separate PRs (as M5 waves did), Wave 2's PR merges a compiling `bridge.*` demo that will `ModuleNotFoundError` at run time because no loader is installed. That's exactly the "public grammar without its sole production lowering" state the phase's Delivery Rule forbids. Fix by (a) moving activation into Wave 3 (or into Wave 4 alongside binary embedding), or (b) explicitly noting that Wave 2 leaves the `SIFR-PYRES-0002` gate in place and only Wave 3 lifts it. As drafted, ordering is inverted.

**F2 — Loader initialization order is not owned by any wave task.**
`python_interop_protocol_architecture.md` line 61 requires the bridge loader to be installed "after CPython initialization and bridge-loader installation, before user `main`" — i.e. inside `sifr_runtime::python::initialize_runtime` between `Py_InitializeFromConfig` (`crates/sifr_runtime/src/python.rs:294–325`) and any user path attach. The Wave 3 bullet "first-position reserved-namespace loader installation" is silent on *when* in the runtime bootstrap it runs, and Wave 5 evidence does not name a "loader-installs-before-user-main" fixture (only "loader ordering," which is ambiguous with `sys.meta_path` position). M7's asyncio loop cutover depends on this ordering; make it an explicit Wave 3 task and Wave 5 evidence line.

**F3 — Wave 3 traceback contract is not spelled out.**
Architecture line 381 requires "stable virtual filenames for tracebacks" that the loader wires into loaded modules. Wave 3 just says "virtual tracebacks." Without an explicit sub-task naming the filename scheme (e.g., `<__sifr_bridge__.p_<key>.<module>>`), how it flows into `co_filename` via `compile(..., filename, ...)`, and that it is stable across runs, Wave 5's "traceback evidence" has no fixed contract to check against. Add "assign, embed, and propagate a deterministic virtual filename into `co_filename` for each embedded module" as a Wave 3 task.

**F4 — Wave 3 collision rejection has no diagnostic-code assignment.**
The architecture says existing `sys.modules` entries in the reserved namespace are rejected as "a setup diagnostic," but neither M0's reserved first-code table nor Wave 3 assigns a code (e.g. a new `SIFR-PYIMP-000X` for reserved-namespace collision, a new code for dynamic-import-in-bridge, and one for third-party auth by a dependency bridge). Without wave-owned code assignments, Wave 5's "collision," "negative-inventory," and dependency-auth evidence points at unnamed error paths. Add explicit code allocations (and their registry entries) to the wave that first raises each.

**F5 — Wave 5 has no dependency-authority negative fixture.**
Architecture (arch line 370–371) and the milestone acceptance ("A dependency bridge cannot authorize its own third-party imports") make this a first-class invariant, but Wave 5's evidence list omits the negative case where a dependency's `python_bridges/*.py` imports an un-authorized third-party module and the root application rejects it. Wave 2 activates the authorization path; Wave 5 must prove it fails closed. Add "dependency bridge attempts to import a root-unauthorized third-party module → root-owned trust rejection" as an explicit Wave 5 fixture.

**F6 — "Resolved package identity" encoding is not owned.**
The architecture reserves `__sifr_bridge__.p_<resolved_package_key>.<module_path>` where the key is "a valid-identifier encoding of the resolved Sifr package identity." `SifrPackageId` today is `name@version#source` (`crates/sifr_package/src/graph/derive.rs:150–155`) — which contains `@`, `.`, `#`, and slashes, none of them valid Python-identifier characters. Wave 2 lists "resolved-package bridge identities" but no task defines the encoding function, its determinism/collision properties, or where it lives (`sifr_package::python::bridge_identity`?). This is the pivot of hermeticity — waves 3/4/5 all key off it. Add an explicit Wave 2 task: define, test, and document the encoding.

---

### Medium severity

**F7 — Wave 1 archive ownership vs Wave 4 archive coverage overlap is ambiguous.**
Wave 1 lists "archive ownership" and Wave 4 lists "archive/install/run coverage without a source checkout." The M6 task list has one bullet — "Include bridge source/inventory in archives and embed only the resolved graph in generated binaries" — spanning both. Split explicitly: Wave 1 extends `required_archive_entries` and `PackageArchiveEntry` (`crates/sifr_package/src/cargo/package.rs:121–139`) to include `src/python_bridges/**` and their inventory manifest; Wave 4 restricts the *binary* embed to the resolved graph and covers deployment from an unpacked archive. Duplicated shorthand invites two waves fighting the same code paths.

**F8 — Cache-identity terminology drifts between plan and architecture.**
Wave 4 says "declaration contracts, and typing inputs"; the M6 task list says "protocol contract, and typing inputs"; the architecture (line 391–393) says "the binding contract." Pick one term — the architecture-native "binding contract" is best — and align both surfaces. As drafted, a reviewer cannot tell whether "declaration," "protocol," and "binding" contract are three things or one.

**F9 — Wave 5 does not name a compiled-Sifr bridge live case.**
M5's Wave 5 named a specific runnable transaction demo; the phase's Verification Policy requires live evidence for supported capabilities. M6 Wave 5 says "live deployment" without naming a concrete compiled Sifr binary. The architecture (line 328–351) uses biip `parse_gtin` and schwifty BIC as canonical bridge examples. Wave 5 should explicitly name at least one compiled biip- or schwifty-backed bridge fixture (or another canonical package) so "live deployment" has a checkable target.

**F10 — Wave 1 does not own bridge Python-syntax diagnostics.**
"Syntax-check and digest their source" is an architecture requirement (line 355). Wave 1 says "static import analysis" and "deterministic source digests" but is silent on what happens when a bridge file is not valid Python. This should be a Wave 1 sub-task and Wave 5 negative fixture; it belongs adjacent to the dynamic-import rejection task, not conflated with it.

**F11 — Bridge inventory contribution to canonical requirement set is not wave-owned.**
`sifr_package::python::requirements::PythonRequirementKind::BridgeImport` already exists (`crates/sifr_package/src/python/requirements.rs:8`) but no wave lists "connect Wave 1 static-import inventory into `canonical_python_requirements` via `BridgeImport` contributions." Without that explicit task, Wave 2's "root-owned third-party import authorization" has no data source. Add it to Wave 2 (or the tail of Wave 1) and cite the existing enum by name.

**F12 — Non-standard source layout for `python_bridges/` is unspecified.**
`SifrManifest.source_roots` supports non-default source roots (`crates/sifr_package/src/manifest/sifr.rs:71`). Wave 1 says "package-owned `src/python_bridges/` source" but does not say whether the discovery path composes with custom `[source]` roots or is fixed to `src/python_bridges/`. Given hermeticity, fixed-path is defensible — but say so, and add a Wave 1 rejection diagnostic for `python_bridges/` outside the expected location.

---

### Lower severity

**F13 — "First-position" ordering guarantee is not verified after third-party mutation.**
Wave 5 lists "loader ordering." Architecture requires the loader to occupy `sys.meta_path[0]`, but user or bridge code may insert its own `MetaPathFinder`. Wave 5 evidence should specifically assert that even when a bridge or user code mutates `sys.meta_path`, resolution of a reserved-namespace import still routes through the Sifr loader (either by re-checking on each attach, by pinning position, or by name-space claim regardless of order).

**F14 — Wave 3 does not mention CPython C-API vs PyO3 high-level API.**
PyO3's high-level API does not expose `MetaPathFinder` registration convenience; the loader likely needs `ffi::PyImport_GetModuleDict`, `ffi::PyList_Insert`, and `unsafe` blocks. Given the crate-wide `#![warn(unsafe_code)]`, Wave 3 should note the runtime module scope (`crates/sifr_runtime/src/python.rs:1` already permits `unsafe_code`) and identify whether the loader is Python-source-based or C-level, so reviewers know which safety review path applies.

**F15 — Wave 4's "resolved graph only" needs an explicit closure predicate.**
"Embed only the resolved graph's bridge table" is architecturally correct but leaves ambiguity for dev-dependencies, unused bridge modules within the graph, and packages that publish bridges but whose declarations are not called in the current binary. Wave 4 should say whether the closure is "every bridge module of every package in the resolved graph" or "every bridge module transitively reachable from the entrypoint's declarations." The former is simpler; the latter is smaller. Pick one and record it.

**F16 — Wave 5 milestone-closure documentation update is unlisted.**
M5 waves explicitly closed the milestone with a doc update PR. Wave 5 says "milestone closure evidence" but not "update `plans/roadmap.md`, `internal_docs/architecture.md`, the capability matrix, and the M6 checkbox." Add these as concrete deliverables so the exit gate is unambiguous.

**F17 — `bridge.*` grammar reservation vs distribution name collision unowned.**
Architecture line 92–93 requires that a Python distribution named `bridge` is only reachable through a non-reserved Sifr target. This is a lowering/parsing invariant that should be tested in Wave 5's negative fixtures (or Wave 2's activation guard); no wave currently owns it.

**F18 — Wave 5 "negative-inventory" term is undefined.**
Wave 5 lists "negative-inventory" — but the M6 tasks and validation list use "static import inventory and rejected dynamic import fixtures." Rename to keep parity with the acceptance text, so the reviewer can map wave→task→validation 1:1.

---

VERDICT: CHANGES_REQUESTED
