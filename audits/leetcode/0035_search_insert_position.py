from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 35: Search Insert Position
# Python version

def searchInsert(nums: List[int], target: int) -> int:
    # O(log n) and O(1)
    
    
    low, high = 0, len(nums)
    while low<high:
        mid = low +(high - low) // 2
        if target > nums[mid]:
            low = mid + 1
        else:
            high = mid
    return low



def main():
    assert searchInsert([1,3,5,6], 5) == 2
    assert searchInsert([1,3,5,6], 2) == 1
    assert searchInsert([1,3,5,6], 7) == 4

if __name__ == "__main__":
    main()
