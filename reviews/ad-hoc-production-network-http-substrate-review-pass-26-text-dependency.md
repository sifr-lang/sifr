Audit complete across all four documents. Here is the verdict and findings.

---

## PASS

No blocking gaps found. The documents are consistent and implementation-ready. Specific audit answers:

---

### 1. Surface discovery and classification

All required surfaces are discovered and correctly placed:

| Surface | Classified in matrix? | Classification |
|---|---|---|
| URL parsing/building | Yes | `production-substrate` (ASCII/UTF-8); M1 for non-UTF-8 labels; M2 for IDNA |
| Percent encoding/decoding | Yes | `production-substrate` (byte/ASCII/UTF-8); M1 for named encodings/error handlers |
| Query/form helpers | Yes | `production-substrate` (byte/ASCII/UTF-8 safe); M1 for non-UTF-8 form decoding |
| HTTP headers | Yes | `production-substrate` (protocol token/ASCII); M1 for non-ASCII/user-text conversion |
| HTTP bodies | Yes | binary streaming only; M1 for charset decoding |
| Cookie header parsing | Yes | header-level syntax only; M1 for percent-decoded/user-text |
| TLS cert display | Yes | typed raw/DER or ASCII-safe only; M1 for decoded display names; M2 for Unicode normalization/IDNA |
| Diagnostics | Yes | static ASCII templates with typed evidence; M1 for decoded remote text; M3 for locale-sensitive formatting |
| Observability hooks | Yes | stable ASCII keys with typed fields; M1 for decoded previews; M3 for locale-sensitive formatting |
| Demos/fixtures | Yes | binary loopback and ASCII/UTF-8 only; M1 for `open(..., encoding=...)` |
| Phase 41 handoff | M0 DoD + execution ledger: must classify | States `deferred-to-phase-41` and `blocked-on-*` are defined for this |
| HTTP client handoff | M0 DoD + execution ledger: must classify | Same; `deferred-to-http-client-phase` state is defined |

Phase 41 and HTTP client handoff surfaces are not yet classified as explicit rows in the matrix, but the M0 definition of done explicitly requires it (`ad-hoc-production-network-http-platform-substrate.md:472–474`) and the execution ledger enforces it (`ad-hoc-production-network-http-platform-substrate-execution.md:128–129`). The classification states are available. This is correct design for a pre-M0 draft.

---

### 2. Milestone blockers

All assignments are correct:

- **M1 (encoding + explicit text I/O)**: correctly gates charset body decoding, non-ASCII header/cookie text, non-UTF-8 URL forms, named percent-encoding error handlers, decoded diagnostics/observability, and network demos requiring `open(..., encoding=...)`.
- **M2 (Unicode core)**: correctly gates IDNA host canonicalization, Unicode normalization-sensitive cert display, and URL host alignment check.
- **M2.5 (segmentation)**: correctly has zero network surface dependencies. Grapheme/word boundaries are not required for protocol correctness; network error messages use static ASCII templates; no surface needs segmentation before M3.
- **M3 (locale-sensitive formatting)**: correctly gates locale-sensitive network logging, formatted diagnostics, and observability label formatting. The network doc's "this phase must not introduce locale-derived defaults" is consistent with M3's prohibition on `setlocale` and implicit locale encoding.

No milestone is assigned to a surface whose provider is a different milestone.

---

### 3. Accidental local decoding, UTF-8 coercion, or locale-derived behavior

No surface is accidentally permitted to bypass the text/i18n provider. Key checks:

- **URL/IDNA through the `url` crate**: The `url` crate bundles `idna` (its own Unicode tables). The doc does not treat this as a silent exception. `ad-hoc-production-network-http-platform-substrate.md:594` requires M0 to explicitly prove the URL crate's Unicode version aligns with text/i18n M2, or keep Unicode host canonicalization `blocked-on-text-i18n-m2` and restrict to ASCII/already-punycode hosts. The Quality Contract (`line 753`) prohibits a "local Unicode data table" meaning hand-rolled implementation; an ecosystem crate accepted through the M0 dependency-decision process with a version-alignment check is not the same thing. The guard is present and correctly placed.
- **HTTP bodies**: "Bodies are `Bytes`/stream values in this phase. `body.text(...)`, charset-aware decoding, and text body previews are blocked on M1." No fallback decoding or UTF-8 coercion is permitted.
- **Content-Type charset**: "may parse and preserve charset labels, but must not decode payloads locally." No local codec dispatch.
- **Diagnostics/observability**: "static ASCII diagnostic templates with typed evidence" — no decoded body snippets, no locale-derived formatting before M3.
- **Cookies**: "Cookie names/values may remain typed header strings/bytes. Percent-decoded or non-UTF-8 cookie text is blocked on M1." No silent UTF-8 coercion.
- **TLS cert display**: "typed raw/DER or ASCII-safe fields only" before M1/M2. No invented string decoding.
- **Quality Contract** (`lines 753–758`): "No local encoding registry, local Unicode data table, locale-derived default encoding, fallback decoder, or duplicate text error-handler table may be introduced in this phase." The When-unblocked rule (`line 177`): "must call `sifr.encoding`, `sifr.unicode`, `sifr.io`, or `sifr.i18n` from the text/i18n phase." Both constraints are explicit and enforceable.

---

### 4. Binary/protocol substrate remains implementation-ready

The matrix's first row — TCP, UDP, DNS socket transport, TLS handshakes, HTTP/1.1 and HTTP/2 framing — is classified `production-substrate` with no text/i18n dependency. All five implementation milestones (M1–M4) can begin on the binary transport stack without waiting on any text/i18n milestone. The text/i18n gates only apply to optional consumer surfaces layered on top of that stack.

---

### 5. Execution ledger consistency

The execution ledger matches the main doc on every material point:

- Remediation checklist item `[x]` at `execution.md:76` confirms the text/i18n dependency matrix was added.
- Remediation checklist item `[x]` at `execution.md:77` confirms the no-local-encoding/Unicode/locale rule was enforced.
- The CPython Evidence Scan section (`execution.md:128–129`) requires recording text/i18n dependency states for all 12 surfaces including Phase 41 and HTTP client handoff, consistent with the main doc's M0 scope.
- All six implementation milestone slots are `pending`, which is correct: M0 has not yet produced the dependency records and inventory artifacts that unlock M1–M5.

One ledger status observation (non-blocking): the text/i18n discovery entry at `execution.md:51–53` reads "Result: in progress" with past-tense description of what was classified. Since this review pass validates the matrix as complete, the ledger should record the review result (this pass) and close the "in progress" status before M0 starts.

---

### Non-blocking editorial observation

The text/i18n M1 scope (`ad-hoc-production-text-i18n-stdlib-parity.md`, milestone_text_i18n_1) lists three network surfaces as the "exact cross-phase unblock point" examples (non-UTF-8 URL quoting/parsing, HTTP body text decoding, network demos). The network doc's matrix has nine distinct M1-blocked surfaces. The phrase "exact" qualifies the milestone identity (M1 is the precise unblock milestone, not M0 or M2), not the exhaustiveness of the example list. Implementation is not endangered because the network doc's cross-phase contract explicitly says all `blocked-on-text-i18n-m1` features unblock on M1 closure, and the M1 DoD tests encoding substrate capabilities rather than consumer surfaces. No remediation is required before implementation starts.
