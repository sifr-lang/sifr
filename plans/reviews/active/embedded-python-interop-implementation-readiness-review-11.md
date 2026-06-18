I read the current plan (895 lines, under the 900 cap) and the round-10 review. Here's the round-11 readout.

## 1. Round-10 blocking gaps — resolved?

| Round-10 gap | Status |
|---|---|
| Sifr→Python argument conversion rule (Conversion Rules) | ✅ Added as a bullet at line 316. Covers `None`/bool/int/float/str/bytes/`py.Object`, rejects implicit container conversion, routes overflow to `py.TypeConversionError`. |
| Trust enforcement owner & DoD | ✅ milestone_py_3 scope (line 724) + DoD fixtures bullet (line 730). |
| `import python <name>` future-sugar block deleted | ✅ Replaced with single decisive line at 273. |
| Free-threaded CPython language | ✅ Both line 52 (Non-Goals) and line 130 (probe validation) are clean — no deferral language left. |
| `PyValue` placeholder | ⚠️ Variants added at line 184, but introduced one new minor ambiguity — see §2. |

## 2. New ambiguity from the PyValue/conversion wording

One small issue worth fixing before commit:

**Line 184:** `pub enum PyValue<'a> { None_, Bool(bool), Int(PyInt), Float(f64), Str(&'a str), Bytes(&'a [u8]), Object(&'a PyObjectHandle) }`

- `PyInt` is not defined anywhere else in the document (grep confirms zero other references). The Conversion Rules at lines 300–301 distinguish "exact `int`" from "fixed-width `py.to[int32]` etc.", so a single `Int(PyInt)` collapses a distinction the contract makes elsewhere. An implementer reading this cold cannot tell whether `PyInt` is arb-precision, fixed-width, or a sum of both.

Smallest fix (line-neutral): split into the two variants the contract already commits to —

```rust
Int(i128), IntFixed(PyIntFixedWidth),
```

Or, keep `Int(PyInt)` and add one sentence below the block defining `PyInt` as "an internal packer that holds either an exact Python `int` or a checked fixed-width Sifr integer." The two-variant form is cleaner because it mirrors the conversion contract directly.

Minor, non-blocking: the enum is declared `PyValue<'a>` but the function signatures at lines 197–198 use bare `PyValue` (readers can infer the elided lifetime). And the new argument-conversion bullet at line 316 says "call arguments" without naming `kwargs` — but `kwargs: &[(&str, PyValue)]` on line 198 transitively binds the same rule, so this reads fine.

## 3. Trust enforcement — right milestone & DoD?

Yes. milestone_py_3 owns both static HIR enforcement and the runtime `@trust_python_dynamic` root check, with `SIFR-PYTRUST` diagnostics, and DoD fixtures cover static imports, dynamic imports, native roots, wildcard rejection, and package-graph load failures. Optional polish: the DoD bullet says "dynamic imports" — could be tightened to "`@trust_python_dynamic` runtime root check" to mirror the scope phrase exactly, but not required.

## 4. Remaining blockers under no-fallback/no-deferral

The PyInt clarification in §2 is the only outstanding item. Everything else surveys clean:

- `grep -i "future\|may exist\|deferred\|TBD"` — the only "future" hit is "future tensor libraries" at line 461 describing DLPack's reach (descriptive, not a deferral).
- All `fallback` mentions are policy constraints ("no fallback allowed"), not escape hatches.
- No "audited future phase" / "until a future audit" remnants.

## 5. Verdict

Almost ready — one small definitional touch on `PyInt` (line 184) is the only thing standing between the current draft and commit-ready. With that fix, the phase is implementation-ready, self-contained, and aligned with the no-fallback / no-deferral policy. **Not yet ready to commit — apply the PyInt clarification first.**
