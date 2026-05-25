

All 12 round-1 findings are addressed. Let me verify the concrete patches:

1. **Cache key derivation** — Lines 852-853: metadata keys include schema version + registry + URL + path/query + content-negotiation + Vary fields; archive keys are checksum-derived. ✅
2. **`--locked` semantics** — Lines 527-528 and 856-857 are explicit: must not resolve beyond lock, may fetch+verify missing source archives already selected by lock, only allowed mutation is package-cache writes for those entries. ✅
3. **Coalescing checksum conflict** — Lines 847-848: coalesced requests share physical transfer only when source identity + auth scope match, but each caller validates independently; mismatch against one expected checksum fails that request with `SIFR-PACKAGE-0502`; response matching none is never cached. ✅
4. **`CorruptCacheEntry`** — Lines 824-829: enum is `CacheHit`, `MetadataRevalidated`, `Downloaded`, `CorruptCacheEntry`, `OfflineMissing` — all noun-states, no verbs. ✅
5. **HTTP caching module ownership** — Lines 850-851: `sources::registry::http_adapter` handles HTTP caching through cache-read/write callbacks into `sources::cache`. ✅
6. **Auth-scope/Vary exclusion** — Lines 852-853: bearer tokens, passwords, cookies, and raw authorization headers never enter cache keys. ✅
7. **Retry policy** — Lines 867-868: bounded exponential backoff with full jitter, `Retry-After` honored on `429`, final retry count in diagnostics, exact budgets from dependency audit. ✅
8. **Progress rendering** — Lines 869-870: structured downloader events (`source_id`, `bytes_received`, `total_bytes`); CLI rendering is interactive-only; plan/dry-run is authoritative and deterministic regardless of progress display. ✅
9. **`DownloadCoordinator` lifecycle** — Lines 843-844: per top-level command invocation (`sync`, `fetch`, `check`, `build`, `run`, `test`), does not persist across CLI processes, all download tasks share same in-flight map within one invocation. ✅
10. **`Vary` normative statement** — Lines 853-854: `Vary: Accept-Encoding` or no `Vary` are cacheable; `Vary: *`, `Vary: Authorization`, `Vary: Cookie`, or other unsupported fields make response non-cacheable and emit registry protocol diagnostic. ✅
11. **uv-installer mapping** — noted as maintainer-level detail; acceptable. ✅
12. **Archive re-fetch cross-reference** — Line 863: "cleaned up as described in [Artifact Cache And Generated Cargo Integration]" — consistent with the rest of the doc. ✅

**One remaining minor gap**, not Severity 1:

The mandatory property tests section (lines 159-160) says `--locked` may populate the package cache "for immutable entries already present in `sifr.lock`", but the architecture section (lines 856-857) more precisely says "source archives already selected by the lockfile". "Already present in" is slightly looser than "already selected by" — the architecture section has the right precision. The mandatory property tests are an authoritative contract that implementers will read directly; they should match the architecture section's phrasing.

**Patch:** In the mandatory property tests paragraph, change:
> It may populate the package cache only for immutable entries already present in `sifr.lock`
To:
> It may populate the package cache only for immutable source archives already selected by `sifr.lock`

That's the only patch needed. The downloader/cache architecture is **implementation-ready**. The uv reuse boundary is correctly drawn, the locked/frozen/offline/cache semantics are precise and internally consistent, and the state machine, coalescing, retry, atomic-population, and credential-redaction semantics are all implementable from the doc.
