Reviewing the new section. The diff shows three types of changes: Reuse Strategy line update, Correctness Test Suite additions, and the new "Downloader And Source Cache Architecture" section. Here are findings ordered by severity:

---

## Severity 1 — Gaps that will cause implementation disputes

**1. Cache key derivation is missing entirely.**

The doc describes *using* `CacheKey` in `DownloadRequest` but never defines how it's derived. Specifically:

- **Metadata cache keys** vs. **archive cache keys** are two distinct namespaces with different invalidation semantics. Index metadata revalidation uses ETag/Last-Modified; archive content-addressing uses the checksum. The doc only states archives are checksum-addressed, leaving metadata key derivation implicit.
- `Vary` header handling in the HTTP adapter section mentions `Vary` but doesn't say what field(s) Sifr includes in the cache key for Vary responses (e.g., `Accept-Encoding`, or any auth-scope header).

**Patch:** After the HTTP caching paragraph, add:
> Metadata cache keys are derived from the normalized registry URL plus the set of Vary header field names present in the response. Archive cache keys are derived from the content checksum. The cache key schema version is included in the key path so schema upgrades can be versioned without relying on uv's rkyv/rkyv-based record format.

---

**2. `--locked` fetch semantics are ambiguous.**

The doc says "`--locked` may fetch packages already selected by `sifr.lock`". The word "may" creates a trap: the implementation could interpret this as "must not fetch at all" (strict frozen semantics) or "may fetch to fill cache" (lenient). Cargo's `--locked` doesn't fetch; uv's `--frozen` doesn't fetch; uv's `--offline` doesn't fetch. The doc should pick one behavior.

**Patch:** Change to:
> `--locked` does not fetch new packages. It validates that all required source archives are already in the cache. If any are missing, it fails with `SIFR-PACKAGE-0203`. `--frozen` additionally forbids network metadata revalidation.

---

**3. Coalescing conflict: checksum mismatch after coalesced resolve.**

The doc says concurrent callers wait on the same in-flight result and "revalidate the returned checksum and manifest digest before using it." But what if two packages in the same solve resolve to the same source URL with *different* expected checksums? This is possible if the index is inconsistent or if a URL dependency has an unverified checksum at solve time. uv-installer handles this by validating compatibility after the fact. The doc needs a mismatch failure case.

**Patch:** In the coalescing paragraph, add:
> If two coalesced requests have different `expected_checksum` values and the returned checksum matches neither, the download fails with `SIFR-PACKAGE-0502` before cache population. A mismatch against only one request's expectation fails only that request.

---

**4. `DownloadState::CorruptPurged` conflates state with action.**

`CorruptPurged` names both a condition (corruption detected) and an action (purged). The enum should name observable states; the action belongs in `diag`. Compare the rest of the enum: `CacheHit`, `MetadataRevalidated`, `Downloaded`, `OfflineMissing` — all outcomes, no verbs.

**Patch:** Rename to `CorruptCacheEntry`. The purge action is already implied by "corrupt cache entries are purged only after verification failure is reported" in the milestone definition-of-done.

---

## Severity 2 — Inconsistencies that need resolution

**5. HTTP caching section doesn't map to a named module.**

The module map has `sources::registry::http_adapter` owning HTTP request construction, but no module owns the HTTP *caching* logic (304 handling, Cache-Control, ETag/Last-Modified state machine, stale reads). This is architecturally orphaned. Either `http_adapter` owns it, or a new `sources::cache::http_cache` submodule is needed.

**Patch:** In the HTTP caching paragraph, add a parenthetical:
> ...private-client HTTP caching... handled by `sources::registry::http_adapter` through its cache-read and cache-write callbacks.

---

**6. Missing `Vary` header policy for credentials and auth scope.**

The doc mentions `Vary` in the HTTP caching paragraph but never says whether auth-scope or bearer tokens are included in the Vary set. They shouldn't be (credentials never enter cache keys), but this needs explicit statement.

**Patch:** In the HTTP caching paragraph, add:
> Auth-scope headers and bearer tokens are never included in `Vary` or cache keys. Cache entries are registry-scope only.

---

**7. Retry policy is underspecified for backoff details.**

