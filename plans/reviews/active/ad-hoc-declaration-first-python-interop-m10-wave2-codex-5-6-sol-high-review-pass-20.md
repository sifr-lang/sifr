# M10 Wave 2 review pass 20

- Reviewer: Codex CLI, `gpt-5.6-sol`, high reasoning, fast service tier
- Scope: complete committed `main...HEAD` diff after pass 19 remediation
- Verdict: **CHANGES REQUIRED**

## Findings

1. High: frontend re-export collection cloned imported function, class, and constant types without localizing embedded user-class identities. A multi-hop facade over two same-named generic classes therefore accepted a cross-assignment between the final aliases.
2. High: class-pattern inference read fields from the unspecialized class declaration. Matching `Box[int]` bound a captured field as `T`, and nested generic patterns had the same defect.
3. Medium: the phase and evidence ledger could not claim complete re-export, transitive-ancestry, or declared pattern-capture closure until multi-hop and generic permanent regressions passed.

## Remediation disposition

- Reopened M10 Wave 2.
- Localized complete re-exported function, class, constant, and ancestry type
  graphs across multi-hop facade boundaries, retaining distinct same-name class
  identities and generic metadata.
- Specialized class-pattern fields from the narrowed subject type in both
  inference and final HIR lowering, including nested generic patterns.
- Added multi-hop same-name re-export, factory-signature, transitive-ancestry,
  and generic/nested pattern cases to the permanent matrix.
- Full codegen (`825/825`), lowering (`754` passed, one ignored), frontend
  (`47/47`), workspace Clippy, formatting, maintainability, and the `2673`-file
  size guardrail pass. The authoritative create-PR gate passes Python interop
  `11/11`, runtime platform `28` variants with one gated skip, and E2E `131/131`
  with signature `7c39b8c1dd4fec7c` and `42/42` cache hits.
- Satisfaction is tracked in review pass 21.

The reviewer independently confirmed the peer-receiver operator fixture, separate-statement direct imports, native compound return fixture, focused suites, HIR maintainability, and file-size guardrail.
