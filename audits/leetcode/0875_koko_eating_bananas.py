from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 875: Koko Eating Bananas
# Python version

def minEatingSpeed(piles: List[int], h: int) -> int:
    l, r = 1, max(piles)
    res = r

    while l <= r:
        k = (l + r) // 2

        totalTime = 0
        for p in piles:
            totalTime += math.ceil(float(p) / k)
        if totalTime <= h:
            res = k
            r = k - 1
        else:
            l = k + 1
    return res



def main():
    assert minEatingSpeed([3,6,7,11], 8) == 4
    assert minEatingSpeed([30,11,23,4,20], 5) == 30

if __name__ == "__main__":
    main()
