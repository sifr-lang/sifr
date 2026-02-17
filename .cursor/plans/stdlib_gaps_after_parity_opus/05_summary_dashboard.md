# Summary Dashboard

At-a-glance metrics for Sifr stdlib vs CPython stdlib.

---

## Overall Coverage

| Metric | Value |
|--------|-------|
| Total Sifr stdlib modules | 37 |
| Total CPython stdlib modules | ~289 |
| Module-level coverage | 37/289 = **12.8%** |
| Applicable CPython modules (excluding Python-specific, deprecated) | ~180 |
| Module-level coverage (applicable only) | 37/180 = **20.6%** |
| Average function-level coverage (existing modules) | **~35%** |
| Modules with >70% coverage | 4 (`bisect`, `textwrap`, `heapq`, `timeit`) |
| Modules with <20% coverage | 8 (`io`, `argparse`, `csv`, `shutil`, `datetime`, `platform`, `logging`, `pathlib`) |
| Modules with 0% CPython parity | 1 (`functools` — has non-CPython functions only) |

---

## Module Coverage Heatmap

### Green Zone (>60% coverage) — 8 modules
```
bisect       ████████░░  80%
textwrap     ███████░░░  70%
heapq        ███████░░░  70%
string       ██████▌░░░  65%
tomllib      ██████▌░░░  65%
statistics   ██████░░░░  60%
math         ██████░░░░  60%
fnmatch      ██████░░░░  60%
timeit       ██████░░░░  60%
```

### Yellow Zone (30-59% coverage) — 12 modules
```
env          █████░░░░░  50%
re           ████░░░░░░  40%
graphlib     ████░░░░░░  40%
time         ███▌░░░░░░  35%
json         ███░░░░░░░  30%
hashlib      ███░░░░░░░  30%
base64       ███░░░░░░░  30%
bytes        ███░░░░░░░  30%  (non-standard API)
secrets      ███░░░░░░░  30%
pathlib      ███░░░░░░░  30%
tempfile     ███░░░░░░░  30%
collections  ██▌░░░░░░░  25%
```

### Red Zone (<30% coverage) — 17 modules
```
glob         ██▌░░░░░░░  25%
ipaddress    ██▌░░░░░░░  25%  (non-standard API)
os           ██░░░░░░░░  20%
uuid         ██░░░░░░░░  20%
difflib      ██░░░░░░░░  20%
itertools    ██░░░░░░░░  19%
shutil       █▌░░░░░░░░  15%
io           █▌░░░░░░░░  15%
platform     █▌░░░░░░░░  15%
logging      █▌░░░░░░░░  15%
datetime     █▌░░░░░░░░  15%
csv          █▌░░░░░░░░  15%
bytes        █▌░░░░░░░░  15%
random       █▌░░░░░░░░  12.5%
argparse     ▌░░░░░░░░░  5%
functools    ░░░░░░░░░░  0%  (CPython parity)
```

---

## Top 10 Most Impactful Gaps

Ranked by how much they would improve real-world Sifr usability:

| Rank | Gap | Impact | Effort |
|------|-----|--------|--------|
| 1 | **`functools.reduce`** missing | Blocks functional programming patterns | Small |
| 2 | **`random.choice/shuffle/sample`** missing | Blocks common random operations | Small |
| 3 | **`itertools` missing 17 iterator types** | Blocks iterator-heavy code | Medium |
| 4 | **`subprocess` module missing entirely** | Can't spawn processes | Medium |
| 5 | **`datetime` returns strings, not objects** | Can't do date arithmetic properly | Large |
| 6 | **`json.loads` returns string, not dict** | Can't work with JSON data structurally | Large (needs typed dict) |
| 7 | **`open()` built-in missing** | Can't do streaming file I/O | Medium |
| 8 | **`collections.deque` missing** | Missing fundamental data structure | Medium |
| 9 | **`argparse.ArgumentParser` missing** | Can't build proper CLI tools | Large |
| 10 | **`sys` module missing** | Can't access `sys.argv`, `sys.exit`, `sys.platform` | Small |

---

## Modules by Sifr-Specific Design Impact

### Modules where Sifr's design makes porting straightforward
- `math` — pure functions, maps directly
- `statistics` — pure functions, maps directly
- `string` — constants + pure functions
- `bisect` — pure algorithms
- `heapq` — pure algorithms
- `textwrap` — pure string manipulation
- `fnmatch` — pure string matching
- `difflib` — pure algorithms
- `graphlib` — pure algorithms
- `itertools` — generators map to Rust iterators

### Modules where Sifr's design requires significant adaptation
- `json` — needs typed return values (dict/list), not strings
- `datetime` — needs full class hierarchy, not string wrappers
- `io` — needs file object protocol, streaming, `with` support
- `csv` — needs file object protocol
- `argparse` — needs rich class with builder pattern
- `collections` — needs generic containers
- `re` — needs Pattern class, flags enum
- `logging` — needs handler/formatter class hierarchy
- `pickle` — fundamentally incompatible (runtime type inspection)

### Modules that Sifr's design replaces entirely
- `typing` → built-in type system
- `dataclasses` → auto-derived traits
- `enum` → union types + literal types
- `abc` → protocols
- `copy` → `.clone()`
- `pprint` → auto-derived `Debug`
- `gc` → ownership/RAII
- `weakref` → ownership model

---

## Missing Module Priority Matrix

```
                    HIGH IMPACT
                        │
    subprocess ─────────┼──────────── asyncio
    sys                 │              http
    socket              │              urllib
    threading           │              sqlite3
                        │
  LOW EFFORT ───────────┼─────────── HIGH EFFORT
                        │
    html                │              xml
    operator            │              configparser
    calendar            │              zipfile
    decimal             │              email
                        │
                    LOW IMPACT
```

---

## Function Count Summary

| Category | Count |
|----------|-------|
| Functions Sifr has | ~200 |
| Functions CPython has (in Sifr's 37 modules) | ~550 |
| Function-level gap (existing modules) | ~350 |
| Functions in HIGH-priority missing modules | ~300+ |
| **Total function gap** | **~650+** |

---

## Recommendations

### Immediate (can do now, high ROI)
1. Add `reduce` to `functools` (and remove `identity`/`clamp`)
2. Expose `choice` in `sifr.random` (intrinsic already exists)
3. Add `shuffle`, `sample` to `random`
4. Add `accumulate`, `dropwhile`, `takewhile`, `filterfalse`, `zip_longest` to `itertools`
5. Add `acosh`, `asinh`, `atanh`, `cbrt`, `isqrt` to `math`
6. Add `bisect`/`insort` aliases to `bisect`
7. Add `quantiles`, `multimode`, `covariance`, `correlation` to `statistics`
8. Create `sifr.sys` module with `argv`, `exit`, `platform`

### Short-term (next phase)
1. Create `sifr.subprocess` module
2. Full `datetime` class (not string-based)
3. `collections.deque` class
4. `re.compile` → Pattern class with flags
5. Proper `open()` function with file objects
6. `argparse.ArgumentParser` class

### Medium-term (requires language features)
1. Typed JSON parsing (needs generic dict/union return types)
2. Streaming I/O (needs file object protocol)
3. Compression modules (zipfile, gzip, tarfile)
4. XML processing
5. Database access (sqlite3)

### Long-term (requires async/threading)
1. `asyncio` module
2. `socket` module
3. `http` client/server
4. `threading` module
5. `concurrent.futures`
