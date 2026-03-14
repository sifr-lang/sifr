from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 115: Distinct Subsequences
# Python version

def numDistinct(s: str, t: str) -> int:
    cache = {}

    for i in range(len(s) + 1):
        cache[(i, len(t))] = 1
    for j in range(len(t)):
        cache[(len(s), j)] = 0

    for i in range(len(s) - 1, -1, -1):
        for j in range(len(t) - 1, -1, -1):
            if s[i] == t[j]:
                cache[(i, j)] = cache[(i + 1, j + 1)] + cache[(i + 1, j)]
            else:
                cache[(i, j)] = cache[(i + 1, j)]
    return cache[(0, 0)]



def main():
    assert numDistinct("rabbbit", "rabbit") == 3
    assert numDistinct("babgbag", "bag") == 5

if __name__ == "__main__":
    main()
