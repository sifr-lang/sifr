from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 268: Missing Number
# Python version

def missingNumber(nums: List[int]) -> int:
    res = len(nums)

    for i in range(len(nums)):
        res += i - nums[i]
    return res



def main():
    assert missingNumber([3,0,1]) == 2
    assert missingNumber([0,1]) == 2

if __name__ == "__main__":
    main()
