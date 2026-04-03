## operators_and_assignment

OK

## collection_comprehensions

Initial reviewer notes:

> 1. The tuple array `[(\"alice\", 95_i64), (\"bob\", 87_i64)]` supposedly had an invalid mixed type shape and would not compile.

Disposition: not accepted. That claim is incorrect: the Rust file compiled cleanly under plain `rustc`, produced the expected output, and the tuple type is the intended `(&str, i64)` shape.

## ordering_rules

OK
