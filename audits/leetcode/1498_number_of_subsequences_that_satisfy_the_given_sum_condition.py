from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1498: Number Of Subsequences That Satisfy The Given Sum Condition
# Python version

def numSubseq(nums: List[int], target: int) -> int:
    nums.sort()

    res, mod = 0, (10**9 + 7)

    left, right = 0, len(nums) - 1
    while  left <= right:
        if (nums[left] + nums[right]) > target:
            right -= 1
        else:
            res += 1 << (right - left)
            left += 1
    return res % mod



def main():
    assert numSubseq([3, 5, 6, 7], 9) == 4
    assert numSubseq([3, 3, 6, 8], 10) == 6
    assert numSubseq([2, 3, 3, 4, 6, 7], 12) == 61

if __name__ == "__main__":
    main()
