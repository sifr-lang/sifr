# M10 milestone review — pass 12

- Reviewer: Codex CLI
- Model: `gpt-5.6-sol`
- Reasoning: high
- Service tier: fast
- Reviewed HEAD: `dc07946b7b0171ebf5015ea8d4a2701dbf096def`
- Scope: full M10 implementation, complete milestone history, and review passes 1–11
- Review tree: clean detached worktree
- Verdict: **CHANGES REQUESTED**

## Findings

1. **High — basename-based global exemptions collapse distinct canonical
   stdlib classes.** `sifr.csv.Error` and `sifr.configparser.Error` retain the
   global Rust `Error` name and are stripped as infrastructure. Their union
   identities consequently collide as well.
2. **High — inheritance loses canonical stdlib parent identity.** `HirClass`
   and `SuperCall` retain only the source/alias spelling, so a local subclass
   of an imported sealed stdlib class type-checks but emits references to a
   nonexistent alias instead of the canonical Rust parent.
3. **High — match and try paths compare aliased stdlib classes by raw names.**
   Class patterns can miss canonical union/`Result` members, while aliased
   `except` handlers can be reported uncovered or lower against inconsistent
   error identities.
4. **High — generic method and operator self-type shortcuts are
   basename-only.** A distinct canonical same-basename class can be mistaken
   for the local class, and compiler-prefixed generic self types can bypass
   source-name escaping.

## Required remediation

- Exempt only exact canonical identities that genuinely use global runtime
  representations; independently seal same-basename stdlib declarations.
- Carry resolved parent type/identity through class and `super()` HIR and
  render executable parent references through identity-aware type rendering.
- Carry resolved class identities through patterns, handler coverage, and body
  error sets; compare exact canonical identities and specializations.
- Introduce one exact self-type predicate for generic methods and operators,
  rendering the real self target through the class implementation target and
  every other type through the shared identity-aware renderer.
- Add native regressions covering duplicate stdlib `Error` aliases and unions,
  direct/aliased stdlib inheritance, aliased union/`Result` matching and
  `except`, compiler-prefixed generic self returns, and canonical/local
  same-basename operators.

## Reviewer validation

- Audited the complete `origin/main...HEAD` diff in a clean detached worktree.
- Re-grounded the phase contract, pass-11 remediation, and authoritative gate
  evidence.
- Traced canonical identities through type rendering, stdlib sealing/DCE,
  unions, inheritance, patterns, try handlers, generics, and operators.
- Identified no separate Medium or Low findings.

Final reviewer verdict: `CHANGES REQUESTED — four High identity gaps remain`.
