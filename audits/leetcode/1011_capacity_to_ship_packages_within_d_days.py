from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1011: Capacity To Ship Packages Within D Days
# Python version

def shipWithinDays(weights: List[int], days: int) -> int:
    l, r = max(weights), sum(weights)
    min_cap = r

    def canShip(cap):
        ships, curCap = 1, cap
        for w in weights:
            if curCap - w < 0:
                ships += 1
                curCap = cap
            curCap -= w
        return ships <= days

    while l <= r:
        cap = (l + r) // 2
        if canShip(cap):
            min_cap = min(min_cap, cap)
            r = cap - 1
        else:
            l = cap + 1

    return min_cap






def main():
    assert shipWithinDays([1,2,3,4,5,6,7,8,9,10], 5) == 15

if __name__ == "__main__":
    main()
