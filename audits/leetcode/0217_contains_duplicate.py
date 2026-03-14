from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 217: Contains Duplicate
# Python version

def containsDuplicate(nums: List[int]) -> bool:
    hashset = set()

    for n in nums:
        if n in hashset:
            return True
        hashset.add(n)
    return False



def main():
    assert containsDuplicate([1,2,3,1]) == True
    assert containsDuplicate([1,2,3,4]) == False

if __name__ == "__main__":
    main()
