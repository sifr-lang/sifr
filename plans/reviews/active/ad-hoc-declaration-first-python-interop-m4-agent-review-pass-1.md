## Summary

The M4 milestone is largely wired up correctly — sealed identities, semantic close consuming the receiver, target probes, cache-key fingerprints, `record_field` attribute-then-item, `expect_instance` factory rejection, must-use side table with control-flow-join checks, and a live biip/schwifty declaration-first binary. The runtime unit tests exercise semantic close, poison-on-failure, recursive nested-path errors, and factory mismatches with exact-once release assertions, all of which pass.

However, the codegen for compound return types has two runtime-panic bugs (Tuple containing a Record / Option), the class-method must-use exit check drifts from the top-level one and can false-positive on bindings that were consumed inside popped nested scopes, reassignment silently abandons a live `cleanup=close` obligation, and the aggregate-transfer helper doesn't cover comprehensions/match. The declaration-first tests only substring-check rendered Rust and never build the wrapper, so the tuple/record and Option-in-tuple panics ship undetected. Findings #1–#4 must be fixed before this milestone can meet the M4 acceptance criteria; #5–#7 are ordering-safe but should follow.

**Gate decision:** the top four findings each break an explicit M4 acceptance clause (no data-dependent panics in generated code, cleanup exact-once, must-use transfer through moves/returns/aggregates), and they are direct code defects rather than doc gaps.

NOT SATISFIED
