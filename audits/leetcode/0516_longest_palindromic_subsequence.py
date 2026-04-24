
# LeetCode 516: Longest Palindromic Subsequence
# Python version

def longestPalindromeSubseq(s: str) -> int:
    t = s[::-1]
    memo = {}

    def lcs(i: int, j: int) -> int:
        if i == len(s) or j == len(t):
            return 0

        cached = memo.get((i, j), -1)
        if cached != -1:
            return cached

        if s[i] == t[j]:
            ans = 1 + lcs(i + 1, j + 1)
        else:
            ans = max(lcs(i + 1, j), lcs(i, j + 1))

        memo[(i, j)] = ans
        return ans

    return lcs(0, 0)


def main():
    assert longestPalindromeSubseq("bbbab") == 4
    assert longestPalindromeSubseq("cbbd") == 2

if __name__ == "__main__":
    main()
