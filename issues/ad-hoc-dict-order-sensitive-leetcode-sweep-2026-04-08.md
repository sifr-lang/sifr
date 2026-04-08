# Ad-hoc: Dict-Order-Sensitive LeetCode Sweep (2026-04-08)

## Why
Sifr intentionally maps `dict` to Rust `HashMap`, and dict iteration order is unspecified by architecture. Some LeetCode fixtures may still accidentally rely on Python insertion-order dict iteration and pass nondeterministically.

## Goal
Audit the full `audits/leetcode/` corpus for order-sensitive patterns and rewrite fixtures to order-agnostic logic where needed.

## Trigger
Reviewer follow-up from `reviews/leetcode-2554-runtime-semantic-closure-review-pass5.md` (A2).

## Scope
- Search for patterns like dict-built-from-range followed by direct key iteration.
- Search for any logic where correctness depends on deterministic dict key order.
- Adapt fixtures to deterministic alternatives (`range`, `sorted(...)`, explicit ordered structures) based on intended algorithm.

## Acceptance criteria
- No fixture in `audits/leetcode/` relies on unspecified dict iteration order for correctness.
- Corpus rerun remains green after adaptations.
- Taxonomy/report updated if any additional fixtures are changed.

## Checklist
- [ ] Pattern scan completed
- [ ] Candidate fixtures reviewed and classified
- [ ] Required fixture adaptations implemented
- [ ] Full corpus rerun completed
- [ ] Results documented
