# Object Model Audit Report

**5 PASS / 1 FAIL** out of 6 tests.

## Issues Found

### Issue 1: Dunder Methods Trigger Move Semantics (Known)
**Test:** 04 | Using `a + b` then `a == ...` fails with `use of moved value: 'a'`. This is the same move semantics issue from the type system audit -- operator overloading consumes the left operand.

## What Works
- Equality vs identity: `==` for value equality, `is`/`is not` for None checks
- Truthiness: `bool(0)` falsy, `bool(1)` truthy, `bool("")` falsy, None is falsy, optional narrowing via truthiness
- Mutability: lists mutable (append works), strings immutable (methods return new), tuples immutable
- Attribute access: `obj.field`, `obj.method()`, chained method calls (`p.greet().upper()`)
- Hash built-in: `hash(42)`, `hash("hello")`, consistent hashing
