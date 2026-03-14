from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 72: Edit Distance
# Python version

def minDistance(word1: str, word2: str) -> int:
    dp = [[float("inf")] * (len(word2) + 1) for i in range(len(word1) + 1)]

    for j in range(len(word2) + 1):
        dp[len(word1)][j] = len(word2) - j
    for i in range(len(word1) + 1):
        dp[i][len(word2)] = len(word1) - i

    for i in range(len(word1) - 1, -1, -1):
        for j in range(len(word2) - 1, -1, -1):
            if word1[i] == word2[j]:
                dp[i][j] = dp[i + 1][j + 1]
            else:
                dp[i][j] = 1 + min(dp[i + 1][j], dp[i][j + 1], dp[i + 1][j + 1])
    return dp[0][0]



def main():
    assert minDistance("horse", "ros") == 3
    assert minDistance("intention", "execution") == 5

if __name__ == "__main__":
    main()
