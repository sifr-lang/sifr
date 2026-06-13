# Ad Hoc Phase: Production Stdlib Platform Contract

Status: approved shared baseline; platform-contract review passes 3a-3d recorded `PASS`, provider phases have closed against this contract, and network/HTTP M0 must verify inventories against it.
Phase placement: shared contract created before `milestone_text_i18n_0` closes and required before any split production-stdlib M1 implementation starts.
Phase owner: text/i18n, concurrency/runtime, and network/HTTP phase owners jointly

## Objective

Define the cross-phase contract that makes the text/i18n, concurrency/runtime, and network/HTTP phases one coherent Sifr platform rather than three independent inventories.

This contract is not a compatibility layer. It standardizes native Sifr substrate rules for ownership, cancellation, backpressure, typed error nesting, observability, host support, security/resource limits, stability states, and executable cross-phase golden fixtures.

## Ordering

The shared contract artifacts are created before `milestone_text_i18n_0` closes:

- `verification/platform/platform_contract.md`
- `verification/platform/platform_contract.json`
- `verification/platform/supported_host_matrix.md`
- `verification/platform/golden/manifest.json`
- `verification/platform/golden/*.sifr`
- `scripts/run_platform_golden.sh`

The shared contract must receive an external review `PASS` before text/i18n M1 opens. Concurrency/runtime M0 and network/HTTP M0 then verify their inventories against the approved contract instead of creating competing local contracts.

## Native Substrate First

All three phases build Sifr-native production substrate first. CPython sources and tests are evidence, fixture material, and waiver input. CPython-shaped modules become `compat-adapter` work only after the native substrate exists and only when the adapter remains a good long-term API.

Partial public modules, fallback paths, hidden compatibility aliases, legacy behavior, dynamic global registries, and toy demo surfaces are rejected unless explicitly classified as `test-only-harness` or `internal-only`.

## State Vocabulary

Every phase inventory must use these terminal states for public, semi-public, and internal surfaces:

| State | Meaning |
| --- | --- |
| `production-public` | Stable recommended Sifr API for user code. |
| `production-substrate` | Stable substrate consumed by public APIs or later phases; may be lower-level than normal user APIs. |
| `compat-adapter` | Thin adapter over production substrate with proven migration value; never the primary product shape. |
| `internal-only` | Implementation detail hidden from stable imports. |
| `test-only-harness` | Fixture or harness code unavailable as user API. |
| `deferred-to-phase-X` | Explicitly deferred to a named phase or issue. |
| `blocked-on-phase-X` | Blocked until a named provider milestone closes. |
| `host-limited` | Supported only on named hosts with documented behavior and tests. |
| `unsupported-with-diagnostic` | Unsupported and must produce a stable diagnostic when referenced. |
| `rejected` | Intentionally out of product scope with rationale and revisit rule if any. |

Every CPython evidence/test family must use these states:

| State | Meaning |
| --- | --- |
| `mined-as-substrate-fixture` | Behavior becomes a Sifr-native fixture. |
| `adapted-for-sifr-api` | Behavior is adapted to a Sifr-native API shape. |
| `compat-adapter-deferred` | Relevant only if a future compatibility adapter is approved. |
| `blocked-on-phase-X` | Needs a named provider milestone. |
| `external-signal` | Useful evidence but not local validation because it depends on external state. |
| `waived-with-rationale` | Explicitly waived with rationale and reviewer sign-off. |
| `rejected` | Legacy, unsafe, dynamic, deprecated, or non-product behavior. |

Every stable or semi-stable surface must also declare one stability level:

- `stable-public-api`
- `stable-production-substrate`
- `unstable-internal-substrate`
- `compiler-known-intrinsic`
- `compatibility-adapter`
- `test-only-harness`

## Ownership And Lifetime

| Resource family | Contract |
| --- | --- |
| Bytes, HTTP bodies, subprocess pipes, and IPC payloads | Owned by default. Borrowing across async/task/process boundaries requires compiler proof. |
| TCP streams, TLS streams, HTTP body streams, and subprocess pipes | Linear resources with explicit close/shutdown/drop behavior; double-close and use-after-close are diagnostics or typed errors. |
| Incremental codecs | Unique mutable state; no concurrent aliasing or hidden shared mutation. |
| Executor futures and task handles | Must be observed, joined, cancelled, or explicitly consumed; unobserved failure is diagnosed. |
| Text values | Always valid Unicode scalar text; arbitrary bytes never become text without explicit decoding. |

## Cancellation And Timeout

Cancellation is typed evidence, not an invisible drop path.

| Surface | Required behavior |
| --- | --- |
| Task cancellation | Uses the canonical task cancellation/deadline model and reports typed cancellation. |
| Queue/channel cancellation | Cancelled send/receive preserves channel invariants and reports whether a value moved. |
| Subprocess timeout | Converts to typed timeout and then follows the configured terminate/kill policy. |
| HTTP/2 stream cancellation | Sends or observes `RST_STREAM` where protocol state requires it and reports typed cancellation. |
| TLS shutdown cancellation | Preserves memory safety, reports partial shutdown evidence, and never downgrades verification. |
| Text decode cancellation | Streaming decoders preserve linear state and report exhausted/partial-state errors explicitly. |

## Backpressure

Backpressure semantics are shared across queues, async streams, subprocess pipes, HTTP bodies, and IPC:

- bounded buffers have explicit capacity and typed full/closed outcomes
- unbounded buffers require a product justification and resource-limit entry
- producers cannot hide unbounded buffering inside adapters
- cancellation of a producer or consumer records whether data was accepted, dropped, or still pending
- HTTP body streaming and subprocess pipes must not buffer whole payloads unless an API name explicitly says it collects

## Typed Error Nesting

Higher-level errors must preserve lower-level evidence:

```text
HttpError::Tls(TlsError::Transport(NetError::Dns(DnsError)))
ProcessError::Pipe(PipeError::TextDecode(DecodeError))
TaskError::Worker(WorkerError::PanicBoundary(RuntimeError))
```

Every phase error map must define nesting conversions, redaction rules, and equality/classification behavior for tests. Exception-only control flow is rejected.

## Observability

Structured events across all three phases use shared field names:

- `sifr.phase`
- `sifr.operation`
- `sifr.correlation_id`
- `sifr.task_id`
- `sifr.resource_id`
- `sifr.host`
- `sifr.error.kind`
- `sifr.error.cause`
- `sifr.blocked_on`
- `sifr.duration_ms`

Redaction is required for URLs with credentials, headers, cookies, request/response bodies, certificate fields, subprocess command lines, environment variables, IPC payloads, locale data paths, and translation catalog contents. No phase may introduce a separate diagnostics bus or incompatible correlation model.

## Supported Host Matrix

The shared host matrix lives at:

- `verification/platform/supported_host_matrix.md`

It supersedes per-phase host matrices. Phase-specific inventories link to rows in the shared matrix for DNS resolver behavior, socket options, TLS roots, locale names, signals, subprocess spawning, multiprocessing/process-worker strategy, path encoding, file descriptor inheritance, and process termination behavior.

## Security And Resource Ownership

| Concern | Owning phase |
| --- | --- |
| Buffer/body size limits and parser DoS input | network/HTTP |
| Connection, request, and concurrent stream limits | network/HTTP plus concurrency/runtime |
| TLS certificate verification, roots, mTLS, and pinning policy | network/HTTP |
| Request smuggling, header normalization, and HTTP/2 flow-control abuse | network/HTTP |
| Redirect, proxy, CONNECT, auth, cookie, and compression bomb policy | future HTTP client / Phase 41, with substrate hooks from network/HTTP |
| Codec amplification and malformed byte sequences | text/i18n |
| Malicious `.mo` catalog, plural expression, and translation payload handling | text/i18n |
| Locale data discovery and host-limited formatting | text/i18n |
| Subprocess resource limits and `@shell_exec` security surface | concurrency/runtime |
| IPC payload size and panic-free malformed-message handling | concurrency/runtime |
| Cancellation storms, task explosion, queue bounds, and offload capacity | concurrency/runtime |
| Log redaction for URLs, headers, bodies, cookies, certificates, subprocess commands, env, and catalogs | shared contract with owning phase-specific fields |

## Cross-Phase Golden Fixtures

Golden programs are executable acceptance fixtures, not documentation demos. The manifest lives at:

- `verification/platform/golden/manifest.json`

Each entry must include:

```json
{
  "program": "async_http_binary_body.sifr",
  "command": "cargo run -q -p sifr -- run verification/platform/golden/async_http_binary_body.sifr",
  "expected_exit": 0,
  "expected_stdout_contains": ["platform-golden: pass"],
  "expected_diagnostic_contains": [],
  "depends_on": ["milestone_network_http_1"],
  "must_not_depend_on": ["milestone_text_i18n_1"],
  "blocked_until": [],
  "checks": ["binary body stream", "no charset decoding"]
}
```

`scripts/run_platform_golden.sh` runs unblocked entries and skips entries whose `blocked_until` milestones are not closed. `scripts/run_all_tests.sh` must call it once the manifest exists. Non-blocked fixtures must pass at each phase final milestone.

Seed fixtures:

| Program | Expected | Contract proven |
| --- | --- | --- |
| `async_tcp_binary_echo.sifr` | pass | TCP binary stream, timeout, cancellation, no text dependency. |
| `tls_loopback_cert_failure.sifr` | pass | safe TLS verification and typed certificate failure. |
| `http2_binary_stream_cancel.sifr` | pass | HTTP/2 streaming, bounded memory, `RST_STREAM` cancellation evidence. |
| `phase41_binary_request_handoff.sifr` | pass | framework receives typed headers/protocol metadata and binary body stream. |
| `http_body_text_before_text_i18n.sifr` | diagnostic | no local UTF-8 fallback before text/i18n M1. |
| `http_body_text_after_text_i18n.sifr` | pass after M1 | explicit charset decoding through text/i18n substrate. |
| `subprocess_binary_pipeline_timeout.sifr` | pass | binary pipes, timeout, termination policy, typed errors. |
| `subprocess_text_explicit_encoding.sifr` | pass after M1 | subprocess text mode consumes explicit encoding substrate. |
| `executor_result_error_observation.sifr` | pass | executor/offload returns `Result[T, E]` and observed typed failures. |
| `locale_host_limited_formatting.sifr` | pass/host-limited | explicit locale formatting and host matrix behavior. |
| `unsupported_cpython_imports.sifr` | diagnostic | bare CPython-shaped imports produce stable replacement diagnostics. |

## M0 Acceptance

Each split phase M0 must:

1. Reference this contract in its inventory.
2. Use the shared terminal states and stability levels.
3. Add or update relevant rows in the shared host matrix.
4. Add or update relevant golden fixture manifest entries.
5. Map security/resource concerns to this ownership table.
6. Prove no local substitute exists for a provider-owned substrate.
