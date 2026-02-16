# Ecosystem

This phase makes Sifr a practical language for building real-world applications. It adds async/await, a web framework, database access, typed serialization, authentication, production web features, Redis, object storage, email, and data processing — everything needed to build and deploy production web services and data pipelines.

---

## milestone_async: Async Runtime

status: pending

**Goal:** Add async/await language support. This is a language feature milestone -- it adds the async primitives that milestone_web_db (web, database) builds on.

### Language Features

- `**async def` / `await`:** maps to Rust `async fn` / `.await`
- **Async runtime:** built on `tokio` (bundled automatically when async is used)
- `**sifr.net`:** TCP/UDP sockets (async) -> wraps `tokio::net`
- `**sifr.task`:** task spawning, sleep, timeouts -> wraps `tokio::task` + `tokio::time`
- **Async iterators:** `async for` over async streams. Builds on the lazy `Iterator` state machine codegen delivered in Phase 7 (Stdlib Parity, milestone_lazy_iterators) — the async variant wraps the same state machine pattern with `async fn next()`.
- `**async with`:** async context managers for resources that require async setup/teardown (e.g., database connections, HTTP sessions). Builds on the sync `with` statement and `ContextManager` protocol (`__enter__`/`__exit__`) delivered in Phase 7 (Stdlib Parity, milestone_compiler_hardening). Codegen: the `__aenter__` and `__aexit__` methods are `async fn`, and the `with` block `.await`s them. Maps to Rust's async scope pattern with `Drop` + async cleanup.
- **Async generators:** `yield` inside `async def` produces an async iterator. Codegen: combines the lazy state machine from milestone_lazy_iterators (Phase 7) with async/await from this milestone.

### Example

```python
from sifr.task import sleep
from sifr.net import TcpListener

async def handle_connection(stream: TcpStream):
    data: str = await stream.read()
    await stream.write(f"Echo: {data}")

async def main():
    listener = await TcpListener.bind("0.0.0.0:8080")
    while True:
        stream = await listener.accept()
        await handle_connection(stream)
```

### Async Error Propagation

The `?` operator works across `.await` points. Async functions returning `Result` propagate errors the same way as sync functions. Closures captured across `.await` points must be `Send + 'static` (the compiler enforces this and emits clear diagnostics if violated).

### Concurrency Primitives

milestone_async also provides basic cross-task communication primitives:

- `**sifr.sync.Lock`:** async mutex for shared mutable state. Codegen: `tokio::sync::Mutex<T>`.
- `**sifr.sync.Channel`:** async channel for message passing. Codegen: `tokio::sync::mpsc::channel`.
- `**sifr.sync.Semaphore`:** async semaphore for rate limiting. Codegen: `tokio::sync::Semaphore`.

### Definition of Done (milestone_async)

- `async def` compiles to Rust `async fn`
- `await` compiles to `.await`
- Tokio runtime is automatically bundled when async is used
- `?` operator works across `.await` points
- Async closures captured across `.await` are checked for `Send + 'static`
- `sifr.task.spawn` works for concurrent tasks
- `async with` works for async context managers
- Async generators (`yield` in `async def`) produce async iterators (builds on lazy iterator state machine from Phase 7)
- `sifr.sync.Lock`, `sifr.sync.Channel`, `sifr.sync.Semaphore` work for cross-task coordination
- E2E pass tests: async_basic, await_chain, task_spawn, async_error_propagation, async_with_basic, async_generator_basic, lock_basic, channel_basic
- Milestone demo in `./demos/milestone_async_demo.sifr`

---

## milestone_networking_stdlib: Networking Standard Library

status: pending

**Goal:** Add networking-related stdlib modules that depend on the async runtime from milestone_async. These modules bridge the gap between the synchronous stdlib (from the Stdlib Architecture and Stdlib Parity phases) and the web framework (milestone_web_db).

**Full plan:** [.cursor/plans/hybrid_stdlib_architecture_67d3c0a1.md](.cursor/plans/hybrid_stdlib_architecture_67d3c0a1.md) (see "Modules to Defer to Ecosystem Phase")

### Modules

- `sifr/subprocess.sifr` -- full Popen API (wraps new `_sifr.process` intrinsics)
- `sifr/socket.sifr` -- TCP/UDP (wraps new `_sifr.net` intrinsics)
- `sifr/http.sifr` -- HTTP client (wraps `_sifr.net` + potentially `reqwest` crate)
- `sifr/url.sifr` -- URL parsing (pure Sifr or wraps `url` crate)

### Definition of Done (milestone_networking_stdlib)

- Each networking module compiles and works with async I/O
- All fallible operations return `Result` or `Option`
- E2E pass tests: subprocess_run, socket_tcp, http_get, url_parse
- Integration with the async runtime (tokio) is seamless

---

## milestone_web_db: Web and Database

status: pending

**Goal:** Enable production web applications and database-backed services. This is the milestone that makes sifr useful for the most common Python use case: web APIs.

### Web Framework (`sifr.web`)

Thin wrapper around `axum` -- the most popular async Rust web framework:

