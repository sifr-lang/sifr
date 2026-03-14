from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 27: Remove Element
# Python version

def removeElement(nums: List[int], val: int) -> int:
    k = 0
    for i in range(len(nums)):
        if nums[i] != val:
            nums[k] = nums[i]
            k += 1
    return k

# Optimized solution with the same time and space complexity

def removeElement(nums: List[int], val: int) -> int:
    # Avoid unessary copy operations in a previous solution, when k == i and nums[i] != val 
    # by swapping nums[i] and the last element of the array (nums[n])
    n = len(nums)
    i = 0

    while i < n:
        if nums[i] == val:
            nums[i], nums[n - 1] = nums[n - 1], nums[i]
            n -= 1  # decrement the length of the array by discarding the last element
        else:
            i += 1
    
    return n



def main():
    assert removeElement([3,2,2,3], 3) == 2
    assert removeElement([0,1,2,2,3,0,4,2], 2) == 5

if __name__ == "__main__":
    main()
