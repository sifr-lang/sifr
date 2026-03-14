from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 77: Combinations
# Python version

def combine(n: int, k: int) -> List[List[int]]:
    res = []
    def helper(start, comb):
        if len(comb) == k:
            res.append(comb.copy())
            return
        for i in range(start, n+1):
            comb.append(i)
            helper(i+1, comb)
            comb.pop()
    helper(1, [])
    return res



def main():
    assert combine(4, 2) == [[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]

if __name__ == "__main__":
    main()
