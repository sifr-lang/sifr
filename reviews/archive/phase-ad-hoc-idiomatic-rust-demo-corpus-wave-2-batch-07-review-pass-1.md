Looking at these three files for behavioral equivalence and idiomatic Rust quality.

## demos/string/idiomatic.rs

The `capwords` implementation works correctly for the test cases. However, there's one approachability concern:

**Line 12-13**: `chars.as_str().to_lowercase()` is an unusual pattern. The `Chars` iterator doesn't have a direct `as_str()` method—you're relying on the fact that `chars` borrows the original string. This works but may confuse readers. A more common approach would be to iterate with indices or collect back.

Also, `first.to_uppercase()` on a single `char` allocates a `String`, which is technically correct but heavyweight for what's usually a single character.

These are minor style concerns, not correctness issues.

## demos/textwrap/idiomatic.rs

The `ValueError` struct is well-implemented but could use `std::io::Error` or `thiserror::Error` for better ergonomics. However, for an educational demo showing the API, defining it locally is acceptable.

The `dedent`, `wrap`, `fill`, `indent`, and `shorten` functions all appear behaviorally correct.

## demos/fnmatch/idiomatic.rs

**Line 28-29**: `fnmatch` and `fnmatchcase` are identical implementations. In Python's `fnmatch`, `fnmatch` is case-sensitive on Unix/macOS but case-insensitive on Windows, while `fnmatchcase` is always case-sensitive. Since the test only validates that `fnmatchcase` is case-sensitive (not that `fnmatch` differs), the current implementation passes the tests but doesn't demonstrate the distinction.

For an educational demo, this is a minor missed opportunity to show how Rust could express the case-insensitive variant differently (e.g., using `to_lowercase()` on both arguments).

## Summary

No actionable issues. All implementations pass behavioral tests and are idiomatic enough for educational value. The stylistic observations above are nitpicks that don't affect correctness or maintainability in meaningful ways.