- **Routing:** decorator-based route registration
- **Request/Response:** typed request parsing, JSON responses
- **Middleware:** logging, CORS, auth hooks
- **Static files:** serve static assets
- **WebSockets:** real-time communication
- **Graceful shutdown:** `app.run()` automatically handles SIGINT/SIGTERM, drains in-flight requests, and exits cleanly. No user code needed -- it is the default behavior. Codegen: `axum::serve(...).with_graceful_shutdown(shutdown_signal())` using `tokio::signal`.
- **Shutdown hooks:** `app.on_shutdown(cleanup_fn)` registers async cleanup functions (close DB pools, flush logs). Codegen: runs registered functions after the server stops accepting connections.
- **Health check:** `app.health_check("/health")` registers a health endpoint returning 200 OK. Standard for container orchestration (Kubernetes, ECS).

```python
from sifr.web import App, Request, Response, Router

app = App()

@app.get("/")
async def index(req: Request) -> Response:
    return Response.text("Hello, World!")

@app.get("/users/{id}")
async def get_user(req: Request) -> Response:
    user_id: str = req.params["id"]
    return Response.json({"id": user_id, "name": "Alice"})

@app.post("/users")
async def create_user(req: Request) -> Response:
    body: dict[str, str] = await req.json()
    return Response.json(body, status=201)

def main():
    app.run(host="0.0.0.0", port=8000)
```

### HTTP Client (`sifr.http`)

Thin wrapper around `reqwest`:

```python
from sifr.http import get, post

async def fetch_data() -> dict[str, str]:
    response = await get("https://api.example.com/data")
    return await response.json()
```

### Database (`sifr.db`)

Two tiers of database support:

**Embedded SQLite (`sifr.db.sqlite`)** -- zero-config, no external server needed. Wraps `rusqlite`:

- **Synchronous API:** simple and fast for prototyping, CLI tools, and small apps
- **In-memory or file-backed:** `Database.open(":memory:")` or `Database.open("app.db")`
- **Prepared statements, transactions, typed parameters**

```python
from sifr.db.sqlite import Database

db = Database.open("app.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
db.execute("INSERT INTO users (name) VALUES (?)", "Alice")

for row in db.query("SELECT * FROM users"):
    print(f"{row.id}: {row.name}")
```

**Async databases (`sifr.db`)** -- production-grade, wraps `sqlx` (async, compile-time checked SQL):

- **Connection pools:** PostgreSQL, MySQL, SQLite
- **Typed queries:** compile-time SQL validation
- **Transactions:** context-manager style
- **Migrations:** schema management

```python
from sifr.db import Database, query

db = Database.connect("postgres://localhost/myapp")

async def get_user(id: int) -> dict[str, str] | None:
    row = await db.query_one("SELECT name, email FROM users WHERE id = $1", id)
    if row is not None:
        return {"name": row.name, "email": row.email}
    return None
```

### Rust Crate Mapping

- `sifr.web` -> `axum` + `tower` (middleware) + `tower-http` (static files, body limits)
- `sifr.web` graceful shutdown -> `tokio::signal` (SIGINT/SIGTERM handling)
- `sifr.http` -> `reqwest`
- `sifr.db.sqlite` -> `rusqlite` (synchronous, embedded)
- `sifr.db` -> `sqlx` (async, compile-time checked)
- Generated Cargo.toml includes these as dependencies automatically

### SQLx Build-time Contract

SQLx's compile-time SQL checking requires database metadata at build time. Sifr supports two modes:

- **Online mode (development):** the compiler connects to a running database during compilation to validate SQL queries. Connection string is read from `DATABASE_URL` in `.env` or `sifr.toml`.
- **Offline mode (CI/production):** SQL metadata is cached in a `sqlx-data.json` file (generated by `sifr db prepare`). The compiler reads this file instead of connecting to a database. This file is committed to version control for reproducible CI builds.

The compiler emits a clear error if neither a database connection nor offline metadata is available, with instructions on how to set up either mode.

### Definition of Done (milestone_web_db)

- `sifr.web` routes compile to axum handlers
- Decorator-based routing (`@app.get("/")`) works
- Request/Response types are correctly typed
- `sifr.http` GET/POST requests work end-to-end
- `sifr.db.sqlite` embedded SQLite works (open, execute, query, transactions)
- `sifr.db` connects to PostgreSQL/SQLite via sqlx
- SQL queries are validated at compile time (online or offline mode)
- `sifr db prepare` generates offline metadata
- Graceful shutdown: `app.run()` handles SIGINT/SIGTERM, drains in-flight requests
- Shutdown hooks: `app.on_shutdown(fn)` registers cleanup functions that run on shutdown
- Health check: `app.health_check("/health")` registers a 200 OK health endpoint
- E2E pass tests: web_hello, http_get, sqlite_basic, db_query, graceful_shutdown, health_check
- Milestone demo in `./demos/milestone_web_db_demo.sifr` (simple REST API with embedded SQLite)

---

## milestone_typed_serde: Typed Serialization and Request Validation

status: pending

**Goal:** Leverage Sifr's type system to automatically serialize/deserialize classes to/from JSON, and provide typed request/response handling in `sifr.web`. This is Sifr's biggest differentiator -- what Pydantic, Zod, and serde derive do manually, Sifr does automatically because the compiler knows the types.

**Depends on:** milestone_web_db (web framework must exist), milestone_classes (classes must exist), milestone_generics (generic type parameters needed for `Json[T]`)

### Typed JSON Serialization (`sifr.json` enhancement)

