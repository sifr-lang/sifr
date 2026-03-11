# Existing Modules — Gap Analysis

Detailed function-by-function comparison of all 37 Sifr stdlib modules against their CPython equivalents.

**Legend:**
- ✅ = Sifr has this
- ❌ = Missing from Sifr
- ⚠️ = Partial / different API
- 🚫 = Not applicable to Sifr's design

---

## Tier 1: Core Modules (from stdlib_migration)

### 1. `sifr.math` vs `math` — Coverage: ~60% (35/58 functions)

**Constants:**

| CPython | Sifr | Status |
|---------|------|--------|
| `pi` | `pi` | ✅ |
| `e` | `e` | ✅ |
| `tau` | `tau` | ✅ |
| `inf` | `inf` | ✅ |
| `nan` | `nan` | ✅ |

**Trigonometric:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `sin(x)` | `sin(x)` | ✅ | |
| `cos(x)` | `cos(x)` | ✅ | |
| `tan(x)` | `tan(x)` | ✅ | |
| `asin(x)` | `asin(x)` | ✅ | |
| `acos(x)` | `acos(x)` | ✅ | |
| `atan(x)` | `atan(x)` | ✅ | |
| `atan2(y, x)` | `atan2(y, x)` | ✅ | |
| `sinh(x)` | `sinh(x)` | ✅ | |
| `cosh(x)` | `cosh(x)` | ✅ | |
| `tanh(x)` | `tanh(x)` | ✅ | |
| `acosh(x)` | — | ❌ | Inverse hyperbolic cosine |
| `asinh(x)` | — | ❌ | Inverse hyperbolic sine |
| `atanh(x)` | — | ❌ | Inverse hyperbolic tangent |
| `hypot(x, y)` | `hypot(x, y)` | ✅ | |
| `dist(p, q)` | — | ❌ | Euclidean distance between two points |

**Exponential / Logarithmic:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `exp(x)` | `exp(x)` | ✅ | |
| `expm1(x)` | `expm1(x)` | ✅ | |
| `exp2(x)` | — | ❌ | 2^x |
| `log(x)` | `log(x)` | ✅ | |
| `log1p(x)` | `log1p(x)` | ✅ | |
| `log2(x)` | `log2(x)` | ✅ | |
| `log10(x)` | `log10(x)` | ✅ | |
| `pow(x, y)` | `pow(x, y)` | ✅ | |

**Rounding / Truncation:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `ceil(x)` | `ceil(x)` | ✅ | |
| `floor(x)` | `floor(x)` | ✅ | |
| `trunc(x)` | `trunc(x)` | ✅ | |
| `fabs(x)` | `fabs(x)` | ✅ | |
| `copysign(x, y)` | `copysign(x, y)` | ✅ | |
| `fmod(x, y)` | `fmod(x, y)` | ✅ | |
| `remainder(x, y)` | — | ❌ | IEEE 754-style remainder |
| `modf(x)` | — | ❌ | Returns (fractional, integer) parts |
| `frexp(x)` | — | ❌ | Returns (mantissa, exponent) |
| `ldexp(x, i)` | — | ❌ | x * 2^i |

**Special Functions:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `erf(x)` | — | ❌ | Error function |
| `erfc(x)` | — | ❌ | Complementary error function |
| `gamma(x)` | — | ❌ | Gamma function |
| `lgamma(x)` | — | ❌ | Log of absolute value of gamma |

**Floating-Point Inspection:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `isfinite(x)` | `isfinite(x)` | ✅ | |
| `isinf(x)` | `isinf(x)` | ✅ | |
| `isnan(x)` | `isnan(x)` | ✅ | |
| `isclose(a, b)` | `isclose(a, b, rel_tol)` | ✅ | |
| `isnormal(x)` | — | ❌ | Check if normal float |
| `issubnormal(x)` | — | ❌ | Check if subnormal float |
| `signbit(x)` | — | ❌ | Check sign bit |
| `nextafter(x, y)` | — | ❌ | Next representable float |
| `ulp(x)` | — | ❌ | Unit in the last place |

**Combinatorics / Integer:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `factorial(n)` | `factorial(n)` | ✅ | |
| `gcd(a, b)` | `gcd(a, b)` | ✅ | |
| `lcm(a, b)` | `lcm(a, b)` | ✅ | |
| `comb(n, k)` | `comb(n, k)` | ✅ | |
| `perm(n, k)` | `perm(n, k)` | ✅ | |
| `isqrt(n)` | — | ❌ | Integer square root |

**Aggregation:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `prod(iterable)` | `prod(data)` | ✅ | |
| `fsum(iterable)` | — | ❌ | Accurate floating-point sum |
| `sumprod(p, q)` | — | ❌ | Sum of products |

**Other:**

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `sqrt(x)` | `sqrt(x)` | ✅ | |
| `cbrt(x)` | — | ❌ | Cube root |
| `degrees(x)` | `degrees(x)` | ✅ | |
| `radians(x)` | `radians(x)` | ✅ | |
| `fma(x, y, z)` | — | ❌ | Fused multiply-add |
| `fmax(x, y)` | — | ❌ | Max of two floats (handles NaN) |
| `fmin(x, y)` | — | ❌ | Min of two floats (handles NaN) |

**Missing count: 23 functions**

---

