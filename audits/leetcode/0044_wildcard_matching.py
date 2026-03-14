from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 44: Wildcard Matching
# Python version

def isMatch(s, p):
    dp = [[False] * (len(p) + 1) for _ in range(len(s) + 1)]
    dp[-1][-1] = True
    for i in range(len(s), -1, -1):
        for j in range(len(p) - 1, -1, -1):
            if p[j] == '*':
                dp[i][j] = dp[i][j + 1] or (i < len(s) and dp[i + 1][j])
            else:
                dp[i][j] = i < len(s) and (p[j] == s[i] or p[j] == '?') and dp[i + 1][j + 1]
    return dp[0][0]



def main():
    assert isMatch("aa", "a") == False
    assert isMatch("aa", "*") == True
    assert isMatch("cb", "?a") == False

if __name__ == "__main__":
    main()