Enhance `sifr.json` (from milestone_core_stdlib) so that classes can be serialized/deserialized automatically:

```python
class User:
    name: str
    email: str
    age: int

from sifr.json import dumps, loads

user = User("Alice", "alice@example.com", 30)
json_str: str = dumps(user)                          # '{"name":"Alice","email":"alice@example.com","age":30}'
parsed: User = loads(json_str, User)                  # typed deserialization
```

**Codegen:** The compiler auto-derives `serde::Serialize` and `serde::Deserialize` on all classes. `dumps(obj)` emits `serde_json::to_string(&obj)?`. `loads(s, T)` emits `serde_json::from_str::<T>(s)?`. Returns `Result[T, JsonError]` on parse failure.

**Supported types:** All primitive types, `list[T]`, `dict[K, V]`, `tuple[...]`, `Option[T]` (serialized as `null`), nested classes, union types (serialized as tagged enum). `bytes` fields serialize as base64.

### Typed Web Request/Response (`sifr.web` enhancement)

Replace the untyped `Request` / `Response` pattern with typed extractors:

```python
from sifr.web import App, Json, Path, Query

app = App()

class CreateUserRequest:
    name: str
    email: str
    age: int

class UserResponse:
    id: int
    name: str
    email: str

@app.post("/users")
async def create_user(body: Json[CreateUserRequest]) -> Json[UserResponse]:
    # body.data is already parsed and validated as CreateUserRequest
    user = save_user(body.data)
    return Json(UserResponse(id=user.id, name=user.name, email=user.email))

@app.get("/users/{user_id}")
async def get_user(user_id: Path[int]) -> Json[UserResponse]:
    # user_id.data is already parsed as int from the path
    ...

class SearchParams:
    q: str
    page: int = 1
    limit: int = 20

@app.get("/search")
async def search(params: Query[SearchParams]) -> Json[list[UserResponse]]:
    # params.data is parsed from query string with defaults applied
    ...
```

**Codegen:** `Json[T]` maps to axum's `axum::Json<T>` extractor. `Path[T]` maps to `axum::extract::Path<T>`. `Query[T]` maps to `axum::extract::Query<T>`. The compiler verifies that `T` has serde derives. Validation errors automatically return 422 with a structured error body.

### Form Data and Multipart File Uploads

```python
from sifr.web import App, Form, Multipart, UploadFile

class LoginForm:
    username: str
    password: str

@app.post("/login")
async def login(form: Form[LoginForm]) -> Json[dict[str, str]]:
    # form.data is parsed from application/x-www-form-urlencoded
    ...

@app.post("/upload")
async def upload(file: UploadFile) -> Json[dict[str, str]]:
    data: bytes = await file.read()
    filename: str = file.filename
    content_type: str = file.content_type
    await file.save("/uploads/" + filename)
    return Json({"filename": filename, "size": str(len(data))})

@app.post("/upload-multiple")
async def upload_many(files: Multipart) -> Json[list[str]]:
    names: list[str] = []
    async for field in files:
        if field.filename is not None:
            await field.save("/uploads/" + field.filename)
            names.append(field.filename)
    return Json(names)
```

**Codegen:** `Form[T]` maps to `axum::Form<T>`. `UploadFile` and `Multipart` map to `axum::extract::Multipart` (requires the `multipart` feature). File size limits are configurable via `app.config(max_upload_size=50 * 1024 * 1024)` which maps to `tower_http::limit::RequestBodyLimitLayer`.

### Rust Crate Mapping

- Typed serde: `serde` (290M+ downloads/month) + `serde_json` (190M+ downloads/month) -- the most downloaded crates in the Rust ecosystem
- Form/multipart: `axum` with `multipart` feature (already a dependency)
- Body limits: `tower-http` (already a dependency from milestone_web_db)

### Definition of Done (milestone_typed_serde)

- Classes auto-derive `Serialize`/`Deserialize` -- no manual annotation needed
- `dumps(obj)` serializes any class to JSON string
- `loads(s, T)` deserializes JSON string to typed class, returns `Result[T, JsonError]`
- Nested classes, lists, dicts, optionals, unions serialize correctly
- `Json[T]`, `Path[T]`, `Query[T]`, `Form[T]` extractors work in web handlers
- `UploadFile` and `Multipart` handle file uploads
- Validation errors return 422 with structured error body
- File upload size limits configurable
- E2E pass tests: typed_json_roundtrip, typed_request_body, typed_path_param, typed_query_param, form_parsing, file_upload, multipart_upload
- E2E fail tests: json_parse_wrong_type, missing_required_field, upload_exceeds_limit
- Milestone demo in `./demos/milestone_typed_serde_demo.sifr`

---

## milestone_crypto_auth: Cryptography and Authentication

status: pending

**Goal:** Provide the cryptographic primitives and authentication building blocks that every web application needs. Password hashing with secure defaults, JWT tokens, encryption, and HMAC. The principle: make the secure choice the easy choice.

**Depends on:** milestone_typed_serde (JWT payloads need typed serialization), milestone_ext_stdlib (`sifr.hashlib` provides data hashing; this milestone adds password hashing and encryption which are distinct)

### Password Hashing (`sifr.crypto.password`)

