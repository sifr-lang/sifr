# milestone_ext_stdlib — Extended Standard Library

## 1. Product Requirements

### Objective

Fill out the remaining stdlib modules — utilities commonly needed in real programs. Uses the stdlib infrastructure established in milestone_core_stdlib.

### Scope — Scoped Down for Initial Implementation

**In Scope:**

1. **`sifr.time`** — `time_now() -> float` (epoch seconds), `sleep(seconds)`, `time_format(epoch, fmt) -> str`
2. **`sifr.random`** — `random_int(min, max) -> int`, `random_float() -> float`, `random_choice(items) -> int`
3. **`sifr.re`** — `re_match(pattern, text) -> bool`, `re_find(pattern, text) -> str | None`, `re_replace(pattern, replacement, text) -> str`
4. **`sifr.hash`** — `sha256(s) -> str`, `md5(s) -> str`
5. **`sifr.encoding`** — `base64_encode(s) -> str`, `base64_decode(s) -> str`

**Out of Scope (deferred):**

| Feature | Reason |
| --- | --- |
| `sifr.stream` | Complex, requires Read/Write traits |
| `sifr.log` | Lower priority |
| `sifr.math` (additional) | Already implemented in milestone_core_stdlib |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | `sifr.time`: time_now, sleep, time_format work |
| AC-2 | `sifr.random`: random_int, random_float, random_choice work |
| AC-3 | `sifr.re`: regex match, find, replace work |
| AC-4 | `sifr.hash`: sha256, md5 produce correct hex digests |
| AC-5 | `sifr.encoding`: base64 encode/decode roundtrip works |
| AC-6 | All existing E2E tests pass (no regressions) |

---

## 2. Solution Design

### 2.1 Rust Crate Mapping

| Sifr Module | Rust Crate | Cargo Dependency |
| --- | --- | --- |
| `sifr.time` | `std::time` + `chrono` | `chrono = "0.4"` |
| `sifr.random` | `rand` | `rand = "0.8"` |
| `sifr.re` | `regex` | `regex = "1"` |
| `sifr.hash` | `sha2` + `md5` | `sha2 = "0.10"`, `md5 = "0.7"` |
| `sifr.encoding` | `base64` | `base64 = "0.22"` |

### 2.2 Function-to-Rust Mapping

| Sifr Function | Rust Code |
| --- | --- |
| `time_now()` | `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()` |
| `sleep(s)` | `std::thread::sleep(Duration::from_secs_f64(s))` |
| `random_int(min, max)` | `rand::Rng::gen_range(min..=max)` |
| `random_float()` | `rand::random::<f64>()` |
| `re_match(pat, text)` | `regex::Regex::new(pat).unwrap().is_match(text)` |
| `re_find(pat, text)` | `regex::Regex::new(pat).unwrap().find(text).map(\|m\| m.as_str().to_string())` |
| `re_replace(pat, rep, text)` | `regex::Regex::new(pat).unwrap().replace_all(text, rep).to_string()` |
| `sha256(s)` | `sha2::Digest::finalize(sha2::Sha256::digest(s.as_bytes()))` hex format |
| `md5(s)` | `md5::compute(s.as_bytes())` hex format |
| `base64_encode(s)` | `base64::Engine::encode(base64::engine::general_purpose::STANDARD, s)` |
| `base64_decode(s)` | `String::from_utf8(base64::Engine::decode(STANDARD, s).unwrap()).unwrap()` |

### 2.3 Testing Strategy

- E2E tests for each module
- Demo: `demos/milestone_ext_stdlib_demo.sifr`
