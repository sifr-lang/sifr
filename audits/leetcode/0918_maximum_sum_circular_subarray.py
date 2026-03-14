from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 918: Maximum Sum Circular Subarray
# Python version

def maxSubarraySumCircular(nums: List[int]) -> int:
    globMax, globMin = nums[0], nums[0]
    curMax, curMin = 0, 0
    total = 0
    
    for i, n in enumerate(nums):
        curMax = max(curMax + n, n)
        curMin = min(curMin + n, n)
        total += n
        globMax = max(curMax, globMax)
        globMin = min(curMin, globMin)

    return max(globMax, total - globMin) if globMax > 0 else globMax



def main():
    assert maxSubarraySumCircular([1,-2,3,-2]) == 3
    assert maxSubarraySumCircular([5,-3,5]) == 10
    assert maxSubarraySumCircular([-3,-2,-3]) == -2

if __name__ == "__main__":
    main()