```python
from sifr.crypto.password import hash_password, verify_password

# Hash with secure defaults (Argon2id, auto-generated salt)
hashed: str = hash_password("my_secret_password")

# Verify -- constant-time comparison, auto-detects algorithm
is_valid: bool = verify_password("my_secret_password", hashed)

# Explicit algorithm choice (for legacy systems)
from sifr.crypto.password import Algorithm
hashed_bcrypt: str = hash_password("password", algorithm=Algorithm.Bcrypt)
```

**Codegen:** `hash_password()` maps to `argon2::Argon2::default().hash_password()` with `SaltString::generate()`. `verify_password()` maps to `argon2::PasswordHash::new(hash).verify_password()`. Both run on a blocking thread pool (`tokio::task::spawn_blocking`) to avoid blocking the async runtime.

**Design:** Returns `str` (PHC string format) not raw bytes -- the hash string includes algorithm, salt, and parameters so it is self-describing. `verify_password` auto-detects whether the hash is Argon2 or Bcrypt from the prefix.

### JWT Tokens (`sifr.crypto.jwt`)

```python
from sifr.crypto.jwt import encode, decode, JwtError

class TokenPayload:
    user_id: int
    role: str
    exp: int  # expiration timestamp

secret: str = sifr.env.get("JWT_SECRET")

# Create token
payload = TokenPayload(user_id=42, role="admin", exp=1700000000)
token: str = encode(payload, secret)

# Decode and validate (checks expiration automatically)
result: Result[TokenPayload, JwtError] = decode(token, secret, TokenPayload)
match result:
    case Ok(data):
        print(f"User {data.user_id} with role {data.role}")
    case Err(e):
        print(f"Invalid token: {e}")
```

**Codegen:** `encode()` maps to `jsonwebtoken::encode()` with `Header::default()` (HS256). `decode()` maps to `jsonwebtoken::decode::<T>()` with `Validation::default()`. The payload class must have serde derives (auto-derived from milestone_typed_serde). Supports HS256, HS384, HS512, RS256, ES256 via `Algorithm` enum.

### Encryption (`sifr.crypto`)

```python
from sifr.crypto import encrypt, decrypt, generate_key

# Symmetric encryption (AES-256-GCM -- authenticated encryption)
key: bytes = generate_key()                              # 32 random bytes
ciphertext: bytes = encrypt(b"secret data", key)         # includes nonce + tag
plaintext: Result[bytes, CryptoError] = decrypt(ciphertext, key)

# HMAC signing
from sifr.crypto import hmac_sign, hmac_verify
signature: bytes = hmac_sign(b"message", key)
is_valid: bool = hmac_verify(b"message", signature, key)
```

**Codegen:**

- `generate_key()` -> `rand::thread_rng().gen::<[u8; 32]>()`
- `encrypt()` -> `aes_gcm::Aes256Gcm::encrypt()` with random nonce prepended to ciphertext
- `decrypt()` -> `aes_gcm::Aes256Gcm::decrypt()` extracting nonce from first 12 bytes
- `hmac_sign()` -> `hmac::Hmac::<sha2::Sha256>::new_from_slice(key)` then `.update(msg)` then `.finalize()`
- `hmac_verify()` -> same but `.verify_slice(signature)`

### Secure Random (`sifr.crypto.random`)

```python
from sifr.crypto.random import token_hex, token_bytes, token_urlsafe

session_id: str = token_hex(32)        # 64-char hex string
raw: bytes = token_bytes(32)           # 32 random bytes
csrf: str = token_urlsafe(32)          # base64url-encoded random
```

**Codegen:** All map to `rand::thread_rng()` with appropriate encoding. These are convenience wrappers matching Python's `secrets` module.

### Rust Crate Mapping

