# M10 Wave 2 Whole-Diff Review — Pass 25

Reviewer: agent, high reasoning, fast service tier
Scope: complete committed `main...2ac7fd50c` diff (317 files, +14,284/-1,271)
Verdict: **CHANGES REQUIRED**

## Findings

### High — consuming subclass coercion remains incomplete for union arguments and covariant union/`Result` values

The registry call adapter constructs a union variant before attempting the
owned subclass upcast. The upcast helper only handles a direct class target, so
an accepted `Child` argument to `own Root | int` emits an invalid
`IntOrRoot::Root(Child::new(...))`. Already-wrapped covariant union and `Result`
values are likewise accepted by the type system but are not converted by call,
local, or return lowering. Native reproductions fail with Rust type mismatches.

Required remediation: make consuming coercion recursive over union and `Result`
representations and apply it consistently at arguments, assignments, and
returns. Add direct, transitive, imported/re-exported native coverage.

### High — recursive capability analysis uses class basenames instead of canonical identities

Affine, Clone, equality, Hash, Debug, and task-sendability recursion guards key
visited classes by local basename. Distinct declarations such as `a.Root` and
`b.Root` therefore collide, allowing traversal to stop before reaching nested
affine or trait-incompatible fields. This can make Sifr accept a program whose
emitted Rust representation does not satisfy the claimed capability.

Required remediation: key recursive traversal by canonical declaration
identity and, where capability depends on specialization, concrete type
arguments. Add multi-module repeated-basename capability and affine tests.

## Cleared Areas

- Writable-buffer exclusivity is rooted through legal nested receiver places;
  affine field/index projections remain rejected by the whole-aggregate rule.
- Exact canonical ancestry wins over unique-tail fallback.
- Fieldless generics use non-owning `PhantomData<fn() -> T>` and direct concrete
  capability checks align with the emitted representation.
- Buffer acquisition validation, overlap admission, bounded access, exporter
  retention, explicit/drop release, and exact-once raw release remain sound.
- No new user-triggerable runtime panic path was found.
- NumPy-complete matrices, demos, and public documentation remain Wave 3 work.

## Final Verdict

**CHANGES REQUIRED**
