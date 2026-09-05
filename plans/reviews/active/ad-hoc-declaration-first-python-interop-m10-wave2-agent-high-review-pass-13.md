# M10 Wave 2 review pass 13

Reviewer: agent, high reasoning, fast service tier

Scope: complete `main...095215a93` Wave 2 implementation after pass-12
remediation

Verdict: **CHANGES REQUIRED**

The reviewer confirmed the pass-12 transitive-inheritance and contextual hash
repairs, then found four remaining compiler capability gaps:

1. dictionary value projections and formatting consumers still admitted
   transitive `NonSend` classes whose emitted Rust representation lacks the
   required `Clone`, `Display`, or `Debug` trait;
2. `dict(iterable)` did not validate that its inferred key type can satisfy the
   `HashMap` `Eq + Hash` requirement, allowing specialized generic class keys;
3. list method validation conflated `Clone`, `PartialEq`, and total `Ord`, so
   `list[set[float]].contains(...)` and `list[float].sort()` survived lowering;
4. the phase ledger and architecture could not claim closure until those paths
   had permanent native-negative evidence and passed local validation.

## Remediation

The compiler now gates dictionary cloning projections, dictionary construction,
`print`, `str`, f-string interpolation, and `repr` against the exact Rust traits
their code generators use. List methods have separate Clone, structural
equality, and total-order checks. Explicit `None` formatting avoids an invalid
Rust unit `Display` requirement, while task/failure/timeout/select wrappers and
`JoinItemId` are admitted according to the traits their compiler-owned runtime
representations actually implement. Eight permanent fail fixtures cover the
review reproducers and adjacent formatting paths, expanding the compile-fail
matrix to `510/510`. Full type-system `102/102`, lowering `742` passed with one
ignored, and code-generation `817/817` suites pass, as do workspace Clippy,
formatting, HIR maintainability, and the `900`-line file-size guardrail over
`2636` files. The authoritative create-PR facade passes Python interop `11/11`,
runtime platform `28/28` with one gated skip, and E2E `131/131`; all enforced
step budgets pass. A fresh complete `main...HEAD` review remains required.

The known dirty Ruff submodule was explicitly ignored and preserved.
