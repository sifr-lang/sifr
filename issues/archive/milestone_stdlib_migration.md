## milestone_stdlib_migration: Migrate 13 Modules to .sifr

---

### 1. Product Requirements

#### **Title**

milestone_stdlib_migration: Migrate All 13 Existing Stdlib Modules to .sifr Files

---

#### **Objective / Problem Statement**

With the intrinsics layer and two-phase compilation pipeline established in milestone_intrinsics, the 13 existing stdlib modules still resolve via the intrinsic fallback path (`get_stdlib_as_intrinsic`). This milestone migrates all 13 modules to `.sifr` files that import from `_sifr.*` intrinsics, making them thin wrappers. After migration, the fallback path and `emit_intrinsic_call` codegen (~352 lines) can be deleted.

---

#### **Scope**

##### Features In

1. Create `.sifr` files for all 13 existing stdlib modules in `lib/sifr/`
2. Each `.sifr` file imports from `_sifr.*` and re-exports user-facing functions
3. Delete the `get_stdlib_as_intrinsic` fallback path
4. Delete `emit_intrinsic_call` (~352 lines) from codegen
5. Rename `sifr.hash` to `sifr.hashlib`, `sifr.encoding` to `sifr.base64` in tests

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| New stdlib modules | Deferred to milestone_stdlib_expansion |
| Expanding existing module APIs | Deferred to milestone_stdlib_parity |

---

### **Acceptance Criteria**

| **AC-ID** | Criterion |
| --- | --- |
| AC-1 | Every `from sifr.X import Y` resolves to a `.sifr` file (no intrinsic fallback) |
| AC-2 | `emit_intrinsic_call` deleted from codegen |
| AC-3 | All E2E tests pass with zero regressions |
| AC-4 | Demo: `demos/milestone_stdlib_migration_demo.sifr` works correctly |

---

## 2. Solution Design

### Migration Order (simplest to most complex)

1. `env.sifr` (2 functions, wraps `_sifr.sys`)
2. `bytes.sifr` (4 functions, wraps `_sifr.bytes`)
3. `base64.sifr` (2 functions, rename from `encoding`, wraps `_sifr.crypto`)
4. `math.sifr` (12 functions + 2 constants, wraps `_sifr.math`)
5. `hashlib.sifr` (2 functions, rename from `hash`, wraps `_sifr.crypto`)
6. `io.sifr` (4 functions, wraps `_sifr.io`)
7. `os.sifr` (2 functions, wraps `_sifr.sys`)
8. `json.sifr` (2 functions, wraps `_sifr.json`)
9. `time.sifr` (3 functions, wraps `_sifr.time`)
10. `random.sifr` (3 functions, wraps `_sifr.crypto`)
11. `re.sifr` (3 functions, wraps `_sifr.regex`)
12. `collections.sifr` (14 functions, wraps `_sifr.collections`)
13. `test.sifr` (already done in M1, verify)

### Technical Approach

Each `.sifr` file is a thin wrapper that imports from the corresponding `_sifr.*` intrinsic module. The stdlib `.sifr` files are allowed to import from `_sifr.*` (the import blocking only applies to user code).

### Final Cleanup

- Delete `emit_intrinsic_call` (~352 lines) from codegen
- Delete `get_stdlib_as_intrinsic` fallback from stdlib.rs
- Delete old intrinsic function definitions that are no longer needed
- Update Cargo dep injection to use `_sifr.*` module names only
