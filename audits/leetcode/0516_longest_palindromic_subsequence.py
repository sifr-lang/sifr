from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 516: Longest Palindromic Subsequence
# Python version

def longestPalindromeSubseq(s: str) -> int:   
    # Dynamic Programming
    dp = [ [0] * (len(s) + 1) for i in range(len(s) + 1)]
    res = 0
    
    for i in range(len(s)):
        for j in range(len(s) - 1, i - 1, -1):
            if s[i] == s[j]:
                dp[i][j] = 1 if i == j else 2
                if i - 1 >= 0:
                    dp[i][j] += dp[i - 1][j + 1]
            else:
                dp[i][j] = dp[i][j + 1]
                if i - 1 >= 0:
                    dp[i][j] = max(dp[i][j], dp[i - 1][j])
            res = max(res, dp[i][j])
    return res


    # Memoization
    cache = {}

    def dfs(i, j):
        if i < 0 or j == len(s):
            return 0
        if (i, j) in cache:
            return cache[(i, j)]

        if s[i] == s[j]:
            length = 1 if i == j else 2
            cache[(i, j)] = length + dfs(i - 1, j + 1)
        else:
            cache[(i, j)] = max(dfs(i - 1, j), dfs(i, j + 1))
        return cache[(i, j)]
    
    for i in range(len(s)):
        dfs(i, i) # odd length
        dfs(i, i + 1) # even length

    return max(cache.values())
    
# LCS Solution

def longestPalindromeSubseq(s: str) -> int:
    return longestCommonSubsequence(s, s[::-1])
    


def longestCommonSubsequence(s1: str, s2: str) -> int:
    N, M = len(s1), len(s2)
    dp = [[0] * (M+1) for _ in range(N+1)]

    for i in range(N):
        for j in range(M):
            if s1[i] == s2[j]:
                dp[i+1][j+1] = 1 + dp[i][j]
            else:
                dp[i+1][j+1] = max(dp[i][j+1], dp[i+1][j])
    return dp[N][M]



def main():
    assert longestPalindromeSubseq("bbbab") == 4
    assert longestPalindromeSubseq("cbbd") == 2

if __name__ == "__main__":
    main()
