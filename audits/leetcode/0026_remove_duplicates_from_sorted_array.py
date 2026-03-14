from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 26: Remove Duplicates From Sorted Array
# Python version

def removeDuplicates(nums: List[int]) -> int:
    L = 1
    
    for R in range(1, len(nums)):
        if nums[R] != nums[R - 1]:
            nums[L] = nums[R]
            L += 1
    return L



def main():
    assert removeDuplicates([1,1,2]) == 2
    assert removeDuplicates([0,0,1,1,1,2,2,3,3,4]) == 5

if __name__ == "__main__":
    main()
