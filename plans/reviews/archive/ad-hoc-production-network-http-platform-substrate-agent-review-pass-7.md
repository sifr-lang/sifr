# Review (agent Pass 7): Production Network and HTTP Platform Substrate — Final Reviewer Cleanup Verification

**Reviewer:** agent
**Scope (the six final-cleanup edits only, per request):**
- `issues/ad-hoc-production-network-http-platform-substrate.md` (working tree, full-doc consistency sweep)
- `issues/ad-hoc-production-network-http-platform-substrate-execution.md` (working tree)
- Repo verification: working-tree diff against `c3d716e86`; merge commit `f30e31f9e` existence; `reviews/...agent-review-pass-6.md` residual-risk list

**Verdict:** **PASS** — all six cleanup edits landed, each is internally consistent and propagated to every site that needed it, none contradicts the previously verified contract chain, and no implementation-readiness blocker is introduced or left open. The ledger entry records the cleanup accurately, remediation-for-remediation. The phase remains implementation-ready for M0.

---

## Edit-by-edit verification

### 1. `ByteBuffer` placeholder replaces public pseudo-type `bytes` — LANDED, FULLY PROPAGATED

All seven API-shape occurrences now read `Result[Option[ByteBuffer], ...]`: the Stream I/O ownership row (`substrate.md:413`), `TcpStream.read_chunk` (`:452`), `TcpReadHalf.read_chunk` (`:457`), the ownership-model bullet (`:470`), `TlsStream.read_chunk` (`:527`), and `TlsReadHalf.read_chunk` (`:533`). The placeholder is defined in three mutually consistent places: `:413` and `:470` mark it "pending the M0 byte-buffer decision," and the byte-buffer contract paragraph (`:496`) states `ByteBuffer` "is only a placeholder for the public type name until M0 records the final name, namespace, and import path." A matching M0 scope bullet was added (`:771`, "Define the placeholder-to-final mapping for public byte-buffer names used in this phase doc"), which composes with the pre-existing `:782` (define the type itself) rather than duplicating it.

Residue sweep: no `Option[bytes]`, `-> bytes`, or other lowercase pseudo-type signature remains anywhere in the doc. Every surviving `bytes` token is either `max_bytes` (a parameter name, fine), the Rust `bytes` crate (`:362`, internal-only, with its public-exposure sentence correctly reworded to "Sifr-owned byte-buffer values"), or prose about wire bytes. The `sifr.http` type table's `BodyChunk` row (`:618`, "public Sifr byte-buffer type selected by M0") is consistent with the placeholder semantics — it defers to the same M0 decision rather than naming a second type.

### 2. `hyper_util_necessity.md` proof artifact — LANDED, CONSISTENT IN BOTH SITES

The Ring 4 conditional row (`:376`) now ends: "If enabled, M4 must include `hyper_util_necessity.md` showing the Hyper-only attempt, custom adapter code avoided, selected features, and proof that no public Sifr lifecycle, shutdown, or type contract depends on Hyper-Util." The M4 definition of done (`:1018`) requires the same artifact with an identical four-part content list. The two sites agree verbatim in substance, and both are conditioned on "if enabled," preserving the crate's conditional/internal-only status everywhere else it is stated (`:415`, `:1005`, `:1133`). No site implies hyper-util is now accepted unconditionally; the Sifr-owned graceful-shutdown default and the feature blocklist in the row are untouched.

### 3. Ring 5 absence proof in generated release snapshots — LANDED, COHERENT

New M0 DoD bullet (`:818`): "Generated release dependency snapshots prove Ring 5 dev/test/demo crates, including `tokio-test`, `proptest`, `rcgen`, and `tracing-subscriber`, are absent from production feature combinations." The four named crates are exactly the four Ring 5 rows in the ecosystem table (`:380-383`), so the enumeration cannot drift from the ring definition. Placing the proof in M0 is coherent with the doc's existing position that M0 owns the candidate lockfile pins and records generated-project snapshot impact (`:344`), and it composes with — rather than conflicts with — the M2 TLS-specific snapshot gate (`:920`) and the M5 requirement to add snapshots "for all new feature combinations" (`:1048`), which provides the end-state re-proof once M1-M5 wire the real features.

*Observation (non-blocking):* at M0 the proof is necessarily against the M0-pinned dependency plan, since the feature combinations themselves ship in M1-M5. M0 should record which feature combinations its snapshot covered so the M5 snapshot can be diffed against it. This is an execution note for the M0 artifact, not a contract gap — the M5 DoD already closes the loop.

### 4. `TlsStream.close()` / `TlsWriteHalf.close()` M0 disposition — LANDED, NO CONTRADICTION

New paragraph at `:557`: M0 must define, before M2 starts, whether close consumes the handle and closes TCP directly, whether it first attempts `close_notify()`, how cancellation during close is reported, and how close failure preserves typed `TlsError`/nested `NetError` evidence. Cross-checked against the surrounding TLS contract:

- It deepens, not duplicates, the existing M0 TLS stream contract list, which already names `close` as an M0-defined/M2-implemented semantic (`:547`).
- It dovetails exactly with `:574`, which already required M0 to record whether `close_notify()` performs TCP write-side half-close "or only sends and flushes the TLS close alert while leaving TCP write-side closure to `close()`" — the new paragraph defines the other side of that same boundary. No circularity or conflict.
- M2 scope still implements "TLS `flush`, `close`, and `close_notify` behavior according to the M0 TLS stream contract" (`:888`), so the M0-defines/M2-implements ownership chain verified in passes 1-3 is intact.
- This partially retires the pass-6 residual risk: `TlsWriteHalf.close()` disposition is now an explicit M0 gate. `TlsReadHalf.close()` (`:534`) disposition remains covered by the generic `close` contract item (`:547`) and the split-half evidence rule (`:565`) under the final affine-handle rules — carried residual, same as the TCP read-half, not a blocker.

