from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1984: Minimum Difference Between Highest And Lowest Of K Scores
# Python version

def minimumDifference(nums: List[int], k: int) -> int:
    nums.sort()
    l, r = 0, k - 1
    res = float("inf")
    
    while r < len(nums):
        res = min(res, nums[r] - nums[l])
        l, r = l + 1, r + 1
    return res



def main():
    assert minimumDifference([90], 1) == 0

if __name__ == "__main__":
    main()
