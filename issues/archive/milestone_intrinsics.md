## milestone_intrinsics: Intrinsics Layer and Stdlib Compilation Pipeline

---

### 1. Product Requirements

#### **Title**

milestone_intrinsics: Intrinsics Layer and Stdlib Compilation Pipeline

---

#### **Objective / Problem Statement**

Sifr's stdlib is currently implemented entirely as Rust codegen -- 13 modules and 57 functions are hardcoded in `stdlib.rs` (type registry) and `emit_stdlib_call` in `lib.rs` (~352 lines of Rust code generation). This approach doesn't scale: adding new stdlib functions requires modifying the compiler itself, users can't read stdlib source code, and the stdlib can't be written in Sifr.

This milestone rewires the stdlib plumbing to support a three-tier hybrid architecture:
- **Tier 1: Rust Intrinsics (`_sifr.*`)** -- compiler-provided primitives for OS access, unsafe code, crate bindings
- **Tier 2: Sifr Stdlib (`sifr.*`)** -- `.sifr` files that import from `_sifr.*` and provide user-facing APIs
- **Tier 3: User Code** -- imports from `sifr.*`

No new user-facing features are added, but this establishes the architecture that all subsequent stdlib milestones build on.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| Zero regressions on existing 173 E2E pass tests and 17 fail tests | Stdlib rename must not break any existing functionality |
| Single binary distribution | Stdlib `.sifr` files must be embedded via `include_str!`, not external files |
| Backward compatibility during transition | Old `sifr.*` imports must still work through the new architecture |

---

#### **Scope**

##### Features In

1. Rename `sifr.*` intrinsic registry to `_sifr.*` in stdlib.rs and codegen
2. Rename `emit_stdlib_call` to `emit_intrinsic_call`
3. Create `lib/sifr/` directory with embedded `.sifr` stdlib files
4. Implement two-phase compilation (stdlib `.sifr` files first, then user code)
5. Block user imports of `_sifr.*` with compile error
6. Proof-of-concept: `lib/sifr/test.sifr` as a pure Sifr stdlib module
7. Demo showcasing the new architecture

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Migrating all 13 modules to `.sifr` | Deferred to milestone_stdlib_migration |
| New stdlib modules | Deferred to milestone_stdlib_expansion |
| Deleting `emit_intrinsic_call` | Still needed for intrinsics; only renamed |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona | Use-Case / Benefit | Dependencies | **AC-ID** |
| --- | --- | --- | --- |
| Sifr developer | `from sifr.test import assert_eq` resolves to `.sifr` file | Two-phase compilation | AC-1 |
| Sifr developer | Existing stdlib imports continue to work | Backward compat | AC-2 |
| Sifr developer | `from _sifr.fs import read_bytes` blocked in user code | Import blocking | AC-3 |

---

### **Acceptance Criteria**

| **AC-ID** | Persona | Criterion |
| --- | --- | --- |
| AC-1 | Developer | **Given** a user writes `from sifr.test import assert_eq` **When** the compiler processes the import **Then** it resolves to `lib/sifr/test.sifr` and the function works correctly |
| AC-2 | Developer | **Given** existing E2E tests use `from sifr.math import sqrt` etc. **When** the compiler runs **Then** all 173 pass tests and 17 fail tests still pass |
| AC-3 | Developer | **Given** a user writes `from _sifr.fs import read_bytes` **When** the compiler processes the import **Then** it emits a compile error: "cannot import from _sifr.* (internal intrinsics)" |
| AC-4 | Developer | **Given** the compiler binary is built **When** stdlib `.sifr` files are needed **Then** they are embedded in the binary via `include_str!` (no external files) |

---

## 2. Solution Design

### **2.1 Functional Requirements**

