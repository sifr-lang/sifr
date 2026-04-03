## control_flow

Initial reviewer note:

> 1. The tuple-length print was hardcoded as `3` instead of computing the length from the tuple value.

Disposition: accepted. I replaced the naked literal with a tiny `tuple_len(&point)` helper so the companion still demonstrates tuple-length evaluation intentionally instead of printing an unexplained constant.

## control_flow_paths

Initial reviewer note:

> 1. `unreachable_tail()` no longer showed the syntactically present but unreachable second return path from the Sifr demo.

Disposition: accepted. I restored that teaching point with `#[allow(unreachable_code)]` and an explicit unreachable tail expression after `return 9;`, which keeps the CFG shape visible while preserving clean observable behavior.

## compiled_expressions

OK
