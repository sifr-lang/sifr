from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 53: Maximum Subarray (Kadane's Algorithm)
# Python version with test cases

def maxSubArray(nums: list[int]) -> int:
    max_sum: int = nums[0]
    current_sum: int = nums[0]
    for i in range(1, len(nums)):
        if current_sum < 0:
            current_sum = nums[i]
        else:
            current_sum = current_sum + nums[i]
        if current_sum > max_sum:
            max_sum = current_sum
    return max_sum

def main():
    assert maxSubArray([-2, 1, -3, 4, -1, 2, 1, -5, 4]) == 6
    assert maxSubArray([1]) == 1
    assert maxSubArray([5, 4, -1, 7, 8]) == 23
    assert maxSubArray([-1]) == -1
    assert maxSubArray([-2, -1]) == -1

if __name__ == "__main__":
    main()