### 2. `sifr.os` vs `os` — Coverage: ~20% (16/80+ functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `getcwd()` | `getcwd()` | ✅ | |
| `listdir(path)` | `listdir(path)` | ✅ | |
| `mkdir(path)` | `mkdir(path)` | ✅ | |
| `makedirs(path)` | `makedirs(path)` | ✅ | |
| `rmdir(path)` | `rmdir(path)` | ✅ | |
| `remove(path)` | `remove_file(path)` | ⚠️ | Different name |
| `rename(old, new)` | `rename(src, dst)` | ✅ | |
| `walk(top)` | `walk_dir(path)` | ⚠️ | Returns flat list, not (dirpath, dirnames, filenames) tuples |
| `system(command)` | `run_command(cmd)` | ⚠️ | Returns output, not exit code |
| `chdir(path)` | — | ❌ | Change directory |
| `getenv(key)` | — | ❌ | In `sifr.env` as `env_get`, not in `sifr.os` |
| `putenv(key, val)` | — | ❌ | In `sifr.env` as `env_set`, not in `sifr.os` |
| `environ` | — | ❌ | Dict-like env access |
| `getpid()` | — | ❌ | Process ID |
| `getuid()` | — | ❌ | User ID (Unix) |
| `cpu_count()` | — | ❌ | Number of CPUs |
| `urandom(n)` | — | ❌ | Random bytes |
| `access(path, mode)` | — | ❌ | Check file permissions |
| `chmod(path, mode)` | — | ❌ | Change permissions |
| `stat(path)` | — | ❌ | File status |
| `scandir(path)` | — | ❌ | Directory iterator |
| `popen(cmd)` | — | ❌ | Pipe to command |
| `path.join(*paths)` | — | ❌ | In `sifr.pathlib` as `join_path` |
| `path.exists(path)` | — | ❌ | In `sifr.pathlib` / `sifr.io` |
| `path.isfile(path)` | `is_file(path)` | ✅ | |
| `path.isdir(path)` | `is_dir(path)` | ✅ | |
| `path.basename(path)` | — | ❌ | In `sifr.pathlib` |
| `path.dirname(path)` | — | ❌ | In `sifr.pathlib` |
| `path.splitext(path)` | — | ❌ | Split into (root, ext) |
| `path.abspath(path)` | — | ❌ | Absolute path |
| `path.expanduser(path)` | — | ❌ | Expand ~ |
| `path.normpath(path)` | — | ❌ | Normalize path |
| `path.relpath(path)` | — | ❌ | Relative path |
| `path.getsize(path)` | — | ❌ | File size |
| `path.getmtime(path)` | — | ❌ | Modification time |
| `path.islink(path)` | — | ❌ | Check if symlink |
| `path.isabs(path)` | — | ❌ | Check if absolute |
| `sep` | — | ❌ | Path separator constant |
| `linesep` | — | ❌ | Line separator constant |
| `name` | — | ❌ | OS name ('posix', 'nt') |
| `devnull` | — | ❌ | Null device path |

**Design note:** Sifr splits `os` functionality across `sifr.os`, `sifr.env`, `sifr.pathlib`, and `sifr.io`. This is arguably better design than CPython's monolithic `os` module, but users coming from Python will expect `os.path.join`, `os.environ`, etc.

---

### 3. `sifr.io` vs `io` — Coverage: ~15% (5 functions vs 15+ classes)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `open(file, mode)` | — | ❌ | **Critical gap** — Python's most-used function |
| `read_text` (via Path) | `read_text(path)` | ✅ | |
| `write_text` (via Path) | `write_text(path, content)` | ✅ | |
| `read_lines` | `read_lines(path)` | ✅ | Not in CPython's io |
| `exists` | `exists(path)` | ✅ | Not in CPython's io |
| `append_text` | `append_text(path, content)` | ✅ | Not in CPython's io |
| `StringIO` | — | ❌ | In-memory text stream |
| `BytesIO` | — | ❌ | In-memory bytes stream |
| `FileIO` | — | ❌ | Raw file I/O |
| `BufferedReader` | — | ❌ | Buffered reading |
| `BufferedWriter` | — | ❌ | Buffered writing |
| `TextIOWrapper` | — | ❌ | Text wrapper |
| `SEEK_SET/CUR/END` | — | ❌ | Seek constants |

**Design note:** Sifr's approach (read_text/write_text as simple functions) is practical for common cases but lacks streaming I/O, binary I/O, and the file-object protocol that many Python APIs depend on (csv.reader, json.load, etc.).

---

### 4. `sifr.re` vs `re` — Coverage: ~40% (5 functions + Match class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `search(pattern, string)` | `search(pattern, text)` | ✅ | Returns `str | None`, not Match |
| `search` → Match | `search_match(pattern, text)` | ⚠️ | Different name; CPython's `search` returns Match |
| `match(pattern, string)` | — | ❌ | Match at start of string |
| `fullmatch(pattern, string)` | — | ❌ | Match entire string |
| `sub(pattern, repl, string)` | `sub(pattern, replacement, text)` | ✅ | |
| `subn(pattern, repl, string)` | — | ❌ | Sub with replacement count |
| `findall(pattern, string)` | `findall(pattern, text)` | ✅ | |
| `finditer(pattern, string)` | — | ❌ | Iterator of Match objects |
| `split(pattern, string)` | `split(pattern, text)` | ✅ | |
| `compile(pattern)` | — | ❌ | Compile to Pattern object |
| `escape(pattern)` | — | ❌ | Escape special characters |
| `purge()` | — | ❌ | Clear regex cache |
| `Match.group()` | `Match.group()` | ✅ | |
| `Match.groups()` | — | ❌ | All groups as tuple |
| `Match.groupdict()` | — | ❌ | Named groups as dict |
| `Match.start()` | `Match.start()` | ✅ | |
| `Match.end()` | `Match.end()` | ✅ | |
| `Match.span()` | `Match.span()` | ✅ | |
| `Match.string` | — | ❌ | Original string |
| `Match.re` | — | ❌ | Pattern used |
| `IGNORECASE` | — | ❌ | Flag |
| `MULTILINE` | — | ❌ | Flag |
| `DOTALL` | — | ❌ | Flag |
| `VERBOSE` | — | ❌ | Flag |

**Design note:** `search` returning `str | None` instead of `Match | None` is a significant API divergence. `search_match` exists but has a non-standard name.

---

