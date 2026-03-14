from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 442: Find All Duplicates In An Array
# Python version

def findDuplicates(nums: List[int]) -> List[int]:
    res = []

    for n in nums:
        n = abs(n)
        if nums[n - 1] < 0:
            res.append(n)
        nums[n - 1] = -nums[n - 1]
    
    return res



def main():
    assert findDuplicates([4, 3, 2, 7, 8, 2, 3, 1]) == [2, 3]
    assert findDuplicates([1, 1, 2]) == [1]
    assert findDuplicates([1]) == []

if __name__ == "__main__":
    main()
