# M10 Wave 2 whole-diff review — pass 9

Reviewer: agent, high reasoning, fast service tier, read-only whole-diff review.

## Verdict

**CHANGES REQUIRED. Not satisfied; M10 Wave 2 is not ready to merge.**

## Blockers

1. **High — equality capability remains unsound and accepts invalid Rust.** `supports_structural_equality` equates equality support with absence of affine resources. Membership checks only affinity, while code generation calls Rust `.contains()` and requires `PartialEq`. Probes accepted `Any in list[Any]` and callable-bearing classes in lists even though their Rust representations do not implement `PartialEq`. `is` and `is not` also bypass comparison validation, lower to `==` and `!=`, and accept `python.Buffer` despite its Rust type having no `PartialEq`. Implement and consistently enforce a real equality/identity capability.
2. **High — tuple/star unpacking still accepts ownership- and clone-invalid programs.** Star unpack rejects only affine elements, but code generation unconditionally emits `.clone()` and `.to_vec()`. Probes accepted `list[Any]` and lists of callable-bearing non-`Clone` classes. Direct tuple unpack records consumption but emits a direct pattern move; a borrowed `tuple[Holder, int]` unpack returning `Holder` is accepted even though it cannot produce an owned `Holder`. Gate preserved sources on clone capability and use genuinely consuming extraction for owned sources.
3. **High — affine captures still bypass async-generator rejection and reach invalid code generation.** Validation examines only the nested generator's declared parameters and return type. A probe with an outer owned `python.Buffer` captured by a parameterless nested async generator passes checking. Nested-function code generation then uses the generic path without async-generator materialization setup. Validate free-variable captures and either implement nested-generator code generation or reject the unsupported form.
4. **Medium — activation evidence remains materially overstated.** The capability ledger and phase tracker mark equality, unpack ownership, and async-generator sendability closed despite the accepted-invalid programs above. Keep those claims incomplete until permanent compiler/codegen evidence closes them.

## Required closure

Implement a real recursive equality capability and enforce it for membership and identity lowering; make tuple/star unpacking respect source ownership and clone capability; reject or correctly lower nested generators with affine captures; add permanent regression evidence; correct the activation claims; rerun affected and authoritative validation from the requested clean target; and submit the complete Wave 2 diff to another independent whole-diff review.