* Rename all `"sifr.*"` match arms to `"_sifr.*"` in `stdlib.rs` and codegen
* Rename `emit_stdlib_call` to `emit_intrinsic_call`, `used_stdlib_modules` to `used_intrinsic_modules`
* Create `lib/sifr/test.sifr` with pure Sifr implementations of assert functions
* Embed stdlib `.sifr` files in the compiler binary using `include_str!`
* Add two-phase compilation: compile stdlib `.sifr` files before user files
* Add `_sifr.*` import blocking for user code
* Update import resolution to check stdlib `.sifr` modules before falling back to intrinsics

---

### **2.2 Non-Functional Requirements**

| ID | Requirement |
| --- | --- |
| NFR-1 | No measurable compilation time regression (stdlib compilation adds < 100ms) |
| NFR-2 | Compiler binary size increase < 50KB (embedded `.sifr` sources are small) |

---

### **2.3 High-Level Architecture**

```
User Code (.sifr files)
    ↓ imports from sifr.*
Stdlib .sifr files (embedded via include_str!)
    ↓ imports from _sifr.*
Intrinsic Registry (stdlib.rs + emit_intrinsic_call)
    ↓ generates
Rust Source Code
```

---

### **2.4 Detailed Component Design**

**sifr_hir/src/stdlib.rs**
- Rename `get_stdlib_module()` to `get_intrinsic_module()`
- Rename `is_stdlib_module()` to `is_intrinsic_module()`
- Change all match arms from `"sifr.io"` to `"_sifr.io"`, etc.
- Keep all function type signatures unchanged

**sifr_codegen/src/lib.rs**
- Rename `emit_stdlib_call()` to `emit_intrinsic_call()`
- Rename `used_stdlib_modules` to `used_intrinsic_modules`
- Rename `stdlib_functions` to `intrinsic_functions`
- Update Cargo dep injection match arms from `"sifr.*"` to `"_sifr.*"`
- Update the pre-scan that collects stdlib imports to check for `"_sifr."` prefix

**sifr_hir/src/lower.rs**
- Add new resolution path: when import is `sifr.*`, check stdlib `.sifr` module exports first
- Keep existing intrinsic resolution as fallback (for modules not yet migrated to `.sifr`)
- Add `_sifr.*` import blocking: if source is user code and module starts with `_sifr.`, emit error
- Accept a new parameter `stdlib_exports: &ExternalDefs` for pre-compiled stdlib modules

**sifr_driver/src/lib.rs**
- Add `STDLIB_FILES` constant: `&[(&str, &str)]` mapping module names to embedded source
- Add `compile_stdlib()` function that parses and lowers all stdlib `.sifr` files
- Update `build()` and `build_project()` to call `compile_stdlib()` first
- Pass stdlib exports to user module lowering via `ExternalDefs`

**lib/sifr/test.sifr** (new file)
- Pure Sifr implementation of `assert_eq`, `assert_ne`, `assert_true`, `assert_false`
- Uses `print()` and conditionals -- no intrinsics needed

---

### **2.9 Trade-offs & Alternatives**

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| Embed `.sifr` files via `include_str!` | Single binary, no file system deps | Slightly larger binary | Chosen: simplicity and distribution |
| Load `.sifr` files from disk at runtime | Smaller binary, editable stdlib | Requires file system, complicates distribution | Rejected |
| Rename intrinsics in-place without `.sifr` files | Simpler change | Doesn't establish the architecture | Rejected: need the full pipeline |

---

### **2.10 Testing Strategy**

| **AC-ID** | Test Layer | Happy-Path Check | Non-Happy / Edge Check | Tooling & Automation | Pass/Fail Gate |
| --- | --- | --- | --- | --- | --- |
| AC-1 | E2E | `from sifr.test import assert_eq` works | Import non-existent function from sifr.test | `cargo test` | All pass |
| AC-2 | E2E | All 173 pass tests pass | All 17 fail tests still fail correctly | `cargo test` | Zero regressions |
| AC-3 | E2E | `from _sifr.fs import read_bytes` emits error | Various `_sifr.*` import attempts | `cargo test` | Compile error emitted |
| AC-4 | Unit | `STDLIB_FILES` contains test.sifr | Empty stdlib file handling | `cargo test` | Embedded correctly |
