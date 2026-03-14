from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1: Two Sum
# Python version

def twoSum(nums: List[int], target: int) -> List[int]:
    prevMap = {}  # val -> index

    for i, n in enumerate(nums):
        diff = target - n
        if diff in prevMap:
            return [prevMap[diff], i]
        prevMap[n] = i



def main():
    assert twoSum([2,7,11,15], 9) == [0, 1]
    assert twoSum([3,2,4], 6) == [1, 2]
    assert twoSum([3,3], 6) == [0, 1]

if __name__ == "__main__":
    main()
