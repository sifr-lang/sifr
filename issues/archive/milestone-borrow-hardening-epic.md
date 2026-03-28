# milestone_borrow_hardening — Borrow Exclusivity and Diagnostics

## 1. Product Requirements

### Objective

Harden the borrow-by-default model with exclusivity enforcement, clear error messages, comprehensive tests, and stdlib updates. This milestone ensures the ownership model is production-ready and documented before async/concurrency features are built on top.

### Scope

**In Scope:**

1. Mutable borrow exclusivity tracking (`is_mut_borrowed` on `VarInfo`)
2. Clear, actionable diagnostic error messages for borrow violations
3. Update 50 borrowing audit tests for borrow-by-default semantics
4. New E2E pass/fail tests for borrow_default, mut_param, own_param, exclusivity
5. Stdlib updates: `mut` on mutating collection functions

**Out of Scope:**

| Feature | Reason |
| --- | --- |
| Async/concurrency features | Deferred to milestone_async |
| Lifetime annotations | Not needed for Sifr's model |
| Borrowed parameter escape detection | Complex; deferred to future milestone |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | **Given** `f(mut x, mut x)`, **When** checked, **Then** error: "cannot borrow 'x' as mutable because it is already borrowed" |
| AC-2 | **Given** `f(mut x, x)`, **When** checked, **Then** error: cannot borrow as both mutable and immutable |
| AC-3 | **Given** all 50 borrowing audit tests, **When** run, **Then** pass/fail correctly under borrow-by-default |
| AC-4 | **Given** new E2E tests for borrow_default, mut_param, own_param, **When** run, **Then** all pass |
| AC-5 | **Given** `set_add(items, val)`, **When** Rust is emitted, **Then** first param uses `&mut` |
| AC-6 | **Given** `cargo test`, **When** run, **Then** all tests pass |

## 2. Solution Design

### Key Files

| File | Changes |
| --- | --- |
| `crates/sifr_hir/src/scope.rs` | Add `is_mut_borrowed` to `VarInfo` |
| `crates/sifr_hir/src/lower.rs` | Exclusivity checking, error messages |
| `crates/sifr_driver/src/lib.rs` | Error formatting |
| `audits/borrowing/*.sifr` | Update tests |
| `crates/sifr/tests/e2e/pass/` | New pass tests |
| `crates/sifr/tests/e2e/fail/` | New fail tests |
| `crates/sifr_hir/src/stdlib.rs` | `mut` on mutating functions |

### Testing Strategy

| AC-ID | Test Layer | Check |
| --- | --- | --- |
| AC-1, AC-2 | E2E fail | Double mut borrow produces error |
| AC-3 | Audit | All 50 borrowing tests updated |
| AC-4 | E2E | New pass/fail tests for borrow model |
| AC-5 | E2E | Stdlib mutating functions work with `mut` |
| AC-6 | CI | `cargo test` passes |
