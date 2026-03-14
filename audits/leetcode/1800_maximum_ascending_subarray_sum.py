from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1800: Maximum Ascending Subarray Sum
# Python version

def maxAscendingSum(nums: List[int]) -> int:
    curSum = results = nums[0]

    for i in range(1, len(nums)):
        if nums[i] <= nums[i - 1]:
            curSum = 0
        curSum += nums[i]
        results = max(curSum, results)

    return results



def main():
    assert maxAscendingSum([10,20,30,5,10,50]) == 65

if __name__ == "__main__":
    main()