### 5. HTTP/2 priority/extension behavior as an M0 decision — LANDED, COMPLETE DECISION SPACE

New security/resource row (`:733`): "M0 must explicitly accept, ignore, reject, or defer HTTP/2 priority and extension-frame behavior before M4 starts. Unknown extension frames must map to typed protocol handling and must not panic or silently bypass resource limits." Verified:

- It complements the adjacent HTTP/2 abuse row (`:732`) without overlap: abuse covers SETTINGS/flow-control/PING/RST_STREAM/GOAWAY/malformed frames; the new row covers the priority and extension-frame dimension those lists omitted.
- "Accept, ignore, reject, or defer" is an exhaustive decision space, so M0 cannot satisfy the row with silence — this closes a genuine decision-by-discovery hole rather than adding prose.
- The no-panic and typed-protocol-handling requirements match the phase's `h2`-to-typed-`HttpError`/`ProtocolError` mapping (`:377`) and the repo-wide no-runtime-panic guarantee.
- M4's conformance DoD (`:1017`) lists M0-inventory behaviors with a non-exhaustive "including," so whatever M0 decides here flows into M4 loopback coverage through the existing conformance-inventory mechanism. No milestone-ordering conflict: the row's "before M4 starts" matches the table's house style.

### 6. UDP production-consumer burden with fixture-insufficiency rationale — LANDED, ALL FOUR NORMATIVE SITES

The strengthened gate — a named near-term production consumer **and** an explanation of why TCP/TLS/HTTP loopback fixtures are insufficient for that consumer — now appears in all four normative sites: the ecosystem UDP decision row (`:414`), the M1 scope bullet (`:837`), and resolved-decision rule 6 (`:1136`); the M1 DoD (`:870`) additionally requires "the recorded production-consumer rationale is checked in" before UDP loopback tests count. The four sites agree on both halves of the burden. The two summary mentions (`:18`, `:71`) state the consumer gate without the insufficiency clause — abbreviations of the normative rule, not contradictions, since neither is a decision site. The constrained surface list, the no-partial-public-API fallback (`deferred-to-phase-X` or `rejected` with rationale), and the broadcast/multicast deferrals are untouched.

## Contradiction and blocker sweep

- The working-tree diff against `c3d716e86` touches only the two issue files, and every hunk belongs to one of the six cleanup items or its direct propagation (M0 scope bullet `:771`, M1 DoD `:870`, M4 DoD `:1018`, M0 DoD `:818`). No unrelated drift.
- No edit changes a milestone owner, adds a classification label, weakens a prior gate, or reopens a decision verified in passes 1-6. All six tighten existing contracts in the M0-decides/M1-M4-implements direction the phase already uses.
- No new implementation-readiness blocker: each new requirement has a defined owner (M0 or M4), a defined deadline relative to the milestone graph, and a defined artifact or DoD check.

## Ledger accuracy

`execution.md:141-144` records the final reviewer cleanup pass with the correct source attribution (user-provided final review in agent), the correct result ("`PASS with small cleanup edits`" with the six cleanup areas named), and a remediation sentence whose six clauses map one-for-one onto the verified edits above — no overclaim, no omission. The matching checklist item (`:203`) closes the chain after the pass-6 entry. One presentational note, non-blocking: the cleanup-pass entry sits above the PR #2490 merge-ledger bullet (`:145-149`, merge commit `f30e31f9e`, verified present in history) even though these edits postdate that merge; the entry makes no claim of being included in that PR, but it has no PR link yet — add the docs PR link to the ledger when the cleanup PR opens, consistent with ledger practice.

## Residual risks (carried)

- `TlsReadHalf.close()` / `TcpReadHalf.close()` disposition while the sibling half is live remains an M0 day-one decision under the final affine-handle rules (narrowed by edit 4, which settled the stream and write-half side).
- The M0 Ring 5 snapshot proof is against the M0-pinned dependency plan; the M5 all-feature-combination snapshot DoD provides the end-state guarantee.
- Metrics schema can still slip to deferred at M5; `aws-lc-rs` build tooling remains an M2 evidence item. Both unchanged from passes 4-6.

---

## Bottom line

**PASS.** All six final-cleanup edits are present, consistent, and propagated: `ByteBuffer` is an explicitly-marked M0 placeholder with no lowercase pseudo-type residue; `hyper_util_necessity.md` is a conditional proof artifact required identically in the ring table and M4 DoD; Ring 5 absence is an M0 DoD snapshot proof naming exactly the four Ring 5 crates; TLS `close()` disposition is an explicit M0 gate that dovetails with the existing `close_notify()` boundary question; HTTP/2 priority/extension behavior is an exhaustive M0 decision with typed, panic-free handling; and UDP acceptance now carries the two-part consumer-plus-insufficiency burden in all four normative sites with a checked-in-rationale DoD gate. The execution ledger records the cleanup pass faithfully. No contradiction, no new blocker — the phase remains implementation-ready for M0.