### 5. `sifr.json` vs `json` — Coverage: ~30% (2/4 functions, 0/3 classes)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `loads(s)` | `loads(s)` | ✅ | Returns str (serialized), not dict/list |
| `dumps(obj)` | `dumps` | ⚠️ | Available via intrinsic but not re-exported cleanly |
| `load(fp)` | — | ❌ | Load from file object |
| `dump(obj, fp)` | — | ❌ | Dump to file object |
| `JSONEncoder` | — | ❌ | Encoder class |
| `JSONDecoder` | — | ❌ | Decoder class |
| `JSONDecodeError` | — | ❌ | Error type |

**Design note:** `loads` returning `str` instead of a structured type (dict/list) is a fundamental limitation. True JSON parsing requires returning typed data structures, which needs generic dict/list support.

---

### 6. `sifr.time` vs `time` — Coverage: ~35% (5/14 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `time()` | `time()` | ✅ | |
| `sleep(secs)` | `sleep(seconds)` | ✅ | |
| `strftime(format, t)` | `strftime(epoch, fmt)` | ✅ | |
| `perf_counter()` | `perf_counter()` | ✅ | |
| `monotonic()` | `monotonic()` | ✅ | |
| `time_ns()` | — | ❌ | Nanosecond precision |
| `perf_counter_ns()` | — | ❌ | Nanosecond precision |
| `monotonic_ns()` | — | ❌ | Nanosecond precision |
| `process_time()` | — | ❌ | CPU time |
| `thread_time()` | — | ❌ | Thread CPU time |
| `gmtime(secs)` | — | ❌ | UTC struct_time |
| `localtime(secs)` | — | ❌ | Local struct_time |
| `mktime(t)` | — | ❌ | struct_time to timestamp |
| `strptime(string, format)` | — | ❌ | Parse time string |
| `ctime(secs)` | — | ❌ | Timestamp to string |
| `asctime(t)` | — | ❌ | struct_time to string |
| `get_clock_info(name)` | — | ❌ | Clock info |
| `struct_time` | — | ❌ | Time tuple class |

---

### 7. `sifr.hashlib` vs `hashlib` — Coverage: ~30% (4/14+ functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `sha256(data)` | `sha256(s)` | ⚠️ | Takes str, not bytes; returns hex string directly |
| `md5(data)` | `md5(s)` | ⚠️ | Same |
| `sha1(data)` | `sha1(s)` | ✅ | |
| `sha512(data)` | `sha512(s)` | ✅ | |
| `sha224(data)` | — | ❌ | |
| `sha384(data)` | — | ❌ | |
| `blake2b(data)` | — | ❌ | |
| `blake2s(data)` | — | ❌ | |
| `sha3_224(data)` | — | ❌ | |
| `sha3_256(data)` | — | ❌ | |
| `sha3_384(data)` | — | ❌ | |
| `sha3_512(data)` | — | ❌ | |
| `new(name, data)` | — | ❌ | Generic hash constructor |
| `pbkdf2_hmac(...)` | — | ❌ | Key derivation |
| `scrypt(...)` | — | ❌ | Key derivation |
| `file_digest(file, name)` | — | ❌ | Hash file contents |
| `algorithms_guaranteed` | — | ❌ | Set of guaranteed algorithms |
| `algorithms_available` | — | ❌ | Set of available algorithms |

**Design note:** CPython's hashlib returns hash objects with `.hexdigest()`, `.digest()`, `.update()` methods. Sifr returns hex strings directly — simpler but less flexible (can't do incremental hashing).

---

### 8. `sifr.base64` vs `base64` — Coverage: ~30% (4/16 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `b64encode(s)` | `b64encode(s)` | ✅ | |
| `b64decode(s)` | `b64decode(s)` | ✅ | |
| `urlsafe_b64encode(s)` | `urlsafe_b64encode(s)` | ✅ | |
| `urlsafe_b64decode(s)` | `urlsafe_b64decode(s)` | ✅ | |
| `b32encode(s)` | — | ❌ | |
| `b32decode(s)` | — | ❌ | |
| `b16encode(s)` | — | ❌ | |
| `b16decode(s)` | — | ❌ | |
| `b85encode(s)` | — | ❌ | |
| `b85decode(s)` | — | ❌ | |
| `a85encode(s)` | — | ❌ | |
| `a85decode(s)` | — | ❌ | |
| `standard_b64encode(s)` | — | ❌ | |
| `standard_b64decode(s)` | — | ❌ | |
| `encode(input, output)` | — | ❌ | File-based |
| `decode(input, output)` | — | ❌ | File-based |

---

### 9. `sifr.random` vs `random` — Coverage: ~12.5% (3/24 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `random()` | `random()` | ✅ | |
| `uniform(a, b)` | `uniform(min, max)` | ✅ | |
| `randint(a, b)` | `randint(min, max)` | ✅ | |
| `choice(seq)` | — | ❌ | Intrinsic exists (`random_choice`) but not exposed |
| `choices(population, weights, k)` | — | ❌ | Weighted random choices |
| `shuffle(x)` | — | ❌ | Shuffle list in-place |
| `sample(population, k)` | — | ❌ | Random sample without replacement |
| `randrange(start, stop, step)` | — | ❌ | Random int in range |
| `seed(a)` | — | ❌ | Seed the RNG |
| `getstate()` | — | ❌ | Get RNG state |
| `setstate(state)` | — | ❌ | Set RNG state |
| `getrandbits(k)` | — | ❌ | Random k-bit integer |
| `randbytes(n)` | — | ❌ | Random bytes |
| `triangular(low, high, mode)` | — | ❌ | Triangular distribution |
| `gauss(mu, sigma)` | — | ❌ | Gaussian distribution |
| `normalvariate(mu, sigma)` | — | ❌ | Normal distribution |
| `lognormvariate(mu, sigma)` | — | ❌ | Log-normal distribution |
| `expovariate(lambd)` | — | ❌ | Exponential distribution |
| `gammavariate(alpha, beta)` | — | ❌ | Gamma distribution |
| `betavariate(alpha, beta)` | — | ❌ | Beta distribution |
| `paretovariate(alpha)` | — | ❌ | Pareto distribution |
| `vonmisesvariate(mu, kappa)` | — | ❌ | Von Mises distribution |
| `weibullvariate(alpha, beta)` | — | ❌ | Weibull distribution |
| `binomialvariate(n, p)` | — | ❌ | Binomial distribution |
| `Random` class | — | ❌ | RNG class |
| `SystemRandom` class | — | ❌ | OS-based RNG |

