from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 162: Find Peak Element
# Python version

def findPeakElement(nums: List[int]) -> int:
    l, r = 0, len(nums) - 1
    while l <= r:
        mid = (r + l) // 2
        if mid < len(nums) - 1 and nums[mid] < nums[mid+1]:
            l = mid + 1
        elif mid > 0 and nums[mid] < nums[mid-1]:
            r = mid - 1
        else:
            break
    return mid
    



def main():
    assert findPeakElement([1,2,3,1]) == 2

if __name__ == "__main__":
    main()
