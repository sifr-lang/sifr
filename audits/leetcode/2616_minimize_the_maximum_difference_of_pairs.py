from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2616: Minimize The Maximum Difference Of Pairs
# Python version

def minimizeMax(nums: List[int], p: int) -> int:
    nums.sort()

    def checkPair(mid):
        count, i = 0, 0

        while i < len(nums) - 1:
            if nums[i + 1] - nums[i] <= mid:
                count += 1
                i += 2
            else:
                i += 1

        return count >= p

    left, right = 0, nums[-1] - nums[0]

    while left < right:
        mid = (left + right) // 2

        if checkPair(mid):
            right = mid
        else:
            left = mid + 1

    return left



def main():
    assert minimizeMax([10,1,2,7,1,3], 2) == 1

if __name__ == "__main__":
    main()