---

### 10. `sifr.bytes` vs `bytes`/`bytearray` — Coverage: ~15% (4 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `bytes.decode(encoding)` | `decode_utf8(bytes)` | ⚠️ | Only UTF-8 |
| `str.encode(encoding)` | `encode_utf8(s)` | ⚠️ | Only UTF-8 |
| `bytes.hex()` | `bytes_to_hex(bytes)` | ✅ | |
| `bytes.fromhex(s)` | `bytes_from_hex(s)` | ✅ | |
| `bytes(...)` constructor | — | ❌ | |
| `bytearray(...)` constructor | — | ❌ | |
| `bytes.count(sub)` | — | ❌ | |
| `bytes.find(sub)` | — | ❌ | |
| `bytes.replace(old, new)` | — | ❌ | |
| `bytes.split(sep)` | — | ❌ | |
| `bytes.join(iterable)` | — | ❌ | |
| `bytes.strip()` | — | ❌ | |
| `bytes.startswith(prefix)` | — | ❌ | |
| `bytes.endswith(suffix)` | — | ❌ | |
| `bytes.upper()` / `lower()` | — | ❌ | |
| `bytearray` (mutable) | — | ❌ | Mutable byte sequence |

**Design note:** In Sifr, `bytes` should probably be a built-in type (like `str`, `list`, `dict`) rather than a stdlib module. CPython's `bytes` is a built-in type with ~40 methods.

---

### 11. `sifr.collections` vs `collections` — Coverage: ~25% (2/9 classes)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `Counter` | `Counter` | ⚠️ | Basic implementation — missing `update`, `subtract`, `elements`, arithmetic ops |
| `defaultdict` | `defaultdict_*` | ⚠️ | Intrinsic functions, not a class; limited to `int` default |
| `deque` | — | ❌ | Double-ended queue — very commonly used |
| `OrderedDict` | — | ❌ | Ordered dictionary |
| `ChainMap` | — | ❌ | Multiple dict view |
| `namedtuple` | — | ❌ | Named tuple factory |
| `UserDict` | — | 🚫 | For subclassing dict — not needed with Sifr's type system |
| `UserList` | — | 🚫 | For subclassing list — not needed |
| `UserString` | — | 🚫 | For subclassing str — not needed |

**Counter missing methods:** `update(iterable)`, `subtract(iterable)`, `elements()`, `__add__`, `__sub__`, `__and__`, `__or__`, `most_common()` (exists but limited), `copy()`, `clear()`

---

### 12. `sifr.env` vs `os.environ` — Coverage: ~50% (2 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `os.environ[key]` | `env_get(key)` | ⚠️ | Function, not dict-like |
| `os.environ[key] = val` | `env_set(key, val)` | ⚠️ | Function, not dict-like |
| `os.environ.get(key, default)` | — | ❌ | Get with default |
| `os.environ.keys()` | — | ❌ | All env var names |
| `os.environ.values()` | — | ❌ | All env var values |
| `os.environ.items()` | — | ❌ | All env var pairs |
| `os.unsetenv(key)` | — | ❌ | Remove env var |

**Design note:** `sifr.env` is a non-standard module. In CPython, environment variables are accessed via `os.environ` (a dict-like object) or `os.getenv()`.

---

### 13. `sifr.test` — Custom Module (No CPython Equivalent)

This is a Sifr-specific test assertion module. CPython's equivalent would be `unittest` + `assert` statement. No gap analysis needed — this is intentionally different.

---

## Tier 2: Expansion Modules

### 14. `sifr.string` vs `string` — Coverage: ~65% (9 constants + 1 function)

| CPython | Sifr | Status |
|---------|------|--------|
| `ascii_lowercase` | `ascii_lowercase` | ✅ |
| `ascii_uppercase` | `ascii_uppercase` | ✅ |
| `ascii_letters` | `ascii_letters` | ✅ |
| `digits` | `digits` | ✅ |
| `hexdigits` | `hexdigits` | ✅ |
| `octdigits` | `octdigits` | ✅ |
| `punctuation` | `punctuation` | ✅ |
| `whitespace` | `whitespace` | ✅ |
| `printable` | `printable` | ✅ |
| `capwords(s)` | `capwords(s)` | ✅ |
| `Formatter` class | — | ❌ |
| `Template` class | — | ❌ |

---

### 15. `sifr.statistics` vs `statistics` — Coverage: ~60% (12/20 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `mean(data)` | `mean(data)` | ✅ | |
| `fmean(data)` | `fmean(data)` | ✅ | |
| `geometric_mean(data)` | `geometric_mean(data)` | ✅ | |
| `harmonic_mean(data)` | `harmonic_mean(data)` | ✅ | |
| `median(data)` | `median(data)` | ✅ | |
| `median_low(data)` | `median_low(data)` | ✅ | |
| `median_high(data)` | `median_high(data)` | ✅ | |
| `median_grouped(data)` | — | ❌ | Grouped median |
| `mode(data)` | `mode(data)` | ✅ | int only |
| `multimode(data)` | — | ❌ | All modes |
| `quantiles(data, n)` | — | ❌ | Quantile boundaries |
| `variance(data)` | `variance(data)` | ✅ | |
| `pvariance(data)` | `pvariance(data)` | ✅ | |
| `stdev(data)` | `stdev(data)` | ✅ | |
| `pstdev(data)` | `pstdev(data)` | ✅ | |
| `covariance(x, y)` | — | ❌ | Covariance |
| `correlation(x, y)` | — | ❌ | Pearson correlation |
| `linear_regression(x, y)` | — | ❌ | Linear regression |
| `NormalDist` class | — | ❌ | Normal distribution class |
| `StatisticsError` | — | ❌ | Error type |

