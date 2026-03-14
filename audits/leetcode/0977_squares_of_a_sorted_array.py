from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 977: Squares Of A Sorted Array
# Python version

def sortedSquares(nums: List[int]) -> List[int]:
    n = len(nums)
    res = [0] * n
    l, r = 0, n - 1
    
    while l <= r:
        left, right = abs(nums[l]), abs(nums[r])
        if left > right:
            res[r - l] = left * left
            l += 1
        else:
            res[r - l] = right * right
            r -= 1
    return res



def main():
    assert sortedSquares([-4,-1,0,3,10]) == [0, 1, 9, 16, 100]

if __name__ == "__main__":
    main()
