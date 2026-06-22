I verified the fix by tracing the logic through the actual fixture files.

**Verification of the fix in `check_fixture_matrix.py:383-406`:**

The `_rust_bound_function_name` builds `binding_prefix = f"@rust({crate_token}"` then calls `_has_crate_token_boundary(stripped, len(binding_prefix))`. The boundary helper rejects when the next character is alphanumeric or `_`, accepts everything else (and end-of-string).

Walked through the previously vulnerable crate pairs against their actual examples:

- crate `tracing` vs `@rust(tracing_subscriber.fmt.init, ...)`: `stripped[len("@rust(tracing")]` = `_` → boundary fail → correctly skipped. (`fixtures/ecosystem_cli_certification/examples/tracing-subscriber.sifr:9`)
- crate `tracing` vs `@rust(tracing.info, ...)`: next char = `.` → accepted. (`fixtures/ecosystem_cli_certification/examples/tracing.sifr:9`)
- crate `http` vs `@rust(http.Request.builder, ...)`: next char = `.` → accepted. `http-body` (token `http_body`) won't match the `http` prefix scan because `http_body` itself fails the boundary on `_`. (`fixtures/async_ecosystem_matrix/examples/http.sifr:12`)
- Same boundary protection now correctly partitions `serde` / `serde_derive` / `serde_json`, `tokio` / `tokio_postgres` / `tokio_tungstenite`, and `tower` / `tower_http`.

Delimiters appearing in real examples — `.`, `,`, `)`, whitespace — all pass `_has_crate_token_boundary`. Identifier continuations (`_`, alnum) all fail. End-of-string returns True so a bare `@rust(crate` line wouldn't be rejected on length alone (no such case in fixtures, but the behavior is consistent).

The runner output (`fixtures=31 diagnostics=10 crates=44 package_examples=51`) matches `REQUIRED_FIXTURES` (31) and `REQUIRED_CRATES` (44), confirming coverage is intact.

No remaining actionable findings.