---

### 16. `sifr.bisect` vs `bisect` — Coverage: ~80% (4/6 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `bisect_left(a, x)` | `bisect_left(a, x)` | ✅ | Generic |
| `bisect_right(a, x)` | `bisect_right(a, x)` | ✅ | Generic |
| `insort_left(a, x)` | `insort_left(a, x)` | ✅ | Generic |
| `insort_right(a, x)` | `insort_right(a, x)` | ✅ | Generic |
| `bisect` (alias) | — | ❌ | Alias for `bisect_right` |
| `insort` (alias) | — | ❌ | Alias for `insort_right` |

---

### 17. `sifr.functools` vs `functools` — Coverage: ~0% (0/12 CPython functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `reduce(function, iterable)` | — | ❌ | **High priority** — very commonly used |
| `partial(func, *args)` | — | ❌ | Partial application |
| `lru_cache(maxsize)` | — | ❌ | Memoization decorator |
| `cache` | — | ❌ | Unbounded cache |
| `cached_property` | — | ❌ | Cached property descriptor |
| `total_ordering` | — | ❌ | Class decorator for comparison methods |
| `cmp_to_key(func)` | — | ❌ | Comparison function to key function |
| `singledispatch` | — | ❌ | Single-dispatch generic function |
| `wraps(wrapped)` | — | ❌ | Decorator for wrapper functions |
| `update_wrapper(wrapper, wrapped)` | — | ❌ | Update wrapper function |
| `partialmethod` | — | ❌ | Partial for methods |
| `singledispatchmethod` | — | ❌ | Single-dispatch for methods |
| — | `identity(x)` | ⚠️ | **Not in CPython** |
| — | `clamp(value, min, max)` | ⚠️ | **Not in CPython** |

**Critical finding:** Sifr's `functools` has 0% CPython parity. `identity` and `clamp` are useful but are not CPython functions. `reduce` is the most commonly used functools function and is completely missing.

---

### 18. `sifr.secrets` vs `secrets` — Coverage: ~30% (2/7 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `token_hex(nbytes)` | `token_hex(nbytes)` | ✅ | |
| `randbelow(n)` | `randbelow(n)` | ✅ | |
| `token_bytes(nbytes)` | — | ❌ | Random bytes |
| `token_urlsafe(nbytes)` | — | ❌ | URL-safe random token |
| `choice(sequence)` | — | ❌ | Secure random choice |
| `randbits(k)` | — | ❌ | Random k-bit integer |
| `compare_digest(a, b)` | — | ❌ | Constant-time comparison |

---

### 19. `sifr.heapq` vs `heapq` — Coverage: ~70% (8/11 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `heappush(heap, item)` | `heappush(heap, item)` | ✅ | |
| `heappop(heap)` | `heappop(heap)` | ✅ | |
| `heapify(x)` | `heapify(data)` | ✅ | |
| `heapreplace(heap, item)` | `heapreplace(heap, item)` | ✅ | |
| `heappushpop(heap, item)` | `heappushpop(heap, item)` | ✅ | |
| `nlargest(n, iterable)` | `nlargest(n, data)` | ✅ | |
| `nsmallest(n, iterable)` | `nsmallest(n, data)` | ✅ | |
| `merge(*iterables)` | — | ❌ | Merge sorted iterables |
| `heappush_max(heap, item)` | — | ❌ | Max-heap push |
| `heappop_max(heap)` | — | ❌ | Max-heap pop |
| `heapify_max(x)` | — | ❌ | Max-heap heapify |

**Note:** Sifr also has `heappop_rest(heap)` which is not in CPython.

---

### 20. `sifr.itertools` vs `itertools` — Coverage: ~19% (4/21 iterator types)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `chain(*iterables)` | `chain(a, b)` | ⚠️ | Only 2 lists, not variadic |
| `islice(iterable, stop)` | `islice(data, stop)` | ⚠️ | Missing start/step params |
| `pairwise(iterable)` | `pairwise(data)` | ✅ | |
| `batched(iterable, n)` | `batched(data, n)` | ✅ | |
| `repeat(object, times)` | `repeat_val(value, times)` | ⚠️ | Different name, int only |
| `accumulate(iterable, func)` | — | ❌ | Running totals |
| `chain.from_iterable(it)` | — | ❌ | Chain from single iterable |
| `combinations(iterable, r)` | — | ❌ | r-length combinations |
| `combinations_with_replacement(it, r)` | — | ❌ | |
| `compress(data, selectors)` | — | ❌ | Filter by selectors |
| `count(start, step)` | — | ❌ | Infinite counter |
| `cycle(iterable)` | — | ❌ | Infinite cycling |
| `dropwhile(predicate, it)` | — | ❌ | Drop while predicate true |
| `filterfalse(predicate, it)` | — | ❌ | Filter where predicate false |
| `groupby(iterable, key)` | — | ❌ | Group consecutive elements |
| `permutations(iterable, r)` | — | ❌ | r-length permutations |
| `product(*iterables)` | — | ❌ | Cartesian product |
| `starmap(function, iterable)` | — | ❌ | Map with unpacked args |
| `takewhile(predicate, it)` | — | ❌ | Take while predicate true |
| `tee(iterable, n)` | — | ❌ | Create n independent iterators |
| `zip_longest(*iterables)` | — | ❌ | Zip with fill value |

