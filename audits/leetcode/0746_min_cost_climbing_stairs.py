from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 746: Min Cost Climbing Stairs
# Python version

def minCostClimbingStairs(cost: List[int]) -> int:
    for i in range(len(cost) - 3, -1, -1):
        cost[i] += min(cost[i + 1], cost[i + 2])

    return min(cost[0], cost[1])



def main():
    assert minCostClimbingStairs([10,15,20]) == 15

if __name__ == "__main__":
    main()
