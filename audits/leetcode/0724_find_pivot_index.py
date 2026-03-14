from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 724: Find Pivot Index
# Python version

def pivotIndex(nums: List[int]) -> int:
    total = sum(nums)  # O(n)

    leftSum = 0
    for i in range(len(nums)):
        rightSum = total - nums[i] - leftSum
        if leftSum == rightSum:
            return i
        leftSum += nums[i]
    return -1



def main():
    assert pivotIndex([1,7,3,6,5,6]) == 3
    assert pivotIndex([1,2,3]) == -1

if __name__ == "__main__":
    main()
