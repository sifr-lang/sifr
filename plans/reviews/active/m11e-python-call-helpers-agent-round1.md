# M11e Python Call Helper Migration - agent Review, Round 1

## Scope

Migrate `_sifr.python.py_call` and `_sifr.python.py_call_attr` from
compiler-retained intrinsic lowering (`sifr_codegen::intrinsics::registry::python`
+ `sifr_retained_intrinsics::python`) into compiled stdlib declarations backed
by `sifr_stdlib::python::py_call` / `py_call_attr`. Kwargs are split into
`kwargs_keys: list[str]` and `kwargs_values: list[tuple[int, int]]` because
direct Rust interop cannot round-trip `list[tuple[str, tuple[int, int]]]`.
Runtime CPython substrate (`sifr_runtime::python::call_object` /
`call_attr`) is unchanged; the stdlib shim delegates.

## Verification of intent

- Intrinsic dispatch removed:
  - `crates/sifr_codegen/src/intrinsics/registry/python.rs` drops `py_call` /
    `py_call_attr` match arms and the two `lower_py_call*` helpers.
  - `crates/sifr_retained_intrinsics/src/python.rs` drops the retained signature
    rows.
  - `internal_docs/stdlib_retained_compiler_intrinsics.toml` allowlist entries
    removed.
  - `scripts/check_stdlib_migration_closure.py` adds both names to
    `RETIRED_INTRINSICS`.
- No compiler-side call site still lowers to `sifr_runtime::python::call_object`
  / `call_attr`; `rg -n 'sifr_runtime::python::call_object|call_attr' crates/`
  turns up only the runtime module itself, callback/coroutine internals, and
  the stdlib shim - no codegen path.
- Public `sifr.python.call` / `call_attr` signatures unchanged
  (`list[Object]`, `list[tuple[str, Object]]`); only the internal marshaling
  differs. The keys/values split follows the M11d pattern already accepted for
  `py_from_dict_str` / `py_from_record`.
- Runtime call semantics preserved: `sifr_stdlib::python::py_call` recombines
  keys+values into `Vec<(&str, ObjectRaw)>` and forwards to
  `python::call_object`; `py_call_attr` forwards similarly, preserving the
  runtime's callable-open / call / close sequence in `object_ops::call_attr`.
- No fallback path introduced.
- Both new tests (`python_call_helpers_are_owned_by_compiled_stdlib_declarations`,
  `python_call_helpers_codegen_through_sifr_stdlib`) assert intrinsic removal
  and that `_sifr.python`'s emitted Rust contains
  `sifr_stdlib::python::py_call(` / `py_call_attr(` plus the `kwargs_keys` /
  `kwargs_values` tokens.

## Findings

### 1. `keyed_object_handles` mismatch message threading is fine but slightly noisy - Nit

`crates/sifr_stdlib/src/python.rs:224` now takes a `mismatch_message: &str`
so `py_call` / `py_call_attr` can attribute the error to the call surface
rather than to a "keyed object constructor". Correct - but the two constructor
call sites (`py_from_dict_str`, `py_from_record`) now duplicate the same
literal. A `const CONSTRUCTOR_MISMATCH: &str = "..."` or letting the helper
take an `enum { Call, CallAttr, Constructor }` would remove the duplication.
Non-blocking.

### 2. Two-pass kwargs marshaling at the sifr level - Nit / consistency

`stdlib/sifr/python.sifr:293-294` and `316-317` walk `kwargs` twice:
`_keys_from_keyed_objects` then `_handles_from_keyed_objects`. The removed
`_keyed_handles_from_objects` did it in a single pass. This is intentionally
symmetric with the M11d constructors (`from_dict_str`, `from_record` at
`stdlib/sifr/python.sifr:667-668` / `678-679`) and consistent with the
now-required split shape at the Rust interop boundary, so the extra pass is
paid for uniformly. Acceptable; keep the pattern consistent within this
module. Non-blocking.

### 3. Defensive length check in `keyed_object_handles` is unreachable from
sifr callers - Info

Both `kwargs_keys` and `kwargs_values` are built by iterating the same
`kwargs: list[tuple[str, Object]]` in order, so their lengths are equal by
construction. The `if keys.len() != values.len()` guard therefore only fires
against direct Rust callers. This mirrors what M11d already accepted; keep it
as defense-in-depth. No action.

### 4. `stdlib/sifr/python.sifr` at 886 lines - Watch

The file is 14 lines under the 900-line guardrail. Not a violation today, but
any further Python surface migration (e.g. remaining collection extractors,
Arrow / DLPack helpers) will hit the cap. Worth flagging on the phase doc so
the next slice budgets a responsibility-based split (e.g. carve keyed-object
marshaling helpers into a separate module) rather than growing this file.
Non-blocking for this PR.

### 5. Empty round-1 review file was pre-created - Info

`plans/reviews/active/m11e-python-call-helpers-agent-round1.md` existed as an
empty file when the review started; this pass fills it. No action.

## Confidence checks

- Correctness: kwargs key/value correspondence preserved by index in both
  languages; lengths equal by construction on the sifr side and re-checked on
  the Rust side. Positional args untouched.
- Coverage: closure script + registry-absence test + codegen-emission test
  triangulate the migration. `cargo test -p sifr_codegen`, `-p sifr_driver`,
  `-p sifr_retained_intrinsics`, `-p sifr_stdlib --features python`, and the
  four migration/allowlist/manifest/resource guards were all run per the task
  brief.
- Behavior: single-file `check`/`emit` of
  `verification/areas/python_interop/fixtures/primitive_conversion/primitive_roundtrip.sifr`
  showed only `sifr_stdlib::python::py_call` / `py_call_attr` and the
  underlying `sifr_runtime::python::call_object` / `call_attr` in the emitted
  Rust - no orphaned intrinsic path. The `numpy_full_example.sifr`
  SIFR-PYTRUST-0002 result is single-file-trust-mode noise and not a slice
  regression.

## Verdict

**PR-ready.** All five findings are non-blocking (three nits, two info). The
migration is symmetric with M11a-d, removes both compiler-retained rows, keeps
public `sifr.python.call` / `call_attr` behavior intact, and adds guardrails
in the intrinsic-registry and codegen-emission test layers. File-size
headroom on `stdlib/sifr/python.sifr` should be tracked in the phase doc for
the next Python slice.