**Also in Sifr but not in CPython:**
- `chain_str(a, b)` — string-specific chain
- `take(n, data)` — take first n elements
- `flatten(lists)` — flatten nested lists
- `enumerate_list(data)` — enumerate to list

---

### 21. `sifr.textwrap` vs `textwrap` — Coverage: ~70% (5/6 functions)

| CPython | Sifr | Status |
|---------|------|--------|
| `wrap(text, width)` | `wrap(text, width)` | ✅ |
| `fill(text, width)` | `fill(text, width)` | ✅ |
| `dedent(text)` | `dedent(text)` | ✅ |
| `indent(text, prefix)` | `indent(text, prefix)` | ✅ |
| `shorten(text, width)` | `shorten(text, width)` | ✅ |
| `TextWrapper` class | — | ❌ |

---

### 22. `sifr.csv` vs `csv` — Coverage: ~15% (4 functions vs full module)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `reader(csvfile)` | `parse_csv(text)` | ⚠️ | Takes string, not file; no quoted field support |
| `writer(csvfile)` | `format_csv(rows)` | ⚠️ | Returns string, not file writer |
| `parse_row` | — | ⚠️ | Not in CPython (Sifr-specific) |
| `format_row` | — | ⚠️ | Not in CPython (Sifr-specific) |
| `DictReader` | — | ❌ | Dict-based reader |
| `DictWriter` | — | ❌ | Dict-based writer |
| `Dialect` | — | ❌ | Dialect configuration |
| `Sniffer` | — | ❌ | Dialect detection |
| `QUOTE_*` constants | — | ❌ | Quoting modes |
| Quoted field handling | — | ❌ | Fields with commas/quotes |
| Custom delimiters | — | ❌ | Tab, pipe, etc. |

---

### 23. `sifr.argparse` vs `argparse` — Coverage: ~5% (3 helper functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `ArgumentParser` class | — | ❌ | **Entire class missing** |
| `parse_args()` | — | ❌ | |
| `add_argument(...)` | — | ❌ | |
| `add_subparsers()` | — | ❌ | |
| `Namespace` | — | ❌ | |
| `HelpFormatter` | — | ❌ | |
| `FileType` | — | ❌ | |
| `BooleanOptionalAction` | — | ❌ | |
| — | `parse_flag(args, flag)` | ⚠️ | Sifr-specific helper |
| — | `parse_option(args, name, default)` | ⚠️ | Sifr-specific helper |
| — | `parse_positional(args, index, default)` | ⚠️ | Sifr-specific helper |

**Design note:** Sifr's argparse is fundamentally different from CPython's. CPython has a rich `ArgumentParser` class with declarative argument definitions. Sifr has 3 simple helper functions for manual parsing. This is the largest API gap of any existing module.

---

### 24. `sifr.fnmatch` vs `fnmatch` — Coverage: ~60% (3/5 functions)

| CPython | Sifr | Status |
|---------|------|--------|
| `fnmatch(name, pattern)` | `fnmatch(name, pattern)` | ✅ |
| `fnmatchcase(name, pattern)` | `fnmatchcase(name, pattern)` | ✅ |
| `filter(names, pattern)` | `filter(names, pattern)` | ✅ |
| `filterfalse(names, pattern)` | — | ❌ |
| `translate(pattern)` | — | ❌ |

---

### 25. `sifr.glob` vs `glob` — Coverage: ~25% (1/4 functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `glob(pathname)` | `glob(directory, pattern)` | ⚠️ | Different signature — takes dir + pattern separately |
| `iglob(pathname)` | — | ❌ | Lazy iterator version |
| `escape(pathname)` | — | ❌ | Escape special characters |
| `translate(pathname)` | — | ❌ | Convert to regex |

---

### 26. `sifr.shutil` vs `shutil` — Coverage: ~15% (3/15+ functions)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `copy(src, dst)` | `copy(src, dst)` | ✅ | |
| `move(src, dst)` | `move_file(src, dst)` | ⚠️ | Different name |
| `rmtree(path)` | `rmtree(path)` | ✅ | |
| `copy2(src, dst)` | — | ❌ | Copy with metadata |
| `copytree(src, dst)` | — | ❌ | Copy directory tree |
| `copyfile(src, dst)` | — | ❌ | Copy file contents only |
| `copymode(src, dst)` | — | ❌ | Copy permissions |
| `copystat(src, dst)` | — | ❌ | Copy metadata |
| `which(name)` | — | ❌ | Find executable |
| `disk_usage(path)` | — | ❌ | Disk usage stats |
| `get_terminal_size()` | — | ❌ | Terminal dimensions |
| `make_archive(...)` | — | ❌ | Create archive |
| `unpack_archive(...)` | — | ❌ | Extract archive |
| `chown(path, user, group)` | — | ❌ | Change ownership |
| `ignore_patterns(...)` | — | ❌ | Pattern-based ignore |

---

### 27. `sifr.tempfile` vs `tempfile` — Coverage: ~30% (3/7 functions, 0/4 classes)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `mkstemp(prefix)` | `mkstemp(prefix)` | ⚠️ | Returns path string, not (fd, name) tuple |
| `mkdtemp(prefix)` | `mkdtemp(prefix)` | ✅ | |
| `gettempdir()` | `gettempdir()` | ✅ | Via intrinsic |
| `mktemp(prefix)` | `mktemp_path(prefix)` | ⚠️ | Different name |
| `gettempprefix()` | — | ❌ | |
| `NamedTemporaryFile` | — | ❌ | Context manager class |
| `TemporaryFile` | — | ❌ | Context manager class |
| `SpooledTemporaryFile` | — | ❌ | In-memory temp file |
| `TemporaryDirectory` | — | ❌ | Context manager class |

---

## Tier 3: Parity Modules

