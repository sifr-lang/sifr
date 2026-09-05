# Rust Interop Certification 5 Review — Round 3

Reviewer: agent (`--effort medium`)

Scope: revised certification 5 working-tree implementation, excluding the
unrelated `editor_integrations` submodule change.

## Reviewer output

The reviewer independently ran both bound tests, fixture/scenario self-tests,
and inventory checks. All seven round-2 findings were confirmed fixed.

Remaining findings:

1. **Medium-high:** `clone=arc` is inert metadata and contradicts the normative
   architecture text that says all clones share one handle-state cell. The
   evidence only clones inside Rust and deliberately observes independent outer
   `HandleSlot`s. Drop the unused policy and independent-slot normalization, or
   reconcile the compiler/runtime contract.
2. **Medium:** the new opaque-class `pass` exemption is indentation-scoped and
   also exempts a nested placeholder method. Restrict it to a class whose sole
   body statement is one direct `pass`, and mutation-test a nested method.
3. **Low:** `.set_skip_set_lib_name()` narrows the Redis handshake but is
   undisclosed and unanchored. Add a required token/mutation and document the
   exercised-frame restriction.
4. **Low:** opaque-resource scenario self-tests copy a local `target/` tree;
   ignore it during `copytree`.

**VERDICT: NOT SATISFIED**
