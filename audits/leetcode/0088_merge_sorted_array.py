from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 88: Merge Sorted Array
# Python version

def merge(nums1: List[int], m: int, nums2: List[int], n: int) -> None:
    """
    Do not return anything, modify nums1 in-place instead.
    """
    while m > 0 and n > 0:
        if nums1[m-1] >= nums2[n-1]:
            nums1[m+n-1] = nums1[m-1]
            m -= 1
        else:
            nums1[m+n-1] = nums2[n-1]
            n -= 1
    if n > 0:
        nums1[:n] = nums2[:n]


def main():
    arg0 = [1, 2, 3, 0, 0, 0]
    arg1 = 3
    arg2 = [2, 5, 6]
    arg3 = 3
    _result = merge(arg0, arg1, arg2, arg3)
    assert arg0 == [1, 2, 2, 3, 5, 6]
    arg0 = [1]
    arg1 = 1
    arg2 = []
    arg3 = 0
    _result = merge(arg0, arg1, arg2, arg3)
    assert arg0 == [1]
    arg0 = [0]
    arg1 = 0
    arg2 = [1]
    arg3 = 1
    _result = merge(arg0, arg1, arg2, arg3)
    assert arg0 == [1]

if __name__ == "__main__":
    main()
