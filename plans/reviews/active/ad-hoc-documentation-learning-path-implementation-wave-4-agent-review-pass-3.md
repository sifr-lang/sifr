Wave 4 remains **PR-ready**. No blockers introduced.

The two cleanup changes are clean:

- **`docs/from-python.mdx:76`** — "Use `mut` when a function should mutate a borrowed parameter, and `own mut` when it should take ownership and mutate the value." Concise, accurate, consistent with the ownership page's contract table.

- **`docs/language/ownership.mdx:79`** — "Use `own mut` when the function consumes a value and needs to mutate it." Direct parallel to the `own` sentence on line 66 ("Use `own` when a function consumes a value."). The removed "before returning or dropping it" phrase was redundant given the code example immediately below.

Both sentences are technically correct, stylistically tighter than before, and introduce no inconsistencies with the rest of the ownership model documentation.
