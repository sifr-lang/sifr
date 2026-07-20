# M10 Milestone Codex Review Pass 3

- Reviewer: Codex CLI `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Scope: complete M10 implementation plus milestone-review remediations, range
  `e4fdc942ed..b73b68a53`
- Review tree: clean detached worktree at committed HEAD
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — required M10 native evidence failed from a clean checkout.** Generic
   repository ignore rules omitted checksum-required vendored files. The first
   clean run lacked 344 nested `Cargo.lock` files; after exposing those, a clean
   validation rerun also identified ignored vendored `build/` and `.vscode/`
   sources. The pinned CPython 3.11 runtime tests passed `5/5`, but all five
   compiled examples failed during Rust bridge probing when required vendored
   files were absent.
2. **Low — public documentation overstated nominal `PythonError` enforcement.**
   The shared predicate accepts the exact five-field structural contract, while
   the public documentation said the type had to be imported from `sifr.python`.
3. **Low — exit evidence still marked active `PYZC` as reserved.** The evidence
   table contradicted the active diagnostic registry and M10 capability ledger.

## Reviewer validation

- Python buffer lowering passed `37/37`.
- Python buffer code generation passed `10/10`.
- The Python interop evidence self-test passed.
- The clean CPython 3.11 lane's exact runtime release tests passed `5/5`.
- File-size, HIR maintainability, and diff-integrity guardrails passed.
- The prior writable-`Self`, Python error-channel, and roadmap findings were
  confirmed fixed.

## Required remediation

- Track every checksum-required vendored file so required profiles run from a
  clean checkout.
- Document the structural `PythonError` field contract accurately.
- Move active `SIFR-PYZC-0001` evidence out of the reserved-family list.
