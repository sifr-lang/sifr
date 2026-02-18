# Web Stack

This phase makes Sifr a practical language for building production web applications. It adds a web framework, database access, typed web extractors, authentication, production features, external services, and data processing.

---

## milestone_web_framework: Web Framework

status: pending

**Goal:** Enable web applications with a Pythonic API over axum.

**Depends on:** milestone_async_core (async runtime), milestone_typed_serde_core (typed serialization for request/response)

### Work Items

Thin wrapper around `axum`:

- Routing, request/response, middleware
- Decorator-based routing (`@app.get("/")`, `@app.post("/")`)
- Graceful shutdown, health checks
- Static files, WebSockets

### Definition of Done (milestone_web_framework)

- `sifr.web` routes compile to axum handlers
- Decorator-based routing (`@app.get("/")`) works
- Graceful shutdown and health checks work
- Static file serving works
- E2E pass tests: web_hello, web_routing, web_middleware, graceful_shutdown, health_check
- Milestone demo in `./demos/milestone_web_framework_demo.sifr`

---

## milestone_database: Database Access

status: pending

**Goal:** Enable database-backed applications — both embedded SQLite for simple use cases and async PostgreSQL/MySQL for production.

**Depends on:** milestone_async_core (async runtime for sqlx connection pools)

### Work Items

- `sifr.db.sqlite` (wraps `rusqlite`) — embedded SQLite, synchronous API
- `sifr.db` (wraps `sqlx`) — async PostgreSQL/MySQL/SQLite with connection pools, typed queries, transactions, migrations

### Definition of Done (milestone_database)

- `sifr.db.sqlite` embedded SQLite works (open, execute, query, transactions)
- `sifr.db` connects to PostgreSQL/SQLite via sqlx
- Connection pooling works
- Typed query results work (leverages generics from Phase 13)
- Transactions and migrations work
- E2E pass tests: sqlite_basic, sqlite_transactions, db_query, db_pool, db_migrations
- Milestone demo in `./demos/milestone_database_demo.sifr`

---

## milestone_typed_web_extractors: Typed Web Extractors

status: pending

**Goal:** Provide typed request/response handling in `sifr.web`. This is the web-dependent half of typed serialization.

**Depends on:** milestone_web_framework (web framework must exist), milestone_typed_serde_core (auto-serde must exist)

### Work Items

- `Json[T]`, `Path[T]`, `Query[T]`, `Form[T]` extractors
- `UploadFile`, `Multipart` file uploads
- Validation errors -> 422

### Definition of Done (milestone_typed_web_extractors)

- `Json[T]`, `Path[T]`, `Query[T]`, `Form[T]` extractors work in web handlers
- `UploadFile` and `Multipart` handle file uploads
- Validation errors return 422 with structured error body
- E2E pass tests: typed_request_body, typed_path_param, typed_query_param, form_parsing, file_upload
- E2E fail tests: json_parse_wrong_type, missing_required_field, upload_exceeds_limit

---

## milestone_crypto_auth: Cryptography and Authentication

status: pending

**Goal:** Provide cryptographic primitives and authentication building blocks.

**Depends on:** milestone_typed_web_extractors (JWT payloads need typed serialization and web context)

### Work Items

- Password hashing (Argon2id), JWT, AES-256-GCM encryption, HMAC

### Definition of Done (milestone_crypto_auth)

- `hash_password()` / `verify_password()` with Argon2id
- JWT `encode()` / `decode()` with typed payloads
- AES-256-GCM `encrypt()` / `decrypt()`
- HMAC `hmac_sign()` / `hmac_verify()`
- E2E pass tests: password_hash_verify, jwt_roundtrip, aes_encrypt_decrypt, hmac_sign_verify
- E2E fail tests: wrong_password_rejected, expired_jwt_rejected, tampered_ciphertext_rejected

---

## milestone_web_production: Production Web Features

status: pending

**Goal:** Add production-grade web features.

**Depends on:** milestone_crypto_auth (rate limiting may use token-based identification)

### Work Items

- JSON structured logging, request tracing, rate limiting, CORS

### Definition of Done (milestone_web_production)

- `configure(format="json")` switches logging to JSON output
- `app.use_tracing()` logs every request with method, path, status, duration, request ID
- Rate limiting works (global and per-route)
- CORS configuration works
- E2E pass tests: json_logging, request_tracing, rate_limit, cors
- E2E fail tests: rate_limit_exceeded_429, cors_blocked_origin

---

## milestone_web_services: External Services

status: pending

**Goal:** Provide clients for common external services.

**Depends on:** milestone_web_production (production features should be in place)

### Modules

- `sifr.redis` — Redis client (async, connection pooling, pub/sub)
- `sifr.storage` — S3-compatible object storage (wraps `rust-s3`)
- `sifr.email` — SMTP email (wraps `lettre`)

### Definition of Done (milestone_web_services)

- Redis: connect, get/set, typed JSON, hash/list/set operations, pub/sub
- Storage: put/get/delete objects, presigned URLs, S3-compatible endpoints
- Email: send text/HTML email, attachments, multiple recipients
- All operations return `Result[T, E]`
- E2E tests for each service

---

## milestone_data_processing: Data Processing

status: pending

**Goal:** Enable data science and data engineering workflows.

**Depends on:** milestone_typed_serde_core (typed serialization for CSV/Parquet type mapping), milestone_async_core (async runtime for potential lazy evaluation)

### Work Items

- `sifr.data` (wraps `polars`) — DataFrame library with lazy evaluation, expressions, CSV/Parquet I/O

### Definition of Done (milestone_data_processing)

- `sifr.data.DataFrame` wraps polars with Pythonic API
- Lazy evaluation chain (filter, group_by, agg, sort) compiles correctly
- CSV/Parquet read/write works end-to-end
- E2E pass tests: dataframe_basic, csv_roundtrip
- Milestone demo in `./demos/milestone_data_processing_demo.sifr`

---

## Milestone Ordering

- **milestone_web_framework first:** The web framework is the core of the web stack. Depends on async runtime and typed serde.
- **milestone_database second:** Database access depends on the async runtime for connection pools. Independent of the web framework — a CLI tool can use `sifr.db` without `sifr.web`.
- **milestone_typed_web_extractors third:** Typed extractors depend on both the web framework and typed serde core.
- **milestone_crypto_auth fourth:** Authentication depends on typed serialization for JWT payloads.
- **milestone_web_production fifth:** Production features layer on top of the web framework and auth.
- **milestone_web_services sixth:** External services (Redis, S3, email) build on the full web stack.
- **milestone_data_processing independent:** Data processing depends only on typed serde and the async runtime — NOT on the web stack. It can proceed in parallel with web milestones or after them.
