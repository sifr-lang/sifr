from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 75: Sort Colors
# Python version

def sortColors(nums: List[int]) -> None:
    low = 0
    high = len(nums) - 1
    mid = 0

    while mid <= high:
        if nums[mid] == 0:
            nums[low], nums[mid] = nums[mid], nums[low]
            low += 1
            mid += 1
        elif nums[mid] == 1:
            mid +=1
        else:
            nums[mid], nums[high] = nums[high], nums[mid]
            high -= 1
    return nums



def main():
    arg0 = [2, 0, 2, 1, 1, 0]
    _result = sortColors(arg0)
    assert arg0 == [0, 0, 1, 1, 2, 2]
    arg0 = [2, 0, 1]
    _result = sortColors(arg0)
    assert arg0 == [0, 1, 2]

if __name__ == "__main__":
    main()
