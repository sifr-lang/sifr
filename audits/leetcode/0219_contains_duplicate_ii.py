from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 219: Contains Duplicate Ii
# Python version

def containsNearbyDuplicate(nums: List[int], k: int) -> bool:
    window = set()
    L = 0

    for R in range(len(nums)):
        if R - L > k:
            window.remove(nums[L])
            L += 1
        if nums[R] in window:
            return True
        window.add(nums[R])
    return False



def main():
    assert containsNearbyDuplicate([1,2,3,1], 3) == True
    assert containsNearbyDuplicate([1,2,3,1,2,3], 2) == False

if __name__ == "__main__":
    main()
