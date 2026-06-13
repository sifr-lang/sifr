## milestone_stdlib_parity: Gap Closing and Parity Audit

---

### 1. Product Requirements

#### **Title**

milestone_stdlib_parity: Expand Existing Modules, Add Remaining Modules, Run Parity Audit

---

#### **Objective / Problem Statement**

With 18 stdlib modules now available (13 original + 5 new from milestone_stdlib_expansion), this milestone closes remaining gaps by expanding existing modules with missing functions, adding ~10 new modules, and running a parity audit against CPython's top modules. The goal is to reach 37+ total stdlib modules with 60%+ coverage of the top 20 CPython modules.

---

#### **Scope**

##### Part A: Expand Existing Modules

1. `math.sifr` -- add trig inverses (asin, acos, atan, atan2), hyperbolic (sinh, cosh, tanh), constants (tau, inf), and utility (isnan, isinf, degrees, radians, factorial, gcd, lcm, copysign, fmod, trunc, isclose)
2. `os.sifr` -- add getcwd, listdir, mkdir, rmdir, remove, rename, path_exists, path_isfile, path_isdir (wraps new `_sifr.fs` intrinsics)
3. `re.sifr` -- add findall, split (wraps `_sifr.regex`)
4. `random.sifr` -- add shuffle, sample, seed, uniform, choice (wraps `_sifr.crypto`)
5. `io.sifr` -- add append_text, read_bytes, write_bytes (wraps `_sifr.fs`)
6. `collections.sifr` -- add Counter, defaultdict-like patterns
7. `string.sifr` -- add capwords, Template-like formatting
8. `statistics.sifr` -- add mode, harmonic_mean, geometric_mean
9. `bisect.sifr` -- add insort_left, insort_right

##### Part B: New Modules

###### Pure Sifr (no new intrinsics)
1. `difflib.sifr` -- sequence matching (SequenceMatcher-like)
2. `graphlib.sifr` -- topological sort
3. `ipaddress.sifr` -- IPv4/IPv6 address parsing and validation

###### Intrinsic-backed
4. `timeit.sifr` -- wraps `_sifr.time` for benchmarking
5. `platform.sifr` -- wraps `_sifr.sys` for OS/platform info
6. `pathlib.sifr` -- wraps `_sifr.fs` for path manipulation
7. `uuid.sifr` -- wraps `_sifr.crypto` for UUID generation
8. `logging.sifr` -- wraps `_sifr.io` + `_sifr.time` for structured logging
9. `datetime.sifr` -- wraps new `_sifr.datetime` for date/time types
10. `tomllib.sifr` -- wraps new `_sifr.toml` for TOML parsing

##### Part C: Parity Audit
- Run comprehensive audit against CPython's top 20 modules
- Produce `audits/STDLIB_PARITY_MASTER_REPORT.md`
- Target: 60%+ function coverage across top 20 modules

### **Acceptance Criteria**

| **AC-ID** | Criterion |
| --- | --- |
| AC-1 | All expanded existing modules compile with new functions and have E2E tests |
| AC-2 | All new modules compile, import correctly, and have E2E tests |
| AC-3 | 37+ total stdlib modules available |
| AC-4 | Parity audit report generated showing 60%+ coverage |
| AC-5 | All existing tests continue to pass |
| AC-6 | Demo `demos/milestone_stdlib_parity_demo.sifr` works |

---

### 2. Solution Design

#### **2.1 Functional Requirements**

* Expand existing `.sifr` modules with additional functions/constants
* Add new `_sifr.*` intrinsic modules where needed (e.g., `_sifr.datetime`, `_sifr.toml`)
* Add corresponding `emit_intrinsic_call` entries in codegen for new intrinsics
* Create new `.sifr` wrapper modules for each new stdlib module
* Register all new modules in `STDLIB_FILES` in driver
* Run parity audit comparing Sifr stdlib against CPython

#### **2.2 Non-Functional Requirements**

| ID | Requirement |
| --- | --- |
| NFR-1 | No regression in existing test suite |
| NFR-2 | Compilation time increase < 500ms for stdlib compilation |
| NFR-3 | Each new module must have at least one E2E test |

#### **2.3 PR Structure**

- PR 1: Part A (expand existing modules + new intrinsics needed)
- PR 2: Part B pure Sifr modules (difflib, graphlib, ipaddress)
- PR 3: Part B intrinsic-backed modules (timeit, platform, pathlib, uuid, logging, datetime, tomllib)
- PR 4: Part C parity audit report

#### **2.4 Testing Strategy**

| **AC-ID** | Test Layer | Check |
| --- | --- | --- |
| AC-1 | E2E pass tests | Each expanded function has a test case |
| AC-2 | E2E pass tests | Each new module has a dedicated test file |
| AC-3 | Count check | Verify 37+ modules in STDLIB_FILES |
| AC-4 | Audit script | Generate and verify parity report |
| AC-5 | Full test suite | `cargo test` passes |
| AC-6 | Demo run | `cargo run -- run demos/milestone_stdlib_parity_demo.sifr` succeeds |