### 28. `sifr.graphlib` vs `graphlib` — Coverage: ~40% (1 class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `TopologicalSorter` | `TopologicalSorter` | ⚠️ | Has `add`/`static_order`; missing `prepare`, `is_active`, `done`, `get_ready` |
| `CycleError` | — | ❌ | Exception for cycles |

---

### 29. `sifr.uuid` vs `uuid` — Coverage: ~20% (1/7 functions, partial class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `uuid4()` | `uuid4_obj()` | ⚠️ | Different name; returns UUID object |
| `uuid1()` | — | ❌ | Time-based UUID |
| `uuid3(namespace, name)` | — | ❌ | MD5-based UUID |
| `uuid5(namespace, name)` | — | ❌ | SHA1-based UUID |
| `uuid6()` | — | ❌ | Sortable time-based |
| `uuid7()` | — | ❌ | Unix time-based |
| `uuid8(...)` | — | ❌ | Custom UUID |
| `UUID` class | `UUID` class | ⚠️ | Has `hex`, `urn`, `to_str`, `version`; missing `int`, `bytes`, `fields`, `node`, `time`, `clock_seq`, `variant` |
| `NAMESPACE_DNS` | — | ❌ | |
| `NAMESPACE_URL` | — | ❌ | |
| `SafeUUID` | — | ❌ | |

---

### 30. `sifr.platform` vs `platform` — Coverage: ~15% (2/15+ functions)

| CPython | Sifr | Status |
|---------|------|--------|
| `system()` | `system()` | ✅ |
| `machine()` | `machine()` | ✅ |
| `node()` | — | ❌ (intrinsic exists: `platform_node`) |
| `release()` | — | ❌ |
| `version()` | — | ❌ |
| `processor()` | — | ❌ |
| `platform()` | — | ❌ |
| `uname()` | — | ❌ |
| `python_version()` | — | ❌ (would be `sifr_version()`) |
| `python_implementation()` | — | ❌ |

---

### 31. `sifr.pathlib` vs `pathlib` — Coverage: ~30% (13 methods + 6 functions)

See detailed comparison in the `sifr.os` section above. Key missing Path methods:
- `resolve()`, `absolute()`, `glob(pattern)`, `rglob(pattern)`, `iterdir()`
- `stat()`, `chmod(mode)`, `rename(target)`, `unlink()`, `rmdir()`, `touch()`
- `open(mode)`, `with_name(name)`, `with_suffix(suffix)`, `match(pattern)`
- `relative_to(other)`, `parts`, `root`, `anchor`, `parents`
- `Path.home()`, `Path.cwd()`, `/` operator

---

### 32. `sifr.logging` vs `logging` — Coverage: ~15% (5 functions + Logger class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `getLogger(name)` | `getLogger(name)` | ✅ | |
| `Logger.debug(msg)` | `Logger.debug(msg)` | ✅ | |
| `Logger.info(msg)` | `Logger.info(msg)` | ✅ | |
| `Logger.warning(msg)` | `Logger.warning(msg)` | ✅ | |
| `Logger.error(msg)` | `Logger.error(msg)` | ✅ | |
| `Logger.critical(msg)` | `Logger.critical(msg)` | ✅ | |
| `Logger.setLevel(level)` | `Logger.set_level(level)` | ⚠️ | Different name |
| `basicConfig(...)` | — | ❌ | Configure root logger |
| `FileHandler` | — | ❌ | Log to file |
| `StreamHandler` | — | ❌ | Log to stream |
| `Formatter` | — | ❌ | Log format customization |
| `Filter` | — | ❌ | Log filtering |
| `Handler` | — | ❌ | Base handler class |
| `LogRecord` | — | ❌ | Log record class |
| `DEBUG/INFO/WARNING/ERROR/CRITICAL` | — | ❌ | Level constants |
| `addLevelName(level, name)` | — | ❌ | |
| `disable(level)` | — | ❌ | |
| Module-level `debug/info/warning/error` | `log_debug/log_info/log_warn/log_error` | ⚠️ | Different names |

---

### 33. `sifr.difflib` vs `difflib` — Coverage: ~20% (2/6 functions, 0/3 classes)

| CPython | Sifr | Status |
|---------|------|--------|
| `get_close_matches(word, possibilities)` | `get_close_matches(word, possibilities, n, cutoff)` | ✅ |
| `unified_diff(a, b)` | `unified_diff(a, b)` | ✅ |
| `context_diff(a, b)` | — | ❌ |
| `ndiff(a, b)` | — | ❌ |
| `diff_bytes(...)` | — | ❌ |
| `SequenceMatcher` class | — | ❌ |
| `Differ` class | — | ❌ |
| `HtmlDiff` class | — | ❌ |

---

### 34. `sifr.ipaddress` vs `ipaddress` — Coverage: ~25% (7 functions, 0/6 classes)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `ip_address(addr)` | — | ❌ | Factory function |
| `ip_network(addr)` | — | ❌ | Network factory |
| `ip_interface(addr)` | — | ❌ | Interface factory |
| `IPv4Address` class | — | ❌ | Full address class |
| `IPv6Address` class | — | ❌ | Full address class |
| `IPv4Network` class | — | ❌ | Network class |
| `IPv6Network` class | — | ❌ | Network class |
| — | `is_valid_ipv4(addr)` | ⚠️ | Sifr-specific |
| — | `ip_to_int(addr)` | ⚠️ | Sifr-specific |
| — | `int_to_ip(val)` | ⚠️ | Sifr-specific |
| — | `is_private(addr)` | ⚠️ | Method on IPv4Address in CPython |
| — | `is_loopback(addr)` | ⚠️ | Method on IPv4Address in CPython |
| — | `is_multicast(addr)` | ⚠️ | Method on IPv4Address in CPython |
| — | `is_global(addr)` | ⚠️ | Method on IPv4Address in CPython |