The doc correctly lists error codes and backoff classification but omits:
- Initial delay and cap (e.g., 50ms initial, 30s cap)
- Jitter strategy (none / full / decorrelated)
- Whether `429` responses with `Retry-After` headers bypass backoff in favor of the server-specified delay
- Whether `408` is always retryable or only when the connection was reused (to avoid thundering herd)

These will be invented during implementation. The doc should either specify them or explicitly say "implementation-defined within the bounded exponential backoff contract."

**Patch:** In the retry paragraph, add:
> Backoff uses full jitter with a 50ms initial delay and a 30s cap. `429` responses with `Retry-After` headers honor the server-specified value before applying backoff. `408` is retryable only on fresh connections, not on connection reuse.

---

**8. Progress rendering: "optional" conflicts with the completeness contract.**

The doc says "Progress rendering is optional" but the milestones say dry-run JSON is authoritative. If progress is optional, how does `sifr build` render progress for interactive users? The tension between "progress is optional" and "all CLI commands have deterministic output" needs resolution.

**Patch:** Change the progress paragraph to:
> Progress is a CLI rendering concern, not a plan/dry-run concern. The downloader emits structured progress events (`bytes_received`, `total_bytes`, `source_id`). The CLI renderer decides whether to display them interactively. Plan/dry-run output is authoritative and deterministic regardless of progress rendering.

---

## Severity 3 — Minor polish

**9. `DownloadCoordinator` lifecycle is unspecified.**

"Per-invocation" is mentioned but not defined. Is it created per command, per `sifr sync`, or per workspace lock? If `sifr build` reuses the same coordinator as `sifr sync`, the in-flight map persists across commands. If it doesn't, coalescing can't cross command boundaries.

**Patch:** Add to the concurrency section:
> The `DownloadCoordinator` is created per top-level command invocation (sync, fetch, build). It does not persist across separate CLI invocations. Within a single invocation, all download tasks share the same coordinator and in-flight map.

---

**10. `Vary` header in metadata caching needs a normative statement.**

`Vary: Accept-Encoding` is standard, but sparse index servers may add other Vary headers (e.g., language, custom versioning). The doc should state that Sifr requires `Vary: Accept-Encoding` or `Vary: *` and rejects responses with other Vary fields unless explicitly configured.

**Patch:** In the HTTP caching paragraph, add:
> Sifr requires `Vary: Accept-Encoding` or `Vary: *` on index responses. Responses with other Vary fields fail with a registry protocol diagnostic unless the registry is explicitly configured to handle them.

---

**11. uv-installer mention in Correctness Test Suite is orphaned.**

The diff adds `uv-installer::preparer` to the test port list, but the new architecture section doesn't mention what `preparer` maps to in Sifr's module map. The mapping is: `preparer` = `sources::cache` (archive extraction + validation + atomic visibility). This should be noted in `TRACEABILITY.md` but is acceptable to leave out of the phase doc since it's maintainer-level detail.

---

**12. Archive re-fetch if cache is corrupt/missing: already in cache invalidation section.**

The milestone definition-of-done says "corrupt cache entries are purged only after verification failure is reported, and offline/frozen modes never perform disallowed network or cache writes." This is already stated in the lockfile model section ("Corrupt package-cache entries are deleted only after checksum verification fails"). The new architecture section should reference the existing text rather than restating it.

**Patch:** In the atomic source-cache population section, change the last sentence to:
> ...corrupt entries remain unreachable from compiler/editor paths, and are cleaned up as described in [Artifact Cache And Generated Cargo Integration](#artifact-cache-and-generated-cargo-integration).

---

## Summary

The new section is structurally sound and the reuse boundary is correctly drawn. The most critical fixes are:

1. **Cache key derivation** (Severity 1) — without this, the metadata/archive distinction is unimplementable
2. **`--locked` fetch semantics** (Severity 1) — the "may" creates an ambiguity that will produce inconsistent implementations
3. **Coalescing checksum conflict** (Severity 1) — the "validate afterward" step needs explicit mismatch handling
4. **`CorruptPurged` naming** (Severity 1) — minor but the enum convention is violated
5. **HTTP caching module ownership** (Severity 2) — the cache-read/write logic needs a home in the module map

Items 5-8 are addressable with a few lines each. Items 9-12 are nice-to-have precision. Once items 1-4 are addressed, the section is implementation-ready.
