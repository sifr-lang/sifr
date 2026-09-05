# M10 Wave 2 review pass 11

Reviewer: agent, high reasoning, fast service tier

Scope: complete `main...HEAD` Wave 2 implementation after pass-10 remediation

Verdict: **CHANGES REQUIRED**

The reviewer confirmed the direct pass-10 cases, then found five High defects in
the broader compiler capability surface:

1. member-versus-union equality recursively removed `None` from non-optional
   unions and stack-overflowed the compiler;
2. list, set, and dictionary membership accepted a union member but emitted the
   unwrapped Rust member for a union collection;
3. child-class derives ignored the traits implemented by the embedded parent;
4. specialized generic classes were accepted as hash keys although their emitted
   generic Rust declarations did not implement `Eq + Hash`; and
5. callable-bearing error classes emitted `Display` and `Error` implementations
   without the required `Debug` implementation.

The reviewer also required the architecture, capability ledger, and phase status
to remain pending until these paths have permanent native or negative evidence.
The known dirty Ruff submodule was explicitly ignored and preserved.

Remediation is tracked in the phase issue and will receive another complete
`main...HEAD` review after local validation.
