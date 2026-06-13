**Review: Methodology and Corpus-Noise Angle**

**1. The >=80 cutoff**

The cutoff is defensible as a triage heuristic but is not calibrated. All manually-promoted items above it have `changed_total_lines` ranging from 82 to 163, a 2x spread that the threshold treats uniformly. The one sub-cutoff manual inclusion (`0295_find_median_from_data_stream` at 56) is correctly justified by an asymptotic behavior change rather than raw size, which is the right reasoning — but this means the 80 boundary is arbitrary relative to the actual severity signal. A lower base threshold (e.g., 50–60) combined with a mandatory manual rescue lane for algorithmic-divergence cases would be more honest about what the number actually gates.

**2. Corpus-noise misclassification risks**

The strongest concern is in **Category 3 ("Okay The Way They Are")**. All three items have `similarity_ratio` in the 0.20–0.32 range, which is not negligible noise — it means 68–80% of the code is divergent. The justification leans heavily on Python-side fixture bloat ("multiple implementations in one file"), which is valid for `0200_number_of_islands` (py_lines=95, three full implementations stated) and partially for `0104` (93 lines for what should be a 10-line recursive function). But `0516_longest_palindromic_subsequence` (72→33 lines, similarity 0.21) is a 2x compression with a single stated solution family — the large diff may reflect genuine algorithmic re-expression rather than Python-side redundancy, and placing it in "okay" without stronger evidence is tentative.

**Category 5 and Category 3 overlap** is a methodological concern: `0023_merge_k_sorted_lists` and `0133_clone_graph` appear in Category 5 as "needs corpus cleanup" but also appear in Category 1 as "Should Have Parity, Rewrite Mainly." `0023` has a 0.07 similarity_ratio, which is genuinely low — the rewrite debt is real and not primarily corpus noise. But `0133` (similarity 0.088) is a 71→20 line compression that could plausibly be attributed to the Python side using a verbose Node/AdjacencyList class vs. the Sifr side using a simpler representation, which may not be rewrite debt so much as representation choice.

**Shared helper boilerplate** is acknowledged in the preconditions but no items are actually flagged for it in the scan. If the diff-scanning script does not compute a "mirrored dead code cancels out" signal, this is a blind spot that could systematically undercount divergence for problems that share graph/list helper utilities between Python and Sifr versions.

**3. The "okay the way they are" list — too permissive or too narrow**

As written, it is slightly **too permissive**. The rationale is sound in principle (Python fixture is bloated, Sifr already matches a clean solution), but the similarity ratios in the 0.20–0.32 range mean the claim that the Sifr version "already matches" is doing a lot of work without external validation. At minimum, each should be verified against the actual Sifr fixture to confirm the clean solution is indeed canonical and not just the least-bad version of a compromised port.

The list is not **too narrow** — there are no obvious candidates that should be added based on the data.

**4. Strongest methodological correction**

The single most impactful fix is to **decompose `changed_total_lines` into a boilerplate-noise component and an algorithmic-divergence component** before applying any threshold. The current structure acknowledges this need in the preconditions but then uses the raw number as the primary gating signal. A pragmatic version: use the scan's `length_delta` and `similarity_ratio` together to flag items where high `changed_total_lines` is paired with high `similarity_ratio` (genuine algorithmic divergence) vs. high `changed_total_lines` paired with very low `similarity_ratio` and large `length_delta` (possible representation/boilerplate re-expression). Items in the latter bucket should go to corpus cleanup before they are used to justify language design priorities.

A secondary fix: add a **minimum similarity-ratio floor** (e.g., < 0.15) as a required signal for Category 1 placement, so that a high-diff item with moderate similarity (0.25–0.35) is not auto-escalated without manual inspection. This would prevent `0295` (similarity 0.317, which is moderate) from being rescued solely on algorithmic grounds when the same pattern could equally reflect corpus divergence.