- Password hashing: `argon2` (1.4M downloads/month, RustCrypto team) + `bcrypt` (273K downloads/month) for legacy support
- JWT: `jsonwebtoken` (4.6M downloads/month, #1 in Authentication category, 996 reverse deps)
- Encryption: `aes-gcm` (RustCrypto, 6.5M downloads/month via `aead`) -- NCC Group security audited
- HMAC: `hmac` + `sha2` (RustCrypto, already used by `sifr.hashlib` in milestone_ext_stdlib)
- Random: `rand` (already a dependency from milestone_ext_stdlib)

### Definition of Done (milestone_crypto_auth)

- `hash_password()` produces Argon2id hashes with secure defaults
- `verify_password()` verifies Argon2 and Bcrypt hashes with constant-time comparison
- Password hashing runs on blocking thread pool (does not block async runtime)
- `encode()` / `decode()` create and validate JWT tokens with typed payloads
- JWT expiration is checked automatically on decode
- `encrypt()` / `decrypt()` provide AES-256-GCM authenticated encryption
- `hmac_sign()` / `hmac_verify()` provide HMAC-SHA256 message authentication
- `token_hex()`, `token_bytes()`, `token_urlsafe()` generate cryptographically secure random values
- E2E pass tests: password_hash_verify, password_bcrypt_legacy, jwt_roundtrip, jwt_expiration, aes_encrypt_decrypt, hmac_sign_verify, secure_random
- E2E fail tests: wrong_password_rejected, expired_jwt_rejected, tampered_ciphertext_rejected, wrong_hmac_rejected
- Milestone demo in `./demos/milestone_crypto_auth_demo.sifr`

---

## milestone_web_production: Production Web Features

status: pending

**Goal:** Enhance the web stack with production-grade features that every deployed web application needs: structured JSON logging with request tracing, rate limiting, and CORS configuration. This milestone layers on top of `sifr.logging` (Phase 7: `logging.Logger` class with level filtering from milestone_stdlib_class_rollout) and `sifr.web` (milestone_web_db: basic routing and middleware) without modifying those locked milestones.

**Depends on:** milestone_crypto_auth (rate limiting may use token-based identification), milestone_web_db (web framework must exist), milestone_stdlib_class_rollout (`sifr.logging.Logger` provides named loggers with level filtering; this milestone extends it with JSON output and request tracing)

### Enhanced Logging (`sifr.logging` extensions)

Phase 7's `sifr.logging` (from milestone_stdlib_class_rollout) provides the `Logger` class with named loggers and level filtering (debug, info, warning, error, critical). This milestone adds production features on top:

```python
from sifr.logging import configure, info, warn, error

# JSON output mode for production (machine-readable logs)
configure(format="json")  # default is "pretty" (human-readable)

# Structured context fields -- key-value pairs attached to log entries
info("request handled", user_id=123, path="/api/users", status=200, duration_ms=42)
# JSON output: {"level":"INFO","message":"request handled","user_id":123,"path":"/api/users","status":200,"duration_ms":42,"timestamp":"2026-02-15T10:30:00Z"}

warn("slow query", query="SELECT * FROM users", duration_ms=1500)
error("payment failed", order_id="abc123", error="card_declined")
```

**Codegen:**

- `configure(format="json")` maps to `tracing_subscriber::fmt().json().init()`
- `configure(format="pretty")` maps to `tracing_subscriber::fmt().pretty().init()` (default, same as Phase 3)
- Structured fields `info("msg", key=val)` map to `tracing::info!(key = val, "msg")` -- tracing already supports this natively, this milestone adds the Sifr syntax sugar

### Request Tracing Middleware

Automatic request/response logging for web handlers -- every HTTP request is logged with method, path, status, and duration:

```python
from sifr.web import App
from sifr.logging import configure

configure(format="json")

app = App()
app.use_tracing()  # enable request tracing middleware

# Every request now automatically logs:
# {"level":"INFO","message":"request","method":"GET","path":"/api/users","status":200,"duration_ms":12,"request_id":"a1b2c3"}
```

**Codegen:** `app.use_tracing()` maps to `tower_http::trace::TraceLayer::new_for_http()` added as axum middleware. Request IDs are generated via `uuid::Uuid::new_v4()` and propagated via `tower_http::request_id::SetRequestIdLayer`.

### Rate Limiting

Protect endpoints from abuse with configurable rate limiting:

```python
from sifr.web import App, RateLimit

app = App()

# Global rate limit: 100 requests per minute per IP
app.use_rate_limit(RateLimit(requests=100, window=60))

# Per-route rate limit
@app.post("/login")
@app.rate_limit(requests=5, window=60)  # 5 attempts per minute
async def login(body: Json[LoginRequest]) -> Json[LoginResponse]:
    ...
```

**Codegen:** Rate limiting uses `tower::limit::RateLimitLayer` for global limits. Per-route limits use an in-memory token bucket (backed by `moka` cache for concurrent access). Returns 429 Too Many Requests when exceeded. IP extraction uses axum's `ConnectInfo` extractor.

### CORS Configuration

Explicit CORS configuration for API servers:

```python
from sifr.web import App, Cors

app = App()

# Allow specific origins
app.use_cors(Cors(
    origins=["https://myapp.com", "https://staging.myapp.com"],
    methods=["GET", "POST", "PUT", "DELETE"],
    headers=["Authorization", "Content-Type"],
    max_age=3600,
))

# Or allow all (development mode)
app.use_cors(Cors.allow_all())
```

**Codegen:** Maps to `tower_http::cors::CorsLayer` with the specified configuration. `Cors.allow_all()` maps to `CorsLayer::permissive()`.

### Rust Crate Mapping

- Logging: `tracing-subscriber` (260M+ downloads/month, Tokio team) -- JSON formatter already built in
- Request tracing: `tower-http` (already a dep from milestone_web_db) -- `TraceLayer`, `SetRequestIdLayer`
- Request IDs: `uuid` (130M+ downloads/month)
- Rate limiting: `tower` (already a dep) + `moka` (57M+ downloads, production-grade concurrent cache)
- CORS: `tower-http` (already a dep) -- `CorsLayer`

### Definition of Done (milestone_web_production)

- `configure(format="json")` switches logging to JSON output
- Structured context fields (`info("msg", key=val)`) emit key-value pairs in log output
- `app.use_tracing()` logs every request with method, path, status, duration, and request ID
- Request IDs are generated and propagated through the request lifecycle
- Global rate limiting (`app.use_rate_limit()`) works with configurable requests/window
- Per-route rate limiting (`@app.rate_limit()`) works independently of global limits
- Rate limit exceeded returns 429 with Retry-After header
- CORS configuration (`app.use_cors()`) works with specific origins and `allow_all()`
- E2E pass tests: json_logging, structured_fields, request_tracing, rate_limit_global, rate_limit_per_route, cors_specific_origin, cors_allow_all
- E2E fail tests: rate_limit_exceeded_429, cors_blocked_origin
- Milestone demo in `./demos/milestone_web_production_demo.sifr`

---

## milestone_redis: Redis Client

status: pending

**Goal:** Provide a native async Redis client for caching, session storage, pub/sub, and general key-value operations. Redis is the universal infrastructure layer for web applications -- used for caching, rate limiting, session storage, and real-time messaging.

**Depends on:** milestone_async (async runtime), milestone_typed_serde (typed values need serialization for cache get/set)

### Key-Value Operations (`sifr.redis`)

```python
from sifr.redis import Redis

redis = await Redis.connect("redis://localhost:6379")

# Basic key-value
await redis.set("user:1:name", "Alice")
name: str | None = await redis.get("user:1:name")

# With expiration (TTL in seconds)
await redis.set("session:abc123", session_data, ttl=3600)

# Typed get/set -- serialize classes to JSON automatically
class CachedUser:
    id: int
    name: str
    email: str

await redis.set_json("user:1", CachedUser(1, "Alice", "alice@example.com"))
user: CachedUser | None = await redis.get_json("user:1", CachedUser)

# Atomic operations
count: int = await redis.incr("page:views")
await redis.expire("temp:key", 60)
exists: bool = await redis.exists("user:1:name")
await redis.delete("old:key")
```

### Hash, List, and Set Operations

```python
# Hash maps (Redis HSET/HGET)
await redis.hset("user:1", "name", "Alice")
await redis.hset("user:1", "email", "alice@example.com")
name: str | None = await redis.hget("user:1", "name")
all_fields: dict[str, str] = await redis.hgetall("user:1")

# Lists (Redis LPUSH/RPUSH/LPOP/RPOP)
await redis.lpush("queue:tasks", "task1")
await redis.rpush("queue:tasks", "task2")
task: str | None = await redis.lpop("queue:tasks")

# Sets (Redis SADD/SMEMBERS)
await redis.sadd("tags:post:1", "rust", "sifr", "web")
tags: set[str] = await redis.smembers("tags:post:1")
```

### Pub/Sub

```python
from sifr.redis import Redis

redis = await Redis.connect("redis://localhost:6379")

# Subscribe
async def on_message(channel: str, message: str):
    print(f"[{channel}] {message}")

await redis.subscribe("notifications", on_message)

# Publish (from another connection)
await redis.publish("notifications", "New user signed up!")
```

### Connection Pooling

```python
from sifr.redis import Redis

# Connection pool (default: 10 connections)
redis = await Redis.connect("redis://localhost:6379", pool_size=20)

# All operations automatically use the pool
# No manual connection management needed
```

**Codegen:** All operations map to `redis` crate (redis-rs) commands. `Redis.connect()` maps to `redis::Client::open()` + `redis::aio::MultiplexedConnection`. Pool management uses `redis::aio::ConnectionManager`. `set_json`/`get_json` combine redis commands with `serde_json` serialization from milestone_typed_serde.

### Rust Crate Mapping

- `redis` crate (redis-rs): v1.0+, 4,156 GitHub stars, official Redis recommendation, tokio async support, connection pooling, cluster support. Used by major Rust projects.

### Definition of Done (milestone_redis)

- `Redis.connect()` establishes async connection with connection pooling
- Basic key-value: `get`, `set`, `delete`, `exists`, `expire`, `incr`, `ttl`
- Typed JSON: `set_json`, `get_json` with automatic serialization
- Hash operations: `hset`, `hget`, `hgetall`, `hdel`
- List operations: `lpush`, `rpush`, `lpop`, `rpop`, `lrange`, `llen`
- Set operations: `sadd`, `srem`, `smembers`, `sismember`
- Pub/sub: `subscribe`, `publish` with async message handler
- Connection pooling works transparently
- TTL/expiration works on all key types
- All operations return `Result[T, RedisError]`
- E2E pass tests: redis_connect, redis_get_set, redis_typed_json, redis_hash, redis_list, redis_set, redis_pubsub, redis_ttl, redis_pool
- E2E fail tests: redis_connection_refused, redis_wrong_type
- Milestone demo in `./demos/milestone_redis_demo.sifr`

---

## milestone_storage: Object Storage (S3)

status: pending

**Goal:** Provide a native client for S3-compatible object storage. Works with AWS S3, Cloudflare R2, MinIO, DigitalOcean Spaces, and any S3-compatible service. Object storage is essential for file uploads, media hosting, backups, and static asset delivery in web applications.

**Depends on:** milestone_async (async runtime), milestone_typed_serde (metadata serialization), milestone_web_db (commonly used with web handlers for file upload flows)

### Basic Operations (`sifr.storage`)

```python
from sifr.storage import Bucket

# Connect to S3 (reads credentials from env: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
bucket = Bucket("my-bucket", region="us-east-1")

# Upload
await bucket.put("photos/avatar.jpg", image_data)
await bucket.put("docs/report.pdf", pdf_bytes, content_type="application/pdf")

# Upload from file path
await bucket.put_file("backups/db.sql", "/tmp/dump.sql")

# Download
data: Result[bytes, StorageError] = await bucket.get("photos/avatar.jpg")
await bucket.get_file("photos/avatar.jpg", "/tmp/avatar.jpg")  # download to file

# Delete
await bucket.delete("old/file.txt")

# Check existence
exists: bool = await bucket.exists("photos/avatar.jpg")

# List objects
objects: list[ObjectInfo] = await bucket.list(prefix="photos/")
for obj in objects:
    print(f"{obj.key}: {obj.size} bytes, modified {obj.last_modified}")
```

### Presigned URLs

```python
# Generate a presigned URL for temporary access (no credentials needed to access)
download_url: str = await bucket.presign_get("photos/avatar.jpg", expires=3600)  # 1 hour
upload_url: str = await bucket.presign_put("uploads/new-file.jpg", expires=600)   # 10 minutes
```

### S3-Compatible Services

```python
from sifr.storage import Bucket

# Cloudflare R2
bucket = Bucket(
    "my-r2-bucket",
    endpoint="https://ACCOUNT_ID.r2.cloudflarestorage.com",
    region="auto",
)

# MinIO (local development)
bucket = Bucket(
    "dev-bucket",
    endpoint="http://localhost:9000",
    region="us-east-1",
)

# DigitalOcean Spaces
bucket = Bucket(
    "my-space",
    endpoint="https://nyc3.digitaloceanspaces.com",
    region="nyc3",
)
```

### Web Integration (upload flow)

```python
from sifr.web import App, UploadFile, Json
from sifr.storage import Bucket
from sifr.crypto.random import token_urlsafe

app = App()
bucket = Bucket("uploads-bucket", region="us-east-1")

@app.post("/upload")
async def upload_file(file: UploadFile) -> Json[dict[str, str]]:
    key: str = f"uploads/{token_urlsafe(16)}/{file.filename}"
    data: bytes = await file.read()
    await bucket.put(key, data, content_type=file.content_type)
    url: str = await bucket.presign_get(key, expires=86400)
    return Json({"key": key, "url": url})
```

**Codegen:** All operations map to the `rust-s3` crate. `Bucket(name, region)` maps to `s3::Bucket::new()` with `s3::Region` and `s3::creds::Credentials::from_env()`. `put()` maps to `bucket.put_object()`. `get()` maps to `bucket.get_object()`. Presigned URLs use `bucket.presign_get()` / `bucket.presign_put()`.

**Why `rust-s3` over `aws-sdk-s3`:** `rust-s3` supports S3-compatible services (R2, MinIO, Spaces) natively with custom endpoints. `aws-sdk-s3` is AWS-only and pulls in the entire AWS SDK (~14MB). `rust-s3` is 52KB, async-native, and explicitly designed for multi-provider use.

### Rust Crate Mapping

- `rust-s3`: v0.37+, 6M+ total downloads, supports S3/R2/MinIO/Spaces/Wasabi/GCP, tokio async, multipart uploads

### Definition of Done (milestone_storage)

- `Bucket` connects to S3-compatible storage using env credentials
- `put()` / `put_file()` upload bytes or files to object storage
- `get()` / `get_file()` download objects to bytes or files
- `delete()` removes objects
- `exists()` checks object existence
- `list()` lists objects with prefix filtering, returns `ObjectInfo` (key, size, last_modified)
- `presign_get()` / `presign_put()` generate presigned URLs with configurable expiration
- Custom endpoints work for R2, MinIO, DigitalOcean Spaces
- Content-type is set correctly on upload
- All operations return `Result[T, StorageError]`
- E2E pass tests: s3_put_get, s3_delete, s3_list, s3_presign, s3_put_file, s3_custom_endpoint
- E2E fail tests: s3_bucket_not_found, s3_object_not_found, s3_invalid_credentials
- Milestone demo in `./demos/milestone_storage_demo.sifr`

---

## milestone_email: Email

status: pending

**Goal:** Provide a native email client for sending transactional emails (registration confirmations, password resets, notifications). Supports plain text, HTML, and attachments over SMTP.

**Depends on:** milestone_async (async SMTP transport), milestone_typed_serde (email template data)

### Sending Email (`sifr.email`)

```python
from sifr.email import Email, SmtpClient

# Configure SMTP (reads from env: SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASSWORD)
smtp = await SmtpClient.connect(
    host="smtp.example.com",
    port=587,
    username="user@example.com",
    password="secret",
    tls=True,
)

# Simple text email
email = Email(
    to="alice@example.com",
    subject="Welcome to Sifr!",
    body="Hello Alice, welcome aboard.",
)
await smtp.send(email)

# HTML email
email = Email(
    to="alice@example.com",
    subject="Your weekly report",
    html="<h1>Weekly Report</h1><p>Here are your stats...</p>",
)
await smtp.send(email)

# With attachments
email = Email(
    to="alice@example.com",
    subject="Invoice attached",
    body="Please find your invoice attached.",
    attachments=["/path/to/invoice.pdf"],
)
await smtp.send(email)

# Multiple recipients, CC, BCC
email = Email(
    to=["alice@example.com", "bob@example.com"],
    cc=["manager@example.com"],
    bcc=["archive@example.com"],
    from_addr="noreply@myapp.com",
    subject="Team update",
    body="Important update for the team.",
)
await smtp.send(email)
```

### Environment-based Configuration

```python
from sifr.email import SmtpClient

# Reads SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASSWORD from environment
smtp = await SmtpClient.from_env()
```

**Codegen:** `SmtpClient.connect()` maps to `lettre::AsyncSmtpTransport::relay()` with `lettre::transport::smtp::authentication::Credentials`. `Email(...)` builds a `lettre::Message` via `lettre::message::MessageBuilder`. Attachments use `lettre::message::MultiPart` and `lettre::message::Attachment`. HTML emails use `lettre::message::SinglePart::html()`. Async transport uses `lettre::AsyncSmtpTransport` with tokio runtime.

### Rust Crate Mapping

- `lettre`: v0.11+, 2,173 GitHub stars, the standard Rust email library, async support via tokio, MIME/attachment support, TLS via rustls, actively maintained

### Definition of Done (milestone_email)

- `SmtpClient.connect()` establishes async SMTP connection with TLS
- `SmtpClient.from_env()` reads SMTP configuration from environment variables
- `Email(...)` constructs plain text, HTML, or multipart emails
- Multiple recipients, CC, BCC, custom from address work
- File attachments work
- `smtp.send(email)` sends email asynchronously, returns `Result[None, EmailError]`
- E2E pass tests: email_text, email_html, email_attachment, email_multiple_recipients, email_from_env
- E2E fail tests: email_invalid_address, email_smtp_connection_refused
- Milestone demo in `./demos/milestone_email_demo.sifr`

---

## milestone_data_processing: Data Processing

status: pending

**Goal:** Enable data science and data engineering workflows. This is what makes sifr competitive with Python's pandas/polars ecosystem.

### DataFrame Library (`sifr.data`)

Thin wrapper around `polars` -- the fastest DataFrame library, written in Rust:

- **DataFrame creation:** from CSV, Parquet, JSON, dicts
- **Lazy evaluation:** query optimization before execution
- **Expressions:** filter, select, group_by, join, sort, aggregate
- **I/O:** CSV, Parquet, JSON, Arrow IPC, cloud storage
- **Streaming:** process datasets larger than RAM

```python
from sifr.data import DataFrame, col, lit

def main():
    # Read data
    df = DataFrame.read_csv("sales.csv")

    # Transform (lazy evaluation)
    result = (
        df.lazy()
        .filter(col("amount") > 100)
        .group_by("region")
        .agg(
            col("amount").sum().alias("total"),
            col("amount").mean().alias("average"),
            col("id").count().alias("count"),
        )
        .sort("total", descending=True)
        .collect()
    )

    # Write output
    result.write_parquet("summary.parquet")
    print(result)
```

### Additional Data Modules

- `**sifr.csv`:** simple CSV read/write (for when full DataFrame is overkill) -> wraps `csv` crate
- `**sifr.args`:** CLI argument parsing with typed arguments -> wraps `clap` (derive mode)

### Rust Crate Mapping

- `sifr.data` -> `polars`
- `sifr.csv` -> `csv`
- `sifr.args` -> `clap`

### Definition of Done (milestone_data_processing)

- `sifr.data.DataFrame` wraps polars DataFrame with Pythonic API
- Lazy evaluation chain (filter, group_by, agg, sort) compiles correctly
- CSV/Parquet read/write works end-to-end
- `sifr.args` provides typed CLI argument parsing
- E2E pass tests: dataframe_basic, csv_roundtrip, cli_args
- Milestone demo in `./demos/milestone_data_processing_demo.sifr` (data pipeline)

---

## Milestone ordering

Why the milestones within this phase are in this order:

- **milestone_async before milestone_networking_stdlib:** The async runtime must exist before networking stdlib modules (socket, http, subprocess) that require async I/O primitives.
- **milestone_networking_stdlib before milestone_web_db:** Networking stdlib modules (socket, http, url) provide the foundation that the web framework and database milestones build on.
- **milestone_async before milestone_web_db:** Async runtime is needed for web framework and database access.
- **milestone_typed_serde after milestone_web_db:** The web framework must exist before we can add typed extractors (`Json[T]`, `Form[T]`). Typed serde also enhances `sifr.json` from milestone_core_stdlib with class serialization.
- **milestone_crypto_auth after milestone_typed_serde:** JWT payloads are classes that need auto-serde. Password hashing and encryption are independent but benefit from the typed patterns established in milestone_typed_serde.
- **milestone_web_production after milestone_crypto_auth:** Production web features (logging, tracing, rate limiting, CORS) layer on top of the web framework and benefit from having auth in place (rate limiting by authenticated user, request tracing with user context).
- **milestone_redis after milestone_web_production:** Redis is used for session storage (which needs auth tokens from milestone_crypto_auth), caching (which needs typed JSON serialization from milestone_typed_serde), and rate limiting state (which can be upgraded from in-memory to Redis-backed after this milestone). The `set_json`/`get_json` methods depend on auto-serde.
- **milestone_storage after milestone_redis:** Object storage is often used alongside Redis (cache presigned URLs, track upload status). The web upload integration pattern (`UploadFile` -> S3) depends on milestone_typed_serde's file upload support.
- **milestone_email after milestone_storage:** Email is the least dependent on other milestones but benefits from having the full web stack available (send emails from web handlers with attachments from object storage).
- **milestone_data_processing remains last in this phase:** Data processing is independent of web infrastructure and serves a different use case (data science/engineering).