**Design note:** CPython's ipaddress uses rich class hierarchy (IPv4Address, IPv6Address, etc.). Sifr has standalone functions that only handle IPv4 strings. The API design is fundamentally different.

---

### 35. `sifr.timeit` vs `timeit` — Coverage: ~60% (3/3 functions, 0/1 class)

| CPython | Sifr | Status |
|---------|------|--------|
| `timeit(stmt, number)` | `timeit(stmt, number)` | ✅ |
| `repeat(stmt, repeat, number)` | `repeat(stmt, count, number)` | ✅ |
| `default_timer()` | `default_timer()` | ✅ |
| `Timer` class | — | ❌ |

---

### 36. `sifr.tomllib` vs `tomllib` — Coverage: ~65% (2/2 functions, 0/1 class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `loads(s)` | `loads(text)` | ✅ | Returns str (serialized), not dict |
| `load(fp)` | `load(path)` | ⚠️ | Takes path string, not file object |
| `TOMLDecodeError` | — | ❌ | Error type |

---

### 37. `sifr.datetime` vs `datetime` — Coverage: ~15% (3 functions + timedelta class)

| CPython | Sifr | Status | Notes |
|---------|------|--------|-------|
| `datetime` class | — | ❌ | Full datetime class (Sifr has functions, not a class) |
| `date` class | — | ❌ | Date-only class |
| `time` class | — | ❌ | Time-only class |
| `timedelta` class | `timedelta` class | ⚠️ | Has `total_seconds`, `days`, `seconds`, `__add__`, `__sub__`, `__eq__`; missing many methods |
| `timezone` class | — | ❌ | Timezone class |
| `tzinfo` class | — | ❌ | Abstract timezone base |
| `datetime.now()` | `now()` | ⚠️ | Returns str, not datetime object |
| `datetime.strftime(fmt)` | `format_datetime(dt, fmt)` | ⚠️ | Function, not method |
| `datetime.fromtimestamp(ts)` | `from_timestamp(ts)` | ⚠️ | Returns str, not datetime object |
| `datetime.strptime(s, fmt)` | — | ❌ | Parse datetime string |
| `datetime.fromisoformat(s)` | — | ❌ | Parse ISO format |
| `datetime.utcnow()` | — | ❌ | UTC now |
| `datetime.combine(date, time)` | — | ❌ | Combine date and time |
| `datetime.replace(...)` | — | ❌ | Replace fields |
| `datetime.date()` | — | ❌ | Extract date |
| `datetime.time()` | — | ❌ | Extract time |
| `datetime.timestamp()` | — | ❌ | Convert to timestamp |
| `datetime.isoformat()` | — | ❌ | ISO format string |
| `datetime.weekday()` | — | ❌ | Day of week |
| `timedelta` arithmetic | `__add__`, `__sub__` | ⚠️ | Missing `__mul__`, `__truediv__`, `__floordiv__`, `__mod__`, `__neg__`, `__abs__` |
| `MINYEAR` / `MAXYEAR` | — | ❌ | Constants |
| `UTC` | — | ❌ | UTC timezone constant |

---

## Summary Scorecard

| Module | Sifr Coverage | CPython Functions | Sifr Functions | Gap |
|--------|--------------|-------------------|----------------|-----|
| `math` | ~60% | 58 | 35 | 23 |
| `os` | ~20% | 80+ | 16 | 64+ |
| `io` | ~15% | 15+ classes | 5 | 10+ |
| `re` | ~40% | 15+ | 5 + Match | 10+ |
| `json` | ~30% | 4 + 3 classes | 2 | 5+ |
| `time` | ~35% | 14+ | 5 | 9+ |
| `hashlib` | ~30% | 14+ | 4 | 10+ |
| `base64` | ~30% | 16 | 4 | 12 |
| `random` | ~12.5% | 24+ | 3 | 21+ |
| `bytes` | ~15% | 40+ methods | 4 | 36+ |
| `collections` | ~25% | 9 classes | 2 | 7 |
| `env` | ~50% | 7+ | 2 | 5+ |
| `string` | ~65% | 12 | 10 | 2 |
| `statistics` | ~60% | 20 | 12 | 8 |
| `bisect` | ~80% | 6 | 4 | 2 |
| `functools` | **0%** | 12 | 0 (2 non-CPython) | **12** |
| `secrets` | ~30% | 7 | 2 | 5 |
| `heapq` | ~70% | 11 | 8 | 3 |
| `itertools` | ~19% | 21 | 4 | 17 |
| `textwrap` | ~70% | 6 | 5 | 1 |
| `csv` | ~15% | 10+ classes | 4 | 6+ |
| `argparse` | **~5%** | 10+ classes | 0 (3 non-CPython) | **10+** |
| `fnmatch` | ~60% | 5 | 3 | 2 |
| `glob` | ~25% | 4 | 1 | 3 |
| `shutil` | ~15% | 15+ | 3 | 12+ |
| `tempfile` | ~30% | 7 + 4 classes | 3 | 8+ |
| `graphlib` | ~40% | 1 class + 1 exc | 1 class | 1 |
| `uuid` | ~20% | 7 + 1 class | 1 + partial class | 7+ |
| `platform` | ~15% | 15+ | 2 | 13+ |
| `pathlib` | ~30% | 40+ methods | 13 + 6 | 27+ |
| `logging` | ~15% | 20+ | 5 + Logger | 15+ |
| `difflib` | ~20% | 6 + 3 classes | 2 | 7+ |
| `ipaddress` | ~25% | 3 + 6 classes | 7 (non-CPython API) | 9+ |
| `timeit` | ~60% | 3 + 1 class | 3 | 1 |
| `tomllib` | ~65% | 2 + 1 class | 2 | 1 |
| `datetime` | ~15% | 6 classes | 1 class + 3 funcs | 5+ |

**Weighted average coverage: ~35%**
