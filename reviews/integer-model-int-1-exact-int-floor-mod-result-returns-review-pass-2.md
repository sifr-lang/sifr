

All validations pass. The blocker from pass 1 is resolved.

**Verifier summary:**
- `cargo fmt` — no output (already clean)
- `cargo check -p sifr_codegen` — clean
- E2E fixture `exact_int_floor_mod_result_return` — passes (compile + run)

**Blocker resolution confirmed:**

The fix at `function_emitter.rs:463–467` adds an explicit `else if nested_returns_sifr_int_result` branch alongside the existing `nested_returns_sifr_int` branch:

```rust
ret: if nested_returns_sifr_int {
    Some(RustType::Named("SifrInt".to_string()))
} else if nested_returns_sifr_int_result {
    Some(result_int_return_type_to_sifr_int(&func.return_type))  // ← new
} else {
    self.lower_function_return_type(func, false)
},
```

This eliminates the dependency on the side effect of inserting into `sifr_int_result_function_returns` before `lower_function_return_type` is called for the recursive case.

**The reviewer is satisfied.**
