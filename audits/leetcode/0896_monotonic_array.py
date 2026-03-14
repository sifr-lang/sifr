from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 896: Monotonic Array
# Python version

def isMonotonic(nums: List[int]) -> bool:
    increasing = decreasing = True
    
    for i in range(len(nums) - 1):
        if nums[i] > nums[i + 1]:
            increasing = False
        if nums[i] < nums[i + 1]:
            decreasing = False
    
    return increasing or decreasing



def main():
    assert isMonotonic([1,2,2,3]) == True
    assert isMonotonic([6,5,4,4]) == True
    assert isMonotonic([1,3,2]) == False

if __name__ == "__main__":
    main()
