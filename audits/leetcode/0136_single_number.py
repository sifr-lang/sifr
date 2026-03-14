from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 136: Single Number
# Python version

def singleNumber(nums: List[int]) -> int:
    res = 0
    for n in nums:
        res = n ^ res
    return res



def main():
    assert singleNumber([2,2,1]) == 1
    assert singleNumber([4,1,2,1,2]) == 4

if __name__ == "__main__":
    main()
