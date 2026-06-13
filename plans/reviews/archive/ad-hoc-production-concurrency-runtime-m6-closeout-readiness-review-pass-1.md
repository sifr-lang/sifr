# M6 Closeout Audit

## Verdict: **FAIL** — only docs-cleanup blockers remain; the substantive DoD is met

The five DoD items are met in substance (evidence below). The only blockers are self-contradictory wording in the M6 design artifact that still says M6 implementation work remains — which prevents an honest "close" stamp even though the underlying engineering is done.

## DoD walk-through

| DoD bullet | State | Evidence anchor |
|---|---|---|
| "IPC is safe, typed, versioned, and panic-free." | ✓ | `ipc_schema` (FNV-1a-128 v1 hash + canonical descriptor), `ipc_frame` (length-prefixed Postcard, typed `IpcFrameError`, no data-dependent unwrap/expect), `ipc_connection` (protocol overlap + schema identity/range, max-frame negotiation). |
| "IPC is explicitly layered over an accepted process/transport substrate and has deterministic close, backpressure, cancellation, and malformed-frame behavior." | ✓ | Substrate layering recorded in design "Transport Boundary". `ipc_transport` + `ipc_process_pipe_fixture` cover bootstrap, completion, in-flight cancel, shutdown/`Terminating`, bounded backpressure, malformed frames, unsupported payloads on real Unix child stdin/stdout. |
| "Unsupported payloads are compile-time diagnostics where possible." | ✓ | `SIFR-OWN-0013` via compiler-erased `sifr.ipc.require_serializable(...)`, recursive eligibility in `sifr_lowering`, `ipc_schema_extraction` for accepted concrete type graphs with `IpcSchemaType::Unsupported` carrying rejected-type evidence. |
| "Sendability/shareability and IPC payload eligibility diagnostics pass representative fixtures." | ✓ | `ipc_payload_require_serializable_basic` (pass) + `ipc_payload_process_resource_rejected` / `ipc_payload_sync_endpoint_rejected` (fail). Both in create-pr and merge e2e manifests. |
| "All M6 CPython process-pool/multiprocessing test families are classified with shared evidence states." | ✓ | Focused `sifr.ipc` missing-member fixtures for `ProcessPoolExecutor`, `Process`, `Queue`, `Pipe`, `Pool`, `fork`, `forkserver`, `shared_memory`. Classifications recorded in design "CPython-Shaped API Classification". |

Windows process-pipe fixture being `host-limited` is consistent with the phase contract's accepted evidence states (it never asserts cross-host parity), and the public process-worker pool is explicitly out of M6 per the user's framing.

## Blocking findings (docs-only)

1. **Design doc Status line still says "In progress"** and asserts `Generated worker integration remains M6 implementation work` (M6 design doc excerpt, Status paragraph). Closing M6 while the artifact literally claims M6 work remains is internally inconsistent.

2. **Multiple traceability/host-matrix rows still describe generated worker integration and similar follow-ups as "M6 follow-up work"** rather than `deferred-to-phase-X`. Examples in the excerpts:
   - Schema-extraction row: "generated worker integration remains follow-up work."
   - Connection-state row: "Process-pipe fixtures and payload eligibility validation are tracked separately; compiler diagnostics and generated worker integration remain follow-up work."
   - Unix pipe-fixture row: "Windows fixtures and generated worker integration remain follow-up work."
   These contradict the user-stated framing that the public process-worker pool (which is what "generated worker integration" reduces to without a public API) is not part of M6.

## Stale wording worth cleaning up (non-blocking)

- The "Phase M6 contract excerpt" pasted in the prompt begins with `- `Lib/test/test_warnings/`` and a DoD list about "Warning/filter global-state parity" and "Generator-decorator helpers" — this is **M5 content leaking into the M6 excerpt**. Verify the real phase doc isn't carrying the same paste error before closing.
- The repeated qualifier "M6 implementation work" / "M6-owned follow-up work" should be normalized to `deferred-to-phase-X` everywhere generated worker integration appears, so the M6 ledger reads consistently after closeout.

## Minimal next slice (single docs-only PR)

1. Edit `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md`:
   - Replace Status line: `In progress` → closed/Met DoD with a one-line summary of the five DoD items and links to merge ledgers.
   - Reframe "Generated worker integration remains M6 implementation work" → "Generated worker integration and public worker-pool APIs are deferred-to-phase-X over this M6 substrate." Apply the same reframing wherever the phrase appears in the design doc, host matrix typed-IPC rows, and ledger summary lines.
2. Update `internal_docs/roadmap.md` and the M6 phase issue (`issues/`) to mark `milestone_concurrency_runtime_6` closed with links to the M6 ledger entries (#2437, #2439, #2441, #2443, #2445, #2447, #2450, #2452, #2454, #2455, #2458, #2460, #2462, #2464).
3. Verify the host matrix Windows row continues to read `host-limited` (acceptable) and is not implicitly upgraded by the closeout language.
4. Run `scripts/run_all_tests.sh --profile create-pr`; this is a docs-only change so `git diff --check` + file-size guardrail + create-pr lane should suffice.
5. Reviewer pass + merge ledger entry, then mark M6 closed.

No additional engineering slice is required to satisfy the M6 DoD as written.
